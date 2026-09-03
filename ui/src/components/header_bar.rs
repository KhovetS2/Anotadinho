//! Header bar.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::api::{self, GitFileEntry};
use crate::components::icon::Icon;
use crate::dialog::PendingDialog;

#[derive(Properties, PartialEq, Clone)]
pub struct HeaderBarProps {
    pub vault_name: Option<String>,
    pub vault_path: Option<String>,
    pub sidebar_collapsed: bool,
    pub theme_light: bool,
    pub autosave_enabled: bool,
    pub vim_mode_enabled: bool,
    /// `None` = vault não é um repositório git (ou `git` não
    /// instalado) — indicador simplesmente não aparece. `Some(vec![])`
    /// = repo git limpo, sem mudanças.
    pub git_files: Option<Vec<GitFileEntry>>,
    /// Abre o modal de diálogo do app (ver `crate::dialog`) — usado
    /// pra pedir a mensagem de commit e mostrar erro de pull/push.
    pub open_dialog: Callback<PendingDialog>,
    /// Disparado depois de um pull/commit+push bem-sucedido, pra quem
    /// possui o estado de `git_files`/lista de páginas re-buscar
    /// (ciclo 119).
    #[prop_or_default]
    pub on_git_changed: Callback<()>,
    /// Quantas propostas do agente aguardam revisão (ciclo 210).
    ///
    /// Fica no cabeçalho porque proposta pendente é assunto de QUALQUER
    /// página: sem isso o agente propõe e ninguém vê até abrir a tela de
    /// propostas por acaso.
    #[prop_or_default]
    pub propostas_pendentes: usize,
    /// Abre a tela de revisão.
    #[prop_or_default]
    pub on_abrir_propostas: Callback<()>,
    pub on_toggle_sidebar: Callback<()>,
    pub on_toggle_theme: Callback<()>,
    /// Abre a tela de aparência (ciclo 253) — tema, cor de destaque e
    /// forma dos botões. O alternador claro/escuro continua ao lado: ele
    /// é o atalho de um clique, isto é a escolha completa.
    pub on_open_aparencia: Callback<()>,
    pub on_toggle_autosave: Callback<()>,
    pub on_toggle_vim_mode: Callback<()>,
    pub on_open_vim_settings: Callback<()>,
    pub on_open_global_keymap_settings: Callback<()>,
    pub on_open_cheatsheet: Callback<()>,
    pub on_close_vault: Callback<()>,
    pub on_open_vault: Callback<()>,
}

