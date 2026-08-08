//! Componente raiz da aplicação.

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::api;
use crate::api::PageMeta;
use crate::components::command_palette::{CommandPalette, PaletteAction};
use crate::components::dialog_host::DialogHost;
use crate::components::editor::Editor;
use crate::components::empty_state::EmptyState;
use crate::components::global_keymap_modal::GlobalKeymapModal;
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
    let git_files = use_state(|| None::<Vec<api::GitFileEntry>>);
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
    let global_keymap = use_state(state::load_global_keymap);
    let global_keymap_settings_open = use_state(|| false);
    let on_global_keymap_change = {
        let global_keymap = global_keymap.clone();
        Callback::from(move |new_keymap: state::GlobalKeymap| {
            state::save_global_keymap(&new_keymap);
            global_keymap.set(new_keymap);
        })
    };
    // Ponte pro Editor (ciclo 105) — `Some((action, nonce))` quando o
    // GlobalKeymap dispara Salvar/Desfazer/Refazer; o nonce garante que
    // o efeito do Editor reage de novo mesmo pra ações repetidas em
    // sequência (só trocar a `action` não bastaria se ela repetir).
    let global_editor_action = use_state(|| None::<(state::GlobalEditorAction, u32)>);
    let global_editor_action_nonce = use_mut_ref(|| 0u32);
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
        let git_files = git_files.clone();
        use_effect_with(vault_path.clone(), move |_| {
            let mut interval: Option<gloo_timers::callback::Interval> = None;
            if let Some(ref p) = *vault_path {
                let path = p.clone();
                // Busca inicial imediata — sem isso o indicador só
                // apareceria depois do primeiro tick do intervalo (3s).
                {
                    let path = path.clone();
                    let git_files = git_files.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(status) = api::git_status(&path).await {
                            git_files.set(status);
                        }
                    });
                }
                // Guarda contra empilhar processos `git` se `git status`
                // demorar mais que os 3s do intervalo — só dispara a
                // próxima checagem se a anterior já terminou.
                let git_busy = std::rc::Rc::new(std::cell::RefCell::new(false));
                let iv = gloo_timers::callback::Interval::new(3000, move || {
                    let path = path.clone();
                    let list_version = list_version.clone();
                    let git_files = git_files.clone();
                    let git_busy = git_busy.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(true) = api::check_changes(&path).await {
                            list_version.set(*list_version + 1);
                        }
                        if !*git_busy.borrow() {
                            *git_busy.borrow_mut() = true;
                            if let Ok(status) = api::git_status(&path).await {
                                git_files.set(status);
                            }
                            *git_busy.borrow_mut() = false;
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
    // Pede o título e cria a página — sem template (comportamento de
    // sempre) ou a partir de um template (ciclo 100), decidido por
    // quem chama. Único lugar que sabe pedir título + criar, reusado
    // pelos dois braços de `new_page_action`.
    fn prompt_title_and_create(
        open_dialog: &Callback<PendingDialog>,
        vault: String,
        list_version: UseStateHandle<u32>,
        on_page_selected: Callback<PageMeta>,
        template_path: Option<String>,
    ) {
        open_dialog.emit(PendingDialog::Prompt {
            title: "Título da nova página".to_string(),
            default: "Nova nota".to_string(),
            on_submit: Callback::from(move |title: String| {
                let vault = vault.clone();
                let list_version = list_version.clone();
                let on_page_selected = on_page_selected.clone();
                let template_path = template_path.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let result = match template_path {
                        Some(tpl) => api::create_page_from_template(&vault, &tpl, &title).await,
                        None => api::create_page(&vault, &title).await,
                    };
                    if let Ok(meta) = result {
                        on_page_selected.emit(meta);
                        list_version.set(*list_version + 1);
                    }
                });
            }),
        });
    }

    let new_page_action = {
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        let open_dialog = open_dialog.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let list_version = list_version.clone();
            let on_page_selected = on_page_selected.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let templates = api::list_templates(&vault).await.unwrap_or_default();
                if templates.is_empty() {
                    prompt_title_and_create(&open_dialog, vault, list_version, on_page_selected, None);
                    return;
                }
                let mut options: Vec<(String, String)> =
                    vec![(String::new(), "Página em branco".to_string())];
                options.extend(templates.into_iter().map(|t| (t.path, t.title)));
                let vault2 = vault.clone();
                let list_version2 = list_version.clone();
                let on_page_selected2 = on_page_selected.clone();
                let open_dialog2 = open_dialog.clone();
                open_dialog.emit(PendingDialog::Select {
                    title: "Escolher template".to_string(),
                    options,
                    on_select: Callback::from(move |template_path: String| {
                        let template = if template_path.is_empty() { None } else { Some(template_path.clone()) };
                        prompt_title_and_create(
                            &open_dialog2,
                            vault2.clone(),
                            list_version2.clone(),
                            on_page_selected2.clone(),
                            template,
                        );
                    }),
                });
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

    let export_vault_action = {
        let vault_path = vault_path.clone();
        let open_dialog = open_dialog.clone();
        Callback::from(move |_: ()| {
            let vault = (*vault_path).clone().unwrap_or_default();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::export_folder(&vault, "").await {
                    Ok(dump) => crate::download::download_text_file("vault.md", "text/markdown", &dump),
                    Err(e) => open_dialog.emit(PendingDialog::Alert {
                        message: format!("Erro ao exportar vault: {}", e),
                    }),
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
        let export_vault_action = export_vault_action.clone();
        Callback::from(move |action: PaletteAction| match action {
            PaletteAction::NewPage => new_page_action.emit(()),
            PaletteAction::NewFolder => new_folder_action.emit(()),
            PaletteAction::ToggleTheme => toggle_theme.emit(()),
            PaletteAction::ToggleSidebar => toggle_sidebar.emit(()),
            PaletteAction::Today => today_action.emit(()),
            PaletteAction::ViewTags => view_tags_action.emit(()),
            PaletteAction::ViewAssets => view_assets_action.emit(()),
            PaletteAction::ExportVault => export_vault_action.emit(()),
        })
    };

    // GlobalKeymap dispatcher (ciclo 105) — olha a tecla pressionada,
    // acha qual ação do `GlobalKeymap` corresponde (todas com Ctrl/Cmd
    // implícito, ver `state::GlobalKeymap`), dispara o callback certo.
    // Escape continua um caso à parte, fora do keymap (não está na
    // lista de ações customizáveis).
    let onkeydown = {
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        let palette_open = palette_open.clone();
        let new_page_action = new_page_action.clone();
        let new_folder_action = new_folder_action.clone();
        let toggle_theme = toggle_theme.clone();
        let toggle_sidebar = toggle_sidebar.clone();
        let today_action = today_action.clone();
        let view_tags_action = view_tags_action.clone();
        let view_assets_action = view_assets_action.clone();
        let toggle_vim_mode = toggle_vim_mode.clone();
        let on_tab_close = on_tab_close.clone();
        let global_keymap = global_keymap.clone();
        let global_editor_action = global_editor_action.clone();
        let global_editor_action_nonce = global_editor_action_nonce.clone();
        Callback::from(move |e: KeyboardEvent| {
            let ctrl = e.ctrl_key() || e.meta_key();

            if e.key() == "Escape" {
                if !ctrl && selected_page.is_some() {
                    selected_page.set(None);
                }
                return;
            }
            if !ctrl {
                return;
            }

            let key = e.key();
            let km = &*global_keymap;
            let matches = |bound: &str| !bound.is_empty() && key.eq_ignore_ascii_case(bound);
            let fire_editor_action = |action: state::GlobalEditorAction| {
                let mut nonce = global_editor_action_nonce.borrow_mut();
                *nonce += 1;
                global_editor_action.set(Some((action, *nonce)));
            };

            if matches(&km.new_page) {
                e.prevent_default();
                new_page_action.emit(());
            } else if matches(&km.new_folder) {
                e.prevent_default();
                new_folder_action.emit(());
            } else if matches(&km.toggle_theme) {
                e.prevent_default();
                toggle_theme.emit(());
            } else if matches(&km.toggle_sidebar) {
                e.prevent_default();
                toggle_sidebar.emit(());
            } else if matches(&km.today) {
                e.prevent_default();
                today_action.emit(());
            } else if matches(&km.view_tags) {
                e.prevent_default();
                view_tags_action.emit(());
            } else if matches(&km.view_assets) {
                e.prevent_default();
                view_assets_action.emit(());
            } else if matches(&km.open_palette) {
                e.prevent_default();
                palette_open.set(true);
            } else if matches(&km.save) {
                e.prevent_default();
                fire_editor_action(state::GlobalEditorAction::Save);
            } else if matches(&km.undo) {
                e.prevent_default();
                fire_editor_action(state::GlobalEditorAction::Undo);
            } else if matches(&km.redo) {
                e.prevent_default();
                fire_editor_action(state::GlobalEditorAction::Redo);
            } else if matches(&km.toggle_vim_mode) {
                e.prevent_default();
                toggle_vim_mode.emit(());
            } else if matches(&km.next_tab) {
                e.prevent_default();
                let tabs = (*open_tabs).clone();
                if !tabs.is_empty() {
                    if let Some(ref sel) = *selected_page {
                        let pos = tabs.iter().position(|t| t.path == sel.path).unwrap_or(0);
                        let next = (pos + 1) % tabs.len();
                        selected_page.set(Some(tabs[next].clone()));
                    } else {
                        selected_page.set(Some(tabs[0].clone()));
                    }
                }
            } else if matches(&km.prev_tab) {
                e.prevent_default();
                let tabs = (*open_tabs).clone();
                if !tabs.is_empty() {
                    if let Some(ref sel) = *selected_page {
                        let pos = tabs.iter().position(|t| t.path == sel.path).unwrap_or(0);
                        let prev = (pos + tabs.len() - 1) % tabs.len();
                        selected_page.set(Some(tabs[prev].clone()));
                    } else {
                        selected_page.set(Some(tabs[0].clone()));
                    }
                }
            } else if matches(&km.close_tab) {
                e.prevent_default();
                let tabs = (*open_tabs).clone();
                if let Some(ref sel) = *selected_page {
                    if let Some(pos) = tabs.iter().position(|t| t.path == sel.path) {
                        on_tab_close.emit(pos);
                    }
                }
            } else if matches(&km.focus_sidebar) {
                e.prevent_default();
                // Comportamento completo (destacar item + navegar por
                // seta) chega no ciclo 106 — aqui só reserva a tecla.
            } else if matches(&km.focus_editor) {
                e.prevent_default();
                // Idem — comportamento completo em ciclo futuro.
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
                git_files={(*git_files).clone()}
                on_toggle_sidebar={toggle_sidebar}
                on_toggle_theme={toggle_theme}
                on_toggle_autosave={toggle_autosave}
                on_toggle_vim_mode={toggle_vim_mode}
                on_open_vim_settings={{
                    let vim_settings_open = vim_settings_open.clone();
                    Callback::from(move |_: ()| vim_settings_open.set(true))
                }}
                on_open_global_keymap_settings={{
                    let global_keymap_settings_open = global_keymap_settings_open.clone();
                    Callback::from(move |_: ()| global_keymap_settings_open.set(true))
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
                                global_action={*global_editor_action}
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
            if *global_keymap_settings_open {
                <GlobalKeymapModal
                    keymap={(*global_keymap).clone()}
                    on_change={on_global_keymap_change}
                    on_close={{
                        let global_keymap_settings_open = global_keymap_settings_open.clone();
                        Callback::from(move |_: ()| global_keymap_settings_open.set(false))
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
