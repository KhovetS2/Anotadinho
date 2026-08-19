//! Base extensível para embeds inline: blocos `{{ type: "kanban" }} ...
//! {{ /kanban }}` dentro de uma página comum, parseados em dados
//! estruturados. A UI renderiza cada um como componente Yew de verdade
//! (não texto cru); o CLI opera os mesmos dados sem abrir janela
//! nenhuma.
//!
//! Por que não usar fence markdown (` ```kanban ``` `)? Colide
//! semanticamente com blocos de código de verdade — alguém que quisesse
//! mostrar um trecho de código chamado "kanban" teria o mesmo tratamento.
//! O wrapper `{{ }}` não existe em CommonMark, então nunca conflita com
//! nada do markdown normal.
//!
//! Mora no `core` desde o ciclo 149 (antes era `ui/src/embed.rs`): tudo
//! aqui é lógica pura de parse/serialize, e o `anotadinho-cli` precisa
//! dela pra um agente headless conseguir ler e escrever embed sem montar
//! YAML na mão. O que depende de WASM (varredura do vault via IPC)
//! ficou na UI.
//!
//! Extensão: um novo tipo de embed é 1 variante em `EmbedKind` + 1 par
//! parse/serialize em `EmbedData` + 1 componente Yew (`components/embeds/`
//! na UI) + 1 braço no dispatcher `InlineEmbed`. O menu `/` do editor se
//! gera sozinho a partir de `EmbedKind::all()` (ciclo 148).

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
    /// `{{ type: "callout" }}` — caixa de destaque com corpo markdown.
    Callout,
    /// `{{ type: "columns" }}` — painéis markdown lado a lado.
    Columns,
    /// `{{ type: "gallery" }}` — grade de imagens do vault.
    Gallery,
    /// `{{ type: "query" }}` — consulta viva sobre o vault.
    Query,
    /// `{{ type: "timeline" }}` — cronograma de barras por intervalo.
    Timeline,
}

impl EmbedKind {
    /// Reconhece o `type` do wrapper (`kanban`/`calendar`/`table`).
    pub fn from_type_name(name: &str) -> Option<Self> {
        match name {
            "kanban" => Some(Self::Kanban),
            "calendar" => Some(Self::Calendar),
            "table" => Some(Self::Table),
            "callout" => Some(Self::Callout),
            "columns" => Some(Self::Columns),
            "gallery" => Some(Self::Gallery),
            "query" => Some(Self::Query),
            "timeline" => Some(Self::Timeline),
            _ => None,
        }
    }

