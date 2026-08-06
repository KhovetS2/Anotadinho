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
///
/// Um trecho de markdown que passou por edição (ex: `html_to_markdown`, que
/// não preserva a quebra de linha final do texto original) pode acabar sem
/// `\n` no fim. Sem isso o próximo segmento (ex: a fence de um embed) gruda
/// na mesma linha e corrompe o arquivo — então garante a quebra aqui, uma
/// vez, em vez de depender de cada chamador lembrar disso.
pub fn join(segments: &[DocSegment]) -> String {
    let mut out = String::new();
    for segment in segments {
        match segment {
            DocSegment::Markdown(text) => {
                out.push_str(text);
                if !text.is_empty() && !text.ends_with('\n') {
                    out.push('\n');
                }
            }
            DocSegment::Embed(data) => out.push_str(&data.to_fence_text()),
        }
    }
    out
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
        // yaml_scalar é necessário aqui: um item "Revisar PR #42 (Backlog)"
        // sem aspas faria o YAML tratar " #42..." como comentário (regra do
        // YAML: '#' precedido de espaço abre comentário em escalar plano) e
        // cortar o resto da linha — título truncado E coluna errada no
        // reparse seguinte.
        let cols = self.columns.iter().map(|c| yaml_scalar(c)).collect::<Vec<_>>().join(", ");
        let mut out = format!("columns: [{}]\n", cols);
        out.push_str("items:");
        for item in &self.items {
            let composite = format!("{} ({})", item.title, item.column);
            out.push_str(&format!("\n  - {}", yaml_scalar(&composite)));
        }
        out
    }

    /// Adiciona uma coluna nova ao fim do board.
    pub fn add_column(&mut self, name: String) {
        self.columns.push(name);
    }

    /// Renomeia a coluna no índice `idx`, atualizando também todo card que
    /// apontava pro nome antigo.
    pub fn rename_column(&mut self, idx: usize, new_name: String) {
        let Some(old_name) = self.columns.get(idx).cloned() else { return };
        if let Some(c) = self.columns.get_mut(idx) {
            *c = new_name.clone();
        }
        for item in &mut self.items {
            if item.column == old_name {
                item.column = new_name.clone();
            }
        }
    }

    /// Remove a coluna no índice `idx` e todos os cards nela.
    pub fn remove_column(&mut self, idx: usize) {
        let Some(name) = self.columns.get(idx).cloned() else { return };
        self.columns.remove(idx);
        self.items.retain(|item| item.column != name);
    }

    /// Adiciona um card novo na coluna dada.
    pub fn add_card(&mut self, column: String, title: String) {
        self.items.push(KanbanEmbedItem { title, column });
    }

    /// Edita o título do card no índice `idx`.
    pub fn edit_card(&mut self, idx: usize, new_title: String) {
        if let Some(item) = self.items.get_mut(idx) {
            item.title = new_title;
        }
    }

    /// Remove o card no índice `idx`.
    pub fn remove_card(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.items.remove(idx);
        }
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

    /// Adiciona um evento novo.
    pub fn add_entry(&mut self, date: String, title: String) {
        self.entries.push(CalendarEntry { date, title });
    }

    /// Edita a data e/ou o título do evento no índice `idx`.
    pub fn edit_entry(&mut self, idx: usize, new_date: String, new_title: String) {
        if let Some(entry) = self.entries.get_mut(idx) {
            entry.date = new_date;
            entry.title = new_title;
        }
    }

    /// Remove o evento no índice `idx`.
    pub fn remove_entry(&mut self, idx: usize) {
        if idx < self.entries.len() {
            self.entries.remove(idx);
        }
    }
}

/// Tipo de uma coluna da tabela — controla como a célula é editada e
/// renderizada. `Text` é o padrão (compatível com qualquer tabela markdown
/// comum, sem preâmbulo de configuração nenhum).
#[derive(Debug, Clone, PartialEq)]
pub enum ColumnKind {
    /// Texto livre.
    Text,
    /// Caixa de seleção (valor `"true"`/`"false"` na célula).
    Checkbox,
    /// Valor de uma lista fixa de opções, mostrado como badge colorido.
    Select {
        /// Opções permitidas, na ordem em que aparecem no seletor.
        options: Vec<String>,
    },
}

