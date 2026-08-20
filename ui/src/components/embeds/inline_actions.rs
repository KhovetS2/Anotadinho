//! Botões de ação (`{{ type: "actions" }}`) — o embed que FAZ coisas.
//!
//! Todos os outros embeds mostram. Este opera: transforma uma página
//! comum (em especial uma `type: landing`) num painel. No fluxo do
//! agent-os, criar uma spec era abrir a sidebar, achar a pasta certa,
//! achar o template certo e digitar o título; aqui vira um clique — e o
//! que o botão faz está declarado em YAML legível no próprio `.md`,
//! então humano e agente leem a mesma coisa.
//!
//! As ações são uma lista FECHADA de operações do próprio app
//! (`ActionSpec`). Nada de rodar comando de shell ou processo externo:
//! um `.md` que chegasse de terceiro executaria código só por ser
//! aberto. Nome de ação desconhecido vira botão desabilitado com aviso,
//! preservando o YAML original.

use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::components::icon::Icon;
use crate::dialog::PendingDialog;
use crate::embed::{ActionSpec, ActionsEmbedData, ActionsLayout};

/// Props do `InlineActions`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineActionsProps {
    /// Botões.
    pub data: ActionsEmbedData,
    /// Path do vault.
    pub vault_path: String,
    /// Disparado quando os botões mudam (hoje só por remoção).
    pub on_change: Callback<ActionsEmbedData>,
    /// Abre a página criada/indicada.
    pub on_page_selected: Callback<PageMeta>,
    /// Diálogos do app (pedir título, avisar erro).
    pub open_dialog: Callback<PendingDialog>,
    /// Abre a paleta de comandos já preenchida.
    pub on_search: Callback<String>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Barra de botões de ação.
#[function_component(InlineActions)]
pub fn inline_actions(props: &InlineActionsProps) -> Html {
    let nav_group = props.nav_group.clone();

    html! {
        <div class={classes!("actions-embed", format!("actions-embed--{}",
            if props.data.layout == ActionsLayout::Grid { "grid" } else { "row" }))}
            data-nav-group={nav_group.clone()} data-nav-item={nav_group.clone()} data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
            { for props.data.buttons.iter().enumerate().map(|(idx, button)| {
                let spec = button.spec();
                let runnable = button.is_runnable();
                let is_primary = button.variant.as_deref() == Some("primary");

                let onclick = {
                    let spec = spec.clone();
                    let vault_path = props.vault_path.clone();
                    let on_page_selected = props.on_page_selected.clone();
                    let open_dialog = props.open_dialog.clone();
                    let on_search = props.on_search.clone();
                    Callback::from(move |_| {
                        run_action(
                            &spec,
                            &vault_path,
                            &on_page_selected,
                            &open_dialog,
                            &on_search,
                        );
                    })
                };

                let on_remove = {
                    let data = props.data.clone();
                    let on_change = props.on_change.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        let mut next = data.clone();
                        next.remove_button(idx);
                        on_change.emit(next);
                    })
                };

                let title = match &spec {
                    ActionSpec::Unknown(name) => format!("Ação desconhecida: {name}"),
                    _ if !runnable => "Botão incompleto — falta preencher o alvo no YAML".to_string(),
                    ActionSpec::NewFromTemplate { template, folder } => {
                        format!("Cria página de {template} em {}", folder.clone().unwrap_or_else(|| "pages".into()))
                    }
                    ActionSpec::OpenPage { path } => format!("Abre {path}"),
                    ActionSpec::SetProperty { path, field, value } => {
                        format!("Grava {field}: {value} em {path}")
                    }
                    ActionSpec::RunSearch { query } => format!("Busca por \"{query}\""),
                };

                html! {
                    <span class="actions-embed__slot" key={idx}>
                        <button
                            class={classes!("actions-embed__btn",
                                is_primary.then_some("actions-embed__btn--primary"),
                                (!runnable).then_some("actions-embed__btn--disabled"))}
                            type="button"
                            disabled={!runnable}
                            title={title}
                            data-nav-item="actions-button" data-nav-parent={nav_group.clone()}
                            {onclick}>
                            if let Some(icon) = &button.icon {
                                <Icon name={icon_name(icon)} />
                            }
                            { button.label.clone() }
                        </button>
                        <button class="actions-embed__remove" type="button" title="Remover botão"
                            data-nav-item="actions-remove" data-nav-parent={nav_group.clone()}
                            onclick={on_remove}>
                            <Icon name="x" />
                        </button>
                    </span>
                }
            }) }
            if props.data.buttons.is_empty() {
                <p class="actions-embed__empty">
                    { "Nenhum botão. Edite o YAML do embed pra adicionar (ver guia do Agent OS)." }
                </p>
            }
        </div>
    }
}

