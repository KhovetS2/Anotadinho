//! Navegação por teclado compartilhada pelos menus dropdown "próprios"
//! do app (⚙, popover de git status, "⋯" do editor) — não usam o
//! componente `Modal` (que já ganhou foco automático/trap/Escape no
//! ciclo 124), são `<div>` popover implementados cada um por conta
//! própria, com abrir/fechar via estado local. Ciclo 125 dá a esses
//! três a mesma navegação por teclado que a paleta de comandos já
//! tinha desde o ciclo 091: foco automático no primeiro item ao abrir,
//! setas pra mover entre os itens.

use gloo_events::{EventListener, EventListenerOptions};
use wasm_bindgen::JsCast;

/// Foca o primeiro `<button>` dentro do container referenciado — chamar
/// quando o menu abre (efeito disparado pela mudança do estado
/// `open`/`*_open`).
pub fn focus_first_item(container: &yew::NodeRef) {
    let Some(el) = container.cast::<web_sys::Element>() else { return };
    let Ok(Some(first)) = el.query_selector("button") else { return };
    if let Ok(html_el) = first.dyn_into::<web_sys::HtmlElement>() {
        let _ = html_el.focus();
    }
}

/// Move o foco entre os `<button>` filhos do container, na direção
/// indicada (`1` = próximo, `-1` = anterior), com wrap-around — chamar
/// no handler de `ArrowDown`/`ArrowUp` do menu.
pub fn move_item_focus(container: &yew::NodeRef, direction: i32) {
    let Some(el) = container.cast::<web_sys::Element>() else { return };
    let Ok(items) = el.query_selector_all("button") else { return };
    let len = items.length();
    if len == 0 {
        return;
    }
    let active = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.active_element());
    let mut current_idx: Option<u32> = None;
    if let Some(active) = &active {
        for i in 0..len {
            if let Some(item) = items.item(i) {
                if active.is_same_node(Some(&item)) {
                    current_idx = Some(i);
                    break;
                }
            }
        }
    }
    let next_idx = match current_idx {
        Some(i) => ((i as i64 + direction as i64).rem_euclid(len as i64)) as u32,
        None => 0,
    };
    if let Some(item) = items.item(next_idx) {
        if let Ok(html_el) = item.dyn_into::<web_sys::HtmlElement>() {
            let _ = html_el.focus();
        }
    }
}

/// Listener de Escape que CONSOME a tecla: fecha o popup e não deixa
/// ninguém mais ver o evento.
///
/// Existe por causa do bug do ciclo 161: `app.rs` desseleciona a página
/// aberta em QUALQUER Escape sem Ctrl, então fechar um menu/popover
/// fazia o editor sumir junto ("Selecione uma página na sidebar", com a
/// aba ainda aberta). Quem abre um popup e trata Escape precisa
/// consumir a tecla.
///
/// Roda na fase de CAPTURA na `window`, então dispara ANTES do handler
/// delegado do Yew (que fica num elemento lá embaixo) — é isso que
/// garante que o popup mais interno ganha: um seletor de data aberto
/// DENTRO de um modal fecha só o seletor, sem levar o modal junto.
///
/// O `EventListener` devolvido tem que ser mantido vivo (guardar no
/// cleanup do `use_effect_with`); ao ser dropado, o listener sai.
pub fn escape_consumer<F: Fn() + 'static>(on_escape: F) -> EventListener {
    let window = web_sys::window().expect("no global window");
    EventListener::new_with_options(
        &window,
        "keydown",
        EventListenerOptions {
            phase: gloo_events::EventListenerPhase::Capture,
            passive: false,
        },
        move |e| {
            let Some(ke) = e.dyn_ref::<web_sys::KeyboardEvent>() else { return };
            if ke.key() != "Escape" {
                return;
            }
            e.stop_propagation();
            e.prevent_default();
            on_escape();
        },
    )
}
