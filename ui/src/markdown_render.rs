//! Renderizador Markdown → HTML usando pulldown-cmark.

use pulldown_cmark::{html, Options, Parser, Tag, TagEnd, Event, CodeBlockKind};
use pulldown_cmark::CowStr;

/// Converte Markdown em HTML.
///
/// Separa o frontmatter YAML (se houver) antes de renderizar — sem isso o
/// pulldown-cmark trata `---\ntitle: ...\n---` como texto solto (thematic
/// break + heading mal formado).
pub fn render(markdown: &str) -> String {
    let body = anotadinho_core::MarkdownCodec::split_frontmatter(markdown)
        .map(|(_, body)| body)
        .unwrap_or(markdown);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(body, options);
    let mut in_kanban = false;

    let events = parser.map(move |event| {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(ref lang))) => {
                let lang_str = lang.as_ref();
                if lang_str == "kanban" || lang_str == "calendar" || lang_str == "table" {
                    in_kanban = true;
                    Event::Html(CowStr::from(format!("<div class=\"embed-{}\" data-embed=\"{}\">", lang_str, lang_str)))
                } else {
                    let cls = if lang_str.is_empty() { String::new() } else { format!(" class=\"language-{}\"", lang_str) };
                    Event::Html(CowStr::from(format!("<pre><code{}>", cls)))
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if in_kanban {
                    in_kanban = false;
                    Event::Html(CowStr::from("</div>"))
                } else {
                    Event::Html(CowStr::from("</code></pre>"))
                }
            }
            other => other,
        }
    });

    let mut html_output = String::new();
    html::push_html(&mut html_output, events);
    html_output
}
