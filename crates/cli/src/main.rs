//! CLI headless do Anotadinho.
//!
//! Reaproveita os handlers de `anotadinho-ipc` (a mesma lógica que o
//! app Tauri expõe pro frontend Yew) sem nenhuma dependência de Tauri —
//! dá pra listar/ler/buscar/exportar o vault de um terminal comum, sem
//! a janela do app aberta. Pensado pra um agente (Claude Code ou
//! outro processo) conseguir consumir o vault programaticamente.

use anotadinho_core::embed::{self, DocSegment, EmbedData, EmbedKind};
use anotadinho_core::query::{Aggregate, AggregateOp, Condition, Query, QueryOp, Sort};
use anotadinho_ipc::{
    handle_create_page_from_template, handle_export_folder, handle_list_templates,
    handle_read_page, handle_read_page_versioned, handle_scan_vault, handle_search_content,
    handle_write_page_checked,
};
use clap::{Parser, Subcommand};
use std::io::Read;

#[derive(Parser)]
#[command(
    name = "anotadinho-cli",
    version,
    about = "Acesso headless ao vault do Anotadinho — sem precisar da janela Tauri"
)]
struct Cli {
    /// Path do vault (pasta raiz com pages/, journals/, templates/ etc).
    #[arg(long)]
    vault: String,

