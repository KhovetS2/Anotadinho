//! Cabeçalho compartilhado pras páginas de tipo específico (kanban,
//! calendário, tabela, tags, assets, grafo — ciclo 130). Antes deste
//! ciclo, essas páginas não tinham NENHUMA forma de acessar o painel
//! de Propriedades (que só existia dentro do `Editor`, e esses tipos
//! nunca renderizam `Editor` — ver `page_view.rs`): depois de criar
//! uma dessas páginas (inclusive via o comando de tipo específico da
//! paleta, ciclo 128), não tinha como editar seu título/tags/tipo pela
//! UI. Achado documentado na auditoria final do ciclo 129.

use anotadinho_core::Frontmatter;
use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::components::properties_panel::PropertiesPanel;
use crate::dialog::PendingDialog;

/// Props do `TypedPageHeader`.
#[derive(Properties, PartialEq, Clone)]
pub struct TypedPageHeaderProps {
    pub vault_path: String,
    pub page: PageMeta,
    pub open_dialog: Callback<PendingDialog>,
    /// Disparado depois de uma alteração de propriedades persistida
    /// com sucesso — o pai (`page_view.rs`) usa isso pra refazer a
    /// leitura do `type` da página (se o usuário mudar o tipo aqui,
    /// o componente certo tem que assumir na hora, sem precisar
    /// trocar de página e voltar).
    #[prop_or_default]
    pub on_properties_changed: Callback<()>,
}

#[function_component(TypedPageHeader)]
pub fn typed_page_header(props: &TypedPageHeaderProps) -> Html {
    let open = use_state(|| false);
    // `(Frontmatter, corpo original intocado)` — carregado sob demanda
    // ao abrir, pra não pagar uma leitura extra do arquivo em toda
    // renderização da página tipada (o corpo não é reaproveitado pra
    // exibição, só precisa sobreviver intocado até a próxima gravação).
    let loaded: UseStateHandle<Option<(Frontmatter, String)>> = use_state(|| None);

    let on_open = {
        let vault_path = props.vault_path.clone();
        let page_path = props.page.path.clone();
        let open = open.clone();
        let loaded = loaded.clone();
        Callback::from(move |_: MouseEvent| {
            let vault_path = vault_path.clone();
            let page_path = page_path.clone();
            let open = open.clone();
            let loaded = loaded.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(content) = api::read_page(&vault_path, &page_path).await {
                    let (fm, body) = anotadinho_core::MarkdownCodec::split_frontmatter(&content)
                        .map(|(fm, body)| (fm, body.to_string()))
                        .unwrap_or_default();
                    loaded.set(Some((fm, body)));
                    open.set(true);
                }
            });
        })
    };

    let on_close = {
        let open = open.clone();
        Callback::from(move |_: ()| open.set(false))
    };

    // Reserializa frontmatter + corpo ORIGINAL intocado (mesmo formato
    // de bloco que `editor.rs::on_frontmatter_change` usa) e grava
    // direto — diferente do editor, não há sessão de edição contínua
    // aqui, então persiste na hora em vez de só marcar "não salvo".
    let on_change = {
        let vault_path = props.vault_path.clone();
        let page_path = props.page.path.clone();
        let loaded = loaded.clone();
        let on_properties_changed = props.on_properties_changed.clone();
        Callback::from(move |new_fm: Frontmatter| {
            let Some((_, body)) = (*loaded).clone() else { return };
            let vault_path = vault_path.clone();
            let page_path = page_path.clone();
            let loaded = loaded.clone();
            let on_properties_changed = on_properties_changed.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let yaml = serde_yaml::to_string(&new_fm).unwrap_or_default();
                let mut block = String::from("---\n");
                block.push_str(yaml.trim_start_matches("---\n"));
                if !block.ends_with('\n') {
                    block.push('\n');
                }
                block.push_str("---");
                let new_full = format!("{}\n{}", block, body);
                if api::write_page(&vault_path, &page_path, &new_full).await.is_ok() {
                    loaded.set(Some((new_fm, body)));
                    on_properties_changed.emit(());
                }
            });
        })
    };

    html! {
        <header class="editor__header">
            <h2 class="editor__title">{ &props.page.title }</h2>
            <div class="editor__actions">
                <button class="btn btn--ghost btn--sm" onclick={on_open} title="Propriedades">
                    <Icon name="settings" />{ " Propriedades" }
                </button>
            </div>
            if *open {
                if let Some((fm, _)) = (*loaded).clone() {
                    <Modal title={"Propriedades".to_string()} open={true} on_close={on_close}>
                        <PropertiesPanel
                            frontmatter={fm}
                            on_change={on_change}
                            open_dialog={props.open_dialog.clone()}
                        />
                    </Modal>
                }
            }
        </header>
    }
}
