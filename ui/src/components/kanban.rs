//! Kanban board view.

use yew::prelude::*;
use crate::api::{self, PageMeta};

#[derive(Properties, PartialEq, Clone)]
pub struct KanbanProps {
    pub vault_path: String,
    pub on_page_selected: Callback<PageMeta>,
}

const COLUMNS: &[&str] = &["backlog", "todo", "doing", "done"];
const COLUMN_LABELS: &[&str] = &["Backlog", "A Fazer", "Fazendo", "Concluído"];

#[function_component(Kanban)]
pub fn kanban(props: &KanbanProps) -> Html {
    let pages = use_state(Vec::<PageMeta>::new);
    let loading = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                if let Ok(list) = api::list_pages(&vault_path).await {
                    pages.set(list);
                }
                loading.set(false);
            });
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
                { for COLUMNS.iter().enumerate().map(|(ci, col)| {
                    let items: Vec<&PageMeta> = pages.iter()
                        .filter(|p| {
                            let lower = p.title.to_lowercase();
                            if *col == "backlog" && !lower.contains("todo") && !lower.contains("doing") && !lower.contains("done") { return true; }
                            lower.contains(col)
                        })
                        .collect();
                    html! {
                        <div class="kanban__column">
                            <div class="kanban__col-header">
                                <span class="kanban__col-title">{ COLUMN_LABELS[ci] }</span>
                                <span class="kanban__col-count">{ items.len() }</span>
                            </div>
                            <div class="kanban__col-body">
                                { for items.iter().map(|page| {
                                    let meta = (*page).clone();
                                    let on_page_selected = on_page_selected.clone();
                                    html! {
                                        <div class="kanban__card"
                                            onclick={Callback::from(move |_| on_page_selected.emit(meta.clone()))}
                                        >
                                            <span class="kanban__card-title">{ &page.title }</span>
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
