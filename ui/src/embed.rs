//! Base extensível para embeds inline: blocos `{{ type: "kanban" }} ...
//! {{ /kanban }}` dentro de uma página comum, parseados em dados
//! estruturados e renderizados como componentes Yew de verdade (não texto
//! cru).
//!
//! Por que não usar fence markdown (` ```kanban ``` `)? Colide
//! semanticamente com blocos de código de verdade — alguém que quisesse
//! mostrar um trecho de código chamado "kanban" teria o mesmo tratamento.
//! O wrapper `{{ }}` não existe em CommonMark, então nunca conflita com
//! nada do markdown normal.
//!
//! Extensão: um novo tipo de embed é 1 variante em `EmbedKind` + 1 par
//! parse/serialize em `EmbedData` + 1 componente Yew (`components/embeds/`)
//! + 1 braço de `match` no editor. Nada mais precisa mudar.

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use serde::{Deserialize, Serialize};

/// Paleta de cores compartilhada pra badges de tag/select — usada pela
/// tabela (Select/MultiSelect) e pelo calendário (cor do evento), pra não
/// duplicar as cores em cada componente.
pub const BADGE_PALETTE: [&str; 4] = ["badge--info", "badge--success", "badge--warning", "badge--error"];

/// Classe CSS de badge pra `value` dentro da lista `options`, ciclando
/// pela `BADGE_PALETTE` conforme a posição — mesma opção sempre cai na
/// mesma cor enquanto a ordem das opções não mudar.
pub fn badge_class(options: &[String], value: &str) -> &'static str {
    match options.iter().position(|o| o == value) {
        Some(i) => BADGE_PALETTE[i % BADGE_PALETTE.len()],
        None => "badge",
    }
}

/// Tipos de embed inline reconhecidos pelo `type` do wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedKind {
    /// `{{ type: "kanban" }}` — board com colunas e cards.
    Kanban,
    /// `{{ type: "calendar" }}` — lista de eventos por data.
    Calendar,
    /// `{{ type: "table" }}` — tabela com colunas tipadas.
    Table,
}

impl EmbedKind {
    /// Reconhece o `type` do wrapper (`kanban`/`calendar`/`table`).
    pub fn from_type_name(name: &str) -> Option<Self> {
        match name {
            "kanban" => Some(Self::Kanban),
            "calendar" => Some(Self::Calendar),
            "table" => Some(Self::Table),
            _ => None,
        }
    }

    /// Nome usado no wrapper (`{{ type: "X" }}` / `{{ /X }}`).
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Kanban => "kanban",
            Self::Calendar => "calendar",
            Self::Table => "table",
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

/// Reconhece uma linha de abertura `{{ type: "kanban" }}`. Espaçamento
/// livre: `{{type:"kanban"}}` bate igual.
fn parse_open_tag(line: &str) -> Option<EmbedKind> {
    let inner = line.trim().strip_prefix("{{")?.strip_suffix("}}")?.trim();
    let rest = inner.strip_prefix("type")?.trim_start();
    let rest = rest.strip_prefix(':')?.trim();
    let quoted = rest.strip_prefix('"')?.strip_suffix('"')?;
    EmbedKind::from_type_name(quoted)
}

/// Reconhece a linha de fechamento `{{ /kanban }}` correspondente a `kind`.
fn parse_close_tag(line: &str, kind: EmbedKind) -> bool {
    let Some(inner) = line.trim().strip_prefix("{{").and_then(|s| s.strip_suffix("}}")) else {
        return false;
    };
    inner
        .trim()
        .strip_prefix('/')
        .map(|name| name.trim() == kind.type_name())
        .unwrap_or(false)
}

/// Segmenta o corpo (já sem frontmatter) em uma sequência ordenada de
/// trechos de markdown comum e embeds. Varre linha a linha com um cursor
/// de offset de byte — não perde nem reformata o texto original fora dos
/// wrappers reconhecidos. Um wrapper aberto sem fechamento correspondente
/// consome até o fim do texto (sem pânico, degrada de forma previsível).
pub fn segment(body: &str) -> Vec<DocSegment> {
    let mut segments = Vec::new();
    let mut cursor = 0usize;
    let mut pos = 0usize;
    let mut lines = body.split_inclusive('\n');

    while let Some(line) = lines.next() {
        let line_start = pos;
        pos += line.len();

        let Some(kind) = parse_open_tag(line) else { continue };

        if cursor < line_start {
            segments.push(DocSegment::Markdown(body[cursor..line_start].to_string()));
        }

        let content_start = pos;
        let mut content_end = body.len();
        for next_line in lines.by_ref() {
            let next_start = pos;
            pos += next_line.len();
            if parse_close_tag(next_line, kind) {
                content_end = next_start;
                break;
            }
        }

        let raw = &body[content_start..content_end];
        segments.push(DocSegment::Embed(EmbedData::parse(kind, raw)));
        cursor = pos;
    }

    if cursor < body.len() {
        segments.push(DocSegment::Markdown(body[cursor..].to_string()));
    }

    segments
}

/// Reconstrói o corpo original a partir dos segmentos (round-trip com
/// `segment`, exceto por normalização de formatação dentro dos wrappers).
///
/// Um trecho de markdown que passou por edição (ex: `html_to_markdown`, que
/// não preserva a quebra de linha final do texto original) pode acabar sem
/// `\n` no fim. Sem isso o próximo segmento (ex: o wrapper de um embed)
/// gruda na mesma linha e corrompe o arquivo — então garante a quebra
/// aqui, uma vez, em vez de depender de cada chamador lembrar disso.
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
    /// Parseia o conteúdo interno de um wrapper no tipo correspondente.
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