/// `Icon` exige `&'static str`; o nome vem do arquivo. Converte pros
/// nomes conhecidos e cai num genérico quando não bate — ícone
/// inventado no YAML não deve sumir com o botão.
fn icon_name(name: &str) -> &'static str {
    match name {
        "file-text" => "file-text",
        "folder" => "folder",
        "calendar" => "calendar",
        "search" => "search",
        "check" => "check",
        "edit" => "edit",
        "home" => "home",
        "link" => "link",
        "clock" => "clock",
        "network" => "network",
        "image" => "image",
        "table" => "table",
        "columns" => "columns",
        "layout" => "layout",
        "settings" => "settings",
        "download" => "download",
        "git-branch" => "git-branch",
        "message-circle" => "message-circle",
        "paperclip" => "paperclip",
        "external-link" => "external-link",
        "info" => "info",
        "lightbulb" => "lightbulb",
        _ => "zap",
    }
}

/// Executa a ação. Erro vira `Alert` — a alternativa (falhar em
/// silêncio) deixaria o usuário achando que o clique funcionou.
fn run_action(
    spec: &ActionSpec,
    vault_path: &str,
    on_page_selected: &Callback<PageMeta>,
    open_dialog: &Callback<PendingDialog>,
    on_search: &Callback<String>,
) {
    match spec {
        ActionSpec::NewFromTemplate { template, folder } => {
            let vault_path = vault_path.to_string();
            let template = template.clone();
            let folder = folder.clone();
            let on_page_selected = on_page_selected.clone();
            let open_dialog_err = open_dialog.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título da nova página".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    if title.trim().is_empty() {
                        return;
                    }
                    let vault_path = vault_path.clone();
                    let template = template.clone();
                    let folder = folder.clone();
                    let on_page_selected = on_page_selected.clone();
                    let open_dialog_err = open_dialog_err.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::create_page_from_template(
                            &vault_path,
                            &template,
                            &title,
                            folder.as_deref(),
                        )
                        .await
                        {
                            Ok(meta) => on_page_selected.emit(meta),
                            Err(e) => open_dialog_err.emit(PendingDialog::Alert {
                                message: format!("Erro ao criar a página: {e}"),
                            }),
                        }
                    });
                }),
            });
        }
        ActionSpec::OpenPage { path } => {
            let title = std::path::Path::new(path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            let section = if path.starts_with("journals/") { "journals" } else { "pages" };
            on_page_selected.emit(PageMeta {
                path: path.clone(),
                title,
                section: section.to_string(),
            });
        }
        ActionSpec::SetProperty { path, field, value } => {
            // Passa pelo `MarkdownCodec::set_frontmatter_field` do core —
            // o MESMO caminho do `anotadinho-cli set-property` — em vez
            // de remontar o bloco de frontmatter aqui.
            let vault_path = vault_path.to_string();
            let path = path.clone();
            let field = field.clone();
            let value = value.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let content = match api::read_page(&vault_path, &path).await {
                    Ok(c) => c,
                    Err(e) => {
                        open_dialog.emit(PendingDialog::Alert {
                            message: format!("Erro ao ler {path}: {e}"),
                        });
                        return;
                    }
                };
                let updated = match anotadinho_core::MarkdownCodec::set_frontmatter_field(
                    &content, &field, &value,
                ) {
                    Ok(u) => u,
                    Err(e) => {
                        open_dialog.emit(PendingDialog::Alert {
                            message: format!("Erro ao gravar {field}: {e}"),
                        });
                        return;
                    }
                };
                match api::write_page(&vault_path, &path, &updated).await {
                    Ok(()) => open_dialog.emit(PendingDialog::Alert {
                        message: format!("{field} de {path} agora é \"{value}\"."),
                    }),
                    Err(e) => open_dialog.emit(PendingDialog::Alert {
                        message: format!("Erro ao gravar {path}: {e}"),
                    }),
                }
            });
        }
        ActionSpec::RunSearch { query } => on_search.emit(query.clone()),
        ActionSpec::Unknown(_) => {}
    }
}
