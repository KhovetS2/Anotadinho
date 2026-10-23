//! `projeto` e `tarefa` no terminal (ciclo 458).
//!
//! O que lê, lê direto. O que escreve PROPÕE, como `propor` e `etapa`:
//! o CLI é o canal do agente, e um `tarefa etapa --para executar` gravado
//! direto mandaria código pro repositório sem ninguém ter olhado. A
//! pessoa cria e move pela janela e pela TUI, que chamam os mesmos
//! `handle_*` sem proposta no meio.

use anotadinho_core::projeto::{self, EtapaTarefa, Quem, Tarefa};
use anotadinho_ipc::{handle_listar_projetos, handle_montar_tarefa, handle_read_page, handle_tarefas_do_projeto};
use clap::Subcommand;

/// Operações sobre projetos.
#[derive(Subcommand)]
pub(crate) enum ProjetoCommand {
    /// Propõe um projeto novo: página do projeto, arquitetura e conversa
    /// geral, num lote só. Página que já existe fica de fora.
    Novo {
        /// Título do projeto.
        titulo: String,
        /// Nome da pasta. Omitido = derivado do título.
        #[arg(long)]
        slug: Option<String>,
    },
    /// Lista os projetos, com a contagem de tarefas por etapa.
    Listar,
    /// Lista as tarefas de um projeto.
    Tarefas {
        /// Slug do projeto.
        slug: String,
    },
    /// Imprime o MODELO de plano de projeto (ciclo 470): o formato que
    /// `projeto montar` lê. Comece por aqui pra montar a base de um
    /// projeto.
    ModeloDePlano,
    /// Propõe o projeto inteiro de um plano YAML — projeto, arquitetura,
    /// sprints e tarefas — num lote só. Num projeto que já existe, só
    /// acrescenta sprints e tarefas, numeradas depois das que existem.
    Montar {
        /// Arquivo do plano. `-` lê do stdin.
        #[arg(long)]
        file: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
    /// O projeto numa chamada: sprints (com a ativa) e as tarefas com a
    /// etapa e a sprint de cada uma.
    Ver {
        /// Slug do projeto.
        slug: String,
    },
}

/// Operações sobre sprints (ciclo 470). O que escreve, propõe.
#[derive(Subcommand)]
pub(crate) enum SprintCommand {
    /// Propõe uma sprint nova, planejada.
    Nova {
        /// Slug do projeto.
        slug: String,
        /// Nome da sprint.
        nome: String,
        /// O que ela entrega.
        #[arg(long, default_value = "")]
        objetivo: String,
        /// Início, AAAA-MM-DD.
        #[arg(long)]
        inicio: Option<String>,
        /// Fim, AAAA-MM-DD.
        #[arg(long)]
        fim: Option<String>,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
    /// Lista as sprints do projeto, com estado e quantas tarefas cada uma tem.
    Listar {
        /// Slug do projeto.
        slug: String,
    },
    /// Propõe iniciar uma sprint planejada (pelo número ou pelo nome).
    Iniciar {
        /// Slug do projeto.
        slug: String,
        /// Número ou nome da sprint.
        sprint: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
    /// Propõe encerrar a sprint ativa, num lote: a sprint e as pendentes.
    Encerrar {
        /// Slug do projeto.
        slug: String,
        /// Número ou nome da sprint.
        sprint: String,
        /// Pra onde vão as tarefas não feitas: `proxima` ou `backlog`.
        #[arg(long)]
        pendentes: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
}

/// Operações sobre tarefas.
#[derive(Subcommand)]
pub(crate) enum TarefaCommand {
    /// Propõe uma tarefa nova, em `ideia`.
    Nova {
        /// Slug do projeto.
        slug: String,
        /// Título da tarefa.
        titulo: String,
        /// Arquivo com o corpo. Omitido = o esqueleto (Objetivo,
        /// Critérios de aceite, Abordagem, Fora de escopo). `-` lê do
        /// stdin.
        #[arg(long)]
        file: Option<String>,
        /// Por que esta tarefa.
        #[arg(long, default_value = "")]
        motivo: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
        /// Lote: várias tarefas que só fazem sentido juntas.
        #[arg(long)]
        lote: Option<String>,
        /// A sprint (número ou nome). Omitido: backlog.
        #[arg(long)]
        sprint: Option<String>,
    },
    /// Propõe pôr a tarefa numa sprint, ou no backlog.
    Sprint {
        /// Caminho da tarefa, relativo ao vault.
        page_path: String,
        /// Número ou nome da sprint, ou `backlog`.
        sprint: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
    /// Propõe mover a etapa de uma tarefa. Só as transições humanas:
    /// `executando` e `revisao` são registro do app.
    Etapa {
        /// Caminho da tarefa, relativo ao vault.
        page_path: String,
        /// Destino: ideia, detalhando, pronta, executar, feita ou
        /// bloqueada.
        #[arg(long)]
        para: String,
        /// Quem propôs.
        #[arg(long, default_value = "agente")]
        autor: String,
    },
}

fn hoje() -> String {
    crate::agora_legivel().split_whitespace().next().unwrap_or("").to_string()
}

fn ler_arquivo_ou_stdin(file: &str) -> Result<String, String> {
    if file == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).map_err(|e| format!("erro ao ler stdin: {e}"))?;
        return Ok(buf);
    }
    std::fs::read_to_string(file).map_err(|e| format!("erro ao ler {file}: {e}"))
}

pub(crate) fn run_projeto(vault: &str, json: bool, action: ProjetoCommand) -> Result<(), String> {
    match action {
        ProjetoCommand::Novo { titulo, slug } => {
            let slug = anotadinho_core::fluxo::slug_de_titulo(slug.as_deref().unwrap_or(&titulo));
            let lote = format!("projeto-{slug}");
            let raiz = std::path::Path::new(vault);
            let mut propostas = 0;
            for (caminho, conteudo) in projeto::montar_projeto(&slug, &titulo, &hoje()) {
                // Propor por cima de uma página existente viraria
                // SUBSTITUIR — e a arquitetura de alguém trocada pelo
                // esqueleto, a um clique de "aplicar".
                if raiz.join(&caminho).exists() {
                    eprintln!("já existe, fica como está: {caminho}");
                    continue;
                }
                let id = crate::propor_conteudo(
                    vault,
                    caminho,
                    conteudo,
                    format!("criar o projeto \"{}\"", titulo.trim()),
                    "cli".to_string(),
                    Some(lote.clone()),
                )?;
                println!("{id}");
                propostas += 1;
            }
            if propostas == 0 {
                println!("nada a propor: o projeto {slug} já existe");
            }
        }
        ProjetoCommand::Listar => {
            let projetos = handle_listar_projetos(vault.to_string())?;
            let mut linhas = Vec::new();
            for p in projetos {
                let tarefas = handle_tarefas_do_projeto(vault.to_string(), p.slug.clone())?;
                linhas.push((p, tarefas));
            }
            if json {
                let v: Vec<_> = linhas
                    .iter()
                    .map(|(p, ts)| {
                        serde_json::json!({
                            "slug": p.slug, "titulo": p.titulo, "caminho": p.caminho,
                            "etapas": contagem_json(ts),
                        })
                    })
                    .collect();
                return crate::print_json(&v);
            }
            if linhas.is_empty() {
                println!("nenhum projeto em {}", projeto::PASTA);
            }
            for (p, ts) in &linhas {
                println!("{:<24} {}  [{}]", p.slug, p.titulo, contagem_texto(ts));
            }
        }
        ProjetoCommand::ModeloDePlano => print!("{}", anotadinho_core::plano::MODELO_DE_PLANO),
        ProjetoCommand::Montar { file, autor } => {
            let yaml = ler_arquivo_ou_stdin(&file)?;
            let r = anotadinho_ipc::handle_propor_plano(vault.to_string(), yaml, crate::agora_legivel(), autor)?;
            if json {
                return crate::print_json(&serde_json::json!({ "slug": r.slug, "lote": r.lote, "ids": r.ids, "avisos": r.avisos }));
            }
            for id in &r.ids {
                println!("{id}");
            }
            for a in &r.avisos {
                eprintln!("aviso: {a}");
            }
            eprintln!("{} proposta(s) no lote {} — revise com `propostas` e aplique o lote junto", r.ids.len(), r.lote);
        }
        ProjetoCommand::Ver { slug } => {
            let e = anotadinho_ipc::handle_estado_do_projeto(vault.to_string(), slug.clone())?;
            if json {
                return crate::print_json(&serde_json::json!({
                    "projeto": e.projeto, "config": e.config, "sprints": e.sprints,
                    "sprint_ativa": anotadinho_core::sprint::ativa(&e.sprints).map(|s| s.numero),
                    "tarefas": e.tarefas, "avisos": e.avisos,
                }));
            }
            println!("{}  ({})", e.projeto.titulo, e.projeto.slug);
            println!("repositório: {}{}", e.config.repositorio.as_deref().unwrap_or("(nenhum)"), if e.config.pausado { " · fila PAUSADA" } else { "" });
            println!("\nsprints:");
            if e.sprints.is_empty() {
                println!("  (nenhuma — o quadro mostra todas as tarefas)");
            }
            let por_sprint = anotadinho_core::sprint::PorSprint::novo(&e.tarefas);
            for s in &e.sprints {
                let (feitas, total) = por_sprint.resumo(s.numero);
                println!("  {:02}  {:<10} {}  [{feitas}/{total} feitas]", s.numero, s.estado.slug(), s.nome);
            }
            println!("\ntarefas:");
            for t in &e.tarefas {
                let sprint = t.sprint.map(|n| format!("sprint {n}")).unwrap_or_else(|| "backlog".into());
                println!("  {:03}  {:<11} {:<10} {}", t.numero, t.etapa.slug(), sprint, t.titulo);
            }
            for a in &e.avisos {
                println!("\naviso: {a}");
            }
        }
        ProjetoCommand::Tarefas { slug } => {
            let tarefas = handle_tarefas_do_projeto(vault.to_string(), slug.clone())?;
            if json {
                return crate::print_json(&tarefas);
            }
            if tarefas.is_empty() {
                println!("nenhuma tarefa em {}", projeto::pasta_das_tarefas(&slug));
            }
            for t in &tarefas {
                println!("{:03}  {:<11} {}  {}", t.numero, t.etapa.slug(), t.titulo, t.caminho);
            }
        }
    }
    Ok(())
}

pub(crate) fn run_tarefa(vault: &str, action: TarefaCommand) -> Result<(), String> {
    match action {
        TarefaCommand::Nova { slug, titulo, file, motivo, autor, lote, sprint } => {
            if !std::path::Path::new(vault).join(projeto::caminho_do_projeto(&slug)).exists() {
                return Err(format!("não existe o projeto {slug} (crie com `projeto novo`)"));
            }
            let corpo = match &file {
                Some(f) => ler_arquivo_ou_stdin(f)?,
                None => String::new(),
            };
            let (caminho, mut conteudo) =
                handle_montar_tarefa(vault.to_string(), slug.clone(), titulo.clone(), corpo, hoje())?;
            if let Some(r) = sprint.as_deref() {
                let s = sprint_do_projeto(vault, &slug, r)?;
                conteudo = anotadinho_core::sprint::mudar_sprint_da_tarefa(&conteudo, Some(s.numero))?;
            }
            let motivo = if motivo.trim().is_empty() { format!("tarefa nova: {}", titulo.trim()) } else { motivo };
            let id = crate::propor_conteudo(vault, caminho, conteudo, motivo, autor, lote)?;
            println!("{id}");
        }
        TarefaCommand::Etapa { page_path, para, autor } => {
            let destino = EtapaTarefa::from_slug(&para).ok_or_else(|| {
                format!(
                    "etapa desconhecida: \"{para}\". Use uma de: {}.",
                    EtapaTarefa::all().iter().map(|e| e.slug()).collect::<Vec<_>>().join(", ")
                )
            })?;
            let atual = handle_read_page(vault.to_string(), page_path.clone())?;
            let novo = projeto::mudar_etapa(&atual, destino, Quem::Humano)?;
            let id = crate::propor_conteudo(
                vault,
                page_path,
                novo,
                format!("mover a tarefa para \"{}\"", destino.slug()),
                autor,
                None,
            )?;
            println!("{id}");
        }
        TarefaCommand::Sprint { page_path, sprint, autor } => {
            let atual = handle_read_page(vault.to_string(), page_path.clone())?;
            let t = Tarefa::ler(&page_path, &atual).ok_or_else(|| format!("{page_path} não é uma tarefa"))?;
            let nova = if sprint.trim().eq_ignore_ascii_case("backlog") {
                None
            } else {
                let s = sprint_do_projeto(vault, &t.projeto, &sprint)?;
                if s.estado == anotadinho_core::sprint::EstadoSprint::Encerrada {
                    return Err(format!("a sprint \"{}\" está encerrada", s.nome));
                }
                Some(s.numero)
            };
            let novo = anotadinho_core::sprint::mudar_sprint_da_tarefa(&atual, nova)?;
            let destino = nova.map(|n| format!("sprint {n}")).unwrap_or_else(|| "backlog".into());
            let id = crate::propor_conteudo(vault, page_path, novo, format!("mover a tarefa pra {destino}"), autor, None)?;
            println!("{id}");
        }
    }
    Ok(())
}

/// A sprint de um projeto pelo número ou nome, com um erro que lista as
/// que existem — é o que a LLM precisa pra acertar na segunda tentativa.
fn sprint_do_projeto(vault: &str, slug: &str, referencia: &str) -> Result<anotadinho_core::sprint::Sprint, String> {
    let sprints = anotadinho_ipc::handle_sprints_do_projeto(vault.to_string(), slug.to_string())?;
    anotadinho_ipc::achar_sprint(&sprints, referencia).ok_or_else(|| {
        let nomes: Vec<String> = sprints.iter().map(|s| format!("{} ({}, {})", s.nome, s.numero, s.estado.slug())).collect();
        format!("não existe a sprint \"{referencia}\" em {slug}. Existem: {}", if nomes.is_empty() { "nenhuma".into() } else { nomes.join(", ") })
    })
}

pub(crate) fn run_sprint(vault: &str, json: bool, action: SprintCommand) -> Result<(), String> {
    use anotadinho_core::sprint::{DestinoDasPendentes, EstadoSprint};
    match action {
        SprintCommand::Nova { slug, nome, objetivo, inicio, fim, autor } => {
            let id = anotadinho_ipc::handle_propor_sprint(vault.to_string(), slug, nome, objetivo, inicio, fim, crate::agora_legivel(), autor)?;
            println!("{id}");
        }
        SprintCommand::Listar { slug } => {
            let sprints = anotadinho_ipc::handle_sprints_do_projeto(vault.to_string(), slug.clone())?;
            let tarefas = handle_tarefas_do_projeto(vault.to_string(), slug.clone())?;
            let por_sprint = anotadinho_core::sprint::PorSprint::novo(&tarefas);
            if json {
                let v: Vec<_> = sprints
                    .iter()
                    .map(|s| {
                        let (feitas, total) = por_sprint.resumo(s.numero);
                        serde_json::json!({ "sprint": s, "feitas": feitas, "total": total })
                    })
                    .collect();
                return crate::print_json(&v);
            }
            if sprints.is_empty() {
                println!("nenhuma sprint em {}", anotadinho_core::sprint::pasta_das_sprints(&slug));
            }
            for s in &sprints {
                let (feitas, total) = por_sprint.resumo(s.numero);
                let datas = match (&s.inicio, &s.fim) {
                    (Some(i), Some(f)) => format!("  {i} → {f}"),
                    _ => String::new(),
                };
                println!("{:02}  {:<10} {}  [{feitas}/{total} feitas]{datas}", s.numero, s.estado.slug(), s.nome);
            }
        }
        SprintCommand::Iniciar { slug, sprint, autor } => {
            let s = sprint_do_projeto(vault, &slug, &sprint)?;
            let todas = anotadinho_ipc::handle_sprints_do_projeto(vault.to_string(), slug)?;
            let texto = handle_read_page(vault.to_string(), s.caminho.clone())?;
            let novo = anotadinho_core::sprint::mudar_estado(&texto, &s.caminho, EstadoSprint::Ativa, &todas)?;
            let id = crate::propor_conteudo(vault, s.caminho, novo, format!("iniciar a sprint \"{}\"", s.nome), autor, None)?;
            println!("{id}");
        }
        SprintCommand::Encerrar { slug, sprint, pendentes, autor } => {
            let destino = match pendentes.trim() {
                "proxima" | "próxima" => DestinoDasPendentes::ProximaSprint,
                "backlog" => DestinoDasPendentes::Backlog,
                outro => return Err(format!("--pendentes \"{outro}\": use `proxima` ou `backlog`")),
            };
            let s = sprint_do_projeto(vault, &slug, &sprint)?;
            let ids = anotadinho_ipc::handle_propor_encerramento(vault.to_string(), s.caminho, destino, crate::agora_legivel(), autor)?;
            for id in ids {
                println!("{id}");
            }
        }
    }
    Ok(())
}

fn contagem_texto(tarefas: &[Tarefa]) -> String {
    let c = projeto::contagem_por_etapa(tarefas);
    if c.is_empty() {
        return "sem tarefas".to_string();
    }
    c.iter().map(|(e, n)| format!("{} {n}", e.slug())).collect::<Vec<_>>().join(", ")
}

fn contagem_json(tarefas: &[Tarefa]) -> serde_json::Value {
    let mut m = serde_json::Map::new();
    for (e, n) in projeto::contagem_por_etapa(tarefas) {
        m.insert(e.slug().to_string(), serde_json::json!(n));
    }
    serde_json::Value::Object(m)
}

/// Os projetos do vault a partir do índice que o `contexto` já leu:
/// `(slug, título, contagem por etapa)`. Sem IO extra, porque o
/// `contexto` é a primeira coisa que todo agente roda.
/// A sprint ativa de cada projeto, pelo índice (ciclo 470): `slug → nome`.
pub(crate) fn sprints_ativas_do_indice(paginas: &[anotadinho_core::PageIndexEntry]) -> std::collections::BTreeMap<String, String> {
    let mut fora = std::collections::BTreeMap::new();
    for p in paginas.iter().filter(|p| p.page_type == anotadinho_core::sprint::TIPO_SPRINT) {
        let ativa = p.properties.get("estado").and_then(|e| anotadinho_core::sprint::EstadoSprint::from_slug(e))
            == Some(anotadinho_core::sprint::EstadoSprint::Ativa);
        if let (true, Some(slug)) = (ativa, projeto::projeto_do_caminho(&p.path)) {
            fora.insert(slug.to_string(), p.title.clone());
        }
    }
    fora
}

pub(crate) fn projetos_do_indice(
    paginas: &[anotadinho_core::PageIndexEntry],
) -> Vec<(String, String, std::collections::BTreeMap<EtapaTarefa, usize>)> {
    let mut saida = Vec::new();
    for p in paginas.iter().filter(|p| p.page_type == projeto::TIPO_PROJETO) {
        let Some(slug) = projeto::projeto_do_caminho(&p.path) else { continue };
        let mut contagem = std::collections::BTreeMap::new();
        for t in paginas.iter().filter(|t| {
            t.page_type == projeto::TIPO_TAREFA && projeto::projeto_do_caminho(&t.path) == Some(slug)
        }) {
            let etapa = t
                .properties
                .get("status")
                .and_then(|s| EtapaTarefa::from_slug(s))
                .unwrap_or_default();
            *contagem.entry(etapa).or_insert(0) += 1;
        }
        saida.push((slug.to_string(), p.title.clone(), contagem));
    }
    saida.sort_by(|a, b| a.0.cmp(&b.0));
    saida
}
