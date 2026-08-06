//! Modal dialog component.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub title: String,
    pub open: bool,
    pub on_close: Callback<()>,
    /// Modais de formulário maior (ex: detalhes do card, com abas) usam
    /// isso pra não ficar espremido nos 420px do diálogo simples.
    #[prop_or_default]
    pub wide: bool,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
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
    html! {
        <div class="modal-overlay" onclick={close.reform(|_| ())}>
            <div class={modal_class} onclick={stop_propagation}>
                <div class="modal__header">
                    <h3 class="modal__title">{ &props.title }</h3>
                    <button class="btn btn--ghost btn--xs" onclick={close.reform(|_| ())}>
                        { "✕" }
                    </button>
                </div>
                <div class="modal__body">
                    { for props.children.iter() }
                </div>
            </div>
        </div>
    }
}
