//! Board kanban inline (dentro de uma fence ```kanban), interativo de
//! verdade: arrastar um card entre colunas atualiza o `EmbedData` via
//! `on_change`, que o editor regrava na fence e salva.

use yew::prelude::*;

use crate::embed::KanbanEmbedData;

/// Props do `InlineKanban`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineKanbanProps {
    /// Dados do board (colunas + cards).
    pub data: KanbanEmbedData,
    /// Disparado quando o board muda (ex: card mudou de coluna).
    pub on_change: Callback<KanbanEmbedData>,
}

/// Board kanban inline.
#[function_component(InlineKanban)]
pub fn inline_kanban(props: &InlineKanbanProps) -> Html {
    let dragging = use_state(|| None::<usize>);

    html! {
        <div class="kanban embed-kanban">
            <div class="kanban__board">
                { for props.data.columns.iter().map(|col| {
                    let items: Vec<(usize, &crate::embed::KanbanEmbedItem)> = props.data.items
                        .iter()
                        .enumerate()
                        .filter(|(_, it)| &it.column == col)
                        .collect();

                    let ondrop = {
                        let col_name = col.clone();
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let dragging = dragging.clone();
                        Callback::from(move |e: DragEvent| {
                            e.prevent_default();
                            if let Some(idx) = *dragging {
                                let mut new_data = data.clone();
                                if let Some(item) = new_data.items.get_mut(idx) {
                                    item.column = col_name.clone();
                                }
                                on_change.emit(new_data);
                            }
                        })
                    };
                    let ondragover = Callback::from(|e: DragEvent| e.prevent_default());

                    html! {
                        <div class="kanban__column" ondrop={ondrop} ondragover={ondragover}>
                            <div class="kanban__col-header">
                                <span class="kanban__col-title">{ col }</span>
                                <span class="kanban__col-count">{ items.len() }</span>
                            </div>
                            <div class="kanban__col-body">
                                { for items.into_iter().map(|(idx, item)| {
                                    let dragging = dragging.clone();
                                    let ondragstart = Callback::from(move |_: DragEvent| dragging.set(Some(idx)));
                                    html! {
                                        <div class="kanban__card" draggable="true" ondragstart={ondragstart}>
                                            <span class="kanban__card-title">{ &item.title }</span>
                                        </div>
                                    }
                                }) }
                            </div>
                        </div>
                    }
                }) }
            </div>
        </div>
    }
}
