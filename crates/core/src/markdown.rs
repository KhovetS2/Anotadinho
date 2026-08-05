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
    if content.starts_with("```") {
        return BlockKind::Code;
    }
    BlockKind::Note
}

fn serialize_block(out: &mut String, block: &Block) {
    let indent = "  ".repeat(block.depth as usize);
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
