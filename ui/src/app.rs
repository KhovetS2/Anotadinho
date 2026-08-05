//! Componente raiz da aplicação.

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::api;
use crate::api::PageMeta;
use crate::components::editor::Editor;
use crate::components::empty_state::EmptyState;
use crate::components::sidebar::Sidebar;
use crate::state;

#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);
    let list_version = use_state(|| 0u32);
    let sidebar_collapsed = use_state(|| false);
    let theme_light = use_state(|| {
        web_sys::window()
            .and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("anotadinho.theme").ok().flatten())
            .map_or(false, |v| v == "light")
    });

    // Apply theme to <html>
    {
        let light = *theme_light;
        use_effect_with(light, move |_| {
            if let Some(html) = web_sys::window().and_then(|w| w.document()).and_then(|d| d.document_element()) {
                if light { html.class_list().add_1("theme-light").ok(); }
                else { html.class_list().remove_1("theme-light").ok(); }
            }
            || {}
        });
    }

    // Polling
    {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        use_effect_with(vault_path.clone(), move |_| {
            let mut interval: Option<gloo_timers::callback::Interval> = None;
            if let Some(ref p) = *vault_path {
                let path = p.clone();
                let iv = gloo_timers::callback::Interval::new(3000, move || {
                    let path = path.clone();
                    let list_version = list_version.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(true) = api::check_changes(&path).await {
                            list_version.set(*list_version + 1);
                        }
                    });
                });
                interval = Some(iv);
            }
            move || drop(interval.take())
        });
    }

    let on_vault_selected = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        Callback::from(move |path: String| {
            let name = state::extract_name_from_path(&path);
            state::save_vault_path(&path);
            state::save_vault_name(&name);
            vault_path.set(Some(path));
            vault_name.set(Some(name));
            selected_page.set(None);
        })
    };

    let on_page_selected = {
        let selected_page = selected_page.clone();
        Callback::from(move |page: PageMeta| selected_page.set(Some(page)))
    };

    let on_close_vault = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        Callback::from(move |_| {
            state::clear_vault();
            vault_path.set(None);
            vault_name.set(None);
            selected_page.set(None);
        })
    };

    let on_page_deleted = {
        let selected_page = selected_page.clone();
        let list_version = list_version.clone();
        Callback::from(move |_| {
            selected_page.set(None);
            list_version.set(*list_version + 1);
        })
    };

    let toggle_sidebar = {
        let collapsed = sidebar_collapsed.clone();
        Callback::from(move |_| collapsed.set(!*collapsed))
    };

    let toggle_theme = {
        let light = theme_light.clone();
        Callback::from(move |_| {
            let next = !*light;
            light.set(next);
            if let Some(s) = web_sys::window().and_then(|w| w.local_storage().ok().flatten()) {
                let _ = s.set_item("anotadinho.theme", if next { "light" } else { "dark" });
            }
        })
    };

    let onkeydown = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let selected_page = selected_page.clone();
        let on_page_selected = on_page_selected.clone();
        Callback::from(move |e: KeyboardEvent| {
            let ctrl = e.ctrl_key() || e.meta_key();
            match (ctrl, e.key().as_str()) {
                (true, "n") => {
                    e.prevent_default();
                    let title = gloo_dialogs::prompt("Título da nova página:", Some("Nova nota")).unwrap_or_default();
                    let title = title.trim().to_string();
                    if title.is_empty() { return; }
                    let vault = (*vault_path).clone().unwrap_or_default();
                    let list_version = list_version.clone();
                    let on_page_selected = on_page_selected.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(meta) = api::create_page(&vault, &title).await {
                            on_page_selected.emit(meta);
                            list_version.set(*list_version + 1);
                        }
                    });
                }
                (false, "Escape") => { if selected_page.is_some() { selected_page.set(None); } }
                _ => {}
            }
        })
    };

    html! {
        <div class="app-root" tabindex="0" {onkeydown}>
            if let (Some(path), Some(name)) = ((*vault_path).clone(), (*vault_name).clone()) {
                <div class="app-layout">
                    <header class="app-header">
                        <button class="btn btn--ghost btn--xs" onclick={toggle_sidebar}>
                            { if *sidebar_collapsed { "▶" } else { "◀" } }
                        </button>
                        <h2 class="app-header__title">{ &name }</h2>
                        <span class="app-header__path">{ &path }</span>
                        <button class="btn btn--ghost btn--xs" onclick={toggle_theme}>
                            { if *theme_light { "☀" } else { "🌙" } }
                        </button>
                        <button class="btn btn--danger btn--sm" onclick={on_close_vault}>{ "Fechar" }</button>
                    </header>
                    <div class="app-body">
                        <Sidebar
                            vault_path={path.clone()}
                            on_page_selected={on_page_selected}
                            list_version={*list_version}
                            collapsed={*sidebar_collapsed}
                        />
                        <Editor
                            vault_path={path.clone()}
                            page={(*selected_page).clone()}
                            on_page_deleted={on_page_deleted}
                        />
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
        </div>
    }
}
