//! Tabela inline (dentro de uma fence ```table), dinâmica: criar/excluir
//! linha e coluna, configurar o tipo de cada coluna (texto, seleção com
//! opções coloridas, ou checkbox). Reaproveita `.task-table__*` (já
//! estilizadas globalmente) e `.badge--*` (já existentes) pras células de
//! seleção, em vez de criar CSS de badge novo.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::FocusEvent;
use yew::prelude::*;

use crate::dialog::PendingDialog;
use crate::embed::{ColumnKind, TableEmbedData};

/// Props do `InlineTable`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineTableProps {
    /// Colunas (com tipo) + linhas.
    pub data: TableEmbedData,
    /// Disparado quando a tabela muda.
    pub on_change: Callback<TableEmbedData>,
    /// Abre o modal de diálogo do app.
    pub open_dialog: Callback<PendingDialog>,
}

const BADGE_PALETTE: [&str; 4] = ["badge--info", "badge--success", "badge--warning", "badge--error"];

fn badge_class(options: &[String], value: &str) -> &'static str {
    match options.iter().position(|o| o == value) {
        Some(i) => BADGE_PALETTE[i % BADGE_PALETTE.len()],
        None => "badge",
    }
}

fn kind_label(kind: &ColumnKind) -> &'static str {
    match kind {
        ColumnKind::Text => "texto",
        ColumnKind::Checkbox => "checkbox",
        ColumnKind::Select { .. } => "selecao",
    }
}

/// Abre o fluxo encadeado de configuração de coluna: nome → tipo → (se
/// seleção) opções. Cada etapa é um `Prompt` — mais simples que um formulário
/// multi-campo dedicado nesta rodada.
fn configure_column(
    open_dialog: &Callback<PendingDialog>,
    current_name: String,
    current_kind: ColumnKind,
    on_done: Callback<(String, ColumnKind)>,
) {
    let open_dialog_kind = open_dialog.clone();
    let current_kind_label = kind_label(&current_kind).to_string();
    let current_options = match &current_kind {
        ColumnKind::Select { options } => options.join(", "),
        _ => String::new(),
    };
    open_dialog.emit(PendingDialog::Prompt {
        title: "Nome da coluna".to_string(),
        default: current_name,
        on_submit: Callback::from(move |name: String| {
            let open_dialog_options = open_dialog_kind.clone();
            let on_done = on_done.clone();
            let name_for_kind = name.clone();
            let current_options = current_options.clone();
            open_dialog_kind.emit(PendingDialog::Prompt {
                title: "Tipo da coluna (texto / selecao / checkbox)".to_string(),
                default: current_kind_label.clone(),
                on_submit: Callback::from(move |kind_input: String| {
                    let name = name_for_kind.clone();
                    match kind_input.trim().to_lowercase().as_str() {
                        "selecao" | "seleção" | "select" => {
                            let on_done = on_done.clone();
                            let name = name.clone();
                            open_dialog_options.emit(PendingDialog::Prompt {
                                title: "Opções (separadas por vírgula)".to_string(),
                                default: current_options.clone(),
                                on_submit: Callback::from(move |opts: String| {
                                    let options = opts.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect();
                                    on_done.emit((name.clone(), ColumnKind::Select { options }));
                                }),
                            });
                        }
                        "checkbox" | "check" => on_done.emit((name, ColumnKind::Checkbox)),
                        _ => on_done.emit((name, ColumnKind::Text)),
                    }
                }),
            });
        }),
    });
}

