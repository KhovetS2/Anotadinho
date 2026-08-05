//! Componente raiz da aplicação.

use yew::prelude::*;

use crate::components::empty_state::EmptyState;

/// Componente raiz.
#[function_component(App)]
pub fn app() -> Html {
    html! {
        <div class="app-root">
            <EmptyState />
        </div>
    }
}
