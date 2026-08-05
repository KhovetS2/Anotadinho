//! Componente de estado vazio (mostrado quando nenhum vault está aberto).
//!
//! Ciclo 002 vai substituir isso pelo vault picker (botão + dialog).

use yew::prelude::*;

/// Componente de estado vazio.
#[function_component(EmptyState)]
pub fn empty_state() -> Html {
    html! {
        <div class="empty-state">
            <div class="empty-state__inner">
                <h1 class="empty-state__title">{ "Anotadinho" }</h1>
                <p class="empty-state__message">
                    { "Selecione um vault para começar." }
                </p>
                <button class="empty-state__button" disabled=true>
                    { "Abrir vault (ciclo 002)" }
                </button>
                <p class="empty-state__hint">
                    { "O picker de vault virá no próximo ciclo." }
                </p>
            </div>
        </div>
    }
}
