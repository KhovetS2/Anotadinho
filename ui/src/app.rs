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
    // Foca `.app-root` uma vez, ao montar (ciclo 137 — bug real
    // reportado pelo usuário). Sem isso, o foco do navegador começa
    // em `<body>` até o usuário clicar em ALGO dentro do app — e
    // como eventos de teclado só borbulham pra CIMA (nunca descem de
    // volta pra dentro de um descendente), um Ctrl+tecla disparado
    // com o foco ainda em `<body>` NUNCA alcança o `onkeydown` do
    // `.app-root`, então NENHUM atalho global funciona (nav-mode,
    // paleta, Ctrl+S, etc.) até o primeiro clique. `use_effect_with`
    // com dependência `()` roda uma vez só, no mount deste componente
    // raiz — que só monta uma vez por sessão do app.
    let app_root_ref = use_node_ref();
    {
        let app_root_ref = app_root_ref.clone();
        use_effect_with((), move |_| {
            if let Some(el) = app_root_ref.cast::<web_sys::HtmlElement>() {
                let _ = el.focus();
            }
            || {}
        });
    }
    // Rede de segurança geral (ciclo 138 — o MESMO bug do ciclo 137,
    // mas acontecendo de novo toda vez que um overlay fecha, não só
    // no boot). `Modal`/`CommandPalette`/`CheatsheetModal`/
    // `VimSettingsModal`/`GlobalKeymapModal`/os modais locais de
    // Propriedades e Histórico do editor desmontam o próprio conteúdo
    // ao fechar (Escape, X, clique fora) sem devolver o foco pra
    // lugar nenhum específico — quando o elemento QUE TINHA foco é
    // removido do DOM, o navegador reseta o foco pro `<body>`. Como
    // `<body>` é ancestral de `.app-root`, isso trava TODOS os
    // atalhos globais de novo, do mesmo jeito que o boot travava.
    //
    // Tentei primeiro um listener de `focusout` na `window` (mais
    // "elegante", só reage quando precisa) — não funcionou de forma
    // confiável: quando o elemento focado é REMOVIDO do DOM (em vez
    // de perder foco por Tab/clique normal), motores diferem em
    // disparar `focusout` ou não, e o WebKitGTK usado aqui
    // aparentemente não dispara nesse caso específico (confirmado ao
    // vivo). Troquei por um polling leve — menos elegante, mas robusto
    // contra QUALQUER jeito de um overlay futuro perder o foco sem
    // precisar lembrar de tratar caso a caso (mesmo padrão de
    // `gloo_timers::callback::Interval` já usado mais abaixo nesta
    // função pra outra coisa).
    {
        let app_root_ref = app_root_ref.clone();
        use_effect_with((), move |_| {
            let interval = gloo_timers::callback::Interval::new(300, move || {
                let fell_to_body = web_sys::window()
                    .and_then(|w| w.document())
                    .and_then(|d| d.active_element())
                    .map(|el| el.tag_name().eq_ignore_ascii_case("body"))
                    .unwrap_or(false);
                if fell_to_body {
                    if let Some(el) = app_root_ref.cast::<web_sys::HtmlElement>() {
                        let _ = el.focus();
                    }
                }
            });
            move || drop(interval)
        });
    }
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
                } else {
                    // Sessão terminou (delegate ou Escape na raiz) —
                    // limpa o indicador do ÚLTIMO item focado (ciclo
                    // 139), que senão ficaria "preso" visualmente já
                    // que ninguém mais chama `focus_item` pra
                    // substituí-lo.
                    crate::nav_mode::clear_item_highlight();
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
    // Consulta com que a paleta abre. Só a ação `run-search` do embed de
    // ações (ciclo 156) preenche isso; Ctrl+K continua abrindo vazia.
    let palette_query = use_state(String::new);
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
                // Contador FORA do handle de estado: `list_version` foi
                // capturado quando o intervalo foi criado, então
                // `*list_version + 1` devolve sempre o mesmo número —
                // o watcher avisava UMA vez por sessão e depois nunca
                // mais (mesmo modo de falha do `edited_ref` no editor).
                // Achado pelo harness do ciclo 177, que testa a recarga
                // automática do 173.
                let tick = std::rc::Rc::new(std::cell::Cell::new(0u32));
                let iv = gloo_timers::callback::Interval::new(3000, move || {
                    let path = path.clone();
                    let list_version = list_version.clone();
                    let tick = tick.clone();
                    let git_files = git_files.clone();
                    let git_busy = git_busy.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(true) = api::check_changes(&path).await {
                            tick.set(tick.get().wrapping_add(1));
                            list_version.set(tick.get());
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
                        Some(tpl) => api::create_page_from_template(&vault, &tpl, &title, None).await,
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
        let palette_query_kb = palette_query.clone();
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
                        // Ciclo 140: agora TAMBÉM exige `focus_is_nav_tracked`
                        // (antes só Enter/Backspace/Escape exigiam, setas se
                        // "autocuravam" voltando pro item 0). Achado real: um
                        // delegate (sidebar) desativa a sessão, mas se o
                        // delegate alvo não isolar a própria seta (ver
                        // `sidebar.rs::on_nav_keydown`, ciclo 140), o evento
                        // ainda bolha até aqui — sem essa guarda, "autocurar"
                        // significava REINICIAR uma sessão nova no meio da
                        // navegação de outra coisa (bug reportado pelo
                        // usuário: "volta pra navegação entre as maiores
                        // sessões"). A recuperação de foco perdido em
                        // `<body>` já é feita de forma mais geral pelo
                        // polling do ciclo 138, então não precisa mais desse
                        // autocuro aqui.
                        "ArrowDown" | "ArrowRight" | "ArrowUp" | "ArrowLeft" => {
                            if !focus_is_nav_tracked {
                                return;
                            }
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
                                    } else if active.has_attribute(crate::nav_mode::ATTR_BLOCO_TEXTO) {
                                        // Bloco de TEXTO (ciclo 174): Enter põe o
                                        // cursor dentro dele e encerra a sessão —
                                        // daqui pra frente é digitação normal.
                                        crate::nav_mode::clear_item_highlight();
                                        crate::components::editor::entrar_no_bloco(&active);
                                        nav_mode_active.set(false);
                                        nav_stack.set(Vec::new());
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
                            let leaving = stack.pop();
                            let new_group = stack.last().cloned().unwrap_or_else(|| "root".to_string());
                            nav_stack.set(stack);
                            if let Some(doc) = doc {
                                // Saindo de um embed: o foco volta pro
                                // TEXTO do editor, não pro topo do app —
                                // quem entrou num embed estava escrevendo
                                // (ciclo 165).
                                if leaving.as_deref().is_some_and(|g| g.starts_with("embed-"))
                                    && new_group == "root"
                                {
                                    focus_editor_text(&doc);
                                    nav_mode_active.set(false);
                                    return;
                                }
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
                            } else if nav_stack.last().is_some_and(|g| g.starts_with("embed-")) {
                                // Sai do embed pro nível dos BLOCOS
                                // (ciclo 174), com o próprio embed
                                // destacado — antes voltava direto pro
                                // texto, o que perdia o lugar.
                                let grupo = nav_stack.last().cloned().unwrap_or_default();
                                nav_stack.set(vec![crate::nav_mode::GRUPO_BLOCOS.to_string()]);
                                if let Some(doc) = doc {
                                    if let Ok(Some(el)) = doc.query_selector(&format!(
                                        "[data-nav-item=\"{}\"]",
                                        grupo.replace('"', "")
                                    )) {
                                        crate::nav_mode::focus_item(&el);
                                    }
                                }
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
                // Limpa a consulta: sem isso, um Ctrl+K depois de uma
                // ação `run-search` reabriria com o termo anterior.
                palette_query_kb.set(String::new());
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
            } else if matches(&km.next_embed) || matches(&km.prev_embed) {
                // Salta pro embed seguinte/anterior da página e ABRE uma
                // sessão de nav-mode dentro dele (ciclo 165). Antes disso
                // os embeds só eram alcançáveis por Tab, um botão por
                // vez: o item de topo `editor` é um delegate, então o
                // motor de navegação nunca descia neles.
                e.prevent_default();
                let forward = matches(&km.next_embed);
                if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                    if let Some(group_id) = adjacent_embed_group(&doc, forward) {
                        let items = crate::nav_mode::items_in_group(&doc, &group_id);
                        if let Some(first) = items.first() {
                            crate::nav_mode::focus_item(first);
                            nav_mode_active.set(true);
                            nav_stack.set(vec![group_id]);
                        }
                    }
                }
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
        <div class="app-root" tabindex="0" ref={app_root_ref} {onkeydown}>
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
                            data-nav-item="editor" data-nav-parent="root"
                            data-nav-group="editor-blocos">
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
                                vault_version={*list_version}
                                on_enter_block_nav={{
                                    let nav_mode_active = nav_mode_active.clone();
                                    let nav_stack = nav_stack.clone();
                                    Callback::from(move |_: ()| {
                                        nav_mode_active.set(true);
                                        nav_stack.set(vec![crate::nav_mode::GRUPO_BLOCOS.to_string()]);
                                    })
                                }}
                                on_search={{
                                    let palette_open = palette_open.clone();
                                    let palette_query = palette_query.clone();
                                    Callback::from(move |q: String| {
                                        palette_query.set(q);
                                        palette_open.set(true);
                                    })
                                }}
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
                    initial_query={(*palette_query).clone()}
                />
            }
        </div>
    }
}

/// Id do grupo de nav-mode do embed seguinte (ou anterior) ao ponto
/// onde o foco/cursor está agora — `None` se a página não tem embed
/// nenhum.
///
/// Os embeds da página se anunciam com `data-nav-group="embed-<i>"`
/// (o `<i>` é o índice do SEGMENTO, gerado pelo editor: dois embeds do
/// mesmo tipo não podem cair no mesmo grupo). A ordem usada é a do
/// documento, e a posição atual sai de `compare_document_position` —
/// assim o salto respeita onde o cursor está no texto, não só qual
/// botão foi focado por último.
fn adjacent_embed_group(doc: &web_sys::Document, forward: bool) -> Option<String> {
    const DOCUMENT_POSITION_FOLLOWING: u16 = 4;
    let Ok(list) = doc.query_selector_all("[data-nav-group^=\"embed-\"]") else { return None };
    let mut groups: Vec<web_sys::Element> = Vec::new();
    for i in 0..list.length() {
        if let Some(el) = list.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
            groups.push(el);
        }
    }
    if groups.is_empty() {
        return None;
    }

    // Âncora: o elemento focado, ou o começo do documento se não houver.
    let anchor = doc.active_element();
    let index_of_anchor = anchor.as_ref().and_then(|a| {
        groups.iter().position(|g| g.contains(Some(a)) || g.is_same_node(Some(a)))
    });

    let target = match index_of_anchor {
        // Já está dentro de um embed: vai pro vizinho, com wrap-around.
        Some(i) => {
            if forward {
                (i + 1) % groups.len()
            } else {
                (i + groups.len() - 1) % groups.len()
            }
        }
        // Está no texto: pega o primeiro embed depois (ou o último
        // antes) da posição do cursor no documento.
        None => {
            let after = anchor.as_ref().map(|a| {
                groups
                    .iter()
                    .position(|g| a.compare_document_position(g) & DOCUMENT_POSITION_FOLLOWING != 0)
            });
            match (forward, after) {
                (true, Some(Some(i))) => i,
                (true, _) => 0,
                (false, Some(Some(i))) if i > 0 => i - 1,
                (false, Some(Some(_))) => groups.len() - 1,
                (false, _) => groups.len() - 1,
            }
        }
    };
    groups.get(target).and_then(|el| el.get_attribute("data-nav-group"))
}

/// Devolve o foco pro texto do editor (o mesmo alvo do "focar editor"
/// do keymap). Usado ao sair de um embed pelo teclado.
fn focus_editor_text(doc: &web_sys::Document) {
    crate::nav_mode::clear_item_highlight();
    if let Some(el) = doc
        .query_selector(".editor__wysiwyg[contenteditable=\"true\"]")
        .ok()
        .flatten()
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}
