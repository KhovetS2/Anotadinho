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
use crate::components::tab_bar::{OpenTab, TabBar, TabFlag};
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

/// Recalcula as flags derivadas e promove a home sem alterar a ordem
/// relativa das outras abas. Toda mutação de `open_tabs` passa por aqui.
fn organize_tabs(mut tabs: Vec<OpenTab>, home_path: Option<&str>) -> Vec<OpenTab> {
    for tab in &mut tabs {
        tab.flags.retain(|flag| *flag != TabFlag::Home);
        if home_path == Some(tab.page.path.as_str()) {
            tab.flags.push(TabFlag::Home);
        }
    }
    if let Some(pos) = tabs.iter().position(|tab| tab.has_flag(TabFlag::Home)) {
        let home = tabs.remove(pos);
        tabs.insert(0, home);
    }
    tabs
}

#[cfg(test)]
mod tab_tests {
    use super::*;

    fn tab(path: &str) -> OpenTab {
        OpenTab::new(PageMeta {
            path: path.into(),
            title: path.into(),
            section: "pages".into(),
        })
    }

    #[test]
    fn home_vai_para_o_inicio_sem_reordenar_as_demais() {
        let tabs = organize_tabs(vec![tab("a"), tab("home"), tab("b")], Some("home"));
        let paths: Vec<_> = tabs.iter().map(|tab| tab.page.path.as_str()).collect();
        assert_eq!(paths, vec!["home", "a", "b"]);
        assert!(tabs[0].has_flag(TabFlag::Home));
        assert!(!tabs[1].has_flag(TabFlag::Home));
    }

    #[test]
    fn sem_home_preserva_a_ordem_e_remove_a_flag() {
        let mut home = tab("home");
        home.flags.push(TabFlag::Home);
        let tabs = organize_tabs(vec![tab("a"), home], None);
        assert_eq!(tabs[0].page.path, "a");
        assert!(tabs.iter().all(|tab| !tab.has_flag(TabFlag::Home)));
    }
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
    // Página aberta ANTES da atual — é o contexto que a conversa manda
    // junto (ciclo 202). Quem sabe a ordem de navegação é quem troca de
    // página, então isto mora aqui e não dentro do painel.
    // Rede de segurança contra o arrasto solto fora de lugar (ciclo 245).
    //
    // Um `drop` que ninguém trata tem comportamento padrão: o webview
    // NAVEGA para o arquivo solto. A página do app é substituída, e a
    // janela fica em branco pra sempre — não há como voltar, porque o
    // que sumiu foi a própria aplicação.
    //
    // Aconteceu de verdade: arrastar uma imagem e soltar um pouco fora da
    // área do editor derrubava o app. Só passou a ser possível quando o
    // ciclo 242 devolveu o arrasto nativo ao webview (`dragDropEnabled:
    // false`) — antes o Tauri engolia tudo, inclusive isto.
    //
    // O editor continua tratando o que cai nele; isto aqui só garante que
    // o que cai FORA não faça nada.
    use_effect_with((), |_| {
        let doc = web_sys::window().and_then(|w| w.document());
        let engolir = wasm_bindgen::closure::Closure::<dyn Fn(web_sys::DragEvent)>::new(
            |e: web_sys::DragEvent| e.prevent_default(),
        );
        if let Some(doc) = &doc {
            for evento in ["dragover", "drop"] {
                let _ = doc
                    .add_event_listener_with_callback(evento, engolir.as_ref().unchecked_ref());
            }
        }
        move || {
            if let Some(doc) = doc {
                for evento in ["dragover", "drop"] {
                    let _ = doc.remove_event_listener_with_callback(
                        evento,
                        engolir.as_ref().unchecked_ref(),
                    );
                }
            }
            drop(engolir);
        }
    });

