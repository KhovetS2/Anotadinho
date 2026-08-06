//! Base extensível para embeds inline: blocos ```kanban/calendar/table
//! dentro de uma página comum, parseados em dados estruturados e
//! renderizados como componentes Yew de verdade (não texto cru).
//!
//! Extensão: um novo tipo de embed é 1 variante em `EmbedKind` + 1 par
//! parse/serialize em `EmbedData` + 1 componente Yew (`components/embeds/`)
//! + 1 braço de `match` no editor. Nada mais precisa mudar.

use pulldown_cmark::{CodeBlockKind, Event, Options, Parser, Tag, TagEnd};
use serde::Deserialize;

/// Tipos de embed inline reconhecidos pela linguagem da fence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedKind {
    /// ```kanban — board com colunas e cards.
    Kanban,
    /// ```calendar — lista de eventos por data.
    Calendar,
    /// ```table — tabela markdown comum.
    Table,
}

impl EmbedKind {
    /// Reconhece a linguagem de uma fence (`kanban`/`calendar`/`table`).
    pub fn from_lang_tag(tag: &str) -> Option<Self> {
        match tag {
            "kanban" => Some(Self::Kanban),
            "calendar" => Some(Self::Calendar),
            "table" => Some(Self::Table),
            _ => None,
        }
    }
}

/// Um trecho do corpo de uma página: markdown comum ou um embed já parseado.
#[derive(Debug, Clone, PartialEq)]
pub enum DocSegment {
    /// Fatia intocada do texto original (renderiza pelo caminho markdown atual).
    Markdown(String),
    /// Um embed reconhecido, já parseado em dados estruturados.
    Embed(EmbedData),
}

/// Segmenta o corpo (já sem frontmatter) em uma sequência ordenada de
/// trechos de markdown comum e embeds, usando os offsets de byte do
/// próprio pulldown-cmark — não perde nem reformata o texto original fora
/// das fences reconhecidas.
pub fn segment(body: &str) -> Vec<DocSegment> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    let parser = Parser::new_ext(body, options);

    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let mut open: Option<(EmbedKind, usize)> = None;

    for (event, range) in parser.into_offset_iter() {
        match event {
            Event::Start(Tag::CodeBlock(CodeBlockKind::Fenced(lang))) => {
                if let Some(kind) = EmbedKind::from_lang_tag(lang.as_ref()) {
                    open = Some((kind, range.start));
                }
            }
            Event::End(TagEnd::CodeBlock) => {
                if let Some((kind, start)) = open.take() {
                    if cursor < start {
                        segments.push(DocSegment::Markdown(body[cursor..start].to_string()));
                    }
                    let raw = fence_inner(&body[start..range.end]);
                    segments.push(DocSegment::Embed(EmbedData::parse(kind, &raw)));
                    cursor = range.end;
                }
            }
            _ => {}
        }
    }

    if cursor < body.len() {
        segments.push(DocSegment::Markdown(body[cursor..].to_string()));
    }

    segments
}

/// Reconstrói o corpo original a partir dos segmentos (round-trip com
/// `segment`, exceto por normalização de formatação dentro das fences).
pub fn join(segments: &[DocSegment]) -> String {
    segments
        .iter()
        .map(|s| match s {
            DocSegment::Markdown(text) => text.clone(),
            DocSegment::Embed(data) => data.to_fence_text(),
        })
        .collect::<Vec<_>>()
        .join("")
}

/// Remove as linhas de abertura (` ```lang `) e fechamento (` ``` `) de um
/// trecho de fence, devolvendo só o conteúdo interno.
fn fence_inner(fence_text: &str) -> String {
    let mut lines: Vec<&str> = fence_text.lines().collect();
    if !lines.is_empty() {
        lines.remove(0);
    }
    if lines.last().map(|l| l.trim() == "```").unwrap_or(false) {
        lines.pop();
    }
    lines.join("\n")
}

/// Dados estruturados de um embed já parseado, com o tipo carregado nele
/// mesmo (`EmbedData::kind()`).
#[derive(Debug, Clone, PartialEq)]
pub enum EmbedData {
    /// Board kanban.
    Kanban(KanbanEmbedData),
    /// Lista de eventos.
    Calendar(CalendarEmbedData),
    /// Tabela.
    Table(TableEmbedData),
}

impl EmbedData {
    /// Parseia o conteúdo interno de uma fence no tipo correspondente.
    pub fn parse(kind: EmbedKind, raw: &str) -> Self {
        match kind {
            EmbedKind::Kanban => EmbedData::Kanban(KanbanEmbedData::parse(raw)),
            EmbedKind::Calendar => EmbedData::Calendar(CalendarEmbedData::parse(raw)),
            EmbedKind::Table => EmbedData::Table(TableEmbedData::parse(raw)),
        }
    }

    /// Tipo deste embed.
    pub fn kind(&self) -> EmbedKind {
        match self {
            EmbedData::Kanban(_) => EmbedKind::Kanban,
            EmbedData::Calendar(_) => EmbedKind::Calendar,
            EmbedData::Table(_) => EmbedKind::Table,
        }
    }

