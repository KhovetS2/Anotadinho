//! IO do módulo Projeto (ciclo 458).
//!
//! Tudo o que GUI, TUI e CLI fazem com projeto passa por aqui. O motivo
//! é o mesmo do ciclo 423: quem tem duas implementações tem duas UIs que
//! divergem. A regra mora em `anotadinho_core::projeto`; aqui só se lê,
//! se grava e se tranca.

use std::path::{Path, PathBuf};

use anotadinho_core::conversa::{self, Autor, Contexto, Mensagem};
use anotadinho_core::projeto::{
    self, ConfigProjetos, EtapaTarefa, Projeto, Quem, Tarefa,
};
use anotadinho_core::MarkdownCodec;
use serde::{Deserialize, Serialize};

use crate::{handle_freio_do_dia, handle_read_page, handle_write_page};

/// O que `handle_criar_projeto` fez.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjetoCriado {
    /// Slug, que é também o nome da pasta.
    pub slug: String,
    /// Páginas criadas agora.
    pub criadas: Vec<String>,
    /// Páginas que já existiam e ficaram como estavam.
    pub existiam: Vec<String>,
}

/// Cria um projeto: as três páginas e a pasta de tarefas.
///
/// Nunca sobrescreve. Rodar de novo num projeto que existe só completa
/// o que falta, e diz o que já estava lá: perder a arquitetura de alguém
/// porque clicou "novo projeto" com o mesmo nome não é aceitável.
pub fn handle_criar_projeto(
    vault_path: String,
    titulo: String,
    slug: Option<String>,
    hoje: String,
) -> Result<ProjetoCriado, String> {
    let titulo = titulo.trim();
    if titulo.is_empty() {
        return Err("o projeto precisa de um título".to_string());
    }
    let slug = anotadinho_core::fluxo::slug_de_titulo(slug.as_deref().unwrap_or(titulo));
    let raiz = Path::new(&vault_path);
    let mut criado = ProjetoCriado { slug: slug.clone(), criadas: Vec::new(), existiam: Vec::new() };
    for (caminho, conteudo) in projeto::montar_projeto(&slug, titulo, &hoje) {
        if raiz.join(&caminho).exists() {
            criado.existiam.push(caminho);
            continue;
        }
        handle_write_page(vault_path.clone(), caminho.clone(), conteudo)?;
        criado.criadas.push(caminho);
    }
    let tarefas = raiz.join(projeto::pasta_das_tarefas(&slug));
    std::fs::create_dir_all(&tarefas).map_err(|e| format!("erro criando {}: {e}", tarefas.display()))?;
    Ok(criado)
}

/// Os projetos do vault, em ordem de slug.
///
/// Lê só `pages/projetos/*/projeto.md`. Uma pasta sem página de projeto
/// (ou com `type` errado) não é projeto, e fica de fora sem erro.
pub fn handle_listar_projetos(vault_path: String) -> Result<Vec<Projeto>, String> {
    let pasta = Path::new(&vault_path).join(projeto::PASTA);
    let Ok(entradas) = std::fs::read_dir(&pasta) else {
        return Ok(Vec::new());
    };
    let mut lista: Vec<Projeto> = entradas
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let slug = e.file_name().to_string_lossy().to_string();
            let caminho = projeto::caminho_do_projeto(&slug);
            let texto = handle_read_page(vault_path.clone(), caminho.clone()).ok()?;
            Projeto::ler(&caminho, &texto)
        })
        .collect();
    lista.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(lista)
}