    /// Nome usado no wrapper (`{{ type: "X" }}` / `{{ /X }}`).
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Kanban => "kanban",
            Self::Calendar => "calendar",
            Self::Table => "table",
            Self::Callout => "callout",
            Self::Columns => "columns",
            Self::Gallery => "gallery",
            Self::Query => "query",
            Self::Timeline => "timeline",
        }
    }

    /// Todos os tipos existentes, na ordem em que aparecem no menu `/`.
    /// Ponto único de verdade: quem quiser listar embeds (menu do
    /// editor, documentação, CLI) itera isto em vez de repetir a lista.
    pub fn all() -> &'static [EmbedKind] {
        &[Self::Kanban, Self::Calendar, Self::Table, Self::Callout, Self::Columns, Self::Gallery, Self::Query, Self::Timeline]
    }

    /// Nome de exibição (menu `/`, títulos de UI).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Kanban => "Kanban",
            Self::Calendar => "Calendário",
            Self::Table => "Tabela de Tarefas",
            Self::Callout => "Destaque",
            Self::Columns => "Colunas",
            Self::Gallery => "Galeria",
            Self::Query => "Consulta",
            Self::Timeline => "Cronograma",
        }
    }

    /// Descrição curta de uma linha (menu `/`).
    pub fn desc(&self) -> &'static str {
        match self {
            Self::Kanban => "Board com colunas e cards",
            Self::Calendar => "Eventos por data, mês/semana/dia",
            Self::Table => "Tabela com colunas tipadas",
            Self::Callout => "Caixa colorida com título e corpo",
            Self::Columns => "Blocos de texto lado a lado",
            Self::Gallery => "Grade de imagens do vault",
            Self::Query => "Lista viva de páginas por filtro",
            Self::Timeline => "Barras por intervalo de datas (Gantt)",
        }
    }

    /// Nome do ícone em `components/icon.rs`.
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Kanban => "columns",
            Self::Calendar => "calendar",
            Self::Table => "table",
            Self::Callout => "info",
            Self::Columns => "layout",
            Self::Gallery => "image",
            Self::Query => "search",
            Self::Timeline => "clock",
        }
    }

    /// Conteúdo inicial de um embed recém-inserido (o miolo entre os
    /// wrappers). `today` (`"YYYY-MM-DD"`) é recebido de fora em vez de
    /// consultado aqui de propósito: mantém a função pura — o relógio é
    /// `js_sys::Date` no WASM e `std::time` fora dele, e este código
    /// precisa rodar nos dois lados (ciclo 149 move o módulo pro
    /// `anotadinho-core`, alcançável pelo CLI).
    pub fn default_body(&self, today: &str) -> String {
        match self {
            Self::Kanban => {
                "columns:\n- Backlog\n- Todo\n- Done\nitems:\n- title: Novo card\n  column: Backlog"
                    .to_string()
            }
            Self::Calendar => format!("entries:\n- date: '{today}'\n  title: Novo evento"),
            Self::Table => "| Tarefa | Status | Prioridade |\n| ------ | ------ | ---------- |\n| Nova tarefa | todo | media |".to_string(),
            Self::Callout => "variant: info\ntitle: Nota\nbody: |\n  Escreva aqui.".to_string(),
            Self::Columns => "columns:\n- width: 1\n  body: |\n    Coluna da esquerda.\n- width: 1\n  body: |\n    Coluna da direita.".to_string(),
            // Nasce vazia: o botão "adicionar do vault" é o caminho,
            // e um item com path inventado só renderizaria placeholder.
            Self::Gallery => "columns: 3\nsize: md\nitems: []".to_string(),
            // Nasce mostrando o vault inteiro em lista: é o recorte que
            // sempre tem resultado, então dá pra ver o embed funcionando
            // antes de configurar qualquer filtro.
            Self::Query => "view: list\nlimit: 10".to_string(),
            Self::Timeline => {
                let end = crate::date_util::add_days(today, 6).unwrap_or_else(|| today.to_string());
                format!("scale: month\nitems:\n- title: Nova etapa\n  start: '{today}'\n  end: '{end}'")
            }
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
    /// Caixa de destaque.
    Callout(CalloutEmbedData),
    /// Painéis lado a lado.
    Columns(ColumnsEmbedData),
    /// Grade de imagens.
    Gallery(GalleryEmbedData),
    /// Consulta viva — o YAML do embed É a consulta.
    Query(crate::query::Query),
    /// Cronograma de barras.
    Timeline(TimelineEmbedData),
}

impl EmbedData {
    /// Parseia o conteúdo interno de um wrapper no tipo correspondente.
    pub fn parse(kind: EmbedKind, raw: &str) -> Self {
        match kind {
            EmbedKind::Kanban => EmbedData::Kanban(KanbanEmbedData::parse(raw)),
            EmbedKind::Calendar => EmbedData::Calendar(CalendarEmbedData::parse(raw)),
            EmbedKind::Table => EmbedData::Table(TableEmbedData::parse(raw)),
            EmbedKind::Callout => EmbedData::Callout(CalloutEmbedData::parse(raw)),
            EmbedKind::Columns => EmbedData::Columns(ColumnsEmbedData::parse(raw)),
            EmbedKind::Gallery => EmbedData::Gallery(GalleryEmbedData::parse(raw)),
            EmbedKind::Query => EmbedData::Query(serde_yaml::from_str(raw).unwrap_or_default()),
            EmbedKind::Timeline => EmbedData::Timeline(TimelineEmbedData::parse(raw)),
        }
    }

    /// Tipo deste embed.
    pub fn kind(&self) -> EmbedKind {
        match self {
            EmbedData::Kanban(_) => EmbedKind::Kanban,
            EmbedData::Calendar(_) => EmbedKind::Calendar,
            EmbedData::Table(_) => EmbedKind::Table,
            EmbedData::Callout(_) => EmbedKind::Callout,
            EmbedData::Columns(_) => EmbedKind::Columns,
            EmbedData::Gallery(_) => EmbedKind::Gallery,
            EmbedData::Query(_) => EmbedKind::Query,
            EmbedData::Timeline(_) => EmbedKind::Timeline,
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
            EmbedData::Callout(d) => d.to_fence_body(),
            EmbedData::Columns(d) => d.to_fence_body(),
            EmbedData::Gallery(d) => d.to_fence_body(),
            EmbedData::Query(q) => serde_yaml::to_string(q).unwrap_or_default(),
            EmbedData::Timeline(d) => d.to_fence_body(),
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
    /// Tags/cores do evento (mesma paleta de badge usada na tabela e no
    /// kanban). Múltiplas tags por evento — antes deste ciclo era só uma
    /// (`tag: string` singular no YAML).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Campo antigo (uma tag só) — só leitura, pra continuar parseando
    /// entradas de antes deste ciclo. Nunca é serializado de volta;
    /// `all_tags()` incorpora ele automaticamente enquanto o evento não
    /// for editado (a primeira edição migra pra `tags` de vez).
    #[serde(default, skip_serializing, rename = "tag")]
    pub legacy_tag: Option<String>,
    /// Horário de início (`"HH:MM"`). `None` = evento de dia inteiro (sem
    /// horário) — comportamento padrão, igual ao de antes deste campo
    /// existir.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<String>,
    /// Horário de fim (`"HH:MM"`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<String>,
    /// Path da página de origem — só populado em entradas sintéticas do
    /// modo Vault (`scan_vault_calendar_entries`), nunca serializado no
    /// YAML do embed. Entradas manuais não têm página de origem: a
    /// entrada É o evento, não uma referência a algo mais.
    #[serde(default, skip)]
    pub page_path: Option<String>,
}

/// Fonte dos eventos exibidos pelo embed `{{ type: "calendar" }}`.
/// `Manual` (padrão) usa `CalendarEmbedData::entries`, editável pelo
/// próprio embed. `Vault` escaneia o vault inteiro por `date::`/`time::`
/// no frontmatter (mesma fonte da página `type: calendar`) — somente
/// leitura aqui, clicar um evento abre a página de origem; editar
/// continua sendo feito na página, não no embed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalendarSource {
    /// Eventos vêm de `CalendarEmbedData::entries`, editáveis no próprio
    /// embed.
    #[default]
    Manual,
    /// Eventos vêm da varredura do vault, somente leitura.
    Vault,
}

fn is_manual_source(m: &CalendarSource) -> bool {
    *m == CalendarSource::Manual
}

impl CalendarEntry {
    /// Todas as tags do evento — usa `tags` se tiver alguma, senão cai
    /// pro campo antigo `legacy_tag` (evento de antes deste ciclo, ainda
    /// não editado). Ponto único de leitura pra UI não precisar saber da
    /// migração.
    pub fn all_tags(&self) -> Vec<String> {
        if !self.tags.is_empty() {
            self.tags.clone()
        } else if let Some(t) = &self.legacy_tag {
            vec![t.clone()]
        } else {
            Vec::new()
        }
    }
}

/// Dados de um embed calendar: lista de eventos.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalendarEmbedData {
    /// Eventos, na ordem em que aparecem no wrapper.
    #[serde(default)]
    pub entries: Vec<CalendarEntry>,
    /// Fonte dos eventos exibidos — ver `CalendarSource`.
    #[serde(default, skip_serializing_if = "is_manual_source")]
    pub mode: CalendarSource,
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

/// Variante visual do callout — define cor e ícone. Nomes escolhidos
/// pelo PAPEL (o que a caixa comunica), não pela cor: a paleta pode
/// mudar sem invalidar arquivo `.md` nenhum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CalloutVariant {
    /// Contexto neutro.
    #[default]
    Info,
    /// Algo concluído/validado.
    Success,
    /// Cuidado, mas não é erro.
    Warning,
    /// Erro, quebra, armadilha.
    Error,
    /// Sugestão, atalho, truque.
    Tip,
}

impl CalloutVariant {
    /// Todas as variantes, na ordem em que aparecem no seletor.
    pub fn all() -> &'static [CalloutVariant] {
        &[Self::Info, Self::Success, Self::Warning, Self::Error, Self::Tip]
    }

    /// Nome usado no YAML e no modificador BEM (`.callout--info`).
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "success",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Tip => "tip",
        }
    }

    /// Nome de exibição (seletor de variante).
    pub fn label(&self) -> &'static str {
        match self {
            Self::Info => "Info",
            Self::Success => "Sucesso",
            Self::Warning => "Atenção",
            Self::Error => "Erro",
            Self::Tip => "Dica",
        }
    }

    /// Ícone (nome em `components/icon.rs`).
    pub fn icon(&self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Success => "check",
            Self::Warning => "alert-triangle",
            Self::Error => "alert-circle",
            Self::Tip => "lightbulb",
        }
    }
}

