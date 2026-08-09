//! Cheatsheet de atalhos (ciclo 108) — overlay somente leitura com
//! todos os binds atuais do `GlobalKeymap` (sempre) e do `VimKeymap`
//! (só se o vim mode estiver ativado), lado a lado. Pra mudar um
//! atalho, ainda usa os modais de configuração dedicados — este é só
//! um resumo rápido de consulta.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::modal::Modal;
use crate::state::{GlobalKeymap, VimKeymap};

/// Props do `CheatsheetModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct CheatsheetModalProps {
    pub global_keymap: GlobalKeymap,
    pub vim_keymap: VimKeymap,
    pub vim_mode_enabled: bool,
    pub on_close: Callback<()>,
}

#[function_component(CheatsheetModal)]
pub fn cheatsheet_modal(props: &CheatsheetModalProps) -> Html {
    // `Modal` genérico não trata Escape sozinho (só clique fora/✕) —
    // critério de aceite deste ciclo pede Escape explicitamente.
    {
        let on_close = props.on_close.clone();
        use_effect_with((), move |_| {
            let window = web_sys::window().expect("no global window");
            let listener = EventListener::new(&window, "keydown", move |e| {
                if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                    if e.key() == "Escape" {
                        on_close.emit(());
                    }
                }
            });
            move || drop(listener)
        });
    }

    html! {
        <Modal title={"Atalhos".to_string()} open={true} on_close={props.on_close.clone()} wide=true>
            <div class="cheatsheet">
                <div class="cheatsheet__section">
                    <h4 class="cheatsheet__heading">{ "Globais (Ctrl/Cmd + tecla)" }</h4>
                    <ul class="cheatsheet__list">
                        { for props.global_keymap.labeled_fields().iter().map(|(label, key)| html! {
                            <li class="cheatsheet__row">
                                <span class="cheatsheet__label">{ *label }</span>
                                <kbd class="cheatsheet__key">
                                    { if key.is_empty() { "—".to_string() } else { format!("Ctrl+{}", key) } }
                                </kbd>
                            </li>
                        }) }
                    </ul>
                </div>
                <div class="cheatsheet__section">
                    <h4 class="cheatsheet__heading">{ "Navegação (fixos, não remapeáveis)" }</h4>
                    <ul class="cheatsheet__list">
                        <li class="cheatsheet__row">
                            <span class="cheatsheet__label">{ "Mover foco" }</span>
                            <kbd class="cheatsheet__key">{ "Tab / Shift+Tab" }</kbd>
                        </li>
                        <li class="cheatsheet__row">
                            <span class="cheatsheet__label">{ "Fechar modal, menu ou paleta aberta" }</span>
                            <kbd class="cheatsheet__key">{ "Escape" }</kbd>
                        </li>
                        <li class="cheatsheet__row">
                            <span class="cheatsheet__label">{ "Navegar itens de um menu/paleta aberta" }</span>
                            <kbd class="cheatsheet__key">{ "↑ / ↓" }</kbd>
                        </li>
                        <li class="cheatsheet__row">
                            <span class="cheatsheet__label">{ "Ativar item focado (nó do grafo, card, linha, chip)" }</span>
                            <kbd class="cheatsheet__key">{ "Enter / Espaço" }</kbd>
                        </li>
                    </ul>
                </div>
                if props.vim_mode_enabled {
                    <div class="cheatsheet__section">
                        <h4 class="cheatsheet__heading">{ "Vim mode (modo Normal)" }</h4>
                        <ul class="cheatsheet__list">
                            { for props.vim_keymap.labeled_fields().iter().map(|(label, key)| html! {
                                <li class="cheatsheet__row">
                                    <span class="cheatsheet__label">{ *label }</span>
                                    <kbd class="cheatsheet__key">{ *key }</kbd>
                                </li>
                            }) }
                        </ul>
                    </div>
                }
            </div>
        </Modal>
    }
}