/// As tarefas de um projeto, em ordem de número.
///
/// Lê só a pasta de tarefas: a fila checa isto a cada poucos segundos, e
/// varrer as 250 páginas do vault pra olhar dez seria o tipo de custo
/// que só aparece quando o vault cresce.
pub fn handle_tarefas_do_projeto(vault_path: String, slug: String) -> Result<Vec<Tarefa>, String> {
    let pasta = projeto::pasta_das_tarefas(&slug);
    let Ok(entradas) = std::fs::read_dir(Path::new(&vault_path).join(&pasta)) else {
        return Ok(Vec::new());
    };
    let mut lista: Vec<Tarefa> = entradas
        .flatten()
        .filter_map(|e| {
            let nome = e.file_name().to_string_lossy().to_string();
            let caminho = format!("{pasta}/{nome}");
            if !projeto::e_caminho_de_tarefa(&caminho) {
                return None;
            }
            let texto = handle_read_page(vault_path.clone(), caminho.clone()).ok()?;
            Tarefa::ler(&caminho, &texto)
        })
        .collect();
    lista.sort_by(|a, b| (a.numero, &a.caminho).cmp(&(b.numero, &b.caminho)));
    Ok(lista)
}

/// O texto de uma tarefa nova, e onde ela vai morar.
///
/// Não grava: a GUI e a TUI gravam (é a pessoa criando), o CLI propõe (é
/// o canal do agente). O número sai da pasta como ela está agora.
pub fn handle_montar_tarefa(
    vault_path: String,
    slug: String,
    titulo: String,
    corpo: String,
    hoje: String,
) -> Result<(String, String), String> {
    if titulo.trim().is_empty() {
        return Err("a tarefa precisa de um título".to_string());
    }
    let pasta = Path::new(&vault_path).join(projeto::pasta_das_tarefas(&slug));
    let existentes: Vec<String> = std::fs::read_dir(&pasta)
        .map(|it| it.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    let numero = projeto::proximo_numero(&existentes);
    let caminho = projeto::caminho_da_tarefa(&slug, numero, &titulo);
    Ok((caminho, projeto::montar_tarefa(&slug, &titulo, &corpo, &hoje)))
}

/// Move a etapa de uma tarefa, como a PESSOA.
///
/// Não existe variante que receba `Quem`: `executando` e `revisao` são
/// registro do que o agente fez, e o único jeito de chegar lá é pela
/// fila (`handle_tarefas_a_executar`) e pelo fechamento
/// (`handle_fechar_tarefa`).
pub fn handle_mudar_etapa_tarefa(
    vault_path: String,
    caminho: String,
    destino: EtapaTarefa,
) -> Result<String, String> {
    let atual = handle_read_page(vault_path.clone(), caminho.clone())?;
    let novo = projeto::mudar_etapa(&atual, destino, Quem::Humano)?;
    handle_write_page(vault_path, caminho, novo.clone())?;
    Ok(novo)
}

/// A config dos projetos. Sem arquivo, vazia (cada projeto vale o padrão).
///
/// Arquivo ilegível é ERRO, e não padrão: o padrão tem `pausado: false`,
/// e uma config quebrada que despausasse a fila sem ninguém pedir é o
/// tipo de surpresa que custa um repositório.
pub fn handle_ler_projetos_cfg(vault_path: String) -> Result<ConfigProjetos, String> {
    let arquivo = Path::new(&vault_path).join(projeto::ARQUIVO_CONFIG);
    let Ok(texto) = std::fs::read_to_string(&arquivo) else {
        return Ok(ConfigProjetos::new());
    };
    serde_json::from_str(&texto).map_err(|e| format!("projetos.json ilegível: {e}"))
}

/// Grava a config dos projetos.
pub fn handle_gravar_projetos_cfg(vault_path: String, config: ConfigProjetos) -> Result<(), String> {
    let arquivo = Path::new(&vault_path).join(projeto::ARQUIVO_CONFIG);
    if let Some(pai) = arquivo.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&arquivo, json).map_err(|e| format!("erro gravando projetos.json: {e}"))
}

/// Uma tarefa reservada e pronta pra ir pro agente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrabalhoDeTarefa {
    /// Slug do projeto.
    pub projeto: String,
    /// Caminho da tarefa, já em `executando`.
    pub tarefa: String,
    /// A conversa onde a resposta vai ser gravada.
    pub conversa: String,
    /// O prompt exato, já montado e podado por `envio::montar`.
    pub prompt: String,
    /// Diretório de trabalho do agente: o repositório do projeto.
    pub cwd: String,
}

