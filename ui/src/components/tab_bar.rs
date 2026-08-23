//! Tab bar: mostra documentos abertos como tabs.

use yew::prelude::*;
use crate::api::PageMeta;
use crate::components::icon::Icon;

/// Características independentes que uma aba pode acumular. O modelo é
/// genérico de propósito, embora o ciclo 220 só precise da flag da home.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TabFlag {
    Home,
}

#[derive(Debug, Clone, PartialEq)]
pub struct OpenTab {
    pub page: PageMeta,
    pub flags: Vec<TabFlag>,
}

impl OpenTab {
    pub fn new(page: PageMeta) -> Self {
        Self { page, flags: Vec::new() }
    }

    pub fn has_flag(&self, flag: TabFlag) -> bool {
        self.flags.contains(&flag)
    }
}

#[derive(Properties, PartialEq, Clone)]
pub struct TabBarProps {
    pub tabs: Vec<OpenTab>,
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
        <div class="tab-bar" tabindex="0" data-nav-item="tabbar" data-nav-parent="root" data-nav-group="tabbar">
            { for props.tabs.iter().enumerate().map(|(i, tab)| {
                let page = &tab.page;
                let is_active = props.active_path.as_deref() == Some(page.path.as_str());
                let is_home = tab.has_flag(TabFlag::Home);
                let class = classes!("tab-bar__tab", is_active.then_some("tab-bar__tab--active"), is_home.then_some("tab-bar__tab--fixed"));
                let on_select = props.on_select.clone();
                let meta = page.clone();
                let on_close = props.on_close.clone();
                // Indicador do atalho fixo Ctrl+1..9 (ciclo 107) — só
                // as 9 primeiras abas têm um bind (mesma convenção de
                // navegador/editor de código).
                let shortcut_num = if i < 9 { Some(i + 1) } else { None };
                let title = match shortcut_num {
                    Some(n) => format!("{} (Ctrl+{})", page.title, n),
                    None => page.title.clone(),
                };
                // Ciclo 133: a tab-bar nunca teve NENHUM suporte de
                // teclado (nem tabindex) — reaproveita o mesmo helper
                // já usado por kanban/calendário/tabela/tags (ciclo
                // 127) pra Enter/Espaço equivalerem ao clique.
                let activate = {
                    let on_select = on_select.clone();
                    let meta = meta.clone();
                    Callback::from(move |_: ()| on_select.emit(meta.clone()))
                };
                let onclick = {
                    let activate = activate.clone();
                    Callback::from(move |_: MouseEvent| activate.emit(()))
                };
                let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                let nav_item_id = format!("tab-{}", i);
                html! {
                    <div {class} data-path={page.path.clone()}>
                        <span class="tab-bar__tab-title" {title} tabindex="0"
                            data-nav-item={nav_item_id} data-nav-parent="tabbar"
                            {onclick} {onkeydown}>
                            if let Some(n) = shortcut_num {
                                <span class="tab-bar__tab-num">{ n }</span>
                            }
                            if is_home {
                                <Icon name="home" />
                            } else {
                                { &page.title }
                            }
                        </span>
                        if !is_home {
                            <button class="tab-bar__tab-close"
                                onclick={Callback::from(move |_| on_close.emit(i))}>
                                { "×" }
                            </button>
                        }
                    </div>
                }
            }) }
        </div>
    }
}
