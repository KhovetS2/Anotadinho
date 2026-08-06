//! Kanban board - reads blocks with column:: property.

use yew::prelude::*;
use crate::api::{self, PageMeta};

#[derive(Properties, PartialEq, Clone)]
pub struct KanbanProps {
    pub vault_path: String,
    pub page: Option<PageMeta>,
    pub on_page_selected: Callback<PageMeta>,
}

#[derive(Debug, Clone, PartialEq)]
struct Card { path: String, title: String, column: String }

#[function_component(Kanban)]
pub fn kanban(props: &KanbanProps) -> Html {
    let cards = use_state(Vec::<Card>::new);
    let columns = use_state(|| vec!["backlog".to_string(), "todo".to_string(), "doing".to_string(), "done".to_string()]);
    let loading = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let page = props.page.clone();
        let cards = cards.clone();
        let loading = loading.clone();

        use_effect_with(page.clone(), move |_| {
            let should_run = page.is_some();
            if should_run {
                let p = page.as_ref().unwrap();
                let vault = vault_path.clone();
                let page_path = p.path.clone();
                let cards = cards.clone();
                let loading = loading.clone();
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(content) = api::read_page(&vault, &page_path).await {
                        let mut list = Vec::new();
                        for line in content.lines() {
                            let t = line.trim();
                            if t.starts_with("- ") {
                                let body = &t[2..];
                                let mut col = "backlog".to_string();
                                let mut title = String::new();
                                for l in body.split("  ") {
                                    if let Some(v) = l.strip_prefix("column:: ") {
                                        col = v.trim().to_string();
                                    } else if let Some(v) = l.strip_prefix("title:: ") {
                                        title = v.trim().to_string();
                                    } else if let Some(v) = l.strip_prefix("id:: ") {
                                        // skip id
                                    } else {
                                        if title.is_empty() { title = l.to_string(); }
                                    }
                                }
                                if title.is_empty() { title = body.to_string(); }
                                list.push(Card { path: page_path.clone(), title, column: col });
                            }
                        }
                        cards.set(list);
                    }
                    loading.set(false);
                });
            }
            || {}
        });
    }

    if *loading {
        return html! { <div class="kanban"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let on_page_selected = props.on_page_selected.clone();

    html! {
        <div class="kanban">
            <div class="kanban__board">
                { for (*columns).iter().map(|col| {
                    let items: Vec<&Card> = cards.iter().filter(|c| c.column == *col).collect();
                    let col_header = match col.as_str() {
                        "backlog" => "Backlog", "todo" => "A Fazer",
                        "doing" => "Fazendo", "done" => "Concluído",
                        _ => col.as_str()
                    };
                    html! {
                        <div class="kanban__column">
                            <div class="kanban__col-header">
                                <span class="kanban__col-title">{ col_header }</span>
                                <span class="kanban__col-count">{ items.len() }</span>
                            </div>
                            <div class="kanban__col-body">
                                { for items.iter().map(|card| {
                                    let path = card.path.clone();
                                    let title = card.title.clone();
                                    let meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                                    let on_page_selected = on_page_selected.clone();
                                    html! {
                                        <div class="kanban__card"
                                            onclick={Callback::from(move |_| on_page_selected.emit(meta.clone()))}
                                        >
                                            <span class="kanban__card-title">{ &card.title }</span>
                                        </div>
                                    }
                                }) }
                            </div>
                        </div>
                    }
                }) }
            </div>
        </div>
    }
}
