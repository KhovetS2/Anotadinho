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
    handle_aplicar_proposta, handle_listar_propostas, handle_propor, handle_read_page,
    handle_read_page_versioned, handle_recusar_proposta, handle_scan_vault, handle_search_content,
    handle_write_page_checked,
};
mod mcp;

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
    /// Imprime o conteúdo bruto (.md) de uma página, ou de UM BLOCO
    /// dela quando o path vem com `^id` (ciclo 176).
    Read {
        /// Path relativo ao vault (ex: pages/minha-nota.md) ou
        /// `pages/minha-nota.md^abc123` pra um bloco só.
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
        /// Grava mesmo com erro de validação (ciclo 189). Existe pra não
        /// travar caso legítimo que as regras não previram.
        #[arg(long, global = true)]
        forcar: bool,
        #[command(subcommand)]
        action: EmbedCommand,
    },
    /// Propõe uma escrita pra revisão humana, em vez de gravar direto
    /// (ciclo 204). É o modo recomendado pra agente: você vê o diff e
    /// decide.
    Propor {
        /// Página alvo, relativa ao vault.
        page_path: String,
        /// Arquivo com o conteúdo proposto. Omitido = lê do stdin.
        #[arg(long)]
        file: Option<String>,
        /// Por que esta mudança.
        #[arg(long, default_value = "")]
        motivo: String,
        /// Quem propôs.
        #[arg(long, default_value = "cli")]
        autor: String,
    },
    /// Propõe mover a etapa de uma página do fluxo (ciclo 229).
    ///
    /// NÃO grava: monta o conteúdo novo e manda pela mesma fila de
    /// revisão do `propor`. É como um agente fecha o que implementou
    /// sem decidir sozinho que está pronto.
    Etapa {
        /// Página alvo, relativa ao vault.
        page_path: String,
        /// Destino: rascunho, em-revisao, aprovada, em-execucao,
        /// concluida ou bloqueada.
        #[arg(long)]
        para: String,
        /// Arquivo com o resumo do que foi feito. `-` lê do stdin.
        #[arg(long)]
        resumo: Option<String>,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
    /// Prepara a pasta do vault: estrutura, templates, padrões, prompts
    /// e a página inicial (ciclo 233). Não sobrescreve nada.
    Init,
    /// Mapa do vault numa chamada só (ciclo 236).
    ///
    /// Pensado pra ser a PRIMEIRA coisa que um agente roda: diz onde as
    /// coisas moram, o que está em cada etapa do fluxo, o que espera
    /// revisão e quais padrões existem pra anexar. Sem ele, o mesmo
    /// entendimento custa uma dezena de `list-pages` e `read`.
    Contexto,
    /// Lista as propostas pendentes de revisão.
    Propostas,
    /// Aplica uma proposta aprovada.
    Aplicar {
        /// Id da proposta.
        id: String,
    },
    /// Descarta uma proposta sem aplicar.
    Recusar {
        /// Id da proposta.
        id: String,
    },
    /// Sobe um servidor MCP por stdio expondo o vault (ciclo 205).
    ///
    /// Só a leitura é direta: a única escrita é `propor`, que passa pela
    /// revisão humana. Um agente conectado aqui não consegue gravar
    /// página nenhuma sozinho.
    Mcp,
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
    /// Confere os embeds da página sem gravar nada (ciclo 189): diz o
    /// que já está no disco e não deveria estar.
    Check {
        /// Path relativo ao vault.
        page_path: String,
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
            // `caminho^id` devolve só aquele bloco — é o que deixa um
            // agente seguir uma referência `![[Página^id]]` sem baixar
            // a página inteira.
            if let Some((caminho, id)) = page_path.split_once('^') {
                let content = handle_read_page(cli.vault, caminho.to_string())?;
                let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&content);
                match anotadinho_core::links::find_block(corpo, id) {
                    Some(bloco) => println!("{bloco}"),
                    None => return Err(format!("{caminho} não tem o bloco ^{id}")),
                }
            } else {
                let content = handle_read_page(cli.vault, page_path)?;
                print!("{}", content);
            }
        }
        Command::Search { query } => {
            let results = handle_search_content(cli.vault, query)?;
            if cli.json {
                print_json(&results)?;
            } else {
                for hit in results {
                    // Resultado que veio de dentro de um embed diz o que
                    // é ("Kanban · coluna Backlog") — no terminal o
                    // agente vê a mesma coisa que a pessoa vê na janela.
                    match hit.origem {
                        Some(origem) => println!("{}\t[{}]\t{}", hit.path, origem, hit.snippet),
                        None => println!("{}\t{}", hit.path, hit.snippet),
                    }
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
        Command::Propor { page_path, file, motivo, autor } => {
            let conteudo = match &file {
                Some(p) => std::fs::read_to_string(p)
                    .map_err(|e| format!("erro ao ler {p}: {e}"))?,
                None => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .map_err(|e| format!("erro ao ler stdin: {e}"))?;
                    buf
                }
            };
            let id = propor_conteudo(&cli.vault, page_path, conteudo, motivo, autor)?;
            println!("{id}");
        }
        Command::Etapa { page_path, para, resumo, autor } => {
            let destino = anotadinho_core::fluxo::Etapa::from_slug(&para).ok_or_else(|| {
                format!(
                    "etapa desconhecida: \"{para}\". Use uma de: {}.",
                    anotadinho_core::fluxo::Etapa::all()
                        .iter()
                        .map(|e| e.slug())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            })?;
            let texto = match &resumo {
                Some(p) if p == "-" => {
                    let mut buf = String::new();
                    std::io::stdin()
                        .read_to_string(&mut buf)
                        .map_err(|e| format!("erro ao ler stdin: {e}"))?;
                    Some(buf)
                }
                Some(p) => Some(
                    std::fs::read_to_string(p).map_err(|e| format!("erro ao ler {p}: {e}"))?,
                ),
                None => None,
            };
            let atual = handle_read_page(cli.vault.clone(), page_path.clone())?;
            let novo = anotadinho_core::fluxo::aplicar_etapa_no_texto(
                &atual,
                destino,
                texto.as_deref().map(str::trim).filter(|s| !s.is_empty()),
            )?;
            let id = propor_conteudo(
                &cli.vault,
                page_path,
                novo,
                format!("mover a etapa para \"{}\"", destino.slug()),
                autor,
            )?;
            println!("{id}");
        }
        Command::Init => {
            let criados = anotadinho_ipc::handle_criar_vault(cli.vault.clone())?;
            if criados.is_empty() {
                println!("nada a fazer: já estava tudo lá");
            } else {
                for c in &criados {
                    println!("{c}");
                }
            }
        }
        Command::Contexto => {
            let paginas = handle_scan_vault(cli.vault.clone())?;
            let propostas = handle_listar_propostas(cli.vault.clone()).unwrap_or_default();
            if cli.json {
                print_json(&contexto_json(&cli.vault, &paginas, &propostas))?;
            } else {
                print!("{}", contexto_texto(&cli.vault, &paginas, &propostas));
            }
        }
        Command::Mcp => {
            mcp::servir(cli.vault)?;
        }
        Command::Propostas => {
            let lista = handle_listar_propostas(cli.vault)?;
            if cli.json {
                print_json(&lista)?;
            } else if lista.is_empty() {
                println!("nenhuma proposta pendente");
            } else {
                for p in &lista {
                    println!(
                        "{}\t{}\t{:?}\t{}\t{}",
                        p.id, p.alvo, p.operacao, p.autor, p.motivo
                    );
                }
            }
        }
        Command::Aplicar { id } => {
            let alvo = handle_aplicar_proposta(cli.vault, id)?;
            println!("{alvo}");
        }
        Command::Recusar { id } => {
            handle_recusar_proposta(cli.vault, id)?;
        }
        Command::Embed { forcar, action } => {
            let vault = cli.vault.clone();
            let json = cli.json;
            run_embed(&vault, json, forcar, action)?
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
/// Monta e enfileira uma proposta. Compartilhado por `propor` e `etapa`
/// para os dois entrarem na fila exatamente do mesmo jeito.
fn propor_conteudo(
    vault: &str,
    page_path: String,
    conteudo: String,
    motivo: String,
    autor: String,
) -> Result<String, String> {
    let existe = std::path::Path::new(vault).join(&page_path).exists();
    let proposta = anotadinho_core::proposta::Proposta {
        // Id derivado do alvo + relógio: legível na listagem e único o
        // bastante pra duas propostas seguidas na mesma página não se
        // sobrescreverem.
        id: format!(
            "{}-{}",
            anotadinho_core::fluxo::slug_de_titulo(&page_path),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_secs())
                .unwrap_or(0)
        ),
        autor,
        quando: agora_legivel(),
        motivo,
        alvo: page_path,
        operacao: if existe {
            anotadinho_core::proposta::Operacao::Substituir
        } else {
            anotadinho_core::proposta::Operacao::Criar
        },
        conteudo,
    };
    handle_propor(vault.to_string(), proposta)
}

/// Conta quantas páginas há por valor de uma chave, em ordem estável.
fn contar_por<F>(paginas: &[anotadinho_core::PageIndexEntry], chave: F) -> Vec<(String, usize)>
where
    F: Fn(&anotadinho_core::PageIndexEntry) -> Option<String>,
{
    let mut mapa: std::collections::BTreeMap<String, usize> = Default::default();
    for p in paginas {
        if let Some(v) = chave(p) {
            *mapa.entry(v).or_default() += 1;
        }
    }
    let mut v: Vec<_> = mapa.into_iter().collect();
    v.sort_by(|a, b| b.1.cmp(&a.1).then(a.0.cmp(&b.0)));
    v
}

/// A pasta de uma página (`pages/specs/x.md` → `pages/specs`).
fn pasta_de(path: &str) -> String {
    match path.rfind('/') {
        Some(i) => path[..i].to_string(),
        None => ".".to_string(),
    }
}

fn contexto_texto(
    vault: &str,
    paginas: &[anotadinho_core::PageIndexEntry],
    propostas: &[anotadinho_core::proposta::Proposta],
) -> String {
    use std::fmt::Write;
    let mut s = String::new();
    let _ = writeln!(s, "vault: {vault}");
    let _ = writeln!(s, "páginas: {}", paginas.len());
    let _ = writeln!(s);

    let _ = writeln!(s, "## Onde as coisas moram");
    for (pasta, n) in contar_por(paginas, |p| Some(pasta_de(&p.path))) {
        let _ = writeln!(s, "  {pasta:<32} {n}");
    }
    let _ = writeln!(s);

    let _ = writeln!(s, "## Tipos de página");
    for (tipo, n) in contar_por(paginas, |p| {
        (!p.page_type.is_empty() && p.page_type != "md").then(|| p.page_type.clone())
    }) {
        let _ = writeln!(s, "  {tipo:<32} {n}");
    }
    let _ = writeln!(s);

    // O estado do fluxo é a pergunta mais frequente: "o que está
    // esperando alguma coisa de mim?"
    let _ = writeln!(s, "## Fluxo, por etapa");
    for artefato in ["spec", "proposta", "execucao"] {
        let do_tipo: Vec<_> = paginas.iter().filter(|p| p.page_type == artefato).collect();
        if do_tipo.is_empty() {
            continue;
        }
        let _ = writeln!(s, "  {artefato}:");
        let mut por_etapa: std::collections::BTreeMap<&str, Vec<&str>> = Default::default();
        for p in &do_tipo {
            let etapa = p.properties.get("status").map(String::as_str).unwrap_or("(sem etapa)");
            por_etapa.entry(etapa).or_default().push(&p.title);
        }
        for (etapa, titulos) in por_etapa {
            let _ = writeln!(s, "    {etapa} ({})", titulos.len());
            for t in titulos.iter().take(8) {
                let _ = writeln!(s, "      - {t}");
            }
            if titulos.len() > 8 {
                let _ = writeln!(s, "      … e mais {}", titulos.len() - 8);
            }
        }
    }
    let _ = writeln!(s);

    let _ = writeln!(s, "## Esperando revisão humana");
    if propostas.is_empty() {
        let _ = writeln!(s, "  nada pendente");
    } else {
        for p in propostas {
            let _ = writeln!(s, "  {} → {}", p.id, p.alvo);
        }
    }
    let _ = writeln!(s);

    let _ = writeln!(s, "## Padrões (anexe o que for do assunto)");
    for p in paginas.iter().filter(|p| p.path.starts_with("pages/padroes/")) {
        let _ = writeln!(s, "  {:<44} {}", p.path, p.title);
    }
    let _ = writeln!(s);

    let _ = writeln!(s, "## Prompts padrão");
    for p in anotadinho_core::prompt_padrao::descobrir(paginas.to_vec()) {
        let _ = writeln!(s, "  {:<44} {}", p.path, p.title);
    }
    s
}

fn contexto_json(
    vault: &str,
    paginas: &[anotadinho_core::PageIndexEntry],
    propostas: &[anotadinho_core::proposta::Proposta],
) -> serde_json::Value {
    serde_json::json!({
        "vault": vault,
        "paginas": paginas.len(),
        "pastas": contar_por(paginas, |p| Some(pasta_de(&p.path)))
            .into_iter().map(|(k, v)| serde_json::json!({ "pasta": k, "paginas": v })).collect::<Vec<_>>(),
        "tipos": contar_por(paginas, |p| (!p.page_type.is_empty() && p.page_type != "md").then(|| p.page_type.clone()))
            .into_iter().map(|(k, v)| serde_json::json!({ "tipo": k, "paginas": v })).collect::<Vec<_>>(),
        "propostas_pendentes": propostas.iter().map(|p| serde_json::json!({ "id": p.id, "alvo": p.alvo })).collect::<Vec<_>>(),
        "padroes": paginas.iter().filter(|p| p.path.starts_with("pages/padroes/"))
            .map(|p| serde_json::json!({ "path": p.path, "titulo": p.title })).collect::<Vec<_>>(),
        "prompts": anotadinho_core::prompt_padrao::descobrir(paginas.to_vec())
            .into_iter().map(|p| serde_json::json!({ "path": p.path, "titulo": p.title })).collect::<Vec<_>>(),
    })
}

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
        EmbedData::Fluxo(d) => format!("{} — {}", d.artefato.label(), d.etapa.label()),
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

/// Confere um embed antes de gravar (ciclo 189).
///
/// Valida só o embed TOCADO, não a página inteira: um embed inválido
/// pré-existente noutro ponto não pode travar uma edição que não tem
/// nada a ver com ele.
///
/// `Erro` aborta com saída != 0 e nada é gravado; `Aviso` só imprime.
/// `--forcar` deixa passar mesmo com erro.
fn conferir_antes_de_gravar(
    vault: &str,
    data: &EmbedData,
    forcar: bool,
) -> Result<(), String> {
    let ctx = embed::ValidationCtx::com_vault(vault);
    let problemas = data.validate(&ctx);
    if problemas.is_empty() {
        return Ok(());
    }
    for p in &problemas {
        let marca = match p.severidade {
            embed::Severidade::Erro => "erro",
            embed::Severidade::Aviso => "aviso",
        };
        eprintln!("{marca}: {} — {}", p.onde, p.mensagem);
    }
    if EmbedData::tem_erro(&problemas) && !forcar {
        return Err(
            "o embed tem erro de validação e nada foi gravado (use --forcar pra gravar assim mesmo)"
                .to_string(),
        );
    }
    Ok(())
}

fn run_embed(vault: &str, json: bool, forcar: bool, action: EmbedCommand) -> Result<(), String> {
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
        EmbedCommand::Check { page_path } => {
            let doc = PageDoc::load(vault, &page_path)?;
            let ctx = embed::ValidationCtx::com_vault(vault);
            let mut achados: Vec<serde_json::Value> = Vec::new();
            let mut tem_erro = false;
            for (index, pos) in doc.embed_positions().iter().enumerate() {
                let DocSegment::Embed(data) = &doc.segments[*pos] else {
                    unreachable!()
                };
                for p in data.validate(&ctx) {
                    tem_erro |= p.severidade == embed::Severidade::Erro;
                    achados.push(serde_json::json!({
                        "index": index,
                        "type": data.kind().type_name(),
                        "severidade": p.severidade,
                        "onde": p.onde,
                        "mensagem": p.mensagem,
                    }));
                }
            }
            if json {
                print_json(&achados)?;
            } else if achados.is_empty() {
                println!("nenhum problema em {page_path}");
            } else {
                for a in &achados {
                    println!(
                        "{}\t{}\t{}\t{} — {}",
                        a["index"],
                        a["type"].as_str().unwrap_or(""),
                        a["severidade"].as_str().unwrap_or(""),
                        a["onde"].as_str().unwrap_or(""),
                        a["mensagem"].as_str().unwrap_or("")
                    );
                }
            }
            // Saída != 0 quando há erro: dá pra usar num hook de commit.
            if tem_erro {
                std::process::exit(1);
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
            let novo = EmbedData::parse(kind, &raw);
            conferir_antes_de_gravar(vault, &novo, forcar)?;
            doc.segments[pos] = DocSegment::Embed(novo);
            doc.save(vault, &page_path)?;
        }
        EmbedCommand::AddCard { page_path, index, column, title } => {
            mutate_embed(vault, &page_path, index, forcar, |data| match data {
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
            mutate_embed(vault, &page_path, index, forcar, |data| match data {
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
            mutate_embed(vault, &page_path, index, forcar, |data| match data {
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
    forcar: bool,
    f: impl FnOnce(&mut EmbedData) -> Result<(), String>,
) -> Result<(), String> {
    let mut doc = PageDoc::load(vault, page_path)?;
    let (pos, _) = doc.embed_at(index)?;
    let DocSegment::Embed(data) = &mut doc.segments[pos] else {
        unreachable!("posição veio de embed_at")
    };
    f(data)?;
    // Depois da mutação e ANTES do save: é o estado que iria pro disco
    // que precisa ser conferido, não o que estava lá antes.
    conferir_antes_de_gravar(vault, data, forcar)?;
    doc.save(vault, page_path)
}

/// Parseia `campo=valor` / `campo!=valor` / `campo~valor` / `campo?` /
/// `campo>valor` / `campo<valor` numa `Condition`.
///
/// A ordem de teste importa: `!=` tem que vir antes de `=`, senão
/// `status!=done` viraria o campo `status!` igual a `done`.
pub(crate) fn parse_condition(raw: &str) -> Result<Condition, String> {
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

/// `"YYYY-MM-DD HH:MM"` sem dependência de crate de data — o formato só
/// precisa ser legível e ordenável.
pub(crate) fn agora_legivel() -> String {
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dias = segundos / 86_400;
    let hora = (segundos % 86_400) / 3600;
    let minuto = (segundos % 3600) / 60;
    // Conversão de dias desde 1970 pra data civil (algoritmo de Howard
    // Hinnant) — o mesmo que o `date_util` do core já usa.
    let (ano, mes, dia) = crate::civil_de_dias(dias as i64);
    format!("{ano:04}-{mes:02}-{dia:02} {hora:02}:{minuto:02}")
}

/// Dias desde 1970-01-01 → (ano, mês, dia).
fn civil_de_dias(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}
