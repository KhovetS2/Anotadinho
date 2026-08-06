//! Lista de eventos inline (dentro de uma fence ```calendar), agrupada por
//! data — dinâmica: criar/editar/excluir evento. Diálogos usam o modal do
//! app (`crate::dialog`), não `window.prompt`.

use std::collections::BTreeMap;

use yew::prelude::*;

use crate::dialog::PendingDialog;
use crate::embed::CalendarEmbedData;

/// Props do `InlineCalendar`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineCalendarProps {
    /// Eventos.
    pub data: CalendarEmbedData,
    /// Disparado quando a lista de eventos muda.
    pub on_change: Callback<CalendarEmbedData>,
    /// Abre o modal de diálogo do app.
    pub open_dialog: Callback<PendingDialog>,
}

/// Pede data e depois título via dois `Prompt` encadeados (mais simples
/// que generalizar o dialog pra formulário multi-campo nesta rodada).
fn ask_date_then_title(
    open_dialog: &Callback<PendingDialog>,
    date_default: String,
    title_default: String,
    on_done: Callback<(String, String)>,
) {
    let open_dialog_inner = open_dialog.clone();
    open_dialog.emit(PendingDialog::Prompt {
        title: "Data do evento (AAAA-MM-DD)".to_string(),
        default: date_default,
        on_submit: Callback::from(move |date: String| {
            let on_done = on_done.clone();
            let date_for_submit = date.clone();
            open_dialog_inner.emit(PendingDialog::Prompt {
                title: "Título do evento".to_string(),
                default: title_default.clone(),
                on_submit: Callback::from(move |title: String| {
                    on_done.emit((date_for_submit.clone(), title));
                }),
            });
        }),
    });
}

/// Lista de eventos inline agrupada por data.
#[function_component(InlineCalendar)]
pub fn inline_calendar(props: &InlineCalendarProps) -> Html {
    let grouped: BTreeMap<&str, Vec<usize>> = {
        let mut map: BTreeMap<&str, Vec<usize>> = BTreeMap::new();
        for (i, entry) in props.data.entries.iter().enumerate() {
            map.entry(entry.date.as_str()).or_default().push(i);
        }
        map
    };

    let add_entry = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            ask_date_then_title(
                &open_dialog,
                String::new(),
                String::new(),
                Callback::from(move |(date, title): (String, String)| {
                    let mut new_data = data.clone();
                    new_data.add_entry(date, title);
                    on_change.emit(new_data);
                }),
            );
        })
    };

    html! {
        <div class="calendar embed-calendar">
            <div class="calendar__header">
                <span class="calendar__count">{ props.data.entries.len() } {" eventos"}</span>
                <button class="calendar__add" onclick={add_entry}>{ "+ evento" }</button>
            </div>
            <div class="calendar__list">
                { for grouped.into_iter().map(|(date, idxs)| {
                    html! {
                        <div class="calendar__day">
                            <div class="calendar__day-header">
                                <span class="calendar__day-date">{ date }</span>
                                <span class="calendar__day-count">{ idxs.len() }</span>
                            </div>
                            { for idxs.into_iter().map(|i| {
                                let entry = &props.data.entries[i];
                                let data = props.data.clone();
                                let on_change = props.on_change.clone();
                                let open_dialog = props.open_dialog.clone();
                                let cur_date = entry.date.clone();
                                let cur_title = entry.title.clone();
                                let onclick = Callback::from(move |_: MouseEvent| {
                                    let data = data.clone();
                                    let on_change = on_change.clone();
                                    ask_date_then_title(
                                        &open_dialog,
                                        cur_date.clone(),
                                        cur_title.clone(),
                                        Callback::from(move |(new_date, new_title): (String, String)| {
                                            let mut new_data = data.clone();
                                            new_data.edit_entry(i, new_date, new_title);
                                            on_change.emit(new_data);
                                        }),
                                    );
                                });

                                let data = props.data.clone();
                                let on_change = props.on_change.clone();
                                let open_dialog = props.open_dialog.clone();
                                let title_for_confirm = entry.title.clone();
                                let ondelete = Callback::from(move |e: MouseEvent| {
                                    e.stop_propagation();
                                    let data = data.clone();
                                    let on_change = on_change.clone();
                                    open_dialog.emit(PendingDialog::Confirm {
                                        message: format!("Excluir evento \"{}\"?", title_for_confirm),
                                        confirm_label: "Excluir".to_string(),
                                        on_confirm: Callback::from(move |_| {
                                            let mut new_data = data.clone();
                                            new_data.remove_entry(i);
                                            on_change.emit(new_data);
                                        }),
                                    });
                                });

                                html! {
                                    <div class="calendar__item" onclick={onclick}>
                                        <span class="calendar__item-title">{ &entry.title }</span>
                                        <button class="calendar__item-delete" onclick={ondelete} title="Excluir">{ "✕" }</button>
                                    </div>
                                }
                            }) }
                        </div>
                    }
                }) }
            </div>
        </div>
    }
}
