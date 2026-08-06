//! Board kanban inline (dentro de uma fence ```kanban), dinâmico: criar/
//! editar/excluir card, criar/renomear/excluir coluna, mover card entre
//! colunas. Diálogos usam o modal do app (`crate::dialog`), não
//! `window.prompt`/`confirm`.
//!
//! Drag-and-drop é feito com eventos de mouse simples (mousedown/
//! mouseenter/mouseup) em vez do Drag and Drop nativo do HTML5
//! (draggable/dragstart/dragover/drop) — o WebKitGTK (engine do Tauri no
//! Linux) tem suporte historicamente instável a essa API nativa.

use yew::prelude::*;

use crate::dialog::PendingDialog;
use crate::embed::{KanbanEmbedData, KanbanEmbedItem};

/// Props do `InlineKanban`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineKanbanProps {
    /// Dados do board (colunas + cards).
    pub data: KanbanEmbedData,
    /// Disparado quando o board muda (card movido/criado/editado/excluído,
    /// coluna criada/renomeada/excluída).
    pub on_change: Callback<KanbanEmbedData>,
    /// Abre o modal de diálogo do app.
    pub open_dialog: Callback<PendingDialog>,
}

/// Board kanban inline.
#[function_component(InlineKanban)]
pub fn inline_kanban(props: &InlineKanbanProps) -> Html {
    let dragging = use_state(|| None::<usize>);

    let board_onmouseup = {
        let dragging = dragging.clone();
        Callback::from(move |_: MouseEvent| dragging.set(None))
    };

    html! {
        <div class="kanban embed-kanban">
            <div class="kanban__board" onmouseup={board_onmouseup}>
                { for props.data.columns.iter().enumerate().map(|(col_idx, col)| {
                    let items: Vec<(usize, &KanbanEmbedItem)> = props.data.items
                        .iter()
                        .enumerate()
                        .filter(|(_, it)| &it.column == col)
                        .collect();

                    let ondrop_mouseup = {
                        let col_name = col.clone();
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let dragging = dragging.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            if let Some(idx) = *dragging {
                                let mut new_data = data.clone();
                                if let Some(item) = new_data.items.get_mut(idx) {
                                    item.column = col_name.clone();
                                }
                                on_change.emit(new_data);
                            }
                            dragging.set(None);
                        })
                    };

                    let rename_column = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let current_name = col.clone();
                        Callback::from(move |_: MouseEvent| {
                            let data = data.clone();
                            let on_change = on_change.clone();
                            open_dialog.emit(PendingDialog::Prompt {
                                title: "Renomear coluna".to_string(),
                                default: current_name.clone(),
                                on_submit: Callback::from(move |new_name: String| {
                                    let mut new_data = data.clone();
                                    new_data.rename_column(col_idx, new_name);
                                    on_change.emit(new_data);
                                }),
                            });
                        })
                    };

                    let delete_column = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let col_name = col.clone();
                        let n_cards = items.len();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            let data = data.clone();
                            let on_change = on_change.clone();
                            let message = if n_cards > 0 {
                                format!("Excluir coluna \"{}\"? {} card(s) serão excluídos junto.", col_name, n_cards)
                            } else {
                                format!("Excluir coluna \"{}\"?", col_name)
                            };
                            open_dialog.emit(PendingDialog::Confirm {
                                message,
                                confirm_label: "Excluir".to_string(),
                                on_confirm: Callback::from(move |_| {
                                    let mut new_data = data.clone();
                                    new_data.remove_column(col_idx);
                                    on_change.emit(new_data);
                                }),
                            });
                        })
                    };

                    let add_card = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let col_name = col.clone();
                        Callback::from(move |_: MouseEvent| {
                            let data = data.clone();
                            let on_change = on_change.clone();
                            let col_name = col_name.clone();
                            open_dialog.emit(PendingDialog::Prompt {
                                title: "Novo card".to_string(),
                                default: String::new(),
                                on_submit: Callback::from(move |title: String| {
                                    let mut new_data = data.clone();
                                    new_data.add_card(col_name.clone(), title);
                                    on_change.emit(new_data);
                                }),
                            });
                        })
                    };

                    let column_class = if dragging.is_some() {
                        "kanban__column kanban__column--drag-active"
                    } else {
                        "kanban__column"
                    };

                    html! {
                        <div class={column_class} onmouseup={ondrop_mouseup}>
                            <div class="kanban__col-header">
                                <span class="kanban__col-title" onclick={rename_column} title="Renomear coluna">{ col }</span>
                                <span class="kanban__col-count">{ items.len() }</span>
                                <button class="kanban__col-delete" onclick={delete_column} title="Excluir coluna">{ "✕" }</button>
                            </div>
                            <div class="kanban__col-body">
                                { for items.into_iter().map(|(idx, item)| {
                                    let dragging_start = dragging.clone();
                                    let onmousedown = Callback::from(move |_: MouseEvent| dragging_start.set(Some(idx)));

                                    let edit_card = {
                                        let data = props.data.clone();
                                        let on_change = props.on_change.clone();
                                        let open_dialog = props.open_dialog.clone();
                                        let current_title = item.title.clone();
                                        Callback::from(move |e: MouseEvent| {
                                            e.stop_propagation();
                                            let data = data.clone();
                                            let on_change = on_change.clone();
                                            open_dialog.emit(PendingDialog::Prompt {
                                                title: "Editar card".to_string(),
                                                default: current_title.clone(),
                                                on_submit: Callback::from(move |new_title: String| {
                                                    let mut new_data = data.clone();
                                                    new_data.edit_card(idx, new_title);
                                                    on_change.emit(new_data);
                                                }),
                                            });
                                        })
                                    };
                                    let delete_card = {
                                        let data = props.data.clone();
                                        let on_change = props.on_change.clone();
                                        let open_dialog = props.open_dialog.clone();
                                        let title = item.title.clone();
                                        Callback::from(move |e: MouseEvent| {
                                            e.stop_propagation();
                                            let data = data.clone();
                                            let on_change = on_change.clone();
                                            open_dialog.emit(PendingDialog::Confirm {
                                                message: format!("Excluir card \"{}\"?", title),
                                                confirm_label: "Excluir".to_string(),
                                                on_confirm: Callback::from(move |_| {
                                                    let mut new_data = data.clone();
                                                    new_data.remove_card(idx);
                                                    on_change.emit(new_data);
                                                }),
                                            });
                                        })
                                    };
                                    let stop_mousedown = Callback::from(|e: MouseEvent| e.stop_propagation());

                                    html! {
                                        <div class="kanban__card" {onmousedown}>
                                            <span class="kanban__card-title">{ &item.title }</span>
                                            <span class="kanban__card-actions">
                                                <button class="kanban__card-action" onmousedown={stop_mousedown.clone()} onclick={edit_card} title="Editar">{ "✎" }</button>
                                                <button class="kanban__card-action" onmousedown={stop_mousedown} onclick={delete_card} title="Excluir">{ "✕" }</button>
                                            </span>
                                        </div>
                                    }
                                }) }
                                <button class="kanban__add-card" onclick={add_card}>{ "+ card" }</button>
                            </div>
                        </div>
                    }
                }) }
                <div class="kanban__add-column">
                    <button class="kanban__add-column-btn" onclick={{
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        Callback::from(move |_: MouseEvent| {
                            let data = data.clone();
                            let on_change = on_change.clone();
                            open_dialog.emit(PendingDialog::Prompt {
                                title: "Nova coluna".to_string(),
                                default: String::new(),
                                on_submit: Callback::from(move |name: String| {
                                    let mut new_data = data.clone();
                                    new_data.add_column(name);
                                    on_change.emit(new_data);
                                }),
                            });
                        })
                    }}>{ "+ coluna" }</button>
                </div>
            </div>
        </div>
    }
}