    /// Emite JSON em vez de texto legível (nos comandos de listagem).
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lista páginas em pages/ e journals/, com filtros opcionais.
    ListPages {
        /// Filtra por prefixo de path (ex: pages/specs).
        #[arg(long)]
        folder: Option<String>,
        /// Filtra por tag — repetível, todas devem estar presentes
        /// (AND). Lê o frontmatter de cada página candidata.
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Filtra por `status` do frontmatter (campo livre, ex: usado
        /// pelas specs do esquema de agent-os).
        #[arg(long)]
        status: Option<String>,
        /// Filtra por `priority` do frontmatter.
        #[arg(long)]
        priority: Option<String>,
    },
    /// Imprime o conteúdo bruto (.md) de uma página.
    Read {
        /// Path relativo ao vault (ex: pages/minha-nota.md).
        page_path: String,
    },
    /// Busca full-text (FTS5) no conteúdo das páginas.
    Search {
        /// Termo de busca.
        query: String,
    },
    /// Exporta o dump concatenado do vault, ou de uma pasta específica.
    Export {
        /// Pasta a exportar (ex: pages/specs). Omitido = vault inteiro.
        #[arg(long)]
        folder: Option<String>,
    },
    /// Lista templates em templates/.
    ListTemplates,
    /// Cria uma página nova a partir de um template, substituindo
    /// `{{title}}` (e `{{date}}`, ciclo 112) pelo título escolhido.
    NewFromTemplate {
        /// Path do template relativo ao vault (ex: templates/spec.md).
        template_path: String,
        /// Título da página nova.
        title: String,
    },
    /// Grava um campo do frontmatter, preservando o corpo intocado.
    /// `title`/`type`/`tags` (lista separada por vírgula) setam o
    /// campo tipado; qualquer outra chave vai pra `extra` (ciclo 098).
    SetProperty {
        /// Path relativo ao vault (ex: pages/specs/minha-spec.md).
        page_path: String,
        /// Nome do campo de frontmatter.
        key: String,
        /// Novo valor.
        value: String,
    },
    /// Lê e escreve os embeds inline (`{{ type: "..." }}`) de uma
    /// página, sem montar YAML na mão (ciclo 157).
    Embed {
        #[command(subcommand)]
        action: EmbedCommand,
    },
    /// Fica observando o vault e imprime uma linha JSON por mudança
    /// (ciclo 172) — pra um agente REAGIR em vez de ficar consultando
    /// de tempos em tempos.
    Watch {
        /// Só eventos de páginas sob este prefixo (ex: pages/specs).
        #[arg(long)]
        folder: Option<String>,
        /// Além do evento, lê a página alterada e emite o valor deste
        /// campo do frontmatter (ex: `status`).
        #[arg(long = "property")]
        property: Option<String>,
        /// Intervalo de checagem em milissegundos.
        #[arg(long, default_value = "500")]
        intervalo_ms: u64,
    },
    /// Executa uma consulta sobre o vault — o MESMO motor do embed
    /// `{{ type: "query" }}` (ciclo 158), pra o agente ver no terminal
    /// exatamente o recorte que o humano vê na página.
    Query {
        /// Prefixo de pasta (ex: pages/specs). Omitido = vault inteiro.
        #[arg(long)]
        from: Option<String>,
        /// Filtra por tag — repetível, todas precisam bater (AND).
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Condição sobre um campo, repetível. Formatos: `campo=valor`
        /// (é), `campo!=valor` (não é), `campo~valor` (contém),
        /// `campo?` (existe), `campo>valor` e `campo<valor`.
        #[arg(long = "where")]
        conditions: Vec<String>,
        /// Campo pelo qual ordenar.
        #[arg(long)]
        sort: Option<String>,
        /// Ordena decrescente (só com `--sort`).
        #[arg(long)]
        desc: bool,
        /// Máximo de resultados.
        #[arg(long)]
        limit: Option<usize>,
        /// Campos extras mostrados na saída legível (repetível).
        #[arg(long = "field")]
        fields: Vec<String>,
        /// Agrupa os resultados por um campo (ciclo 169).
        #[arg(long)]
        group_by: Option<String>,
        /// Agregado por grupo, repetível: `count`, `sum:campo`,
        /// `avg:campo`, `min:campo`, `max:campo`.
        #[arg(long = "aggregate")]
        aggregates: Vec<String>,
        /// Roda a consulta declarada num embed `query` de uma página,
        /// em vez de montar a consulta pelos argumentos. Formato:
        /// `<page_path>:<índice do embed>`.
        #[arg(long)]
        from_embed: Option<String>,
    },
}

/// Operações sobre os embeds de uma página. O índice é a POSIÇÃO ENTRE
/// OS EMBEDS (0, 1, 2...), não entre os segmentos de texto — é o que
/// `embed list` imprime.
#[derive(Subcommand)]
enum EmbedCommand {
    /// Lista os embeds da página: índice, tipo e um resumo.
    List {
        /// Path relativo ao vault.
        page_path: String,
    },
    /// Imprime o conteúdo de um embed (YAML pra quase todos os tipos,
    /// tabela markdown pro tipo `table`).
    Get {
        /// Path relativo ao vault.
        page_path: String,
        /// Índice do embed na página.
        index: usize,
    },
    /// Substitui o conteúdo de um embed pelo texto lido de `--file`
    /// (ou do stdin). O texto passa pelo parser do tipo antes de ser
    /// gravado — um agente nunca escreve direto no arquivo.
    Set {
        /// Path relativo ao vault.
        page_path: String,
        /// Índice do embed na página.
        index: usize,
        /// Arquivo com o conteúdo novo. Omitido = lê do stdin.
        #[arg(long)]
        file: Option<String>,
    },
    /// Adiciona um card num embed de kanban.
    AddCard {
        /// Path relativo ao vault.
        page_path: String,
        /// Índice do embed na página.
        index: usize,
        /// Coluna de destino.
        #[arg(long)]
        column: String,
        /// Título do card.
        #[arg(long)]
        title: String,
    },
    /// Adiciona uma linha num embed de tabela.
    AddRow {
        /// Path relativo ao vault.
        page_path: String,
        /// Índice do embed na página.
        index: usize,
        /// Células separadas por vírgula, na ordem das colunas.
        #[arg(long)]
        values: String,
    },
    /// Adiciona um evento num embed de calendário.
    AddEvent {
        /// Path relativo ao vault.
        page_path: String,
        /// Índice do embed na página.
        index: usize,
        /// Data `YYYY-MM-DD`.
        #[arg(long)]
        date: String,
        /// Título do evento.
        #[arg(long)]
        title: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(msg) = run(cli) {
        eprintln!("erro: {}", msg);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ListPages { folder, tags, status, priority } => {
            // Mesmo motor do embed de consulta e do subcomando `query`
            // (ciclo 158): antes esta função tinha o filtro próprio dela,
            // lendo página por página — duas implementações do mesmo
            // conceito, que divergiriam na primeira mudança.
            let mut conditions = Vec::new();
            if let Some(value) = status {
                conditions.push(Condition { field: "status".into(), op: QueryOp::Eq, value });
            }
            if let Some(value) = priority {
                conditions.push(Condition { field: "priority".into(), op: QueryOp::Eq, value });
            }
            let query = Query { from: folder, tags, conditions, ..Default::default() };
            let entries = handle_scan_vault(cli.vault.clone())?;
            let pages: Vec<anotadinho_ipc::PageMeta> = query
                .run(&entries)
                .into_iter()
                .map(|e| anotadinho_ipc::PageMeta {
                    // Título = nome do arquivo, NÃO o `title` do
                    // frontmatter: é o que `list-pages` sempre imprimiu,
                    // e trocar isso quebraria script de agente que já
                    // usa a saída. (`PageIndexEntry::title` prefere o
                    // frontmatter, que é o certo pra consulta — daí a
                    // diferença entre `list-pages` e `query`.)
                    title: std::path::Path::new(&e.path)
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_else(|| e.title.clone()),
                    path: e.path.clone(),
                    section: e.section.clone(),
                })
                .collect();
            if cli.json {
                print_json(&pages)?;
            } else {
                for p in pages {
                    println!("{}\t{}\t{}", p.section, p.title, p.path);
                }
            }
        }
        Command::Read { page_path } => {
            let content = handle_read_page(cli.vault, page_path)?;
            print!("{}", content);
        }
        Command::Search { query } => {
            let results = handle_search_content(cli.vault, query)?;
            if cli.json {
                print_json(&results)?;
            } else {
                for (path, snippet) in results {
                    println!("{}\t{}", path, snippet);
                }
            }
        }
        Command::Export { folder } => {
            let dump = handle_export_folder(cli.vault, folder.unwrap_or_default())?;
            print!("{}", dump);
        }
        Command::ListTemplates => {
            let templates = handle_list_templates(cli.vault)?;
            if cli.json {
                print_json(&templates)?;
            } else {
                for t in templates {
                    println!("{}\t{}", t.title, t.path);
                }
            }
        }
        Command::NewFromTemplate { template_path, title } => {
            let meta = handle_create_page_from_template(cli.vault, template_path, title, None)?;
            println!("{}", meta.path);
        }
        Command::SetProperty { page_path, key, value } => {
            // Lê com versão e devolve ela na gravação (ciclo 173): se o
            // app (ou outro agente) escreveu entre a leitura e a
            // escrita, falha em vez de passar por cima.
            let page = handle_read_page_versioned(cli.vault.clone(), page_path.clone())?;
            let updated =
                anotadinho_core::MarkdownCodec::set_frontmatter_field(&page.content, &key, &value)
                    .map_err(|e| e.to_string())?;
            handle_write_page_checked(cli.vault, page_path, updated, page.version)?;
        }
        Command::Query {
            from,
            tags,
            conditions,
            sort,
            desc,
            limit,
            fields,
            group_by,
            aggregates,
            from_embed,
        } => {
            let query = match &from_embed {
                Some(spec) => query_from_embed(&cli.vault, spec)?,
                None => {
                    let mut conds = Vec::new();
                    for raw in &conditions {
                        conds.push(parse_condition(raw)?);
                    }
                    let mut aggs = Vec::new();
                    for raw in &aggregates {
                        aggs.push(parse_aggregate(raw)?);
                    }
                    Query {
                        from,
                        tags,
                        conditions: conds,
                        sort: sort.map(|field| Sort { field, desc }),
                        limit,
                        columns: fields.clone(),
                        group_by,
                        aggregate: aggs,
                        ..Default::default()
                    }
                }
            };
            let entries = handle_scan_vault(cli.vault.clone())?;
            // Com agrupamento a saída legível ganha cabeçalho e rodapé
            // por grupo; o `--json` continua sendo a lista achatada, que
            // é o que um agente consome.
            if query.group_by.is_some() || !query.aggregate.is_empty() {
                if cli.json {
                    print_json(&query.run(&entries))?;
                } else {
                    for grupo in query.run_grouped(&entries) {
                        if !grupo.rotulo.is_empty() {
                            println!("# {} ({})", grupo.rotulo, grupo.itens.len());
                        }
                        for entry in &grupo.itens {
                            println!("{}\t{}", entry.path, entry.title);
                        }
                        for (rotulo, valor) in &grupo.agregados {
                            println!("  {rotulo}: {valor}");
                        }
                    }
                }
                return Ok(());
            }
            let results = query.run(&entries);
            if cli.json {
                print_json(&results)?;
            } else {
                let columns = if query.columns.is_empty() { &fields } else { &query.columns };
                for entry in results {
                    let extra: Vec<String> = columns
                        .iter()
                        .map(|c| entry.field(c).unwrap_or_default())
                        .collect();
                    if extra.is_empty() {
                        println!("{}\t{}", entry.path, entry.title);
                    } else {
                        println!("{}\t{}\t{}", entry.path, entry.title, extra.join("\t"));
                    }
                }
            }
        }
        Command::Watch { folder, property, intervalo_ms } => {
            let vault = anotadinho_vault::VaultIo::open(&cli.vault);
            let watcher = anotadinho_vault::VaultWatcher::start(vault.root().to_path_buf())
                .map_err(|e| e.to_string())?;
            // Ctrl+C encerra pelo SIGINT padrão (código 130, a
            // convenção do shell). Converter pra 0 exigiria uma
            // dependência só pra isso — e 130 é o que qualquer
            // `while read` já entende como "o produtor parou".
            loop {
                for evento in watcher.drain_events() {
                    if let Some(prefixo) = &folder {
                        if !evento.path.starts_with(prefixo.as_str()) {
                            continue;
                        }
                    }
                    let mut linha = serde_json::json!({
                        "path": evento.path,
                        "kind": evento.kind,
                        "ts": agora_unix(),
                    });
                    if let Some(campo) = &property {
                        // Lê o valor NOVO do campo — é o que fecha o
                        // caso "rode algo quando a spec virar
                        // in-progress" sem o agente reler o arquivo.
                        if evento.kind != "deleted" {
                            if let Ok(conteudo) =
                                handle_read_page(cli.vault.clone(), evento.path.clone())
                            {
                                let entry = anotadinho_core::PageIndexEntry::from_content(
                                    &evento.path,
                                    "",
                                    "",
                                    &conteudo,
                                );
                                linha[campo.as_str()] = serde_json::json!(entry.field(campo));
                            }
                        }
                    }
                    println!("{linha}");
                    use std::io::Write;
                    let _ = std::io::stdout().flush();
                }
                std::thread::sleep(std::time::Duration::from_millis(intervalo_ms));
            }
        }
        Command::Embed { action } => {
            let vault = cli.vault.clone();
            let json = cli.json;
            run_embed(&vault, json, action)?
        }
    }
    Ok(())
}

/// Estado de uma página aberta pra mexer nos embeds: o frontmatter cru
/// (preservado byte a byte) e os segmentos do corpo.
struct PageDoc {
    frontmatter: String,
    segments: Vec<DocSegment>,
    /// Versão do arquivo no momento da leitura (ciclo 173) — devolvida
    /// na gravação pra recusar escrita se alguém mexeu no meio.
    version: Option<String>,
}

impl PageDoc {
    fn load(vault: &str, page_path: &str) -> Result<Self, String> {
        let page = handle_read_page_versioned(vault.to_string(), page_path.to_string())?;
        let (frontmatter, body) =
            anotadinho_core::MarkdownCodec::split_frontmatter_text(&page.content);
        Ok(Self {
            frontmatter: frontmatter.to_string(),
            segments: embed::segment(body),
            version: page.version,
        })
    }

