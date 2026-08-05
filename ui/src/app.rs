//! Componente raiz da aplicação.
//!
//! Gerencia o estado global: vault aberto vs tela inicial.
//! Se um vault está aberto, mostra header + sidebar + editor placeholder.

use yew::prelude::*;

use crate::api::PageMeta;
use crate::components::empty_state::EmptyState;
use crate::components::sidebar::Sidebar;
use crate::state;

/// Componente raiz.
#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);

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

    let on_page_selected = {
        let selected_page = selected_page.clone();
        Callback::from(move |page: PageMeta| {
            selected_page.set(Some(page));
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
                        <Sidebar vault_path={path.clone()} on_page_selected={on_page_selected} />
                        <main class="app-main">
                            if let Some(ref page) = *selected_page {
                                <p class="app-main__placeholder">{ format!("Selecionado: {}", page.title) }</p>
                            } else {
                                <p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p>
                            }
                        </main>
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
        </div>
    }
}
