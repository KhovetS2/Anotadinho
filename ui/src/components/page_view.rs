//! PageView: roteia para o renderer correto baseado no frontmatter.type.

use yew::prelude::*;
use crate::api::PageMeta;
use crate::components::assets_page::AssetsPage;
use crate::components::calendar::Calendar;
use crate::components::editor::Editor;
use crate::components::kanban::Kanban;
use crate::components::tags_page::TagsPage;
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
    /// Se o vim mode (modal Normal/Insert) está ativado.
    #[prop_or(false)]
    pub vim_mode_enabled: bool,
    /// Mapa de teclas do vim mode.
    #[prop_or_default]
    pub vim_keymap: crate::state::VimKeymap,
    /// Ação disparada de fora via `GlobalKeymap` (ciclo 105) — repassada
    /// direto pro `Editor`, único componente que sabe reagir a ela.
    #[prop_or_default]
    pub global_action: Option<(crate::state::GlobalEditorAction, u32)>,
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
        "tags" => html! {
            <TagsPage vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
        },
        "assets" => html! {
            <AssetsPage vault_path={props.vault_path.clone()} open_dialog={props.open_dialog.clone()} />
        },
        // "landing" não tem componente próprio — é uma página normal
        // (mesmo Editor de sempre), só marcada como tal pra aparecer com
        // ícone diferente na sidebar e pra poder ser definida como
        // "início" (ver botão 🏠 no editor). "Customizável com
        // componentes" já vem de graça: o corpo dela usa o sistema de
        // embeds inline (`{{ type: "kanban" }}` etc) igual qualquer
        // outra página.
        "landing" | _ => html! {
            <Editor
                vault_path={props.vault_path.clone()}
                page={props.page.clone()}
                on_page_deleted={props.on_page_deleted.clone()}
                open_dialog={props.open_dialog.clone()}
                on_page_selected={props.on_page_selected.clone()}
                autosave_enabled={props.autosave_enabled}
                vim_mode_enabled={props.vim_mode_enabled}
                vim_keymap={props.vim_keymap.clone()}
                global_action={props.global_action}
            />
        },
    }
}