    /// Índices (no vetor de segmentos) dos segmentos que são embed, na
    /// ordem — o índice do usuário é a posição NESTA lista.
    fn embed_positions(&self) -> Vec<usize> {
        self.segments
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, DocSegment::Embed(_)))
            .map(|(i, _)| i)
            .collect()
    }

    fn embed_at(&self, index: usize) -> Result<(usize, &EmbedData), String> {
        let positions = self.embed_positions();
        let pos = *positions.get(index).ok_or_else(|| {
            format!(
                "índice {index} fora do intervalo: a página tem {} embed(s)",
                positions.len()
            )
        })?;
        match &self.segments[pos] {
            DocSegment::Embed(data) => Ok((pos, data)),
            DocSegment::Markdown(_) => unreachable!("posição veio de embed_positions"),
        }
    }

    /// Reescreve a página com os segmentos atuais, preservando o
    /// frontmatter e todo markdown ao redor.
    fn save(&self, vault: &str, page_path: &str) -> Result<(), String> {
        let body = embed::join(&self.segments);
        let content = if self.frontmatter.is_empty() {
            body
        } else {
            format!("{}\n{}", self.frontmatter, body)
        };
        handle_write_page_checked(
            vault.to_string(),
            page_path.to_string(),
            content,
            self.version.clone(),
        )
        .map(|_| ())
    }
}

