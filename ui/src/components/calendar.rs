//! Calendar view - mostra paginas com date:: property. Lista cronológica
//! (padrão, comportamento original) + visões de grade Mês/Semana/Dia —
//! só navegação/visualização, sem edição (isso é o embed
//! `{{ type: "calendar" }}`, um componente completamente separado).

use std::collections::BTreeMap;

use wasm_bindgen::JsCast;
use yew::prelude::*;
use crate::api::{self, PageMeta};
use crate::date_util;

#[derive(Properties, PartialEq, Clone)]
pub struct CalendarProps {
    pub vault_path: String,
    pub on_page_selected: Callback<PageMeta>,
}

#[derive(Debug, Clone, PartialEq)]
struct DayItem { path: String, title: String, date: String, time: Option<String> }

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    List,
    Month,
    Week,
    Day,
}

const WEEKDAY_LABELS: [&str; 7] = ["D", "S", "T", "Q", "Q", "S", "S"];

/// Início (domingo) da semana que contém `(y, m, d)`.
fn week_start(y: i32, m: u32, d: u32) -> (i32, u32, u32) {
    let wd = date_util::weekday_of(y, m, d);
    let date_str = date_util::format_date(y, m, d);
    let start_str = date_util::add_days(&date_str, -(wd as i64)).unwrap_or(date_str);
    date_util::parse_date(&start_str).unwrap_or((y, m, d))
}

/// Desloca `(y, m, d)` em `delta` meses, ajustando o dia se o mês de
/// destino for mais curto.
fn add_months(y: i32, m: u32, d: u32, delta: i32) -> (i32, u32, u32) {
    let mut total = y * 12 + m as i32 - 1 + delta;
    let ny = total.div_euclid(12);
    total = total.rem_euclid(12);
    let nm = (total + 1) as u32;
    let max_d = date_util::days_in_month(ny, nm);
    (ny, nm, d.min(max_d))
}

