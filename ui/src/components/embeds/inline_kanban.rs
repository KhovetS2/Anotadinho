//! Board kanban inline (dentro de um wrapper `{{ type: "kanban" }}`),
//! dinâmico: criar/editar/excluir card, criar/renomear/excluir coluna,
//! mover card entre colunas E reordenar dentro da mesma coluna. Clicar num
//! card (sem arrastar) abre o modal de detalhes (`CardDetailModal`).
//! Diálogos simples usam o modal do app (`crate::dialog`), não
//! `window.prompt`/`confirm`.
//!
//! Drag-and-drop é feito com eventos de mouse simples (mousedown/mouseup)
//! em vez do Drag and Drop nativo do HTML5 — o WebKitGTK (engine do Tauri
//! no Linux) tem suporte historicamente instável a essa API nativa. Um
//! listener de `mouseup` no `window` inteiro garante que o estado de
//! arraste nunca fica preso, mesmo se o usuário soltar o mouse fora da
//! área do board.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::embeds::CardDetailModal;
use crate::components::icon::Icon;
use crate::dialog::PendingDialog;
use crate::embed::{KanbanCard, KanbanEmbedData};

/// Props do `InlineKanban`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineKanbanProps {
    /// Dados do board (colunas + cards).
    pub data: KanbanEmbedData,
    /// Path do vault (pra anexos no modal de detalhes).
    pub vault_path: String,
    /// Disparado quando o board muda (card movido/criado/editado/excluído,
    /// coluna criada/renomeada/excluída).
    pub on_change: Callback<KanbanEmbedData>,
    /// Abre o modal de diálogo do app.
    pub open_dialog: Callback<PendingDialog>,
    /// Id do grupo de navegação por teclado (ciclo 165), gerado pelo
    /// editor por segmento.
    pub nav_group: String,
}

