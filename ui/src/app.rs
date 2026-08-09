//! Componente raiz da aplicação.

use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::api;
use crate::api::PageMeta;
use crate::components::cheatsheet_modal::CheatsheetModal;
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

/// `?` abre o cheatsheet de atalhos (ciclo 108) — mas só fora de campo
/// de texto/contenteditable, senão digitar um "?" de verdade numa nota
/// ou na busca da sidebar abriria o overlay sem querer.
fn is_text_input_target(e: &KeyboardEvent) -> bool {
    let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Element>().ok()) else { return false };
    let tag = target.tag_name().to_lowercase();
    if matches!(tag.as_str(), "input" | "textarea" | "select") {
        return true;
    }
    target.get_attribute("contenteditable").as_deref() == Some("true")
}

#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);
    let list_version = use_state(|| 0u32);
    let git_files = use_state(|| None::<Vec<api::GitFileEntry>>);
    let sidebar_collapsed = use_state(|| false);
    let open_tabs = use_state(Vec::<PageMeta>::new);
    // Página inicial (ciclo 089) — movida pra cá (ciclo 109) de dentro
    // do `Editor` porque a `TabBar` (irmã do `Editor`, não descendente)
    // também precisa saber qual página é a inicial, pra mostrar só o
    // ícone 🏠 na aba fixa em vez do título.
    let home_page = use_state(|| None::<String>);
    {
        let vault_path = vault_path.clone();
        let home_page = home_page.clone();
        use_effect_with(vault_path.clone(), move |_| {
            home_page.set(vault_path.as_ref().and_then(|v| state::load_home_page(v)));
            || {}
        });
    }
    let on_toggle_home = {
        let vault_path = vault_path.clone();
        let home_page = home_page.clone();
        Callback::from(move |path: String| {
            let Some(ref vault) = *vault_path else { return };
            if home_page.as_deref() == Some(path.as_str()) {
                state::clear_home_page(vault);
                home_page.set(None);
            } else {
                state::save_home_page(vault, &path);
                home_page.set(Some(path));
            }
        })
    };
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
    // Modo de navegação hierárquico por teclado (ciclo 133) —
    // `nav_mode_enabled` é a capacidade (persistida, alternada via
    // `toggle_nav_mode` no `GlobalKeymap`, mesmo padrão do vim mode);
    // `nav_mode_active`/`nav_stack` são a SESSÃO de navegação em si,
    // sempre transitórias — começam em `false`/vazia a cada boot,
    // mesmo com a capacidade ligada (entra na primeira seta
    // pressionada, não precisa persistir "estava navegando"). Pilha
    // vazia = nível raiz (grupo `"root"` de `nav_mode.rs`); cada
    // entrada é o id de um grupo (`data-nav-group`) em que o usuário
    // desceu via Enter.
    let nav_mode_enabled = use_state(state::load_nav_mode_enabled);
    let toggle_nav_mode = {
        let nav_mode_enabled = nav_mode_enabled.clone();
        Callback::from(move |_: ()| {
            let next = !*nav_mode_enabled;
            state::save_nav_mode_enabled(next);
            nav_mode_enabled.set(next);
        })
    };
    let nav_mode_active = use_state(|| false);
    let nav_stack = use_state(Vec::<String>::new);
    // Destaque visual do GRUPO atual (não só do item focado — o
    // usuário pediu especificamente pra saber "em qual wrapper está",
    // e `:focus-visible` já cuida do item em si). Imperativo porque o
    // elemento do grupo pode viver em qualquer componente filho
    // (header/sidebar/tabbar) sem precisar passar `nav_stack` como
    // prop pra cada um só pra isso — mesma filosofia de "consultar o
    // DOM ao vivo" do resto do nav-mode.
    {
        let nav_mode_active = nav_mode_active.clone();
        let nav_stack = nav_stack.clone();
        use_effect_with((*nav_mode_active, (*nav_stack).clone()), move |(active, stack)| {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                if let Ok(stale) = doc.query_selector_all(".nav-mode__region-active") {
                    for i in 0..stale.length() {
                        if let Some(el) = stale.item(i).and_then(|n| n.dyn_into::<web_sys::HtmlElement>().ok()) {
                            let _ = el.class_list().remove_1("nav-mode__region-active");
                            let _ = el.style().remove_property("--nav-mode-depth-color");
                        }
                    }
                }
                if *active {
                    if let Some(group_id) = stack.last() {
                        if let Some(el) = doc
                            .query_selector(&format!("[data-nav-group=\"{}\"]", group_id.replace('"', "")))
                            .ok()
                            .flatten()
                            .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
                        {
                            let _ = el.class_list().add_1("nav-mode__region-active");
                            // Ciclo 136: cor varia com a profundidade (azul→roxo)
                            // pra dar uma pista visual de "quão fundo" a sessão
                            // está, além do texto do badge.
                            let _ = el.style().set_property(
                                "--nav-mode-depth-color",
                                &crate::nav_mode::depth_color_css(stack.len()),
                            );
                        }
                    }
                }
            }
            || {}
        });
    }
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
    // "Focar sidebar" (ciclo 105/106) — nonce que a `Sidebar` observa
    // pra ativar a navegação por teclado (destaca o primeiro item e
    // foca o container). `None` = nunca ativado (não força foco no
    // boot do app).
    let sidebar_activate_nav = use_state(|| None::<u32>);
    let sidebar_activate_nav_nonce = use_mut_ref(|| 0u32);
    // Cheatsheet de atalhos (ciclo 108) — leitura dos dois keymaps.
    let cheatsheet_open = use_state(|| false);
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

    // Depois de um pull/commit+push (ciclo 119), rebusca git status
    // imediatamente (sem esperar o próximo tick do polling de 3s,
    // ciclo 103) e a lista de páginas (pull pode trazer arquivos
    // novos/mudados de outra máquina).
    let on_git_changed = {
        let vault_path = vault_path.clone();
        let git_files = git_files.clone();
        let list_version = list_version.clone();
        Callback::from(move |_: ()| {
            let Some(path) = (*vault_path).clone() else { return };
            let git_files = git_files.clone();
            let list_version = list_version.clone();
            list_version.set(*list_version + 1);
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(status) = api::git_status(&path).await {
                    git_files.set(status);
                }
            });
        })
    };

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

    // Mesmo padrão de `prompt_title_and_create`, mas cria com um
    // `page_type` fixo (kanban/calendar/table/graph) em vez de
    // template — caminho separado porque as duas formas de "resolver
    // qual conteúdo a página nasce com" (template markdown vs. tipo de
    // frontmatter) não compartilham o `match` interno sem ficar mais
    // confuso do que duas funções pequenas.
    fn prompt_title_and_create_typed(
        open_dialog: &Callback<PendingDialog>,
        vault: String,
        list_version: UseStateHandle<u32>,
        on_page_selected: Callback<PageMeta>,
        page_type: &'static str,
    ) {
        open_dialog.emit(PendingDialog::Prompt {
            title: "Título da nova página".to_string(),
            default: "Nova nota".to_string(),
            on_submit: Callback::from(move |title: String| {
                let vault = vault.clone();
                let list_version = list_version.clone();
                let on_page_selected = on_page_selected.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(meta) = api::create_page_with_type(&vault, &title, page_type).await {
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
        let cheatsheet_open = cheatsheet_open.clone();
        let vault_path = vault_path.clone();
        let list_version = list_version.clone();
        let on_page_selected = on_page_selected.clone();
        let open_dialog = open_dialog.clone();
        Callback::from(move |action: PaletteAction| match action {
            PaletteAction::NewPage => new_page_action.emit(()),
            PaletteAction::NewFolder => new_folder_action.emit(()),
            PaletteAction::ToggleTheme => toggle_theme.emit(()),
            PaletteAction::ToggleSidebar => toggle_sidebar.emit(()),
            PaletteAction::Today => today_action.emit(()),
            PaletteAction::ViewTags => view_tags_action.emit(()),
            PaletteAction::ViewAssets => view_assets_action.emit(()),
            PaletteAction::ExportVault => export_vault_action.emit(()),
            PaletteAction::ViewCheatsheet => cheatsheet_open.set(true),
            PaletteAction::NewPageOfType(page_type) => {
                let vault = (*vault_path).clone().unwrap_or_default();
                prompt_title_and_create_typed(
                    &open_dialog,
                    vault,
                    list_version.clone(),
                    on_page_selected.clone(),
                    page_type,
                );
            }
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
        let sidebar_activate_nav = sidebar_activate_nav.clone();
        let sidebar_activate_nav_nonce = sidebar_activate_nav_nonce.clone();
        let cheatsheet_open = cheatsheet_open.clone();
        let nav_mode_enabled = nav_mode_enabled.clone();
        let nav_mode_active = nav_mode_active.clone();
        let nav_stack = nav_stack.clone();
        let toggle_nav_mode = toggle_nav_mode.clone();
        let pending_dialog = pending_dialog.clone();
        let vim_settings_open = vim_settings_open.clone();
        let global_keymap_settings_open = global_keymap_settings_open.clone();
        Callback::from(move |e: KeyboardEvent| {
            let ctrl = e.ctrl_key() || e.meta_key();

            if e.key() == "?" && !ctrl && !is_text_input_target(&e) {
                e.prevent_default();
                cheatsheet_open.set(true);
                return;
            }

            // Modo de navegação hierárquico (ciclo 133) — vem ANTES do
            // Escape genérico logo abaixo porque, com uma sessão
            // ativa, Escape pertence ao nav-mode (sobe um nível ou sai
            // de vez), não ao "desselecionar página" de sempre. Setas
            // não usam Ctrl (igual Backspace/Enter aqui dentro), por
            // isso esse bloco também precisa vir antes do
            // `if !ctrl { return; }` logo abaixo.
            //
            // Suprimido por completo (ciclo 136) enquanto qualquer
            // overlay estiver aberto — sem isso, Escape fechando um
            // modal/menu/paleta TAMBÉM subia um nível (ou saía) do
            // nav-mode na mesma tecla, já que os dois "ouvem" o mesmo
            // Escape. Cada overlay continua dono do próprio teclado
            // (Modal já trata Tab/Escape sozinho, paleta e cheatsheet
            // idem) — aqui só evita o nav-mode competir por cima.
            let any_overlay_open = pending_dialog.is_some()
                || *palette_open
                || *cheatsheet_open
                || *vim_settings_open
                || *global_keymap_settings_open;
            if !ctrl && !any_overlay_open {
                let key = e.key();
                let doc = web_sys::window().and_then(|w| w.document());
                // Menus dropdown "locais" (⚙ do header, popover de git,
                // "⋯" do editor — ciclo 125) não são overlays do
                // `app.rs`, então `any_overlay_open` não os vê. Sinal
                // mais genérico: quando um deles abre, o auto-foco já
                // move o `activeElement` pra DENTRO do menu (não é
                // mais o item do nav-mode que tinha `data-nav-item`) —
                // se o foco atual não é algo que o próprio nav-mode
                // colocou lá, o teclado pertence a quem quer que tenha
                // roubado o foco, não ao nav-mode.
                let focus_is_nav_tracked = doc.as_ref()
                    .and_then(|d| d.active_element())
                    .is_some_and(|el| el.has_attribute("data-nav-item"));
                if *nav_mode_active {
                    match key.as_str() {
                        // Setas se AUTO-CURAM mesmo sem `focus_is_nav_tracked`
                        // (ex: depois de um menu local fechar e deixar o foco
                        // em `<body>`) — `index_of` já cai pro item 0 quando
                        // o foco atual não é achado na lista, então só
                        // apertar a seta de novo recupera a navegação.
                        "ArrowDown" | "ArrowRight" | "ArrowUp" | "ArrowLeft" => {
                            e.prevent_default();
                            if let Some(doc) = doc {
                                let group_id = nav_stack.last().cloned().unwrap_or_else(|| "root".to_string());
                                let items = crate::nav_mode::items_in_group(&doc, &group_id);
                                if !items.is_empty() {
                                    let active = doc.active_element();
                                    let idx = crate::nav_mode::index_of(&items, active.as_ref());
                                    let forward = matches!(key.as_str(), "ArrowDown" | "ArrowRight");
                                    let next_idx = match idx {
                                        Some(i) if forward => (i + 1) % items.len(),
                                        Some(i) => (i + items.len() - 1) % items.len(),
                                        None => 0,
                                    };
                                    crate::nav_mode::focus_item(&items[next_idx]);
                                }
                            }
                            return;
                        }
                        "Enter" => {
                            // Se o foco escapou pra algo que o nav-mode não
                            // controla (menu local aberto, ou body depois
                            // que ele fechou), esse Enter não é do nav-mode
                            // — deixa passar sem `preventDefault`.
                            if !focus_is_nav_tracked {
                                return;
                            }
                            e.prevent_default();
                            if let Some(doc) = doc {
                                if let Some(active) = doc.active_element() {
                                    if let Some(delegate) = crate::nav_mode::delegate_of(&active) {
                                        // Entrega o teclado pra um sistema de
                                        // navegação já existente (sidebar,
                                        // ciclo 106; ou o Ctrl+L de sempre pro
                                        // editor) — nav-mode sai da sessão em
                                        // vez de tentar navegar dentro.
                                        match delegate.as_str() {
                                            "sidebar" => {
                                                let mut nonce = sidebar_activate_nav_nonce.borrow_mut();
                                                *nonce += 1;
                                                sidebar_activate_nav.set(Some(*nonce));
                                            }
                                            "editor" => {
                                                // Página de texto normal: foca o
                                                // contenteditable (mesma query do
                                                // Ctrl+L de sempre). Páginas
                                                // tipadas (kanban/calendário/
                                                // tabela/grafo, ciclo 134) não
                                                // têm esse elemento — cai pro
                                                // primeiro item focável dentro do
                                                // conteúdo marcado
                                                // `data-nav-content-root`
                                                // (cada um já tem seu próprio
                                                // Enter/Espaço dos ciclos
                                                // 126/127, só precisa do foco
                                                // inicial).
                                                let target = doc
                                                    .query_selector(".editor__wysiwyg[contenteditable=\"true\"]")
                                                    .ok()
                                                    .flatten()
                                                    .or_else(|| {
                                                        doc.query_selector("[data-nav-content-root] [tabindex=\"0\"]")
                                                            .ok()
                                                            .flatten()
                                                    });
                                                if let Some(el) = target {
                                                    crate::nav_mode::focus_item(&el);
                                                }
                                            }
                                            _ => {}
                                        }
                                        nav_mode_active.set(false);
                                        nav_stack.set(Vec::new());
                                    } else if let Some(group_id) = crate::nav_mode::group_of(&active) {
                                        // Desce um nível — vira o grupo atual.
                                        let mut stack = (*nav_stack).clone();
                                        stack.push(group_id.clone());
                                        let items = crate::nav_mode::items_in_group(&doc, &group_id);
                                        if let Some(first) = items.first() {
                                            crate::nav_mode::focus_item(first);
                                        }
                                        nav_stack.set(stack);
                                    } else if let Some(html_el) = active.dyn_ref::<web_sys::HtmlElement>() {
                                        // Folha — mesma ação de um clique real.
                                        html_el.click();
                                    }
                                }
                            }
                            return;
                        }
                        "Backspace" => {
                            if !focus_is_nav_tracked {
                                return;
                            }
                            e.prevent_default();
                            let mut stack = (*nav_stack).clone();
                            stack.pop();
                            let new_group = stack.last().cloned().unwrap_or_else(|| "root".to_string());
                            nav_stack.set(stack);
                            if let Some(doc) = doc {
                                let items = crate::nav_mode::items_in_group(&doc, &new_group);
                                if let Some(first) = items.first() {
                                    crate::nav_mode::focus_item(first);
                                }
                            }
                            return;
                        }
                        "Escape" => {
                            // O caso que este ciclo (136) corrige: um menu
                            // local (⚙, popover de git, "⋯") já tem seu
                            // próprio listener de Escape — sem essa guarda,
                            // fechar o menu TAMBÉM subia um nível (ou saía)
                            // do nav-mode na mesma tecla.
                            if !focus_is_nav_tracked {
                                return;
                            }
                            e.prevent_default();
                            if nav_stack.is_empty() {
                                nav_mode_active.set(false);
                            } else {
                                nav_stack.set(Vec::new());
                                if let Some(doc) = doc {
                                    let items = crate::nav_mode::items_in_group(&doc, "root");
                                    if let Some(first) = items.first() {
                                        crate::nav_mode::focus_item(first);
                                    }
                                }
                            }
                            return;
                        }
                        _ => {}
                    }
                } else if *nav_mode_enabled
                    && !is_text_input_target(&e)
                    && matches!(key.as_str(), "ArrowDown" | "ArrowUp" | "ArrowLeft" | "ArrowRight")
                {
                    // Primeira seta com a capacidade ligada — inicia a
                    // sessão direto na lista de regiões de topo, sem
                    // precisar de uma tecla dedicada só pra "entrar".
                    e.prevent_default();
                    nav_mode_active.set(true);
                    nav_stack.set(Vec::new());
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        let items = crate::nav_mode::items_in_group(&doc, "root");
                        if let Some(first) = items.first() {
                            crate::nav_mode::focus_item(first);
                        }
                    }
                    return;
                }
            }

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

            // Ctrl+1..9 pula direto pra aba de índice 0-8 — FIXO, fora
            // do GlobalKeymap customizável de propósito (ciclo 107,
            // mesma convenção de navegador/editor de código: uma coisa
            // a menos pra configurar, e menos chance de colisão com uma
            // tecla que o usuário reatribuiu por engano).
            if let Some(digit) = key.chars().next().filter(|_| key.len() == 1).and_then(|c| c.to_digit(10)) {
                if (1..=9).contains(&digit) {
                    e.prevent_default();
                    let tabs = (*open_tabs).clone();
                    if let Some(tab) = tabs.get(digit as usize - 1) {
                        selected_page.set(Some(tab.clone()));
                    }
                    return;
                }
            }

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
            } else if matches(&km.toggle_nav_mode) {
                e.prevent_default();
                toggle_nav_mode.emit(());
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
                let mut nonce = sidebar_activate_nav_nonce.borrow_mut();
                *nonce += 1;
                sidebar_activate_nav.set(Some(*nonce));
            } else if matches(&km.focus_editor) {
                e.prevent_default();
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(el) = doc.query_selector(".editor__wysiwyg[contenteditable=\"true\"]").ok().flatten()
                        .and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok())
                    {
                        let _ = el.focus();
                    }
                }
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
                open_dialog={open_dialog.clone()}
                on_git_changed={on_git_changed}
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
                on_open_cheatsheet={{
                    let cheatsheet_open = cheatsheet_open.clone();
                    Callback::from(move |_: ()| cheatsheet_open.set(true))
                }}
                on_close_vault={on_close_vault}
                on_open_vault={on_open_vault_shortcut}
            />
            if *nav_mode_active {
                <span class="nav-mode-badge" style={format!("--nav-mode-depth-color: {};", crate::nav_mode::depth_color_css(nav_stack.len()))}>
                    { if nav_stack.is_empty() {
                        "-- NAV: Regiões --".to_string()
                    } else {
                        format!("-- NAV: {} --", nav_stack.join(" > "))
                    } }
                </span>
            }
            if vault_open {
                <div class="app-layout">
                    <div class="app-body">
                        <Sidebar
                            vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                            on_page_selected={on_page_selected.clone()}
                            list_version={*list_version}
                            collapsed={*sidebar_collapsed}
                            open_dialog={open_dialog.clone()}
                            activate_nav_signal={*sidebar_activate_nav}
                        />
                        <div class="app-main-panel" tabindex="0"
                            data-nav-item="editor" data-nav-parent="root" data-nav-delegate="editor">
                            <TabBar
                                tabs={(*open_tabs).clone()}
                                active_path={selected_page.as_ref().map(|p| p.path.clone())}
                                on_select={on_tab_select}
                                on_close={on_tab_close}
                                home_path={(*home_page).clone()}
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
                                home_page={(*home_page).clone()}
                                on_toggle_home={on_toggle_home.clone()}
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
            if *cheatsheet_open {
                <CheatsheetModal
                    global_keymap={(*global_keymap).clone()}
                    vim_keymap={(*vim_keymap).clone()}
                    vim_mode_enabled={*vim_mode}
                    on_close={{
                        let cheatsheet_open = cheatsheet_open.clone();
                        Callback::from(move |_: ()| cheatsheet_open.set(false))
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