    /// Serializa de volta pro texto completo do wrapper
    /// (`{{ type: "X" }}` ... `{{ /X }}`), pronto pra ser gravado no
    /// corpo da página.
    pub fn to_fence_text(&self) -> String {
        let name = self.kind().type_name();
        let body = match self {
            EmbedData::Kanban(d) => d.to_fence_body(),
            EmbedData::Calendar(d) => d.to_fence_body(),
            EmbedData::Table(d) => d.to_fence_body(),
        };
        format!("{{{{ type: \"{name}\" }}}}\n{body}\n{{{{ /{name} }}}}\n")
    }
}

/// Um item de checklist dentro de um card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChecklistItem {
    /// Texto do sub-item.
    pub text: String,
    /// Se já foi concluído.
    #[serde(default)]
    pub done: bool,
}

/// Um comentário num card.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comment {
    /// Texto do comentário.
    pub text: String,
    /// Data livre (ex: `"2026-08-06"`), preenchida na hora de criar.
    pub created: String,
}

/// Um anexo num card — arquivo copiado pra `assets/` do vault.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Attachment {
    /// Nome de exibição.
    pub name: String,
    /// Path relativo ao vault (ex: `"assets/diagrama.png"`).
    pub path: String,
}

/// Um card do kanban, com campos ricos opcionais além de título/coluna.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KanbanCard {
    /// Título do card.
    pub title: String,
    /// Coluna atual.
    pub column: String,
    /// Descrição longa, opcional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Tags/labels do card.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Data de vencimento livre (`"YYYY-MM-DD"`), opcional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub due: Option<String>,
    /// Sub-itens de checklist.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checklist: Vec<ChecklistItem>,
    /// Comentários, em ordem cronológica.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub comments: Vec<Comment>,
    /// Arquivos anexados.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<Attachment>,
}

/// Dados de um embed kanban: colunas + cards.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct KanbanEmbedData {
    /// Colunas do board, em ordem.
    pub columns: Vec<String>,
    /// Cards.
    #[serde(default)]
    pub items: Vec<KanbanCard>,
}

impl KanbanEmbedData {
    fn parse(raw: &str) -> Self {
        let mut data: Self = serde_yaml::from_str(raw).unwrap_or_default();
        if data.columns.is_empty() {
            data.columns = vec!["Backlog".to_string(), "Todo".to_string(), "Done".to_string()];
        }
        data
    }

    fn to_fence_body(&self) -> String {
        // derive de serde: sem montagem de string na mão, sem risco de um
        // "#"/":" no meio de um campo virar sintaxe YAML por engano (bug
        // real do ciclo anterior) — o serde_yaml escapa tudo sozinho.
        serde_yaml::to_string(self).unwrap_or_default()
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

    /// Adiciona um card novo (só título/coluna) na coluna dada. Campos
    /// ricos se adicionam depois pelo modal de detalhes.
    pub fn add_card(&mut self, column: String, title: String) {
        self.items.push(KanbanCard { title, column, ..Default::default() });
    }

    /// Edita o título do card no índice `idx` (edição rápida).
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

    /// Substitui o card inteiro no índice `idx` (usado pelo modal de
    /// detalhes, que edita vários campos de uma vez).
    pub fn update_card(&mut self, idx: usize, card: KanbanCard) {
        if let Some(slot) = self.items.get_mut(idx) {
            *slot = card;
        }
    }

    /// Alterna concluído/pendente de um item de checklist.
    pub fn toggle_checklist_item(&mut self, card_idx: usize, item_idx: usize) {
        if let Some(item) = self
            .items
            .get_mut(card_idx)
            .and_then(|c| c.checklist.get_mut(item_idx))
        {
            item.done = !item.done;
        }
    }

    /// Adiciona um item de checklist ao card.
    pub fn add_checklist_item(&mut self, card_idx: usize, text: String) {
        if let Some(card) = self.items.get_mut(card_idx) {
            card.checklist.push(ChecklistItem { text, done: false });
        }
    }

    /// Remove um item de checklist do card.
    pub fn remove_checklist_item(&mut self, card_idx: usize, item_idx: usize) {
        if let Some(card) = self.items.get_mut(card_idx) {
            if item_idx < card.checklist.len() {
                card.checklist.remove(item_idx);
            }
        }
    }

    /// Adiciona um comentário ao card.
    pub fn add_comment(&mut self, card_idx: usize, text: String, created: String) {
        if let Some(card) = self.items.get_mut(card_idx) {
            card.comments.push(Comment { text, created });
        }
    }

    /// Adiciona um anexo ao card (já copiado pra `assets/`; ver
    /// `crate::api::copy_to_assets`).
    pub fn add_attachment(&mut self, card_idx: usize, name: String, path: String) {
        if let Some(card) = self.items.get_mut(card_idx) {
            card.attachments.push(Attachment { name, path });
        }
    }

    /// Remove um anexo do card.
    pub fn remove_attachment(&mut self, card_idx: usize, attachment_idx: usize) {
        if let Some(card) = self.items.get_mut(card_idx) {
            if attachment_idx < card.attachments.len() {
                card.attachments.remove(attachment_idx);
            }
        }
    }

    /// Move o card no índice `from_idx` pra coluna `to_column`, na posição
    /// logo antes de `before_card_idx` (índice ORIGINAL, antes da
    /// remoção) — ou pro fim da coluna se `before_card_idx` for `None`.
    /// Resolve trocar de coluna e reordenar dentro da mesma coluna com uma
    /// única operação.
    pub fn move_card(&mut self, from_idx: usize, to_column: String, before_card_idx: Option<usize>) {
        if from_idx >= self.items.len() {
            return;
        }
        let mut card = self.items.remove(from_idx);
        card.column = to_column;
        let insert_at = match before_card_idx {
            Some(idx) => {
                let adjusted = if idx > from_idx { idx - 1 } else { idx };
                adjusted.min(self.items.len())
            }
            None => self.items.len(),
        };
        self.items.insert(insert_at, card);
    }
}

/// Um evento do calendário.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarEntry {
    /// Data de início (`YYYY-MM-DD`). `None` = evento "sem data" — fica
    /// fora da grade, na gaveta de eventos sem data, até o usuário
    /// arrastar pra um dia ou definir pelo modal.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<String>,
    /// Título do evento.
    pub title: String,
    /// Data de fim (inclusiva), se o evento se estender por vários dias.
    /// `None` = evento de 1 dia só.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_date: Option<String>,
    /// Tag/cor do evento (mesma paleta de badge usada na tabela).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tag: Option<String>,
    /// Horário de início (`"HH:MM"`). `None` = evento de dia inteiro (sem
    /// horário) — comportamento padrão, igual ao de antes deste campo
    /// existir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Horário de fim (`"HH:MM"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
}

