//! Lista de eventos inline (dentro de uma fence ```calendar), agrupada por
//! data. Click num evento permite editar o título via prompt, atualizando
//! o `EmbedData` via `on_change`.

use std::collections::BTreeMap;

use yew::prelude::*;

use crate::embed::CalendarEmbedData;

/// Props do `InlineCalendar`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineCalendarProps {
    /// Eventos.
    pub data: CalendarEmbedData,
    /// Disparado quando um evento é editado.
    pub on_change: Callback<CalendarEmbedData>,
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

    html! {
        <div class="calendar embed-calendar">
            <div class="calendar__header">
                <span class="calendar__count">{ props.data.entries.len() } {" eventos"}</span>
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
                                let onclick = Callback::from(move |_| {
                                    let current = data.entries[i].title.clone();
                                    if let Some(new_title) = gloo_dialogs::prompt("Editar evento:", Some(&current)) {
                                        let new_title = new_title.trim();
                                        if !new_title.is_empty() && new_title != current {
                                            let mut new_data = data.clone();
                                            new_data.entries[i].title = new_title.to_string();
                                            on_change.emit(new_data);
                                        }
                                    }
                                });
                                html! {
                                    <div class="calendar__item" onclick={onclick}>
                                        <span class="calendar__item-title">{ &entry.title }</span>
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
