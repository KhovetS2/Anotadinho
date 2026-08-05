//! Componente de estado vazio.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct EmptyStateProps {
    pub on_vault_selected: Callback<String>,
}

#[function_component(EmptyState)]
pub fn empty_state(props: &EmptyStateProps) -> Html {
    let on_vault_selected = props.on_vault_selected.clone();
    let onclick = Callback::from(move |_| {
        let on_vault_selected = on_vault_selected.clone();
        wasm_bindgen_futures::spawn_local(async move {
            if let Ok(Some(path)) = crate::api::open_folder_dialog().await {
                on_vault_selected.emit(path);
            }
        });
    });

    html! {
        <div class="empty-state">
            <div class="empty-state__inner">
                <h1 class="empty-state__title">{ "Anotadinho" }</h1>
                <p class="empty-state__message">{ "Selecione um vault para começar." }</p>
                <button class="btn btn--primary" {onclick}>{ "Abrir vault" }</button>
            </div>
        </div>
    }
}