/// O que a checagem da fila achou.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FilaDosProjetos {
    /// O que iniciar agora. Já reservado: quem chama TEM que iniciar ou
    /// fechar como bloqueada, senão a tarefa fica presa em `executando`.
    pub iniciar: Vec<TrabalhoDeTarefa>,
    /// Por que algo que estava em `executar` não saiu, pra tela contar.
    pub avisos: Vec<String>,
}

/// Quanto tempo uma trava pode ficar sem dono antes de ser considerada
/// abandonada. A checagem toda leva milissegundos; um processo que morreu
/// segurando a trava não pode parar a fila pra sempre.
const TRAVA_ABANDONADA_S: u64 = 30;

/// A trava de arquivo que deixa UM processo reservar por vez.
///
/// GUI e TUI no mesmo vault checam a fila cada uma no seu relógio. Sem a
/// trava, as duas podem ler `executar` ao mesmo tempo e as duas
/// reservarem: a releitura só protege quem chega DEPOIS da gravação do
/// outro. `create_new` é atômico no sistema de arquivos, nos dois
/// sistemas.
struct Trava(PathBuf);

impl Trava {
    fn pegar(vault_path: &str) -> Result<Trava, String> {
        let arquivo = Path::new(vault_path).join(".anotadinho/projetos.lock");
        if let Some(pai) = arquivo.parent() {
            std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
        }
        for _ in 0..2 {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&arquivo) {
                Ok(_) => return Ok(Trava(arquivo)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let velha = std::fs::metadata(&arquivo)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|d| d.as_secs() >= TRAVA_ABANDONADA_S);
                    if !velha {
                        return Err("outra janela está conferindo a fila agora".to_string());
                    }
                    let _ = std::fs::remove_file(&arquivo);
                }
                Err(e) => return Err(format!("não consegui travar a fila: {e}")),
            }
        }
        Err("não consegui travar a fila".to_string())
    }
}

impl Drop for Trava {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// O que a fila precisa pra montar o prompt — é o que a UI já tem das
/// preferências.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParametrosDaFila {
    /// `"YYYY-MM-DD HH:MM"`: o ipc não tem relógio.
    pub agora: String,
    /// Teto de tokens do prompt (`0` desliga a poda).
    pub teto: usize,
    /// Quantas mensagens recentes da conversa da tarefa entram.
    pub limite_historico: usize,
}

/// Confere a fila de todos os projetos e RESERVA o que pode começar.
///
/// É a peça que a GUI e a TUI compartilham. Cada uma tem o seu executor
/// de agente (ciclo 459), mas a decisão do que rodar é uma só.
///
/// A vaga é contada pelo VAULT, e não pela memória do processo: uma
/// tarefa em `executando` ocupa vaga, seja qual for o frontend que a
/// pegou. Com isso, duas janelas abertas não somam concorrência.
pub fn handle_tarefas_a_executar(
    vault_path: String,
    params: ParametrosDaFila,
) -> Result<FilaDosProjetos, String> {
    let mut saida = FilaDosProjetos::default();
    let projetos = handle_listar_projetos(vault_path.clone())?;
    if projetos.is_empty() {
        return Ok(saida);
    }
    let config = handle_ler_projetos_cfg(vault_path.clone())?;

    // Barato primeiro: sem nada em `executar`, não há o que travar.
    let mut com_pedido: Vec<(Projeto, Vec<Tarefa>)> = Vec::new();
    for p in projetos {
        let tarefas = handle_tarefas_do_projeto(vault_path.clone(), p.slug.clone())?;
        if tarefas.iter().any(|t| t.etapa == EtapaTarefa::Executar) {
            com_pedido.push((p, tarefas));
        }
    }
    if com_pedido.is_empty() {
        return Ok(saida);
    }

    let freio = handle_freio_do_dia(vault_path.clone(), params.agora.clone())?;
    if freio.parou() {
        saida.avisos.push(format!("fila parada pelo freio do dia: {}", freio.motivo()));
        return Ok(saida);
    }

    let _trava = Trava::pegar(&vault_path)?;
    for (p, tarefas) in com_pedido {
        let cfg = config.get(&p.slug).cloned().unwrap_or_default();
        if cfg.pausado {
            saida.avisos.push(format!("{}: fila pausada", p.slug));
            continue;
        }
        let Some(repositorio) = cfg.repositorio.clone().filter(|r| !r.trim().is_empty()) else {
            saida.avisos.push(format!("{}: sem repositório configurado — escolha a pasta do código", p.slug));
            continue;
        };
        if !Path::new(&repositorio).is_dir() {
            saida.avisos.push(format!("{}: o repositório {repositorio} não é uma pasta", p.slug));
            continue;
        }
        let ocupadas = tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Executando).count();
        let vagas = cfg.concorrencia.max(1).saturating_sub(ocupadas);
        let fila = projeto::fila(&tarefas);
        for t in tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Executar) {
            let pendentes = projeto::dependencias_pendentes(t, &tarefas);
            if !pendentes.is_empty() {
                saida.avisos.push(format!("{}: esperando {}", t.caminho, pendentes.join(", ")));
            }
        }
        for t in fila.into_iter().take(vagas) {
            match reservar_e_preparar(&vault_path, &p, t, &tarefas, &repositorio, &params) {
                Ok(Some(trabalho)) => saida.iniciar.push(trabalho),
                Ok(None) => {}
                Err(e) => saida.avisos.push(format!("{}: {e}", t.caminho)),
            }
        }
    }
    Ok(saida)
}