/// Dados de um embed callout: uma caixa de destaque com título e corpo
/// markdown.
///
/// O corpo guarda MARKDOWN, não HTML — é o que mantém o `.md` no disco
/// legível e editável por fora (um agente via CLI, ou `git diff`).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct CalloutEmbedData {
    /// Cor/ícone da caixa.
    #[serde(default, deserialize_with = "deserialize_variant_lenient")]
    pub variant: CalloutVariant,
    /// Título mostrado no cabeçalho. Pode ser vazio.
    #[serde(default)]
    pub title: String,
    /// Se o corpo nasce recolhido.
    #[serde(default, skip_serializing_if = "is_false")]
    pub collapsed: bool,
    /// Corpo em markdown.
    #[serde(default)]
    pub body: String,
}

fn is_false(b: &bool) -> bool {
    !*b
}

/// Aceita QUALQUER string em `variant:`, caindo no default quando o
/// nome não é conhecido. Sem isso, um `variant: roxo` (versão futura,
/// erro de digitação, ou um agente escrevendo pelo CLI) derruba a
/// deserialização da struct INTEIRA — e o embed volta vazio, apagando
/// título e corpo do usuário na primeira regravação.
fn deserialize_variant_lenient<'de, D>(deserializer: D) -> Result<CalloutVariant, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let raw = String::deserialize(deserializer)?;
    Ok(CalloutVariant::all()
        .iter()
        .copied()
        .find(|v| v.slug() == raw.trim().to_lowercase())
        .unwrap_or_default())
}

impl CalloutEmbedData {
    fn parse(raw: &str) -> Self {
        serde_yaml::from_str(raw).unwrap_or_default()
    }

    fn to_fence_body(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Troca a variante.
    pub fn set_variant(&mut self, variant: CalloutVariant) {
        self.variant = variant;
    }

    /// Troca o título.
    pub fn set_title(&mut self, title: String) {
        self.title = title;
    }

    /// Troca o corpo markdown.
    pub fn set_body(&mut self, body: String) {
        self.body = body;
    }

    /// Recolhe/expande.
    pub fn toggle_collapsed(&mut self) {
        self.collapsed = !self.collapsed;
    }
}

/// Um painel do embed de colunas.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColumnPane {
    /// Largura relativa, em unidades de fração (`1fr`, `2fr`...).
    /// Inteiro de propósito: mantém o YAML legível pro agente
    /// (`width: 2`) e evita percentual que não fecha em 100.
    #[serde(default = "default_pane_width")]
    pub width: u8,
    /// Conteúdo do painel, em markdown.
    #[serde(default)]
    pub body: String,
}

fn default_pane_width() -> u8 {
    1
}

impl Default for ColumnPane {
    fn default() -> Self {
        Self { width: 1, body: String::new() }
    }
}

/// Dados de um embed columns: painéis markdown lado a lado.
///
/// Existe porque markdown é linear — tudo empilha numa coluna só. Uma
/// landing page ou painel (ciclo 160) precisa de conteúdo lado a lado
/// sem sair do arquivo `.md`.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct ColumnsEmbedData {
    /// Painéis, da esquerda pra direita.
    #[serde(default)]
    pub columns: Vec<ColumnPane>,
}

impl ColumnsEmbedData {
    /// Máximo de painéis. Acima disso cada um fica estreito demais pra
    /// caber texto legível na largura de uma página.
    pub const MAX_COLUMNS: usize = 4;

    fn parse(raw: &str) -> Self {
        let mut data: Self = serde_yaml::from_str(raw).unwrap_or_default();
        // Sem painel nenhum não há o que renderizar (e o embed ficaria
        // invisível, impossível de consertar pela interface).
        if data.columns.is_empty() {
            data.columns = vec![ColumnPane::default(), ColumnPane::default()];
        }
        data.columns.truncate(Self::MAX_COLUMNS);
        for pane in &mut data.columns {
            pane.width = pane.width.clamp(1, 6);
        }
        data
    }

