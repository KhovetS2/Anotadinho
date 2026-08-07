//! Popover pra escolher um horário — substitui o `input[type=time]`
//! nativo, mesmo padrão visual do `DatePicker`. Diferente do date, o
//! input de hora nativo não tinha um bug de popup-não-fecha (é um
//! spinner inline, não um overlay separado), mas ainda destoava do resto
//! da UI — esse popover fecha o círculo de identidade visual própria
//! começado com o `DatePicker`.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::date_util;

/// Props do `TimePicker`.
#[derive(Properties, PartialEq, Clone)]
pub struct TimePickerProps {
    /// Horário atualmente selecionado (`"HH:MM"`), se houver.
    pub value: Option<String>,
    /// Disparado ao escolher um horário na lista.
    pub on_pick: Callback<String>,
    /// Disparado ao clicar fora do popover ou apertar Escape.
    pub on_close: Callback<()>,
}

const STEP_MINUTES: u32 = 15;
const ITEM_PX: i32 = 28;

/// Popover de horário: lista rolável de horários de 15 em 15 minutos,
/// já aberta na posição do horário atual/mais próximo.
#[function_component(TimePicker)]
pub fn time_picker(props: &TimePickerProps) -> Html {
    let list_ref = use_node_ref();

    // Fecha ao clicar fora ou apertar Escape — mesmo padrão do
    // `DatePicker` e dos outros dropdowns do app.
    {
        let on_close = props.on_close.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no global window");

            let close_outside = on_close.clone();
            let mousedown = EventListener::new(&window, "mousedown", move |e| {
                let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                let Some(target) = target else { return };
                if target.closest(".time-picker").ok().flatten().is_none() {
                    close_outside.emit(());
                }
            });

            let close_escape = on_close.clone();
            let keydown = EventListener::new(&window, "keydown", move |e| {
                if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                    if e.key() == "Escape" {
                        close_escape.emit(());
                    }
                }
            });

            move || {
                drop(mousedown);
                drop(keydown);
            }
        });
    }

    // Rola a lista pra deixar o horário selecionado (ou o mais próximo de
    // agora, se nenhum) já visível ao abrir, em vez de sempre começar em
    // 00:00.
    {
        let list_ref = list_ref.clone();
        let value = props.value.clone();
        use_effect_with((), move |_| {
            if let Some(el) = list_ref.cast::<web_sys::Element>() {
                let minutes = value
                    .as_deref()
                    .and_then(date_util::parse_time)
                    .map(|(h, m)| date_util::minutes_since_midnight(h, m))
                    .unwrap_or_else(date_util::now_minutes);
                let idx = (minutes / STEP_MINUTES) as i32;
                el.set_scroll_top((idx * ITEM_PX - ITEM_PX * 3).max(0));
            }
            || {}
        });
    }

    let selected = props.value.clone();
    let total_steps = (24 * 60) / STEP_MINUTES;

    html! {
        <div class="time-picker">
            <div class="time-picker__list" ref={list_ref}>
                { for (0..total_steps).map(|i| {
                    let total = i * STEP_MINUTES;
                    let label = date_util::format_time(total / 60, total % 60);
                    let is_selected = selected.as_deref() == Some(label.as_str());
                    let class = classes!("time-picker__item", is_selected.then_some("time-picker__item--selected"));
                    let on_pick = props.on_pick.clone();
                    let label_click = label.clone();
                    let onclick = Callback::from(move |e: MouseEvent| {
                        e.stop_propagation();
                        on_pick.emit(label_click.clone());
                    });
                    html! { <button {class} {onclick}>{ label }</button> }
                }) }
            </div>
        </div>
    }
}