/// Uma coluna da tabela: nome + tipo.
#[derive(Debug, Clone, PartialEq)]
pub struct TableColumn {
    /// Nome/cabeçalho da coluna.
    pub name: String,
    /// Tipo da célula.
    pub kind: ColumnKind,
}

/// Dados de um embed table: colunas (com tipo) + linhas.
///
/// Por padrão a fence é só uma tabela markdown comum (todas as colunas
/// nascem `Text`, 100% compatível com qualquer tabela já existente). Só
/// quando alguma coluna tem um tipo diferente, um preâmbulo YAML é
/// escrito antes da tabela, separado por uma linha `---` (mesma convenção
/// visual do frontmatter da página):
///
/// ```text
/// columns:
///   - name: Status
///     type: select
///     options: [todo, doing, done]
/// ---
/// | Tarefa | Status |
/// | ------ | ------ |
/// | API    | done   |
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TableEmbedData {
    /// Colunas, em ordem.
    pub columns: Vec<TableColumn>,
    /// Linhas de dados (cada uma com `columns.len()` células).
    pub rows: Vec<Vec<String>>,
}

#[derive(Deserialize, Default)]
struct RawColumnConfig {
    name: String,
    #[serde(rename = "type", default)]
    kind: Option<String>,
    #[serde(default)]
    options: Vec<String>,
}

#[derive(Deserialize, Default)]
struct RawTableYaml {
    #[serde(default)]
    columns: Vec<RawColumnConfig>,
}

impl TableEmbedData {
    fn parse(raw: &str) -> Self {
        let (config_part, table_part) = match raw.find("\n---\n") {
            Some(pos) => (Some(&raw[..pos]), &raw[pos + 5..]),
            None => (None, raw),
        };

        let configured: Vec<(String, ColumnKind)> = config_part
            .and_then(|yaml| serde_yaml::from_str::<RawTableYaml>(yaml).ok())
            .map(|parsed| {
                parsed
                    .columns
                    .into_iter()
                    .map(|c| {
                        let kind = match c.kind.as_deref() {
                            Some("select") => ColumnKind::Select { options: c.options },
                            Some("checkbox") => ColumnKind::Checkbox,
                            _ => ColumnKind::Text,
                        };
                        (c.name, kind)
                    })
                    .collect()
            })
            .unwrap_or_default();

        let mut options = Options::empty();
        options.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(table_part, options);

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

        let columns = headers
            .into_iter()
            .map(|name| {
                let kind = configured
                    .iter()
                    .find(|(n, _)| *n == name)
                    .map(|(_, k)| k.clone())
                    .unwrap_or(ColumnKind::Text);
                TableColumn { name, kind }
            })
            .collect();

        Self { columns, rows }
    }

    fn to_fence_body(&self) -> String {
        let mut out = String::new();

        let has_typed = self.columns.iter().any(|c| c.kind != ColumnKind::Text);
        if has_typed {
            out.push_str("columns:\n");
            for c in &self.columns {
                out.push_str(&format!("  - name: {}\n", yaml_scalar(&c.name)));
                match &c.kind {
                    ColumnKind::Text => {}
                    ColumnKind::Checkbox => out.push_str("    type: checkbox\n"),
                    ColumnKind::Select { options } => {
                        out.push_str("    type: select\n");
                        let opts = options.iter().map(|o| yaml_scalar(o)).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("    options: [{}]\n", opts));
                    }
                }
            }
            out.push_str("---\n");
        }

