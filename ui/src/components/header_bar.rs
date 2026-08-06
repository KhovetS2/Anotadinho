//! Header bar.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
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
    let menu_ref = use_node_ref();
    let toggle_menu = { let m = menu_open.clone(); Callback::from(move |_| m.set(!*m)) };

    // Fecha o menu ao clicar fora dele ou apertar Escape — sem isso ele só
    // fechava clicando de novo no botão ⚙ ou (por acidente) ao recarregar
    // a página.
    {
        let menu_open = menu_open.clone();
        let menu_ref = menu_ref.clone();
        use_effect_with(*menu_open, move |open| {
            let mut listeners = Vec::new();
            if *open {
                let window = web_sys::window().expect("no global window");

                let close_on_outside = {
                    let menu_open = menu_open.clone();
                    let menu_ref = menu_ref.clone();
                    EventListener::new(&window, "mousedown", move |e| {
                        let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                        if let Some(el) = menu_ref.cast::<web_sys::Element>() {
                            if !el.contains(Some(&target)) {
                                menu_open.set(false);
                            }
                        }
                    })
                };
                let close_on_escape = {
                    let menu_open = menu_open.clone();
                    EventListener::new(&window, "keydown", move |e| {
                        if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                            if e.key() == "Escape" {
                                menu_open.set(false);
                            }
                        }
                    })
                };
                listeners.push(close_on_outside);
                listeners.push(close_on_escape);
            }
            move || drop(listeners)
        });
    }

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
                <div class="header-menu-wrapper" ref={menu_ref}>
                    <button class="btn btn--ghost btn--xs" onclick={toggle_menu}>{ "⚙" }</button>
                    if *menu_open {
                        <div class="header-menu">
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_open_vault = props.on_open_vault.clone();
                                Callback::from(move |_| { menu_open.set(false); on_open_vault.emit(()); })
                            }}>
                                {"Abrir vault"}
                            </button>
                            if props.vault_path.is_some() {
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let menu_open = menu_open.clone();
                                    let on_close_vault = props.on_close_vault.clone();
                                    Callback::from(move |_| { menu_open.set(false); on_close_vault.emit(()); })
                                }}>
                                    {"Fechar vault"}
                                </button>
                            }
                            <div class="divider"></div>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_toggle_theme = props.on_toggle_theme.clone();
                                Callback::from(move |_| { menu_open.set(false); on_toggle_theme.emit(()); })
                            }}>
                                { if props.theme_light { "🌙 Tema escuro" } else { "☀ Tema claro" } }
                            </button>
                        </div>
                    }
                </div>
            </div>
        </header>
    }
}
