//! Tabela inline (dentro de uma fence ```table), com células editáveis.
//! Reaproveita as classes `.task-table__*` (já estilizadas globalmente,
//! não presas ao escopo `.editor__wysiwyg`) em vez de criar CSS novo.

use wasm_bindgen::JsCast;
use web_sys::FocusEvent;
use yew::prelude::*;

use crate::embed::TableEmbedData;

/// Props do `InlineTable`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineTableProps {
    /// Cabeçalhos + linhas.
    pub data: TableEmbedData,
    /// Disparado quando uma célula é editada.
    pub on_change: Callback<TableEmbedData>,
}

/// Tabela inline com células editáveis.
#[function_component(InlineTable)]
pub fn inline_table(props: &InlineTableProps) -> Html {
    html! {
        <div class="embed-table">
            <table class="task-table__table">
                <thead>
                    <tr>
                        { for props.data.headers.iter().map(|h| html! { <th class="task-table__th">{ h }</th> }) }
                    </tr>
                </thead>
                <tbody>
                    { for props.data.rows.iter().enumerate().map(|(ri, row)| {
                        html! {
                            <tr class="task-table__row">
                                { for row.iter().enumerate().map(|(ci, cell)| {
                                    let data = props.data.clone();
                                    let on_change = props.on_change.clone();
                                    let onblur = Callback::from(move |e: FocusEvent| {
                                        let Some(target) = e.target() else { return };
                                        let Ok(el) = target.dyn_into::<web_sys::Element>() else { return };
                                        let text = el.text_content().unwrap_or_default();
                                        let mut new_data = data.clone();
                                        if let Some(cell) = new_data.rows.get_mut(ri).and_then(|r| r.get_mut(ci)) {
                                            if *cell != text {
                                                *cell = text;
                                                on_change.emit(new_data);
                                            }
                                        }
                                    });
                                    html! {
                                        <td class="task-table__td" contenteditable="true" onblur={onblur}>{ cell }</td>
                                    }
                                }) }
                            </tr>
                        }
                    }) }
                </tbody>
            </table>
        </div>
    }
}
