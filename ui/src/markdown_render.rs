//! Renderizador Markdown → HTML usando pulldown-cmark.

use pulldown_cmark::{html, Options, Parser};

/// Converte Markdown em HTML.
///
/// Separa o frontmatter YAML (se houver) antes de renderizar — sem isso o
/// pulldown-cmark trata `---\ntitle: ...\n---` como texto solto (thematic
/// break + heading mal formado).
///
/// Embeds (`{{ type: "kanban" }}` etc) são recortados ANTES de chegar
/// aqui, por `crate::embed::segment` — o que sobra pra esta função é
/// sempre markdown "de verdade", então fences ` ```kanban ``` ` viram
/// blocos de código normais, sem tratamento especial.
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
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
