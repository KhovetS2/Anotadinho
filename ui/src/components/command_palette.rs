//! Paleta de comandos (Ctrl+K) — navegar pra qualquer página do vault ou
//! disparar um comando nomeado, tudo pelo teclado, sem precisar do
//! mouse. Mesmo padrão visual/mecânico do menu `/` do editor (lista
//! filtrada, ArrowUp/Down/Enter, fechar ao Escape/clicar fora).

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::components::icon::Icon;
use crate::components::sidebar::render_excerpt_highlight;

/// Só busca conteúdo (via `SearchIndex` FTS5, ciclo 094) a partir desse
/// tamanho de query — evita disparar buscas caras a cada tecla no
/// início da digitação.
const CONTENT_SEARCH_MIN_LEN: usize = 3;

/// Comando nomeado disparado pela paleta — tratado centralmente em
/// `App` (é quem tem os callbacks de tema/sidebar/etc já em mãos).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaletteAction {
    NewPage,
    NewFolder,
    ToggleTheme,
    ToggleSidebar,
    Today,
    ViewTags,
    ViewAssets,
    ExportVault,
    ViewCheatsheet,
    /// Cria uma página já nascendo com `type: <page_type>` no
    /// frontmatter (ciclo 128) — pede só o título (mesmo
    /// `PendingDialog::Prompt` de `NewPage`), sem passar pelo painel
    /// de Propriedades pra trocar o tipo depois de criada.
    NewPageOfType(&'static str),
    /// Conversa nova com o agente, em um passo (ciclo 208): cria em
    /// `pages/conversas/`, já com `type: conversa` e a página atual
    /// anexada como contexto, e abre.
    NovaConversa,
}

const COMMANDS: &[(&str, PaletteAction)] = &[
    ("Nova conversa com o agente", PaletteAction::NovaConversa),
    ("Nova página", PaletteAction::NewPage),
    ("Nova pasta", PaletteAction::NewFolder),
    ("Nova página: Kanban", PaletteAction::NewPageOfType("kanban")),
    ("Nova página: Calendário", PaletteAction::NewPageOfType("calendar")),
    ("Nova página: Tabela de tarefas", PaletteAction::NewPageOfType("table")),
    ("Nova página: Grafo de conexões", PaletteAction::NewPageOfType("graph")),
    ("Nova página: Conversa", PaletteAction::NewPageOfType("conversa")),
    ("Alternar tema", PaletteAction::ToggleTheme),
    ("Alternar sidebar", PaletteAction::ToggleSidebar),
    ("Ir pra Hoje (journal)", PaletteAction::Today),
    ("Ver Tags", PaletteAction::ViewTags),
    ("Ver Assets", PaletteAction::ViewAssets),
    ("Exportar vault inteiro", PaletteAction::ExportVault),
    ("Ver atalhos", PaletteAction::ViewCheatsheet),
];

