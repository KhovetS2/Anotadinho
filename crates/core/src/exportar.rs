//! Exportar uma página como HTML (ciclo 362), o "Exportar HTML" do menu
//! da página na janela.
//!
//! A janela exporta o que está desenhado: o markdown vira HTML e cada
//! embed entra como a própria cerca num `<pre>` — um kanban não tem
//! forma estática que valha mais que o texto dele. Aqui é o mesmo, a
//! partir do arquivo, com o mesmo estilo de impressão.

use crate::embed::{segment, DocSegment};

/// O estilo da página exportada, o da janela.
const ESTILO: &str = "body{max-width:800px;margin:3rem auto;font-family:system-ui,Inter,sans-serif;line-height:1.8;color:#1a1a1a;padding:0 1.5rem}\
h1,h2,h3{margin-top:2rem;font-weight:600}\
h1{font-size:2rem} h2{font-size:1.5rem} h3{font-size:1.2rem}\
pre{background:#f4f4f4;padding:1.2rem;border-radius:8px;overflow-x:auto}\
code{background:#f0f0f0;padding:0.2em 0.4em;border-radius:4px;font-size:0.9em;font-family:JetBrains Mono,Fira Code,monospace}\
pre code{background:none;padding:0}\
table{border-collapse:collapse;width:100%;margin:1rem 0}\
td,th{border:1px solid #ddd;padding:8px 12px}\
th{background:#f8f8f8;text-align:left}\
img{max-width:100%;height:auto;border-radius:8px}\
blockquote{border-left:4px solid #8B5CF6;padding-left:1rem;color:#666;margin:1rem 0}\
@media print{body{max-width:100%;margin:0;font-size:12pt}}";

fn escapar(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// O HTML completo da página: `corpo` é o markdown sem o frontmatter.
pub fn pagina_em_html(titulo: &str, corpo: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut opcoes = Options::empty();
    opcoes.insert(Options::ENABLE_TABLES);
    opcoes.insert(Options::ENABLE_STRIKETHROUGH);
    opcoes.insert(Options::ENABLE_TASKLISTS);
    let mut miolo = String::new();
    for seg in segment(corpo) {
        match seg {
            DocSegment::Markdown(md) => html::push_html(&mut miolo, Parser::new_ext(&md, opcoes)),
            DocSegment::Embed(dados) => miolo.push_str(&format!("<pre>{}</pre>", escapar(&dados.to_fence_text()))),
        }
    }
    format!(
        "<!DOCTYPE html>\n<html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><title>{}</title><style>{ESTILO}</style></head><body>{miolo}</body></html>",
        escapar(titulo)
    )
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn markdown_vira_html_e_embed_vira_a_cerca_escapada() {
        let html = pagina_em_html("Notas <1>", "# Título\n\n- [ ] tarefa **forte**\n\n{{ type: \"callout\" }}\nvariant: info\ntitle: Nota\nbody: |\n  <b>x</b>\n{{ /callout }}\n");
        assert!(html.starts_with("<!DOCTYPE html>"));
        assert!(html.contains("<title>Notas &lt;1&gt;</title>"));
        assert!(html.contains("<h1>Título</h1>"));
        assert!(html.contains("<strong>forte</strong>"));
        assert!(html.contains("<pre>{{ type: \"callout\" }}"), "{html}");
        assert!(html.contains("&lt;b&gt;x&lt;/b&gt;"), "{html}");
    }
}