/// Reserva uma tarefa e monta o que o agente recebe.
///
/// `None` = alguém mudou a tarefa entre a listagem e agora (a pessoa
/// desistiu, ou outro processo reservou antes da trava existir): não é
/// erro, só não é mais nossa.
fn reservar_e_preparar(
    vault_path: &str,
    p: &Projeto,
    lida: &Tarefa,
    todas: &[Tarefa],
    repositorio: &str,
    params: &ParametrosDaFila,
) -> Result<Option<TrabalhoDeTarefa>, String> {
    // Releitura: a listagem é de antes da trava.
    let texto = handle_read_page(vault_path.to_string(), lida.caminho.clone())?;
    let Some(tarefa) = Tarefa::ler(&lida.caminho, &texto) else { return Ok(None) };
    if tarefa.etapa != EtapaTarefa::Executar {
        return Ok(None);
    }

    // A conversa e a pergunta vêm ANTES da reserva: se gravar a conversa
    // falhar, a tarefa continua em `executar` e a próxima checagem tenta
    // de novo, em vez de ficar presa em `executando` sem agente nenhum.
    let conversa_path = projeto::caminho_da_conversa_da_tarefa(&p.slug, tarefa.numero);
    let conversa_atual = handle_read_page(vault_path.to_string(), conversa_path.clone()).ok();
    let base = conversa_atual.clone().unwrap_or_else(|| pagina_da_conversa(p, &tarefa));
    let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&base);
    let historico = conversa::parse(corpo);
    let pergunta = projeto::pergunta_de_executar(&tarefa);
    let minha = Mensagem { autor: Autor::Voce, quando: params.agora.clone(), texto: pergunta.clone() };
    let corpo_novo = conversa::append(corpo, &minha);
    let gravado = if fm.is_empty() { corpo_novo } else { format!("{fm}\n{corpo_novo}") };

    let contextos = contextos_da_execucao(vault_path, p, &tarefa, &texto, todas);
    let envio = anotadinho_core::envio::montar(
        &contextos,
        &historico,
        &pergunta,
        params.limite_historico,
        params.teto,
        vault_path,
    );

    handle_write_page(vault_path.to_string(), conversa_path.clone(), gravado)?;
    let reservado = projeto::mudar_etapa(&texto, EtapaTarefa::Executando, Quem::Sistema)?;
    handle_write_page(vault_path.to_string(), tarefa.caminho.clone(), reservado)?;

    Ok(Some(TrabalhoDeTarefa {
        projeto: p.slug.clone(),
        tarefa: tarefa.caminho.clone(),
        conversa: conversa_path,
        prompt: envio.prompt,
        cwd: repositorio.to_string(),
    }))
}

