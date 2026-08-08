//! Modal de configuração de atalhos do vim mode — fino wrapper sobre
//! `KeymapCaptureModal` (ciclo 104), que sabe traduzir de/para
//! `VimKeymap` especificamente.

use yew::prelude::*;

use crate::components::keymap_capture_modal::KeymapCaptureModal;
use crate::state::VimKeymap;

const HINT: &str = "Clique numa tecla e pressione a nova — Esc cancela. \"Apagar linha\"/\"Copiar linha\" pedem a tecla 2x seguidas (como dd/yy do vim de verdade).";

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
    let fields: Vec<(&'static str, String)> = props
        .keymap
        .labeled_fields()
        .into_iter()
        .map(|(label, key)| (label, key.to_string()))
        .collect();

    let on_change = {
        let keymap = props.keymap.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |(label, key): (String, String)| {
            let mut new_keymap = keymap.clone();
            new_keymap.set_by_label(&label, key);
            on_change.emit(new_keymap);
        })
    };

    html! {
        <KeymapCaptureModal
            title={"Atalhos do Vim mode".to_string()}
            hint={HINT.to_string()}
            {fields}
            {on_change}
            on_close={props.on_close.clone()}
        />
    }
}