    fn to_fence_body(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Adiciona um painel vazio no fim, respeitando `MAX_COLUMNS`.
    pub fn add_column(&mut self) {
        if self.columns.len() < Self::MAX_COLUMNS {
            self.columns.push(ColumnPane::default());
        }
    }

    /// Remove o painel `idx`. Nunca remove o último — um embed sem
    /// painel some da tela sem deixar como desfazer.
    pub fn remove_column(&mut self, idx: usize) {
        if self.columns.len() > 1 && idx < self.columns.len() {
            self.columns.remove(idx);
        }
    }

    /// Troca o markdown do painel `idx`.
    pub fn set_body(&mut self, idx: usize, body: String) {
        if let Some(pane) = self.columns.get_mut(idx) {
            pane.body = body;
        }
    }

    /// Soma `delta` à largura do painel `idx`, limitando entre 1 e 6.
    pub fn adjust_width(&mut self, idx: usize, delta: i8) {
        if let Some(pane) = self.columns.get_mut(idx) {
            pane.width = (pane.width as i16 + delta as i16).clamp(1, 6) as u8;
        }
    }

    /// `grid-template-columns` correspondente às larguras (ex:
    /// `"1fr 2fr"`).
    pub fn grid_template(&self) -> String {
        self.columns
            .iter()
            .map(|p| format!("{}fr", p.width))
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Altura das miniaturas da galeria.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GallerySize {
    /// Miniatura pequena (contato).
    Sm,
    /// Padrão.
    #[default]
    Md,
    /// Grande (detalhe).
    Lg,
}

impl GallerySize {
    /// Todos os tamanhos, do menor pro maior.
    pub fn all() -> &'static [GallerySize] {
        &[Self::Sm, Self::Md, Self::Lg]
    }

    /// Nome no YAML e no modificador BEM.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }

    /// Nome de exibição.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Sm => "P",
            Self::Md => "M",
            Self::Lg => "G",
        }
    }
}

/// Um item da galeria.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct GalleryItem {
    /// Path relativo ao vault (`assets/foto.png`) ou URL externa.
    pub path: String,
    /// Legenda mostrada abaixo da miniatura.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub caption: String,
}

/// Dados de um embed gallery: grade de imagens com legenda.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GalleryEmbedData {
    /// Quantas colunas na grade (1 a 6).
    #[serde(default = "default_gallery_columns")]
    pub columns: u8,
    /// Altura das miniaturas.
    #[serde(default)]
    pub size: GallerySize,
    /// Itens, na ordem em que aparecem.
    #[serde(default)]
    pub items: Vec<GalleryItem>,
}

fn default_gallery_columns() -> u8 {
    3
}

impl Default for GalleryEmbedData {
    fn default() -> Self {
        Self { columns: 3, size: GallerySize::default(), items: Vec::new() }
    }
}

impl GalleryEmbedData {
    fn parse(raw: &str) -> Self {
        let mut data: Self = serde_yaml::from_str(raw).unwrap_or_default();
        data.columns = data.columns.clamp(1, 6);
        data
    }

    fn to_fence_body(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Adiciona um item no fim.
    pub fn add_item(&mut self, path: String) {
        self.items.push(GalleryItem { path, caption: String::new() });
    }

    /// Remove o item `idx`.
    pub fn remove_item(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.items.remove(idx);
        }
    }

    /// Troca a legenda do item `idx`.
    pub fn set_caption(&mut self, idx: usize, caption: String) {
        if let Some(item) = self.items.get_mut(idx) {
            item.caption = caption;
        }
    }

    /// Move o item `idx` uma posição pra esquerda (`-1`) ou direita
    /// (`1`). Nas pontas não faz nada.
    pub fn move_item(&mut self, idx: usize, delta: i8) {
        let Some(target) = idx.checked_add_signed(delta as isize) else { return };
        if idx < self.items.len() && target < self.items.len() {
            self.items.swap(idx, target);
        }
    }

    /// Soma `delta` ao número de colunas, entre 1 e 6.
    pub fn adjust_columns(&mut self, delta: i8) {
        self.columns = (self.columns as i16 + delta as i16).clamp(1, 6) as u8;
    }

    /// Troca o tamanho das miniaturas.
    pub fn set_size(&mut self, size: GallerySize) {
        self.size = size;
    }
}

/// Escala do eixo de tempo do cronograma.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineScale {
    /// 7 dias.
    Week,
    /// ~5 semanas.
    #[default]
    Month,
    /// ~13 semanas.
    Quarter,
}

impl TimelineScale {
    /// Todas as escalas, da menor pra maior.
    pub fn all() -> &'static [TimelineScale] {
        &[Self::Week, Self::Month, Self::Quarter]
    }

    /// Nome no YAML e no modificador BEM.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
        }
    }

    /// Nome de exibição.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Week => "Semana",
            Self::Month => "Mês",
            Self::Quarter => "Trimestre",
        }
    }

    /// Quantos dias a janela cobre.
    pub fn days(&self) -> i64 {
        match self {
            Self::Week => 7,
            Self::Month => 35,
            Self::Quarter => 91,
        }
    }
}

/// Fonte dos itens do cronograma. `Manual` guarda os itens no próprio
/// wrapper; `Vault` monta a partir do frontmatter das páginas (somente
/// leitura) — mesma divisão que o calendário inline já usa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimelineSource {
    /// Itens do próprio embed.
    #[default]
    Manual,
    /// Itens vindos do vault.
    Vault,
}

fn is_manual_timeline(s: &TimelineSource) -> bool {
    *s == TimelineSource::Manual
}

/// Uma barra do cronograma.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TimelineItem {
    /// Rótulo da barra.
    pub title: String,
    /// Início (`YYYY-MM-DD`). `None` = item sem data, fica na gaveta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    /// Fim inclusivo. `None` = barra de 1 dia.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<String>,
    /// Tags — a primeira dá a cor da barra (`badge_class`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Página de origem, só em itens do modo Vault (nunca serializado).
    #[serde(default, skip)]
    pub page: Option<String>,
}

