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

/// Extrai os títulos de todos os wikilinks `[[Título]]` num texto
/// markdown, na ordem em que aparecem (com duplicatas, se o mesmo link
/// aparecer mais de uma vez) — pula blocos de código, mesmo critério
/// de `linkify`. Usado pelo grafo de backlinks (ciclo 120), que
/// precisa da lista de TODOS os links de cada página pra montar as
/// arestas, não só substituir `[[..]]` por link markdown.
pub fn extract_titles(markdown: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_fence = false;
    for line in markdown.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") || trimmed.starts_with("~~~") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        extract_titles_line(line, &mut out);
    }
    out
}

fn extract_titles_line(line: &str, out: &mut Vec<String>) {
    let mut em_codigo = false;
    let mut i = 0;
    while i < line.len() {
        // Mesmo critério do `linkify_line`: sem isso o grafo ganhava
        // uma aresta pra uma página "Página" que só existe no exemplo.
        if line.as_bytes()[i] == b'`' {
            em_codigo = !em_codigo;
            i += 1;
            continue;
        }
        if !em_codigo && line[i..].starts_with("[[") {
            if let Some(rel_end) = line[i + 2..].find("]]") {
                let bruto = &line[i + 2..i + 2 + rel_end];
                if !bruto.is_empty() && !bruto.contains('[') && !bruto.contains(']') {
                    // O grafo liga pelo ALVO; o alias é só o que se lê.
                    let (alvo, _) = anotadinho_core::links::split_wikilink(bruto);
                    out.push(alvo);
                    i = i + 2 + rel_end + 2;
                    continue;
                }
            }
        }
        let ch = line[i..].chars().next().unwrap();
        i += ch.len_utf8();
    }
}

fn linkify_line(line: &str) -> String {
    let mut out = String::with_capacity(line.len());
    let mut em_codigo = false;
    let mut i = 0;
    while i < line.len() {
        // Código INLINE também é exemplo de sintaxe, não link (ciclo
        // 191). O fence já era pulado; a crase não, então uma página que
        // EXPLICA a sintaxe escrevendo `` `[[Página]]` `` via o próprio
        // exemplo virar um link de verdade, e o leitor via a URL
        // percent-encoded (`anotadinho://page/P%C3%A1gina`) no lugar do
        // que devia ser mostrado. Mesmo tratamento que
        // `markdown_render::marcar_linha` já fazia pra transclusão.
        if line.as_bytes()[i] == b'`' {
            em_codigo = !em_codigo;
            out.push('`');
            i += 1;
            continue;
        }
        if !em_codigo && line[i..].starts_with("[[") {
            if let Some(rel_end) = line[i + 2..].find("]]") {
                let bruto = &line[i + 2..i + 2 + rel_end];
                if !bruto.is_empty() && !bruto.contains('[') && !bruto.contains(']') {
                    // `[[alvo|texto]]` (ciclo 192): o texto exibido é o
                    // alias; o href leva o ALVO. `\|` no alvo é uma barra
                    // literal — `|` é nome de arquivo válido no POSIX.
                    let (alvo, alias) = anotadinho_core::links::split_wikilink(bruto);
                    let texto = alias.as_deref().unwrap_or(&alvo);
                    out.push('[');
                    out.push_str(texto);
                    out.push_str("](");
                    out.push_str(SCHEME_PREFIX);
                    // O href leva o miolo CRU (alvo + alias + escapes),
                    // não só o alvo: é o que permite ao `html_to_md`
                    // reconstruir o `[[...]]` original ao salvar. Se
                    // levasse só o alvo, o alias se perderia; se levasse
                    // só o texto visível, o alvo é que se perdia — que
                    // era o comportamento antes do ciclo 192.
                    out.push_str(&encode_title(bruto));
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

    // ── alias e barra literal (ciclo 192) ─────────────────────────

    #[test]
    fn alias_vira_o_texto_e_o_href_leva_o_alvo() {
        let saida = linkify("veja [[pages/produto/grafo.md|Grafo do Vault]]\n");
        assert!(saida.contains("[Grafo do Vault]("), "o texto é o alias: {saida}");
        assert!(
            saida.contains("pages%2Fproduto%2Fgrafo.md%7CGrafo%20do%20Vault"),
            "o href leva o miolo cru, pra o save reconstruir igual: {saida}"
        );
    }

    #[test]
    fn sem_alias_o_texto_continua_sendo_o_alvo() {
        let saida = linkify("veja [[Missão]]\n");
        assert!(saida.contains("[Missão](anotadinho://page/Miss%C3%A3o)"), "{saida}");
    }

    #[test]
    fn barra_escapada_nao_vira_alias() {
        // Arquivo `estranho|nome.md`, legal no POSIX.
        let saida = linkify(r"veja [[estranho\|nome]] fim" .to_string().as_str());
        assert!(saida.contains("[estranho|nome]("), "o texto é o nome inteiro: {saida}");
        assert!(saida.contains("estranho%5C%7Cnome"), "o escape é preservado no href: {saida}");
    }

    #[test]
    fn extract_titles_devolve_o_alvo_e_nao_o_alias() {
        assert_eq!(
            extract_titles("[[pages/produto/grafo.md|Grafo do Vault]]\n"),
            vec!["pages/produto/grafo.md".to_string()]
        );
    }

    #[test]
    fn wikilink_em_codigo_inline_nao_vira_link() {
        // Regressão do ciclo 191: a página que EXPLICA a sintaxe
        // mostrava `anotadinho://page/P%C3%A1gina` em vez do exemplo.
        let entrada = "## `[[Página]]` — link\n";
        assert_eq!(linkify(entrada), entrada);
    }

    #[test]
    fn wikilink_fora_do_codigo_na_mesma_linha_ainda_vira_link() {
        let saida = linkify("`[[Nao]]` mas [[Sim]] vale\n");
        assert!(saida.contains("`[[Nao]]`"), "o exemplo em código ficou intacto: {saida}");
        assert!(saida.contains("](anotadinho://page/Sim)"), "o de fora virou link: {saida}");
    }

    #[test]
    fn extract_titles_ignora_codigo_inline() {
        // Senão o grafo ganha uma aresta pra uma página que só existe
        // no exemplo de sintaxe.
        assert_eq!(extract_titles("veja `[[Exemplo]]` e [[Real]]\n"), vec!["Real".to_string()]);
    }

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

    #[test]
    fn extract_titles_finds_multiple_links() {
        let md = "veja [[A]] e também [[B]]\noutra linha com [[C]]";
        assert_eq!(extract_titles(md), vec!["A", "B", "C"]);
    }

    #[test]
    fn extract_titles_skips_fenced_code() {
        let md = "texto\n```\n[[Não]]\n```\n[[Sim]]";
        assert_eq!(extract_titles(md), vec!["Sim"]);
    }

    #[test]
    fn extract_titles_empty_for_no_links() {
        assert!(extract_titles("nada aqui").is_empty());
    }

    #[test]
    fn extract_titles_keeps_duplicates() {
        let md = "[[A]] e de novo [[A]]";
        assert_eq!(extract_titles(md), vec!["A", "A"]);
    }
}