    /// Serializa de volta pro texto completo da fence (com ```lang / ```),
    /// pronto pra ser gravado no corpo da página.
    pub fn to_fence_text(&self) -> String {
        let (lang, body) = match self {
            EmbedData::Kanban(d) => ("kanban", d.to_fence_body()),
            EmbedData::Calendar(d) => ("calendar", d.to_fence_body()),
            EmbedData::Table(d) => ("table", d.to_fence_body()),
        };
        format!("```{lang}\n{body}\n```\n")
    }
}

/// Um card do kanban.
#[derive(Debug, Clone, PartialEq)]
pub struct KanbanEmbedItem {
    /// Título do card.
    pub title: String,
    /// Coluna atual.
    pub column: String,
}

/// Dados de um embed kanban: colunas + cards.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct KanbanEmbedData {
    /// Colunas do board, em ordem.
    pub columns: Vec<String>,
    /// Cards.
    pub items: Vec<KanbanEmbedItem>,
}

#[derive(Deserialize, Default)]
struct RawKanbanYaml {
    #[serde(default)]
    columns: Vec<String>,
    #[serde(default)]
    items: Vec<String>,
}

impl KanbanEmbedData {
    fn parse(raw: &str) -> Self {
        let parsed: RawKanbanYaml = serde_yaml::from_str(raw).unwrap_or_default();
        let columns = if parsed.columns.is_empty() {
            vec!["Backlog".to_string(), "Todo".to_string(), "Done".to_string()]
        } else {
            parsed.columns
        };
        let default_col = columns.first().cloned().unwrap_or_else(|| "Backlog".to_string());

        let items = parsed
            .items
            .iter()
            .map(|raw_item| parse_kanban_item(raw_item, &columns, &default_col))
            .collect();

        Self { columns, items }
    }

    fn to_fence_body(&self) -> String {
        let mut out = format!("columns: [{}]\n", self.columns.join(", "));
        out.push_str("items:");
        for item in &self.items {
            out.push_str(&format!("\n  - {} ({})", item.title, item.column));
        }
        out
    }
}

fn parse_kanban_item(raw_item: &str, columns: &[String], default_col: &str) -> KanbanEmbedItem {
    let trimmed = raw_item.trim();
    if let Some(open) = trimmed.rfind('(') {
        if trimmed.ends_with(')') {
            let title = trimmed[..open].trim().to_string();
            let col_raw = trimmed[open + 1..trimmed.len() - 1].trim();
            let column = columns
                .iter()
                .find(|c| c.eq_ignore_ascii_case(col_raw))
                .cloned()
                .unwrap_or_else(|| col_raw.to_string());
            return KanbanEmbedItem { title, column };
        }
    }
    KanbanEmbedItem {
        title: trimmed.to_string(),
        column: default_col.to_string(),
    }
}

/// Um evento do calendário.
#[derive(Debug, Clone, PartialEq)]
pub struct CalendarEntry {
    /// Data (formato livre, tipicamente `YYYY-MM-DD`).
    pub date: String,
    /// Título do evento.
    pub title: String,
}

/// Dados de um embed calendar: lista de eventos `data: título`.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CalendarEmbedData {
    /// Eventos, na ordem em que aparecem na fence.
    pub entries: Vec<CalendarEntry>,
}

impl CalendarEmbedData {
    fn parse(raw: &str) -> Self {
        let entries = raw
            .lines()
            .filter_map(|line| {
                let line = line.trim();
                if line.is_empty() {
                    return None;
                }
                let (date, title) = line.split_once(':')?;
                let date = date.trim();
                let title = title.trim();
                if date.is_empty() || title.is_empty() {
                    return None;
                }
                Some(CalendarEntry {
                    date: date.to_string(),
                    title: title.to_string(),
                })
            })
            .collect();
        Self { entries }
    }