/// Dados de um embed timeline.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct TimelineEmbedData {
    /// Escala do eixo.
    #[serde(default)]
    pub scale: TimelineScale,
    /// De onde vêm os itens.
    #[serde(default, skip_serializing_if = "is_manual_timeline")]
    pub source: TimelineSource,
    /// Itens (só no modo Manual).
    #[serde(default)]
    pub items: Vec<TimelineItem>,
}

/// Posição e largura de uma barra dentro da janela visível, em
/// porcentagem — recortada nas bordas quando o intervalo passa da
/// janela. `None` quando o item não aparece na janela (ou não tem
/// início).
///
/// Função pura de propósito: é a única aritmética não trivial do
/// cronograma, e é onde erro de um dia aparece.
pub fn bar_span(
    start: Option<&str>,
    end: Option<&str>,
    window_start: &str,
    window_days: i64,
) -> Option<(f64, f64)> {
    let start = start?;
    // Fim ausente = barra de 1 dia (o próprio início).
    let end = end.unwrap_or(start);
    let (from, to) = if crate::date_util::days_between(start, end).unwrap_or(0) < 0 {
        // Intervalo invertido no arquivo: trata como 1 dia em vez de
        // desenhar barra de largura negativa.
        (start, start)
    } else {
        (start, end)
    };

    let offset = crate::date_util::days_between(window_start, from)?;
    // `days_between` é exclusivo no fim; a barra inclui o dia final.
    let length = crate::date_util::days_between(from, to)? + 1;

    let visible_start = offset.max(0);
    let visible_end = (offset + length).min(window_days);
    if visible_end <= visible_start {
        return None;
    }

    let pct = 100.0 / window_days as f64;
    Some((visible_start as f64 * pct, (visible_end - visible_start) as f64 * pct))
}

impl TimelineEmbedData {
    fn parse(raw: &str) -> Self {
        serde_yaml::from_str(raw).unwrap_or_default()
    }

    fn to_fence_body(&self) -> String {
        serde_yaml::to_string(self).unwrap_or_default()
    }

    /// Adiciona um item novo com intervalo.
    pub fn add_item(&mut self, title: String, start: String, end: String) {
        self.items.push(TimelineItem {
            title,
            start: Some(start),
            end: Some(end),
            ..Default::default()
        });
    }

    /// Adiciona um item sem data — fica na gaveta até ser arrastado
    /// pra grade.
    pub fn add_unscheduled(&mut self, title: String) {
        self.items.push(TimelineItem { title, ..Default::default() });
    }

    /// Remove o item `idx`.
    pub fn remove_item(&mut self, idx: usize) {
        if idx < self.items.len() {
            self.items.remove(idx);
        }
    }

    /// Substitui o item `idx` inteiro.
    pub fn update_item(&mut self, idx: usize, item: TimelineItem) {
        if let Some(slot) = self.items.get_mut(idx) {
            *slot = item;
        }
    }

    /// Desloca o item `idx` pra começar em `new_start`, preservando a
    /// duração. Item que ainda não tinha data só ganha o início (é o
    /// caminho de arrastar da gaveta pra grade).
    pub fn move_item(&mut self, idx: usize, new_start: String) {
        let Some(item) = self.items.get_mut(idx) else { return };
        if let (Some(old_start), Some(end)) = (&item.start, &item.end) {
            let duration = crate::date_util::days_between(old_start, end).unwrap_or(0);
            if let Some(new_end) = crate::date_util::add_days(&new_start, duration) {
                item.end = Some(new_end);
            }
        }
        item.start = Some(new_start);
    }

