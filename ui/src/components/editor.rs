//! Editor WYSIWYG contenteditable + toolbar + slash commands.

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

struct SlashItem {
    label: &'static str,
    desc: &'static str,
    html: &'static str,
}

static SLASH_ITEMS: &[SlashItem] = &[
    SlashItem { label: "Título 1", desc: "Título grande", html: "<h1>Título</h1>" },
    SlashItem { label: "Título 2", desc: "Título médio", html: "<h2>Título</h2>" },
    SlashItem { label: "Título 3", desc: "Título pequeno", html: "<h3>Título</h3>" },
    SlashItem { label: "Lista", desc: "Lista com marcadores", html: "<ul><li>Item</li></ul>" },
    SlashItem { label: "Checklist", desc: "Lista de tarefas", html: "<div><input type='checkbox'> Tarefa</div>" },
    SlashItem { label: "Citação", desc: "Bloco de citação", html: "<blockquote>Citação</blockquote>" },
    SlashItem { label: "Código", desc: "Bloco de código", html: "<pre><code>código</code></pre>" },
    SlashItem { label: "Tabela", desc: "Tabela 3×2", html: "<table><tr><td>A</td><td>B</td><td>C</td></tr><tr><td></td><td></td><td></td></tr></table>" },
    SlashItem { label: "Linha", desc: "Divisor horizontal", html: "<hr>" },
];

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

    let slash_open = use_state(|| false);
    let slash_text = use_state(String::new);
    let slash_idx = use_state(|| 0usize);

    let filtered: Vec<usize> = SLASH_ITEMS.iter().enumerate()
        .filter(|(_, item)| {
            let q = slash_text.to_lowercase();
            q.is_empty() || item.label.to_lowercase().contains(&q) || item.desc.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();

    let char_count = content_md.chars().count();
    let word_count = content_md.split_whitespace().count();

    {
        let content_md = content_md.clone(); let page = props.page.clone();
        let vault_path = props.vault_path.clone(); let saved_content = saved_content.clone();
        let loading = loading.clone(); let error = error.clone();
        let editor_ref = editor_ref.clone();

        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page {
                let vault_path = vault_path.clone(); let path = p.path.clone();
                let content_md = content_md.clone(); let saved_content = saved_content.clone();
                let loading = loading.clone(); let error = error.clone();
                let editor_ref = editor_ref.clone();
                loading.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            let html = crate::markdown_render::render(&text);
                            if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                                div.set_inner_html(&html);
                            }
                            content_md.set(text.clone()); saved_content.set(text);
                        }
                        Err(e) => { error.set(Some(e)); }
                    }
                    loading.set(false);
                });
            } else {
                content_md.set(String::new()); saved_content.set(String::new()); error.set(None);
                if let Some(div) = editor_ref.cast::<web_sys::Element>() { div.set_inner_html(""); }
                loading.set(false);
            }
            || ()
        });
    }

    if props.page.is_none() {
        return html! { <main class="app-main"><p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p></main> };
    }

    let page = props.page.as_ref().unwrap().clone();

    let doc_exec = {
        let edited = edited.clone();
        let doc = web_sys::window().and_then(|w| w.document());
        move |cmd: &str, val: &str| {
            if let Some(ref doc) = doc {
                let args = js_sys::Array::new();
                args.push(&wasm_bindgen::JsValue::from_str(cmd));
                args.push(&wasm_bindgen::JsValue::from_bool(false));
                if !val.is_empty() { args.push(&wasm_bindgen::JsValue::from_str(val)); }
                if let Some(f) = js_sys::Reflect::get(doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                    .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                { let _ = f.apply(doc, &args); }
            }
            edited.set(true);
        }
    };

    let toolbar_cb = {
        let exec = doc_exec.clone();
        Callback::from(move |(cmd, val): (String, String)| exec(&cmd, &val))
    };

    let do_save = {
        let content_md = content_md.clone(); let saved_content = saved_content.clone();
        let saving = saving.clone(); let error = error.clone(); let status = status.clone();
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let editor_ref = editor_ref.clone(); let edited = edited.clone();
        Callback::from(move |_| {
            if *saving { return; }
            let md = if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                crate::html_to_md::html_to_markdown(&div)
            } else { (*content_md).clone() };
            let saved_content = saved_content.clone(); let saving = saving.clone();
            let error = error.clone(); let status = status.clone();
            let vault_path = vault_path.clone(); let page_path = page_path.clone();
            let content_md = content_md.clone(); let edited = edited.clone();
            saving.set(true); error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::write_page(&vault_path, &page_path, &md).await {
                    Ok(()) => { content_md.set(md.clone()); saved_content.set(md); edited.set(false); status.set(Some("Salvo".to_string())); }
                    Err(e) => { error.set(Some(e)); }
                }
                saving.set(false);
            });
        })
    };

    let on_keydown = {
        let do_save = do_save.clone();
        let slash_open = slash_open.clone(); let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let filtered_len = filtered.len();
        Callback::from(move |e: KeyboardEvent| {
            if (e.ctrl_key()||e.meta_key()) && e.key()=="s" { e.prevent_default(); do_save.emit(()); return; }

            if *slash_open {
                match e.key().as_str() {
                    "Escape" => { slash_open.set(false); slash_text.set(String::new()); slash_idx.set(0); e.prevent_default(); }
                    "ArrowDown" => { e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + 1) % filtered_len); } }
                    "ArrowUp" => { e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + filtered_len - 1) % filtered_len); } }
                    "Enter" => { e.prevent_default(); return; } // handled by select_slash
                    "Backspace" => {
                        e.prevent_default();
                        if !slash_text.is_empty() { slash_text.set(slash_text[..slash_text.len()-1].to_string()); }
                        else { slash_open.set(false); slash_idx.set(0); }
                    }
                    _ if e.key().len() == 1 => { slash_text.set(format!("{}{}", *slash_text, e.key())); slash_idx.set(0); e.prevent_default(); }
                    _ => {}
                }
                return;
            }

            if e.key() == "/" && !e.ctrl_key() && !e.meta_key() {
                slash_open.set(true); slash_text.set(String::new()); slash_idx.set(0);
                e.prevent_default();
            }
        })
    };

    let select_slash = {
        let slash_open = slash_open.clone();
        let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let exec_fn = doc_exec.clone();
        let items = filtered.clone();
        Callback::from(move |_| {
            if let Some(&item_idx) = items.get(*slash_idx) {
                let item = &SLASH_ITEMS[item_idx];
                exec_fn("insertHTML", item.html);
            }
            slash_open.set(false);
            slash_text.set(String::new());
            slash_idx.set(0);
        })
    };

    let on_delete = {
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let page_title = page.title.clone(); let cb = props.on_page_deleted.clone();
        Callback::from(move |_| {
            if !gloo_dialogs::confirm(&format!("Excluir \"{}\"?", page_title)) { return; }
            let vault_path = vault_path.clone(); let page_path = page_path.clone(); let cb = cb.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api::delete_page(&vault_path, &page_path).await {
                    web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                } else { cb.emit(()); }
            });
        })
    };

    let save_label = if *saving { "Salvando..." } else if *edited { "Salvar *" } else { "Salvar" };

    let toolbar = |label: &str, title: &str, cmd: &str, val: &str| {
        let cb = toolbar_cb.clone();
        let c = cmd.to_string(); let v = val.to_string();
        html! { <button class="toolbar-btn" onclick={Callback::from(move |_| cb.emit((c.clone(), v.clone())))} title={title.to_string()}>{ label }</button> }
    };

    let on_edit = { let e = edited.clone(); Callback::from(move |_| e.set(true)) };

    html! {
        <main class="editor">
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
                <div class="editor__wysiwyg" ref={editor_ref} contenteditable="true"
                    spellcheck="false" onkeydown={on_keydown} oninput={on_edit} />
                if *slash_open {
                    <div class="slash-menu">
                        <div class="slash-menu__header">
                            <span>{ "/" }{ &*slash_text }</span>
                            <span class="slash-menu__hint">{ format!("{} resultados", filtered.len()) }</span>
                        </div>
                        <div class="slash-menu__list">
                            { for filtered.iter().enumerate().map(|(vi, &item_idx)| {
                                let item = &SLASH_ITEMS[item_idx];
                                let class = if vi == *slash_idx { "slash-menu__item slash-menu__item--active" } else { "slash-menu__item" };
                                let sel = select_slash.clone();
                                html! {
                                    <div {class} onclick={Callback::from(move |_| sel.emit(()))}>
                                        <span class="slash-menu__item-label">{ item.label }</span>
                                        <span class="slash-menu__item-desc">{ item.desc }</span>
                                    </div>
                                }
                            }) }
                        </div>
                    </div>
                }
                <div class="editor__statusbar">
                    <span>{ format!("{} palavras · {} caracteres", word_count, char_count) }</span>
                    <span class="editor__statusbar-hint">{ "Digite / para comandos" }</span>
                </div>
            }
        </main>
    }
}
