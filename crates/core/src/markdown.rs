//! Markdown parser/serializer com suporte a block IDs e properties.

use anyhow::{bail, Result};

use crate::block::{Block, BlockId, BlockKind};
use crate::page::{Frontmatter, Page, PageId};
use crate::property::Property;

/// Parser/serializer de Markdown para o modelo de página.
pub struct MarkdownCodec;

impl MarkdownCodec {
    /// Converte texto Markdown em uma `Page`.
    ///
    /// O `page_id` é o path relativo usado como ID da página.
    pub fn parse(text: &str) -> Result<Page> {
        Self::parse_with_id(text, PageId::from_path("untitled.md"))
    }

    /// Parse com PageId explícito.
    pub fn parse_with_id(text: &str, id: PageId) -> Result<Page> {
        let (frontmatter, body) = split_frontmatter(text)?;
        let blocks = parse_blocks(body);
        Ok(Page {
            id,
            frontmatter,
            blocks,
        })
    }

    /// Separa o frontmatter YAML do corpo, sem tocar no texto do corpo
    /// (retorna uma fatia do `text` original — sem reserializar blocos).
    /// Útil quando quem chama só precisa do `type`/`title`/`tags` e quer
    /// renderizar o corpo verbatim (ex: markdown → HTML).
    pub fn split_frontmatter(text: &str) -> Result<(Frontmatter, &str)> {
        split_frontmatter(text)
    }

    /// Separa o texto em `(bloco de frontmatter cru incluindo os "---",
    /// corpo)`, sem parsear o YAML. Diferente de `split_frontmatter`, não
    /// falha em YAML inválido e não perde campos desconhecidos — serve pra
    /// preservar o frontmatter original ao regravar um corpo editado
    /// (ex: editor que só edita o corpo, não os metadados).
    /// Se não houver frontmatter, retorna `("", text)`.
    pub fn split_frontmatter_text(text: &str) -> (&str, &str) {
        split_frontmatter_text(text)
    }

    /// Converte uma `Page` de volta em texto Markdown.
    pub fn serialize(page: &Page) -> Result<String> {
        let mut out = String::new();

        // Frontmatter
        let yaml = serde_yaml::to_string(&page.frontmatter)
            .map_err(|e| anyhow::anyhow!("frontmatter serialize: {}", e))?;
        // serde_yaml adds trailing newline; wrap with ---
        out.push_str("---\n");
        out.push_str(yaml.trim_start_matches("---\n"));
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str("---\n");

        if !page.blocks.is_empty() {
            out.push('\n');
            for block in &page.blocks {
                serialize_block(&mut out, block);
            }
        }

        Ok(out)
    }
}

fn split_frontmatter(text: &str) -> Result<(Frontmatter, &str)> {
    let trimmed = text.trim_start_matches('\u{feff}'); // BOM
    if !trimmed.starts_with("---") {
        return Ok((Frontmatter::default(), trimmed));
    }

    let after_first = &trimmed[3..];
    let after_first = after_first.strip_prefix('\n').unwrap_or(after_first);

    if let Some(end) = after_first.find("\n---") {
        let yaml_str = &after_first[..end];
        let rest = &after_first[end + 4..];
        let rest = rest.strip_prefix('\n').unwrap_or(rest);
        let fm: Frontmatter = if yaml_str.trim().is_empty() {
            Frontmatter::default()
        } else {
            serde_yaml::from_str(yaml_str)
                .map_err(|e| anyhow::anyhow!("frontmatter parse: {}", e))?
        };
        return Ok((fm, rest));
    }

    bail!("frontmatter aberto sem fechamento ---");
}

fn split_frontmatter_text(text: &str) -> (&str, &str) {
    let bom_len = text.len() - text.trim_start_matches('\u{feff}').len();
    let trimmed = &text[bom_len..];
    if !trimmed.starts_with("---") {
        return ("", text);
    }

    let after_first = &trimmed[3..];
    let nl_len = if after_first.starts_with('\n') { 1 } else { 0 };
    let after_first = &after_first[nl_len..];

    if let Some(end) = after_first.find("\n---") {
        let close_end = end + 4; // consome "\n---"
        let rest = &after_first[close_end..];
        let rest_nl_len = if rest.starts_with('\n') { 1 } else { 0 };

        let fm_end = 3 + nl_len + close_end;
        let body_start = fm_end + rest_nl_len;

        return (&trimmed[..fm_end], &trimmed[body_start..]);
    }

    ("", text)
}

