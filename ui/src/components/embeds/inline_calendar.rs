//! Calendário inline como grade mensal de verdade (dentro do wrapper
//! `{{ type: "calendar" }}`) — eventos de 1 dia ou com intervalo
//! (`end_date`) viram barras na grade, cor por tag, clicar numa barra
//! abre o `EventDetailModal`, clicar numa área vazia do dia cria um
//! evento rápido, arrastar uma barra pra outro dia reagenda preservando
//! a duração. Antes disso era só uma lista agrupada por data — nunca foi
//! uma grade de calendário de verdade.

use std::collections::BTreeSet;

use gloo_events::EventListener;
use yew::prelude::*;

use crate::components::embeds::EventDetailModal;
use crate::date_util;
use crate::dialog::PendingDialog;
use crate::embed::{CalendarEmbedData, CalendarEntry};

/// Props do `InlineCalendar`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineCalendarProps {
    /// Eventos.
    pub data: CalendarEmbedData,
    /// Disparado quando a lista de eventos muda.
    pub on_change: Callback<CalendarEmbedData>,
    /// Abre o modal de diálogo do app (usado no fluxo rápido de criação).
    pub open_dialog: Callback<PendingDialog>,
}

const WEEKDAY_LABELS: [&str; 7] = ["D", "S", "T", "Q", "Q", "S", "S"];
const MAX_LANES: usize = 3;

/// Uma barra de evento posicionada numa semana da grade.
struct Bar {
    entry_idx: usize,
    lane: usize,
    start_col: usize,
    end_col: usize,
}

/// Aloca as barras de uma semana em lanes sem sobreposição (algoritmo
/// guloso: ordena por início, cada evento vai na primeira lane livre).
/// Eventos que não cabem nas `MAX_LANES` visíveis incrementam o contador
/// de overflow nas colunas (dias) que tocam.
fn pack_week(entries: &[CalendarEntry], week_dates: &[String; 7]) -> (Vec<Bar>, [usize; 7]) {
    let week_start = week_dates[0].as_str();
    let week_end = week_dates[6].as_str();

    let mut touching: Vec<(usize, usize, usize)> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let e_start = e.date.as_str();
        let e_end = e.end_date.as_deref().unwrap_or(e_start);
        if e_end < week_start || e_start > week_end {
            continue;
        }
        let clipped_start = if e_start > week_start { e_start } else { week_start };
        let clipped_end = if e_end < week_end { e_end } else { week_end };
        let start_col = date_util::days_between(week_start, clipped_start).unwrap_or(0).max(0) as usize;
        let end_col = date_util::days_between(week_start, clipped_end).unwrap_or(0).max(0) as usize;
        touching.push((i, start_col, end_col));
    }
    touching.sort_by_key(|&(_, start_col, _)| start_col);

    let mut lane_end: Vec<i64> = Vec::new();
    let mut bars = Vec::new();
    let mut overflow = [0usize; 7];

    for (entry_idx, start_col, end_col) in touching {
        let mut placed = false;
        for (lane, last_end) in lane_end.iter_mut().enumerate() {
            if *last_end < start_col as i64 {
                *last_end = end_col as i64;
                bars.push(Bar { entry_idx, lane, start_col, end_col });
                placed = true;
                break;
            }
        }
        if !placed {
            if lane_end.len() < MAX_LANES {
                lane_end.push(end_col as i64);
                bars.push(Bar { entry_idx, lane: lane_end.len() - 1, start_col, end_col });
            } else {
                for c in overflow.iter_mut().take(end_col + 1).skip(start_col) {
                    *c += 1;
                }
            }
        }
    }
    (bars, overflow)
}

