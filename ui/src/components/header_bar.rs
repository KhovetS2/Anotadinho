//! Header bar.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderBarProps {
    pub vault_name: Option<String>,
    pub vault_path: Option<String>,
    pub sidebar_collapsed: bool,
    pub theme_light: bool,
    pub on_toggle_sidebar: Callback<()>,
    pub on_toggle_theme: Callback<()>,
    pub on_close_vault: Callback<()>,
    pub on_open_vault: Callback<()>,
}

#[function_component(HeaderBar)]
pub fn header_bar(props: &HeaderBarProps) -> Html {
    let menu_open = use_state(|| false);
    let toggle_menu = { let m = menu_open.clone(); Callback::from(move |_| m.set(!*m)) };

    html! {
        <header class="header-bar">
            <div class="header-bar__left">
                <button class="btn btn--ghost btn--xs" onclick={props.on_toggle_sidebar.reform(|_| ())}>
                    { if props.sidebar_collapsed { "▶" } else { "◀" } }
                </button>
                <span class="header-bar__title">{ "Anotadinho" }</span>
                if let Some(ref name) = props.vault_name {
                    <span class="header-bar__vault">{ name }</span>
                }
            </div>
            <div class="header-bar__right">
                <button class="btn btn--ghost btn--xs" onclick={props.on_toggle_theme.reform(|_| ())}>
                    { if props.theme_light { "☀" } else { "🌙" } }
                </button>
                <div class="header-menu-wrapper">
                    <button class="btn btn--ghost btn--xs" onclick={toggle_menu}>{ "⚙" }</button>
                    if *menu_open {
                        <div class="header-menu">
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={props.on_open_vault.reform(|_| ())}>
                                {"Abrir vault"}
                            </button>
                            if props.vault_path.is_some() {
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={props.on_close_vault.reform(|_| ())}>
                                    {"Fechar vault"}
                                </button>
                            }
                            <div class="divider"></div>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={props.on_toggle_theme.reform(|_| ())}>
                                { if props.theme_light { "🌙 Tema escuro" } else { "☀ Tema claro" } }
                            </button>
                        </div>
                    }
                </div>
            </div>
        </header>
    }
}