fn parse_blocks(body: &str) -> Vec<Block> {
    let mut blocks = Vec::new();
    let mut lines = body.lines().peekable();

    while let Some(line) = lines.next() {
        let stripped = line.trim_end();
        if stripped.is_empty() {
            continue;
        }

        // Conta indentação (espaços; tab = 2)
        let depth = count_depth(line);

        // Remove indent e opcional "- "
        let content_line = {
            let t = line.trim_start();
            t.strip_prefix("- ").or_else(|| t.strip_prefix('-')).unwrap_or(t)
        };

        // Fence de código (```lang ... ```): consome tudo verbatim até o
        // fechamento como um único Block, sem passar pelo scan de
        // properties/continuação (o conteúdo da fence é opaco).
        if let Some(lang) = content_line.strip_prefix("```") {
            let lang = lang.trim();
            let lang = if lang.is_empty() { None } else { Some(lang.to_string()) };
            let mut fence_lines = Vec::new();
            for next in lines.by_ref() {
                if next.trim() == "```" {
                    break;
                }
                fence_lines.push(next.to_string());
            }
            blocks.push(Block {
                id: BlockId::new(),
                content: fence_lines.join("\n"),
                kind: BlockKind::Code(lang),
                properties: Vec::new(),
                depth,
            });
            continue;
        }

        let mut properties = Vec::new();
        let mut content_parts = Vec::new();
        let mut block_id = BlockId::new();

        // Primeira linha pode ser property ou conteúdo
        if let Some(prop) = Property::parse(content_line) {
            if prop.key == "id" {
                if let Some(id) = BlockId::parse(&prop.value) {
                    block_id = id;
                }
            } else {
                properties.push((prop.key, prop.value));
            }
        } else if !content_line.is_empty() {
            content_parts.push(content_line.to_string());
        }

        // Continuação: linhas indentadas sob o bloco (properties / conteúdo)
        // Qualquer nova bullet `-` inicia outro bloco.
        while let Some(next) = lines.peek() {
            let n = *next;
            if n.trim().is_empty() {
                break;
            }
            let next_depth = count_depth(n);
            let next_trim = n.trim_start();

            if next_trim.starts_with('-') {
                break;
            }

            if next_depth > depth {
                lines.next();
                if let Some(prop) = Property::parse(next_trim) {
                    if prop.key == "id" {
                        if let Some(id) = BlockId::parse(&prop.value) {
                            block_id = id;
                        }
                    } else {
                        properties.push((prop.key, prop.value));
                    }
                } else if !next_trim.is_empty() {
                    content_parts.push(next_trim.to_string());
                }
            } else {
                break;
            }
        }

        let content = content_parts.join("\n");
        let kind = infer_kind(&content, &properties);

        blocks.push(Block {
            id: block_id,
            content,
            kind,
            properties,
            depth,
        });
    }

    blocks
}

fn count_depth(line: &str) -> u8 {
    let mut spaces = 0u32;
    for c in line.chars() {
        match c {
            ' ' => spaces += 1,
            '\t' => spaces += 2,
            _ => break,
        }
    }
    (spaces / 2).min(u8::MAX as u32) as u8
}

fn infer_kind(content: &str, properties: &[(String, String)]) -> BlockKind {
    if properties.iter().any(|(k, _)| k == "tipo" || k == "status") {
        if properties.iter().any(|(k, _)| k == "status") {
            return BlockKind::Task;
        }
        return BlockKind::Custom;
    }
    if let Some(rest) = content.strip_prefix('#') {
        let level = 1 + rest.chars().take_while(|c| *c == '#').count();
        return BlockKind::Heading(level.min(6) as u8);
    }
    if content.starts_with("> ") {
        return BlockKind::Quote;
    }
    if let Some(lang) = content.strip_prefix("```") {
        let lang = lang.trim();
        return BlockKind::Code(if lang.is_empty() { None } else { Some(lang.to_string()) });
    }
    BlockKind::Note
}

