//! Componente raiz da aplicação.
//!
//! Gerencia o estado global: vault aberto vs tela inicial.
//! Se um vault está aberto, mostra header com path + sidebar.
//! Se não, mostra EmptyState com botão de seleção.

use yew::prelude::*;

use crate::components::empty_state::EmptyState;
use crate::state;

/// Componente raiz.
#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());

    let on_vault_selected = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        Callback::from(move |path: String| {
            let name = state::extract_name_from_path(&path);
            state::save_vault_path(&path);
            state::save_vault_name(&name);
            vault_path.set(Some(path));
            vault_name.set(Some(name));
        })
    };

    html! {
        <div class="app-root">
            if let (Some(path), Some(name)) = ((*vault_path).clone(), (*vault_name).clone()) {
                <div class="app-layout">
                    <header class="app-header">
                        <h2 class="app-header__title">{ &name }</h2>
                        <span class="app-header__path">{ &path }</span>
                    </header>
                    <div class="app-body">
                        <aside class="app-sidebar">
                            <p class="app-sidebar__hint">{ "Vault aberto. Sidebar virá no ciclo 003." }</p>
                        </aside>
                        <main class="app-main">
                            <p class="app-main__placeholder">{ "Editor virá nos próximos ciclos." }</p>
                        </main>
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
        </div>
    }
}