#[function_component(Calendar)]
pub fn calendar(props: &CalendarProps) -> Html {
    let items = use_state(Vec::<DayItem>::new);
    let loading = use_state(|| true);
    let view_mode = use_state(|| ViewMode::List);
    let anchor = use_state(date_util::today);

    {
        let vault_path = props.vault_path.clone();
        let items = items.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let vault_path = vault_path.clone();
            let items = items.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                // Reaproveita o mesmo scanner do embed `{{ type: "calendar" }}`
                // em modo Vault (`crate::embed::scan_vault_calendar_entries`)
                // — mesma fonte de dados, uma implementação só, e ganha
                // suporte a `time::` de graça.
                let entries = crate::embed::scan_vault_calendar_entries(&vault_path).await;
                let mut list: Vec<DayItem> = entries.into_iter()
                    .filter_map(|e| {
                        let date = e.date?;
                        let path = e.page_path?;
                        Some(DayItem { path, title: e.title, date, time: e.start_time })
                    })
                    .collect();
                list.sort_by(|a, b| a.date.cmp(&b.date).then(a.time.cmp(&b.time)));
                items.set(list);
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="calendar"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let on_page_selected = props.on_page_selected.clone();
    let today_str = date_util::today_string();

    let grouped: BTreeMap<&str, Vec<&DayItem>> = {
        let mut map: BTreeMap<&str, Vec<&DayItem>> = BTreeMap::new();
        for item in items.iter() {
            map.entry(&item.date[..]).or_default().push(item);
        }
        map
    };

    let (ay, am, ad) = *anchor;
    let go_prev = {
        let anchor = anchor.clone();
        let view_mode = *view_mode;
        Callback::from(move |_: MouseEvent| {
            anchor.set(match view_mode {
                ViewMode::Month => add_months(ay, am, ad, -1),
                ViewMode::Week => {
                    let s = date_util::format_date(ay, am, ad);
                    date_util::parse_date(&date_util::add_days(&s, -7).unwrap()).unwrap_or((ay, am, ad))
                }
                ViewMode::Day => {
                    let s = date_util::format_date(ay, am, ad);
                    date_util::parse_date(&date_util::add_days(&s, -1).unwrap()).unwrap_or((ay, am, ad))
                }
                ViewMode::List => (ay, am, ad),
            });
        })
    };
    let go_next = {
        let anchor = anchor.clone();
        let view_mode = *view_mode;
        Callback::from(move |_: MouseEvent| {
            anchor.set(match view_mode {
                ViewMode::Month => add_months(ay, am, ad, 1),
                ViewMode::Week => {
                    let s = date_util::format_date(ay, am, ad);
                    date_util::parse_date(&date_util::add_days(&s, 7).unwrap()).unwrap_or((ay, am, ad))
                }
                ViewMode::Day => {
                    let s = date_util::format_date(ay, am, ad);
                    date_util::parse_date(&date_util::add_days(&s, 1).unwrap()).unwrap_or((ay, am, ad))
                }
                ViewMode::List => (ay, am, ad),
            });
        })
    };
    let go_today = {
        let anchor = anchor.clone();
        Callback::from(move |_: MouseEvent| anchor.set(date_util::today()))
    };
    let on_view_change = {
        let view_mode = view_mode.clone();
        Callback::from(move |e: Event| {
            let Some(target) = e.target() else { return };
            let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() else { return };
            view_mode.set(match select.value().as_str() {
                "month" => ViewMode::Month,
                "week" => ViewMode::Week,
                "day" => ViewMode::Day,
                _ => ViewMode::List,
            });
        })
    };

    let render_day_cell = |date_str: String, in_range: bool| -> Html {
        let is_today = date_str == today_str;
        let day_items = grouped.get(date_str.as_str()).cloned().unwrap_or_default();
        let (_, _, d) = date_util::parse_date(&date_str).unwrap_or((0, 0, 0));
        let class = classes!(
            "page-calendar__cell",
            (!in_range).then_some("page-calendar__cell--muted"),
            is_today.then_some("page-calendar__cell--today"),
        );
        html! {
            <div {class}>
                <span class="page-calendar__cell-daynum">{ d }</span>
                <div class="page-calendar__cell-items">
                    { for day_items.iter().map(|item| {
                        let meta = PageMeta { path: item.path.clone(), title: item.title.clone(), section: "pages".to_string() };
                        let label = match &item.time {
                            Some(t) => format!("{} {}", t, item.title),
                            None => item.title.clone(),
                        };
                        let activate = {
                            let on_page_selected = on_page_selected.clone();
                            let meta = meta.clone();
                            Callback::from(move |_: ()| on_page_selected.emit(meta.clone()))
                        };
                        let onclick = {
                            let activate = activate.clone();
                            Callback::from(move |_: MouseEvent| activate.emit(()))
                        };
                        let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                        html! {
                            <div class="page-calendar__cell-item" title={label.clone()}
                                tabindex="0" {onclick} {onkeydown}>
                                { label }
                            </div>
                        }
                    }) }
                </div>
            </div>
        }
    };

    let body = match *view_mode {
        ViewMode::List => html! {
            <div class="calendar__list">
                if grouped.is_empty() {
                    <p class="calendar__empty">{"Nenhum evento com date:: encontrado. Adicione 'date:: 2026-08-06' em qualquer página."}</p>
                } else {
                    { for grouped.iter().map(|(date, day_items)| {
                        html! {
                            <div class="calendar__day">
                                <div class="calendar__day-header">
                                    <span class="calendar__day-date">{ date }</span>
                                    <span class="calendar__day-count">{ day_items.len() }</span>
                                </div>
                                { for day_items.iter().map(|item| {
                                    let path = item.path.clone();
                                    let title = item.title.clone();
                                    let meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                                    let time_label = item.time.clone();
                                    let activate = {
                                        let on_page_selected = on_page_selected.clone();
                                        let meta = meta.clone();
                                        Callback::from(move |_: ()| on_page_selected.emit(meta.clone()))
                                    };
                                    let onclick = {
                                        let activate = activate.clone();
                                        Callback::from(move |_: MouseEvent| activate.emit(()))
                                    };
                                    let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                                    html! {
                                        <div class="calendar__item" tabindex="0" {onclick} {onkeydown}>
                                            if let Some(t) = time_label {
                                                <span class="calendar__item-time">{ t }</span>
                                            }
                                            <span class="calendar__item-title">{ &item.title }</span>
                                        </div>
                                    }
                                }) }
                            </div>
                        }
                    }) }
                }
            </div>
        },
        ViewMode::Month => {
            let first_weekday = date_util::weekday_of(ay, am, 1);
            let days_in_month = date_util::days_in_month(ay, am);
            let (py, pm) = date_util::prev_month(ay, am);
            let days_in_prev = date_util::days_in_month(py, pm);
            let (ny, nm) = date_util::next_month(ay, am);

            let mut cells: Vec<(i32, u32, u32, bool)> = Vec::with_capacity(42);
            for i in 0..first_weekday {
                cells.push((py, pm, days_in_prev - (first_weekday - 1 - i), false));
            }
            for d in 1..=days_in_month {
                cells.push((ay, am, d, true));
            }
            let mut trailing = 1;
            while cells.len() < 42 {
                cells.push((ny, nm, trailing, false));
                trailing += 1;
            }

            html! {
                <div class="page-calendar">
                    <div class="page-calendar__weekdays">
                        { for WEEKDAY_LABELS.iter().map(|w| html! { <span>{ *w }</span> }) }
                    </div>
                    <div class="page-calendar__grid page-calendar__grid--month">
                        { for cells.iter().map(|&(y, m, d, in_month)| render_day_cell(date_util::format_date(y, m, d), in_month)) }
                    </div>
                </div>
            }
        }
        ViewMode::Week => {
            let (wy, wm, wd) = week_start(ay, am, ad);
            let week_dates: Vec<String> = (0..7).map(|i| date_util::add_days(&date_util::format_date(wy, wm, wd), i).unwrap()).collect();
            html! {
                <div class="page-calendar">
                    <div class="page-calendar__weekdays">
                        { for WEEKDAY_LABELS.iter().map(|w| html! { <span>{ *w }</span> }) }
                    </div>
                    <div class="page-calendar__grid page-calendar__grid--week">
                        { for week_dates.iter().map(|d| render_day_cell(d.clone(), true)) }
                    </div>
                </div>
            }
        }
        ViewMode::Day => {
            let date_str = date_util::format_date(ay, am, ad);
            html! {
                <div class="page-calendar">
                    <div class="page-calendar__grid page-calendar__grid--day">
                        { render_day_cell(date_str, true) }
                    </div>
                </div>
            }
        }
    };

    let header_label = match *view_mode {
        ViewMode::List => "Todos os eventos".to_string(),
        ViewMode::Month | ViewMode::Week => format!("{} {}", date_util::month_name(am), ay),
        ViewMode::Day => format!("{} de {} de {}", ad, date_util::month_name(am).to_lowercase(), ay),
    };

    html! {
        <div class="calendar">
            <div class="calendar__header">
                <h2>{"Calendário"}</h2>
                if *view_mode != ViewMode::List {
                    <button class="calendar-grid__nav-btn" onclick={go_prev}>{ "‹" }</button>
                    <span class="calendar-grid__month-label">{ header_label }</span>
                    <button class="calendar-grid__nav-btn" onclick={go_next}>{ "›" }</button>
                    <button class="calendar-grid__today-btn" onclick={go_today}>{ "Hoje" }</button>
                }
                <select class="calendar-grid__view-select" onchange={on_view_change}>
                    <option value="list" selected={*view_mode == ViewMode::List}>{ "Lista" }</option>
                    <option value="month" selected={*view_mode == ViewMode::Month}>{ "Mês" }</option>
                    <option value="week" selected={*view_mode == ViewMode::Week}>{ "Semana" }</option>
                    <option value="day" selected={*view_mode == ViewMode::Day}>{ "Dia" }</option>
                </select>
                <span class="calendar__header-spacer" />
                <span class="calendar__count">{ items.len() } {" eventos"}</span>
            </div>
            { body }
        </div>
    }
}