        out.push_str("| ");
        out.push_str(&self.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>().join(" | "));
        out.push_str(" |\n| ");
        out.push_str(&self.columns.iter().map(|_| "---").collect::<Vec<_>>().join(" | "));
        out.push_str(" |");
        for row in &self.rows {
            out.push_str("\n| ");
            out.push_str(&row.join(" | "));
            out.push_str(" |");
        }
        out
    }

    /// Adiciona uma linha vazia (uma célula por coluna).
    pub fn add_row(&mut self) {
        self.rows.push(vec![String::new(); self.columns.len()]);
    }

    /// Remove a linha no índice `idx`.
    pub fn remove_row(&mut self, idx: usize) {
        if idx < self.rows.len() {
            self.rows.remove(idx);
        }
    }

    /// Adiciona uma coluna nova (tipo `Text`) ao fim, com célula vazia em
    /// toda linha existente.
    pub fn add_column(&mut self, name: String) {
        self.columns.push(TableColumn { name, kind: ColumnKind::Text });
        for row in &mut self.rows {
            row.push(String::new());
        }
    }

    /// Remove a coluna no índice `idx` e a célula correspondente de toda linha.
    pub fn remove_column(&mut self, idx: usize) {
        if idx >= self.columns.len() {
            return;
        }
        self.columns.remove(idx);
        for row in &mut self.rows {
            if idx < row.len() {
                row.remove(idx);
            }
        }
    }

    /// Renomeia a coluna no índice `idx`.
    pub fn set_column_name(&mut self, idx: usize, name: String) {
        if let Some(c) = self.columns.get_mut(idx) {
            c.name = name;
        }
    }

    /// Troca o tipo da coluna no índice `idx`.
    pub fn set_column_kind(&mut self, idx: usize, kind: ColumnKind) {
        if let Some(c) = self.columns.get_mut(idx) {
            c.kind = kind;
        }
    }

    /// Define o valor da célula em `(row, col)`.
    pub fn set_cell(&mut self, row: usize, col: usize, value: String) {
        if let Some(r) = self.rows.get_mut(row) {
            if col < r.len() {
                r[col] = value;
            }
        }
    }
}

