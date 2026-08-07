//! Calendário inline como grade de verdade (dentro do wrapper
//! `{{ type: "calendar" }}`) — visões Mês (padrão), Semana e Dia. Eventos
//! de 1 dia ou com intervalo (`end_date`) viram barras contínuas, eventos
//! com horário (`start_time`/`end_time`) viram blocos posicionados na
//! grade de horas das visões Semana/Dia, cor por tag, clicar numa
//! barra/bloco abre o `EventDetailModal`, clicar numa área vazia cria um
//! evento rápido, arrastar reagenda preservando a duração (entre dias —
//! mudar o horário arrastando verticalmente fica pra um ciclo futuro).

use std::collections::BTreeSet;

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::embeds::EventDetailModal;
use crate::date_util;
use crate::dialog::PendingDialog;
use crate::embed::{badge_class, CalendarEmbedData, CalendarEntry};

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
const WEEKDAY_ABBR: [&str; 7] = ["DOM", "SEG", "TER", "QUA", "QUI", "SEX", "SÁB"];
const MAX_LANES: usize = 3;
const HOUR_PX: f64 = 48.0;
const SNAP_MINUTES: f64 = 15.0;

/// Converte um deslocamento vertical (px, a partir do topo da coluna do
/// dia = 0h) em minutos desde meia-noite, arredondado pro múltiplo de
/// `SNAP_MINUTES` mais próximo (mesmo "encaixe" de 15 em 15 minutos do
/// Google Calendar).
fn y_to_snapped_minutes(y_px: f64) -> u32 {
    let raw_minutes = (y_px / HOUR_PX) * 60.0;
    let snapped = (raw_minutes / SNAP_MINUTES).round() * SNAP_MINUTES;
    snapped.clamp(0.0, 23.0 * 60.0 + 45.0) as u32
}

#[derive(Clone, Copy, PartialEq)]
enum ViewMode {
    Month,
    Week,
    Day,
}

/// Uma barra de evento posicionada num intervalo de dias visíveis (semana
/// inteira na visão Mês, ou a janela de 1/7 dias das visões Dia/Semana).
struct Bar {
    entry_idx: usize,
    lane: usize,
    start_col: usize,
    end_col: usize,
}

/// Aloca as barras de `entries` que tocam `day_dates` em lanes sem
/// sobreposição (algoritmo guloso: ordena por início, cada evento vai na
/// primeira lane livre). Eventos que não cabem nas `MAX_LANES` visíveis
/// incrementam o contador de overflow nas colunas (dias) que tocam.
/// Genérico sobre o tamanho da janela — usado tanto pela semana inteira
/// (7 dias, visão Mês) quanto pela janela de Dia/Semana.
///
/// `exclude_timed`: quando `true`, eventos com `start_time` ficam de fora
/// (usado pra faixa de dia inteiro das visões Semana/Dia, onde um evento
/// com horário já ganha um bloco posicionado na grade de horas — mostrar
/// ele nos dois lugares seria duplicado). Na visão Mês (sem grade de
/// horas pra mostrar o horário de outro jeito) passa `false`, todo evento
/// vira barra independente de ter horário ou não.
fn pack_days(entries: &[CalendarEntry], day_dates: &[String], exclude_timed: bool) -> (Vec<Bar>, Vec<usize>) {
    let n = day_dates.len();
    let mut overflow = vec![0usize; n];
    if n == 0 {
        return (Vec::new(), overflow);
    }
    let window_start = day_dates[0].as_str();
    let window_end = day_dates[n - 1].as_str();

    let mut touching: Vec<(usize, usize, usize)> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        if exclude_timed && e.start_time.is_some() {
            continue;
        }
        // Evento sem data (na gaveta) não aparece na grade.
        let Some(e_start) = e.date.as_deref() else { continue };
        let e_end = e.end_date.as_deref().unwrap_or(e_start);
        if e_end < window_start || e_start > window_end {
            continue;
        }
        let clipped_start = if e_start > window_start { e_start } else { window_start };
        let clipped_end = if e_end < window_end { e_end } else { window_end };
        let start_col = date_util::days_between(window_start, clipped_start).unwrap_or(0).max(0) as usize;
        let end_col = date_util::days_between(window_start, clipped_end).unwrap_or(0).max(0) as usize;
        touching.push((i, start_col, end_col));
    }
    touching.sort_by_key(|&(_, start_col, _)| start_col);

    let mut lane_end: Vec<i64> = Vec::new();
    let mut bars = Vec::new();

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

