//! Sidebar com lista de páginas do vault.
//!
//! Mostra duas seções: Pages (vault/pages/) e Journals (vault/journals/).
//! Click em um item emite callback com o path da página selecionada.

use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Props da Sidebar.
#[derive(Properties, PartialEq, Clone)]
pub struct SidebarProps {
    /// Path absoluto do vault aberto.
    pub vault_path: String,
    /// Callback ao selecionar uma página (path relativo).
    pub on_page_selected: Callback<PageMeta>,
}

/// Componente Sidebar.
#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let pages = use_state(Vec::<PageMeta>::new);
    let selected_path = use_state(|| None::<String>);
    let loading = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        let loading = loading.clone();

        use_effect_with((), move |_| {
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

    let page_items: Vec<&PageMeta> = pages.iter().filter(|p| p.section == "pages").collect();
    let journal_items: Vec<&PageMeta> = pages.iter().filter(|p| p.section == "journals").collect();

    let render_section = |title: &str, items: &[&PageMeta],
                          selected_path: &UseStateHandle<Option<String>>,
                          on_page_selected: &Callback<PageMeta>| {
        let empty = items.is_empty();
        html! {
            <div class="sidebar-section">
                <h3 class="sidebar-section__title">{ title }</h3>
                if empty {
                    <p class="sidebar-section__empty">{ "Nenhuma página ainda" }</p>
                } else {
                    <ul class="sidebar-list">
                        { for items.iter().map(|page| {
                            let path = page.path.clone();
                            let title = page.title.clone();
                            let page_meta = (*page).clone();
                            let is_selected = selected_path.as_deref() == Some(path.as_str());
                            let class = if is_selected { "sidebar-item sidebar-item--selected" } else { "sidebar-item" };
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
            </div>
        }
    };

    html! {
        <aside class="app-sidebar">
            if *loading {
                <p class="app-sidebar__hint">{ "Carregando..." }</p>
            } else {
                { render_section("Pages", &page_items, &selected_path, &props.on_page_selected) }
                { render_section("Journals", &journal_items, &selected_path, &props.on_page_selected) }
            }
        </aside>
    }
}

fn page_icon(section: &str) -> &'static str {
    match section {
        "journals" => "📅",
        _ => "📄",
    }
}