fn serialize_block(out: &mut String, block: &Block) {
    let indent = "  ".repeat(block.depth as usize);

    // Fences são opacas e não vivem dentro de uma bullet "- id:: ..." —
    // serializa como texto de fence puro pra dar round-trip com o parser.
    if let BlockKind::Code(lang) = &block.kind {
        out.push_str(&indent);
        out.push_str("```");
        if let Some(l) = lang {
            out.push_str(l);
        }
        out.push('\n');
        for line in block.content.lines() {
            out.push_str(&indent);
            out.push_str(line);
            out.push('\n');
        }
        out.push_str(&indent);
        out.push_str("```\n");
        return;
    }

    out.push_str(&indent);
    out.push_str("- ");

    // id:: always first
    out.push_str(&format!("id:: {}\n", block.id));

    for (k, v) in &block.properties {
        out.push_str(&indent);
        out.push_str("  ");
        out.push_str(&format!("{}:: {}\n", k, v));
    }

    if !block.content.is_empty() {
        for (i, line) in block.content.lines().enumerate() {
            out.push_str(&indent);
            if i == 0 && block.properties.is_empty() {
                // content on same visual block; already wrote "- id::\n"
                out.push_str("  ");
            } else {
                out.push_str("  ");
            }
            out.push_str(line);
            out.push('\n');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_frontmatter_text_preserves_raw_yaml() {
        let text = "---\ntitle: Exemplos de Embeds\ntags: [demo, embed]\n---\n\n# corpo\n";
        let (fm, body) = MarkdownCodec::split_frontmatter_text(text);
        assert_eq!(fm, "---\ntitle: Exemplos de Embeds\ntags: [demo, embed]\n---");
        // body preserva a linha em branco original (mesmo comportamento de
        // split_frontmatter, que também só remove UM '\n' do começo).
        assert_eq!(body, "\n# corpo\n");
        // Rejuntar com um "\n" reconstrói o texto original.
        assert_eq!(format!("{fm}\n{body}"), text);
    }

    #[test]
    fn split_frontmatter_text_no_frontmatter() {
        let text = "# só corpo\n";
        let (fm, body) = MarkdownCodec::split_frontmatter_text(text);
        assert_eq!(fm, "");
        assert_eq!(body, text);
    }

    #[test]
    fn parse_fence_with_lang_as_single_block() {
        let text = "# titulo\n\n```kanban\ncolumns: [Backlog, Todo, Done]\nitems:\n  - Tarefa 1 (backlog)\n```\n\ntexto depois\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.blocks.len(), 3);
        assert_eq!(page.blocks[0].content, "# titulo");
        assert_eq!(
            page.blocks[1].kind,
            BlockKind::Code(Some("kanban".to_string()))
        );
        assert_eq!(
            page.blocks[1].content,
            "columns: [Backlog, Todo, Done]\nitems:\n  - Tarefa 1 (backlog)"
        );
        assert_eq!(page.blocks[2].content, "texto depois");
    }

    #[test]
    fn parse_fence_without_lang() {
        let text = "```\nplain code\nmore code\n```\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].kind, BlockKind::Code(None));
        assert_eq!(page.blocks[0].content, "plain code\nmore code");
    }

    #[test]
    fn fence_content_not_mistaken_for_properties() {
        // "columns:" (um só ":") não pode virar Property (precisa de "::").
        let text = "```calendar\n2026-08-06: Revisão de código\n```\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.blocks.len(), 1);
        assert!(page.blocks[0].properties.is_empty());
        assert_eq!(page.blocks[0].content, "2026-08-06: Revisão de código");
    }

    #[test]
    fn serialize_roundtrip_fence_block() {
        let text = "```table\n| a | b |\n| - | - |\n| 1 | 2 |\n```\n";
        let page = MarkdownCodec::parse(text).unwrap();
        let out = MarkdownCodec::serialize(&page).unwrap();
        let page2 = MarkdownCodec::parse(&out).unwrap();
        assert_eq!(page2.blocks.len(), 1);
        assert_eq!(page2.blocks[0].kind, page.blocks[0].kind);
        assert_eq!(page2.blocks[0].content, page.blocks[0].content);
    }

    #[test]
    fn parse_plain_without_frontmatter() {
        let page = MarkdownCodec::parse("- hello world\n").unwrap();
        assert!(page.frontmatter.title.is_none());
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].content, "hello world");
    }

    #[test]
    fn parse_frontmatter_title_and_tags() {
        let text = "---\ntitle: Minha Nota\ntags:\n  - a\n  - b\n---\n\n- corpo\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.frontmatter.title.as_deref(), Some("Minha Nota"));
        assert_eq!(page.frontmatter.tags, vec!["a", "b"]);
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].content, "corpo");
    }

    #[test]
    fn parse_frontmatter_bare_date_created() {
        // Regressão: páginas reais do vault usam "created: YYYY-MM-DD" (sem
        // horário), que não é um DateTime RFC3339 válido. Se o parse falhar
        // aqui, o fallback do markdown_render acaba renderizando a página
        // inteira (frontmatter incluso) como texto solto.
        let text = "---\ntitle: Sobre o Anotadinho\ntags: [projeto, docs]\ncreated: 2026-08-04\n---\n\n# Sobre\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.frontmatter.title.as_deref(), Some("Sobre o Anotadinho"));
        assert_eq!(page.frontmatter.tags, vec!["projeto", "docs"]);
        assert_eq!(page.frontmatter.created.as_deref(), Some("2026-08-04"));
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].content, "# Sobre");
    }

    #[test]
    fn parse_block_with_id_and_property() {
        let id = uuid::Uuid::new_v4();
        let text = format!(
            "---\ntitle: T\n---\n\n- id:: {}\n  status:: done\n  conteudo aqui\n",
            id
        );
        let page = MarkdownCodec::parse(&text).unwrap();
        assert_eq!(page.blocks.len(), 1);
        assert_eq!(page.blocks[0].id.0, id);
        assert_eq!(
            page.blocks[0].properties,
            vec![("status".into(), "done".into())]
        );
        assert_eq!(page.blocks[0].content, "conteudo aqui");
        assert_eq!(page.blocks[0].kind, BlockKind::Task);
    }

    #[test]
    fn parse_nested_depth() {
        let text = "- root\n  - child\n";
        let page = MarkdownCodec::parse(text).unwrap();
        assert_eq!(page.blocks.len(), 2);
        assert_eq!(page.blocks[0].depth, 0);
        assert_eq!(page.blocks[0].content, "root");
        assert_eq!(page.blocks[1].depth, 1);
        assert_eq!(page.blocks[1].content, "child");
    }

    #[test]
    fn serialize_roundtrip_frontmatter() {
        let text = "---\ntitle: Round\ntags:\n  - x\n---\n\n- hello\n";
        let page = MarkdownCodec::parse(text).unwrap();
        let out = MarkdownCodec::serialize(&page).unwrap();
        let page2 = MarkdownCodec::parse(&out).unwrap();
        assert_eq!(page2.frontmatter.title, page.frontmatter.title);
        assert_eq!(page2.frontmatter.tags, page.frontmatter.tags);
        assert_eq!(page2.blocks.len(), 1);
        assert_eq!(page2.blocks[0].content, "hello");
    }

    #[test]
    fn serialize_preserves_block_id() {
        let id = uuid::Uuid::new_v4();
        let text = format!("- id:: {}\n  body\n", id);
        let page = MarkdownCodec::parse(&text).unwrap();
        let out = MarkdownCodec::serialize(&page).unwrap();
        let page2 = MarkdownCodec::parse(&out).unwrap();
        assert_eq!(page2.blocks[0].id.0, id);
        assert_eq!(page2.blocks[0].content, "body");
    }
}
