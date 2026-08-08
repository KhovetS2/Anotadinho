//! Renderiza o modal certo pro `PendingDialog` atual (ver `crate::dialog`).

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::components::modal::Modal;
use crate::dialog::PendingDialog;

/// Props do `DialogHost`.
#[derive(Properties, PartialEq, Clone)]
pub struct DialogHostProps {
    /// Diálogo pendente (ou `None` se nada aberto).
    pub pending: Option<PendingDialog>,
    /// Disparado quando o modal deve fechar (cancelar/✕/clique fora).
    pub on_dismiss: Callback<()>,
}

/// Host do modal de diálogo — um só por app, montado no topo da árvore.
#[function_component(DialogHost)]
pub fn dialog_host(props: &DialogHostProps) -> Html {
    let input_value = use_state(String::new);

    {
        let input_value = input_value.clone();
        use_effect_with(props.pending.clone(), move |pending| {
            if let Some(PendingDialog::Prompt { default, .. }) = pending {
                input_value.set(default.clone());
            }
            || ()
        });
    }

    let Some(dialog) = props.pending.clone() else {
        return html! {};
    };
    let on_dismiss = props.on_dismiss.clone();

    let (title, body) = match dialog {
        PendingDialog::Alert { message } => {
            let dismiss = on_dismiss.clone();
            (
                "Aviso".to_string(),
                html! {
                    <>
                        <p class="modal__message">{ message }</p>
                        <div class="modal__actions">
                            <button class="btn btn--primary btn--sm" onclick={dismiss.reform(|_| ())}>{ "OK" }</button>
                        </div>
                    </>
                },
            )
        }
        PendingDialog::Confirm { message, confirm_label, on_confirm } => {
            let dismiss = on_dismiss.clone();
            let confirm = {
                let on_dismiss = on_dismiss.clone();
                Callback::from(move |_: MouseEvent| {
                    // Fecha ANTES de chamar on_confirm: se on_confirm abrir um
                    // outro diálogo (fluxo encadeado), esse novo diálogo tem
                    // que "vencer" — se a ordem fosse invertida, o dismiss
                    // rodaria depois e apagaria o diálogo novo.
                    on_dismiss.emit(());
                    on_confirm.emit(());
                })
            };
            (
                "Confirmar".to_string(),
                html! {
                    <>
                        <p class="modal__message">{ message }</p>
                        <div class="modal__actions">
                            <button class="btn btn--ghost btn--sm" onclick={dismiss.reform(|_| ())}>{ "Cancelar" }</button>
                            <button class="btn btn--danger btn--sm" onclick={confirm}>{ confirm_label }</button>
                        </div>
                    </>
                },
            )
        }
        PendingDialog::Prompt { title, on_submit, .. } => {
            let dismiss = on_dismiss.clone();
            let value = (*input_value).clone();
            let oninput = {
                let input_value = input_value.clone();
                Callback::from(move |e: InputEvent| {
                    if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                        input_value.set(input.value());
                    }
                })
            };
            let submit: Callback<()> = {
                let on_dismiss = on_dismiss.clone();
                let input_value = input_value.clone();
                Callback::from(move |_: ()| {
                    let v = (*input_value).trim().to_string();
                    if v.is_empty() { return; }
                    // Mesma ordem do Confirm: fecha antes de chamar
                    // on_submit, pra um diálogo encadeado (ex: nome → tipo →
                    // opções da coluna da tabela) não ser fechado por engano
                    // pelo dismiss que "deveria" ser deste diálogo.
                    on_dismiss.emit(());
                    on_submit.emit(v);
                })
            };
            let onkeydown = {
                let submit = submit.clone();
                Callback::from(move |e: KeyboardEvent| {
                    if e.key() == "Enter" { submit.emit(()); }
                })
            };
            (
                title,
                html! {
                    <>
                        <input class="input" type="text" {value} {oninput} {onkeydown} autofocus={true} />
                        <div class="modal__actions">
                            <button class="btn btn--ghost btn--sm" onclick={dismiss.reform(|_| ())}>{ "Cancelar" }</button>
                            <button class="btn btn--primary btn--sm" onclick={submit.reform(|_: MouseEvent| ())}>{ "OK" }</button>
                        </div>
                    </>
                },
            )
        }
        PendingDialog::Select { title, options, on_select } => {
            let dismiss = on_dismiss.clone();
            (
                title,
                html! {
                    <>
                        <ul class="modal__select-list">
                            { for options.iter().map(|(value, label)| {
                                let onclick = {
                                    let on_dismiss = on_dismiss.clone();
                                    let on_select = on_select.clone();
                                    let value = value.clone();
                                    Callback::from(move |_: MouseEvent| {
                                        // Mesma ordem do Confirm/Prompt: fecha antes
                                        // de emitir, pra suportar diálogo encadeado.
                                        on_dismiss.emit(());
                                        on_select.emit(value.clone());
                                    })
                                };
                                html! {
                                    <li>
                                        <button class="modal__select-item" {onclick}>{ label }</button>
                                    </li>
                                }
                            }) }
                        </ul>
                        <div class="modal__actions">
                            <button class="btn btn--ghost btn--sm" onclick={dismiss.reform(|_| ())}>{ "Cancelar" }</button>
                        </div>
                    </>
                },
            )
        }
    };

    html! {
        <Modal {title} open={true} on_close={on_dismiss}>
            { body }
        </Modal>
    }
}