/// Início (domingo) da semana que contém `(y, m, d)`.
fn week_start(y: i32, m: u32, d: u32) -> (i32, u32, u32) {
    let wd = date_util::weekday_of(y, m, d);
    let date_str = date_util::format_date(y, m, d);
    let start_str = date_util::add_days(&date_str, -(wd as i64)).unwrap_or(date_str);
    date_util::parse_date(&start_str).unwrap_or((y, m, d))
}

/// Desloca `(y, m, d)` em `delta` meses, ajustando o dia se o mês de
/// destino for mais curto (ex: 31 de janeiro + 1 mês vira 28/29 de
/// fevereiro, não um overflow pra março).
fn add_months(y: i32, m: u32, d: u32, delta: i32) -> (i32, u32, u32) {
    let mut total = y * 12 + m as i32 - 1 + delta;
    let ny = total.div_euclid(12);
    total = total.rem_euclid(12);
    let nm = (total + 1) as u32;
    let max_d = date_util::days_in_month(ny, nm);
    (ny, nm, d.min(max_d))
}

/// Calendário inline com grades Mês/Semana/Dia.
#[function_component(InlineCalendar)]
pub fn inline_calendar(props: &InlineCalendarProps) -> Html {
    let today = date_util::today();
    let anchor = use_state(|| today);
    let view_mode = use_state(|| ViewMode::Month);
    let dragging = use_state(|| None::<usize>);
    let editing_entry = use_state(|| None::<usize>);
    let drag_pos = use_state(|| None::<(i32, i32)>);
    let hover_day = use_state(|| None::<String>);
    let hour_scroll_ref = use_node_ref();
    // `(entry_idx, is_start_edge)` — qual bloco/borda está sendo
    // redimensionado arrastando a borda de cima (`true`) ou baixo
    // (`false`) na grade de horas.
    let resizing = use_state(|| None::<(usize, bool)>);
    let resize_preview_min = use_state(|| None::<u32>);
    // Gaveta de eventos sem data — recolhida por padrão.
    let drawer_open = use_state(|| false);

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

    // Ghost que segue o cursor durante o arraste — mesmo padrão do
    // InlineKanban.
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

    // Redimensionar duração arrastando a borda de cima/baixo de um bloco
    // com horário: `mousemove` global atualiza a prévia (minutos
    // encaixados no quarto de hora mais próximo, mesma lógica de
    // `y_to_snapped_minutes` usada no drag vertical); `mouseup` confirma a
    // mudança em `start_time`/`end_time` e encerra o resize. Estado
    // separado de `dragging` — não usa o mecanismo de mover/reordenar.
    {
        let resize_preview_min = resize_preview_min.clone();
        let hour_scroll_ref = hour_scroll_ref.clone();
        let resizing_state = resizing.clone();
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        use_effect_with(*resizing, move |resizing_val| {
            let listeners = resizing_val.map(|(idx, is_start_edge)| {
                let window = web_sys::window().expect("no global window");

                let move_hour_scroll_ref = hour_scroll_ref.clone();
                let move_preview = resize_preview_min.clone();
                let mousemove = EventListener::new(&window, "mousemove", move |e| {
                    if let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() {
                        if let Some(el) = move_hour_scroll_ref.cast::<web_sys::Element>() {
                            let rect = el.get_bounding_client_rect();
                            let col_top = rect.top() - el.scroll_top() as f64;
                            let y = e.client_y() as f64 - col_top;
                            move_preview.set(Some(y_to_snapped_minutes(y)));
                        }
                    }
                });

                // Calcula o minuto final direto da posição do mouse (via
                // NodeRef, que sempre reflete o DOM atual) em vez de ler o
                // `resize_preview_min` guardado no `use_state` — esse
                // handler só é criado UMA VEZ (no início do resize), então
                // o clone do handle que ele capturou fica congelado no
                // valor daquele instante; `.set()` chamado depois por outra
                // instância do handle (no listener de `mousemove`, criado
                // no mesmo momento) não atualiza esse clone congelado.
                let up_hour_scroll_ref = hour_scroll_ref.clone();
                let up_data = data.clone();
                let up_on_change = on_change.clone();
                let up_resizing = resizing_state.clone();
                let up_preview = resize_preview_min.clone();
                let mouseup = EventListener::new(&window, "mouseup", move |e| {
                    if let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() {
                        if let Some(el) = up_hour_scroll_ref.cast::<web_sys::Element>() {
                            let rect = el.get_bounding_client_rect();
                            let col_top = rect.top() - el.scroll_top() as f64;
                            let y = e.client_y() as f64 - col_top;
                            let new_min = y_to_snapped_minutes(y);
                            let mut new_data = up_data.clone();
                            new_data.resize_entry_time(idx, is_start_edge, new_min);
                            up_on_change.emit(new_data);
                        }
                    }
                    up_resizing.set(None);
                    up_preview.set(None);
                });

                (mousemove, mouseup)
            });
            if resizing_val.is_none() {
                resize_preview_min.set(None);
            }
            move || drop(listeners)
        });
    }

    // Ao entrar em Semana/Dia, rola a grade de horas pra deixar o horário
    // atual visível (~2h antes dele), em vez de abrir sempre em 0h.
    {
        let hour_scroll_ref = hour_scroll_ref.clone();
        use_effect_with(*view_mode, move |mode| {
            if *mode != ViewMode::Month {
                if let Some(el) = hour_scroll_ref.cast::<web_sys::Element>() {
                    let now_min = date_util::now_minutes();
                    let target = ((now_min as f64 - 120.0).max(0.0) / 60.0) * HOUR_PX;
                    el.set_scroll_top(target as i32);
                }
            }
            || {}
        });
    }

    let (ay, am, ad) = *anchor;
    let anchor_str = date_util::format_date(ay, am, ad);
    let today_str = date_util::today_string();

    let go_prev = {
        let anchor = anchor.clone();
        let view_mode = *view_mode;
        let anchor_str = anchor_str.clone();
        Callback::from(move |_: MouseEvent| {
            anchor.set(match view_mode {
                ViewMode::Month => add_months(ay, am, ad, -1),
                ViewMode::Week => date_util::parse_date(&date_util::add_days(&anchor_str, -7).unwrap()).unwrap_or((ay, am, ad)),
                ViewMode::Day => date_util::parse_date(&date_util::add_days(&anchor_str, -1).unwrap()).unwrap_or((ay, am, ad)),
            });
        })
    };
    let go_next = {
        let anchor = anchor.clone();
        let view_mode = *view_mode;
        let anchor_str = anchor_str.clone();
        Callback::from(move |_: MouseEvent| {
            anchor.set(match view_mode {
                ViewMode::Month => add_months(ay, am, ad, 1),
                ViewMode::Week => date_util::parse_date(&date_util::add_days(&anchor_str, 7).unwrap()).unwrap_or((ay, am, ad)),
                ViewMode::Day => date_util::parse_date(&date_util::add_days(&anchor_str, 1).unwrap()).unwrap_or((ay, am, ad)),
            });
        })
    };
    let go_today = {
        let anchor = anchor.clone();
        Callback::from(move |_: MouseEvent| anchor.set(today))
    };
    let on_view_change = {
        let view_mode = view_mode.clone();
        Callback::from(move |e: Event| {
            let Some(target) = e.target() else { return };
            let Ok(select) = target.dyn_into::<web_sys::HtmlSelectElement>() else { return };
            view_mode.set(match select.value().as_str() {
                "week" => ViewMode::Week,
                "day" => ViewMode::Day,
                _ => ViewMode::Month,
            });
        })
    };

    let add_event = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        let anchor_str = anchor_str.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            let anchor_str = anchor_str.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título do evento".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    let mut new_data = data.clone();
                    new_data.add_entry(anchor_str.clone(), title);
                    on_change.emit(new_data);
                }),
            });
        })
    };

    let add_unscheduled_event = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título do evento (sem data)".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    let mut new_data = data.clone();
                    new_data.add_unscheduled_entry(title);
                    on_change.emit(new_data);
                }),
            });
        })
    };
    let toggle_drawer = {
        let drawer_open = drawer_open.clone();
        Callback::from(move |_: MouseEvent| drawer_open.set(!*drawer_open))
    };

    let existing_tags: Vec<String> = {
        let set: BTreeSet<String> = props.data.entries.iter().flat_map(|e| e.all_tags()).collect();
        set.into_iter().collect()
    };

    // Eventos sem data (`date: None`) — ficam fora da grade, na gaveta.
    // Arrastar um item daqui reusa o MESMO mecanismo de `dragging` já
    // usado pelas barras/blocos da grade: soltar num dia/coluna chama
    // `move_entry`/`move_entry_time` normalmente, que atribuem a data.
    let unscheduled_idxs: Vec<usize> = props.data.entries.iter().enumerate()
        .filter_map(|(i, e)| e.date.is_none().then_some(i))
        .collect();
    let drawer_items = unscheduled_idxs.iter().map(|&idx| {
        let entry = &props.data.entries[idx];
        let class = classes!(
            "calendar-grid__drawer-item",
            entry.all_tags().first().map(|t| badge_class(&existing_tags, t)),
            (*dragging == Some(idx)).then_some("calendar-grid__bar--dragging"),
        );
        let dragging_start = dragging.clone();
        let onmousedown = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            dragging_start.set(Some(idx));
        });
        let editing_entry_click = editing_entry.clone();
        let dragging_click = dragging.clone();
        let onmouseup = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if *dragging_click == Some(idx) {
                editing_entry_click.set(Some(idx));
            }
            dragging_click.set(None);
        });
        html! {
            <div {class} {onmousedown} {onmouseup} title={entry.title.clone()}>
                { &entry.title }
            </div>
        }
    }).collect::<Vec<_>>();

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

    // Rótulo do cabeçalho: depende da visão ativa.
    let header_label = match *view_mode {
        ViewMode::Month => format!("{} {}", date_util::month_name(am), ay),
        ViewMode::Week => format!("{} {}", date_util::month_name(am), ay),
        ViewMode::Day => format!("{} de {} de {}", ad, date_util::month_name(am).to_lowercase(), ay),
    };

    // Dias visíveis na janela atual (só usado por Semana/Dia).
    let window_dates: Vec<String> = match *view_mode {
        ViewMode::Week => {
            let (wy, wm, wd) = week_start(ay, am, ad);
            (0..7).map(|i| date_util::add_days(&date_util::format_date(wy, wm, wd), i).unwrap()).collect()
        }
        ViewMode::Day => vec![anchor_str.clone()],
        ViewMode::Month => Vec::new(),
    };

    let body = match *view_mode {
        ViewMode::Month => render_month_grid(
            props, ay, am, &today_str, &dragging, &hover_day, &editing_entry, &existing_tags, &open_dialog_clone(props),
        ),
        ViewMode::Week | ViewMode::Day => render_day_columns(
            props, &window_dates, &today_str, &dragging, &hover_day, &editing_entry, &existing_tags, &hour_scroll_ref,
            &resizing, &resize_preview_min,
        ),
    };

    html! {
        <div class="calendar-grid">
            <div class="calendar-grid__header">
                <button class="calendar-grid__nav-btn" onclick={go_prev}>{ "‹" }</button>
                <span class="calendar-grid__month-label">{ header_label }</span>
                <button class="calendar-grid__nav-btn" onclick={go_next}>{ "›" }</button>
                <button class="calendar-grid__today-btn" onclick={go_today}>{ "Hoje" }</button>
                <select class="calendar-grid__view-select" onchange={on_view_change}>
                    <option value="month" selected={*view_mode == ViewMode::Month}>{ "Mês" }</option>
                    <option value="week" selected={*view_mode == ViewMode::Week}>{ "Semana" }</option>
                    <option value="day" selected={*view_mode == ViewMode::Day}>{ "Dia" }</option>
                </select>
                <span class="calendar-grid__spacer" />
                <span class="calendar-grid__count">{ props.data.entries.len() } {" eventos"}</span>
                <button class="calendar-grid__add-btn" onclick={add_event}>{ "+ evento" }</button>
            </div>

            { body }

            <div class="calendar-grid__drawer">
                <button class="calendar-grid__drawer-toggle" onclick={toggle_drawer}>
                    { if *drawer_open { "▾" } else { "▸" } }
                    { format!(" Sem data ({})", unscheduled_idxs.len()) }
                </button>
                <button class="calendar-grid__add-btn calendar-grid__add-btn--ghost" onclick={add_unscheduled_event}>{ "+ evento sem data" }</button>
                if *drawer_open {
                    <div class="calendar-grid__drawer-list">
                        if unscheduled_idxs.is_empty() {
                            <span class="calendar-grid__drawer-empty">{ "Nenhum evento sem data." }</span>
                        } else {
                            { for drawer_items }
                        }
                    </div>
                }
            </div>

            if let (Some(idx), Some((x, y))) = (*dragging, *drag_pos) {
                if let Some(entry) = props.data.entries.get(idx) {
                    <div class="calendar-grid__drag-ghost" style={format!("left: {}px; top: {}px;", x + 12, y + 12)}>
                        { &entry.title }
                    </div>
                }
            }
            { for event_modal }
        </div>
    }
}

