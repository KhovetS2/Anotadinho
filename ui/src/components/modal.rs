//! Modal dialog component.

use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::icon::Icon;

#[derive(Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub title: String,
    pub open: bool,
    pub on_close: Callback<()>,
    /// Modais de formulário maior (ex: detalhes do card, com abas) usam
    /// isso pra não ficar espremido nos 420px do diálogo simples.
    #[prop_or_default]
    pub wide: bool,
    /// Incrementado por quem monta o `Modal` toda vez que o CONTEÚDO
    /// troca sem o modal em si fechar/reabrir — ex: `DialogHost`
    /// encadeando Select → Prompt (escolher template → digitar título)
    /// mantém `open=true` o tempo todo, então o auto-foco (efeito
    /// abaixo) precisa de outro gatilho pra saber que tem um novo
    /// primeiro elemento focável pra focar (ciclo 129, achado durante
    /// a auditoria final: sem isso, o segundo diálogo de uma cadeia
    /// nascia sem foco nenhum, quebrando o que o ciclo 124 prometia).
    #[prop_or_default]
    pub focus_nonce: u32,
    #[prop_or_default]
    pub children: Children,
}

/// Elementos considerados "focáveis" pro auto-foco e pro trap de Tab —
/// mesmo critério usado em outros lugares do app que já lidam com foco
/// dinamicamente (ex: `insert_element_at_cursor` no editor).
const FOCUSABLE_SELECTOR: &str =
    "button:not([disabled]), [href], input:not([type=hidden]):not([disabled]), select:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex='-1'])";

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    let modal_ref = use_node_ref();
    let body_ref = use_node_ref();

    // Foco automático (ciclo 124): sem isso, abrir um modal via atalho
    // de teclado (ex: Ctrl+N) deixa o foco onde estava antes — pra
    // navegar as opções por teclado, o usuário precisaria saber que
    // tem que dar Tab às cegas até alcançar o modal. Foca o primeiro
    // elemento focável do CORPO (não o botão "✕" do cabeçalho — esse
    // continua alcançável normalmente via Tab/Shift+Tab, só não é o
    // alvo do auto-foco).
    {
        let body_ref = body_ref.clone();
        let open = props.open;
        use_effect_with((open, props.focus_nonce), move |(open, _)| {
            if *open {
                if let Some(body) = body_ref.cast::<web_sys::Element>() {
                    if let Ok(Some(first)) = body.query_selector(FOCUSABLE_SELECTOR) {
                        if let Ok(el) = first.dyn_into::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    }
                }
            }
            || ()
        });
    }

    if !props.open {
        return html! {};
    }
    let close = props.on_close.clone();
    // stop_propagation no conteúdo é essencial: sem isso, qualquer clique
    // dentro do modal (ex: no botão "OK") borbulha até o onclick do overlay
    // e dispara um close() extra DEPOIS do clique original — o que
    // sobrescreve um diálogo encadeado que o próprio botão acabou de abrir
    // (ex: nome → tipo → opções na configuração de coluna da tabela).
    let stop_propagation = Callback::from(|e: MouseEvent| e.stop_propagation());
    let modal_class = if props.wide { "modal modal--wide" } else { "modal" };

    // Escape fecha; Tab/Shift+Tab ficam presos dentro do modal (trap
    // de foco, ciclo 124) — sem isso, Tab "escapa" pro resto da
    // página (sidebar, abas etc), que continua tecnicamente focável
    // por baixo do overlay.
    let on_keydown = {
        let modal_ref = modal_ref.clone();
        let close = close.clone();
        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "Escape" => {
                    e.prevent_default();
                    close.emit(());
                }
                "Tab" => {
                    let Some(container) = modal_ref.cast::<web_sys::Element>() else { return };
                    let Ok(list) = container.query_selector_all(FOCUSABLE_SELECTOR) else { return };
                    let len = list.length();
                    if len == 0 {
                        return;
                    }
                    let Some(active) = web_sys::window()
                        .and_then(|w| w.document())
                        .and_then(|d| d.active_element())
                    else {
                        return;
                    };
                    let first = list.item(0);
                    let last = list.item(len - 1);
                    if e.shift_key() {
                        if active.is_same_node(first.as_ref()) {
                            e.prevent_default();
                            if let Some(el) = last.and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) {
                                let _ = el.focus();
                            }
                        }
                    } else if active.is_same_node(last.as_ref()) {
                        e.prevent_default();
                        if let Some(el) = first.and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) {
                            let _ = el.focus();
                        }
                    }
                }
                _ => {}
            }
        })
    };

    html! {
        <div class="modal-overlay" onclick={close.reform(|_| ())}>
            <div class={modal_class} ref={modal_ref} onclick={stop_propagation} onkeydown={on_keydown}>
                <div class="modal__header">
                    <h3 class="modal__title">{ &props.title }</h3>
                    <button class="btn btn--ghost btn--xs" onclick={close.reform(|_| ())}>
                        <Icon name="x" />
                    </button>
                </div>
                <div class="modal__body" ref={body_ref}>
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
