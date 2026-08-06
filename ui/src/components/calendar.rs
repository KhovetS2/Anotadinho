//! Calendar view - mostra paginas com date:: property.

use yew::prelude::*;
use crate::api::{self, PageMeta};

#[derive(Properties, PartialEq, Clone)]
pub struct CalendarProps {
    pub vault_path: String,
    pub on_page_selected: Callback<PageMeta>,
}

#[derive(Debug, Clone, PartialEq)]
struct DayItem { path: String, title: String, date: String }

#[function_component(Calendar)]
pub fn calendar(props: &CalendarProps) -> Html {
    let items = use_state(Vec::<DayItem>::new);
    let loading = use_state(|| true);

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
                if let Ok(pages) = api::list_pages(&vault_path).await {
                    let mut list = Vec::new();
                    for page in &pages {
                        if let Ok(content) = api::read_page(&vault_path, &page.path).await {
                            for line in content.lines() {
                                if let Some(date) = line.trim().strip_prefix("date:: ") {
                                    list.push(DayItem {
                                        path: page.path.clone(),
                                        title: page.title.clone(),
                                        date: date.trim().to_string(),
                                    });
                                }
                            }
                        }
                    }
                    list.sort_by(|a, b| a.date.cmp(&b.date));
                    items.set(list);
                }
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="calendar"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let on_page_selected = props.on_page_selected.clone();

    let grouped: std::collections::BTreeMap<&str, Vec<&DayItem>> = {
        let mut map: std::collections::BTreeMap<&str, Vec<&DayItem>> = std::collections::BTreeMap::new();
        for item in items.iter() {
            map.entry(&item.date[..]).or_default().push(item);
        }
        map
    };

    html! {
        <div class="calendar">
            <div class="calendar__header">
                <h2>{"Calendário"}</h2>
                <span class="calendar__count">{ items.len() } {" eventos"}</span>
            </div>
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
                                    let on_page_selected = on_page_selected.clone();
                                    html! {
                                        <div class="calendar__item"
                                            onclick={Callback::from(move |_| on_page_selected.emit(meta.clone()))}
                                        >
                                            <span class="calendar__item-title">{ &item.title }</span>
                                        </div>
                                    }
                                }) }
                            </div>
                        }
                    }) }
                }
            </div>
        </div>
    }
}
