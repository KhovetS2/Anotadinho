//! Editor WYSIWYG contenteditable + slash commands + markdown live formatting.

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
    SlashItem { label: "Imagem", desc: "URL ou arquivo de imagem", html: "__IMG__" },
    SlashItem { label: "Diagrama", desc: "Mermaid (fluxograma)", html: "__MERMAID__" },
    SlashItem { label: "Assets", desc: "Inserir arquivo do vault", html: "__ASSET__" },
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

    // Effect 1: fetch page content when page changes
    {
        let content_md = content_md.clone();
        let page = props.page.clone();
        let vault_path = props.vault_path.clone();
        let saved_content = saved_content.clone();
        let loading = loading.clone();
        let error = error.clone();
        let edited = edited.clone();

        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let content_md = content_md.clone();
                let saved_content = saved_content.clone();
                let loading = loading.clone();
                let error = error.clone();
                let edited = edited.clone();
                loading.set(true);
                error.set(None);
                edited.set(false);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            content_md.set(text.clone());
                            saved_content.set(text);
                        }
                        Err(e) => { error.set(Some(e)); }
                    }
                    loading.set(false);
                });
            } else {
                content_md.set(String::new());
                saved_content.set(String::new());
                error.set(None);
                edited.set(false);
                loading.set(false);
            }
            || ()
        });
    }

    // Effect 2: sync content_md → contenteditable innerHTML (only if changed)
    {
        let content_md = content_md.clone();
        let loading = loading.clone();
        let editor_ref = editor_ref.clone();
        let last_rendered = use_mut_ref(String::new);

        use_effect_with((content_md.clone(), *loading), move |_| {
            let should_run = !*loading && !content_md.is_empty()
                && *last_rendered.borrow() != *content_md;
            if should_run {
                if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                    let html = crate::markdown_render::render(&content_md);
                    div.set_inner_html(&html);
                    *last_rendered.borrow_mut() = (*content_md).clone();
                    let _div = div.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                        init_mermaid_at(&_div);
                    });
                }
            }
            || {}
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
                exec_cmd(doc, cmd, val);
            }
            edited.set(true);
        }
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

    let select_slash = {
        let slash_open = slash_open.clone();
        let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let exec_fn = doc_exec.clone();
        let items = filtered.clone();
        let vault_path = props.vault_path.clone();
        Callback::from(move |_| {
            if let Some(&item_idx) = items.get(*slash_idx) {
                let item = &SLASH_ITEMS[item_idx];
                let html = match item.html {
                    "__IMG__" => {
                        // Open file dialog, copy to assets
                        let path = gloo_dialogs::prompt(
                            "Caminho da imagem ou URL:\n(ex: /home/user/foto.png ou https://...)",
                            None
                        ).unwrap_or_default();
                        if path.is_empty() { String::new() }
                        else if path.starts_with("http") {
                            format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", path.replace('"', "&quot;"))
                        } else {
                            // Copy to assets
                            let vp = vault_path.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                if let Ok(relative) = crate::api::copy_to_assets(&vp, &path).await {
                                    // Insert the image after the slash closes
                                    let doc = web_sys::window().and_then(|w| w.document());
                                    if let Some(doc) = doc {
                                        let html = format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", relative.replace('"', "&quot;"));
                                        let args = js_sys::Array::new();
                                        args.push(&wasm_bindgen::JsValue::from_str("insertHTML"));
                                        args.push(&wasm_bindgen::JsValue::from_bool(false));
                                        args.push(&wasm_bindgen::JsValue::from_str(&html));
                                        if let Some(f) = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                                            .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                                        { let _ = f.apply(&doc, &args); }
                                    }
                                }
                            });
                            String::new() // Don't insert now, async will handle it
                        }
                    }
                    "__MERMAID__" => {
                        let code = gloo_dialogs::prompt("Código Mermaid:\n(ex: graph TD; A-->B)", None).unwrap_or_default();
                        if code.is_empty() { String::new() }
                        else { format!("<div class=\"mermaid\">{}</div>", code.replace('<', "&lt;").replace('>', "&gt;")) }
                    }
                    "__ASSET__" => {
                        let vp = vault_path.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match crate::api::list_assets(&vp).await {
                                Ok(assets) => {
                                    if assets.is_empty() {
                                        gloo_dialogs::alert("Nenhum arquivo em assets/. Use /img para adicionar imagens.");
                                    } else {
                                        let list = assets.join("\n");
                                        let choice = gloo_dialogs::prompt(
                                            &format!("Assets disponíveis:\n{}\n\nDigite o nome do arquivo:", list),
                                            None
                                        ).unwrap_or_default();
                                        if !choice.is_empty() {
                                            let relative = if choice.starts_with("assets/") { choice } else { format!("assets/{}", choice) };
                                            let doc = web_sys::window().and_then(|w| w.document());
                                            if let Some(doc) = doc {
                                                let ext = std::path::Path::new(&relative).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                                                let html = match ext.as_str() {
                                                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => {
                                                        format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", relative.replace('"', "&quot;"))
                                                    }
                                                    _ => {
                                                        format!("<a href=\"{}\">{}</a>", relative.replace('"', "&quot;"), relative)
                                                    }
                                                };
                                                let args = js_sys::Array::new();
                                                args.push(&wasm_bindgen::JsValue::from_str("insertHTML"));
                                                args.push(&wasm_bindgen::JsValue::from_bool(false));
                                                args.push(&wasm_bindgen::JsValue::from_str(&html));
                                                if let Some(f) = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                                                    .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                                                { let _ = f.apply(&doc, &args); }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    gloo_dialogs::alert(&format!("Erro ao listar assets: {}", e));
                                }
                            }
                        });
                        String::new() // async, don't insert now
                    }
                    other => other.to_string()
                };
                if !html.is_empty() {
                    exec_fn("insertHTML", &html);
                }
            }
            slash_open.set(false);
            slash_text.set(String::new());
            slash_idx.set(0);
            wasm_bindgen_futures::spawn_local(async {
                gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Some(el) = doc.query_selector(".editor__wysiwyg .mermaid").ok().flatten() {
                            if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                                init_mermaid_at(&el);
                            }
                        }
                    }
                }
            });
        })
    };

    let on_keydown = {
        let do_save = do_save.clone();
        let slash_open = slash_open.clone(); let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let filtered_len = filtered.len();
        let select_slash = select_slash.clone();
        let vim_normal = use_state(|| false);
        Callback::from(move |e: KeyboardEvent| {
            if (e.ctrl_key()||e.meta_key()) && e.key()=="s" { e.prevent_default(); do_save.emit(()); return; }

            if *slash_open {
                match e.key().as_str() {
                    "Escape" => { slash_open.set(false); slash_text.set(String::new()); slash_idx.set(0); e.prevent_default(); }
                    "ArrowDown" => { e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + 1) % filtered_len); } }
                    "ArrowUp" => { e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + filtered_len - 1) % filtered_len); } }
                    "Enter" => { e.prevent_default(); select_slash.emit(()); return; }
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

            // Markdown block + inline shortcuts on Space/Enter
            if (e.key() == " " || e.key() == "Enter") && !*slash_open {
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        apply_block_shortcut(&window, &doc, &e);
                        apply_inline_formatting(&window, &doc);
                    }
                }
            }
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

    let on_export = {
        let editor_ref = editor_ref.clone();
        let page_title = page.title.clone();
        Callback::from(move |_| {
            if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                let html = div.inner_html();
                let full = format!("<!DOCTYPE html>\n<html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{max-width:800px;margin:2rem auto;font-family:system-ui;line-height:1.7;color:#1a1a1a;padding:0 1rem}}h1,h2,h3{{margin-top:1.5rem}}pre{{background:#f5f5f5;padding:1rem;border-radius:8px}}code{{background:#f0f0f0;padding:0.2em 0.4em;border-radius:4px}}table{{border-collapse:collapse;width:100%}}td,th{{border:1px solid #ddd;padding:8px}}img{{max-width:100%}}blockquote{{border-left:3px solid #ccc;padding-left:1rem;color:#666}}</style></head><body>{}</body></html>", page_title, html);
                let arr = js_sys::Array::new();
                arr.push(&wasm_bindgen::JsValue::from_str(&full));
                let blob = web_sys::Blob::new_with_str_sequence(&arr).ok();
                if let (Some(blob), Some(window)) = (blob, web_sys::window()) {
                    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                    let _ = window.open_with_url_and_target(&url, "_blank");
                }
            }
        })
    };

    let save_label = if *saving { "Salvando..." } else if *edited { "Salvar *" } else { "Salvar" };
    let save_counter = use_state(|| 0u32);
    let on_edit = {
        let e = edited.clone();
        let do_save = do_save.clone();
        let save_counter = save_counter.clone();
        Callback::from(move |_| {
            e.set(true);
            let do_save = do_save.clone();
            let save_counter = save_counter.clone();
            let id = *save_counter + 1;
            save_counter.set(id);
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                if *save_counter == id {
                    do_save.emit(());
                }
            });
        })
    };

    let on_drop = {
        let vault_path = props.vault_path.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            let dt = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("dataTransfer")).ok();
            let files = dt.and_then(|v| js_sys::Reflect::get(&v, &wasm_bindgen::JsValue::from_str("files")).ok())
                .and_then(|v| v.dyn_into::<web_sys::FileList>().ok());
            if let Some(files) = files {
                let doc = web_sys::window().and_then(|w| w.document());
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let name = file.name();
                        let ext = std::path::Path::new(&name).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                        if !matches!(ext.as_str(), "png"|"jpg"|"jpeg"|"gif"|"svg"|"webp") { continue; }
                        if let Ok(blob) = file.slice() {
                            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                            let html = format!("<img src=\"{}\" alt=\"{}\" style=\"max-width:100%;border-radius:8px;\">", url, name.replace('"', "&quot;"));
                            if let Some(ref doc) = doc {
                                let args = js_sys::Array::new();
                                args.push(&wasm_bindgen::JsValue::from_str("insertHTML"));
                                args.push(&wasm_bindgen::JsValue::from_bool(false));
                                args.push(&wasm_bindgen::JsValue::from_str(&html));
                                if let Some(f) = js_sys::Reflect::get(doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                                    .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                                { let _ = f.apply(doc, &args); }
                            }
                        }
                    }
                }
            }
        })
    };
    let on_dragover = Callback::from(|e: DragEvent| { e.prevent_default(); });

    html! {
        <main class="editor">
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <span class="editor__path">{ &page.path }</span>
                <div class="editor__actions">
                    if let Some(ref s) = *status { <span class="editor__status-badge">{ s }</span> }
                    if *edited { <span class="editor__dirty">{ "não salvo" }</span> }
                    <button class="btn btn--danger btn--sm" onclick={on_delete}>{ "Excluir" }</button>
                    <button class="btn btn--ghost btn--sm" onclick={on_export} title="Exportar HTML">{ "⬇" }</button>
                    <button class="btn btn--primary btn--sm" onclick={do_save.reform(|_| ())} disabled={*saving || !*edited}>{ save_label }</button>
                </div>
            </header>
            <div class="editor__body">
                if *loading {
                    <div class="editor__overlay">{ "Carregando..." }</div>
                }
                if let Some(ref err) = *error {
                    <div class="editor__overlay editor__overlay--error">{ err }</div>
                }
                <div class="editor__wysiwyg" ref={editor_ref} contenteditable="true"
                    spellcheck="false" onkeydown={on_keydown} oninput={on_edit}
                    ondrop={on_drop} ondragover={on_dragover} />
            </div>
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
                <span class="editor__statusbar-hint">{ "Digite / ou use # - > * para formatar" }</span>
            </div>
        </main>
    }
}