/// A página de conversa de uma tarefa que ainda não tinha.
fn pagina_da_conversa(p: &Projeto, tarefa: &Tarefa) -> String {
    let md = conversa::montar_pagina(
        &format!("Execução — {}", tarefa.titulo),
        Some(&tarefa.caminho),
        std::slice::from_ref(&tarefa.caminho),
    );
    let md = MarkdownCodec::set_frontmatter_field(&md, "projeto", &p.slug).unwrap_or(md);
    MarkdownCodec::set_frontmatter_field(&md, "tarefa", &tarefa.caminho).unwrap_or(md)
}

/// O que vai anexado na execução: a tarefa, o projeto, a arquitetura, o
/// índice das tarefas e a proposta vinculada, se houver.
///
/// A página que não dá pra ler fica de fora sem derrubar a execução: uma
/// arquitetura apagada não é motivo pra tarefa não rodar.
fn contextos_da_execucao(
    vault_path: &str,
    p: &Projeto,
    tarefa: &Tarefa,
    texto_da_tarefa: &str,
    todas: &[Tarefa],
) -> Vec<Contexto> {
    let ler = |c: &str| handle_read_page(vault_path.to_string(), c.to_string()).ok();
    let projeto_txt = ler(&p.caminho).unwrap_or_default();
    let arq_caminho = projeto::caminho_da_arquitetura(&p.slug);
    let arq = ler(&arq_caminho);
    let mut v = vec![Contexto { nome: tarefa.caminho.clone(), conteudo: texto_da_tarefa.to_string() }];
    v.extend(projeto::contexto_do_projeto(
        (&p.caminho, &projeto_txt),
        arq.as_deref().map(|a| (arq_caminho.as_str(), a)),
        todas,
    ));
    for vinculo in [tarefa.proposta.as_deref(), tarefa.spec.as_deref()].into_iter().flatten() {
        let caminho = vinculo.trim().trim_start_matches("[[").trim_end_matches("]]").trim();
        if let Some(conteudo) = ler(caminho) {
            v.push(Contexto { nome: caminho.to_string(), conteudo });
        }
    }
    v
}

/// Como uma execução de tarefa acabou, do ponto de vista do executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fim", content = "texto", rename_all = "kebab-case")]
pub enum FimDaTarefa {
    /// O agente respondeu: o resultado sai da resposta.
    Respondeu(String),
    /// O processo falhou ou não subiu.
    Falhou(String),
    /// A pessoa parou.
    Interrompida,
}

/// Fecha uma tarefa que estava em `executando`.
///
/// Só a etapa. A resposta na conversa e o registro de execuções já são
/// feitos pelos dois executores pra toda conversa; repetir aqui gravaria
/// a resposta duas vezes.
///
/// Se a tarefa não está mais em `executando` (a pessoa moveu pra
/// `bloqueada` durante a execução), não mexe: a decisão da pessoa vale
/// mais que o relato do agente. Devolve a etapa em que a tarefa ficou.
pub fn handle_fechar_tarefa(
    vault_path: String,
    caminho: String,
    fim: FimDaTarefa,
) -> Result<EtapaTarefa, String> {
    let texto = handle_read_page(vault_path.clone(), caminho.clone())?;
    let tarefa = Tarefa::ler(&caminho, &texto).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    if tarefa.etapa != EtapaTarefa::Executando {
        return Ok(tarefa.etapa);
    }
    let destino = match &fim {
        FimDaTarefa::Respondeu(resposta) => projeto::ler_resultado(resposta).destino(),
        FimDaTarefa::Falhou(_) | FimDaTarefa::Interrompida => EtapaTarefa::Bloqueada,
    };
    let novo = projeto::mudar_etapa(&texto, destino, Quem::Sistema)?;
    handle_write_page(vault_path, caminho, novo)?;
    Ok(destino)
}

#[cfg(test)]
mod testes {
    use super::*;
    use anotadinho_core::projeto::ConfigProjeto;