/// Resumo de uma linha por embed, pra `embed list` dizer o que tem
/// dentro sem despejar o conteúdo.
fn embed_summary(data: &EmbedData) -> String {
    match data {
        EmbedData::Kanban(d) => format!("{} coluna(s), {} card(s)", d.columns.len(), d.items.len()),
        EmbedData::Calendar(d) => format!("{} evento(s)", d.entries.len()),
        EmbedData::Table(d) => format!("{} coluna(s), {} linha(s)", d.columns.len(), d.rows.len()),
        EmbedData::Callout(d) => format!("{} — {}", d.variant.slug(), d.title),
        EmbedData::Columns(d) => format!("{} painel(is)", d.columns.len()),
        EmbedData::Gallery(d) => format!("{} imagem(ns)", d.items.len()),
        EmbedData::Query(q) => format!(
            "consulta em {}",
            q.from.clone().unwrap_or_else(|| "todo o vault".to_string())
        ),
        EmbedData::Timeline(d) => format!("{} item(ns), escala {}", d.items.len(), d.scale.slug()),
        EmbedData::Actions(d) => format!("{} botão(ões)", d.buttons.len()),
    }
}

/// Conteúdo interno do embed, do jeito que está no arquivo: YAML pra
/// quase todos os tipos, tabela markdown pro `table` (o formato dele
/// nasceu como tabela markdown comum de propósito — ver `embed.rs`).
fn embed_body(data: &EmbedData) -> String {
    let text = data.to_fence_text();
    // Recorta as linhas de abertura e fechamento do wrapper.
    let mut lines: Vec<&str> = text.lines().collect();
    if !lines.is_empty() {
        lines.remove(0);
    }
    if !lines.is_empty() {
        lines.pop();
    }
    let body = lines.join("\n");
    if body.ends_with('\n') { body } else { format!("{body}\n") }
}