fn exec_cmd_global(cmd: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let args = js_sys::Array::new();
        args.push(&wasm_bindgen::JsValue::from_str(cmd));
        args.push(&wasm_bindgen::JsValue::from_bool(false));
        if let Some(f) = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("execCommand"))
            .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        { let _ = f.apply(&doc, &args); }
    }
}

fn exec_cmd(doc: &web_sys::Document, cmd: &str, val: &str) {
    let args = js_sys::Array::new();
    args.push(&wasm_bindgen::JsValue::from_str(cmd));
    args.push(&wasm_bindgen::JsValue::from_bool(false));
    if !val.is_empty() { args.push(&wasm_bindgen::JsValue::from_str(val)); }
    if let Some(f) = js_sys::Reflect::get(doc, &wasm_bindgen::JsValue::from_str("execCommand"))
        .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
    { let _ = f.apply(doc, &args); }
}

fn apply_block_shortcut(win: &web_sys::Window, doc: &web_sys::Document, e: &KeyboardEvent) {
    let sel = match win.get_selection().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    if sel.range_count() == 0 { return; }
    let range = match sel.get_range_at(0) {
        Ok(r) => r,
        Err(_) => return,
    };
    let container = match range.start_container() {
        Ok(c) => c,
        Err(_) => return,
    };
    let offset = range.start_offset().unwrap_or(0) as usize;
    let text = container.text_content().unwrap_or_default();
    let prefix = &text[..offset.min(text.len())];
    let is_newline = e.key() == "Enter";

    if prefix.chars().all(|c| c == '#') && prefix.len() >= 1 && prefix.len() <= 6 {
        let level = prefix.len();
        if let Ok(mut r) = doc.create_range() {
            r.set_start(&container, 0u32).ok();
            r.set_end(&container, prefix.len() as u32).ok();
            sel.remove_all_ranges().ok();
            sel.add_range(&r).ok();
        }
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", &format!("h{}", level));
        e.prevent_default();
        return;
    }

    if !is_newline { return; }

    if prefix == "- " || prefix == "* " {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertUnorderedList", "");
        e.prevent_default();
        return;
    }

    if prefix == "> " {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", "blockquote");
        e.prevent_default();
        return;
    }

    if prefix.len() > 2 && prefix[..prefix.len()-1].chars().all(|c| c.is_ascii_digit() || c == '.') && prefix.ends_with(". ") {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertOrderedList", "");
        e.prevent_default();
    }
}