/// Board kanban inline.
#[function_component(InlineKanban)]
pub fn inline_kanban(props: &InlineKanbanProps) -> Html {
    let dragging = use_state(|| None::<usize>);
    let editing_card = use_state(|| None::<usize>);
    let drag_pos = use_state(|| None::<(i32, i32)>);
    let hover_card = use_state(|| None::<usize>);

    // Garante que o estado de arraste nunca fica preso: se o usuário
    // soltar o mouse fora de qualquer card/coluna (sidebar, outro embed,
    // etc.), nenhum onmouseup local dispara, mas este listener global
    // sempre roda e zera `dragging`. Handlers específicos de card/coluna
    // também zeram por conta própria — redundante com este, mas inofensivo.
    {
        let dragging = dragging.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no global window");
            let listener = EventListener::new(&window, "mouseup", move |_event| {
                dragging.set(None);
            });
            move || drop(listener)
        });
    }

    // Ghost que segue o cursor durante o arraste — só existe um listener
    // de mousemove enquanto `dragging` está ativo (liga/desliga junto).
    {
        let drag_pos = drag_pos.clone();
        use_effect_with(*dragging, move |dragging| {
            let listener = dragging.map(|_| {
                let window = web_sys::window().expect("no global window");
                let drag_pos = drag_pos.clone();
                EventListener::new(&window, "mousemove", move |e| {
                    if let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() {
                        drag_pos.set(Some((e.client_x(), e.client_y())));
                    }
                })
            });
            if dragging.is_none() {
                drag_pos.set(None);
            }
            move || drop(listener)
        });
    }

    let board_onmouseup = {
        let dragging = dragging.clone();
        Callback::from(move |_: MouseEvent| dragging.set(None))
    };

    let close_detail = {
        let editing_card = editing_card.clone();
        Callback::from(move |_: ()| editing_card.set(None))
    };

    let card_detail = editing_card.and_then(|idx| {
        props.data.items.get(idx).cloned().map(|card| {
            let data = props.data.clone();
            let on_change = props.on_change.clone();
            let on_card_change = Callback::from(move |updated: KanbanCard| {
                let mut new_data = data.clone();
                new_data.update_card(idx, updated);
                on_change.emit(new_data);
            });
            html! {
                <CardDetailModal
                    {card}
                    vault_path={props.vault_path.clone()}
                    on_change={on_card_change}
                    on_close={close_detail.clone()}
                    open_dialog={props.open_dialog.clone()}
                />
            }
        })
    });

    html! {
        <div class="kanban embed-kanban" data-nav-group={props.nav_group.clone()} data-nav-item={props.nav_group.clone()} data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
            <div class="kanban__board" onmouseup={board_onmouseup}>
                { for props.data.columns.iter().enumerate().map(|(col_idx, col)| {
                    let items: Vec<(usize, &KanbanCard)> = props.data.items
                        .iter()
                        .enumerate()
                        .filter(|(_, it)| &it.column == col)
                        .collect();

                    // Soltar no fundo vazio da coluna (não em cima de um
                    // card específico): acrescenta no fim da coluna.
                    let column_onmouseup = {
                        let col_name = col.clone();
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let dragging = dragging.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            if let Some(idx) = *dragging {
                                let mut new_data = data.clone();
                                new_data.move_card(idx, col_name.clone(), None);
                                on_change.emit(new_data);
                            }
                            dragging.set(None);
                        })
                    };

                    // Uma ação, dois gatilhos: clique e Enter/Espaço
                    // (ciclo 165). `Callback<()>` é o formato que o
                    // `keyboard_activate` espera; o clique adapta com
                    // `reform`.
                    let rename_column_kb: Callback<()> = {
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let current_name = col.clone();
                        Callback::from(move |_: ()| {
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
                        <div class={column_class} onmouseup={column_onmouseup}>
                            <div class="kanban__col-header">
                                <span class="kanban__col-title" tabindex="0" role="button" onclick={rename_column_kb.reform(|_: MouseEvent| ())} title="Renomear coluna"
                                    data-nav-item="kanban-column" data-nav-parent={props.nav_group.clone()}
                                    onkeydown={crate::keyboard_activate::activate_on_enter_or_space(rename_column_kb.clone())}>{ col }</span>
                                <span class="kanban__col-count">{ items.len() }</span>
                                <button class="kanban__col-delete" onclick={delete_column} title="Excluir coluna"
                                    data-nav-item="kanban-column-delete" data-nav-parent={props.nav_group.clone()}><Icon name="x" /></button>
                            </div>
                            <div class="kanban__col-body">
                                { for items.into_iter().map(|(idx, item)| {
                                    let col_name = col.clone();

                                    let onmousedown = {
                                        let dragging = dragging.clone();
                                        Callback::from(move |e: MouseEvent| {
                                            // Sem isso, mousedown+mover o mouse seleciona o
                                            // texto por baixo do cursor e o navegador começa
                                            // um drag nativo de conteúdo (a "sombra do
                                            // container" que aparecia arrastando rápido) —
                                            // atrapalha o nosso drag por mouse próprio.
                                            e.prevent_default();
                                            dragging.set(Some(idx));
                                        })
                                    };

                                    // Só relevante durante um arraste — indica onde o
                                    // card vai ser inserido se soltar aqui.
                                    let onmouseenter = {
                                        let dragging = dragging.clone();
                                        let hover_card = hover_card.clone();
                                        Callback::from(move |_: MouseEvent| {
                                            if dragging.is_some() {
                                                hover_card.set(Some(idx));
                                            }
                                        })
                                    };

                                    // Soltar em cima de OUTRO card: se não
                                    // houve arraste de verdade (soltou no
                                    // mesmo card que começou), é um clique —
                                    // abre o modal de detalhes. Senão,
                                    // reordena/move pra antes deste card.
                                    let onmouseup = {
                                        let data = props.data.clone();
                                        let on_change = props.on_change.clone();
                                        let dragging = dragging.clone();
                                        let editing_card = editing_card.clone();
                                        let col_name = col_name.clone();
                                        Callback::from(move |e: MouseEvent| {
                                            e.stop_propagation();
                                            if let Some(from) = *dragging {
                                                if from == idx {
                                                    editing_card.set(Some(idx));
                                                } else {
                                                    let mut new_data = data.clone();
                                                    new_data.move_card(from, col_name.clone(), Some(idx));
                                                    on_change.emit(new_data);
                                                }
                                            }
                                            dragging.set(None);
                                        })
                                    };

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
                                                title: "Editar título".to_string(),
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

                                    // Ciclo 135: mesma ação de "abrir
                                    // detalhes" que soltar o mouse no
                                    // próprio card (sem arrastar) já
                                    // dispara — cards inline nunca
                                    // tinham NENHUM suporte de teclado.
                                    // Enter/Espaço abrem o card; Alt+setas
                                    // MOVEM (ciclo 167) — o arraste era a
                                    // única forma de reorganizar o board, e
                                    // arrastar não tem equivalente de
                                    // teclado. Alt libera as setas puras pra
                                    // continuarem navegando (nav-mode).
                                    let onkeydown = {
                                        let abrir = crate::keyboard_activate::activate_on_enter_or_space({
                                            let editing_card = editing_card.clone();
                                            Callback::from(move |_: ()| editing_card.set(Some(idx)))
                                        });
                                        let data = props.data.clone();
                                        let on_change = props.on_change.clone();
                                        let coluna_atual = col.clone();
                                        Callback::from(move |e: KeyboardEvent| {
                                            if !e.alt_key() {
                                                abrir.emit(e);
                                                return;
                                            }
                                            let colunas = data.columns.clone();
                                            let pos_coluna = colunas.iter().position(|c| c == &coluna_atual);
                                            let mut novo = data.clone();
                                            match e.key().as_str() {
                                                "ArrowLeft" | "ArrowRight" => {
                                                    let Some(pos) = pos_coluna else { return };
                                                    let destino = if e.key() == "ArrowLeft" {
                                                        pos.checked_sub(1)
                                                    } else if pos + 1 < colunas.len() {
                                                        Some(pos + 1)
                                                    } else {
                                                        None
                                                    };
                                                    let Some(destino) = destino else { return };
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    novo.move_card(idx, colunas[destino].clone(), None);
                                                    on_change.emit(novo);
                                                }
                                                "ArrowUp" | "ArrowDown" => {
                                                    // Reordenar dentro da coluna: acha o
                                                    // vizinho na MESMA coluna e insere
                                                    // antes/depois dele.
                                                    let irmaos: Vec<usize> = data
                                                        .items
                                                        .iter()
                                                        .enumerate()
                                                        .filter(|(_, it)| it.column == coluna_atual)
                                                        .map(|(i, _)| i)
                                                        .collect();
                                                    let Some(pos) = irmaos.iter().position(|i| *i == idx) else { return };
                                                    let alvo = if e.key() == "ArrowUp" {
                                                        pos.checked_sub(1).map(|p| irmaos[p])
                                                    } else {
                                                        irmaos.get(pos + 1).copied()
                                                    };
                                                    let Some(alvo) = alvo else { return };
                                                    e.prevent_default();
                                                    e.stop_propagation();
                                                    let antes = if e.key() == "ArrowUp" {
                                                        Some(alvo)
                                                    } else {
                                                        irmaos.get(pos + 2).copied()
                                                    };
                                                    novo.move_card(idx, coluna_atual.clone(), antes);
                                                    on_change.emit(novo);
                                                }
                                                _ => {}
                                            }
                                        })
                                    };

                                    let card_class = if *dragging == Some(idx) {
                                        "kanban__card kanban__card--dragging"
                                    } else {
                                        "kanban__card"
                                    };

                                    let show_insertion = dragging.is_some() && *dragging != Some(idx) && *hover_card == Some(idx);

                                    let has_extras = item.description.is_some()
                                        || !item.tags.is_empty()
                                        || item.due.is_some()
                                        || !item.checklist.is_empty()
                                        || !item.comments.is_empty()
                                        || !item.attachments.is_empty();

                                    html! {
                                        <>
                                        if show_insertion {
                                            <div class="kanban__insertion-line" />
                                        }
                                        <div class={card_class} tabindex="0" {onmousedown} {onmouseup} {onmouseenter} {onkeydown}
                                            data-nav-item="kanban-card" data-nav-parent={props.nav_group.clone()}>
                                            <div class="kanban__card-main">
                                                <span class="kanban__card-title">{ &item.title }</span>
                                                <span class="kanban__card-actions">
                                                    <button class="kanban__card-action" onmousedown={stop_mousedown.clone()} onclick={edit_card} title="Editar título"
                                                        data-nav-item="kanban-card-edit" data-nav-parent={props.nav_group.clone()}><Icon name="edit" /></button>
                                                    <button class="kanban__card-action" onmousedown={stop_mousedown} onclick={delete_card} title="Excluir"
                                                        data-nav-item="kanban-card-delete" data-nav-parent={props.nav_group.clone()}><Icon name="x" /></button>
                                                </span>
                                            </div>
                                            if has_extras {
                                                <div class="kanban__card-meta">
                                                    if !item.checklist.is_empty() {
                                                        <span class="kanban__card-badge">
                                                            <Icon name="check-square" />
                                                            { format!(" {}/{}", item.checklist.iter().filter(|c| c.done).count(), item.checklist.len()) }
                                                        </span>
                                                    }
                                                    if let Some(due) = &item.due {
                                                        <span class="kanban__card-badge"><Icon name="calendar" />{ format!(" {due}") }</span>
                                                    }
                                                    if !item.comments.is_empty() {
                                                        <span class="kanban__card-badge"><Icon name="message-circle" />{ format!(" {}", item.comments.len()) }</span>
                                                    }
                                                    if !item.attachments.is_empty() {
                                                        <span class="kanban__card-badge"><Icon name="paperclip" />{ format!(" {}", item.attachments.len()) }</span>
                                                    }
                                                </div>
                                            }
                                            if !item.tags.is_empty() {
                                                <div class="kanban__card-tags">
                                                    { for item.tags.iter().map(|t| html! { <span class="badge badge--info">{ t }</span> }) }
                                                </div>
                                            }
                                        </div>
                                        </>
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
            if let (Some(idx), Some((x, y))) = (*dragging, *drag_pos) {
                if let Some(item) = props.data.items.get(idx) {
                    <div class="kanban__drag-ghost" style={format!("left: {}px; top: {}px;", x + 12, y + 12)}>
                        { &item.title }
                    </div>
                }
            }
            { for card_detail }
        </div>
    }
}
