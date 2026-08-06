//! PageView: roteia para o renderer correto baseado no frontmatter.type.

use yew::prelude::*;
use crate::api::PageMeta;
use crate::components::editor::Editor;
use crate::components::kanban::Kanban;

#[derive(Properties, PartialEq, Clone)]
pub struct PageViewProps {
    pub vault_path: String,
    pub page: Option<PageMeta>,
    pub on_page_deleted: Callback<()>,
    pub on_page_selected: Callback<PageMeta>,
}

#[function_component(PageView)]
pub fn page_view(props: &PageViewProps) -> Html {
    let page_type = use_state(|| "md".to_string());
    let loading = use_state(|| false);

    {
        let page = props.page.clone();
        let page_type = page_type.clone();
        let loading = loading.clone();
        let vault_path = props.vault_path.clone();

        use_effect_with(page.clone(), move |_| {
            if let Some(ref p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let page_type = page_type.clone();
                let loading = loading.clone();
                if p.title != "Nova nota" { // skip for brand new empty pages
                    loading.set(true);
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(content) = crate::api::read_page(&vault_path, &path).await {
                            if let Some(fm_start) = content.find("---") {
                                let after_first = &content[fm_start + 3..];
                                if let Some(fm_end) = after_first.find("---") {
                                    let fm = &after_first[..fm_end];
                                    if let Some(t) = fm.lines().find(|l| l.starts_with("type:")) {
                                        let pt = t.trim_start_matches("type:").trim();
                                        page_type.set(pt.to_string());
                                    } else {
                                        page_type.set("md".to_string());
                                    }
                                }
                            }
                        }
                        loading.set(false);
                    });
                }
            } else {
                page_type.set("md".to_string());
            }
            || {}
        });
    }

    if props.page.is_none() {
        return html! { <main class="app-main"><p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p></main> };
    }

    if *loading {
        return html! { <main class="editor"><p class="editor__status">{ "Carregando..." }</p></main> };
    }

    let pt = (*page_type).as_str();
    match pt {
        "kanban" => html! {
            <Kanban
                vault_path={props.vault_path.clone()}
                on_page_selected={props.on_page_selected.clone()}
            />
        },
        _ => html! {
            <Editor
                vault_path={props.vault_path.clone()}
                page={props.page.clone()}
                on_page_deleted={props.on_page_deleted.clone()}
            />
        },
    }
}
