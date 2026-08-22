//! PageView: roteia para o renderer correto baseado no frontmatter.type.

use yew::prelude::*;
use crate::api::PageMeta;
use crate::components::assets_page::AssetsPage;
use crate::components::calendar::Calendar;
use crate::components::editor::Editor;
use crate::components::graph_view::GraphView;
use crate::components::kanban::Kanban;
use crate::components::tags_page::TagsPage;
use crate::components::task_table::TaskTable;
use crate::components::typed_page_header::TypedPageHeader;
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
    /// Path da página inicial do vault (ciclo 089, estado movido pro
    /// `App` no ciclo 109 — ver `TabBar`).
    #[prop_or_default]
    pub home_page: Option<String>,
    /// Alterna a página `path` como inicial (define/remove).
    #[prop_or_default]
    pub on_toggle_home: Callback<String>,
    /// Abre a paleta de comandos já preenchida — ação `run-search` do
    /// embed de ações (ciclo 156). Repassado ao `Editor`, que repassa
    /// aos embeds.
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// Contador que o `App` incrementa quando o watcher acusa mudança no
    /// vault (ciclo 173) — o editor usa como gatilho pra conferir se a
    /// página aberta mudou no disco.
    #[prop_or_default]
    pub vault_version: u32,
    /// Abre a navegação por blocos (ciclo 174) — repassado ao `Editor`.
    #[prop_or_default]
    pub on_enter_block_nav: Callback<()>,
    pub on_leave_block_nav: Callback<()>,
    /// Página aberta ANTES desta — vira contexto da conversa.
    #[prop_or_default]
    pub contexto_path: Option<String>,
    pub nav_mode_active: bool,
}

#[function_component(PageView)]
pub fn page_view(props: &PageViewProps) -> Html {
    let page_type = use_state(|| "md".to_string());
    let loading = use_state(|| false);
    // Ciclo 130: incrementado pelo `TypedPageHeader` depois de uma
    // troca de propriedades persistida — refaz a leitura do `type` da
    // página (se o usuário mudar o tipo pelo painel, o roteamento
    // abaixo precisa reagir sem esperar uma troca de página).
    let reload_nonce = use_state(|| 0u32);

    {
        let page = props.page.clone();
        let page_type = page_type.clone();
        let loading = loading.clone();
        let vault_path = props.vault_path.clone();

        use_effect_with((page.clone(), *reload_nonce), move |(page, _)| {
            let page = page.clone();
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

    // `expect` seguro: já retornamos acima se `props.page` for `None`.
    let current_page = props.page.clone().expect("página presente, checado acima");
    let on_properties_changed = {
        let reload_nonce = reload_nonce.clone();
        Callback::from(move |_: ()| reload_nonce.set(*reload_nonce + 1))
    };
    // Cabeçalho com acesso ao painel de Propriedades (ciclo 130) — os
    // 6 tipos abaixo nunca renderizam `Editor` (único lugar que
    // continha esse painel antes), então ganham esse cabeçalho
    // compartilhado em vez de cada um reimplementar o próprio acesso.
    let typed_header = html! {
        <TypedPageHeader
            vault_path={props.vault_path.clone()}
            page={current_page.clone()}
            open_dialog={props.open_dialog.clone()}
            on_properties_changed={on_properties_changed}
        />
    };

    match (*page_type).as_str() {
        "kanban" => html! {
            <>
                { typed_header }
                <Kanban vault_path={props.vault_path.clone()} page={props.page.clone()} on_page_selected={props.on_page_selected.clone()} />
            </>
        },
        "calendar" => html! {
            <>
                { typed_header }
                <Calendar vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
            </>
        },
        "table" => html! {
            <>
                { typed_header }
                <TaskTable vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
            </>
        },
        // A conversa é uma PÁGINA (ciclo 202): entra aqui do mesmo jeito
        // que kanban e calendário, e o `.md` dela segue legível fora do
        // app.
        "conversa" => match &props.page {
            Some(page) => html! {
                <crate::components::conversa_view::ConversaView
                    vault_path={props.vault_path.clone()}
                    page={page.clone()}
                    contexto_path={props.contexto_path.clone()} />
            },
            None => html! {},
        },
        "tags" => html! {
            <>
                { typed_header }
                <TagsPage vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
            </>
        },
        "assets" => html! {
            <>
                { typed_header }
                <AssetsPage vault_path={props.vault_path.clone()} open_dialog={props.open_dialog.clone()} />
            </>
        },
        "graph" => html! {
            <>
                { typed_header }
                <GraphView vault_path={props.vault_path.clone()} on_page_selected={props.on_page_selected.clone()} />
            </>
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
                home_page={props.home_page.clone()}
                on_toggle_home={props.on_toggle_home.clone()}
                on_search={props.on_search.clone()}
                vault_version={props.vault_version}
                on_enter_block_nav={props.on_enter_block_nav.clone()}
                on_leave_block_nav={props.on_leave_block_nav.clone()}
                nav_mode_active={props.nav_mode_active}
            />
        },
    }
}
