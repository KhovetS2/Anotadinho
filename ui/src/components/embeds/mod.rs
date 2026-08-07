//! Componentes Yew pra embeds inline. Ponto de extensão: um novo tipo de
//! embed ganha 1 componente aqui + 1 braço de match em `InlineEmbed` — o
//! resto (segmentação, parsing, editor) não muda.

mod card_detail_modal;
mod column_settings_modal;
mod event_detail_modal;
mod inline_calendar;
mod inline_kanban;
mod inline_table;

pub use card_detail_modal::CardDetailModal;
pub use column_settings_modal::ColumnSettingsModal;
pub use event_detail_modal::EventDetailModal;
pub use inline_calendar::InlineCalendar;
pub use inline_kanban::InlineKanban;
pub use inline_table::InlineTable;

use yew::prelude::*;

use crate::api::PageMeta;
use crate::dialog::PendingDialog;
use crate::embed::EmbedData;

/// Props do `InlineEmbed`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineEmbedProps {
    /// Dados do embed (o tipo já vem carregado em `EmbedData::kind()`).
    pub data: EmbedData,
    /// Path do vault (kanban usa pra anexos, table usa pra listar páginas
    /// do vault na célula de tipo Página).
    pub vault_path: String,
    /// Disparado quando o embed é editado (drag de card, edição de célula, etc).
    pub on_change: Callback<EmbedData>,
    /// Abre o modal de diálogo do app (ver `crate::dialog`).
    pub open_dialog: Callback<PendingDialog>,
    /// Navega pra outra página do vault — usado pela célula de tipo
    /// Página da tabela, pra abrir a página vinculada.
    pub on_page_selected: Callback<PageMeta>,
}

/// Dispatcher: renderiza o componente certo pro tipo de `EmbedData`.
#[function_component(InlineEmbed)]
pub fn inline_embed(props: &InlineEmbedProps) -> Html {
    match &props.data {
        EmbedData::Kanban(d) => {
            let on_change = props.on_change.clone();
            html! {
                <InlineKanban data={d.clone()} vault_path={props.vault_path.clone()} on_change={Callback::from(move |d| on_change.emit(EmbedData::Kanban(d)))} open_dialog={props.open_dialog.clone()} />
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
                <InlineTable
                    data={d.clone()}
                    vault_path={props.vault_path.clone()}
                    on_change={Callback::from(move |d| on_change.emit(EmbedData::Table(d)))}
                    open_dialog={props.open_dialog.clone()}
                    on_page_selected={props.on_page_selected.clone()}
                />
            }
        }
    }
}
