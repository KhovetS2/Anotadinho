//! Modal de detalhes de um evento do calendário — título, data de
//! início/fim (via `DatePicker`) e tag/cor. Mesmo padrão do
//! `CardDetailModal`/`ColumnSettingsModal`: reaproveita `Modal`, commit
//! imediato a cada campo, sem botão "Salvar" separado.

use wasm_bindgen::JsCast;
use web_sys::FocusEvent;
use yew::prelude::*;

use crate::components::date_picker::DatePicker;
use crate::components::modal::Modal;
use crate::embed::{badge_class, CalendarEntry};

#[derive(Clone, Copy, PartialEq)]
enum DateField {
    Start,
    End,
}

/// Props do `EventDetailModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct EventDetailModalProps {
    /// Evento sendo editado (snapshot atual).
    pub entry: CalendarEntry,
    /// Tags já usadas em outros eventos do calendário, pra reaproveitar
    /// em vez de criar cor nova toda hora.
    pub existing_tags: Vec<String>,
    /// Disparado a cada mudança no evento (título, datas, tag).
    pub on_change: Callback<CalendarEntry>,
    /// Disparado ao excluir o evento.
    pub on_delete: Callback<()>,
    /// Disparado ao fechar o modal.
    pub on_close: Callback<()>,
}

fn text_of(e: FocusEvent) -> Option<String> {
    let target = e.target()?;
    let el = target.dyn_into::<web_sys::Element>().ok()?;
    el.text_content()
}

/// Modal de detalhes do evento do calendário.
#[function_component(EventDetailModal)]
pub fn event_detail_modal(props: &EventDetailModalProps) -> Html {
    let open_field = use_state(|| None::<DateField>);
    let entry = &props.entry;
    let has_range = entry.end_date.is_some();

    let on_title_blur = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: FocusEvent| {
            if let Some(text) = text_of(e) {
                if text != entry.title {
                    let mut new_entry = entry.clone();
                    new_entry.title = text;
                    on_change.emit(new_entry);
                }
            }
        })
    };

    let toggle_open = |field: DateField| {
        let open_field = open_field.clone();
        Callback::from(move |e: MouseEvent| {
            e.stop_propagation();
            open_field.set(if *open_field == Some(field) { None } else { Some(field) });
        })
    };
    let close_picker = {
        let open_field = open_field.clone();
        Callback::from(move |_: ()| open_field.set(None))
    };

    let pick_start = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        let open_field = open_field.clone();
        Callback::from(move |date: String| {
            let mut new_entry = entry.clone();
            // Move o fim junto, preservando a duração — evita que o fim
            // fique antes do início se o usuário só mexer na data inicial.
            if let Some(end) = &new_entry.end_date {
                let duration = crate::date_util::days_between(&new_entry.date, end).unwrap_or(0);
                new_entry.end_date = crate::date_util::add_days(&date, duration);
            }
            new_entry.date = date;
            on_change.emit(new_entry);
            open_field.set(None);
        })
    };
    let pick_end = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        let open_field = open_field.clone();
        Callback::from(move |date: String| {
            let mut new_entry = entry.clone();
            new_entry.end_date = Some(date);
            on_change.emit(new_entry);
            open_field.set(None);
        })
    };

    let toggle_range = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_entry = entry.clone();
            new_entry.end_date = if new_entry.end_date.is_some() { None } else { Some(new_entry.date.clone()) };
            on_change.emit(new_entry);
        })
    };

    let new_tag_text = use_state(String::new);
    let pick_tag = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        move |tag: Option<String>| {
            let entry = entry.clone();
            let on_change = on_change.clone();
            Callback::from(move |_: MouseEvent| {
                let mut new_entry = entry.clone();
                new_entry.tag = tag.clone();
                on_change.emit(new_entry);
            })
        }
    };
    let submit_new_tag = {
        let entry = entry.clone();
        let on_change = props.on_change.clone();
        let new_tag_text = new_tag_text.clone();
        Callback::from(move |_: MouseEvent| {
            let value = new_tag_text.trim().to_string();
            if value.is_empty() {
                return;
            }
            let mut new_entry = entry.clone();
            new_entry.tag = Some(value);
            on_change.emit(new_entry);
            new_tag_text.set(String::new());
        })
    };

    html! {
        <Modal title="Evento" open={true} on_close={props.on_close.clone()}>
            <div
                class="card-modal__title"
                contenteditable="true"
                onblur={on_title_blur}
            >
                { &entry.title }
            </div>

            <div class="card-modal__section">
                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Início" }</label>
                    <div class="event-modal__date-field">
                        <button class="event-modal__date-chip" onclick={toggle_open(DateField::Start)}>{ &entry.date }</button>
                        if *open_field == Some(DateField::Start) {
                            <DatePicker value={Some(entry.date.clone())} on_pick={pick_start} on_close={close_picker.clone()} />
                        }
                    </div>
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label event-modal__range-toggle">
                        <input class="checkbox" type="checkbox" checked={has_range} onclick={toggle_range} />
                        { "Vários dias" }
                    </label>
                    if has_range {
                        <div class="event-modal__date-field">
                            <button class="event-modal__date-chip" onclick={toggle_open(DateField::End)}>
                                { entry.end_date.clone().unwrap_or_default() }
                            </button>
                            if *open_field == Some(DateField::End) {
                                <DatePicker value={entry.end_date.clone()} on_pick={pick_end} on_close={close_picker.clone()} />
                            }
                        </div>
                    }
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Tag" }</label>
                    <div class="card-modal__tags">
                        { for props.existing_tags.iter().map(|tag| {
                            let is_active = entry.tag.as_deref() == Some(tag.as_str());
                            let class = classes!("badge", badge_class(&props.existing_tags, tag), is_active.then_some("event-modal__tag--active"));
                            let onclick = pick_tag(if is_active { None } else { Some(tag.clone()) });
                            html! { <span {class} {onclick}>{ tag }</span> }
                        }) }
                        <input
                            class="column-settings__add-option-input"
                            type="text"
                            placeholder="Nova tag"
                            value={(*new_tag_text).clone()}
                            oninput={{
                                let new_tag_text = new_tag_text.clone();
                                Callback::from(move |e: InputEvent| {
                                    if let Some(target) = e.target() {
                                        if let Ok(el) = target.dyn_into::<web_sys::HtmlInputElement>() {
                                            new_tag_text.set(el.value());
                                        }
                                    }
                                })
                            }}
                        />
                        <button class="card-modal__add-chip" onclick={submit_new_tag}>{ "+ tag" }</button>
                    </div>
                </div>
            </div>

            <button class="btn btn--danger btn--sm event-modal__delete" onclick={props.on_delete.reform(|_: MouseEvent| ())}>{ "Excluir evento" }</button>
        </Modal>
    }
}
