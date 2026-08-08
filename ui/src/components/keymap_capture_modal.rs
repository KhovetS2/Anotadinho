//! Modal genérico de captura de atalhos — extraído do `VimSettingsModal`
//! (ciclo 092) pra ser reusado por QUALQUER keymap (vim mode, ciclo
//! 105 `GlobalKeymap`, etc). Não sabe nada sobre o significado das
//! ações — só recebe rótulo + tecla atual por campo, e devolve
//! `(rótulo, tecla nova)` quando o usuário reatribui.

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::components::modal::Modal;

/// Props do `KeymapCaptureModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct KeymapCaptureModalProps {
    /// Título do modal.
    pub title: String,
    /// Texto de ajuda acima da lista de atalhos — vazio omite o parágrafo.
    #[prop_or_default]
    pub hint: String,
    /// Campos `(rótulo, tecla atual)`, na ordem exibida.
    pub fields: Vec<(&'static str, String)>,
    /// Disparado com `(rótulo, tecla nova)` quando o usuário reatribui
    /// uma tecla — quem chama decide como aplicar isso ao keymap.
    pub on_change: Callback<(String, String)>,
    /// Fecha o modal.
    pub on_close: Callback<()>,
}

#[function_component(KeymapCaptureModal)]
pub fn keymap_capture_modal(props: &KeymapCaptureModalProps) -> Html {
    let capturing: UseStateHandle<Option<&'static str>> = use_state(|| None);

    html! {
        <Modal title={props.title.clone()} open={true} on_close={props.on_close.clone()}>
            if !props.hint.is_empty() {
                <p class="vim-settings__hint">{ &props.hint }</p>
            }
            <div class="vim-settings">
                { for props.fields.iter().map(|(label, key)| {
                    let label = *label;
                    let is_capturing = *capturing == Some(label);
                    let capturing_set = capturing.clone();
                    let on_change = props.on_change.clone();
                    let onclick = Callback::from(move |_| capturing_set.set(Some(label)));
                    let capturing_kd = capturing.clone();
                    let onkeydown = Callback::from(move |e: KeyboardEvent| {
                        e.prevent_default();
                        e.stop_propagation();
                        let new_key = e.key();
                        capturing_kd.set(None);
                        if new_key == "Escape" { return; }
                        on_change.emit((label.to_string(), new_key));
                    });
                    html! {
                        <div class="vim-settings__row">
                            <span class="vim-settings__label">{ label }</span>
                            if is_capturing {
                                <input class="vim-settings__key-input" autofocus=true
                                    placeholder="pressione uma tecla..." {onkeydown} value="" />
                            } else {
                                <button class="vim-settings__key-btn" {onclick}>{ key }</button>
                            }
                        </div>
                    }
                }) }
            </div>
        </Modal>
    }
}
