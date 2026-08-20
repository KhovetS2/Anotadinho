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
///
/// Transclusão (`![[Página]]`, ciclo 170) vira um marcador vazio aqui e
/// é preenchida depois, no DOM — esta função é síncrona e não alcança o
/// vault.

/// Embrulha o `^id` do fim das linhas num `<span>` discreto.
fn marcar_ids_de_bloco(body: &str) -> String {
    body.split_inclusive('\n')
        .map(|linha| {
            let sem_quebra = linha.trim_end_matches('\n');
            if let Some(id) = anotadinho_core::links::extract_block_id(sem_quebra) {
                let limpo = anotadinho_core::links::strip_block_id(sem_quebra);
                let marcado = format!("{limpo} <span class=\"bloco-id\">^{id}</span>");
                if linha.ends_with('\n') {
                    format!("{marcado}\n")
                } else {
                    marcado
                }
            } else {
                linha.to_string()
            }
        })
        .collect()
}

/// Troca `![[Alvo]]` por um marcador de bloco que o DOM resolve
/// depois. Fora de fence de código.
fn marcar_transclusoes(body: &str) -> String {
    let mut out = String::with_capacity(body.len());
    let mut in_fence = false;
    for linha in body.split_inclusive('\n') {
        let t = linha.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(linha);
            continue;
        }
        if in_fence || !linha.contains("![[") {
            out.push_str(linha);
            continue;
        }
        let mut resto = linha;
        while let Some(pos) = resto.find("![[") {
            let (antes, depois) = resto.split_at(pos);
            out.push_str(antes);
            let miolo = &depois[3..];
            let Some(fim) = miolo.find("]]") else {
                out.push_str(depois);
                resto = "";
                break;
            };
            let alvo = miolo[..fim].trim();
            if alvo.is_empty() || alvo.contains('[') {
                out.push_str(&depois[..fim + 5]);
            } else {
                // HTML cru no meio do markdown: o pulldown-cmark passa
                // adiante, e o marcador chega inteiro no DOM.
                out.push_str(&format!(
                    "\n<div class=\"transclusao\" data-transclusao=\"{}\"></div>\n",
                    alvo.replace('"', "&quot;")
                ));
            }
            resto = &miolo[fim + 2..];
        }
        out.push_str(resto);
    }
    out
}

/// Converte Markdown em HTML (ver doc do módulo acima).
pub fn render(markdown: &str) -> String {
    let body = anotadinho_core::MarkdownCodec::split_frontmatter(markdown)
        .map(|(_, body)| body)
        .unwrap_or(markdown);
    // Transclusão (ciclo 170): vira um marcador ANTES do linkify, senão
    // o `[[..]]` de dentro viraria link e o `!` ficaria solto. O
    // conteúdo real é carregado depois, no DOM, por
    // `upgrade_transclusions_at` — daqui não dá pra ler o vault.
    let body = marcar_transclusoes(body);
    // Id de bloco (ciclo 176): fica no DOM, mas embrulhado num `<span>`
    // que o CSS deixa discreto. Não dá pra ESCONDER de vez: o markdown
    // é recomposto a partir do DOM na hora de salvar, então o que sai
    // da renderização some do arquivo na próxima gravação.
    let body = marcar_ids_de_bloco(&body);
    let body = crate::wikilink::linkify(&body);

    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_FOOTNOTES);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TASKLISTS);

    let parser = Parser::new_ext(&body, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}