fn apply_inline_formatting(win: &web_sys::Window, doc: &web_sys::Document) {
    let sel = match win.get_selection().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    if sel.range_count() == 0 { return; }
    let range = match sel.get_range_at(0) {
        Ok(r) => r,
        Err(_) => return,
    };
    let container = match range.start_container() {
        Ok(c) => c,
        Err(_) => return,
    };
    let offset = range.start_offset().unwrap_or(0) as usize;
    let full_text = container.text_content().unwrap_or_default();
    let before = &full_text[..offset.min(full_text.len())];

    // **bold**
    if let Some(start) = before.rfind("**") {
        let between = &before[start + 2..];
        if let Some(inner_end) = between.find("**") {
            let inner = &between[..inner_end];
            let inner_start = start + 2;
            if !inner.is_empty() {
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, inner_start as u32).ok();
                    r.set_end(&container, (inner_start + inner.len()) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "bold", "");
                }
                let close_pos = inner_start + inner.len();
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, close_pos as u32).ok();
                    r.set_end(&container, (close_pos + 2) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "delete", "");
                }
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, start as u32).ok();
                    r.set_end(&container, (start + 2) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "delete", "");
                }
                return;
            }
        }
    }

    // *italic* (single, not part of **)
    if let Some(start) = before.rfind('*') {
        if start == 0 || before.as_bytes()[start - 1] != b'*' {
            let between = &before[start + 1..];
            if let Some(inner_end) = between.find('*') {
                let inner = &between[..inner_end];
                let inner_start = start + 1;
                if !inner.is_empty() {
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, inner_start as u32).ok();
                        r.set_end(&container, (inner_start + inner.len()) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "italic", "");
                    }
                    let close_pos = inner_start + inner.len();
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, close_pos as u32).ok();
                        r.set_end(&container, (close_pos + 1) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "delete", "");
                    }
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, start as u32).ok();
                        r.set_end(&container, (start + 1) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "delete", "");
                    }
                }
            }
        }
    }

    // `code`
    if let Some(start) = before.rfind('`') {
        let between = &before[start + 1..];
        if let Some(inner_end) = between.find('`') {
            let inner = &between[..inner_end];
            let inner_start = start + 1;
            if !inner.is_empty() {
                let code_html = format!("<code>{}</code>", inner.replace('<', "&lt;").replace('>', "&gt;"));
                let full_end = inner_start + inner.len() + 1;
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, start as u32).ok();
                    r.set_end(&container, full_end as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "insertHTML", &code_html);
                }
            }
        }
    }
}

fn init_mermaid_at(el: &web_sys::Element) {
    if let Some(window) = web_sys::window() {
        if let Some(mermaid) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("mermaid"))
            .ok().and_then(|v| v.dyn_into::<js_sys::Object>().ok())
        {
            let config = js_sys::Object::new();
            js_sys::Reflect::set(&config, &wasm_bindgen::JsValue::from_str("theme"), &wasm_bindgen::JsValue::from_str("dark")).ok();
            js_sys::Reflect::set(&config, &wasm_bindgen::JsValue::from_str("startOnLoad"), &wasm_bindgen::JsValue::from_bool(false)).ok();
            js_sys::Reflect::set(&mermaid, &wasm_bindgen::JsValue::from_str("initialize"), &config).ok();
            let run = js_sys::Reflect::get(&mermaid, &wasm_bindgen::JsValue::from_str("run"))
                .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok());
            if let Some(run_fn) = run {
                let opts = js_sys::Object::new();
                js_sys::Reflect::set(&opts, &wasm_bindgen::JsValue::from_str("nodes"), el).ok();
                let _ = run_fn.call1(&wasm_bindgen::JsValue::null(), &opts);
            }
        }
    }
}
