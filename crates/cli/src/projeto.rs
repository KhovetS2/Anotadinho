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
        TarefaCommand::Nova { slug, titulo, file, motivo, autor, lote } => {
            if !std::path::Path::new(vault).join(projeto::caminho_do_projeto(&slug)).exists() {
                return Err(format!("não existe o projeto {slug} (crie com `projeto novo`)"));
            }
            let corpo = match &file {
                Some(f) => ler_arquivo_ou_stdin(f)?,
                None => String::new(),
            };
            let (caminho, conteudo) =
                handle_montar_tarefa(vault.to_string(), slug, titulo.clone(), corpo, hoje())?;
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
