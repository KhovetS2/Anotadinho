//! Componente de estado vazio (mostrado quando nenhum vault está aberto).
//!
//! Exibe o branding do Anotadinho com um botão "Abrir vault" que
//! invoca o dialog nativo de seleção de pasta.

use yew::prelude::*;

/// Props do EmptyState.
#[derive(Properties, PartialEq, Clone)]
pub struct EmptyStateProps {
    /// Callback chamado quando um vault é selecionado, com o path absoluto.
    pub on_vault_selected: Callback<String>,
}

/// Componente de estado vazio.
#[function_component(EmptyState)]
pub fn empty_state(props: &EmptyStateProps) -> Html {
    let on_vault_selected = props.on_vault_selected.clone();

    let onclick = Callback::from(move |_| {
        let on_vault_selected = on_vault_selected.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match crate::api::open_folder_dialog().await {
                Ok(Some(path)) => {
                    on_vault_selected.emit(path);
                }
                Ok(None) => {}
                Err(e) => {
                    web_sys::console::warn_1(
                        &wasm_bindgen::JsValue::from_str(&format!("dialog error: {}", e)),
                    );
                }
            }
        });
    });

    html! {
        <div class="empty-state">
            <div class="empty-state__inner">
                <h1 class="empty-state__title">{ "Anotadinho" }</h1>
                <p class="empty-state__message">
                    { "Selecione um vault para começar." }
                </p>
                <button class="empty-state__button" {onclick}>
                    { "Abrir vault" }
                </button>
            </div>
        </div>
    }
}
