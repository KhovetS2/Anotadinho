//! Popover de calendário pra escolher 1 data — substitui o
//! `<input type="date">` nativo em toda a UI (o popup nativo não fecha
//! sozinho no WebKitGTK e não tem identidade visual própria, já que é
//! renderizado pelo SO). Matemática de data em `crate::date_util`.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::date_util;

/// Props do `DatePicker`.
#[derive(Properties, PartialEq, Clone)]
pub struct DatePickerProps {
    /// Data atualmente selecionada (`"YYYY-MM-DD"`), se houver.
    pub value: Option<String>,
    /// Disparado ao clicar num dia — quem usa decide se isso já fecha o
    /// popover (normalmente sim, mesmo padrão dos outros dropdowns).
    pub on_pick: Callback<String>,
    /// Disparado ao clicar fora do popover ou apertar Escape.
    pub on_close: Callback<()>,
}

const WEEKDAY_LABELS: [&str; 7] = ["D", "S", "T", "Q", "Q", "S", "S"];

/// Popover de calendário (mês visível + navegação + atalho "Hoje").
#[function_component(DatePicker)]
pub fn date_picker(props: &DatePickerProps) -> Html {
    let today = date_util::today();
    let initial_view = props
        .value
        .as_deref()
        .and_then(date_util::parse_date)
        .map(|(y, m, _)| (y, m))
        .unwrap_or((today.0, today.1));
    let view = use_state(|| initial_view);

    // Fecha ao clicar fora ou apertar Escape — mesmo padrão já usado nos
    // outros dropdowns do app (header menu, slash menu, select da tabela).
    {
        let on_close = props.on_close.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no global window");

            let close_outside = on_close.clone();
            let mousedown = EventListener::new(&window, "mousedown", move |e| {
                let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                let Some(target) = target else { return };
                if target.closest(".date-picker").ok().flatten().is_none() {
                    close_outside.emit(());
                }
            });

            // Consome o Escape (ciclo 161): sem isso a tecla seguia até
            // o handler global do `app.rs` e desselecionava a página —
            // e, aberto DENTRO de um modal, o Escape fechava o modal
            // inteiro em vez de só este seletor.
            let close_escape = on_close.clone();
            let keydown = crate::menu_keyboard::escape_consumer(move || close_escape.emit(()));

            move || {
                drop(mousedown);
                drop(keydown);
            }
        });
    }

    let (vy, vm) = *view;

    let prev_month = {
        let view = view.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            view.set(date_util::prev_month(vy, vm));
        })
    };
    let next_month = {
        let view = view.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            view.set(date_util::next_month(vy, vm));
        })
    };
    let go_today = {
        let view = view.clone();
        let on_pick = props.on_pick.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            let (y, m, _) = date_util::today();
            view.set((y, m));
            on_pick.emit(date_util::today_string());
        })
    };

    // Grade de 6 semanas × 7 dias: dias do mês anterior/seguinte
    // esmaecidos preenchem as pontas.
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
    let selected = props.value.clone();

    html! {
        <div class="date-picker">
            <div class="date-picker__header">
                <button class="date-picker__nav" onclick={prev_month}>{ "‹" }</button>
                <span class="date-picker__month">{ format!("{} {}", date_util::month_name(vm), vy) }</span>
                <button class="date-picker__nav" onclick={next_month}>{ "›" }</button>
            </div>
            <div class="date-picker__weekdays">
                { for WEEKDAY_LABELS.iter().map(|w| html! { <span>{ *w }</span> }) }
            </div>
            <div class="date-picker__grid">
                { for cells.iter().map(|&(y, m, d, in_month)| {
                    let date_str = date_util::format_date(y, m, d);
                    let is_today = date_str == today_str;
                    let is_selected = selected.as_deref() == Some(date_str.as_str());
                    let class = classes!(
                        "date-picker__day",
                        (!in_month).then_some("date-picker__day--muted"),
                        is_today.then_some("date-picker__day--today"),
                        is_selected.then_some("date-picker__day--selected"),
                    );
                    let on_pick = props.on_pick.clone();
                    let onclick = Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_pick.emit(date_str.clone());
                    });
                    html! { <button {class} {onclick}>{ d }</button> }
                }) }
            </div>
            <div class="date-picker__footer">
                <button class="date-picker__today" onclick={go_today}>{ "Hoje" }</button>
            </div>
        </div>
    }
}