    const AGORA: &str = "2026-10-21 10:00";

    fn params() -> ParametrosDaFila {
        ParametrosDaFila { agora: AGORA.into(), teto: 0, limite_historico: 12 }
    }

    /// Vault com um projeto `saas`, um repositório de verdade e `n`
    /// tarefas, todas nascidas em `ideia`.
    fn montar(n: usize) -> (tempfile::TempDir, tempfile::TempDir, String, Vec<String>) {
        let vault = tempfile::TempDir::new().unwrap();
        let repo = tempfile::TempDir::new().unwrap();
        let v = vault.path().to_string_lossy().to_string();
        handle_criar_projeto(v.clone(), "SaaS".into(), Some("saas".into()), "2026-10-21".into()).unwrap();
        let mut cfg = ConfigProjetos::new();
        cfg.insert(
            "saas".into(),
            ConfigProjeto { repositorio: Some(repo.path().to_string_lossy().to_string()), ..Default::default() },
        );
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        let mut caminhos = Vec::new();
        for i in 0..n {
            let (c, md) =
                handle_montar_tarefa(v.clone(), "saas".into(), format!("Tarefa {i}"), String::new(), "2026-10-21".into())
                    .unwrap();
            handle_write_page(v.clone(), c.clone(), md).unwrap();
            caminhos.push(c);
        }
        (vault, repo, v, caminhos)
    }

    fn pedir_execucao(v: &str, caminho: &str) {
        handle_mudar_etapa_tarefa(v.into(), caminho.into(), EtapaTarefa::Pronta).unwrap();
        handle_mudar_etapa_tarefa(v.into(), caminho.into(), EtapaTarefa::Executar).unwrap();
    }

    fn etapa(v: &str, caminho: &str) -> EtapaTarefa {
        Tarefa::ler(caminho, &handle_read_page(v.into(), caminho.into()).unwrap()).unwrap().etapa
    }

    #[test]
    fn criar_de_novo_nao_sobrescreve() {
        let (_g, _r, v, _) = montar(0);
        let arq = projeto::caminho_da_arquitetura("saas");
        handle_write_page(v.clone(), arq.clone(), "---\ntype: docs\n---\nminha arquitetura\n".into()).unwrap();
        let de_novo = handle_criar_projeto(v.clone(), "SaaS".into(), Some("saas".into()), "hoje".into()).unwrap();
        assert!(de_novo.criadas.is_empty());
        assert_eq!(de_novo.existiam.len(), 3);
        assert!(handle_read_page(v, arq).unwrap().contains("minha arquitetura"));
    }