    fn to_fence_body(&self) -> String {
        self.entries
            .iter()
            .map(|e| format!("{}: {}", e.date, e.title))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// Dados de um embed table: cabeçalho + linhas, no formato de uma tabela
/// markdown comum (reaproveita o parser de tabela do pulldown-cmark).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableEmbedData {
    /// Cabeçalhos das colunas.
    pub headers: Vec<String>,
    /// Linhas de dados.
    pub rows: Vec<Vec<String>>,
}

impl TableEmbedData {
    fn parse(raw: &str) -> Self {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(raw, options);

        let mut headers = Vec::new();
        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut in_head = false;
        let mut current_row: Vec<String> = Vec::new();
        let mut current_cell = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::TableHead) => in_head = true,
                Event::End(TagEnd::TableHead) => in_head = false,
                Event::Start(Tag::TableRow) => current_row = Vec::new(),
                Event::End(TagEnd::TableRow) => rows.push(std::mem::take(&mut current_row)),
                Event::Start(Tag::TableCell) => current_cell = String::new(),
                Event::End(TagEnd::TableCell) => {
                    if in_head {
                        headers.push(std::mem::take(&mut current_cell));
                    } else {
                        current_row.push(std::mem::take(&mut current_cell));
                    }
                }
                Event::Text(t) => current_cell.push_str(&t),
                Event::Code(t) => current_cell.push_str(&t),
                _ => {}
            }
        }

        Self { headers, rows }
    }

    fn to_fence_body(&self) -> String {
        let mut out = String::new();
        out.push_str("| ");
        out.push_str(&self.headers.join(" | "));
        out.push_str(" |\n| ");
        out.push_str(
            &self
                .headers
                .iter()
                .map(|_| "---")
                .collect::<Vec<_>>()
                .join(" | "),
        );
        out.push_str(" |");
        for row in &self.rows {
            out.push_str("\n| ");
            out.push_str(&row.join(" | "));
            out.push_str(" |");
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const EXEMPLOS_EMBEDS_BODY: &str = r#"# Exemplos de blocos embedados

Você pode usar blocos especiais dentro de qualquer página `.md`:

## Kanban Embed

```kanban
columns: [Backlog, Todo, Done]
items:
  - Tarefa 1 (backlog)
  - Tarefa 2 (todo)
  - Tarefa 3 (done)
```

## Calendar Embed

```calendar
2026-08-06: Revisão de código
2026-08-07: Deploy produção
2026-08-08: Retrospectiva sprint
```

## Table Embed

```table
| Tarefa | Status | Prioridade |
| ------ | ------ | ---------- |
| API    | done   | alta       |
| UI     | doing  | media      |
| Testes | todo   | alta       |
```

Acima do embed você pode ter texto normal. Abaixo também.
"#;

    #[test]
    fn segment_recognizes_all_three_embeds_in_order() {
        let segments = segment(EXEMPLOS_EMBEDS_BODY);
        let kinds: Vec<Option<EmbedKind>> = segments
            .iter()
            .map(|s| match s {
                DocSegment::Embed(d) => Some(d.kind()),
                DocSegment::Markdown(_) => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec![
                None,
                Some(EmbedKind::Kanban),
                None,
                Some(EmbedKind::Calendar),
                None,
                Some(EmbedKind::Table),
                None,
            ]
        );
    }

    #[test]
    fn segment_preserves_surrounding_markdown_text() {
        let segments = segment(EXEMPLOS_EMBEDS_BODY);
        let DocSegment::Markdown(first) = &segments[0] else {
            panic!("primeiro segmento devia ser markdown");
        };
        assert!(first.contains("# Exemplos de blocos embedados"));
        assert!(first.contains("## Kanban Embed"));

        let DocSegment::Markdown(last) = segments.last().unwrap() else {
            panic!("último segmento devia ser markdown");
        };
        assert!(last.contains("Acima do embed você pode ter texto normal."));
    }

    #[test]
    fn kanban_embed_parses_columns_and_items() {
        let segments = segment(EXEMPLOS_EMBEDS_BODY);
        let DocSegment::Embed(EmbedData::Kanban(data)) = &segments[1] else {
            panic!("esperava embed kanban no índice 1");
        };
        assert_eq!(data.columns, vec!["Backlog", "Todo", "Done"]);
        assert_eq!(data.items.len(), 3);
        assert_eq!(data.items[0].title, "Tarefa 1");
        assert_eq!(data.items[0].column, "Backlog");
        assert_eq!(data.items[2].column, "Done");
    }

    #[test]
    fn calendar_embed_parses_entries() {
        let segments = segment(EXEMPLOS_EMBEDS_BODY);
        let DocSegment::Embed(EmbedData::Calendar(data)) = &segments[3] else {
            panic!("esperava embed calendar no índice 3");
        };
        assert_eq!(data.entries.len(), 3);
        assert_eq!(data.entries[0].date, "2026-08-06");
        assert_eq!(data.entries[0].title, "Revisão de código");
    }

    #[test]
    fn table_embed_parses_headers_and_rows() {
        let segments = segment(EXEMPLOS_EMBEDS_BODY);
        let DocSegment::Embed(EmbedData::Table(data)) = &segments[5] else {
            panic!("esperava embed table no índice 5");
        };
        assert_eq!(data.headers, vec!["Tarefa", "Status", "Prioridade"]);
        assert_eq!(data.rows.len(), 3);
        assert_eq!(data.rows[0], vec!["API", "done", "alta"]);
    }

    #[test]
    fn plain_code_fence_is_not_treated_as_embed() {
        let body = "texto\n\n```rust\nfn main() {}\n```\n\nmais texto\n";
        let segments = segment(body);
        assert!(segments
            .iter()
            .all(|s| matches!(s, DocSegment::Markdown(_))));
    }

    #[test]
    fn kanban_roundtrip_reparse() {
        let data = KanbanEmbedData {
            columns: vec!["Backlog".into(), "Doing".into()],
            items: vec![KanbanEmbedItem { title: "X".into(), column: "Doing".into() }],
        };
        let fence_body = data.to_fence_body();
        let reparsed = KanbanEmbedData::parse(&fence_body);
        assert_eq!(reparsed.columns, data.columns);
        assert_eq!(reparsed.items.len(), 1);
        assert_eq!(reparsed.items[0].title, "X");
        assert_eq!(reparsed.items[0].column, "Doing");
    }
}
