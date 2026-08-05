//! Tab bar: mostra documentos abertos como tabs.

use yew::prelude::*;
use crate::api::PageMeta;

#[derive(Properties, PartialEq, Clone)]
pub struct TabBarProps {
    pub tabs: Vec<PageMeta>,
    pub active_path: Option<String>,
    pub on_select: Callback<PageMeta>,
    pub on_close: Callback<usize>,
}

#[function_component(TabBar)]
pub fn tab_bar(props: &TabBarProps) -> Html {
    if props.tabs.is_empty() {
        return html! {};
    }

    html! {
        <div class="tab-bar">
            { for props.tabs.iter().enumerate().map(|(i, tab)| {
                let is_active = props.active_path.as_deref() == Some(tab.path.as_str());
                let class = if is_active { "tab-bar__tab tab-bar__tab--active" } else { "tab-bar__tab" };
                let on_select = props.on_select.clone();
                let meta = tab.clone();
                let on_close = props.on_close.clone();
                html! {
                    <div {class}>
                        <span class="tab-bar__tab-title"
                            onclick={Callback::from(move |_| on_select.emit(meta.clone()))}>
                            { &tab.title }
                        </span>
                        <button class="tab-bar__tab-close"
                            onclick={Callback::from(move |_| on_close.emit(i))}>
                            { "×" }
                        </button>
                    </div>
                }
            }) }
        </div>
    }
}
