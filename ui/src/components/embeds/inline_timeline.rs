//! Cronograma de barras (`{{ type: "timeline" }}`) — a visão de
//! duração que faltava.
//!
//! O calendário mostra ocupação por dia; nada mostrava um projeto
//! atravessando semanas. Aqui cada item é uma barra posicionada pelo
//! intervalo `start`..`end`, arrastável (move preservando a duração) e
//! redimensionável pelas bordas.
//!
//! O arraste usa eventos de mouse simples + listener global de
//! `mouseup`, igual ao kanban e ao calendário inline: a API nativa de
//! drag-and-drop do HTML5 é instável no WebKitGTK, e o listener global
//! garante que o estado nunca fica preso se o mouse soltar fora da
//! grade. `user-select: none` durante o arraste evita o texto da página
//! ser selecionado junto (bug do ciclo 068).
//!
//! Modo Vault é somente leitura — igual ao `CalendarSource::Vault`: o
//! item É uma página, então editar acontece na página.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::components::icon::Icon;
use crate::date_util;
use crate::embed::{
    badge_class, bar_span, TimelineEmbedData, TimelineItem, TimelineScale, TimelineSource,
};

/// O que está sendo arrastado.
#[derive(Debug, Clone, Copy, PartialEq)]
enum DragMode {
    /// Move a barra inteira.
    Move,
    /// Arrasta a borda esquerda.
    ResizeStart,
    /// Arrasta a borda direita.
    ResizeEnd,
}

/// Props do `InlineTimeline`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineTimelineProps {
    /// Itens e configuração.
    pub data: TimelineEmbedData,
    /// Path do vault (modo Vault).
    pub vault_path: String,
    /// Disparado quando itens/escala/fonte mudam.
    pub on_change: Callback<TimelineEmbedData>,
    /// Abre a página de origem (modo Vault).
    pub on_page_selected: Callback<PageMeta>,
    /// Abre o modal de diálogo do app (criar/renomear item).
    pub open_dialog: Callback<crate::dialog::PendingDialog>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Cronograma inline.