#[function_component(HeaderBar)]
pub fn header_bar(props: &HeaderBarProps) -> Html {
    // Controles da janela (ciclo 180): a barra do sistema saiu
    // (`decorations: false`), então minimizar/maximizar/fechar vivem
    // aqui. O estado de maximizada é consultado na montagem porque o
    // gerenciador de janelas pode abrir o app já maximizado.
    let maximizada = use_state(|| false);
    {
        let maximizada = maximizada.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(v) = crate::api::window_is_maximized().await {
                    maximizada.set(v);
                }
            });
            || ()
        });
    }

    let menu_open = use_state(|| false);
    let menu_ref = use_node_ref();
    // Ref à parte pro CONTEÚDO do menu (não o wrapper, que também tem
    // o botão "⚙" que abre/fecha) — mesma razão do popover de git
    // logo abaixo.
    let menu_content_ref = use_node_ref();
    // Ref só do BOTÃO que abre/fecha — usado pra devolver o foco pra
    // ele ao fechar via Escape (ciclo 136). Sem isso, o foco cai pro
    // `<body>` (fora da árvore do `.app-root`), e o nav-mode nunca
    // mais recebe eventos de teclado depois — bubbling só sobe, não
    // desce, então um keydown que nasce no `<body>` (ancestral do
    // `.app-root`) não alcança o listener do `.app-root`.
    let menu_toggle_ref = use_node_ref();
    let toggle_menu = { let m = menu_open.clone(); Callback::from(move |_| m.set(!*m)) };

    let git_popover_open = use_state(|| false);
    let git_popover_ref = use_node_ref();
    // Ref à parte pro CONTEÚDO do popover (não o wrapper, que também
    // contém o botão que abre/fecha) — sem isso, "focar o primeiro
    // botão" acharia o próprio botão de abrir, não os itens de dentro.
    let git_popover_content_ref = use_node_ref();
    let git_popover_toggle_ref = use_node_ref();
    let toggle_git_popover = { let p = git_popover_open.clone(); Callback::from(move |_| p.set(!*p)) };

    // Mesmo padrão de fechar ao clicar fora/Escape do menu ⚙ acima —
    // ganha foco automático no primeiro item + setas pra navegar
    // (ciclo 125, mesma técnica em `crate::menu_keyboard`).
    {
        let git_popover_open = git_popover_open.clone();
        let git_popover_ref = git_popover_ref.clone();
        let git_popover_content_ref = git_popover_content_ref.clone();
        let git_popover_toggle_ref = git_popover_toggle_ref.clone();
        use_effect_with(*git_popover_open, move |open| {
            let mut listeners = Vec::new();
            if *open {
                crate::menu_keyboard::focus_first_item(&git_popover_content_ref);
                let window = web_sys::window().expect("no global window");
                let close_on_outside = {
                    let git_popover_open = git_popover_open.clone();
                    let git_popover_ref = git_popover_ref.clone();
                    EventListener::new(&window, "mousedown", move |e| {
                        let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                        if let Some(el) = git_popover_ref.cast::<web_sys::Element>() {
                            if !el.contains(Some(&target)) {
                                git_popover_open.set(false);
                            }
                        }
                    })
                };
                let close_on_escape = {
                    let git_popover_open = git_popover_open.clone();
                    let git_popover_content_ref = git_popover_content_ref.clone();
                    let git_popover_toggle_ref = git_popover_toggle_ref.clone();
                    EventListener::new(&window, "keydown", move |e| {
                        if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                            match e.key().as_str() {
                                // Escape é tratado pelo `escape_consumer`
                                // abaixo (ciclo 161) — aqui ele seguiria
                                // até o `app.rs` e fecharia a página.
                                "Escape" => {}
                                "ArrowDown" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&git_popover_content_ref, 1);
                                }
                                "ArrowUp" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&git_popover_content_ref, -1);
                                }
                                _ => {}
                            }
                        }
                    })
                };
                let escape = {
                    let git_popover_open = git_popover_open.clone();
                    let git_popover_toggle_ref = git_popover_toggle_ref.clone();
                    crate::menu_keyboard::escape_consumer(move || {
                        git_popover_open.set(false);
                        if let Some(el) = git_popover_toggle_ref.cast::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    })
                };
                listeners.push(close_on_outside);
                listeners.push(close_on_escape);
                listeners.push(escape);
            }
            move || drop(listeners)
        });
    }

    // Sincronização via git (ciclo 119) — pull e commit+push, sempre
    // disparados por clique explícito, nunca automático.
    let syncing = use_state(|| false);
    let do_pull = {
        let vault_path = props.vault_path.clone();
        let open_dialog = props.open_dialog.clone();
        let on_git_changed = props.on_git_changed.clone();
        let syncing = syncing.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(vault_path) = vault_path.clone() else { return };
            let open_dialog = open_dialog.clone();
            let on_git_changed = on_git_changed.clone();
            let syncing = syncing.clone();
            syncing.set(true);
            wasm_bindgen_futures::spawn_local(async move {
                match api::git_pull(&vault_path).await {
                    Ok(_) => on_git_changed.emit(()),
                    Err(e) => open_dialog.emit(PendingDialog::Alert {
                        message: format!("Erro ao dar pull:\n{}", e),
                    }),
                }
                syncing.set(false);
            });
        })
    };
    let do_commit_and_push = {
        let vault_path = props.vault_path.clone();
        let open_dialog = props.open_dialog.clone();
        let on_git_changed = props.on_git_changed.clone();
        let syncing = syncing.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(vault_path) = vault_path.clone() else { return };
            let open_dialog_for_error = open_dialog.clone();
            let on_git_changed = on_git_changed.clone();
            let syncing = syncing.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Mensagem do commit".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |message: String| {
                    let vault_path = vault_path.clone();
                    let open_dialog = open_dialog_for_error.clone();
                    let on_git_changed = on_git_changed.clone();
                    let syncing = syncing.clone();
                    syncing.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::git_commit_and_push(&vault_path, &message).await {
                            Ok(_) => on_git_changed.emit(()),
                            Err(e) => open_dialog.emit(PendingDialog::Alert {
                                message: format!("Erro ao commitar/dar push:\n{}", e),
                            }),
                        }
                        syncing.set(false);
                    });
                }),
            });
        })
    };

    // Fecha o menu ao clicar fora dele ou apertar Escape — sem isso ele só
    // fechava clicando de novo no botão ⚙ ou (por acidente) ao recarregar
    // a página.
    {
        let menu_open = menu_open.clone();
        let menu_ref = menu_ref.clone();
        let menu_content_ref = menu_content_ref.clone();
        let menu_toggle_ref = menu_toggle_ref.clone();
        use_effect_with(*menu_open, move |open| {
            let mut listeners = Vec::new();
            if *open {
                crate::menu_keyboard::focus_first_item(&menu_content_ref);
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
                    let menu_content_ref = menu_content_ref.clone();
                    let menu_toggle_ref = menu_toggle_ref.clone();
                    EventListener::new(&window, "keydown", move |e| {
                        if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                            match e.key().as_str() {
                                // Escape é tratado pelo `escape_consumer`
                                // abaixo (ciclo 161) — aqui ele seguiria
                                // até o `app.rs` e fecharia a página.
                                "Escape" => {}
                                "ArrowDown" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&menu_content_ref, 1);
                                }
                                "ArrowUp" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&menu_content_ref, -1);
                                }
                                _ => {}
                            }
                        }
                    })
                };
                let escape = {
                    let menu_open = menu_open.clone();
                    let menu_toggle_ref = menu_toggle_ref.clone();
                    crate::menu_keyboard::escape_consumer(move || {
                        menu_open.set(false);
                        if let Some(el) = menu_toggle_ref.cast::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    })
                };
                listeners.push(close_on_outside);
                listeners.push(close_on_escape);
                listeners.push(escape);
            }
            move || drop(listeners)
        });
    }

    html! {
        // `data-tauri-drag-region`: arrastar o header move a janela e
        // duplo clique maximiza — o que a barra do sistema fazia antes
        // (ciclo 180). Botões e campos DENTRO dele não herdam o
        // comportamento: o Tauri só age quando o alvo do clique é o
        // próprio elemento marcado.
        <header class="header-bar" tabindex="0" data-tauri-drag-region="true"
            data-nav-item="header" data-nav-parent="root" data-nav-group="header">
            // Os contêineres de cada lado também arrastam: o vão entre
            // os botões deles cai neles, não no `<header>`.
            <div class="header-bar__left" data-tauri-drag-region="true">
                <button class="btn btn--ghost btn--xs" onclick={props.on_toggle_sidebar.reform(|_| ())}
                    data-nav-item="header-sidebar-toggle" data-nav-parent="header">
                    <Icon name={if props.sidebar_collapsed { "chevron-right" } else { "chevron-left" }} />
                </button>
                // O nome e o vault também arrastam (ciclo 180): o
                // `data-tauri-drag-region` só age quando o alvo do
                // clique É o elemento marcado, então sem isso só o
                // vão entre os dois lados do header arrastaria — pouco
                // e nada óbvio.
                <span class="header-bar__title" data-tauri-drag-region="true">{ "Anotadinho" }</span>
                if let Some(ref name) = props.vault_name {
                    <span class="header-bar__vault" data-tauri-drag-region="true">{ name }</span>
                }
                if let Some(ref files) = props.git_files {
                    <div class="git-status-wrapper" ref={git_popover_ref}>
                        <button class="btn btn--ghost btn--xs git-status__indicator" ref={git_popover_toggle_ref} onclick={toggle_git_popover} title="Status do git"
                            data-nav-item="header-git" data-nav-parent="header">
                            <Icon name="git-branch" />{ format!(" {}", files.len()) }
                        </button>
                        if *git_popover_open {
                            <div class="git-status__popover" ref={git_popover_content_ref}>
                                if files.is_empty() {
                                    <p class="git-status__empty">{ "Sem mudanças" }</p>
                                } else {
                                    <ul class="git-status__list">
                                        { for files.iter().map(|f| html! {
                                            <li class="git-status__item">
                                                <span class="git-status__code">{ &f.status }</span>
                                                <span class="git-status__path">{ &f.path }</span>
                                            </li>
                                        }) }
                                    </ul>
                                }
                                <div class="git-status__actions">
                                    <button class="btn btn--ghost btn--xs" onclick={do_pull} disabled={*syncing}>
                                        { if *syncing { "..." } else { "Pull" } }
                                    </button>
                                    <button class="btn btn--ghost btn--xs" onclick={do_commit_and_push} disabled={*syncing}>
                                        { if *syncing { "..." } else { "Commit + Push" } }
                                    </button>
                                </div>
                            </div>
                        }
                    </div>
                }
            </div>
            <div class="header-bar__right" data-tauri-drag-region="true">
                if props.propostas_pendentes > 0 {
                    <button class="header-bar__propostas btn btn--xs"
                        onclick={props.on_abrir_propostas.reform(|_| ())}
                        data-nav-item="header-propostas" data-nav-parent="header"
                        title={format!("{} proposta(s) do agente esperando revisão", props.propostas_pendentes)}>
                        <Icon name="check-square" />{ props.propostas_pendentes.to_string() }
                    </button>
                }
                <button class="btn btn--ghost btn--xs" onclick={props.on_toggle_theme.reform(|_| ())}
                    data-nav-item="header-theme" data-nav-parent="header">
                    <Icon name={if props.theme_light { "sun" } else { "moon" }} />
                </button>
                <div class="header-menu-wrapper" ref={menu_ref}>
                    <button class="btn btn--ghost btn--xs" ref={menu_toggle_ref} onclick={toggle_menu}
                        data-nav-item="header-menu" data-nav-parent="header"><Icon name="settings" /></button>
                    if *menu_open {
                        <div class="header-menu" ref={menu_content_ref}>
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
                                if props.theme_light {
                                    <Icon name="moon" />{ " Tema escuro" }
                                } else {
                                    <Icon name="sun" />{ " Tema claro" }
                                }
                            </button>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_open_aparencia = props.on_open_aparencia.clone();
                                Callback::from(move |_| { menu_open.set(false); on_open_aparencia.emit(()); })
                            }}>
                                <Icon name="settings" />{ " Aparência..." }
                            </button>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_toggle_autosave = props.on_toggle_autosave.clone();
                                Callback::from(move |_| { menu_open.set(false); on_toggle_autosave.emit(()); })
                            }}>
                                if props.autosave_enabled { <Icon name="check" />{ " " } }
                                { "Salvamento automático" }
                            </button>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_toggle_vim_mode = props.on_toggle_vim_mode.clone();
                                Callback::from(move |_| { menu_open.set(false); on_toggle_vim_mode.emit(()); })
                            }}>
                                if props.vim_mode_enabled { <Icon name="check" />{ " " } }
                                { "Vim mode" }
                            </button>
                            if props.vim_mode_enabled {
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let menu_open = menu_open.clone();
                                    let on_open_vim_settings = props.on_open_vim_settings.clone();
                                    Callback::from(move |_| { menu_open.set(false); on_open_vim_settings.emit(()); })
                                }}>
                                    { "Atalhos do Vim mode..." }
                                </button>
                            }
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_open_global_keymap_settings = props.on_open_global_keymap_settings.clone();
                                Callback::from(move |_| { menu_open.set(false); on_open_global_keymap_settings.emit(()); })
                            }}>
                                { "Atalhos globais..." }
                            </button>
                            <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                let menu_open = menu_open.clone();
                                let on_open_cheatsheet = props.on_open_cheatsheet.clone();
                                Callback::from(move |_| { menu_open.set(false); on_open_cheatsheet.emit(()); })
                            }}>
                                { "Atalhos (?)" }
                            </button>
                        </div>
                    }
                </div>

                <div class="window-controls">
                    <button class="window-controls__btn" type="button" title="Minimizar"
                        data-nav-item="window-minimize" data-nav-parent="header"
                        onclick={Callback::from(|_| {
                            wasm_bindgen_futures::spawn_local(async {
                                let _ = crate::api::window_minimize().await;
                            });
                        })}>
                        <Icon name="window-minimize" />
                    </button>
                    <button class="window-controls__btn" type="button"
                        title={if *maximizada { "Restaurar" } else { "Maximizar" }}
                        data-nav-item="window-maximize" data-nav-parent="header"
                        onclick={{
                            let maximizada = maximizada.clone();
                            Callback::from(move |_| {
                                let maximizada = maximizada.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    if let Ok(nova) = crate::api::window_toggle_maximize().await {
                                        maximizada.set(nova);
                                    }
                                });
                            })
                        }}>
                        <Icon name={if *maximizada { "window-restore" } else { "window-maximize" }} />
                    </button>
                    <button class="window-controls__btn window-controls__btn--fechar" type="button" title="Fechar"
                        data-nav-item="window-close" data-nav-parent="header"
                        onclick={Callback::from(|_| {
                            wasm_bindgen_futures::spawn_local(async {
                                let _ = crate::api::window_close().await;
                            });
                        })}>
                        <Icon name="x" />
                    </button>
                </div>
            </div>
        </header>
    }
}