    #[test]
    fn lista_projetos_e_tarefas_numeradas() {
        let (_g, _r, v, caminhos) = montar(3);
        let ps = handle_listar_projetos(v.clone()).unwrap();
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].slug, "saas");
        let ts = handle_tarefas_do_projeto(v, "saas".into()).unwrap();
        assert_eq!(ts.iter().map(|t| t.numero).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(ts[0].caminho, caminhos[0]);
    }

    #[test]
    fn a_pessoa_nao_marca_executando_a_mao() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        assert!(handle_mudar_etapa_tarefa(v, c[0].clone(), EtapaTarefa::Executando).is_err());
    }

    #[test]
    fn reserva_uma_vez_so() {
        let (_g, repo, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let primeira = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert_eq!(primeira.iniciar.len(), 1, "{:?}", primeira.avisos);
        let t = &primeira.iniciar[0];
        assert_eq!(t.cwd, repo.path().to_string_lossy().as_ref());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executando);
        let segunda = handle_tarefas_a_executar(v, params()).unwrap();
        assert!(segunda.iniciar.is_empty(), "a mesma tarefa saiu duas vezes");
    }

    #[test]
    fn a_pergunta_fica_na_conversa_antes_do_agente() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        let conversa = handle_read_page(v, t.conversa.clone()).unwrap();
        let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&conversa);
        assert!(fm.contains("type: conversa"));
        assert!(fm.contains(&format!("tarefa: {}", c[0])), "{fm}");
        let msgs = conversa::parse(corpo);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].texto.contains(projeto::MARCA_DE_RESULTADO));
        // O prompt leva a tarefa, o projeto e o índice.
        assert!(t.prompt.contains(&c[0]));
        assert!(t.prompt.contains("projeto.md"));
        assert!(t.prompt.contains("(índice)"));
    }

    #[test]
    fn concorrencia_conta_o_que_ja_esta_executando() {
        let (_g, _r, v, c) = montar(2);
        pedir_execucao(&v, &c[0]);
        assert_eq!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.len(), 1);
        pedir_execucao(&v, &c[1]);
        // A primeira ainda roda: a vaga única está ocupada, por quem quer
        // que a tenha pegado.
        assert!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.is_empty());
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
            .unwrap();
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn pausada_nao_perde_o_pedido() {
        // O defeito que derrubou a memória por borda (ciclo 457): pedido
        // feito com a fila pausada tem que rodar quando ela voltar.
        let (_g, _r, v, c) = montar(1);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().pausado = true;
        handle_gravar_projetos_cfg(v.clone(), cfg.clone()).unwrap();
        pedir_execucao(&v, &c[0]);
        let r = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert!(r.iniciar.is_empty());
        assert!(r.avisos.iter().any(|a| a.contains("pausada")), "{:?}", r.avisos);
        cfg.get_mut("saas").unwrap().pausado = false;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn sem_repositorio_avisa_e_nao_reserva() {
        let (_g, _r, v, c) = montar(1);
        handle_gravar_projetos_cfg(v.clone(), ConfigProjetos::new()).unwrap();
        pedir_execucao(&v, &c[0]);
        let r = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert!(r.iniciar.is_empty());
        assert!(r.avisos.iter().any(|a| a.contains("sem repositório")), "{:?}", r.avisos);
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar, "não pode ficar presa em executando");
    }

    #[test]
    fn config_ilegivel_e_erro_e_nao_despausa() {
        let (_g, _r, v, c) = montar(1);
        std::fs::write(Path::new(&v).join(projeto::ARQUIVO_CONFIG), "{ quebrado").unwrap();
        pedir_execucao(&v, &c[0]);
        assert!(handle_tarefas_a_executar(v.clone(), params()).is_err());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar);
    }

    #[test]
    fn trava_ocupada_nao_reserva() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let outra = Trava::pegar(&v).unwrap();
        assert!(handle_tarefas_a_executar(v.clone(), params()).is_err());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar);
        drop(outra);
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn fechar_le_o_resultado() {
        let (_g, _r, v, c) = montar(3);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().concorrencia = 3;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        for x in &c {
            pedir_execucao(&v, x);
        }
        assert_eq!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.len(), 3);
        let fim = |i: usize, f: FimDaTarefa| handle_fechar_tarefa(v.clone(), c[i].clone(), f).unwrap();
        assert_eq!(fim(0, FimDaTarefa::Respondeu("pronto\nRESULTADO: concluida".into())), EtapaTarefa::Revisao);
        assert_eq!(fim(1, FimDaTarefa::Respondeu("acho que foi".into())), EtapaTarefa::Bloqueada);
        assert_eq!(fim(2, FimDaTarefa::Falhou("timeout".into())), EtapaTarefa::Bloqueada);
    }

    #[test]
    fn fechar_respeita_quem_mexeu_durante() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Bloqueada).unwrap();
        let ficou =
            handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
                .unwrap();
        assert_eq!(ficou, EtapaTarefa::Bloqueada);
    }

    #[test]
    fn a_segunda_execucao_continua_a_mesma_conversa() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
            .unwrap();
        // revisao → executar: "refaça".
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Executar).unwrap();
        let t2 = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        assert_eq!(t.conversa, t2.conversa);
        let conversa = handle_read_page(v, t2.conversa).unwrap();
        let (_, corpo) = MarkdownCodec::split_frontmatter_text(&conversa);
        assert_eq!(conversa::parse(corpo).len(), 2);
    }
}
