//! Conversão HTML → Markdown para o editor WYSIWYG.

use wasm_bindgen::JsCast;
use web_sys::Element;

/// Converte HTML (innerHTML de contenteditable) para Markdown.
pub fn html_to_markdown(root: &Element) -> String {
    walk(root, 0).trim().to_string()
}

fn walk(node: &Element, _depth: usize) -> String {
    let tag = node.tag_name().to_lowercase();
    let children = node.children();
    let child_count = children.length();

    match tag.as_str() {
        "h1" => format!("# {}\n", text_of(node)),
        "h2" => format!("## {}\n", text_of(node)),
        "h3" => format!("### {}\n", text_of(node)),
        "h4" => format!("#### {}\n", text_of(node)),
        "h5" => format!("##### {}\n", text_of(node)),
        "h6" => format!("###### {}\n", text_of(node)),
        "strong" | "b" => format!("**{}**", text_of(node)),
        "em" | "i" => format!("*{}*", text_of(node)),
        "u" => text_of(node),
        "s" | "del" | "strike" => format!("~~{}~~", text_of(node)),
        "code" => {
            let parent_tag = node.parent_element()
                .map(|p| p.tag_name().to_lowercase())
                .unwrap_or_default();
            if parent_tag == "pre" {
                format!("```\n{}\n```\n", text_of(node))
            } else {
                format!("`{}`", text_of(node))
            }
        }
        "pre" => {
            // Se o <pre> tem um único filho <code class="language-X">, a
            // fence sai com a linguagem preservada (```X). Sem isso, inserir
            // um embed via slash command (ver editor.rs) perderia a
            // linguagem no primeiro save e nunca viraria um embed de verdade.
            let lang = node.query_selector("code[class*=\"language-\"]").ok().flatten()
                .and_then(|code| code.get_attribute("class"))
                .and_then(|class| {
                    class.split_whitespace()
                        .find_map(|c| c.strip_prefix("language-").map(|s| s.to_string()))
                });
            let lang = lang.unwrap_or_default();
            format!("```{}\n{}\n```\n\n", lang, text_of(node))
        }
        "blockquote" => {
            let body = text_of(node);
            body.lines().map(|l| format!("> {}\n", l)).collect::<String>() + "\n"
        }
        "li" => {
            let body = inline_children(node);
            let kind = node.parent_element()
                .map(|p| p.tag_name().to_lowercase())
                .unwrap_or_default();
            let marker = if kind == "ol" { "1." } else { "-" };
            format!("{} {}\n", marker, body)
        }
        "ul" | "ol" => {
            let mut out = String::new();
            for i in 0..child_count {
                if let Some(child) = children.item(i) {
                    if let Ok(el) = child.dyn_into::<Element>() {
                        out.push_str(&walk(&el, 0));
                    }
                }
            }
            out + "\n"
        }
        "a" => {
            let text = text_of(node);
            let href = node.get_attribute("href").unwrap_or_default();
            if href.is_empty() { text } else { format!("[{}]({})", text, href) }
        }
        "p" |         "div" => {
            if node.class_name().contains("mermaid") {
                let text = text_of(node);
                format!("```mermaid\n{}\n```\n\n", text)
            } else if let Some(kind) = node.get_attribute("data-embed-insert") {
                // Marca deixada por um slash command de embed (ver
                // editor.rs) — vira o wrapper `{{ type: "X" }}` direto,
                // no mesmo formato de `EmbedData::to_fence_text()`, sem
                // passar pelos `<br>` como texto solto.
                let body = inline_children(node);
                format!("{{{{ type: \"{kind}\" }}}}\n{body}\n{{{{ /{kind} }}}}\n\n")
            } else {
                let inner = inline_children(node);
                if inner.is_empty() { "\n".to_string() } else { format!("{}\n\n", inner) }
            }
        }
        "br" => "\n".to_string(),
        "hr" => "---\n".to_string(),
        "img" => {
            let alt = node.get_attribute("alt").unwrap_or_default();
            let src = node.get_attribute("src").unwrap_or_default();
            format!("![{}]({})\n", alt, src)
        }
        "input" => "- [ ] ".to_string(),
        _ => {
            if child_count > 0 {
                let mut out = String::new();
                for i in 0..child_count {
                    if let Some(child) = children.item(i) {
                        if let Ok(el) = child.dyn_into::<Element>() {
                            out.push_str(&walk(&el, 0));
                        }
                    }
                }
                out
            } else {
                text_of(node)
            }
        }
    }
}

fn text_of(node: &Element) -> String {
    inline_children(node).trim().to_string()
}

fn inline_children(node: &Element) -> String {
    let children = node.child_nodes();
    let mut out = String::new();
    for i in 0..children.length() {
        if let Some(child) = children.item(i) {
            match child.node_type() {
                3 => out.push_str(&child.text_content().unwrap_or_default()),
                _ => {
                    if let Ok(el) = child.dyn_into::<Element>() {
                        out.push_str(&walk(&el, 0));
                    }
                }
            }
        }
    }
    out
}