fn run_embed(vault: &str, json: bool, action: EmbedCommand) -> Result<(), String> {
    match action {
        EmbedCommand::List { page_path } => {
            let doc = PageDoc::load(vault, &page_path)?;
            let rows: Vec<serde_json::Value> = doc
                .embed_positions()
                .iter()
                .enumerate()
                .map(|(index, pos)| {
                    let DocSegment::Embed(data) = &doc.segments[*pos] else {
                        unreachable!()
                    };
                    serde_json::json!({
                        "index": index,
                        "type": data.kind().type_name(),
                        "summary": embed_summary(data),
                    })
                })
                .collect();
            if json {
                print_json(&rows)?;
            } else {
                for row in &rows {
                    println!(
                        "{}\t{}\t{}",
                        row["index"], row["type"].as_str().unwrap_or(""), row["summary"].as_str().unwrap_or("")
                    );
                }
            }
        }
        EmbedCommand::Get { page_path, index } => {
            let doc = PageDoc::load(vault, &page_path)?;
            let (_, data) = doc.embed_at(index)?;
            let body = embed_body(data);
            if json {
                print_json(&serde_json::json!({
                    "index": index,
                    "type": data.kind().type_name(),
                    "body": body,
                }))?;
            } else {
                print!("{body}");
            }
        }
        EmbedCommand::Set { page_path, index, file } => {
            let mut doc = PageDoc::load(vault, &page_path)?;
            let (pos, data) = doc.embed_at(index)?;
            let kind = data.kind();
            let raw = match &file {
                Some(path) => std::fs::read_to_string(path)
                    .map_err(|e| format!("erro ao ler {path}: {e}"))?,
                None => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .map_err(|e| format!("erro ao ler stdin: {e}"))?;
                    buf
                }
            };
            // Passa pelo parser do tipo: o que vai pro disco é sempre
            // saída de `to_fence_text`, nunca texto colado direto.
            doc.segments[pos] = DocSegment::Embed(EmbedData::parse(kind, &raw));
            doc.save(vault, &page_path)?;
        }
        EmbedCommand::AddCard { page_path, index, column, title } => {
            mutate_embed(vault, &page_path, index, |data| match data {
                EmbedData::Kanban(d) => {
                    if !d.columns.iter().any(|c| c == &column) {
                        return Err(format!(
                            "coluna \"{column}\" não existe nesse board (tem: {})",
                            d.columns.join(", ")
                        ));
                    }
                    d.add_card(column.clone(), title.clone());
                    Ok(())
                }
                other => Err(wrong_kind("add-card", EmbedKind::Kanban, other.kind())),
            })?;
        }
        EmbedCommand::AddRow { page_path, index, values } => {
            mutate_embed(vault, &page_path, index, |data| match data {
                EmbedData::Table(d) => {
                    let cells: Vec<String> =
                        values.split(',').map(|v| v.trim().to_string()).collect();
                    if cells.len() != d.columns.len() {
                        return Err(format!(
                            "a tabela tem {} coluna(s), mas vieram {} valor(es)",
                            d.columns.len(),
                            cells.len()
                        ));
                    }
                    d.rows.push(cells);
                    Ok(())
                }
                other => Err(wrong_kind("add-row", EmbedKind::Table, other.kind())),
            })?;
        }
        EmbedCommand::AddEvent { page_path, index, date, title } => {
            mutate_embed(vault, &page_path, index, |data| match data {
                EmbedData::Calendar(d) => {
                    d.add_entry(date.clone(), title.clone());
                    Ok(())
                }
                other => Err(wrong_kind("add-event", EmbedKind::Calendar, other.kind())),
            })?;
        }
    }
    Ok(())
}