/// Tabela inline com colunas tipadas e células editáveis.
#[function_component(InlineTable)]
pub fn inline_table(props: &InlineTableProps) -> Html {
    let open_select_cell = use_state(|| None::<(usize, usize)>);

    // Fecha o dropdown de seleção ao clicar fora da célula ou apertar
    // Escape — antes só fechava escolhendo uma opção.
    {
        let open_select_cell = open_select_cell.clone();
        use_effect_with(*open_select_cell, move |open| {
            let mut listeners = Vec::new();
            if open.is_some() {
                let window = web_sys::window().expect("no global window");

                let close_cell = open_select_cell.clone();
                listeners.push(EventListener::new(&window, "mousedown", move |e| {
                    let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                    let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                    let Some(target) = target else { return };
                    if target.closest(".task-table__td--select").ok().flatten().is_none() {
                        close_cell.set(None);
                    }
                }));

                let close_cell = open_select_cell.clone();
                listeners.push(EventListener::new(&window, "keydown", move |e| {
                    if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                        if e.key() == "Escape" {
                            close_cell.set(None);
                        }
                    }
                }));
            }
            move || drop(listeners)
        });
    }

    let add_row = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_data = data.clone();
            new_data.add_row();
            on_change.emit(new_data);
        })
    };

    let add_column = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nome da nova coluna".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |name: String| {
                    let mut new_data = data.clone();
                    new_data.add_column(name);
                    on_change.emit(new_data);
                }),
            });
        })
    };

    let n_cols = props.data.columns.len();

    html! {
        <div class="embed-table">
            <table class="task-table__table">
                <thead>
                    <tr>
                        { for props.data.columns.iter().enumerate().map(|(ci, col)| {
                            let settings = {
                                let data = props.data.clone();
                                let on_change = props.on_change.clone();
                                let open_dialog = props.open_dialog.clone();
                                let name = col.name.clone();
                                let kind = col.kind.clone();
                                Callback::from(move |_: MouseEvent| {
                                    let data = data.clone();
                                    let on_change = on_change.clone();
                                    configure_column(&open_dialog, name.clone(), kind.clone(), Callback::from(move |(new_name, new_kind): (String, ColumnKind)| {
                                        let mut new_data = data.clone();
                                        new_data.set_column_name(ci, new_name);
                                        new_data.set_column_kind(ci, new_kind);
                                        on_change.emit(new_data);
                                    }));
                                })
                            };
                            let delete_col = {
                                let data = props.data.clone();
                                let on_change = props.on_change.clone();
                                let open_dialog = props.open_dialog.clone();
                                let name = col.name.clone();
                                Callback::from(move |_: MouseEvent| {
                                    let data = data.clone();
                                    let on_change = on_change.clone();
                                    open_dialog.emit(PendingDialog::Confirm {
                                        message: format!("Excluir coluna \"{}\"?", name),
                                        confirm_label: "Excluir".to_string(),
                                        on_confirm: Callback::from(move |_| {
                                            let mut new_data = data.clone();
                                            new_data.remove_column(ci);
                                            on_change.emit(new_data);
                                        }),
                                    });
                                })
                            };
                            html! {
                                <th class="task-table__th">
                                    <span class="task-table__th-name">{ &col.name }</span>
                                    <button class="task-table__th-action" onclick={settings} title="Configurar coluna">{ "⚙" }</button>
                                    <button class="task-table__th-action" onclick={delete_col} title="Excluir coluna">{ "✕" }</button>
                                </th>
                            }
                        }) }
                        <th class="task-table__th task-table__th--add">
                            <button class="task-table__add" onclick={add_column} title="Nova coluna">{ "+" }</button>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    { for props.data.rows.iter().enumerate().map(|(ri, row)| {
                        let delete_row = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            let open_dialog = props.open_dialog.clone();
                            Callback::from(move |_: MouseEvent| {
                                let data = data.clone();
                                let on_change = on_change.clone();
                                open_dialog.emit(PendingDialog::Confirm {
                                    message: "Excluir esta linha?".to_string(),
                                    confirm_label: "Excluir".to_string(),
                                    on_confirm: Callback::from(move |_| {
                                        let mut new_data = data.clone();
                                        new_data.remove_row(ri);
                                        on_change.emit(new_data);
                                    }),
                                });
                            })
                        };
                        html! {
                            <tr class="task-table__row">
                                { for row.iter().enumerate().map(|(ci, cell)| {
                                    let kind = props.data.columns.get(ci).map(|c| c.kind.clone()).unwrap_or(ColumnKind::Text);
                                    match kind {
                                        ColumnKind::Checkbox => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            let checked = cell == "true";
                                            let toggle = Callback::from(move |_: MouseEvent| {
                                                let mut new_data = data.clone();
                                                new_data.set_cell(ri, ci, if checked { "false".to_string() } else { "true".to_string() });
                                                on_change.emit(new_data);
                                            });
                                            html! {
                                                <td class="task-table__td">
                                                    <input type="checkbox" {checked} onclick={toggle} />
                                                </td>
                                            }
                                        }
                                        ColumnKind::Select { options } => {
                                            let is_open = *open_select_cell == Some((ri, ci));
                                            let cell_value = cell.clone();
                                            let toggle_open = {
                                                let open_select_cell = open_select_cell.clone();
                                                Callback::from(move |_: MouseEvent| {
                                                    open_select_cell.set(if is_open { None } else { Some((ri, ci)) });
                                                })
                                            };
                                            html! {
                                                <td class="task-table__td task-table__td--select">
                                                    <span class={classes!("badge", badge_class(&options, &cell_value))} onclick={toggle_open}>
                                                        { if cell_value.is_empty() { "—".to_string() } else { cell_value.clone() } }
                                                    </span>
                                                    if is_open {
                                                        <div class="table-select-menu">
                                                            { for options.iter().map(|opt| {
                                                                let data = props.data.clone();
                                                                let on_change = props.on_change.clone();
                                                                let open_select_cell = open_select_cell.clone();
                                                                let opt_value = opt.clone();
                                                                let pick = Callback::from(move |_: MouseEvent| {
                                                                    let mut new_data = data.clone();
                                                                    new_data.set_cell(ri, ci, opt_value.clone());
                                                                    on_change.emit(new_data);
                                                                    open_select_cell.set(None);
                                                                });
                                                                html! {
                                                                    <div class="table-select-menu__item" onclick={pick}>{ opt }</div>
                                                                }
                                                            }) }
                                                        </div>
                                                    }
                                                </td>
                                            }
                                        }
                                        ColumnKind::Text => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            let onblur = Callback::from(move |e: FocusEvent| {
                                                let Some(target) = e.target() else { return };
                                                let Ok(el) = target.dyn_into::<web_sys::Element>() else { return };
                                                let text = el.text_content().unwrap_or_default();
                                                let mut new_data = data.clone();
                                                new_data.set_cell(ri, ci, text);
                                                on_change.emit(new_data);
                                            });
                                            html! {
                                                <td class="task-table__td" contenteditable="true" onblur={onblur}>{ cell }</td>
                                            }
                                        }
                                    }
                                }) }
                                <td class="task-table__td task-table__td--actions">
                                    <button class="task-table__row-delete" onclick={delete_row} title="Excluir linha">{ "✕" }</button>
                                </td>
                            </tr>
                        }
                    }) }
                    <tr class="task-table__row task-table__row--add">
                        <td class="task-table__td" colspan={(n_cols + 1).to_string()}>
                            <button class="task-table__add" onclick={add_row}>{ "+ linha" }</button>
                        </td>
                    </tr>
                </tbody>
            </table>
        </div>
    }
}
