//! Componente raiz da aplicação.
//!
//! Gerencia o estado global: vault aberto vs tela inicial.
//! Se um vault está aberto, mostra header + sidebar + editor.

use yew::prelude::*;

use crate::api::PageMeta;
use crate::components::editor::Editor;
use crate::components::empty_state::EmptyState;
use crate::components::sidebar::Sidebar;
use crate::state;

/// Componente raiz.
#[function_component(App)]
pub fn app() -> Html {
    let vault_path = use_state(|| state::load_vault_path());
    let vault_name = use_state(|| state::load_vault_name());
    let selected_page = use_state(|| None::<PageMeta>);
    let list_version = use_state(|| 0u32);

    let on_vault_selected = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        Callback::from(move |path: String| {
            let name = state::extract_name_from_path(&path);
            state::save_vault_path(&path);
            state::save_vault_name(&name);
            vault_path.set(Some(path));
            vault_name.set(Some(name));
            selected_page.set(None);
        })
    };

    let on_page_selected = {
        let selected_page = selected_page.clone();
        Callback::from(move |page: PageMeta| {
            selected_page.set(Some(page));
        })
    };

    let on_close_vault = {
        let vault_path = vault_path.clone();
        let vault_name = vault_name.clone();
        let selected_page = selected_page.clone();
        Callback::from(move |_| {
            state::clear_vault();
            vault_path.set(None);
            vault_name.set(None);
            selected_page.set(None);
        })
    };

    let on_page_deleted = {
        let selected_page = selected_page.clone();
        let list_version = list_version.clone();
        Callback::from(move |_| {
            selected_page.set(None);
            list_version.set(*list_version + 1);
        })
    };

    html! {
        <div class="app-root">
            if let (Some(path), Some(name)) = ((*vault_path).clone(), (*vault_name).clone()) {
                <div class="app-layout">
                    <header class="app-header">
                        <h2 class="app-header__title">{ &name }</h2>
                        <span class="app-header__path">{ &path }</span>
                        <button class="app-header__close" onclick={on_close_vault}>
                            { "Fechar" }
                        </button>
                    </header>
                    <div class="app-body">
                        <Sidebar
                            vault_path={path.clone()}
                            on_page_selected={on_page_selected}
                            list_version={*list_version}
                        />
                        <Editor
                            vault_path={path.clone()}
                            page={(*selected_page).clone()}
                            on_page_deleted={on_page_deleted}
                        />
                    </div>
                </div>
            } else {
                <EmptyState on_vault_selected={on_vault_selected} />
            }
        </div>
    }
}