fn wrong_kind(command: &str, expected: EmbedKind, found: EmbedKind) -> String {
    format!(
        "{command} só funciona em embed do tipo {}, mas o índice aponta pra um {}",
        expected.type_name(),
        found.type_name()
    )
}

/// Carrega, aplica `f` no embed do índice e grava — só grava se `f`
/// devolver `Ok`, pra um comando no tipo errado não tocar no arquivo.
fn mutate_embed(
    vault: &str,
    page_path: &str,
    index: usize,
    f: impl FnOnce(&mut EmbedData) -> Result<(), String>,
) -> Result<(), String> {
    let mut doc = PageDoc::load(vault, page_path)?;
    let (pos, _) = doc.embed_at(index)?;
    let DocSegment::Embed(data) = &mut doc.segments[pos] else {
        unreachable!("posição veio de embed_at")
    };
    f(data)?;
    doc.save(vault, page_path)
}

/// Parseia `campo=valor` / `campo!=valor` / `campo~valor` / `campo?` /
/// `campo>valor` / `campo<valor` numa `Condition`.
///
/// A ordem de teste importa: `!=` tem que vir antes de `=`, senão
/// `status!=done` viraria o campo `status!` igual a `done`.
fn parse_condition(raw: &str) -> Result<Condition, String> {
    let raw = raw.trim();
    if let Some(field) = raw.strip_suffix('?') {
        return Ok(Condition {
            field: field.trim().to_string(),
            op: QueryOp::Exists,
            value: String::new(),
        });
    }
    for op in [QueryOp::Neq, QueryOp::Eq, QueryOp::Contains, QueryOp::Gt, QueryOp::Lt] {
        if let Some((field, value)) = raw.split_once(op.symbol()) {
            if field.trim().is_empty() {
                break;
            }
            return Ok(Condition {
                field: field.trim().to_string(),
                op,
                value: value.trim().to_string(),
            });
        }
    }
    Err(format!(
        "condição inválida: \"{raw}\". Use campo=valor, campo!=valor, campo~valor, campo?, campo>valor ou campo<valor"
    ))
}