#[function_component(InlineTimeline)]
pub fn inline_timeline(props: &InlineTimelineProps) -> Html {
    // Início da janela visível. `None` = ancorado em "hoje" (recalculado
    // a cada render, pra a página não abrir mostrando semana passada).
    let anchor = use_state(|| None::<String>);
    let dragging = use_state(|| None::<(usize, DragMode)>);
    // Deslocamento em dias do arraste em curso. Dois espelhos do mesmo
    // valor de propósito: o `use_state` re-renderiza a pré-visualização
    // da barra, e o `use_mut_ref` é o que o `mouseup` LÊ — um handle de
    // `use_state` capturado por um efeito fica congelado no valor de
    // quando o efeito foi criado (mesmo motivo do `edited_ref` no
    // editor), e o commit saía sempre com 0 dias.
    let drag_days = use_state(|| 0i64);
    let drag_days_ref = use_mut_ref(|| 0i64);
    let vault_items = use_state(Vec::<TimelineItem>::new);
    let track_ref = use_node_ref();

    let scale = props.data.scale;
    let window_days = scale.days();
    let today = date_util::today_string();
    let window_start = (*anchor)
        .clone()
        .unwrap_or_else(|| date_util::add_days(&today, -(window_days / 4)).unwrap_or_else(|| today.clone()));

    // Modo Vault: uma varredura só (ciclo 150). Páginas com `start`/`date`
    // no frontmatter viram barras; `end`/`due` fecham o intervalo.
    {
        let vault_items = vault_items.clone();
        let vault_path = props.vault_path.clone();
        let is_vault = props.data.source == TimelineSource::Vault;
        use_effect_with((vault_path, is_vault), move |(vault_path, is_vault)| {
            if *is_vault {
                let vault_path = vault_path.clone();
                let vault_items = vault_items.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let entries = api::scan_vault(&vault_path).await.unwrap_or_default();
                    let items = entries
                        .iter()
                        .filter_map(|e| {
                            let start = e
                                .properties
                                .get("start")
                                .or_else(|| e.properties.get("date"))?
                                .clone();
                            Some(TimelineItem {
                                title: e.title.clone(),
                                start: Some(start),
                                end: e.properties.get("end").or_else(|| e.properties.get("due")).cloned(),
                                tags: e.tags.clone(),
                                page: Some(e.path.clone()),
                            })
                        })
                        .collect::<Vec<_>>();
                    vault_items.set(items);
                });
            }
            || {}
        });
    }

    // Enquanto arrasta: converte o movimento horizontal do mouse em dias
    // (a largura da trilha vale `window_days`), e o `mouseup` global
    // commita — sem isso o estado ficaria preso se soltar fora da grade.
    {
        let dragging = dragging.clone();
        let drag_days = drag_days.clone();
        let drag_days_ref = drag_days_ref.clone();
        let track_ref = track_ref.clone();
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let is_manual = props.data.source == TimelineSource::Manual;
        use_effect_with(*dragging, move |current| {
            let Some((idx, mode)) = *current else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let window = web_sys::window().expect("no global window");

            let start_x = std::rc::Rc::new(std::cell::Cell::new(None::<i32>));
            let move_days = drag_days.clone();
            let move_days_ref = drag_days_ref.clone();
            let move_track = track_ref.clone();
            let move_start_x = start_x.clone();
            let mousemove = EventListener::new(&window, "mousemove", move |e| {
                let Some(e) = e.dyn_ref::<web_sys::MouseEvent>() else { return };
                if move_start_x.get().is_none() {
                    move_start_x.set(Some(e.client_x()));
                }
                let Some(el) = move_track.cast::<web_sys::Element>() else { return };
                let width = el.get_bounding_client_rect().width();
                if width <= 0.0 {
                    return;
                }
                let dx = (e.client_x() - move_start_x.get().unwrap_or(e.client_x())) as f64;
                let days = (dx / (width / window_days as f64)).round() as i64;
                *move_days_ref.borrow_mut() = days;
                move_days.set(days);
            });

            let up_dragging = dragging.clone();
            let up_days = drag_days.clone();
            let up_days_ref = drag_days_ref.clone();
            let mouseup = EventListener::new(&window, "mouseup", move |_| {
                let days = *up_days_ref.borrow();
                if is_manual && days != 0 {
                    if let Some(item) = data.items.get(idx) {
                        let mut next = data.clone();
                        match mode {
                            DragMode::Move => {
                                if let Some(new_start) =
                                    item.start.as_deref().and_then(|s| date_util::add_days(s, days))
                                {
                                    next.move_item(idx, new_start);
                                }
                            }
                            DragMode::ResizeStart => {
                                if let Some(new_date) =
                                    item.start.as_deref().and_then(|s| date_util::add_days(s, days))
                                {
                                    next.resize_item(idx, true, new_date);
                                }
                            }
                            DragMode::ResizeEnd => {
                                let base = item.end.clone().or_else(|| item.start.clone());
                                if let Some(new_date) =
                                    base.as_deref().and_then(|s| date_util::add_days(s, days))
                                {
                                    next.resize_item(idx, false, new_date);
                                }
                            }
                        }
                        on_change.emit(next);
                    }
                }
                *up_days_ref.borrow_mut() = 0;
                up_days.set(0);
                up_dragging.set(None);
            });

            Box::new(move || {
                drop(mousemove);
                drop(mouseup);
            })
        });
    }

    let items: Vec<TimelineItem> = if props.data.source == TimelineSource::Vault {
        (*vault_items).clone()
    } else {
        props.data.items.clone()
    };
    let is_manual = props.data.source == TimelineSource::Manual;

    let shift_window = {
        let anchor = anchor.clone();
        let window_start = window_start.clone();
        move |delta: i64| {
            let anchor = anchor.clone();
            let window_start = window_start.clone();
            Callback::from(move |_: MouseEvent| {
                anchor.set(date_util::add_days(&window_start, delta));
            })
        }
    };

    let on_today = {
        let anchor = anchor.clone();
        Callback::from(move |_| anchor.set(None))
    };

    let on_add = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        let today = today.clone();
        Callback::from(move |_| {
            let data = data.clone();
            let on_change = on_change.clone();
            let today = today.clone();
            open_dialog.emit(crate::dialog::PendingDialog::Prompt {
                title: "Título da etapa".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |title: String| {
                    if title.trim().is_empty() {
                        return;
                    }
                    let mut next = data.clone();
                    let end = date_util::add_days(&today, 4).unwrap_or_else(|| today.clone());
                    next.add_item(title, today.clone(), end);
                    on_change.emit(next);
                }),
            });
        })
    };

    // Marcas do eixo: uma por semana (ou por dia na escala Semana).
    let tick_step = if scale == TimelineScale::Week { 1 } else { 7 };
    let ticks: Vec<(f64, String)> = (0..window_days)
        .step_by(tick_step as usize)
        .filter_map(|offset| {
            let date = date_util::add_days(&window_start, offset)?;
            let (_, m, d) = date_util::parse_date(&date)?;
            Some((
                offset as f64 * 100.0 / window_days as f64,
                format!("{d:02}/{m:02}"),
            ))
        })
        .collect();

    let today_pos = date_util::days_between(&window_start, &today)
        .filter(|d| *d >= 0 && *d < window_days)
        .map(|d| d as f64 * 100.0 / window_days as f64);

    let unscheduled: Vec<(usize, &TimelineItem)> = items
        .iter()
        .enumerate()
        .filter(|(_, i)| i.start.is_none())
        .collect();

    let nav_group = props.nav_group.clone();

    html! {
        <div class={classes!("timeline", dragging.is_some().then_some("timeline--dragging"))}
            data-nav-group={nav_group.clone()} data-nav-item={nav_group.clone()} data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
            <div class="timeline__bar">
                <button class="timeline__btn" type="button" title="Período anterior"
                    data-nav-item="timeline-prev" data-nav-parent={nav_group.clone()}
                    onclick={shift_window(-window_days)}><Icon name="chevron-left" /></button>
                <span class="timeline__range">{ format!("{} · {}", window_start, scale.label()) }</span>
                <button class="timeline__btn" type="button" title="Próximo período"
                    data-nav-item="timeline-next" data-nav-parent={nav_group.clone()}
                    onclick={shift_window(window_days)}><Icon name="chevron-right" /></button>
                <button class="timeline__btn timeline__btn--text" type="button"
                    data-nav-item="timeline-today" data-nav-parent={nav_group.clone()}
                    onclick={on_today}>{ "Hoje" }</button>

                <div class="timeline__scales">
                    { for TimelineScale::all().iter().map(|s| {
                        let s = *s;
                        let is_active = s == scale;
                        let onclick = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            Callback::from(move |_| {
                                let mut next = data.clone();
                                next.scale = s;
                                on_change.emit(next);
                            })
                        };
                        html! {
                            <button class={classes!("timeline__scale", is_active.then_some("timeline__scale--active"))}
                                type="button" data-nav-item="timeline-scale" data-nav-parent={nav_group.clone()}
                                {onclick}>{ s.label() }</button>
                        }
                    }) }
                </div>

                <button class="timeline__btn timeline__btn--text" type="button"
                    title="Alternar entre itens do embed e páginas do vault"
                    data-nav-item="timeline-source" data-nav-parent={nav_group.clone()}
                    onclick={{
                        let data = props.data.clone();
                        let on_change = props.on_change.clone();
                        Callback::from(move |_| {
                            let mut next = data.clone();
                            next.source = if next.source == TimelineSource::Manual {
                                TimelineSource::Vault
                            } else {
                                TimelineSource::Manual
                            };
                            on_change.emit(next);
                        })
                    }}>
                    { if is_manual { "Manual" } else { "Vault" } }
                </button>

                if is_manual {
                    <button class="timeline__add" type="button"
                        data-nav-item="timeline-add" data-nav-parent={nav_group.clone()}
                        onclick={on_add}>{ "+ etapa" }</button>
                }
            </div>

            <div class="timeline__axis">
                { for ticks.iter().map(|(pos, label)| html! {
                    <span class="timeline__tick" style={format!("left: {pos}%;")}>{ label.clone() }</span>
                }) }
            </div>

            <div class="timeline__track" ref={track_ref}>
                if let Some(pos) = today_pos {
                    <div class="timeline__today" style={format!("left: {pos}%;")} title="Hoje" />
                }
                { for items.iter().enumerate().filter(|(_, item)| item.start.is_some()).map(|(idx, item)| {
                    let drag_offset = if matches!(*dragging, Some((i, DragMode::Move)) if i == idx) { *drag_days } else { 0 };
                    let resize_start = if matches!(*dragging, Some((i, DragMode::ResizeStart)) if i == idx) { *drag_days } else { 0 };
                    let resize_end = if matches!(*dragging, Some((i, DragMode::ResizeEnd)) if i == idx) { *drag_days } else { 0 };
                    // Pré-visualização: a barra segue o mouse durante o
                    // arraste sem gravar nada — o commit é no mouseup.
                    let preview_start = item.start.as_deref()
                        .and_then(|s| date_util::add_days(s, drag_offset + resize_start));
                    let preview_end = item.end.as_deref().or(item.start.as_deref())
                        .and_then(|s| date_util::add_days(s, drag_offset + resize_end));
                    let Some((left, width)) = bar_span(
                        preview_start.as_deref(),
                        preview_end.as_deref(),
                        &window_start,
                        window_days,
                    ) else {
                        return html! {};
                    };
                    let color = item.tags.first()
                        .map(|t| badge_class(&item.tags, t))
                        .unwrap_or("badge--info");

                    let start_drag = |mode: DragMode| {
                        let dragging = dragging.clone();
                        Callback::from(move |e: MouseEvent| {
                            e.prevent_default();
                            e.stop_propagation();
                            dragging.set(Some((idx, mode)));
                        })
                    };
                    let on_open = {
                        let page = item.page.clone();
                        let title = item.title.clone();
                        let on_page_selected = props.on_page_selected.clone();
                        Callback::from(move |_| {
                            let Some(path) = page.clone() else { return };
                            on_page_selected.emit(PageMeta {
                                path: path.clone(),
                                title: title.clone(),
                                section: if path.starts_with("journals/") { "journals".into() } else { "pages".into() },
                            });
                        })
                    };

                    html! {
                        <div class="timeline__row" key={idx}>
                            <div class={classes!("timeline__bar-item", color)}
                                style={format!("left: {left}%; width: {width}%;")}
                                title={item.title.clone()}
                                tabindex="0" role="button"
                                data-nav-item="timeline-item" data-nav-parent={nav_group.clone()}
                                onmousedown={if is_manual { start_drag(DragMode::Move) } else { Callback::noop() }}
                                onkeydown={{
                                    // Alt+setas movem a barra por dia;
                                    // com Shift, esticam a ponta final
                                    // (ciclo 167). Sem Alt as setas
                                    // continuam navegando entre itens.
                                    let data = props.data.clone();
                                    let on_change = props.on_change.clone();
                                    let item = item.clone();
                                    Callback::from(move |e: web_sys::KeyboardEvent| {
                                        if !is_manual || !e.alt_key() {
                                            return;
                                        }
                                        let delta = match e.key().as_str() {
                                            "ArrowLeft" => -1,
                                            "ArrowRight" => 1,
                                            _ => return,
                                        };
                                        e.prevent_default();
                                        e.stop_propagation();
                                        let mut novo = data.clone();
                                        if e.shift_key() {
                                            let base = item.end.clone().or_else(|| item.start.clone());
                                            if let Some(nova) = base.as_deref().and_then(|d| date_util::add_days(d, delta)) {
                                                novo.resize_item(idx, false, nova);
                                            }
                                        } else if let Some(nova) =
                                            item.start.as_deref().and_then(|d| date_util::add_days(d, delta))
                                        {
                                            novo.move_item(idx, nova);
                                        }
                                        on_change.emit(novo);
                                    })
                                }}
                                onclick={on_open}>
                                if is_manual {
                                    <span class="timeline__handle timeline__handle--start"
                                        onmousedown={start_drag(DragMode::ResizeStart)} />
                                }
                                <span class="timeline__label">{ item.title.clone() }</span>
                                if is_manual {
                                    <span class="timeline__handle timeline__handle--end"
                                        onmousedown={start_drag(DragMode::ResizeEnd)} />
                                }
                            </div>
                        </div>
                    }
                }) }
                if items.iter().all(|i| i.start.is_none()) {
                    <p class="timeline__empty">
                        { if is_manual { "Nenhuma etapa com data — use \"+ etapa\"." } else { "Nenhuma página do vault com start:: ou date::." } }
                    </p>
                }
            </div>

            if is_manual && !unscheduled.is_empty() {
                <div class="timeline__drawer">
                    <span class="timeline__drawer-label">{ "Sem data" }</span>
                    { for unscheduled.iter().map(|(idx, item)| {
                        let idx = *idx;
                        let onclick = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            let window_start = window_start.clone();
                            Callback::from(move |_| {
                                let mut next = data.clone();
                                next.move_item(idx, window_start.clone());
                                on_change.emit(next);
                            })
                        };
                        html! {
                            <button class="timeline__drawer-item" type="button"
                                title="Agendar no início do período visível"
                                data-nav-item="timeline-unscheduled" data-nav-parent={nav_group.clone()}
                                {onclick}>{ item.title.clone() }</button>
                        }
                    }) }
                </div>
            }
        </div>
    }
}
