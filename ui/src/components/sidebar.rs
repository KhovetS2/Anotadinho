//! Sidebar com lista de páginas do vault.
//!
//! Mostra duas seções: Pages (vault/pages/) e Journals (vault/journals/).
//! Click em um item emite callback com o path da página selecionada.
//! Botão "+" na seção Pages cria nova página.
//! Campo de busca filtra por título.

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Props da Sidebar.
#[derive(Properties, PartialEq, Clone)]
pub struct SidebarProps {
    /// Path absoluto do vault aberto.
    pub vault_path: String,
    /// Callback ao selecionar uma página (path relativo).
    pub on_page_selected: Callback<PageMeta>,
    /// Incrementa para forçar reload da lista.
    #[prop_or_default]
    pub list_version: u32,
    /// Sidebar colapsada (compacta).
    #[prop_or_default]
    pub collapsed: bool,
}

/// Componente Sidebar.
#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let pages = use_state(Vec::<PageMeta>::new);
    let selected_path = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let refresh_tick = use_state(|| 0u32);
    let search = use_state(String::new);

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        let loading = loading.clone();
        let tick = (*refresh_tick, props.list_version);

        use_effect_with(tick, move |_| {
            let vault_path = vault_path.clone();
            let pages = pages.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match api::list_pages(&vault_path).await {
                    Ok(list) => {
                        pages.set(list);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        pages.set(Vec::new());
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let filter = search.trim().to_lowercase();
    let all_pages: Vec<PageMeta> = if filter.is_empty() {
        (*pages).clone()
    } else {
        pages
            .iter()
            .filter(|p| p.title.to_lowercase().contains(&filter))
            .cloned()
            .collect()
    };

    let page_items: Vec<PageMeta> = all_pages.iter().filter(|p| p.section == "pages").cloned().collect();
    let journal_items: Vec<PageMeta> = all_pages.iter().filter(|p| p.section == "journals").cloned().collect();

    // Content search results
    let content_results = use_state(Vec::<(String, String)>::new);
    let searching = use_state(|| false);
    {
        let vault_path = props.vault_path.clone();
        let filter = filter.clone();
        let content_results = content_results.clone();
        let searching = searching.clone();
        use_effect_with(filter.clone(), move |_| {
            let should_run = filter.len() >= 3;
            if should_run {
                let vault_path = vault_path.clone();
                let filter = filter.clone();
                let content_results = content_results.clone();
                let searching = searching.clone();
                searching.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::search_content(&vault_path, &filter).await {
                        Ok(r) => content_results.set(r),
                        Err(_) => content_results.set(Vec::new()),
                    }
                    searching.set(false);
                });
            } else {
                content_results.set(Vec::new());
                searching.set(false);
            }
            || {}
        });
    }

    let on_search_input = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                search.set(input.value());
            }
        })
    };

    let clear_search = {
        let search = search.clone();
        Callback::from(move |_| search.set(String::new()))
    };

    let on_search_keydown = {
        let search = search.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                search.set(String::new());
            }
        })
    };

    let on_new_page = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| {
            let title = gloo_dialogs::prompt("Título da nova página:", Some("Nova nota"))
                .unwrap_or_default();
            let title = title.trim().to_string();
            if title.is_empty() {
                return;
            }
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::create_page(&vault_path, &title).await {
                    Ok(meta) => {
                        selected_path.set(Some(meta.path.clone()));
                        on_page_selected.emit(meta);
                        refresh_tick.set(*refresh_tick + 1);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        gloo_dialogs::alert(&format!("Erro ao criar página: {}", e));
                    }
                }
            });
        })
    };

    let on_today = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::open_today_journal(&vault_path).await {
                    Ok(meta) => {
                        selected_path.set(Some(meta.path.clone()));
                        on_page_selected.emit(meta);
                        refresh_tick.set(*refresh_tick + 1);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        gloo_dialogs::alert(&format!("Erro ao abrir journal: {}", e));
                    }
                }
            });
        })
    };

    let has_results = !all_pages.is_empty() || filter.is_empty();

    html! {
        <aside class={ if props.collapsed { "app-sidebar app-sidebar--collapsed" } else { "app-sidebar" } }>
            if props.collapsed {
                <div class="sidebar-collapsed" title="Expandir sidebar">
                    <span class="sidebar-collapsed__icon" title="Pages">{ "📄" }</span>
                    <span class="sidebar-collapsed__icon" title="Journals">{ "📅" }</span>
                    <span class="sidebar-collapsed__icon" title="Buscar">{ "🔍" }</span>
                </div>
            } else {
            <div class="sidebar-search">
                <input
                    class="sidebar-search__input"
                    type="text"
                    placeholder="Buscar páginas..."
                    value={(*search).clone()}
                    oninput={on_search_input}
                    onkeydown={on_search_keydown}
                />
                if !search.is_empty() {
                    <button class="sidebar-search__clear" onclick={clear_search} title="Limpar busca">
                        { "✕" }
                    </button>
                }
            </div>
            if *loading && pages.is_empty() {
                <p class="app-sidebar__hint">{ "Carregando..." }</p>
            } else if !has_results {
                <p class="app-sidebar__hint">{ "Nenhum resultado" }</p>
            } else {
                <div class="sidebar-section">
                    <div class="sidebar-section__header">
                        <h3 class="sidebar-section__title">{ "Pages" }</h3>
                        <button class="sidebar-section__add" title="Nova página" onclick={on_new_page}>
                            { "+" }
                        </button>
                    </div>
                    { render_list(&page_items, &selected_path, &props.on_page_selected) }
                </div>
                <div class="sidebar-section">
                    <div class="sidebar-section__header">
                        <h3 class="sidebar-section__title">{ "Journals" }</h3>
                        <button class="sidebar-section__add sidebar-section__today" title="Journal de hoje" onclick={on_today}>
                            { "Hoje" }
                        </button>
                    </div>
                    { render_list(&journal_items, &selected_path, &props.on_page_selected) }
                </div>
                if !content_results.is_empty() {
                    <div class="sidebar-section">
                        <h3 class="sidebar-section__title">{ format!("Resultados ({})", content_results.len()) }</h3>
                        <ul class="sidebar-list">
                            { for content_results.iter().map(|(path, excerpt)| {
                                let path = path.clone();
                                let excerpt = excerpt.clone();
                                let title = std::path::Path::new(&path).file_stem()
                                    .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                                let page_meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                                let on_page_selected = props.on_page_selected.clone();
                                let selected_path = selected_path.clone();
                                let onclick = Callback::from(move |_| {
                                    selected_path.set(Some(path.clone()));
                                    on_page_selected.emit(page_meta.clone());
                                });
                                html! {
                                    <li class="sidebar-item" {onclick}>
                                        <span class="sidebar-item__icon">{ page_icon("search") }</span>
                                        <div class="sidebar-item__result">
                                            <span class="sidebar-item__title">{ &title }</span>
                                            <span class="sidebar-item__excerpt">{ &excerpt }</span>
                                        </div>
                                    </li>
                                }
                            }) }
                        </ul>
                    </div>
                }
            }
            }
        </aside>
    }
}