/// Calendário inline com grade mensal.
#[function_component(InlineCalendar)]
pub fn inline_calendar(props: &InlineCalendarProps) -> Html {
    let today = date_util::today();
    let view = use_state(|| (today.0, today.1));
    let dragging = use_state(|| None::<usize>);
    let editing_entry = use_state(|| None::<usize>);

    // Zera o arraste sempre que o mouse for solto em qualquer lugar —
    // mesmo padrão do InlineKanban, evita estado de drag preso se o
    // usuário soltar fora da grade.
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

    let (vy, vm) = *view;

    let go_prev = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set(date_util::prev_month(vy, vm)))
    };
    let go_next = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set(date_util::next_month(vy, vm)))
    };
    let go_today = {
        let view = view.clone();
        Callback::from(move |_: MouseEvent| view.set((today.0, today.1)))
    };

    let add_event = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            let today_str = date_util::today_string();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título do evento".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    let mut new_data = data.clone();
                    new_data.add_entry(today_str.clone(), title);
                    on_change.emit(new_data);
                }),
            });
        })
    };

    // Grade de 6 semanas × 7 dias (mesma lógica do DatePicker).
    let first_weekday = date_util::weekday_of(vy, vm, 1);
    let days_in_month = date_util::days_in_month(vy, vm);
    let (py, pm) = date_util::prev_month(vy, vm);
    let days_in_prev = date_util::days_in_month(py, pm);
    let (ny, nm) = date_util::next_month(vy, vm);

    let mut cells: Vec<(i32, u32, u32, bool)> = Vec::with_capacity(42);
    for i in 0..first_weekday {
        cells.push((py, pm, days_in_prev - (first_weekday - 1 - i), false));
    }
    for d in 1..=days_in_month {
        cells.push((vy, vm, d, true));
    }
    let mut trailing = 1;
    while cells.len() < 42 {
        cells.push((ny, nm, trailing, false));
        trailing += 1;
    }

    let today_str = date_util::today_string();

    let existing_tags: Vec<String> = {
        let set: BTreeSet<String> = props.data.entries.iter().filter_map(|e| e.tag.clone()).collect();
        set.into_iter().collect()
    };

    let event_modal = (*editing_entry).and_then(|idx| {
        props.data.entries.get(idx).cloned().map(|entry| {
            let data = props.data.clone();
            let on_change = props.on_change.clone();
            let editing_entry_close = editing_entry.clone();
            let on_close = Callback::from(move |_: ()| editing_entry_close.set(None));

            let data_change = data.clone();
            let on_change_change = on_change.clone();
            let on_entry_change = Callback::from(move |new_entry: CalendarEntry| {
                let mut new_data = data_change.clone();
                new_data.update_entry(idx, new_entry);
                on_change_change.emit(new_data);
            });

            let editing_entry_delete = editing_entry.clone();
            let on_delete = Callback::from(move |_: ()| {
                let mut new_data = data.clone();
                new_data.remove_entry(idx);
                on_change.emit(new_data);
                editing_entry_delete.set(None);
            });

            html! {
                <EventDetailModal
                    {entry}
                    existing_tags={existing_tags.clone()}
                    on_change={on_entry_change}
                    {on_delete}
                    {on_close}
                />
            }
        })
    });

    html! {
        <div class="calendar-grid">
            <div class="calendar-grid__header">
                <button class="calendar-grid__nav-btn" onclick={go_prev}>{ "‹" }</button>
                <span class="calendar-grid__month-label">{ format!("{} {}", date_util::month_name(vm), vy) }</span>
                <button class="calendar-grid__nav-btn" onclick={go_next}>{ "›" }</button>
                <button class="calendar-grid__today-btn" onclick={go_today}>{ "Hoje" }</button>
                <span class="calendar-grid__spacer" />
                <span class="calendar-grid__count">{ props.data.entries.len() } {" eventos"}</span>
                <button class="calendar-grid__add-btn" onclick={add_event}>{ "+ evento" }</button>
            </div>

            <div class="calendar-grid__weekdays">
                { for WEEKDAY_LABELS.iter().map(|w| html! { <span>{ *w }</span> }) }
            </div>

            <div class="calendar-grid__weeks">
                { for cells.chunks(7).map(|week| {
                    let week_dates: [String; 7] = std::array::from_fn(|i| {
                        let (y, m, d, _) = week[i];
                        date_util::format_date(y, m, d)
                    });
                    let (bars, overflow) = pack_week(&props.data.entries, &week_dates);

                    let day_bgs = week.iter().enumerate().map(|(col, &(y, m, d, in_month))| {
                        let date_str = date_util::format_date(y, m, d);
                        let is_today = date_str == today_str;
                        let class = classes!(
                            "calendar-grid__cell-bg",
                            (!in_month).then_some("calendar-grid__cell-bg--muted"),
                            is_today.then_some("calendar-grid__cell-bg--today"),
                        );
                        let style = format!("grid-column: {} / {};", col + 1, col + 2);

                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        let open_dialog = props.open_dialog.clone();
                        let dragging = dragging.clone();
                        let date_for_click = date_str.clone();
                        let date_for_drop = date_str.clone();
                        let onclick = Callback::from(move |_: MouseEvent| {
                            let data = data.clone();
                            let on_change = on_change.clone();
                            let date_for_click = date_for_click.clone();
                            open_dialog.emit(PendingDialog::Prompt {
                                title: "Título do evento".to_string(),
                                default: String::new(),
                                on_submit: Callback::from(move |title: String| {
                                    let mut new_data = data.clone();
                                    new_data.add_entry(date_for_click.clone(), title);
                                    on_change.emit(new_data);
                                }),
                            });
                        });
                        let data_drop = props.data.clone();
                        let on_change_drop = props.on_change.clone();
                        let onmouseup = Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            if let Some(idx) = *dragging {
                                let mut new_data = data_drop.clone();
                                new_data.move_entry(idx, date_for_drop.clone());
                                on_change_drop.emit(new_data);
                            }
                            dragging.set(None);
                        });

                        let overflow_n = overflow[col];
                        html! {
                            <div {class} {style} {onclick} {onmouseup}>
                                <span class="calendar-grid__day-num">{ d }</span>
                                if overflow_n > 0 {
                                    <span class="calendar-grid__overflow">{ format!("+{} mais", overflow_n) }</span>
                                }
                            </div>
                        }
                    });

                    let bar_els = bars.iter().map(|bar| {
                        let entry = &props.data.entries[bar.entry_idx];
                        let style = format!(
                            "grid-column: {} / {}; grid-row: {};",
                            bar.start_col + 1, bar.end_col + 2, bar.lane + 2
                        );
                        let class = classes!(
                            "calendar-grid__bar",
                            entry.tag.as_deref().map(|t| crate::embed::badge_class(&existing_tags, t)),
                            (*dragging == Some(bar.entry_idx)).then_some("calendar-grid__bar--dragging"),
                        );
                        let entry_idx = bar.entry_idx;
                        let dragging_start = dragging.clone();
                        let onmousedown = Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            dragging_start.set(Some(entry_idx));
                        });
                        let editing_entry = editing_entry.clone();
                        let dragging_click = dragging.clone();
                        let onmouseup = Callback::from(move |e: MouseEvent| {
                            e.stop_propagation();
                            // Soltar em cima da própria barra sem ter
                            // "andado" pra outro dia conta como clique —
                            // o mouseup do dia embaixo dela não dispara
                            // porque paramos a propagação aqui.
                            if *dragging_click == Some(entry_idx) {
                                editing_entry.set(Some(entry_idx));
                            }
                            dragging_click.set(None);
                        });
                        html! {
                            <div {class} {style} {onmousedown} {onmouseup} title={entry.title.clone()}>
                                { &entry.title }
                            </div>
                        }
                    });

                    html! {
                        <div class="calendar-grid__week">
                            { for day_bgs }
                            { for bar_els }
                        </div>
                    }
                }) }
            </div>

            { for event_modal }
        </div>
    }
}
