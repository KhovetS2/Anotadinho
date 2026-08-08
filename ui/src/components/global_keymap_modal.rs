//! Modal de configuração dos atalhos globais do app (ciclo 105) —
//! fino wrapper sobre `KeymapCaptureModal` (ciclo 104), que sabe
//! traduzir de/para `GlobalKeymap` especificamente.

use yew::prelude::*;

use crate::components::keymap_capture_modal::KeymapCaptureModal;
use crate::state::GlobalKeymap;

const HINT: &str = "Clique numa tecla e pressione a nova — Esc cancela. Todo atalho aqui é combinado com Ctrl (ou Cmd no Mac); deixe em branco pra não ter atalho.";

/// Props do `GlobalKeymapModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct GlobalKeymapModalProps {
    /// Mapa de atalhos atual.
    pub keymap: GlobalKeymap,
    /// Disparado com o mapa atualizado a cada remapeamento.
    pub on_change: Callback<GlobalKeymap>,
    /// Fecha o modal.
    pub on_close: Callback<()>,
}

#[function_component(GlobalKeymapModal)]
pub fn global_keymap_modal(props: &GlobalKeymapModalProps) -> Html {
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
            title={"Atalhos globais".to_string()}
            hint={HINT.to_string()}
            {fields}
            {on_change}
            on_close={props.on_close.clone()}
        />
    }
}
