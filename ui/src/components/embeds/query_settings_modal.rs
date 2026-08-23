//! Modal de configuração de um embed de consulta: pasta, tags,
//! condições (campo/operador/valor), ordenação, limite, altura, modo de
//! exibição e campos mostrados.
//!
//! Mesma ideia do `ColumnSettingsModal`: configuração com mais de dois
//! campos não cabe numa cadeia de `PendingDialog::Prompt`.

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::query::{Condition, Query, QueryOp, QueryView, Sort};

/// Props do `QuerySettingsModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct QuerySettingsModalProps {
    /// Consulta atual.
    pub query: Query,
    /// Campos oferecidos no seletor (fixos + os vistos no vault).
    pub known_fields: Vec<String>,
    /// Disparado a cada mudança (o embed grava na hora, sem "aplicar").
    pub on_change: Callback<Query>,
    /// Fecha o modal.
    pub on_close: Callback<()>,
}

fn input_value(e: &Event) -> Option<String> {
    e.target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .map(|el| el.value())
}

fn select_value(e: &Event) -> Option<String> {
    e.target()
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .map(|el| el.value())
}

/// Modal de configuração da consulta.
#[function_component(QuerySettingsModal)]
pub fn query_settings_modal(props: &QuerySettingsModalProps) -> Html {
    let q = &props.query;

    // Todo controle segue o mesmo formato: clona a consulta, muda um
    // campo, emite. Sem estado local — o dono é a fonte da verdade.
    let update = {
        let on_change = props.on_change.clone();
        let query = q.clone();
        move |f: Box<dyn Fn(&mut Query)>| {
            let mut next = query.clone();
            f(&mut next);
            on_change.emit(next);
        }
    };
    let update = std::rc::Rc::new(update);

    let on_from = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = input_value(&e) else { return };
            update(Box::new(move |q| {
                let v = v.trim().to_string();
                q.from = if v.is_empty() { None } else { Some(v) };
            }));
        })
    };

    let on_tags = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = input_value(&e) else { return };
            update(Box::new(move |q| {
                q.tags = v
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }));
        })
    };

    let on_limit = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = input_value(&e) else { return };
            update(Box::new(move |q| {
                q.limit = v.trim().parse::<usize>().ok().filter(|n| *n > 0);
            }));
        })
    };

    let on_max_height = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = input_value(&e) else { return };
            update(Box::new(move |q| {
                q.max_height = v.trim().parse::<u16>().ok().filter(|n| *n > 0);
            }));
        })
    };

    let on_sort_field = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = select_value(&e) else { return };
            update(Box::new(move |q| {
                if v.is_empty() {
                    q.sort = None;
                } else {
                    let desc = q.sort.as_ref().map(|s| s.desc).unwrap_or(false);
                    q.sort = Some(Sort { field: v.clone(), desc });
                }
            }));
        })
    };

    let on_sort_dir = {
        let update = update.clone();
        Callback::from(move |_| {
            update(Box::new(|q| {
                if let Some(sort) = &mut q.sort {
                    sort.desc = !sort.desc;
                }
            }));
        })
    };

    let on_columns = {
        let update = update.clone();
        Callback::from(move |e: Event| {
            let Some(v) = input_value(&e) else { return };
            update(Box::new(move |q| {
                q.columns = v
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect();
            }));
        })
    };

    let on_add_condition = {
        let update = update.clone();
        let first_field = props.known_fields.first().cloned().unwrap_or_else(|| "status".to_string());
        Callback::from(move |_| {
            let field = first_field.clone();
            update(Box::new(move |q| {
                q.conditions.push(Condition { field: field.clone(), op: QueryOp::Eq, value: String::new() });
            }));
        })
    };

    html! {
        <Modal title="Configurar consulta" open={true} wide={true} on_close={props.on_close.clone()}>
            <div class="query-settings">
                <label class="query-settings__row">
                    <span class="query-settings__label">{ "Pasta" }</span>
                    <input class="query-settings__input" type="text" placeholder="pages/specs (vazio = vault inteiro)"
                        value={q.from.clone().unwrap_or_default()} onchange={on_from} />
                </label>

                <label class="query-settings__row">
                    <span class="query-settings__label">{ "Tags" }</span>
                    <input class="query-settings__input" type="text" placeholder="spec, api (todas precisam bater)"
                        value={q.tags.join(", ")} onchange={on_tags} />
                </label>

                <div class="query-settings__row query-settings__row--block">
                    <span class="query-settings__label">{ "Condições" }</span>
                    <div class="query-settings__conditions">
                        { for q.conditions.iter().enumerate().map(|(idx, cond)| {
                            let on_field = {
                                let update = update.clone();
                                Callback::from(move |e: Event| {
                                    let Some(v) = select_value(&e) else { return };
                                    update(Box::new(move |q| {
                                        if let Some(c) = q.conditions.get_mut(idx) { c.field = v.clone(); }
                                    }));
                                })
                            };
                            let on_op = {
                                let update = update.clone();
                                Callback::from(move |e: Event| {
                                    let Some(v) = select_value(&e) else { return };
                                    update(Box::new(move |q| {
                                        if let Some(c) = q.conditions.get_mut(idx) {
                                            c.op = QueryOp::all().iter().copied()
                                                .find(|o| o.symbol() == v)
                                                .unwrap_or_default();
                                        }
                                    }));
                                })
                            };
                            let on_value = {
                                let update = update.clone();
                                Callback::from(move |e: Event| {
                                    let Some(v) = input_value(&e) else { return };
                                    update(Box::new(move |q| {
                                        if let Some(c) = q.conditions.get_mut(idx) { c.value = v.clone(); }
                                    }));
                                })
                            };
                            let on_remove = {
                                let update = update.clone();
                                Callback::from(move |_| {
                                    update(Box::new(move |q| {
                                        if idx < q.conditions.len() { q.conditions.remove(idx); }
                                    }));
                                })
                            };
                            let needs_value = cond.op != QueryOp::Exists;

                            html! {
                                <div class="query-settings__condition" key={idx}>
                                    <select class="query-settings__select" onchange={on_field}>
                                        { for props.known_fields.iter().map(|f| html! {
                                            <option value={f.clone()} selected={*f == cond.field}>{ f.clone() }</option>
                                        }) }
                                    </select>
                                    <select class="query-settings__select" onchange={on_op}>
                                        { for QueryOp::all().iter().map(|o| html! {
                                            <option value={o.symbol()} selected={*o == cond.op}>{ o.label() }</option>
                                        }) }
                                    </select>
                                    if needs_value {
                                        <input class="query-settings__input" type="text" placeholder="valor"
                                            value={cond.value.clone()} onchange={on_value} />
                                    }
                                    <button class="query-settings__btn query-settings__btn--danger" type="button"
                                        title="Remover condição" onclick={on_remove}>
                                        <Icon name="x" />
                                    </button>
                                </div>
                            }
                        }) }
                        <button class="query-settings__add" type="button" onclick={on_add_condition}>
                            { "+ condição" }
                        </button>
                    </div>
                </div>

                <div class="query-settings__row">
                    <span class="query-settings__label">{ "Ordenar por" }</span>
                    <select class="query-settings__select" onchange={on_sort_field}>
                        <option value="" selected={q.sort.is_none()}>{ "— sem ordenação —" }</option>
                        { for props.known_fields.iter().map(|f| html! {
                            <option value={f.clone()} selected={q.sort.as_ref().map(|s| &s.field) == Some(f)}>
                                { f.clone() }
                            </option>
                        }) }
                    </select>
                    if let Some(sort) = &q.sort {
                        <button class="query-settings__btn" type="button" onclick={on_sort_dir}
                            title={if sort.desc { "Decrescente" } else { "Crescente" }}>
                            <Icon name={if sort.desc { "chevron-down" } else { "chevron-up" }} />
                        </button>
                    }
                </div>

                <label class="query-settings__row">
                    <span class="query-settings__label">{ "Limite" }</span>
                    <input class="query-settings__input" type="number" min="1" placeholder="sem limite"
                        value={q.limit.map(|l| l.to_string()).unwrap_or_default()} onchange={on_limit} />
                </label>

                <label class="query-settings__row">
                    <span class="query-settings__label">{ "Altura máxima" }</span>
                    <input class="query-settings__input" type="number" min="1" placeholder="384 px (padrão)"
                        value={q.max_height.map(|h| h.to_string()).unwrap_or_default()} onchange={on_max_height} />
                    <span class="query-settings__hint">{ "px" }</span>
                </label>

                <label class="query-settings__row">
                    <span class="query-settings__label">{ "Campos" }</span>
                    <input class="query-settings__input" type="text" placeholder="status, priority"
                        value={q.columns.join(", ")} onchange={on_columns} />
                </label>

                <div class="query-settings__row">
                    <span class="query-settings__label">{ "Exibição" }</span>
                    <div class="query-settings__views">
                        { for QueryView::all().iter().map(|v| {
                            let v = *v;
                            let is_active = v == q.view;
                            let onclick = {
                                let update = update.clone();
                                Callback::from(move |_| {
                                    update(Box::new(move |q| q.view = v));
                                })
                            };
                            html! {
                                <button class={classes!("query-settings__view", is_active.then_some("query-settings__view--active"))}
                                    type="button" {onclick}>{ v.label() }</button>
                            }
                        }) }
                    </div>
                </div>
            </div>
        </Modal>
    }
}