#[derive(Debug, Clone, PartialEq)]
enum Item {
    Command(&'static str, PaletteAction),
    Page(PageMeta),
    /// Resultado de busca de CONTEÚDO (não só título) — página + trecho
    /// com o termo destacado, via `search_content` (ciclo 094 FTS5).
    /// Desde o ciclo 188 carrega junto o `SearchHit` inteiro, que pode
    /// dizer de qual embed o trecho veio e apontar pra ele.
    ContentResult(PageMeta, anotadinho_core::embed::SearchHit),
}

/// Props da `CommandPalette`.
#[derive(Properties, PartialEq, Clone)]
pub struct CommandPaletteProps {
    /// Path do vault — usado só pra buscar a lista de páginas.
    pub vault_path: String,
    /// Disparado quando a paleta deve fechar (Escape, clicar fora, item
    /// selecionado).
    pub on_close: Callback<()>,
    /// Disparado ao selecionar uma página.
    pub on_page_selected: Callback<PageMeta>,
    /// Disparado ao selecionar um comando nomeado.
    pub on_action: Callback<PaletteAction>,
    /// Texto já preenchido ao abrir — usado pela ação `run-search` do
    /// embed de ações (ciclo 156), que abre a paleta com a busca pronta.
    #[prop_or_default]
    pub initial_query: String,
}

/// Paleta de comandos — montada/desmontada pelo pai a cada abrir/fechar
/// (não fica sempre viva escondida), então o estado interno já nasce
/// limpo toda vez.
#[function_component(CommandPalette)]
pub fn command_palette(props: &CommandPaletteProps) -> Html {
    let query = use_state({
        let initial = props.initial_query.clone();
        move || initial
    });
    let idx = use_state(|| 0usize);
    let pages = use_state(Vec::<PageMeta>::new);
    let content_results = use_state(Vec::<anotadinho_core::embed::SearchHit>::new);
    let input_ref = use_node_ref();
    // Ref do item ATIVO — mesmo padrão da sidebar (ciclo 106): um só
    // `NodeRef`, reatribuído a cada render pro item que estiver
    // destacado no momento (não um ref fixo por item da lista).
    let active_item_ref = use_node_ref();

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(list) = api::list_pages(&vault_path).await {
                    pages.set(list);
                }
            });
            || {}
        });
    }

    // Busca de conteúdo (não só título) — reusa `api::search_content`
    // (mesma função da busca da sidebar, ciclo 094 FTS5), com o mesmo
    // gate de tamanho mínimo pra não disparar busca cara a cada tecla.
    // Não bloqueia o match instantâneo por título/comando abaixo — só
    // adiciona uma seção extra quando a busca assíncrona resolve.
    {
        let vault_path = props.vault_path.clone();
        let query = query.clone();
        let content_results = content_results.clone();
        use_effect_with(query.clone(), move |_| {
            if query.len() >= CONTENT_SEARCH_MIN_LEN {
                let vault_path = vault_path.clone();
                let query = query.clone();
                let content_results = content_results.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::search_content(&vault_path, &query).await {
                        Ok(r) => content_results.set(r),
                        Err(_) => content_results.set(Vec::new()),
                    }
                });
            } else {
                content_results.set(Vec::new());
            }
            || {}
        });
    }

    {
        let input_ref = input_ref.clone();
        use_effect_with((), move |_| {
            if let Some(el) = input_ref.cast::<HtmlInputElement>() {
                let _ = el.focus();
            }
            || {}
        });
    }

    // Rola o item destacado pra dentro da área visível ao navegar com
    // ArrowUp/Down — sem isso, passar do fim da lista visível (ou dar
    // wrap-around de volta pro topo) deixa o item ativo escondido fora
    // do scroll, mesmo bug já corrigido na sidebar (ciclo 106) e no
    // menu `/` do editor (ciclo 073/082) — reportado pelo usuário
    // acontecendo aqui também.
    {
        let active_item_ref = active_item_ref.clone();
        use_effect_with(*idx, move |_| {
            if let Some(el) = active_item_ref.cast::<web_sys::Element>() {
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&opts);
            }
            || {}
        });
    }

    let q = query.to_lowercase();
    let title_matches: Vec<PageMeta> = pages.iter()
        .filter(|p| q.is_empty() || p.title.to_lowercase().contains(&q))
        .cloned()
        .collect();
    let items: Vec<Item> = {
        let mut v: Vec<Item> = COMMANDS.iter()
            .filter(|(label, _)| q.is_empty() || label.to_lowercase().contains(&q))
            .map(|(label, action)| Item::Command(label, *action))
            .collect();
        v.extend(title_matches.iter().cloned().map(Item::Page));
        // Resultados de conteúdo: só páginas que NÃO já apareceram por
        // título (evita listar a mesma página duas vezes).
        v.extend(content_results.iter().filter_map(|hit| {
            if title_matches.iter().any(|p| p.path == hit.path) {
                return None;
            }
            let title = std::path::Path::new(&hit.path).file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            let meta = PageMeta { path: hit.path.clone(), title, section: "pages".to_string() };
            Some(Item::ContentResult(meta, hit.clone()))
        }));
        v
    };
    let items_len = items.len();
    // Índice do primeiro `ContentResult` — usado só pra desenhar o
    // separador de seção uma vez, sem quebrar o match instantâneo por
    // título/comando (que continua sempre primeiro na lista).
    let first_content_idx = items.iter().position(|i| matches!(i, Item::ContentResult(..)));

    let select_idx = {
        let on_close = props.on_close.clone();
        let on_page_selected = props.on_page_selected.clone();
        let on_action = props.on_action.clone();
        let items = items.clone();
        Callback::from(move |i: usize| {
            if let Some(item) = items.get(i) {
                match item.clone() {
                    Item::Command(_, action) => on_action.emit(action),
                    Item::Page(meta) => on_page_selected.emit(meta),
                    Item::ContentResult(meta, hit) => {
                        crate::nav_mode::marcar_alvo_de_busca(hit.ancora.as_deref());
                        on_page_selected.emit(meta);
                    }
                }
            }
            on_close.emit(());
        })
    };

    let oninput = {
        let query = query.clone();
        let idx = idx.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                query.set(input.value());
                idx.set(0);
            }
        })
    };

    let onkeydown = {
        let idx = idx.clone();
        let select_idx = select_idx.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "ArrowDown" => { e.prevent_default(); if items_len > 0 { idx.set((*idx + 1) % items_len); } }
                "ArrowUp" => { e.prevent_default(); if items_len > 0 { idx.set((*idx + items_len - 1) % items_len); } }
                "Enter" => { e.prevent_default(); select_idx.emit(*idx); }
                // `stop_propagation`: o Escape chegava no handler global
                // do `app.rs` e desselecionava a página junto (ciclo 161).
                "Escape" => { e.prevent_default(); e.stop_propagation(); on_close.emit(()); }
                _ => {}
            }
        })
    };

    let onclick_overlay = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let stop_propagation = Callback::from(|e: MouseEvent| e.stop_propagation());

    html! {
        <div class="command-palette-overlay" onclick={onclick_overlay}>
            <div class="command-palette" onclick={stop_propagation}>
                <input ref={input_ref} class="command-palette__input" type="text"
                    placeholder="Buscar página ou comando..." value={(*query).clone()}
                    {oninput} {onkeydown} />
                <div class="command-palette__list">
                    if items.is_empty() {
                        <p class="command-palette__empty">{ "Nada encontrado" }</p>
                    } else {
                        { for items.iter().enumerate().map(|(i, item)| {
                            let is_active = i == *idx;
                            let class = if is_active { "command-palette__item command-palette__item--active" } else { "command-palette__item" };
                            let node_ref = if is_active { active_item_ref.clone() } else { NodeRef::default() };
                            let sel = select_idx.clone();
                            let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
                            let onclick = Callback::from(move |_| sel.emit(i));
                            // Separador desenhado uma vez só, antes do primeiro
                            // resultado de conteúdo — títulos/comandos continuam
                            // sempre no topo, sem esperar a busca assíncrona.
                            let section_header = if first_content_idx == Some(i) {
                                html! { <p class="command-palette__section">{ "No conteúdo" }</p> }
                            } else {
                                html! {}
                            };
                            html! {
                                <>
                                    { section_header }
                                    { match item {
                                        Item::Command(label, _) => html! {
                                            <div {class} ref={node_ref} {onmousedown} {onclick}>
                                                <span class="command-palette__item-icon"><Icon name="zap" /></span>
                                                <span class="command-palette__item-title">{ *label }</span>
                                            </div>
                                        },
                                        Item::Page(meta) => html! {
                                            <div {class} ref={node_ref} {onmousedown} {onclick}>
                                                <span class="command-palette__item-icon"><Icon name="file-text" /></span>
                                                <span class="command-palette__item-title">{ &meta.title }</span>
                                            </div>
                                        },
                                        Item::ContentResult(meta, hit) => html! {
                                            <div {class} ref={node_ref} {onmousedown} {onclick}>
                                                <span class="command-palette__item-icon"><Icon name="file-text" /></span>
                                                <div class="command-palette__item-result">
                                                    <span class="command-palette__item-title">{ &meta.title }</span>
                                                    if let Some(origem) = &hit.origem {
                                                        <span class="command-palette__item-origem">{ origem }</span>
                                                    }
                                                    <span class="command-palette__item-excerpt">{ render_excerpt_highlight(&hit.snippet) }</span>
                                                </div>
                                            </div>
                                        },
                                    } }
                                </>
                            }
                        }) }
                    }
                </div>
            </div>
        </div>
    }
}
