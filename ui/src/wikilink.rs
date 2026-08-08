//! Wikilinks (`[[Título da Página]]`) — sintaxe, codificação de URL
//! interna e conversão pra/de link markdown normal.
//!
//! Resolvidos por **título**, não por path (mais natural pra digitar,
//! sobrevive a mover a página de pasta). Ambiguidade (dois arquivos com
//! o mesmo título em pastas diferentes) é resolvida pelo chamador —
//! `[[Título]]` vira `[Título](anotadinho://page/Título-codificado)`,
//! um link markdown normal que o `pulldown_cmark` já sabe renderizar
//! sem extensão nenhuma; só o prefixo do scheme marca que é interno.

/// Prefixo que marca um `href` como wikilink interno (em vez de link
/// externo de verdade).
pub const SCHEME_PREFIX: &str = "anotadinho://page/";

/// Codifica um título pra uso na parte de path de uma URL — só ASCII
/// alfanumérico e `-_.~` passam direto, o resto vira `%XX`. Não é um
/// percent-encoding RFC 3986 completo, só o suficiente pra sobreviver
/// dentro de `[texto](aqui)` do markdown (que quebra com espaço/parênteses
/// não escapados) e voltar exatamente como era.
pub fn encode_title(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    for b in title.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

/// Decodifica de volta pro título original.
pub fn decode_title(encoded: &str) -> String {
    let bytes = encoded.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(byte) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                out.push(byte);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8(out).unwrap_or_default()
}

/// Substitui `[[Título]]` por `[Título](anotadinho://page/Título-codificado)`
/// em markdown bruto, preservando blocos de código (` ``` `/`~~~`) intactos
/// — sem isso, um exemplo de sintaxe wikilink dentro de um bloco de código
/// viraria um link de verdade sem querer.
pub fn linkify(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut in_fence = false;
    for line in markdown.split_inclusive('\n') {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            out.push_str(line);
            continue;
        }
        if in_fence {
            out.push_str(line);
            continue;
        }
        out.push_str(&linkify_line(line));
    }
    out
}

fn linkify_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with("[[") {
            if let Some(rel_end) = line[i + 2..].find("]]") {
                let title = &line[i + 2..i + 2 + rel_end];
                if !title.is_empty() && !title.contains('[') && !title.contains(']') {
                    out.push('[');
                    out.push_str(title);
                    out.push_str("](");
                    out.push_str(SCHEME_PREFIX);
                    out.push_str(&encode_title(title));
                    out.push(')');
                    i = i + 2 + rel_end + 2;
                    continue;
                }
            }
        }
        let ch = line[i..].chars().next().unwrap();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_roundtrip_basic() {
        let title = "Projeto Alpha";
        assert_eq!(decode_title(&encode_title(title)), title);
    }

    #[test]
    fn encode_decode_roundtrip_special_chars() {
        let title = "Reunião (2026-08-07) 100%";
        assert_eq!(decode_title(&encode_title(title)), title);
    }

    #[test]
    fn encode_leaves_safe_chars_untouched() {
        assert_eq!(encode_title("abc-123_x.y~z"), "abc-123_x.y~z");
    }

    #[test]
    fn linkify_simple_wikilink() {
        let out = linkify("veja [[Minha Página]] pra detalhes");
        assert_eq!(
            out,
            format!(
                "veja [Minha Página]({}{}) pra detalhes",
                SCHEME_PREFIX,
                encode_title("Minha Página")
            )
        );
    }

    #[test]
    fn linkify_multiple_wikilinks_in_line() {
        let out = linkify("[[A]] e [[B]]");
        assert!(out.contains(&format!("[A]({}A)", SCHEME_PREFIX)));
        assert!(out.contains(&format!("[B]({}B)", SCHEME_PREFIX)));
    }

    #[test]
    fn linkify_skips_fenced_code_blocks() {
        let md = "texto\n```\n[[Não Vira Link]]\n```\ndepois";
        let out = linkify(md);
        assert!(out.contains("[[Não Vira Link]]"));
        assert!(!out.contains(SCHEME_PREFIX));
    }

    #[test]
    fn linkify_ignores_empty_brackets() {
        assert_eq!(linkify("[[]]"), "[[]]");
    }

    #[test]
    fn linkify_leaves_normal_markdown_links_untouched() {
        let md = "[texto](https://example.com)";
        assert_eq!(linkify(md), md);
    }
}