fn yaml_scalar(s: &str) -> String {
    let needs_quote = s.is_empty() || !s.chars().all(|c| c.is_alphanumeric() || c == ' ' || c == '-' || c == '_');
    if needs_quote {
        format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\""))
    } else {
        s.to_string()
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
    fn join_inserts_missing_newline_before_next_segment() {
        // Regressão: um trecho de markdown editado (ex: vindo de
        // html_to_markdown, que não preserva o \n final) não pode grudar no
        // próximo segmento — isso corrompe o arquivo salvo (ex:
        // "## Kanban Embed```kanban" numa linha só).
        let segments = vec![
            DocSegment::Markdown("## Kanban Embed".to_string()), // sem \n final
            DocSegment::Embed(EmbedData::Kanban(KanbanEmbedData {
                columns: vec!["Backlog".into()],
                items: vec![],
            })),
        ];
        let joined = join(&segments);
        assert!(
            joined.starts_with("## Kanban Embed\n```kanban"),
            "esperava quebra de linha entre o heading e a fence, ficou: {joined:?}"
        );
    }

    #[test]
    fn join_does_not_duplicate_existing_newline() {
        let segments = vec![
            DocSegment::Markdown("texto\n\n".to_string()),
            DocSegment::Embed(EmbedData::Calendar(CalendarEmbedData { entries: vec![] })),
        ];
        let joined = join(&segments);
        assert!(joined.starts_with("texto\n\n```calendar"));
    }

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
        let names: Vec<&str> = data.columns.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["Tarefa", "Status", "Prioridade"]);
        assert!(data.columns.iter().all(|c| c.kind == ColumnKind::Text));
        assert_eq!(data.rows.len(), 3);
        assert_eq!(data.rows[0], vec!["API", "done", "alta"]);
    }

    #[test]
    fn table_typed_columns_roundtrip() {
        let mut data = TableEmbedData {
            columns: vec![
                TableColumn { name: "Tarefa".into(), kind: ColumnKind::Text },
                TableColumn { name: "Status".into(), kind: ColumnKind::Select { options: vec!["todo".into(), "done".into()] } },
                TableColumn { name: "Feito".into(), kind: ColumnKind::Checkbox },
            ],
            rows: vec![vec!["API".into(), "done".into(), "true".into()]],
        };
        let fence_body = data.to_fence_body();
        assert!(fence_body.starts_with("columns:\n"), "esperava preâmbulo YAML, ficou: {fence_body:?}");

        let reparsed = TableEmbedData::parse(&fence_body);
        assert_eq!(reparsed.columns.len(), 3);
        assert_eq!(reparsed.columns[0].kind, ColumnKind::Text);
        assert_eq!(reparsed.columns[1].kind, ColumnKind::Select { options: vec!["todo".into(), "done".into()] });
        assert_eq!(reparsed.columns[2].kind, ColumnKind::Checkbox);
        assert_eq!(reparsed.rows, data.rows);

        // Sem tipos especiais, não deve escrever preâmbulo (retrocompatível).
        data.columns[1].kind = ColumnKind::Text;
        data.columns[2].kind = ColumnKind::Text;
        let plain_body = data.to_fence_body();
        assert!(!plain_body.contains("columns:\n"), "não devia ter preâmbulo: {plain_body:?}");
    }

    #[test]
    fn table_add_remove_row_and_column() {
        let mut data = TableEmbedData {
            columns: vec![TableColumn { name: "A".into(), kind: ColumnKind::Text }],
            rows: vec![vec!["1".into()]],
        };
        data.add_column("B".into());
        assert_eq!(data.columns.len(), 2);
        assert_eq!(data.rows[0], vec!["1", ""]);

        data.add_row();
        assert_eq!(data.rows.len(), 2);
        assert_eq!(data.rows[1], vec!["", ""]);

        data.set_cell(1, 1, "x".into());
        assert_eq!(data.rows[1][1], "x");

        data.remove_column(0);
        assert_eq!(data.columns[0].name, "B");
        assert_eq!(data.rows[0], vec![""]);
        assert_eq!(data.rows[1], vec!["x"]);

        data.remove_row(0);
        assert_eq!(data.rows.len(), 1);
        assert_eq!(data.rows[0], vec!["x"]);
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
    fn kanban_add_card_and_column() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![],
        };
        data.add_column("Done".into());
        data.add_card("Backlog".into(), "Nova tarefa".into());
        assert_eq!(data.columns, vec!["Backlog", "Done"]);
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].title, "Nova tarefa");
        assert_eq!(data.items[0].column, "Backlog");
    }

    #[test]
    fn kanban_edit_and_remove_card() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![KanbanEmbedItem { title: "X".into(), column: "Backlog".into() }],
        };
        data.edit_card(0, "Y".into());
        assert_eq!(data.items[0].title, "Y");
        data.remove_card(0);
        assert!(data.items.is_empty());
    }

    #[test]
    fn kanban_rename_column_cascades_to_items() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into(), "Done".into()],
            items: vec![
                KanbanEmbedItem { title: "A".into(), column: "Backlog".into() },
                KanbanEmbedItem { title: "B".into(), column: "Done".into() },
            ],
        };
        data.rename_column(0, "A Fazer".into());
        assert_eq!(data.columns, vec!["A Fazer", "Done"]);
        assert_eq!(data.items[0].column, "A Fazer");
        assert_eq!(data.items[1].column, "Done");
    }

    #[test]
    fn kanban_remove_column_removes_its_cards() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into(), "Done".into()],
            items: vec![
                KanbanEmbedItem { title: "A".into(), column: "Backlog".into() },
                KanbanEmbedItem { title: "B".into(), column: "Done".into() },
            ],
        };
        data.remove_column(0);
        assert_eq!(data.columns, vec!["Done"]);
        assert_eq!(data.items.len(), 1);
        assert_eq!(data.items[0].title, "B");
    }

    #[test]
    fn calendar_add_edit_remove_entry() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Revisão".into());
        assert_eq!(data.entries.len(), 1);
        data.edit_entry(0, "2026-08-07".into(), "Revisão adiada".into());
        assert_eq!(data.entries[0].date, "2026-08-07");
        assert_eq!(data.entries[0].title, "Revisão adiada");
        data.remove_entry(0);
        assert!(data.entries.is_empty());
    }

    #[test]
    fn kanban_item_with_hash_survives_roundtrip() {
        // Regressão: "#" precedido de espaço vira comentário em YAML plano
        // sem aspas — cortava o título e derrubava o card pra coluna errada.
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![],
        };
        data.add_card("Backlog".into(), "Revisar PR #42".into());
        let fence_body = data.to_fence_body();
        let reparsed = KanbanEmbedData::parse(&fence_body);
        assert_eq!(reparsed.items.len(), 1);
        assert_eq!(reparsed.items[0].title, "Revisar PR #42");
        assert_eq!(reparsed.items[0].column, "Backlog");
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