    let pagina_anterior = use_state(|| None::<String>);
    // Pergunta que uma conversa deve trazer já escrita (ciclo 209) — é
    // como o botão "Planejar implementação" entrega o pedido pronto sem a
    // pessoa redigitar. Guarda o caminho de destino e é zerada assim que
    // a conversa a consome (ciclo 227), senão sobra aqui e reaparece na
    // próxima conversa que a pessoa abrir.
    let pergunta_inicial = use_state(|| None::<crate::components::conversa_view::PerguntaInicial>);
    {
        let pagina_anterior = pagina_anterior.clone();
        let selected_page = selected_page.clone();
        // `use_mut_ref` e não `use_state` pro caminho corrente: handle de
        // `use_state` capturado em efeito congela no valor de criação —
        // o bug que já apareceu nos ciclos 155, 157, 201 e 202.
        let caminho_atual = use_mut_ref(|| None::<String>);
        use_effect_with((*selected_page).clone(), move |atual| {
            let novo = atual.as_ref().map(|p| p.path.clone());
            let anterior = caminho_atual.borrow().clone();
            if novo != anterior {
                if let Some(a) = anterior {
                    pagina_anterior.set(Some(a));
                }
                *caminho_atual.borrow_mut() = novo;
            }
            || ()
        });
    }

    let list_version = use_state(|| 0u32);
    let git_files = use_state(|| None::<Vec<api::GitFileEntry>>);
    /// Quantas propostas aguardam revisão (ciclo 210).
    let propostas_pendentes = use_state(|| 0usize);
    {
        // Reconsulta quando o vault muda — o agente pode propor a
        // qualquer momento, inclusive pelo CLI ou pelo MCP, e nesses
        // casos não há nenhum evento de UI pra reagir.
        let propostas_pendentes = propostas_pendentes.clone();
        let vault_path = vault_path.clone();
        use_effect_with((*list_version, (*vault_path).clone()), move |_| {
            let Some(v) = (*vault_path).clone() else { return };
            wasm_bindgen_futures::spawn_local(async move {
                let n = api::listar_propostas(&v).await.map(|l| l.len()).unwrap_or(0);
                propostas_pendentes.set(n);
            });
        });
    }