/// Parseia `count`, `sum:campo`, `avg:campo`, `min:campo`, `max:campo`.
fn parse_aggregate(raw: &str) -> Result<Aggregate, String> {
    let (nome, campo) = match raw.split_once(':') {
        Some((n, c)) => (n.trim(), c.trim().to_string()),
        None => (raw.trim(), String::new()),
    };
    let op = AggregateOp::all()
        .iter()
        .copied()
        .find(|o| o.slug() == nome.to_lowercase())
        .ok_or_else(|| {
            format!("agregado inválido: \"{raw}\". Use count, sum:campo, avg:campo, min:campo ou max:campo")
        })?;
    if op != AggregateOp::Count && campo.is_empty() {
        return Err(format!("{} precisa de um campo: {}:campo", op.slug(), op.slug()));
    }
    Ok(Aggregate { field: campo, op })
}

/// Lê a consulta declarada num embed `query`: `--from-embed
/// pages/painel.md:0`.
fn query_from_embed(vault: &str, spec: &str) -> Result<Query, String> {
    let (page_path, index) = spec.rsplit_once(':').ok_or_else(|| {
        format!("--from-embed espera <page_path>:<índice>, veio \"{spec}\"")
    })?;
    let index: usize = index
        .trim()
        .parse()
        .map_err(|_| format!("índice inválido em --from-embed: \"{index}\""))?;
    let doc = PageDoc::load(vault, page_path)?;
    let (_, data) = doc.embed_at(index)?;
    match data {
        EmbedData::Query(q) => Ok(q.clone()),
        other => Err(format!(
            "o embed {index} de {page_path} é do tipo {}, não query",
            other.kind().type_name()
        )),
    }
}

/// Instante atual em segundos desde a época (unix). Formato cru de
/// propósito: é o que um agente compara sem parsear data, e evita puxar
/// `chrono` só pra formatar uma linha de log.
fn agora_unix() -> String {
    let agora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    format!("{agora}")
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!("{}", serde_json::to_string_pretty(value).map_err(|e| e.to_string())?);
    Ok(())
}