fn open_dialog_clone(props: &InlineCalendarProps) -> Callback<PendingDialog> {
    props.open_dialog.clone()
}

/// Grade mensal (visão Mês): 6 semanas × 7 dias, eventos como barras.
#[allow(clippy::too_many_arguments)]
fn render_month_grid(
    props: &InlineCalendarProps,
    vy: i32,
    vm: u32,
    today_str: &str,
    dragging: &UseStateHandle<Option<usize>>,
    hover_day: &UseStateHandle<Option<String>>,
    editing_entry: &UseStateHandle<Option<usize>>,
    existing_tags: &[String],
    open_dialog: &Callback<PendingDialog>,
) -> Html {
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

    html! {
        <>
        <div class="calendar-grid__weekdays">
            { for WEEKDAY_LABELS.iter().map(|w| html! { <span>{ *w }</span> }) }
        </div>
        <div class="calendar-grid__weeks">
            { for cells.chunks(7).map(|week| {
                let week_dates: Vec<String> = week.iter().map(|&(y, m, d, _)| date_util::format_date(y, m, d)).collect();
                let (bars, overflow) = pack_days(&props.data.entries, &week_dates, false);

                let day_bgs = week.iter().enumerate().map(|(col, &(y, m, d, in_month))| {
                    let date_str = date_util::format_date(y, m, d);
                    let is_today = date_str == today_str;
                    let is_drop_target = dragging.is_some() && **hover_day == Some(date_str.clone());
                    let class = classes!(
                        "calendar-grid__cell-bg",
                        (!in_month).then_some("calendar-grid__cell-bg--muted"),
                        is_today.then_some("calendar-grid__cell-bg--today"),
                        is_drop_target.then_some("calendar-grid__cell-bg--drop-target"),
                    );
                    let style = format!("grid-column: {} / {};", col + 1, col + 2);

                    let data = props.data.clone();
                    let on_change = props.on_change.clone();
                    let open_dialog = open_dialog.clone();
                    let dragging = dragging.clone();
                    let date_for_click = date_str.clone();
                    let date_for_drop = date_str.clone();
                    let onmouseenter = {
                        let dragging = dragging.clone();
                        let hover_day = hover_day.clone();
                        let date_for_hover = date_str.clone();
                        Callback::from(move |_: MouseEvent| {
                            if dragging.is_some() {
                                hover_day.set(Some(date_for_hover.clone()));
                            }
                        })
                    };
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
                        <div {class} {style} {onclick} {onmouseup} {onmouseenter}>
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
                        entry.all_tags().first().map(|t| badge_class(existing_tags, t)),
                        (**dragging == Some(bar.entry_idx)).then_some("calendar-grid__bar--dragging"),
                    );
                    let entry_idx = bar.entry_idx;
                    let dragging_start = dragging.clone();
                    let onmousedown = Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        e.prevent_default();
                        dragging_start.set(Some(entry_idx));
                    });
                    let editing_entry = editing_entry.clone();
                    let dragging_click = dragging.clone();
                    let onmouseup = Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
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
        </>
    }
}

/// Grade de colunas de dia (visões Semana/Dia): rótulos + faixa de dia
/// inteiro/intervalo + grade de horas com blocos de evento posicionados
/// por horário. `day_dates` tem 7 elementos (Semana) ou 1 (Dia).
#[allow(clippy::too_many_arguments)]
fn render_day_columns(
    props: &InlineCalendarProps,
    day_dates: &[String],
    today_str: &str,
    dragging: &UseStateHandle<Option<usize>>,
    hover_day: &UseStateHandle<Option<String>>,
    editing_entry: &UseStateHandle<Option<usize>>,
    existing_tags: &[String],
    hour_scroll_ref: &NodeRef,
    resizing: &UseStateHandle<Option<(usize, bool)>>,
    resize_preview_min: &UseStateHandle<Option<u32>>,
) -> Html {
    let (bars, _overflow) = pack_days(&props.data.entries, day_dates, true);
    let n = day_dates.len();

    let day_labels = day_dates.iter().map(|date_str| {
        let (_, _, d) = date_util::parse_date(date_str).unwrap_or((0, 0, 0));
        let wd = date_util::parse_date(date_str).map(|(y, m, d)| date_util::weekday_of(y, m, d)).unwrap_or(0);
        let is_today = date_str == today_str;
        let class = classes!("calendar-grid__daygrid-daylabel", is_today.then_some("calendar-grid__daygrid-daylabel--today"));
        html! {
            <div {class}>
                <span class="calendar-grid__daygrid-weekday">{ WEEKDAY_ABBR[wd as usize] }</span>
                <span class="calendar-grid__daygrid-daynum">{ d }</span>
            </div>
        }
    });

    let allday_bgs = day_dates.iter().enumerate().map(|(col, date_str)| {
        let is_drop_target = dragging.is_some() && **hover_day == Some(date_str.clone());
        let class = classes!("calendar-grid__cell-bg", is_drop_target.then_some("calendar-grid__cell-bg--drop-target"));
        let style = format!("grid-column: {} / {};", col + 1, col + 2);

        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        let dragging_c = dragging.clone();
        let date_for_click = date_str.clone();
        let date_for_drop = date_str.clone();
        let onmouseenter = {
            let dragging = dragging.clone();
            let hover_day = hover_day.clone();
            let date_for_hover = date_str.clone();
            Callback::from(move |_: MouseEvent| {
                if dragging.is_some() {
                    hover_day.set(Some(date_for_hover.clone()));
                }
            })
        };
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
            if let Some(idx) = *dragging_c {
                let mut new_data = data_drop.clone();
                new_data.move_entry(idx, date_for_drop.clone());
                on_change_drop.emit(new_data);
            }
            dragging_c.set(None);
        });
        html! { <div {class} {style} {onclick} {onmouseup} {onmouseenter} /> }
    });

    let allday_bars = bars.iter().map(|bar| {
        let entry = &props.data.entries[bar.entry_idx];
        let style = format!("grid-column: {} / {}; grid-row: {};", bar.start_col + 1, bar.end_col + 2, bar.lane + 1);
        let class = classes!(
            "calendar-grid__bar",
            entry.all_tags().first().map(|t| badge_class(existing_tags, t)),
            (**dragging == Some(bar.entry_idx)).then_some("calendar-grid__bar--dragging"),
        );
        let entry_idx = bar.entry_idx;
        let dragging_start = dragging.clone();
        let onmousedown = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            e.prevent_default();
            dragging_start.set(Some(entry_idx));
        });
        let editing_entry = editing_entry.clone();
        let dragging_click = dragging.clone();
        let onmouseup = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
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

    let hour_labels = (0..24).map(|h| {
        let label = if h == 0 { "12 AM".to_string() } else if h < 12 { format!("{} AM", h) } else if h == 12 { "12 PM".to_string() } else { format!("{} PM", h - 12) };
        html! { <div class="calendar-grid__hour-label">{ label }</div> }
    });

    let now_min = date_util::now_minutes();
    let day_columns = day_dates.iter().map(|date_str| {
        let is_today = date_str == today_str;

        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        let date_for_click = date_str.clone();
        let onclick = Callback::from(move |e: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            let date_for_click = date_for_click.clone();
            // Cria o evento já com o horário do ponto clicado (arredondado
            // pro quarto de hora mais próximo) em vez de sempre dia
            // inteiro — mesmo comportamento do Google Calendar.
            let start_min = y_to_snapped_minutes(e.offset_y() as f64);
            let end_min = (start_min + 60).min(23 * 60 + 59);
            let start_time = date_util::format_time(start_min / 60, start_min % 60);
            let end_time = date_util::format_time(end_min / 60, end_min % 60);
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título do evento".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    let mut new_data = data.clone();
                    new_data.add_entry_timed(date_for_click.clone(), title, start_time.clone(), end_time.clone());
                    on_change.emit(new_data);
                }),
            });
        });

        let data_drop = props.data.clone();
        let on_change_drop = props.on_change.clone();
        let dragging_drop = dragging.clone();
        let date_for_drop = date_str.clone();
        let onmouseup = Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            if let Some(idx) = *dragging_drop {
                let mut new_data = data_drop.clone();
                let has_time = new_data.entries.get(idx).map(|en| en.start_time.is_some()).unwrap_or(false);
                if has_time {
                    let start_min = y_to_snapped_minutes(e.offset_y() as f64);
                    let start_time = date_util::format_time(start_min / 60, start_min % 60);
                    new_data.move_entry_time(idx, date_for_drop.clone(), start_time);
                } else {
                    // Evento de dia inteiro/intervalo caiu aqui (raro —
                    // normalmente solta na faixa do topo) — só muda a
                    // data, sem inventar um horário do nada.
                    new_data.move_entry(idx, date_for_drop.clone());
                }
                on_change_drop.emit(new_data);
            }
            dragging_drop.set(None);
        });

        let timed_blocks = props.data.entries.iter().enumerate().filter_map(|(idx, entry)| {
            if entry.date.as_deref() != Some(date_str.as_str()) {
                return None;
            }
            let (sh, sm) = date_util::parse_time(entry.start_time.as_deref()?)?;
            let start_min = date_util::minutes_since_midnight(sh, sm);
            let end_min = entry.end_time.as_deref()
                .and_then(date_util::parse_time)
                .map(|(h, m)| date_util::minutes_since_midnight(h, m))
                .unwrap_or(start_min + 60);

            // Enquanto arrasta a borda de cima/baixo, mostra a duração em
            // tempo real seguindo o cursor em vez de só no soltar — mesmo
            // princípio do ghost de drag (feedback visual imediato).
            let is_resizing_start = **resizing == Some((idx, true));
            let is_resizing_end = **resizing == Some((idx, false));
            let (preview_start, preview_end) = match (is_resizing_start, is_resizing_end, **resize_preview_min) {
                (true, _, Some(new_min)) => (new_min.min(end_min.saturating_sub(15)), end_min),
                (_, true, Some(new_min)) => (start_min, new_min.max(start_min + 15).min(23 * 60 + 59)),
                _ => (start_min, end_min),
            };
            let top = (preview_start as f64 / 60.0) * HOUR_PX;
            let height = (((preview_end.max(preview_start + 15) - preview_start) as f64) / 60.0 * HOUR_PX).max(18.0);
            let style = format!("top: {top}px; height: {height}px;");
            let class = classes!(
                "calendar-grid__timed-block",
                entry.all_tags().first().map(|t| badge_class(existing_tags, t)),
                (**dragging == Some(idx)).then_some("calendar-grid__bar--dragging"),
                (is_resizing_start || is_resizing_end).then_some("calendar-grid__timed-block--resizing"),
            );
            let onmousedown = {
                let dragging = dragging.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    e.prevent_default();
                    dragging.set(Some(idx));
                })
            };
            let onmouseup = {
                let dragging = dragging.clone();
                let editing_entry = editing_entry.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    if *dragging == Some(idx) {
                        editing_entry.set(Some(idx));
                    }
                    dragging.set(None);
                })
            };
            let onmousedown_top = {
                let resizing = resizing.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    e.prevent_default();
                    resizing.set(Some((idx, true)));
                })
            };
            let onmousedown_bottom = {
                let resizing = resizing.clone();
                Callback::from(move |e: MouseEvent| {
                    e.stop_propagation();
                    e.prevent_default();
                    resizing.set(Some((idx, false)));
                })
            };
            Some(html! {
                <div {class} {style} {onmousedown} {onmouseup} title={entry.title.clone()}>
                    <div class="calendar-grid__resize-handle calendar-grid__resize-handle--top" onmousedown={onmousedown_top} />
                    { &entry.title }
                    <div class="calendar-grid__resize-handle calendar-grid__resize-handle--bottom" onmousedown={onmousedown_bottom} />
                </div>
            })
        });

        html! {
            <div class="calendar-grid__day-column" {onclick} {onmouseup}>
                { for timed_blocks }
                if is_today {
                    <div class="calendar-grid__now-line" style={format!("top: {}px;", (now_min as f64 / 60.0) * HOUR_PX)}>
                        <span class="calendar-grid__now-dot" />
                    </div>
                }
            </div>
        }
    });

    html! {
        <>
        <div class="calendar-grid__daygrid-labels">
            <div class="calendar-grid__daygrid-gutter-spacer" />
            { for day_labels }
        </div>
        <div class="calendar-grid__allday-strip">
            <div class="calendar-grid__daygrid-gutter-spacer" />
            <div class="calendar-grid__allday-columns" style={format!("grid-template-columns: repeat({n}, 1fr);")}>
                { for allday_bgs }
                { for allday_bars }
            </div>
        </div>
        <div class="calendar-grid__hour-scroll" ref={hour_scroll_ref.clone()}>
            <div class="calendar-grid__hour-gutter">
                { for hour_labels }
            </div>
            <div class="calendar-grid__hour-columns">
                { for day_columns }
            </div>
        </div>
        </>
    }
}
