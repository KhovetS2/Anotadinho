//! Componente raiz da aplicação.

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::api;
use crate::api::PageMeta;
use crate::components::editor::Editor;
use crate::components::empty_state::EmptyState;
use crate::components::header_bar::HeaderBar;
use crate::components::sidebar::Sidebar;
use crate::components::tab_bar::TabBar;
use crate::state;

#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);
    let list_version = use_state(|| 0u32);
    let sidebar_collapsed = use_state(|| false);
    let open_tabs = use_state(Vec::<PageMeta>::new);
    let vim_mode = use_state(|| false);
    let theme_light = use_state(|| {
        web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("anotadinho.theme").ok().flatten())
            .map_or(false, |v| v == "light")
    });

    // Apply theme
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

    // Track tabs when page is selected
    {
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        use_effect_with(selected_page.clone(), move |_| {
            if let Some(ref page) = *selected_page {
                let mut tabs = (*open_tabs).clone();
                if !tabs.iter().any(|t| t.path == page.path) {
                    tabs.push(page.clone());
                    open_tabs.set(tabs);
                }
            }
            || {}
        });
    }

    let on_vault_selected = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        Callback::from(move |path: String| {
            let name = state::extract_name_from_path(&path);
            state::save_vault_path(&path);
            state::save_vault_name(&name);
            vault_path.set(Some(path));
            vault_name.set(Some(name));
            selected_page.set(None);
            open_tabs.set(Vec::new());
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
        let open_tabs = open_tabs.clone();
        Callback::from(move |_| {
            state::clear_vault();
            vault_path.set(None); vault_name.set(None);
            selected_page.set(None); open_tabs.set(Vec::new());
        })
    };

    let on_page_deleted = {
        let selected_page = selected_page.clone();
        let list_version = list_version.clone();
        let open_tabs = open_tabs.clone();
        Callback::from(move |_| {
            if let Some(ref page) = *selected_page {
                let mut tabs = (*open_tabs).clone();
                tabs.retain(|t| t.path != page.path);
                open_tabs.set(tabs);
            }
            selected_page.set(None);
            list_version.set(*list_version + 1);
        })
    };

    let on_open_vault_shortcut = {
        let cb = on_vault_selected.clone();
        Callback::from(move |_: ()| {
            let cb = cb.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(path)) = api::open_folder_dialog().await {
                    cb.emit(path);
                }
            });
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

    let on_tab_select = {
        let selected_page = selected_page.clone();
        Callback::from(move |page: PageMeta| selected_page.set(Some(page)))
    };

    let on_tab_close = {
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        Callback::from(move |idx: usize| {
            let mut tabs = (*open_tabs).clone();
            if idx < tabs.len() {
                let closed = tabs.remove(idx);
                if selected_page.as_ref().map_or(false, |p| p.path == closed.path) {
                    let next = tabs.get(idx).or_else(|| tabs.get(idx.saturating_sub(1))).cloned();
                    selected_page.set(next);
                }
                open_tabs.set(tabs);
            }
        })
    };

    // Global keyboard (Ctrl+N, Escape, Vim mode toggle)
    let onkeydown = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let selected_page = selected_page.clone();
        let on_page_selected = on_page_selected.clone();
        let sidebar_collapsed = sidebar_collapsed.clone();
        let vim_mode = vim_mode.clone();
        let open_tabs = open_tabs.clone();
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
                (true, "b") => {
                    e.prevent_default();
                    sidebar_collapsed.set(!*sidebar_collapsed);
                }
                // Vim mode toggle
                (false, "Escape") => {
                    if selected_page.is_some() {
                        selected_page.set(None);
                    } else {
                        vim_mode.set(false);
                    }
                }
                // Tab switching
                (true, "w") => {
                    e.prevent_default();
                    let tabs = (*open_tabs).clone();
                    if tabs.is_empty() { return; }
                    if let Some(ref sel) = *selected_page {
                        let pos = tabs.iter().position(|t| t.path == sel.path).unwrap_or(0);
                        let next = (pos + 1) % tabs.len();
                        selected_page.set(Some(tabs[next].clone()));
                    } else {
                        selected_page.set(Some(tabs[0].clone()));
                    }
                }
                (true, "p") => {
                    e.prevent_default();
                    if let Some(ref path) = *vault_path {
                        let vault = path.clone();
                        let on_page_selected = on_page_selected.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Ok(pages) = api::list_pages(&vault).await {
                                let list = pages.iter().map(|p| p.title.clone()).collect::<Vec<_>>().join("\n");
                                let q = gloo_dialogs::prompt(
                                    &format!("Páginas:\n{}\n\nIr para:", list), None
                                ).unwrap_or_default();
                                if let Some(page) = pages.iter().find(|p| p.title.to_lowercase() == q.to_lowercase()) {
                                    on_page_selected.emit(page.clone());
                                }
                            }
                        });
                    }
                }
                _ => {}
            }
        })
    };

    let vault_open = vault_path.is_some();

    html! {
        <div class="app-root" tabindex="0" {onkeydown}>
            <HeaderBar
                vault_name={(*vault_name).clone()}
                vault_path={(*vault_path).clone()}
                sidebar_collapsed={*sidebar_collapsed}
                theme_light={*theme_light}
                on_toggle_sidebar={toggle_sidebar}
                on_toggle_theme={toggle_theme}
                on_close_vault={on_close_vault}
                on_open_vault={on_open_vault_shortcut}
            />
            if vault_open {
                <div class="app-layout">
                    <div class="app-body">
                        <Sidebar
                            vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                            on_page_selected={on_page_selected}
                            list_version={*list_version}
                            collapsed={*sidebar_collapsed}
                        />
                        <div class="app-main-panel">
                            <TabBar
                                tabs={(*open_tabs).clone()}
                                active_path={selected_page.as_ref().map(|p| p.path.clone())}
                                on_select={on_tab_select}
                                on_close={on_tab_close}
                            />
                            <Editor
                                vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                                page={(*selected_page).clone()}
                                on_page_deleted={on_page_deleted}
                            />
                        </div>
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
        </div>
    }
}
