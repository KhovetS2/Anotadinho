//! Tab bar: mostra documentos abertos como tabs.

use yew::prelude::*;
use crate::api::PageMeta;

#[derive(Properties, PartialEq, Clone)]
pub struct TabBarProps {
    pub tabs: Vec<PageMeta>,
    pub active_path: Option<String>,
    pub on_select: Callback<PageMeta>,
    pub on_close: Callback<usize>,
    /// Path da página inicial (ciclo 089) — a aba dessa página mostra só
    /// o ícone 🏠 fixo em vez do título (ciclo 109), com o nome de
    /// verdade só no tooltip.
    #[prop_or_default]
    pub home_path: Option<String>,
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
                let is_home = props.home_path.as_deref() == Some(tab.path.as_str());
                let class = if is_active { "tab-bar__tab tab-bar__tab--active" } else { "tab-bar__tab" };
                let on_select = props.on_select.clone();
                let meta = tab.clone();
                let on_close = props.on_close.clone();
                // Indicador do atalho fixo Ctrl+1..9 (ciclo 107) — só
                // as 9 primeiras abas têm um bind (mesma convenção de
                // navegador/editor de código).
                let shortcut_num = if i < 9 { Some(i + 1) } else { None };
                let title = match shortcut_num {
                    Some(n) => format!("{} (Ctrl+{})", tab.title, n),
                    None => tab.title.clone(),
                };
                html! {
                    <div {class}>
                        <span class="tab-bar__tab-title" {title}
                            onclick={Callback::from(move |_| on_select.emit(meta.clone()))}>
                            if let Some(n) = shortcut_num {
                                <span class="tab-bar__tab-num">{ n }</span>
                            }
                            if is_home { { "🏠" } } else { { &tab.title } }
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
