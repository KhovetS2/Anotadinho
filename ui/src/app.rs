//! Componente raiz da aplicação.

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::api;
use crate::api::PageMeta;
use crate::components::command_palette::{CommandPalette, PaletteAction};
use crate::components::dialog_host::DialogHost;
use crate::components::editor::Editor;
use crate::components::empty_state::EmptyState;
use crate::components::header_bar::HeaderBar;
use crate::components::kanban::Kanban;
use crate::components::page_view::PageView;
use crate::components::vim_settings_modal::VimSettingsModal;
use crate::components::sidebar::Sidebar;
use crate::components::tab_bar::TabBar;
use crate::dialog::PendingDialog;
use crate::state;

#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);
    let list_version = use_state(|| 0u32);
    let sidebar_collapsed = use_state(|| false);
    let open_tabs = use_state(Vec::<PageMeta>::new);
    let vim_mode = use_state(state::load_vim_mode_enabled);
    let vim_keymap = use_state(state::load_vim_keymap);
    let vim_settings_open = use_state(|| false);
    let toggle_vim_mode = {
        let vim_mode = vim_mode.clone();
        Callback::from(move |_: ()| {
            let next = !*vim_mode;
            state::save_vim_mode_enabled(next);
            vim_mode.set(next);
        })
    };
    let on_vim_keymap_change = {
        let vim_keymap = vim_keymap.clone();
        Callback::from(move |new_keymap: state::VimKeymap| {
            state::save_vim_keymap(&new_keymap);
            vim_keymap.set(new_keymap);
        })
    };
    let pending_dialog = use_state(|| None::<PendingDialog>);
    let palette_open = use_state(|| false);
    let theme_light = use_state(|| {
        web_sys::window().and_then(|w| w.local_storage().ok().flatten())
            .and_then(|s| s.get_item("anotadinho.theme").ok().flatten())
            .map_or(false, |v| v == "light")
    });
    let autosave_enabled = use_state(state::load_autosave_enabled);
    let toggle_autosave = {
        let autosave_enabled = autosave_enabled.clone();
        Callback::from(move |_: ()| {
            let next = !*autosave_enabled;
            state::save_autosave_enabled(next);
            autosave_enabled.set(next);
        })
    };

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
        let open_tabs = open_tabs.clone();
        Callback::from(move |page: PageMeta| {
            // Add to tabs if not already there
            let mut tabs = (*open_tabs).clone();
            if !tabs.iter().any(|t| t.path == page.path) {
                tabs.push(page.clone());
                open_tabs.set(tabs);
            }
            selected_page.set(Some(page));
        })
    };

    // Abre automaticamente a página marcada como "início" deste vault (ver
    // `state::home_page`/`Editor`'s botão 🏠), se houver uma e nenhuma
    // página já estiver selecionada — cobre tanto abrir um vault novo
    // quanto reabrir o último vault salvo no boot do app.
    {
        let vault_path = vault_path.clone();
        let selected_page = selected_page.clone();
        let on_page_selected = on_page_selected.clone();
        use_effect_with(vault_path.clone(), move |_| {
            if let Some(ref vp) = *vault_path {
                if selected_page.is_none() {
                    if let Some(home_path) = state::load_home_page(vp) {
                        let title = std::path::Path::new(&home_path)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_default();
                        let section = if home_path.starts_with("journals/") { "journals" } else { "pages" };
                        on_page_selected.emit(PageMeta { path: home_path, title, section: section.to_string() });
                    }
                }
            }
            || {}
        });
    }

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

    let open_dialog: Callback<PendingDialog> = {
        let pending_dialog = pending_dialog.clone();
        Callback::from(move |d: PendingDialog| pending_dialog.set(Some(d)))
    };

    let dismiss_dialog = {
        let pending_dialog = pending_dialog.clone();
        Callback::from(move |_: ()| pending_dialog.set(None))
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

    // Ações reaproveitadas tanto pelos atalhos diretos (Ctrl+N etc)
    // quanto pelos comandos nomeados da paleta (Ctrl+K) — uma
    // implementação só de cada, dois jeitos de disparar.
    let new_page_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        let open_dialog = open_dialog.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            let on_page_selected = on_page_selected.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título da nova página".to_string(),
                default: "Nova nota".to_string(),
                on_submit: Callback::from(move |title: String| {
                    let vault = vault.clone();
                    let list_version = list_version.clone();
                    let on_page_selected = on_page_selected.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(meta) = api::create_page(&vault, &title).await {
                            on_page_selected.emit(meta);
                            list_version.set(*list_version + 1);
                        }
                    });
                }),
            });
        })
    };

    let new_folder_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let open_dialog = open_dialog.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nome da nova pasta".to_string(),
                default: "Nova pasta".to_string(),
                on_submit: Callback::from(move |name: String| {
                    let vault = vault.clone();
                    let list_version = list_version.clone();
                    let folder_path = format!("pages/{}", name.trim().replace(['/', '\\'], "-"));
                    wasm_bindgen_futures::spawn_local(async move {
                        if api::create_folder(&vault, &folder_path).await.is_ok() {
                            list_version.set(*list_version + 1);
                        }
                    });
                }),
            });
        })
    };

    let today_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            let on_page_selected = on_page_selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(meta) = api::open_today_journal(&vault).await {
                    on_page_selected.emit(meta);
                    list_version.set(*list_version + 1);
                }
            });
        })
    };

    let view_tags_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            let on_page_selected = on_page_selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(pages) = api::list_pages(&vault).await {
                    if let Some(existing) = pages.iter().find(|p| p.path == "pages/tags.md") {
                        on_page_selected.emit(existing.clone());
                        return;
                    }
                }
                if let Ok(meta) = api::create_page_with_type(&vault, "Tags", "tags").await {
                    on_page_selected.emit(meta);
                    list_version.set(*list_version + 1);
                }
            });
        })
    };

    let view_assets_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            let on_page_selected = on_page_selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(pages) = api::list_pages(&vault).await {
                    if let Some(existing) = pages.iter().find(|p| p.path == "pages/assets.md") {
                        on_page_selected.emit(existing.clone());
                        return;
                    }
                }
                if let Ok(meta) = api::create_page_with_type(&vault, "Assets", "assets").await {
                    on_page_selected.emit(meta);
                    list_version.set(*list_version + 1);
                }
            });
        })
    };

    let on_palette_action = {
        let new_page_action = new_page_action.clone();
        let new_folder_action = new_folder_action.clone();
        let today_action = today_action.clone();
        let toggle_theme = toggle_theme.clone();
        let toggle_sidebar = toggle_sidebar.clone();
        let view_tags_action = view_tags_action.clone();
        let view_assets_action = view_assets_action.clone();
        Callback::from(move |action: PaletteAction| match action {
            PaletteAction::NewPage => new_page_action.emit(()),
            PaletteAction::NewFolder => new_folder_action.emit(()),
            PaletteAction::ToggleTheme => toggle_theme.emit(()),
            PaletteAction::ToggleSidebar => toggle_sidebar.emit(()),
            PaletteAction::Today => today_action.emit(()),
            PaletteAction::ViewTags => view_tags_action.emit(()),
            PaletteAction::ViewAssets => view_assets_action.emit(()),
        })
    };

    // Global keyboard (Ctrl+N, Ctrl+K, Escape, Vim mode toggle)
    let onkeydown = {
        let selected_page = selected_page.clone();
        let sidebar_collapsed = sidebar_collapsed.clone();
        let open_tabs = open_tabs.clone();
        let palette_open = palette_open.clone();
        let new_page_action = new_page_action.clone();
        Callback::from(move |e: KeyboardEvent| {
            let ctrl = e.ctrl_key() || e.meta_key();
            match (ctrl, e.key().as_str()) {
                (true, "n") => {
                    e.prevent_default();
                    new_page_action.emit(());
                }
                (true, "k") | (true, "p") => {
                    e.prevent_default();
                    palette_open.set(true);
                }
                (true, "b") => {
                    e.prevent_default();
                    sidebar_collapsed.set(!*sidebar_collapsed);
                }
                (false, "Escape") => {
                    if selected_page.is_some() {
                        selected_page.set(None);
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
                autosave_enabled={*autosave_enabled}
                vim_mode_enabled={*vim_mode}
                on_toggle_sidebar={toggle_sidebar}
                on_toggle_theme={toggle_theme}
                on_toggle_autosave={toggle_autosave}
                on_toggle_vim_mode={toggle_vim_mode}
                on_open_vim_settings={{
                    let vim_settings_open = vim_settings_open.clone();
                    Callback::from(move |_: ()| vim_settings_open.set(true))
                }}
                on_close_vault={on_close_vault}
                on_open_vault={on_open_vault_shortcut}
            />
            if vault_open {
                <div class="app-layout">
                    <div class="app-body">
                        <Sidebar
                            vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                            on_page_selected={on_page_selected.clone()}
                            list_version={*list_version}
                            collapsed={*sidebar_collapsed}
                            open_dialog={open_dialog.clone()}
                        />
                        <div class="app-main-panel">
                            <TabBar
                                tabs={(*open_tabs).clone()}
                                active_path={selected_page.as_ref().map(|p| p.path.clone())}
                                on_select={on_tab_select}
                                on_close={on_tab_close}
                            />
                            <PageView
                                vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                                page={(*selected_page).clone()}
                                on_page_deleted={on_page_deleted.clone()}
                                on_page_selected={on_page_selected.clone()}
                                open_dialog={open_dialog.clone()}
                                autosave_enabled={*autosave_enabled}
                                vim_mode_enabled={*vim_mode}
                                vim_keymap={(*vim_keymap).clone()}
                            />
                        </div>
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
            <DialogHost pending={(*pending_dialog).clone()} on_dismiss={dismiss_dialog} />
            if *vim_settings_open {
                <VimSettingsModal
                    keymap={(*vim_keymap).clone()}
                    on_change={on_vim_keymap_change}
                    on_close={{
                        let vim_settings_open = vim_settings_open.clone();
                        Callback::from(move |_: ()| vim_settings_open.set(false))
                    }}
                />
            }
            if *palette_open && vault_open {
                <CommandPalette
                    vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                    on_close={{
                        let palette_open = palette_open.clone();
                        Callback::from(move |_: ()| palette_open.set(false))
                    }}
                    on_page_selected={on_page_selected.clone()}
                    on_action={on_palette_action}
                />
            }
        </div>
    }
}