/// Dados de um embed calendar: lista de eventos.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarEmbedData {
    /// Eventos, na ordem em que aparecem no wrapper.
    #[serde(default)]
    pub entries: Vec<CalendarEntry>,
}

impl CalendarEmbedData {
    fn parse(raw: &str) -> Self {
        serde_yaml::from_str(raw).unwrap_or_default()
    }

    fn to_fence_body(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Adiciona um evento novo de 1 dia.
    pub fn add_entry(&mut self, date: String, title: String) {
        self.entries.push(CalendarEntry { date: Some(date), title, ..Default::default() });
    }

    /// Adiciona um evento novo já com horário (`"HH:MM"`) — usado ao
    /// clicar num horário específico da grade de Semana/Dia.
    pub fn add_entry_timed(&mut self, date: String, title: String, start_time: String, end_time: String) {
        self.entries.push(CalendarEntry {
            date: Some(date),
            title,
            start_time: Some(start_time),
            end_time: Some(end_time),
            ..Default::default()
        });
    }

    /// Adiciona um evento sem data — fica na gaveta até o usuário
    /// arrastar pra um dia da grade ou definir uma data pelo modal.
    pub fn add_unscheduled_entry(&mut self, title: String) {
        self.entries.push(CalendarEntry { date: None, title, ..Default::default() });
    }

    /// Salva a entrada inteira no índice `idx` (usado pelo modal de
    /// detalhes, que edita título/datas/tag juntos).
    pub fn update_entry(&mut self, idx: usize, entry: CalendarEntry) {
        if let Some(e) = self.entries.get_mut(idx) {
            *e = entry;
        }
    }

    /// Remove o evento no índice `idx`.
    pub fn remove_entry(&mut self, idx: usize) {
        if idx < self.entries.len() {
            self.entries.remove(idx);
        }
    }

    /// Desloca o evento no índice `idx` pra começar em `new_start`,
    /// preservando a duração (se tinha `end_date`, desloca junto pela
    /// mesma diferença de dias). Também é o caminho usado pra atribuir
    /// data a um evento "sem data" (arrastar da gaveta pra um dia) — nesse
    /// caso não existe data antiga pra calcular duração, então só define
    /// `date` mesmo.
    pub fn move_entry(&mut self, idx: usize, new_start: String) {
        let Some(entry) = self.entries.get_mut(idx) else { return };
        if let (Some(old_start), Some(end)) = (&entry.date, &entry.end_date) {
            let duration_days = crate::date_util::days_between(old_start, end).unwrap_or(0);
            if let Some(new_end) = crate::date_util::add_days(&new_start, duration_days) {
                entry.end_date = Some(new_end);
            }
        }
        entry.date = Some(new_start);
    }

    /// Desloca o evento no índice `idx` pra uma nova data + horário de
    /// início, preservando a duração em minutos (se tinha
    /// `start_time`/`end_time` definidos). Usado ao arrastar
    /// verticalmente/entre dias na grade de horas (Semana/Dia). Eventos
    /// sem horário (dia inteiro) não são afetados por essa função — a
    /// grade de horas só arrasta blocos que já têm `start_time`.
    pub fn move_entry_time(&mut self, idx: usize, new_date: String, new_start_time: String) {
        let Some(entry) = self.entries.get_mut(idx) else { return };
        let new_start_parsed = crate::date_util::parse_time(&new_start_time);
        if let (Some(start), Some(end), Some((nsh, nsm))) = (&entry.start_time, &entry.end_time, new_start_parsed) {
            if let (Some((sh, sm)), Some((eh, em))) = (crate::date_util::parse_time(start), crate::date_util::parse_time(end)) {
                let duration = crate::date_util::minutes_since_midnight(eh, em) as i64
                    - crate::date_util::minutes_since_midnight(sh, sm) as i64;
                let new_start_min = crate::date_util::minutes_since_midnight(nsh, nsm) as i64;
                let new_end_min = (new_start_min + duration).clamp(0, 23 * 60 + 59) as u32;
                entry.end_time = Some(crate::date_util::format_time(new_end_min / 60, new_end_min % 60));
            }
        }
        entry.start_time = Some(new_start_time);
        entry.date = Some(new_date);
    }

    /// Redimensiona um evento com horário arrastando a borda superior
    /// (`is_start_edge = true`) ou inferior do bloco na grade de horas.
    /// Mantém duração mínima de 15min — se a borda arrastada ultrapassar a
    /// oposta, para no limite em vez de inverter início/fim. Sem efeito em
    /// eventos sem `start_time` (dia inteiro).
    pub fn resize_entry_time(&mut self, idx: usize, is_start_edge: bool, new_minutes: u32) {
        const MIN_DURATION: u32 = 15;
        let Some(entry) = self.entries.get_mut(idx) else { return };
        let Some(start_min) = entry.start_time.as_deref()
            .and_then(crate::date_util::parse_time)
            .map(|(h, m)| crate::date_util::minutes_since_midnight(h, m))
        else { return };
        let end_min = entry.end_time.as_deref()
            .and_then(crate::date_util::parse_time)
            .map(|(h, m)| crate::date_util::minutes_since_midnight(h, m))
            .unwrap_or(start_min + 60);

        if is_start_edge {
            let clamped = new_minutes.min(end_min.saturating_sub(MIN_DURATION));
            entry.start_time = Some(crate::date_util::format_time(clamped / 60, clamped % 60));
        } else {
            let clamped = new_minutes.max(start_min + MIN_DURATION).min(23 * 60 + 59);
            entry.end_time = Some(crate::date_util::format_time(clamped / 60, clamped % 60));
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
    /// Múltiplos valores de uma lista de opções ("tags"), mostrados como
    /// vários badges — a lista de opções cresce dinamicamente: digitar um
    /// valor novo na célula cadastra a opção na coluna.
    MultiSelect {
        /// Opções já cadastradas na coluna, na ordem em que foram criadas.
        options: Vec<String>,
    },
    /// Número (célula editada com `<input type="number">`).
    Number,
    /// Data no formato `"YYYY-MM-DD"`.
    Date,
    /// URL — célula mostra um link clicável.
    Url,
    /// Referência a outra página do vault — guarda o `path` relativo
    /// (`PageMeta::path`); o título exibido é resolvido em runtime.
    PageLink,
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
/// O corpo continua sendo uma tabela markdown comum por padrão (todas as
/// colunas nascem `Text`, 100% compatível com qualquer tabela já
/// existente — decisão deliberada, diferente do kanban/calendário, pra
/// manter uma tabela simples fácil de editar à mão). Só quando alguma
/// coluna tem um tipo diferente, um preâmbulo YAML é escrito antes da
/// tabela, separado por uma linha `---`:
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
                            Some("multiselect") | Some("tags") => ColumnKind::MultiSelect { options: c.options },
                            Some("checkbox") => ColumnKind::Checkbox,
                            Some("number") => ColumnKind::Number,
                            Some("date") => ColumnKind::Date,
                            Some("url") => ColumnKind::Url,
                            Some("page") => ColumnKind::PageLink,
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
                    ColumnKind::Number => out.push_str("    type: number\n"),
                    ColumnKind::Date => out.push_str("    type: date\n"),
                    ColumnKind::Url => out.push_str("    type: url\n"),
                    ColumnKind::PageLink => out.push_str("    type: page\n"),
                    ColumnKind::Select { options } => {
                        out.push_str("    type: select\n");
                        let opts = options.iter().map(|o| yaml_scalar(o)).collect::<Vec<_>>().join(", ");
                        out.push_str(&format!("    options: [{}]\n", opts));
                    }
                    ColumnKind::MultiSelect { options } => {
                        out.push_str("    type: multiselect\n");
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

    /// Adiciona `option` à lista de opções da coluna `idx`, se ela for
    /// `Select`/`MultiSelect` e a opção ainda não existir. Usado tanto pela
    /// criação inline de tag numa célula quanto pelo editor de opções do
    /// modal de configuração de coluna.
    pub fn add_column_option(&mut self, idx: usize, option: String) {
        if option.trim().is_empty() {
            return;
        }
        if let Some(c) = self.columns.get_mut(idx) {
            let options = match &mut c.kind {
                ColumnKind::Select { options } | ColumnKind::MultiSelect { options } => options,
                _ => return,
            };
            if !options.iter().any(|o| o == &option) {
                options.push(option);
            }
        }
    }

    /// Remove `option` da lista de opções da coluna `idx` e de toda célula
    /// que a referencia (pra `Select`, limpa a célula se era o valor
    /// selecionado; pra `MultiSelect`, remove só essa tag da lista).
    pub fn remove_column_option(&mut self, idx: usize, option: &str) {
        let Some(c) = self.columns.get_mut(idx) else { return };
        match &mut c.kind {
            ColumnKind::Select { options } => {
                options.retain(|o| o != option);
                for row in &mut self.rows {
                    if let Some(cell) = row.get_mut(idx) {
                        if cell == option {
                            cell.clear();
                        }
                    }
                }
            }
            ColumnKind::MultiSelect { options } => {
                options.retain(|o| o != option);
                for row in &mut self.rows {
                    if let Some(cell) = row.get_mut(idx) {
                        let remaining: Vec<&str> = cell.split(", ").filter(|t| !t.is_empty() && *t != option).collect();
                        *cell = remaining.join(", ");
                    }
                }
            }
            _ => {}
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

{{ type: "kanban" }}
columns: [Backlog, Todo, Done]
items:
  - title: Tarefa 1
    column: Backlog
  - title: Tarefa 2
    column: Todo
  - title: Tarefa 3
    column: Done
{{ /kanban }}

## Calendar Embed

{{ type: "calendar" }}
entries:
  - date: "2026-08-06"
    title: Revisão de código
  - date: "2026-08-07"
    title: Deploy produção
  - date: "2026-08-08"
    title: Retrospectiva sprint
{{ /calendar }}

## Table Embed

{{ type: "table" }}
| Tarefa | Status | Prioridade |
| ------ | ------ | ---------- |
| API    | done   | alta       |
| UI     | doing  | media      |
| Testes | todo   | alta       |
{{ /table }}

Acima do embed você pode ter texto normal. Abaixo também.
"#;

    #[test]
    fn join_inserts_missing_newline_before_next_segment() {
        // Regressão: um trecho de markdown editado (ex: vindo de
        // html_to_markdown, que não preserva o \n final) não pode grudar no
        // próximo segmento — isso corrompe o arquivo salvo.
        let segments = vec![
            DocSegment::Markdown("## Kanban Embed".to_string()), // sem \n final
            DocSegment::Embed(EmbedData::Kanban(KanbanEmbedData {
                columns: vec!["Backlog".into()],
                items: vec![],
            })),
        ];
        let joined = join(&segments);
        assert!(
            joined.starts_with("## Kanban Embed\n{{ type: \"kanban\" }}"),
            "esperava quebra de linha entre o heading e o wrapper, ficou: {joined:?}"
        );
    }

    #[test]
    fn join_does_not_duplicate_existing_newline() {
        let segments = vec![
            DocSegment::Markdown("texto\n\n".to_string()),
            DocSegment::Embed(EmbedData::Calendar(CalendarEmbedData { entries: vec![] })),
        ];
        let joined = join(&segments);
        assert!(joined.starts_with("texto\n\n{{ type: \"calendar\" }}"));
    }

    #[test]
    fn open_close_tag_tolerate_spacing() {
        assert_eq!(parse_open_tag("{{ type: \"kanban\" }}"), Some(EmbedKind::Kanban));
        assert_eq!(parse_open_tag("{{type:\"kanban\"}}"), Some(EmbedKind::Kanban));
        assert_eq!(parse_open_tag("  {{  type:   \"table\"  }}  "), Some(EmbedKind::Table));
        assert_eq!(parse_open_tag("{{ type: \"desenho\" }}"), None);
        assert_eq!(parse_open_tag("texto normal"), None);

        assert!(parse_close_tag("{{ /kanban }}", EmbedKind::Kanban));
        assert!(parse_close_tag("{{/kanban}}", EmbedKind::Kanban));
        assert!(!parse_close_tag("{{ /calendar }}", EmbedKind::Kanban));
        assert!(!parse_close_tag("{{ type: \"kanban\" }}", EmbedKind::Kanban));
    }

    #[test]
    fn unclosed_wrapper_consumes_to_end_without_panic() {
        let body = "texto\n\n{{ type: \"kanban\" }}\ncolumns: [A]\nitems: []\n";
        let segments = segment(body);
        assert_eq!(segments.len(), 2);
        assert!(matches!(segments[1], DocSegment::Embed(EmbedData::Kanban(_))));
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
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-06"));
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
    fn table_new_column_kinds_roundtrip() {
        let data = TableEmbedData {
            columns: vec![
                TableColumn { name: "Tags".into(), kind: ColumnKind::MultiSelect { options: vec!["urgente".into(), "bug".into()] } },
                TableColumn { name: "Estimativa".into(), kind: ColumnKind::Number },
                TableColumn { name: "Prazo".into(), kind: ColumnKind::Date },
                TableColumn { name: "Link".into(), kind: ColumnKind::Url },
                TableColumn { name: "Relacionado".into(), kind: ColumnKind::PageLink },
            ],
            rows: vec![vec![
                "urgente, bug".into(),
                "8".into(),
                "2026-08-10".into(),
                "https://example.com".into(),
                "pages/kanban-projeto.md".into(),
            ]],
        };
        let fence_body = data.to_fence_body();
        let reparsed = TableEmbedData::parse(&fence_body);
        assert_eq!(reparsed.columns[0].kind, ColumnKind::MultiSelect { options: vec!["urgente".into(), "bug".into()] });
        assert_eq!(reparsed.columns[1].kind, ColumnKind::Number);
        assert_eq!(reparsed.columns[2].kind, ColumnKind::Date);
        assert_eq!(reparsed.columns[3].kind, ColumnKind::Url);
        assert_eq!(reparsed.columns[4].kind, ColumnKind::PageLink);
        assert_eq!(reparsed.rows, data.rows);
    }

    #[test]
    fn table_add_remove_column_option() {
        let mut data = TableEmbedData {
            columns: vec![TableColumn { name: "Tags".into(), kind: ColumnKind::MultiSelect { options: vec![] } }],
            rows: vec![vec!["".into()]],
        };
        data.add_column_option(0, "urgente".into());
        data.add_column_option(0, "bug".into());
        data.add_column_option(0, "urgente".into()); // duplicata ignorada
        assert_eq!(data.columns[0].kind, ColumnKind::MultiSelect { options: vec!["urgente".into(), "bug".into()] });

        data.set_cell(0, 0, "urgente, bug".into());
        data.remove_column_option(0, "urgente");
        assert_eq!(data.columns[0].kind, ColumnKind::MultiSelect { options: vec!["bug".into()] });
        assert_eq!(data.rows[0][0], "bug");

        let mut select_data = TableEmbedData {
            columns: vec![TableColumn { name: "Status".into(), kind: ColumnKind::Select { options: vec!["todo".into(), "done".into()] } }],
            rows: vec![vec!["todo".into()]],
        };
        select_data.remove_column_option(0, "todo");
        assert_eq!(select_data.columns[0].kind, ColumnKind::Select { options: vec!["done".into()] });
        assert_eq!(select_data.rows[0][0], "", "célula devia ser limpa ao remover a opção selecionada");
    }

    #[test]
    fn exemplos_embeds_vault_file_parses() {
        // Regressão de sincronia: a página de demo do vault precisa parsear
        // com a sintaxe atual do wrapper, incluindo os campos ricos do
        // card (description/tags/due/checklist) escritos à mão.
        let raw = include_str!("../../VaultAnotadinho/pages/exemplos-embeds.md");
        let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(raw);
        let segments = segment(body);

        let kinds: Vec<Option<EmbedKind>> = segments
            .iter()
            .map(|s| match s {
                DocSegment::Embed(d) => Some(d.kind()),
                DocSegment::Markdown(_) => None,
            })
            .collect();
        assert_eq!(
            kinds,
            vec![None, Some(EmbedKind::Kanban), None, Some(EmbedKind::Calendar), None, Some(EmbedKind::Table), None]
        );

        let DocSegment::Embed(EmbedData::Kanban(kanban)) = &segments[1] else {
            panic!("esperava embed kanban");
        };
        assert_eq!(kanban.columns, vec!["Backlog", "Todo", "Done"]);
        assert_eq!(kanban.items.len(), 3);
        let card = &kanban.items[0];
        assert_eq!(card.title, "Tarefa 1");
        assert!(card.description.is_some());
        assert_eq!(card.tags, vec!["urgente", "bug"]);
        assert_eq!(card.due.as_deref(), Some("2026-08-10"));
        assert_eq!(card.checklist.len(), 2);
        assert!(!card.checklist[0].done);
        assert!(card.checklist[1].done);

        let DocSegment::Embed(EmbedData::Calendar(calendar)) = &segments[3] else {
            panic!("esperava embed calendar");
        };
        assert_eq!(calendar.entries.len(), 5);
        assert_eq!(calendar.entries[0].date.as_deref(), Some("2026-08-06"));
        assert_eq!(calendar.entries[0].tag.as_deref(), Some("urgente"));
        let ranged = calendar.entries.iter().find(|e| e.end_date.is_some()).expect("esperava 1 evento com end_date");
        assert_eq!(ranged.date.as_deref(), Some("2026-08-10"));
        assert_eq!(ranged.end_date.as_deref(), Some("2026-08-14"));
        assert_eq!(ranged.tag.as_deref(), Some("infra"));
        let timed = calendar.entries.iter().find(|e| e.start_time.is_some()).expect("esperava 1 evento com horário");
        assert_eq!(timed.start_time.as_deref(), Some("14:30"));
        assert_eq!(timed.end_time.as_deref(), Some("15:15"));
        let unscheduled = calendar.entries.iter().find(|e| e.date.is_none()).expect("esperava 1 evento sem data");
        assert_eq!(unscheduled.title, "Ligar pro fornecedor");

        let DocSegment::Embed(EmbedData::Table(table)) = &segments[5] else {
            panic!("esperava embed table");
        };
        assert_eq!(table.rows.len(), 3);
        let kinds: Vec<&ColumnKind> = table.columns.iter().map(|c| &c.kind).collect();
        assert_eq!(kinds[0], &ColumnKind::Text);
        assert_eq!(kinds[1], &ColumnKind::Select { options: vec!["todo".into(), "doing".into(), "done".into()] });
        assert_eq!(kinds[2], &ColumnKind::MultiSelect { options: vec!["urgente".into(), "bug".into(), "infra".into()] });
        assert_eq!(kinds[3], &ColumnKind::Number);
        assert_eq!(kinds[4], &ColumnKind::Date);
        assert_eq!(kinds[5], &ColumnKind::Url);
        assert_eq!(kinds[6], &ColumnKind::PageLink);
        assert_eq!(table.rows[0][2], "infra");
        assert_eq!(table.rows[1][2], "urgente, bug");
        assert_eq!(table.rows[0][6], "pages/kanban-projeto.md");
    }

    #[test]
    fn plain_code_fence_is_not_treated_as_embed() {
        let body = "texto\n\n```rust\nfn main() {}\n```\n\nmais texto\n";
        let segments = segment(body);
        assert!(segments.iter().all(|s| matches!(s, DocSegment::Markdown(_))));
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
        assert!(data.items[0].description.is_none());
        assert!(data.items[0].tags.is_empty());
    }

    #[test]
    fn kanban_edit_and_remove_card() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![KanbanCard { title: "X".into(), column: "Backlog".into(), ..Default::default() }],
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
                KanbanCard { title: "A".into(), column: "Backlog".into(), ..Default::default() },
                KanbanCard { title: "B".into(), column: "Done".into(), ..Default::default() },
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
                KanbanCard { title: "A".into(), column: "Backlog".into(), ..Default::default() },
                KanbanCard { title: "B".into(), column: "Done".into(), ..Default::default() },
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
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-07".into()),
            title: "Revisão adiada".into(),
            end_date: None,
            tag: None,
            ..Default::default()
        });
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-07"));
        assert_eq!(data.entries[0].title, "Revisão adiada");
        data.remove_entry(0);
        assert!(data.entries.is_empty());
    }

    #[test]
    fn calendar_entry_backward_compat_without_end_date_or_tag() {
        // Entradas antigas (antes deste ciclo) não tinham end_date/tag —
        // precisa continuar parseando.
        let data = CalendarEmbedData::parse("entries:\n- date: '2026-08-06'\n  title: Revisão\n");
        assert_eq!(data.entries.len(), 1);
        assert_eq!(data.entries[0].end_date, None);
        assert_eq!(data.entries[0].tag, None);
    }

    #[test]
    fn calendar_entry_range_and_tag_roundtrip() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Sprint".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Sprint".into(),
            end_date: Some("2026-08-10".into()),
            tag: Some("urgente".into()),
            ..Default::default()
        });
        let fence_body = data.to_fence_body();
        let reparsed = CalendarEmbedData::parse(&fence_body);
        assert_eq!(reparsed.entries[0].end_date.as_deref(), Some("2026-08-10"));
        assert_eq!(reparsed.entries[0].tag.as_deref(), Some("urgente"));
    }

    #[test]
    fn calendar_move_entry_preserves_duration() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Sprint".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Sprint".into(),
            end_date: Some("2026-08-08".into()), // 2 dias de duração
            tag: None,
            ..Default::default()
        });
        data.move_entry(0, "2026-08-20".into());
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-20"));
        assert_eq!(data.entries[0].end_date.as_deref(), Some("2026-08-22"));
    }

    #[test]
    fn calendar_entry_start_end_time_roundtrip() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Reunião".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Reunião".into(),
            start_time: Some("09:30".into()),
            end_time: Some("10:15".into()),
            ..Default::default()
        });
        let fence_body = data.to_fence_body();
        let reparsed = CalendarEmbedData::parse(&fence_body);
        assert_eq!(reparsed.entries[0].start_time.as_deref(), Some("09:30"));
        assert_eq!(reparsed.entries[0].end_time.as_deref(), Some("10:15"));
    }

    #[test]
    fn calendar_entry_backward_compat_without_times() {
        let data = CalendarEmbedData::parse("entries:\n- date: '2026-08-06'\n  title: Sem horário\n");
        assert_eq!(data.entries[0].start_time, None);
        assert_eq!(data.entries[0].end_time, None);
    }

    #[test]
    fn calendar_move_entry_single_day_has_no_end_date() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Reunião".into());
        data.move_entry(0, "2026-08-15".into());
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-15"));
        assert_eq!(data.entries[0].end_date, None);
    }

    #[test]
    fn calendar_add_entry_timed() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "09:30".into());
        assert_eq!(data.entries[0].start_time.as_deref(), Some("09:00"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("09:30"));
    }

    #[test]
    fn calendar_move_entry_time_preserves_duration_same_day() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "09:45".into());
        data.move_entry_time(0, "2026-08-06".into(), "14:00".into());
        assert_eq!(data.entries[0].start_time.as_deref(), Some("14:00"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("14:45"));
    }

    #[test]
    fn calendar_move_entry_time_across_days_keeps_duration() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "23:00".into(), "23:30".into());
        data.move_entry_time(0, "2026-08-07".into(), "10:15".into());
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-07"));
        assert_eq!(data.entries[0].start_time.as_deref(), Some("10:15"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("10:45"));
    }

    #[test]
    fn calendar_add_unscheduled_entry_has_no_date() {
        let mut data = CalendarEmbedData::default();
        data.add_unscheduled_entry("Sem data ainda".into());
        assert_eq!(data.entries[0].date, None);
        assert_eq!(data.entries[0].title, "Sem data ainda");
    }

    #[test]
    fn calendar_move_entry_assigns_date_to_unscheduled_entry() {
        // Arrastar da gaveta pra um dia: não tinha data antiga pra
        // calcular duração, só define a data mesmo.
        let mut data = CalendarEmbedData::default();
        data.add_unscheduled_entry("Sem data ainda".into());
        data.move_entry(0, "2026-08-12".into());
        assert_eq!(data.entries[0].date.as_deref(), Some("2026-08-12"));
        assert_eq!(data.entries[0].end_date, None);
    }

    #[test]
    fn calendar_unscheduled_entry_roundtrips_without_date_key() {
        let mut data = CalendarEmbedData::default();
        data.add_unscheduled_entry("Sem data ainda".into());
        let yaml = data.to_fence_body();
        assert!(!yaml.contains("date:"), "não deveria serializar `date:` pra evento sem data:\n{yaml}");
        let reparsed = CalendarEmbedData::parse(&yaml);
        assert_eq!(reparsed.entries[0].date, None);
        assert_eq!(reparsed.entries[0].title, "Sem data ainda");
    }

    #[test]
    fn calendar_move_entry_time_noop_without_start_time() {
        // Evento sem horário (dia inteiro) — move_entry_time só define o
        // novo start_time/date, não deveria inventar um end_time do nada.
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Dia inteiro".into());
        data.move_entry_time(0, "2026-08-06".into(), "10:00".into());
        assert_eq!(data.entries[0].start_time.as_deref(), Some("10:00"));
        assert_eq!(data.entries[0].end_time, None);
    }

    #[test]
    fn calendar_resize_entry_time_moves_start_edge() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "10:00".into());
        // Arrasta a borda de cima pra 09:30 — início muda, fim intacto.
        data.resize_entry_time(0, true, 9 * 60 + 30);
        assert_eq!(data.entries[0].start_time.as_deref(), Some("09:30"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("10:00"));
    }

    #[test]
    fn calendar_resize_entry_time_moves_end_edge() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "10:00".into());
        data.resize_entry_time(0, false, 11 * 60);
        assert_eq!(data.entries[0].start_time.as_deref(), Some("09:00"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("11:00"));
    }

    #[test]
    fn calendar_resize_entry_time_clamps_to_minimum_duration() {
        let mut data = CalendarEmbedData::default();
        data.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "10:00".into());
        // Tenta arrastar a borda de cima passando do fim — trava 15min
        // antes do fim em vez de inverter início/fim.
        data.resize_entry_time(0, true, 10 * 60 + 30);
        assert_eq!(data.entries[0].start_time.as_deref(), Some("09:45"));
        assert_eq!(data.entries[0].end_time.as_deref(), Some("10:00"));

        let mut data2 = CalendarEmbedData::default();
        data2.add_entry_timed("2026-08-06".into(), "Reunião".into(), "09:00".into(), "10:00".into());
        // Tenta arrastar a borda de baixo passando do início.
        data2.resize_entry_time(0, false, 8 * 60 + 30);
        assert_eq!(data2.entries[0].start_time.as_deref(), Some("09:00"));
        assert_eq!(data2.entries[0].end_time.as_deref(), Some("09:15"));
    }

    #[test]
    fn calendar_resize_entry_time_noop_without_start_time() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Dia inteiro".into());
        data.resize_entry_time(0, true, 9 * 60);
        assert_eq!(data.entries[0].start_time, None);
        assert_eq!(data.entries[0].end_time, None);
    }

    #[test]
    fn kanban_item_with_hash_survives_roundtrip() {
        // Regressão do ciclo anterior: "#" precedido de espaço vira
        // comentário em YAML plano sem aspas — cortava o título e derrubava
        // o card pra coluna errada. Agora é serde_yaml derive puro, então
        // isso nunca mais deveria acontecer, pra NENHUM campo.
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
    fn kanban_card_rich_fields_roundtrip() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![],
        };
        data.add_card("Backlog".into(), "Card rico".into());
        data.update_card(0, KanbanCard {
            title: "Card rico".into(),
            column: "Backlog".into(),
            description: Some("Descrição com # e : e \"aspas\"".into()),
            tags: vec!["infra".into(), "urgente".into()],
            due: Some("2026-08-10".into()),
            checklist: vec![
                ChecklistItem { text: "Passo 1".into(), done: true },
                ChecklistItem { text: "Passo 2".into(), done: false },
            ],
            comments: vec![Comment { text: "Começando".into(), created: "2026-08-06".into() }],
            attachments: vec![Attachment { name: "diagrama.png".into(), path: "assets/diagrama.png".into() }],
        });

        let fence_body = data.to_fence_body();
        let reparsed = KanbanEmbedData::parse(&fence_body);
        let card = &reparsed.items[0];
        assert_eq!(card.description.as_deref(), Some("Descrição com # e : e \"aspas\""));
        assert_eq!(card.tags, vec!["infra", "urgente"]);
        assert_eq!(card.due.as_deref(), Some("2026-08-10"));
        assert_eq!(card.checklist.len(), 2);
        assert!(card.checklist[0].done);
        assert!(!card.checklist[1].done);
        assert_eq!(card.comments.len(), 1);
        assert_eq!(card.comments[0].text, "Começando");
        assert_eq!(card.attachments.len(), 1);
        assert_eq!(card.attachments[0].path, "assets/diagrama.png");
    }

    #[test]
    fn kanban_checklist_comment_attachment_mutators() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![KanbanCard { title: "X".into(), column: "Backlog".into(), ..Default::default() }],
        };
        data.add_checklist_item(0, "Fazer isso".into());
        assert_eq!(data.items[0].checklist.len(), 1);
        assert!(!data.items[0].checklist[0].done);
        data.toggle_checklist_item(0, 0);
        assert!(data.items[0].checklist[0].done);
        data.remove_checklist_item(0, 0);
        assert!(data.items[0].checklist.is_empty());

        data.add_comment(0, "Comentário".into(), "2026-08-06".into());
        assert_eq!(data.items[0].comments.len(), 1);

        data.add_attachment(0, "a.png".into(), "assets/a.png".into());
        assert_eq!(data.items[0].attachments.len(), 1);
        data.remove_attachment(0, 0);
        assert!(data.items[0].attachments.is_empty());
    }

    #[test]
    fn kanban_move_card_changes_column_appending_at_end() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into(), "Done".into()],
            items: vec![
                KanbanCard { title: "A".into(), column: "Backlog".into(), ..Default::default() },
                KanbanCard { title: "B".into(), column: "Done".into(), ..Default::default() },
            ],
        };
        data.move_card(0, "Done".into(), None);
        assert_eq!(data.items.len(), 2);
        assert_eq!(data.items[0].title, "B");
        assert_eq!(data.items[1].title, "A");
        assert_eq!(data.items[1].column, "Done");
    }

    #[test]
    fn kanban_move_card_reorders_within_same_column() {
        let mut data = KanbanEmbedData {
            columns: vec!["Backlog".into()],
            items: vec![
                KanbanCard { title: "A".into(), column: "Backlog".into(), ..Default::default() },
                KanbanCard { title: "B".into(), column: "Backlog".into(), ..Default::default() },
                KanbanCard { title: "C".into(), column: "Backlog".into(), ..Default::default() },
            ],
        };
        // Move "C" (idx 2) pra antes de "A" (idx 0, original).
        data.move_card(2, "Backlog".into(), Some(0));
        let titles: Vec<&str> = data.items.iter().map(|c| c.title.as_str()).collect();
        assert_eq!(titles, vec!["C", "A", "B"]);
    }

    #[test]
    fn kanban_roundtrip_reparse() {
        let data = KanbanEmbedData {
            columns: vec!["Backlog".into(), "Doing".into()],
            items: vec![KanbanCard { title: "X".into(), column: "Doing".into(), ..Default::default() }],
        };
        let fence_body = data.to_fence_body();
        let reparsed = KanbanEmbedData::parse(&fence_body);
        assert_eq!(reparsed.columns, data.columns);
        assert_eq!(reparsed.items.len(), 1);
        assert_eq!(reparsed.items[0].title, "X");
        assert_eq!(reparsed.items[0].column, "Doing");
    }
}
