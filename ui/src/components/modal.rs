//! Modal dialog component.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct ModalProps {
    pub title: String,
    pub open: bool,
    pub on_close: Callback<()>,
    #[prop_or_default]
    pub children: Children,
}

#[function_component(Modal)]
pub fn modal(props: &ModalProps) -> Html {
    if !props.open {
        return html! {};
    }
    let close = props.on_close.clone();
    html! {
        <div class="modal-overlay" onclick={close.reform(|_| ())}>
            <div class="modal">
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
