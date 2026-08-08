//! Modal de configuração de atalhos do vim mode — cada ação tem uma
//! tecla; clicar num botão entra em modo "capturar" e a próxima tecla
//! pressionada vira o novo atalho daquela ação (Escape cancela).

use web_sys::KeyboardEvent;
use yew::prelude::*;

use crate::components::modal::Modal;
use crate::state::VimKeymap;

/// Props do `VimSettingsModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct VimSettingsModalProps {
    /// Mapa de teclas atual.
    pub keymap: VimKeymap,
    /// Disparado com o mapa atualizado a cada remapeamento.
    pub on_change: Callback<VimKeymap>,
    /// Fecha o modal.
    pub on_close: Callback<()>,
}

#[function_component(VimSettingsModal)]
pub fn vim_settings_modal(props: &VimSettingsModalProps) -> Html {
    let capturing: UseStateHandle<Option<&'static str>> = use_state(|| None);
    let fields = props.keymap.labeled_fields();

    html! {
        <Modal title={"Atalhos do Vim mode".to_string()} open={true} on_close={props.on_close.clone()}>
            <p class="vim-settings__hint">
                { "Clique numa tecla e pressione a nova — Esc cancela. \"Apagar linha\"/\"Copiar linha\" pedem a tecla 2x seguidas (como dd/yy do vim de verdade)." }
            </p>
            <div class="vim-settings">
                { for fields.iter().map(|&(label, key)| {
                    let is_capturing = *capturing == Some(label);
                    let capturing_set = capturing.clone();
                    let keymap = props.keymap.clone();
                    let on_change = props.on_change.clone();
                    let onclick = Callback::from(move |_| capturing_set.set(Some(label)));
                    let capturing_kd = capturing.clone();
                    let onkeydown = Callback::from(move |e: KeyboardEvent| {
                        e.prevent_default();
                        e.stop_propagation();
                        let key = e.key();
                        capturing_kd.set(None);
                        if key == "Escape" { return; }
                        let mut new_keymap = keymap.clone();
                        new_keymap.set_by_label(label, key);
                        on_change.emit(new_keymap);
                    });
                    html! {
                        <div class="vim-settings__row">
                            <span class="vim-settings__label">{ label }</span>
                            if is_capturing {
                                <input class="vim-settings__key-input" autofocus=true
                                    placeholder="pressione uma tecla..." {onkeydown} value="" />
                            } else {
                                <button class="vim-settings__key-btn" {onclick}>{ key }</button>
                            }
                        </div>
                    }
                }) }
            </div>
        </Modal>
    }
}