    let sidebar_collapsed = use_state(|| false);
    let open_tabs = use_state(Vec::<OpenTab>::new);
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
    {
        let open_tabs = open_tabs.clone();
        use_effect_with((*home_page).clone(), move |home| {
            let organized = organize_tabs((*open_tabs).clone(), home.as_deref());
            if organized != *open_tabs {
                open_tabs.set(organized);
            }
            || {}
        });
    }
    let on_toggle_home = {
        let vault_path = vault_path.clone();
        let home_page = home_page.clone();
        let open_tabs = open_tabs.clone();
        Callback::from(move |path: String| {
            let Some(ref vault) = *vault_path else { return };
            if home_page.as_deref() == Some(path.as_str()) {
                state::clear_home_page(vault);
                home_page.set(None);
                open_tabs.set(organize_tabs((*open_tabs).clone(), None));
            } else {
                state::save_home_page(vault, &path);
                open_tabs.set(organize_tabs((*open_tabs).clone(), Some(&path)));
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
    // Espelho do `nav_mode_active` pra ser LIDO de dentro de efeito
    // (ciclos 155, 157, 201, 213, 218): o handle de `use_state` capturado
    // numa closure congela no valor de quando ela foi criada. Aqui o
    // efeito depende da PÁGINA, não do modo — se o modo entrasse nas
    // dependências, ligar/desligar navegação reancoraria a pilha sozinho.
    let nav_mode_active_ref = use_mut_ref(|| false);
    {
        let nav_mode_active_ref = nav_mode_active_ref.clone();
        use_effect_with(*nav_mode_active, move |ativo| {
            *nav_mode_active_ref.borrow_mut() = *ativo;
            || ()
        });
    }

    // Abrir uma página REANCORA a sessão nos blocos dela (RF1 da spec de
    // teclado, ciclo 250).
    //
    // O caminho relatado: home → "Trabalho recente" → Enter num card → a
    // página abre. A pilha continuava apontando pro grupo do embed, que
    // não existe mais na página nova; a próxima seta caía no resgate de
    // `reancorar_se_perdido`, cujo último recurso é a raiz — e a raiz
    // começa na barra superior. Daí o teclado terminar preso lá em cima,
    // longe do que a pessoa acabou de abrir.
    //
    // Agora a pilha passa a ser exatamente um nível: os blocos da página
    // aberta. Escape dali sobe pra raiz, um nível, como manda o RF2.
    {
        let nav_mode_active_ref = nav_mode_active_ref.clone();
        let nav_stack = nav_stack.clone();
        let caminho = selected_page.as_ref().map(|p: &api::PageMeta| p.path.clone());
        use_effect_with(caminho, move |caminho| {
            if caminho.is_some() && *nav_mode_active_ref.borrow() {
                nav_stack.set(vec![crate::nav_mode::GRUPO_BLOCOS.to_string()]);
                // O conteúdo chega assíncrono; o helper tenta até achar.
                crate::nav_mode::focar_blocos_da_pagina(|_| {});
            }
            || ()
        });
    }
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

    let open_dialog: Callback<PendingDialog> = {
        let pending_dialog = pending_dialog.clone();
        Callback::from(move |d: PendingDialog| pending_dialog.set(Some(d)))
    };

    let on_vault_selected = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        let list_version = list_version.clone();
        let abrir_dialogo = open_dialog.clone();
        Callback::from(move |path: String| {
            let name = state::extract_name_from_path(&path);
            state::save_vault_path(&path);
            state::save_vault_name(&name);
            vault_path.set(Some(path.clone()));
            vault_name.set(Some(name));
            selected_page.set(None);
            open_tabs.set(Vec::new());

            // Abrir uma pasta sem página nenhuma dá um app mudo: sem
            // sidebar, sem template, sem sinal do que fazer. Em vez de
            // deixar a pessoa adivinhar, OFERECE semear (ciclo 233) —
            // oferece, não faz: encher a pasta de alguém com dezessete
            // arquivos sem perguntar não é a mesma coisa que ajudar.
            let list_version = list_version.clone();
            let abrir = abrir_dialogo.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if !api::vault_esta_vazio(&path).await.unwrap_or(false) {
                    return;
                }
                abrir.emit(PendingDialog::Confirm {
                    message: "Esta pasta ainda não tem nada. Preparar como vault, \
                              com estrutura, modelos, padrões e um guia?"
                        .to_string(),
                    confirm_label: "Preparar".to_string(),
                    on_confirm: Callback::from(move |_| {
                        let (path, list_version) = (path.clone(), list_version.clone());
                        wasm_bindgen_futures::spawn_local(async move {
                            if api::criar_vault(&path).await.is_ok() {
                                list_version.set(*list_version + 1);
                            }
                        });
                    }),
                });
            });
        })
    };

    let on_page_selected = {
        let selected_page = selected_page.clone();
        let open_tabs = open_tabs.clone();
        let home_page = home_page.clone();
        Callback::from(move |page: PageMeta| {
            // Add to tabs if not already there
            let mut tabs = (*open_tabs).clone();
            if !tabs.iter().any(|t| t.page.path == page.path) {
                tabs.push(OpenTab::new(page.clone()));
            }
            open_tabs.set(organize_tabs(tabs, home_page.as_deref()));
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
        Callback::from(move |apagada: String| {
            let mut tabs = (*open_tabs).clone();
            tabs.retain(|t| t.page.path != apagada);
            open_tabs.set(tabs);
            // Só sai da tela se era ESTA a página aberta. Apagar uma
            // conversa velha pela sidebar não pode fechar o que a pessoa
            // está lendo.
            if selected_page.as_ref().is_some_and(|p| p.path == apagada) {
                selected_page.set(None);
            }
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

    // Apagar a página ABERTA (ciclo 232). A sidebar cobre qualquer
    // página; isto cobre "estou aqui e não quero mais isto", que é o
    // caso da conversa criada por engano.
    let excluir_pagina_aberta = {
        let vault_path = vault_path.clone();
        let selected_page = selected_page.clone();
        let open_dialog = open_dialog.clone();
        let on_page_deleted = on_page_deleted.clone();
        Callback::from(move |_: ()| {
            let (Some(vault), Some(pagina)) = ((*vault_path).clone(), (*selected_page).clone())
            else {
                return;
            };
            let on_page_deleted = on_page_deleted.clone();
            open_dialog.emit(PendingDialog::Confirm {
                message: format!(
                    "Excluir \"{}\"? O vault está no git, então dá pra recuperar por lá.",
                    pagina.title
                ),
                confirm_label: "Excluir".to_string(),
                on_confirm: Callback::from(move |_| {
                    let (vault, caminho) = (vault.clone(), pagina.path.clone());
                    let on_page_deleted = on_page_deleted.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::delete_page(&vault, &caminho).await {
                            Ok(_) => on_page_deleted.emit(caminho),
                            Err(e) => web_sys::console::warn_1(
                                &wasm_bindgen::JsValue::from_str(&e),
                            ),
                        }
                    });
                }),
            });
        })
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
            if idx < tabs.len() && !tabs[idx].has_flag(TabFlag::Home) {
                let closed = tabs.remove(idx);
                if selected_page.as_ref().map_or(false, |p| p.path == closed.page.path) {
                    let next = tabs.get(idx).or_else(|| tabs.get(idx.saturating_sub(1))).map(|tab| tab.page.clone());
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

    // Conversa nova em um passo (ciclo 208).
    //
    // A página ABERTA vai anexada como contexto, e a origem fica gravada
    // no frontmatter — não em memória, pra sobreviver a fechar o app.
    // Era a queixa do ponto 2 da spec aprovada.
    let nova_conversa = {
        let vault_path = vault_path.clone();
        let selected_page = selected_page.clone();
        let on_page_selected = on_page_selected.clone();
        let list_version = list_version.clone();
        Callback::from(move |_: ()| {
            let Some(vault) = (*vault_path).clone() else { return };
            let atual = (*selected_page).clone();
            let on_page_selected = on_page_selected.clone();
            let list_version = list_version.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let carimbo = crate::state::agora_legivel();
                let titulo = format!("Conversa de {carimbo}");
                // A página aberta entra como contexto E como origem —
                // são coisas diferentes: origem é "de onde nasceu",
                // contexto é "o que o modelo deve consultar".
                let anexos: Vec<String> = atual.as_ref().map(|p| vec![p.path.clone()]).unwrap_or_default();
                let md = anotadinho_core::conversa::montar_pagina(
                    &titulo,
                    atual.as_ref().map(|p| p.path.as_str()),
                    &anexos,
                );
                let path = format!(
                    "pages/conversas/{}.md",
                    anotadinho_core::conversa::nome_de_arquivo(&carimbo)
                );
                if api::write_page(&vault, &path, &md).await.is_ok() {
                    list_version.set(*list_version + 1);
                    on_page_selected.emit(PageMeta {
                        path,
                        title: titulo,
                        section: "pages".to_string(),
                    });
                }
            });
        })
    };

    // Planejar a implementação de uma spec aprovada (ciclo 209).
    //
    // Cria a conversa com a spec ANEXADA e a pergunta já escrita — é o
    // ponto em que o trabalho passa do "o quê" pro "como", e é onde se
    // anexam os padrões que a proposta terá que respeitar.
    let planejar_implementacao = {
        let vault_path = vault_path.clone();
        let selected_page = selected_page.clone();
        let on_page_selected = on_page_selected.clone();
        let list_version = list_version.clone();
        let pergunta_inicial = pergunta_inicial.clone();
        Callback::from(move |pedido: anotadinho_core::fluxo::Pedido| {
            let Some(vault) = (*vault_path).clone() else { return };
            let Some(spec) = (*selected_page).clone() else { return };
            let on_page_selected = on_page_selected.clone();
            let list_version = list_version.clone();
            let pergunta_inicial = pergunta_inicial.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let carimbo = crate::state::agora_legivel();
                // Título do FRONTMATTER, não o nome do arquivo: o
                // `PageMeta` vindo da sidebar traz o stem, e usá-lo
                // fazia a conversa se chamar "Planejar:
                // uso-agentico-do-anotadinho". Mesmo defeito do ciclo 196.
                let paginas = api::scan_vault(&vault).await.unwrap_or_default();
                let titulo_spec = paginas
                    .iter()
                    .find(|p| p.path == spec.path)
                    .map(|p| p.title.clone())
                    .filter(|t| !t.trim().is_empty())
                    .unwrap_or_else(|| spec.title.clone());
                // A página pode ser uma SPEC (planejar) ou uma PROPOSTA
                // (executar) — o mesmo botão, perguntas diferentes.
                let e_proposta = spec.path.contains("/propostas/")
                    || paginas
                        .iter()
                        .find(|p| p.path == spec.path)
                        .map(|p| p.page_type == "proposta")
                        .unwrap_or(false);
                let alterando = pedido == anotadinho_core::fluxo::Pedido::Alterar;
                let artefato = if e_proposta {
                    anotadinho_core::fluxo::Artefato::Proposta
                } else {
                    anotadinho_core::fluxo::Artefato::Spec
                };
                let titulo = if alterando {
                    format!("Alterar: {titulo_spec}")
                } else if e_proposta {
                    format!("Executar: {titulo_spec}")
                } else {
                    format!("Planejar: {titulo_spec}")
                };
                let pergunta = if alterando {
                    anotadinho_core::fluxo::pergunta_de_alteracao(&titulo_spec, artefato)
                } else if e_proposta {
                    anotadinho_core::fluxo::pergunta_de_execucao(&titulo_spec, &spec.path)
                } else {
                    anotadinho_core::fluxo::pergunta_de_planejamento(&titulo_spec)
                };
                // Executar CONTINUA na conversa que gerou a proposta.
                //
                // A proposta guarda o caminho dela em `origem` (é o
                // `promover` que grava isso). Abrir uma conversa nova a
                // cada execução espalhava o histórico do trabalho: a
                // discussão que produziu a proposta numa página, o que
                // o agente fez pra executá-la noutra, sem ligação
                // visível entre as duas.
                //
                // Só vale pra proposta com origem viva. Uma proposta
                // escrita à mão, ou cuja conversa foi apagada, ganha
                // conversa nova como antes.
                let continuacao = if e_proposta {
                    match api::read_page(&vault, &spec.path).await {
                        Ok(conteudo) => {
                            let (_, corpo) =
                                anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
                            anotadinho_core::fluxo::origem_da_pagina(corpo)
                                .filter(|o| paginas.iter().any(|p| p.path == *o))
                                .filter(|o| {
                                    paginas
                                        .iter()
                                        .find(|p| p.path == *o)
                                        .map(|p| p.page_type == "conversa")
                                        .unwrap_or(false)
                                })
                        }
                        Err(_) => None,
                    }
                } else {
                    None
                };

                if let Some(conversa) = continuacao {
                    let titulo_conversa = paginas
                        .iter()
                        .find(|p| p.path == conversa)
                        .map(|p| p.title.clone())
                        .unwrap_or_else(|| titulo.clone());
                    pergunta_inicial.set(Some(
                        crate::components::conversa_view::PerguntaInicial {
                            conversa: conversa.clone(),
                            texto: pergunta.clone(),
                        },
                    ));
                    on_page_selected.emit(PageMeta {
                        path: conversa,
                        title: titulo_conversa,
                        section: "pages".to_string(),
                    });
                    return;
                }

                let md = anotadinho_core::conversa::montar_pagina(
                    &titulo,
                    Some(&spec.path),
                    &[spec.path.clone()],
                );
                let path = format!(
                    "pages/conversas/{}.md",
                    anotadinho_core::conversa::nome_de_arquivo(&carimbo)
                );
                if api::write_page(&vault, &path, &md).await.is_ok() {
                    pergunta_inicial.set(Some(
                        crate::components::conversa_view::PerguntaInicial {
                            conversa: path.clone(),
                            texto: pergunta,
                        },
                    ));
                    list_version.set(*list_version + 1);
                    on_page_selected.emit(PageMeta {
                        path,
                        title: titulo,
                        section: "pages".to_string(),
                    });
                }
            });
        })
    };

    // Abre a tela de revisão. Cria a página se ela não existir — o
    // aviso não pode levar a lugar nenhum (ciclo 210).
    let abrir_propostas = {
        let vault_path = vault_path.clone();
        let on_page_selected = on_page_selected.clone();
        let list_version = list_version.clone();
        Callback::from(move |_: ()| {
            let Some(vault) = (*vault_path).clone() else { return };
            let on_page_selected = on_page_selected.clone();
            let list_version = list_version.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let path = "pages/propostas.md".to_string();
                if api::read_page(&vault, &path).await.is_err() {
                    let md = "---\ntitle: Propostas\ntype: propostas\ntags:\n- agent-os\n---\n";
                    let _ = api::write_page(&vault, &path, md).await;
                    list_version.set(*list_version + 1);
                }
                on_page_selected.emit(PageMeta {
                    path,
                    title: "Propostas".to_string(),
                    section: "pages".to_string(),
                });
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
            PaletteAction::NovaConversa => nova_conversa.emit(()),
            PaletteAction::ExcluirPaginaAberta => excluir_pagina_aberta.emit(()),
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
                        // `hjkl` entram aqui junto das setas (RF3 da
                        // spec de teclado, ciclo 250). Só minúsculas:
                        // `J`/`K` já MOVEM o bloco, que é outra ação.
                        _ if crate::nav_mode::direcao_de_navegacao(&key).is_some() => {
                            if !focus_is_nav_tracked {
                                // Antes de desistir, tenta reancorar
                                // (ciclo 197) — mas SÓ se o foco não for
                                // de ninguém. Com o foco num campo ou
                                // num delegate, a seta é deles.
                                let reancorou = doc.as_ref().is_some_and(|d| {
                                    let grupo = nav_stack.last().cloned().unwrap_or_else(|| "root".to_string());
                                    crate::nav_mode::reancorar_se_perdido(d, &grupo)
                                });
                                if !reancorou {
                                    return;
                                }
                                e.prevent_default();
                                return;
                            }
                            e.prevent_default();
                            if let Some(doc) = doc {
                                let group_id = nav_stack.last().cloned().unwrap_or_else(|| "root".to_string());
                                let items = crate::nav_mode::items_in_group(&doc, &group_id);
                                if !items.is_empty() {
                                    let active = doc.active_element();
                                    let idx = crate::nav_mode::index_of(&items, active.as_ref());
                                    let forward =
                                        crate::nav_mode::direcao_de_navegacao(&key).unwrap_or(true);
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
                                                    .query_selector(".editor__bloco[contenteditable=\"true\"]")
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
                                // Sobe UM nível, não a pilha inteira (RF2,
                                // ciclo 250). Antes isto voltava direto pra
                                // raiz de qualquer profundidade: descer três
                                // níveis e dar Escape jogava a pessoa pro
                                // topo, e o único jeito de subir de um em um
                                // era Backspace. Agora os dois sobem um
                                // nível; a diferença é que Escape na raiz
                                // ENCERRA a sessão, e Backspace não.
                                let mut stack = (*nav_stack).clone();
                                let saindo = stack.pop();
                                let novo = stack.last().cloned().unwrap_or_else(|| "root".to_string());
                                nav_stack.set(stack);
                                if let Some(doc) = doc {
                                    // Volta pro item de ONDE se saiu, não pro
                                    // primeiro do nível de cima: subir devia
                                    // devolver a pessoa ao lugar em que ela
                                    // estava, e não jogá-la no começo da lista.
                                    // O item que representa um grupo é o que
                                    // tem `data-nav-group` igual a ele — a
                                    // mesma relação que o Enter usou pra
                                    // descer.
                                    let de_volta = saindo.as_ref().and_then(|g| {
                                        doc.query_selector(&format!(
                                            "[data-nav-group=\"{}\"]",
                                            g.replace('"', "")
                                        ))
                                        .ok()
                                        .flatten()
                                    });
                                    let alvo = de_volta.or_else(|| {
                                        crate::nav_mode::items_in_group(&doc, &novo)
                                            .into_iter()
                                            .next()
                                    });
                                    if let Some(el) = alvo {
                                        crate::nav_mode::focus_item(&el);
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
                        selected_page.set(Some(tab.page.clone()));
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
                        let pos = tabs.iter().position(|t| t.page.path == sel.path).unwrap_or(0);
                        let next = (pos + 1) % tabs.len();
                        selected_page.set(Some(tabs[next].page.clone()));
                    } else {
                        selected_page.set(Some(tabs[0].page.clone()));
                    }
                }
            } else if matches(&km.prev_tab) {
                e.prevent_default();
                let tabs = (*open_tabs).clone();
                if !tabs.is_empty() {
                    if let Some(ref sel) = *selected_page {
                        let pos = tabs.iter().position(|t| t.page.path == sel.path).unwrap_or(0);
                        let prev = (pos + tabs.len() - 1) % tabs.len();
                        selected_page.set(Some(tabs[prev].page.clone()));
                    } else {
                        selected_page.set(Some(tabs[0].page.clone()));
                    }
                }
            } else if matches(&km.close_tab) {
                e.prevent_default();
                let tabs = (*open_tabs).clone();
                if let Some(ref sel) = *selected_page {
                    if let Some(pos) = tabs.iter().position(|t| t.page.path == sel.path) {
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
                    if let Some(el) = doc.query_selector(".editor__bloco[contenteditable=\"true\"]").ok().flatten()
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
            // Sem a decoração do sistema (ciclo 180) não existe borda
            // pra pegar e redimensionar. Estas faixas invisíveis nas
            // 8 direções devolvem isso: o `mousedown` entrega o arraste
            // pro compositor, que assume dali em diante.
            { for ["n", "s", "w", "e", "nw", "ne", "sw", "se"].iter().map(|dir| {
                let dir = *dir;
                let onmousedown = Callback::from(move |e: MouseEvent| {
                    e.prevent_default();
                    wasm_bindgen_futures::spawn_local(async move {
                        let _ = api::window_start_resize(dir).await;
                    });
                });
                html! { <div class={classes!("window-resize", format!("window-resize--{dir}"))} {onmousedown} /> }
            }) }
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
                propostas_pendentes={*propostas_pendentes}
                on_abrir_propostas={abrir_propostas.clone()}
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
            // O balão flutuante de nav-mode saiu no ciclo 195: a barra de
            // modo do rodapé (ciclo 194) diz a mesma coisa sem cobrir
            // conteúdo nem competir com o que a pessoa está lendo.
            if vault_open {
                <div class="app-layout">
                    <div class="app-body">
                        <Sidebar
                            vault_path={vault_path.as_ref().cloned().unwrap_or_default()}
                            on_page_selected={on_page_selected.clone()}
                            list_version={*list_version}
                            collapsed={*sidebar_collapsed}
                            open_dialog={open_dialog.clone()}
                            on_page_deleted={on_page_deleted.clone()}
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
                                nav_mode_active={*nav_mode_active}
                                on_planejar={planejar_implementacao.clone()}
                                on_fila_mudou={{
                                    let list_version = list_version.clone();
                                    Callback::from(move |_: ()| list_version.set(*list_version + 1))
                                }}
                                contexto_path={(*pagina_anterior).clone()}
                                pergunta_inicial={(*pergunta_inicial).clone()}
                                on_pergunta_consumida={{
                                    let p = pergunta_inicial.clone();
                                    Callback::from(move |_| p.set(None))
                                }}
                                on_enter_block_nav={{
                                    let nav_mode_active = nav_mode_active.clone();
                                    let nav_stack = nav_stack.clone();
                                    Callback::from(move |_: ()| {
                                        nav_mode_active.set(true);
                                        nav_stack.set(vec![crate::nav_mode::GRUPO_BLOCOS.to_string()]);
                                    })
                                }}
                                on_leave_block_nav={{
                                    let nav_mode_active = nav_mode_active.clone();
                                    let nav_stack = nav_stack.clone();
                                    Callback::from(move |_: ()| {
                                        nav_mode_active.set(false);
                                        nav_stack.set(Vec::new());
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
        .query_selector(".editor__bloco[contenteditable=\"true\"]")
        .ok()
        .flatten()
        .and_then(|el| el.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = el.focus();
    }
}
