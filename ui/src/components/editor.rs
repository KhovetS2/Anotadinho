//! Editor Markdown básico: carrega e exibe o conteúdo bruto da página.

use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Props do Editor.
#[derive(Properties, PartialEq, Clone)]
pub struct EditorProps {
    /// Path absoluto do vault.
    pub vault_path: String,
    /// Página selecionada (None = placeholder).
    pub page: Option<PageMeta>,
}

/// Componente Editor.
#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content = use_state(String::new);
    let loading = use_state(|| false);
    let error = use_state(|| None::<String>);

    {
        let content = content.clone();
        let loading = loading.clone();
        let error = error.clone();
        let vault_path = props.vault_path.clone();
        let page = props.page.clone();

        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let content = content.clone();
                let loading = loading.clone();
                let error = error.clone();
                loading.set(true);
                error.set(None);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            content.set(text);
                            error.set(None);
                        }
                        Err(e) => {
                            content.set(String::new());
                            error.set(Some(e));
                        }
                    }
                    loading.set(false);
                });
            } else {
                content.set(String::new());
                error.set(None);
                loading.set(false);
            }
            || ()
        });
    }

    if props.page.is_none() {
        return html! {
            <main class="app-main">
                <p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p>
            </main>
        };
    }

    let page = props.page.as_ref().unwrap();

    html! {
        <main class="editor">
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <span class="editor__path">{ &page.path }</span>
            </header>
            if *loading {
                <p class="editor__status">{ "Carregando..." }</p>
            } else if let Some(ref err) = *error {
                <p class="editor__error">{ err }</p>
            } else {
                <textarea
                    class="editor__textarea"
                    value={(*content).clone()}
                    readonly=true
                    spellcheck="false"
                />
            }
        </main>
    }
}
