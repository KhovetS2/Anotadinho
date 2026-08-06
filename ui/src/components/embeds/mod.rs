//! Componentes Yew pra embeds inline. Ponto de extensão: um novo tipo de
//! embed ganha 1 componente aqui + 1 braço de match em `InlineEmbed` — o
//! resto (segmentação, parsing, editor) não muda.

mod inline_calendar;
mod inline_kanban;
mod inline_table;

pub use inline_calendar::InlineCalendar;
pub use inline_kanban::InlineKanban;
pub use inline_table::InlineTable;

use yew::prelude::*;

use crate::dialog::PendingDialog;
use crate::embed::EmbedData;

/// Props do `InlineEmbed`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineEmbedProps {
    /// Dados do embed (o tipo já vem carregado em `EmbedData::kind()`).
    pub data: EmbedData,
    /// Disparado quando o embed é editado (drag de card, edição de célula, etc).
    pub on_change: Callback<EmbedData>,
    /// Abre o modal de diálogo do app (ver `crate::dialog`).
    pub open_dialog: Callback<PendingDialog>,
}

/// Dispatcher: renderiza o componente certo pro tipo de `EmbedData`.
#[function_component(InlineEmbed)]
pub fn inline_embed(props: &InlineEmbedProps) -> Html {
    match &props.data {
        EmbedData::Kanban(d) => {
            let on_change = props.on_change.clone();
            html! {
                <InlineKanban data={d.clone()} on_change={Callback::from(move |d| on_change.emit(EmbedData::Kanban(d)))} open_dialog={props.open_dialog.clone()} />
            }
        }
        EmbedData::Calendar(d) => {
            let on_change = props.on_change.clone();
            html! {
                <InlineCalendar data={d.clone()} on_change={Callback::from(move |d| on_change.emit(EmbedData::Calendar(d)))} open_dialog={props.open_dialog.clone()} />
            }
        }
        EmbedData::Table(d) => {
            let on_change = props.on_change.clone();
            html! {
                <InlineTable data={d.clone()} on_change={Callback::from(move |d| on_change.emit(EmbedData::Table(d)))} open_dialog={props.open_dialog.clone()} />
            }
        }
    }
}
