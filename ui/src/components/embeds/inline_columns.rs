//! Painéis markdown lado a lado (`{{ type: "columns" }}`).
//!
//! Markdown é linear: tudo empilha numa coluna só. Este embed é o que
//! permite montar uma landing page ou um painel sem sair do arquivo
//! `.md` — cada painel guarda markdown de verdade, editado pelo mesmo
//! `EmbedMarkdownField` do callout (ciclo 151).
//!
//! Não tem divisória arrastável de propósito: arrastar dentro de um
//! embed que convive com seleção de texto foi fonte de bug real (ciclo
//! 068). O ajuste de largura é por botão, em unidades de fração
//! inteiras — que é também o que mantém o YAML legível pra um agente.

use yew::prelude::*;

use crate::components::embeds::EmbedMarkdownField;
use crate::components::icon::Icon;
use crate::dialog::PendingDialog;
use crate::embed::ColumnsEmbedData;

/// Props do `InlineColumns`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineColumnsProps {
    /// Painéis.
    pub data: ColumnsEmbedData,
    /// Disparado quando um painel muda de conteúdo, largura ou some.
    pub on_change: Callback<ColumnsEmbedData>,
    /// Abre o modal de diálogo do app (confirmação ao remover painel
    /// com conteúdo).
    pub open_dialog: Callback<PendingDialog>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Painéis markdown lado a lado.
#[function_component(InlineColumns)]
pub fn inline_columns(props: &InlineColumnsProps) -> Html {
    let nav_group = props.nav_group.clone();
    let can_add = props.data.columns.len() < ColumnsEmbedData::MAX_COLUMNS;
    let can_remove = props.data.columns.len() > 1;

    let on_add = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            let mut new_data = data.clone();
            new_data.add_column();
            on_change.emit(new_data);
        })
    };

    html! {
        <div class="columns-embed" data-nav-group={nav_group.clone()}>
            <div class="columns-embed__grid" style={format!("grid-template-columns: {};", props.data.grid_template())}>
                { for props.data.columns.iter().enumerate().map(|(idx, pane)| {
                    let on_body = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |body: String| {
                            let mut new_data = data.clone();
                            new_data.set_body(idx, body);
                            on_change.emit(new_data);
                        })
                    };
                    let width_step = |delta: i8| {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |_: MouseEvent| {
                            let mut new_data = data.clone();
                            new_data.adjust_width(idx, delta);
                            on_change.emit(new_data);
                        })
                    };
                    let on_remove = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let has_content = !pane.body.trim().is_empty();
                        Callback::from(move |_| {
                            let mut new_data = data.clone();
                            new_data.remove_column(idx);
                            if has_content {
                                // Painel com texto some sem deixar rastro
                                // no arquivo — vale perguntar.
                                let on_change = on_change.clone();
                                open_dialog.emit(PendingDialog::Confirm {
                                    message: "Remover esta coluna? O texto dela será perdido.".to_string(),
                                    confirm_label: "Remover".to_string(),
                                    on_confirm: Callback::from(move |_| on_change.emit(new_data.clone())),
                                });
                            } else {
                                on_change.emit(new_data);
                            }
                        })
                    };

                    html! {
                        <div class="columns-embed__pane" key={idx}>
                            <div class="columns-embed__pane-bar">
                                <span class="columns-embed__width">{ format!("{}fr", pane.width) }</span>
                                <button class="columns-embed__btn" type="button" title="Estreitar"
                                    data-nav-item="columns-narrow" data-nav-parent={nav_group.clone()}
                                    onclick={width_step(-1)}>
                                    <Icon name="chevron-left" />
                                </button>
                                <button class="columns-embed__btn" type="button" title="Alargar"
                                    data-nav-item="columns-widen" data-nav-parent={nav_group.clone()}
                                    onclick={width_step(1)}>
                                    <Icon name="chevron-right" />
                                </button>
                                if can_remove {
                                    <button class="columns-embed__btn columns-embed__btn--danger" type="button" title="Remover coluna"
                                        data-nav-item="columns-remove" data-nav-parent={nav_group.clone()}
                                        onclick={on_remove}>
                                        <Icon name="x" />
                                    </button>
                                }
                            </div>
                            <EmbedMarkdownField
                                markdown={pane.body.clone()}
                                on_change={on_body}
                                nav_group={nav_group.clone()}
                                placeholder="Clique pra escrever esta coluna (Markdown)"
                            />
                        </div>
                    }
                }) }
            </div>
            if can_add {
                <button class="columns-embed__add" type="button"
                    data-nav-item="columns-add" data-nav-parent={nav_group.clone()}
                    onclick={on_add}>
                    { "+ coluna" }
                </button>
            }
        </div>
    }
}