fn render_list(
    items: &[PageMeta],
    selected_path: &UseStateHandle<Option<String>>,
    on_page_selected: &Callback<PageMeta>,
) -> Html {
    if items.is_empty() {
        return html! {
            <p class="sidebar-section__empty">{ "Nenhuma página ainda" }</p>
        };
    }
    html! {
        <ul class="sidebar-list">
            { for items.iter().map(|page| {
                let path = page.path.clone();
                let title = page.title.clone();
                let page_meta = page.clone();
                let is_selected = selected_path.as_deref() == Some(path.as_str());
                let class = if is_selected {
                    "sidebar-item sidebar-item--selected"
                } else {
                    "sidebar-item"
                };
                let on_page_selected = on_page_selected.clone();
                let selected_path = selected_path.clone();
                let path_for_cb = path.clone();
                let onclick = Callback::from(move |_| {
                    selected_path.set(Some(path_for_cb.clone()));
                    on_page_selected.emit(page_meta.clone());
                });
                html! {
                    <li {class} {onclick}>
                        <span class="sidebar-item__icon">{ page_icon(&page.section) }</span>
                        <span class="sidebar-item__title">{ &title }</span>
                    </li>
                }
            }) }
        </ul>
    }
}

fn page_icon(section: &str) -> &'static str {
    match section {
        "journals" => "📅",
        "search" => "🔍",
        _ => "📄",
    }
}
