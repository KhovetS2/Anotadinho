//! Editor WYSIWYG contenteditable + toolbar.

use wasm_bindgen::JsCast;
use yew::prelude::*;
use web_sys::KeyboardEvent;

use crate::api::{self, PageMeta};

#[derive(Properties, PartialEq, Clone)]
pub struct EditorProps {
    pub vault_path: String,
    pub page: Option<PageMeta>,
    #[prop_or_default]
    pub on_page_deleted: Callback<()>,
}

#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content_md = use_state(String::new);
    let saved_content = use_state(String::new);
    let loading = use_state(|| false);
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);
    let status = use_state(|| None::<String>);
    let edited = use_state(|| false);
    let editor_ref = use_node_ref();

    let char_count = content_md.chars().count();
    let word_count = content_md.split_whitespace().count();

    {
        let content_md = content_md.clone();
        let page = props.page.clone();
        let vault_path = props.vault_path.clone();
        let saved_content = saved_content.clone();
        let loading = loading.clone();
        let error = error.clone();
        let editor_ref = editor_ref.clone();

        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let content_md = content_md.clone();
                let saved_content = saved_content.clone();
                let loading = loading.clone();
                let error = error.clone();
                let editor_ref = editor_ref.clone();
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            let html = crate::markdown_render::render(&text);
                            if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                                div.set_inner_html(&html);
                            }
                            content_md.set(text.clone());
                            saved_content.set(text);
                        }
                        Err(e) => { error.set(Some(e)); }
                    }
                    loading.set(false);
                });
            } else {
                content_md.set(String::new()); saved_content.set(String::new());
                error.set(None);
                if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                    div.set_inner_html("");
                }
                loading.set(false);
            }
            || ()
        });
    }

    if props.page.is_none() {
        return html! { <main class="app-main"><p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p></main> };
    }

    let page = props.page.as_ref().unwrap().clone();

    let on_input = {
        let edited = edited.clone();
        Callback::from(move |_| { edited.set(true); })
    };

    let exec_cmd = {
        let edited = edited.clone();
        Callback::from(move |(cmd, val): (String, String)| {
            if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                let mut args = js_sys::Array::new();
                args.push(&wasm_bindgen::JsValue::from_str(&cmd));
                args.push(&wasm_bindgen::JsValue::from_bool(false));
                if !val.is_empty() {
                    args.push(&wasm_bindgen::JsValue::from_str(&val));
                }
                if let Some(f) = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                    .ok()
                    .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                {
                    let _ = f.apply(&doc, &args);
                }
            }
            edited.set(true);
        })
    };

    let do_save = {
        let content_md = content_md.clone(); let saved_content = saved_content.clone();
        let saving = saving.clone(); let error = error.clone(); let status = status.clone();
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let editor_ref = editor_ref.clone();
        let edited = edited.clone();
        Callback::from(move |_| {
            if *saving { return; }
            let md = if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                crate::html_to_md::html_to_markdown(&div)
            } else {
                (*content_md).clone()
            };
            let saved_content = saved_content.clone(); let saving = saving.clone();
            let error = error.clone(); let status = status.clone();
            let vault_path = vault_path.clone(); let page_path = page_path.clone();
            let content_md = content_md.clone();
            let edited = edited.clone();
            saving.set(true); error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::write_page(&vault_path, &page_path, &md).await {
                    Ok(()) => {
                        content_md.set(md.clone());
                        saved_content.set(md);
                        edited.set(false);
                        status.set(Some("Salvo".to_string()));
                    }
                    Err(e) => { error.set(Some(e)); }
                }
                saving.set(false);
            });
        })
    };

    let onkeydown = {
        let s = do_save.clone();
        Callback::from(move |e: KeyboardEvent| {
            if (e.ctrl_key()||e.meta_key()) && e.key()=="s" { e.prevent_default(); s.emit(()); }
        })
    };

    let on_delete = {
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let page_title = page.title.clone(); let cb = props.on_page_deleted.clone();
        Callback::from(move |_| {
            if !gloo_dialogs::confirm(&format!("Excluir \"{}\"?", page_title)) { return; }
            let vault_path = vault_path.clone(); let page_path = page_path.clone();
            let cb = cb.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::delete_page(&vault_path, &page_path).await {
                    Ok(()) => { cb.emit(()); }
                    Err(e) => { web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e)); }
                }
            });
        })
    };

    let save_label = if *saving { "Salvando..." } else if *edited { "Salvar *" } else { "Salvar" };

    let toolbar = |label: &str, title: &str, cmd: &str, val: &str| {
        let exec = exec_cmd.clone();
        let cmd_s = cmd.to_string(); let val_s = val.to_string();
        let onclick = Callback::from(move |_| exec.emit((cmd_s.clone(), val_s.clone())));
        html! { <button class="toolbar-btn" {onclick} title={title.to_string()}>{ label }</button> }
    };

    html! {
        <main class="editor" {onkeydown}>
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <span class="editor__path">{ &page.path }</span>
                <div class="editor__actions">
                    if let Some(ref s) = *status { <span class="editor__status-badge">{ s }</span> }
                    if *edited { <span class="editor__dirty">{ "não salvo" }</span> }
                    <button class="editor__delete" onclick={on_delete}>{ "Excluir" }</button>
                    <button class="editor__save" onclick={do_save.reform(|_| ())} disabled={*saving || !*edited}>{ save_label }</button>
                </div>
            </header>
            <div class="editor__toolbar">
                { toolbar("B", "Negrito", "bold", "") }
                { toolbar("I", "Itálico", "italic", "") }
                { toolbar("U", "Sublinhado", "underline", "") }
                { toolbar("S", "Riscado", "strikeThrough", "") }
                <div class="toolbar-divider"></div>
                { toolbar("H1", "Título 2", "formatBlock", "h2") }
                { toolbar("H2", "Título 3", "formatBlock", "h3") }
                { toolbar("¶", "Parágrafo", "formatBlock", "p") }
                <div class="toolbar-divider"></div>
                { toolbar("\u{2014}", "Lista", "insertUnorderedList", "") }
                { toolbar("1.", "Lista ordenada", "insertOrderedList", "") }
                { toolbar("❝", "Citação", "formatBlock", "blockquote") }
                { toolbar("<>", "Código", "formatBlock", "pre") }
                <div class="toolbar-divider"></div>
                { toolbar("\u{1f517}", "Link", "createLink", "https://") }
            </div>
            if *loading {
                <p class="editor__status">{ "Carregando..." }</p>
            } else if let Some(ref err) = *error {
                <p class="editor__error">{ err }</p>
            } else {
                <div class="editor__wysiwyg" ref={editor_ref} contenteditable="true" spellcheck="false" oninput={on_input} />
                <div class="editor__statusbar">
                    <span>{ format!("{} palavras · {} caracteres", word_count, char_count) }</span>
                </div>
            }
        </main>
    }
}
