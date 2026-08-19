//! Extração de alvos de `[[wikilink]]` fora do WASM.
//!
//! A UI tem o parser dela (`ui/src/wikilink.rs`), que trabalha com
//! POSIÇÕES no texto pra linkificar o markdown na hora de renderizar.
//! Aqui o problema é outro: só a LISTA de alvos, pro grafo de backlinks
//! e pra varredura do vault (`crate::index`) — que rodam no backend, um
//! por página, sem DOM nenhum por perto.
//!
//! Blocos de código cercados (```` ``` ````/`~~~`) são ignorados: um
//! `[[exemplo]]` dentro de um trecho de código é texto, não link.

/// Todos os `[[alvo]]` do texto, na ordem em que aparecem, COM
/// duplicatas e sem tratar alias/âncora — o alvo cru, como escrito.
/// Serve pra contagem de referências (o grafo pesa a aresta pelo número
/// de menções).
pub fn extract_wikilink_raw(markdown: &str) -> Vec<String> {
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
        extract_line(line, &mut out);
    }
    out
}

/// Alvos únicos, com alias (`[[Página|texto]]`) e âncora
/// (`[[Página#seção]]`) recortados — o que sobra é o nome da página
/// referenciada, que é o que o grafo e a varredura precisam pra casar
/// com o título de uma página de verdade.
pub fn extract_wikilink_targets(markdown: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    for raw in extract_wikilink_raw(markdown) {
        let target = raw
            .split('|')
            .next()
            .unwrap_or("")
            .split('#')
            .next()
            .unwrap_or("")
            .trim()
            .to_string();
        if !target.is_empty() && !out.contains(&target) {
            out.push(target);
        }
    }
    out
}

/// Varre uma linha atrás de `[[...]]`. Um par com `[` ou `]` no miolo é
/// ignorado (markdown de link normal aninhado, `[[a](b)]`), mesma regra
/// do parser da UI.
fn extract_line(line: &str, out: &mut Vec<String>) {
    let mut i = 0;
    while i < line.len() {
        if line[i..].starts_with("[[") {
            if let Some(rel_end) = line[i + 2..].find("]]") {
                let title = &line[i + 2..i + 2 + rel_end];
                if !title.is_empty() && !title.contains('[') && !title.contains(']') {
                    out.push(title.to_string());
                    i = i + 2 + rel_end + 2;
                    continue;
                }
            }
        }
        let ch = line[i..].chars().next().unwrap_or('\0');
        i += ch.len_utf8();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_encontra_varios_links_na_ordem() {
        assert_eq!(
            extract_wikilink_raw("[[A]] e [[B]]\ntexto [[C]]"),
            vec!["A", "B", "C"]
        );
    }

    #[test]
    fn raw_mantem_duplicatas() {
        assert_eq!(extract_wikilink_raw("[[A]] [[A]]"), vec!["A", "A"]);
    }

    #[test]
    fn raw_ignora_bloco_de_codigo_cercado() {
        let md = "[[Sim]]\n```\n[[Nao]]\n```\n~~~\n[[TambemNao]]\n~~~";
        assert_eq!(extract_wikilink_raw(md), vec!["Sim"]);
    }

    #[test]
    fn targets_removem_duplicata_alias_e_ancora() {
        let md = "[[Roadmap|o mapa]] [[Roadmap#backlog]] [[Missão]]";
        assert_eq!(extract_wikilink_targets(md), vec!["Roadmap", "Missão"]);
    }

    #[test]
    fn targets_vazio_sem_link() {
        assert!(extract_wikilink_targets("nada aqui [ ] [x]").is_empty());
        assert!(extract_wikilink_targets("[[]]").is_empty());
    }

    #[test]
    fn colchete_aninhado_nao_vira_alvo() {
        assert!(extract_wikilink_targets("[[a](b)]").is_empty());
    }
}
