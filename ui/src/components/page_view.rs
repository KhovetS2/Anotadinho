//! PageView: roteia para o renderer correto baseado no frontmatter.type.

use yew::prelude::*;
use crate::api::PageMeta;
use crate::components::calendar::Calendar;
use crate::components::editor::Editor;
use crate::components::kanban::Kanban;
use crate::components::task_table::TaskTable;
use crate::dialog::PendingDialog;

#[derive(Properties, PartialEq, Clone)]
pub struct PageViewProps {
    pub vault_path: String,
    pub page: Option<PageMeta>,
    pub on_page_deleted: Callback<()>,
    pub on_page_selected: Callback<PageMeta>,
    /// Abre o modal de diálogo do app (ver `crate::dialog`).
    pub open_dialog: Callback<PendingDialog>,
    /// Se falso, o `Editor` não agenda o save automático após alguns
    /// segundos de inatividade — o usuário precisa clicar em "Salvar".
    #[prop_or(true)]
    pub autosave_enabled: bool,
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
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    let pt = match crate::api::read_page(&vault_path, &path).await {
                        Ok(content) => anotadinho_core::MarkdownCodec::split_frontmatter(&content)
                            .map(|(fm, _)| fm.effective_type().to_string())
                            .unwrap_or_else(|_| "md".to_string()),
                        Err(_) => "md".to_string(),
                    };
                    page_type.set(pt);
                    loading.set(false);
                });
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

    match (*page_type).as_str() {
        "kanban" => html! {
            <Kanban vault_path={props.vault_path.clone()} page={props.page.clone()} on_page_selected={props.on_page_selected.clone()} />
        },
        "calendar" => html! {
            <Calendar vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
        },
        "table" => html! {
            <TaskTable vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
        },
        _ => html! {
            <Editor
                vault_path={props.vault_path.clone()}
                page={props.page.clone()}
                on_page_deleted={props.on_page_deleted.clone()}
                open_dialog={props.open_dialog.clone()}
                on_page_selected={props.on_page_selected.clone()}
                autosave_enabled={props.autosave_enabled}
            />
        },
    }
}
