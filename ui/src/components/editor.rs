//! Editor Markdown: carrega, edita e salva o conteúdo bruto da página.

use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Props do Editor.
#[derive(Properties, PartialEq, Clone)]
pub struct EditorProps {
    /// Path absoluto do vault.
    pub vault_path: String,
    /// Página selecionada (None = placeholder).
    pub page: Option<PageMeta>,
    /// Callback após exclusão bem-sucedida.
    #[prop_or_default]
    pub on_page_deleted: Callback<()>,
}

/// Componente Editor.
#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content = use_state(String::new);
    let saved_content = use_state(String::new);
    let loading = use_state(|| false);
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);
    let status = use_state(|| None::<String>);

    {
        let content = content.clone();
        let saved_content = saved_content.clone();
        let loading = loading.clone();
        let error = error.clone();
        let status = status.clone();
        let vault_path = props.vault_path.clone();
        let page = props.page.clone();

        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let content = content.clone();
                let saved_content = saved_content.clone();
                let loading = loading.clone();
                let error = error.clone();
                let status = status.clone();
                loading.set(true);
                error.set(None);
                status.set(None);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            content.set(text.clone());
                            saved_content.set(text);
                            error.set(None);
                        }
                        Err(e) => {
                            content.set(String::new());
                            saved_content.set(String::new());
                            error.set(Some(e));
                        }
                    }
                    loading.set(false);
                });
            } else {
                content.set(String::new());
                saved_content.set(String::new());
                error.set(None);
                status.set(None);
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

    let page = props.page.as_ref().unwrap().clone();
    let dirty = *content != *saved_content;

    let oninput = {
        let content = content.clone();
        let status = status.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e
                .target()
                .and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok())
            {
                content.set(input.value());
                status.set(None);
            }
        })
    };

    let do_save = {
        let content = content.clone();
        let saved_content = saved_content.clone();
        let saving = saving.clone();
        let error = error.clone();
        let status = status.clone();
        let vault_path = props.vault_path.clone();
        let page_path = page.path.clone();
        Callback::from(move |_| {
            if *saving {
                return;
            }
            let content_val = (*content).clone();
            let vault_path = vault_path.clone();
            let page_path = page_path.clone();
            let content = content.clone();
            let saved_content = saved_content.clone();
            let saving = saving.clone();
            let error = error.clone();
            let status = status.clone();
            saving.set(true);
            error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::write_page(&vault_path, &page_path, &content_val).await {
                    Ok(()) => {
                        saved_content.set(content_val);
                        status.set(Some("Salvo".to_string()));
                    }
                    Err(e) => {
                        error.set(Some(e));
                        let _ = content;
                    }
                }
                saving.set(false);
            });
        })
    };

    let onkeydown = {
        let do_save = do_save.clone();
        Callback::from(move |e: KeyboardEvent| {
            if (e.ctrl_key() || e.meta_key()) && e.key() == "s" {
                e.prevent_default();
                do_save.emit(());
            }
        })
    };

    let on_delete = {
        let vault_path = props.vault_path.clone();
        let page_path = page.path.clone();
        let page_title = page.title.clone();
        let on_page_deleted = props.on_page_deleted.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let msg = format!("Excluir a página \"{}\"? Esta ação não pode ser desfeita.", page_title);
            if !gloo_dialogs::confirm(&msg) {
                return;
            }
            let vault_path = vault_path.clone();
            let page_path = page_path.clone();
            let on_page_deleted = on_page_deleted.clone();
            let error = error.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::delete_page(&vault_path, &page_path).await {
                    Ok(()) => on_page_deleted.emit(()),
                    Err(e) => error.set(Some(e)),
                }
            });
        })
    };

    let save_label = if *saving {
        "Salvando..."
    } else if dirty {
        "Salvar *"
    } else {
        "Salvar"
    };

    html! {
        <main class="editor" {onkeydown}>
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <span class="editor__path">{ &page.path }</span>
                <div class="editor__actions">
                    if let Some(ref s) = *status {
                        <span class="editor__status-badge">{ s }</span>
                    }
                    if dirty {
                        <span class="editor__dirty">{ "não salvo" }</span>
                    }
                    <button
                        class="editor__delete"
                        onclick={on_delete}
                        title="Excluir página"
                    >
                        { "Excluir" }
                    </button>
                    <button
                        class="editor__save"
                        onclick={do_save.reform(|_| ())}
                        disabled={*saving || !dirty}
                    >
                        { save_label }
                    </button>
                </div>
            </header>
            if *loading {
                <p class="editor__status">{ "Carregando..." }</p>
            } else if let Some(ref err) = *error {
                <p class="editor__error">{ err }</p>
            } else {
                <textarea
                    class="editor__textarea"
                    value={(*content).clone()}
                    {oninput}
                    spellcheck="false"
                />
            }
        </main>
    }
}