    /// Redimensiona o item `idx` arrastando a borda inicial
    /// (`is_start_edge`) ou final. Nunca inverte: a borda arrastada
    /// para no dia da borda oposta.
    pub fn resize_item(&mut self, idx: usize, is_start_edge: bool, new_date: String) {
        let Some(item) = self.items.get_mut(idx) else { return };
        let Some(start) = item.start.clone() else { return };
        let end = item.end.clone().unwrap_or_else(|| start.clone());
        if is_start_edge {
            if crate::date_util::days_between(&new_date, &end).unwrap_or(0) >= 0 {
                item.start = Some(new_date);
            } else {
                item.start = Some(end.clone());
            }
            item.end = Some(end);
        } else if crate::date_util::days_between(&start, &new_date).unwrap_or(0) >= 0 {
            item.end = Some(new_date);
        } else {
            item.end = Some(start);
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

    #[test]
    fn all_kinds_round_trip_pelo_nome_do_tipo() {
        for kind in EmbedKind::all() {
            assert_eq!(EmbedKind::from_type_name(kind.type_name()), Some(*kind));
        }
    }

    #[test]
    fn all_kinds_tem_metadados_preenchidos() {
        for kind in EmbedKind::all() {
            assert!(!kind.label().is_empty(), "{} sem label", kind.type_name());
            assert!(!kind.desc().is_empty(), "{} sem desc", kind.type_name());
            assert!(!kind.icon().is_empty(), "{} sem icon", kind.type_name());
        }
    }

    #[test]
    fn default_body_de_todo_kind_parseia_em_dados_nao_vazios() {
        for kind in EmbedKind::all() {
            let body = kind.default_body("2026-08-19");
            let data = EmbedData::parse(*kind, &body);
            assert_eq!(data.kind(), *kind);
            match &data {
                EmbedData::Kanban(d) => {
                    assert!(!d.columns.is_empty());
                    assert!(!d.items.is_empty());
                }
                EmbedData::Calendar(d) => {
                    assert_eq!(d.entries.len(), 1);
                    assert_eq!(d.entries[0].date.as_deref(), Some("2026-08-19"));
                }
                EmbedData::Table(d) => {
                    assert!(!d.columns.is_empty());
                    assert!(!d.rows.is_empty());
                }
                EmbedData::Callout(d) => {
                    assert!(!d.title.is_empty());
                    assert!(!d.body.is_empty());
                }
                EmbedData::Columns(d) => {
                    assert_eq!(d.columns.len(), 2);
                    assert!(d.columns.iter().all(|p| !p.body.is_empty()));
                }
                // Galeria nasce VAZIA de propósito (ver `default_body`)
                // — o conteúdo entra pelo picker de assets.
                EmbedData::Gallery(d) => {
                    assert_eq!(d.columns, 3);
                    assert!(d.items.is_empty());
                }
                EmbedData::Query(q) => {
                    assert_eq!(q.limit, Some(10));
                }
                EmbedData::Timeline(d) => {
                    assert_eq!(d.items.len(), 1);
                    assert_eq!(d.items[0].start.as_deref(), Some("2026-08-19"));
                }
            }
        }
    }

    #[test]
    fn callout_roundtrip_com_corpo_multilinha() {
        let data = CalloutEmbedData {
            variant: CalloutVariant::Warning,
            title: "Cuidado: leia antes".to_string(),
            collapsed: true,
            // `:` e `#` no corpo já quebraram serialização montada à mão
            // (ciclo 064) — aqui o derive do serde escapa sozinho.
            body: "Linha 1: com dois pontos\n\n## Título\n\n- item\n".to_string(),
        };
        let text = EmbedData::Callout(data.clone()).to_fence_text();
        let segs = segment(&text);
        assert_eq!(segs.len(), 1);
        let DocSegment::Embed(EmbedData::Callout(back)) = &segs[0] else {
            panic!("esperava embed callout, veio {:?}", segs[0]);
        };
        assert_eq!(back, &data);
    }

    #[test]
    fn callout_vazio_parseia_com_defaults() {
        let d = CalloutEmbedData::parse("");
        assert_eq!(d.variant, CalloutVariant::Info);
        assert!(d.title.is_empty());
        assert!(d.body.is_empty());
        assert!(!d.collapsed);
    }

    #[test]
    fn callout_nao_serializa_collapsed_falso() {
        let d = CalloutEmbedData { title: "T".into(), ..Default::default() };
        let body = d.to_fence_body();
        assert!(!body.contains("collapsed"), "collapsed: false polui o arquivo à toa: {body}");
    }

    #[test]
    fn callout_variant_desconhecida_cai_no_default_sem_perder_o_resto() {
        // Regressão: variante inválida NÃO pode derrubar a struct
        // inteira — o embed voltaria vazio e a primeira regravação
        // apagaria título e corpo do usuário.
        let d = CalloutEmbedData::parse("variant: roxo\ntitle: T\nbody: corpo\n");
        assert_eq!(d.variant, CalloutVariant::Info);
        assert_eq!(d.title, "T");
        assert_eq!(d.body, "corpo");
    }

    #[test]
    fn callout_variant_slugs_batem_com_o_yaml() {
        for v in CalloutVariant::all() {
            let yaml = format!("variant: {}\n", v.slug());
            assert_eq!(CalloutEmbedData::parse(&yaml).variant, *v);
        }
    }

    #[test]
    fn columns_roundtrip_com_larguras_assimetricas() {
        let data = ColumnsEmbedData {
            columns: vec![
                ColumnPane { width: 2, body: "## Esquerda\n\ntexto: com dois pontos\n".into() },
                ColumnPane { width: 1, body: "- direita\n".into() },
            ],
        };
        let text = EmbedData::Columns(data.clone()).to_fence_text();
        let segs = segment(&text);
        let DocSegment::Embed(EmbedData::Columns(back)) = &segs[0] else {
            panic!("esperava embed columns");
        };
        assert_eq!(back, &data);
        assert_eq!(back.grid_template(), "2fr 1fr");
    }

    #[test]
    fn columns_sem_paineis_cai_em_duas_colunas() {
        let d = ColumnsEmbedData::parse("columns: []");
        assert_eq!(d.columns.len(), 2);
        assert_eq!(d.grid_template(), "1fr 1fr");
    }

    #[test]
    fn columns_respeita_o_maximo_e_o_minimo() {
        let mut d = ColumnsEmbedData::parse("");
        for _ in 0..10 {
            d.add_column();
        }
        assert_eq!(d.columns.len(), ColumnsEmbedData::MAX_COLUMNS);

        let mut only_one = ColumnsEmbedData { columns: vec![ColumnPane::default()] };
        only_one.remove_column(0);
        assert_eq!(only_one.columns.len(), 1, "remover o último painel apagaria o embed da tela");
    }

    #[test]
    fn columns_largura_fica_entre_um_e_seis() {
        let mut d = ColumnsEmbedData::parse("");
        d.adjust_width(0, -5);
        assert_eq!(d.columns[0].width, 1);
        d.adjust_width(0, 100);
        assert_eq!(d.columns[0].width, 6);
    }

    #[test]
    fn columns_com_painel_a_mais_no_arquivo_e_truncado() {
        let raw = "columns:\n- body: a\n- body: b\n- body: c\n- body: d\n- body: e\n";
        assert_eq!(ColumnsEmbedData::parse(raw).columns.len(), ColumnsEmbedData::MAX_COLUMNS);
    }

    #[test]
    fn gallery_roundtrip_com_legenda_com_pontuacao() {
        let data = GalleryEmbedData {
            columns: 2,
            size: GallerySize::Lg,
            items: vec![
                GalleryItem { path: "assets/a.png".into(), caption: "Antes: com dois pontos, vírgula".into() },
                GalleryItem { path: "https://exemplo/b.png".into(), caption: String::new() },
            ],
        };
        let text = EmbedData::Gallery(data.clone()).to_fence_text();
        let segs = segment(&text);
        let DocSegment::Embed(EmbedData::Gallery(back)) = &segs[0] else {
            panic!("esperava embed gallery");
        };
        assert_eq!(back, &data);
    }

    #[test]
    fn gallery_colunas_fora_do_intervalo_sao_normalizadas() {
        assert_eq!(GalleryEmbedData::parse("columns: 0\nitems: []").columns, 1);
        assert_eq!(GalleryEmbedData::parse("columns: 99\nitems: []").columns, 6);
        assert_eq!(GalleryEmbedData::parse("items: []").columns, 3);
    }

    #[test]
    fn gallery_move_item_nas_pontas_nao_faz_nada() {
        let mut d = GalleryEmbedData {
            items: vec![
                GalleryItem { path: "a".into(), ..Default::default() },
                GalleryItem { path: "b".into(), ..Default::default() },
            ],
            ..Default::default()
        };
        d.move_item(0, -1);
        assert_eq!(d.items[0].path, "a");
        d.move_item(1, 1);
        assert_eq!(d.items[1].path, "b");
        d.move_item(0, 1);
        assert_eq!(d.items[0].path, "b");
    }

    #[test]
    fn gallery_legenda_vazia_nao_e_serializada() {
        let d = GalleryEmbedData {
            items: vec![GalleryItem { path: "assets/a.png".into(), caption: String::new() }],
            ..Default::default()
        };
        assert!(!d.to_fence_body().contains("caption"));
    }

    #[test]
    fn query_embed_roundtrip_pelo_wrapper() {
        use crate::query::{Condition, Query, QueryOp, QueryView, Sort};
        let q = Query {
            from: Some("pages/specs".into()),
            conditions: vec![Condition { field: "status".into(), op: QueryOp::Eq, value: "backlog".into() }],
            sort: Some(Sort { field: "priority".into(), desc: true }),
            view: QueryView::Table,
            columns: vec!["status".into()],
            ..Default::default()
        };
        let text = EmbedData::Query(q.clone()).to_fence_text();
        let segs = segment(&text);
        let DocSegment::Embed(EmbedData::Query(back)) = &segs[0] else {
            panic!("esperava embed query");
        };
        assert_eq!(back, &q);
    }

    #[test]
    fn timeline_roundtrip_com_tags_e_escala() {
        let data = TimelineEmbedData {
            scale: TimelineScale::Quarter,
            source: TimelineSource::Manual,
            items: vec![TimelineItem {
                title: "Etapa: com dois pontos".into(),
                start: Some("2026-08-01".into()),
                end: Some("2026-08-20".into()),
                tags: vec!["infra".into()],
                page: None,
            }],
        };
        let text = EmbedData::Timeline(data.clone()).to_fence_text();
        let segs = segment(&text);
        let DocSegment::Embed(EmbedData::Timeline(back)) = &segs[0] else {
            panic!("esperava embed timeline");
        };
        assert_eq!(back, &data);
    }

    #[test]
    fn bar_span_posiciona_e_dimensiona_dentro_da_janela() {
        // Janela de 10 dias começando em 2026-08-01; barra do dia 1 ao 5
        // = 5 dias inclusivos = 50%, começando em 0%.
        let (off, len) = bar_span(Some("2026-08-01"), Some("2026-08-05"), "2026-08-01", 10).unwrap();
        assert!((off - 0.0).abs() < 0.001);
        assert!((len - 50.0).abs() < 0.001);

        // Barra de 1 dia no meio: 1 dia = 10%.
        let (off, len) = bar_span(Some("2026-08-06"), None, "2026-08-01", 10).unwrap();
        assert!((off - 50.0).abs() < 0.001);
        assert!((len - 10.0).abs() < 0.001);
    }

    #[test]
    fn bar_span_recorta_nas_bordas_da_janela() {
        // Começa antes da janela: recorta o começo.
        let (off, len) = bar_span(Some("2026-07-25"), Some("2026-08-02"), "2026-08-01", 10).unwrap();
        assert!((off - 0.0).abs() < 0.001);
        assert!((len - 20.0).abs() < 0.001);

        // Termina depois da janela: recorta o fim.
        let (off, len) = bar_span(Some("2026-08-09"), Some("2026-08-30"), "2026-08-01", 10).unwrap();
        assert!((off - 80.0).abs() < 0.001);
        assert!((len - 20.0).abs() < 0.001);
    }

    #[test]
    fn bar_span_fora_da_janela_ou_sem_inicio_e_none() {
        assert!(bar_span(Some("2026-09-01"), None, "2026-08-01", 10).is_none());
        assert!(bar_span(Some("2026-07-01"), Some("2026-07-05"), "2026-08-01", 10).is_none());
        assert!(bar_span(None, Some("2026-08-02"), "2026-08-01", 10).is_none());
    }

    #[test]
    fn bar_span_com_intervalo_invertido_vira_um_dia() {
        let (_, len) = bar_span(Some("2026-08-05"), Some("2026-08-01"), "2026-08-01", 10).unwrap();
        assert!((len - 10.0).abs() < 0.001, "intervalo invertido não pode virar barra negativa");
    }

    #[test]
    fn timeline_move_preserva_duracao_e_resize_nao_inverte() {
        let mut d = TimelineEmbedData {
            items: vec![TimelineItem {
                title: "X".into(),
                start: Some("2026-08-01".into()),
                end: Some("2026-08-05".into()),
                ..Default::default()
            }],
            ..Default::default()
        };
        d.move_item(0, "2026-08-10".into());
        assert_eq!(d.items[0].start.as_deref(), Some("2026-08-10"));
        assert_eq!(d.items[0].end.as_deref(), Some("2026-08-14"));

        // Arrastar a borda inicial pra depois do fim para NO fim.
        d.resize_item(0, true, "2026-08-20".into());
        assert_eq!(d.items[0].start.as_deref(), Some("2026-08-14"));

        // Arrastar a borda final pra antes do início para NO início.
        d.resize_item(0, false, "2026-08-01".into());
        assert_eq!(d.items[0].end.as_deref(), Some("2026-08-14"));
    }

    #[test]
    fn timeline_item_sem_data_ganha_inicio_ao_ser_movido() {
        let mut d = TimelineEmbedData::default();
        d.add_unscheduled("Sem data".into());
        d.move_item(0, "2026-08-09".into());
        assert_eq!(d.items[0].start.as_deref(), Some("2026-08-09"));
        assert_eq!(d.items[0].end, None, "item de 1 dia não precisa de fim explícito");
    }

    #[test]
    fn embed_com_default_body_sobrevive_a_segment_e_join() {
        for kind in EmbedKind::all() {
            let data = EmbedData::parse(*kind, &kind.default_body("2026-08-19"));
            let body = format!("antes\n\n{}\ndepois\n", data.to_fence_text());
            let segs = segment(&body);
            assert!(
                segs.iter().any(|s| matches!(s, DocSegment::Embed(e) if e.kind() == *kind)),
                "{} nao voltou como embed",
                kind.type_name()
            );
            assert_eq!(segment(&join(&segs)), segs);
        }
    }

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
            DocSegment::Embed(EmbedData::Calendar(CalendarEmbedData::default())),
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
        let raw = include_str!("../../../VaultAnotadinho/pages/exemplos-embeds.md");
        let (_, body) = crate::MarkdownCodec::split_frontmatter_text(raw);
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
        assert_eq!(calendar.entries[0].all_tags(), vec!["urgente".to_string()]);
        let ranged = calendar.entries.iter().find(|e| e.end_date.is_some()).expect("esperava 1 evento com end_date");
        assert_eq!(ranged.date.as_deref(), Some("2026-08-10"));
        assert_eq!(ranged.end_date.as_deref(), Some("2026-08-14"));
        assert_eq!(ranged.all_tags(), vec!["infra".to_string()]);
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
        assert_eq!(data.entries[0].all_tags(), Vec::<String>::new());
    }

    #[test]
    fn calendar_entry_legacy_single_tag_still_parses_via_all_tags() {
        // Formato antigo (uma tag só, chave `tag:` singular) continua
        // parseando — `all_tags()` incorpora ele automaticamente.
        let data = CalendarEmbedData::parse("entries:\n- date: '2026-08-06'\n  title: Revisão\n  tag: urgente\n");
        assert_eq!(data.entries[0].all_tags(), vec!["urgente".to_string()]);
        assert_eq!(data.entries[0].tags, Vec::<String>::new());
        assert_eq!(data.entries[0].legacy_tag.as_deref(), Some("urgente"));
    }

    #[test]
    fn calendar_entry_multiple_tags_roundtrip() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Sprint".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Sprint".into(),
            tags: vec!["urgente".into(), "infra".into()],
            ..Default::default()
        });
        let fence_body = data.to_fence_body();
        assert!(!fence_body.contains("tag:"), "não deveria sobrar a chave antiga `tag:` no YAML:\n{fence_body}");
        let reparsed = CalendarEmbedData::parse(&fence_body);
        assert_eq!(reparsed.entries[0].all_tags(), vec!["urgente".to_string(), "infra".to_string()]);
    }

    #[test]
    fn calendar_entry_range_and_tag_roundtrip() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Sprint".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Sprint".into(),
            end_date: Some("2026-08-10".into()),
            tags: vec!["urgente".into()],
            ..Default::default()
        });
        let fence_body = data.to_fence_body();
        let reparsed = CalendarEmbedData::parse(&fence_body);
        assert_eq!(reparsed.entries[0].end_date.as_deref(), Some("2026-08-10"));
        assert_eq!(reparsed.entries[0].all_tags(), vec!["urgente".to_string()]);
    }

    #[test]
    fn calendar_move_entry_preserves_duration() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "Sprint".into());
        data.update_entry(0, CalendarEntry {
            date: Some("2026-08-06".into()),
            title: "Sprint".into(),
            end_date: Some("2026-08-08".into()), // 2 dias de duração
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
    fn calendar_mode_defaults_to_manual_without_key() {
        // YAML de antes deste ciclo não tem `mode:` — precisa continuar
        // parseando como Manual (comportamento idêntico ao de sempre).
        let data = CalendarEmbedData::parse("entries:\n- date: '2026-08-06'\n  title: X\n");
        assert_eq!(data.mode, CalendarSource::Manual);
    }

    #[test]
    fn calendar_mode_manual_not_serialized() {
        let data = CalendarEmbedData::default();
        let fence_body = data.to_fence_body();
        assert!(!fence_body.contains("mode:"), "modo Manual (padrão) não deveria poluir o YAML:\n{fence_body}");
    }

    #[test]
    fn calendar_mode_vault_roundtrips() {
        let mut data = CalendarEmbedData::default();
        data.mode = CalendarSource::Vault;
        let fence_body = data.to_fence_body();
        assert!(fence_body.contains("mode: vault"));
        let reparsed = CalendarEmbedData::parse(&fence_body);
        assert_eq!(reparsed.mode, CalendarSource::Vault);
    }

    #[test]
    fn calendar_entry_page_path_never_serialized() {
        let mut data = CalendarEmbedData::default();
        data.add_entry("2026-08-06".into(), "X".into());
        data.entries[0].page_path = Some("pages/x.md".into());
        let fence_body = data.to_fence_body();
        assert!(!fence_body.contains("page_path"), "page_path é só em memória (modo Vault), nunca deveria ir pro YAML:\n{fence_body}");
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
