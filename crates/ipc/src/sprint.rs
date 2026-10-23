//! IO das sprints e do plano de projeto (ciclo 470).
//!
//! Duas famílias, e a diferença entre elas é a regra do projeto:
//!
//! - **pela pessoa** (`criar`, `iniciar`, `encerrar`, `mudar`): a janela e
//!   a TUI chamam, e grava direto — é a pessoa decidindo;
//! - **pelo agente** (`propor_*`): o CLI e o MCP chamam, e tudo vira
//!   proposta na fila de revisão. O plano inteiro é UM lote.
//!
//! A regra (o que é válido, pra onde vai cada tarefa) é do
//! `core::sprint` e do `core::plano`; aqui só se lê e se grava.

use std::path::Path;

use anotadinho_core::plano::{self, JaExiste};
use anotadinho_core::projeto::{self, Tarefa};
use anotadinho_core::sprint::{self, DestinoDasPendentes, Encerramento, EstadoSprint, Sprint};
use serde::{Deserialize, Serialize};

use crate::projeto::proposta_nova;
use crate::{handle_propor, handle_read_page, handle_tarefas_do_projeto, handle_write_page};

/// As sprints de um projeto, por número. Lê só `sprints/`.
pub fn handle_sprints_do_projeto(vault_path: String, slug: String) -> Result<Vec<Sprint>, String> {
    let pasta = sprint::pasta_das_sprints(&slug);
    let Ok(entradas) = std::fs::read_dir(Path::new(&vault_path).join(&pasta)) else {
        return Ok(Vec::new());
    };
    let mut lista: Vec<Sprint> = entradas
        .flatten()
        .filter_map(|e| {
            let caminho = format!("{pasta}/{}", e.file_name().to_string_lossy());
            if !sprint::e_caminho_de_sprint(&caminho) {
                return None;
            }
            let texto = handle_read_page(vault_path.clone(), caminho.clone()).ok()?;
            Sprint::ler(&caminho, &texto)
        })
        .collect();
    lista.sort_by(|a, b| (a.numero, &a.caminho).cmp(&(b.numero, &b.caminho)));
    Ok(lista)
}

fn nomes_na_pasta(vault_path: &str, pasta: &str) -> Vec<String> {
    std::fs::read_dir(Path::new(vault_path).join(pasta))
        .map(|it| it.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default()
}

/// O que já existe no projeto, pra o plano numerar sem colidir.
fn ja_existe(vault_path: &str, slug: &str) -> Result<JaExiste, String> {
    let sprints = handle_sprints_do_projeto(vault_path.to_string(), slug.to_string())?;
    Ok(JaExiste {
        projeto: Path::new(vault_path).join(projeto::caminho_do_projeto(slug)).exists(),
        proxima_tarefa: projeto::proximo_numero(&nomes_na_pasta(vault_path, &projeto::pasta_das_tarefas(slug))),
        proxima_sprint: projeto::proximo_numero(&nomes_na_pasta(vault_path, &sprint::pasta_das_sprints(slug))),
        sprints: sprints.into_iter().map(|s| (s.nome, s.numero)).collect(),
    })
}

fn sprint_por_caminho(vault_path: &str, caminho: &str) -> Result<(Sprint, String, Vec<Sprint>), String> {
    let texto = handle_read_page(vault_path.to_string(), caminho.to_string())?;
    let s = Sprint::ler(caminho, &texto).ok_or_else(|| format!("{caminho} não é uma sprint"))?;
    let todas = handle_sprints_do_projeto(vault_path.to_string(), s.projeto.clone())?;
    Ok((s, texto, todas))
}

/// A sprint de um projeto pelo NÚMERO ou pelo NOME — o que a pessoa e a
/// LLM digitam no CLI.
pub fn achar_sprint(sprints: &[Sprint], referencia: &str) -> Option<Sprint> {
    let r = referencia.trim();
    r.parse::<u32>()
        .ok()
        .and_then(|n| sprints.iter().find(|s| s.numero == n))
        .or_else(|| sprints.iter().find(|s| s.nome.trim().eq_ignore_ascii_case(r)))
        .cloned()
}

// ── pela pessoa ─────────────────────────────────────────────────────

/// Cria uma sprint planejada e devolve o caminho.
pub fn handle_criar_sprint(
    vault_path: String,
    slug: String,
    nome: String,
    objetivo: String,
    inicio: Option<String>,
    fim: Option<String>,
    hoje: String,
) -> Result<String, String> {
    if nome.trim().is_empty() {
        return Err("a sprint precisa de um nome".to_string());
    }
    if !Path::new(&vault_path).join(projeto::caminho_do_projeto(&slug)).exists() {
        return Err(format!("não existe o projeto {slug}"));
    }
    let numero = projeto::proximo_numero(&nomes_na_pasta(&vault_path, &sprint::pasta_das_sprints(&slug)));
    let caminho = sprint::caminho_da_sprint(&slug, numero, &nome);
    let md = sprint::montar_sprint(&slug, &nome, &objetivo, inicio.as_deref(), fim.as_deref(), &hoje);
    handle_write_page(vault_path, caminho.clone(), md)?;
    Ok(caminho)
}

/// Inicia uma sprint planejada. Com outra ativa no projeto, recusa.
pub fn handle_iniciar_sprint(vault_path: String, caminho: String) -> Result<(), String> {
    let (_, texto, todas) = sprint_por_caminho(&vault_path, &caminho)?;
    let novo = sprint::mudar_estado(&texto, &caminho, EstadoSprint::Ativa, &todas)?;
    handle_write_page(vault_path, caminho, novo)
}

/// Encerra a sprint ativa: grava as pendentes no destino e a sprint como
/// encerrada.
///
/// As TAREFAS primeiro, a sprint por último: se uma gravação falhar no
/// meio, a sprint continua ativa e encerrar de novo termina o serviço.
/// Na ordem inversa, uma falha deixaria pendentes presas numa sprint já
/// encerrada, invisíveis no quadro.
pub fn handle_encerrar_sprint(
    vault_path: String,
    caminho: String,
    destino: DestinoDasPendentes,
) -> Result<Encerramento, String> {
    let (s, texto, todas) = sprint_por_caminho(&vault_path, &caminho)?;
    let tarefas = handle_tarefas_do_projeto(vault_path.clone(), s.projeto.clone())?;
    let plano = sprint::plano_de_encerramento(&s, &todas, &tarefas, destino)?;
    for (tarefa, nova) in &plano.mudancas {
        let atual = handle_read_page(vault_path.clone(), tarefa.clone())?;
        handle_write_page(vault_path.clone(), tarefa.clone(), sprint::mudar_sprint_da_tarefa(&atual, *nova)?)?;
    }
    let novo = sprint::mudar_estado(&texto, &caminho, EstadoSprint::Encerrada, &todas)?;
    handle_write_page(vault_path, caminho, novo)?;
    Ok(plano)
}

/// Põe uma tarefa numa sprint, ou no backlog (`None`).
pub fn handle_mudar_sprint_da_tarefa(vault_path: String, caminho: String, sprint: Option<u32>) -> Result<(), String> {
    let atual = handle_read_page(vault_path.clone(), caminho.clone())?;
    let tarefa = Tarefa::ler(&caminho, &atual).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    if let Some(n) = sprint {
        let sprints = handle_sprints_do_projeto(vault_path.clone(), tarefa.projeto.clone())?;
        let s = sprints.iter().find(|s| s.numero == n).ok_or_else(|| format!("não existe a sprint {n} em {}", tarefa.projeto))?;
        if s.estado == EstadoSprint::Encerrada {
            return Err(format!("a sprint \"{}\" está encerrada: tarefa nova não entra no histórico", s.nome));
        }
    }
    handle_write_page(vault_path, caminho, sprint::mudar_sprint_da_tarefa(&atual, sprint)?)
}

// ── pelo agente: tudo propõe ────────────────────────────────────────

/// O que propor um plano gerou.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanoProposto {
    /// O slug do projeto.
    pub slug: String,
    /// O lote: aplicar ou recusar é UMA decisão.
    pub lote: String,
    /// Os ids das propostas, na ordem.
    pub ids: Vec<String>,
    /// O que não impediu, mas vale dizer.
    pub avisos: Vec<String>,
}

/// Propõe o projeto inteiro de um plano YAML, num lote só (RNF1).
pub fn handle_propor_plano(vault_path: String, yaml: String, quando: String, autor: String) -> Result<PlanoProposto, String> {
    let p = plano::ler_plano(&yaml)?;
    let slug = anotadinho_core::fluxo::slug_de_titulo(p.projeto.slug.as_deref().unwrap_or(p.projeto.titulo.trim()));
    let ja = ja_existe(&vault_path, &slug)?;
    let hoje = quando.split_whitespace().next().unwrap_or("").to_string();
    let r = plano::montar_plano(&p, &ja, &hoje)?;
    let lote = format!("plano-{slug}-{}", anotadinho_core::fluxo::slug_de_titulo(&quando));
    let mut ids = Vec::new();
    for (caminho, conteudo) in r.paginas {
        let proposta = proposta_nova(
            &vault_path,
            &autor,
            caminho,
            conteudo,
            format!("plano do projeto {}", p.projeto.titulo.trim()),
            quando.clone(),
            Some(lote.clone()),
        );
        ids.push(handle_propor(vault_path.clone(), proposta)?);
    }
    Ok(PlanoProposto { slug: r.slug, lote, ids, avisos: r.avisos })
}

/// Propõe uma sprint planejada.
// Campos soltos, e não uma struct: espelham um a um os parâmetros da
// ferramenta do MCP e do comando do CLI que chamam isto.
#[allow(clippy::too_many_arguments)]
pub fn handle_propor_sprint(
    vault_path: String,
    slug: String,
    nome: String,
    objetivo: String,
    inicio: Option<String>,
    fim: Option<String>,
    quando: String,
    autor: String,
) -> Result<String, String> {
    if nome.trim().is_empty() {
        return Err("a sprint precisa de um nome".to_string());
    }
    if !Path::new(&vault_path).join(projeto::caminho_do_projeto(&slug)).exists() {
        return Err(format!("não existe o projeto {slug}"));
    }
    let numero = projeto::proximo_numero(&nomes_na_pasta(&vault_path, &sprint::pasta_das_sprints(&slug)));
    let hoje = quando.split_whitespace().next().unwrap_or("").to_string();
    let md = sprint::montar_sprint(&slug, &nome, &objetivo, inicio.as_deref(), fim.as_deref(), &hoje);
    let caminho = sprint::caminho_da_sprint(&slug, numero, &nome);
    let p = proposta_nova(&vault_path, &autor, caminho, md, format!("sprint nova: {}", nome.trim()), quando, None);
    handle_propor(vault_path, p)
}

/// Propõe uma tarefa, já numa sprint se `sprint` vier (nome ou número).
// Campos soltos, e não uma struct: espelham um a um os parâmetros da
// ferramenta do MCP e do comando do CLI que chamam isto.
#[allow(clippy::too_many_arguments)]
pub fn handle_propor_tarefa(
    vault_path: String,
    slug: String,
    titulo: String,
    corpo: String,
    sprint_ref: Option<String>,
    tags: Vec<String>,
    prazo: Option<String>,
    quando: String,
    lote: Option<String>,
    autor: String,
) -> Result<String, String> {
    if !Path::new(&vault_path).join(projeto::caminho_do_projeto(&slug)).exists() {
        return Err(format!("não existe o projeto {slug} (crie com `projeto novo` ou `projeto montar`)"));
    }
    let hoje = quando.split_whitespace().next().unwrap_or("").to_string();
    let (caminho, mut md) = crate::handle_montar_tarefa(vault_path.clone(), slug.clone(), titulo.clone(), corpo, hoje)?;
    if let Some(r) = sprint_ref.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
        let sprints = handle_sprints_do_projeto(vault_path.clone(), slug.clone())?;
        let s = achar_sprint(&sprints, r).ok_or_else(|| {
            let nomes: Vec<String> = sprints.iter().map(|s| format!("{} ({})", s.nome, s.numero)).collect();
            format!("não existe a sprint \"{r}\" em {slug}. Existem: {}", if nomes.is_empty() { "nenhuma".into() } else { nomes.join(", ") })
        })?;
        md = sprint::mudar_sprint_da_tarefa(&md, Some(s.numero))?;
    }
    // Tags e prazo (ciclo 476): o prazo inválido é recusado AQUI, antes de
    // virar proposta — aplicar uma proposta com "5/11" gravaria um prazo
    // que nunca vence.
    if !tags.is_empty() {
        md = projeto::com_tags(&md, &tags)?;
    }
    if let Some(p) = prazo.as_deref().filter(|p| !p.trim().is_empty()) {
        md = projeto::com_prazo(&md, Some(p))?;
    }
    let p = proposta_nova(&vault_path, &autor, caminho, md, format!("tarefa nova: {}", titulo.trim()), quando, lote);
    handle_propor(vault_path, p)
}

/// As propostas que encerrar uma sprint causaria, num lote (o CLI).
pub fn handle_propor_encerramento(
    vault_path: String,
    caminho: String,
    destino: DestinoDasPendentes,
    quando: String,
    autor: String,
) -> Result<Vec<String>, String> {
    let (s, texto, todas) = sprint_por_caminho(&vault_path, &caminho)?;
    let tarefas = handle_tarefas_do_projeto(vault_path.clone(), s.projeto.clone())?;
    let plano = sprint::plano_de_encerramento(&s, &todas, &tarefas, destino)?;
    let lote = format!("encerrar-{}-{}", s.projeto, s.numero);
    let mut ids = Vec::new();
    for (tarefa, nova) in &plano.mudancas {
        let atual = handle_read_page(vault_path.clone(), tarefa.clone())?;
        let p = proposta_nova(
            &vault_path,
            &autor,
            tarefa.clone(),
            sprint::mudar_sprint_da_tarefa(&atual, *nova)?,
            format!("encerrar \"{}\": pendente vai pra {}", s.nome, nova.map(|n| format!("sprint {n}")).unwrap_or("o backlog".into())),
            quando.clone(),
            Some(lote.clone()),
        );
        ids.push(handle_propor(vault_path.clone(), p)?);
    }
    let novo = sprint::mudar_estado(&texto, &caminho, EstadoSprint::Encerrada, &todas)?;
    let p = proposta_nova(&vault_path, &autor, caminho, novo, format!("encerrar a sprint \"{}\"", s.nome), quando, Some(lote));
    ids.push(handle_propor(vault_path, p)?);
    Ok(ids)
}

#[cfg(test)]
mod testes {
    use super::*;

    const AGORA: &str = "2026-10-22 10:00";

    fn vault_com_projeto() -> (tempfile::TempDir, String) {
        let d = tempfile::TempDir::new().unwrap();
        let v = d.path().to_string_lossy().to_string();
        crate::handle_criar_projeto(v.clone(), "SaaS".into(), Some("saas".into()), "2026-10-22".into()).unwrap();
        (d, v)
    }

    fn nova_tarefa(v: &str, titulo: &str) -> String {
        crate::handle_criar_tarefa(v.into(), "saas".into(), titulo.into(), "2026-10-22".into()).unwrap()
    }

    fn sprint_da(v: &str, c: &str) -> Option<u32> {
        Tarefa::ler(c, &handle_read_page(v.into(), c.into()).unwrap()).unwrap().sprint
    }

    #[test]
    fn criar_iniciar_e_recusar_a_segunda_ativa() {
        let (_d, v) = vault_com_projeto();
        let a = handle_criar_sprint(v.clone(), "saas".into(), "Fundação".into(), "base".into(), None, None, "hoje".into()).unwrap();
        let b = handle_criar_sprint(v.clone(), "saas".into(), "Agenda".into(), String::new(), None, None, "hoje".into()).unwrap();
        assert_eq!((a.as_str(), b.as_str()), ("pages/projetos/saas/sprints/01-fundacao.md", "pages/projetos/saas/sprints/02-agenda.md"));
        handle_iniciar_sprint(v.clone(), a.clone()).unwrap();
        let erro = handle_iniciar_sprint(v.clone(), b.clone()).unwrap_err();
        assert!(erro.contains("Fundação"), "{erro}");
        let e = crate::handle_estado_do_projeto(v, "saas".into()).unwrap();
        assert_eq!(e.sprints.len(), 2);
        assert_eq!(sprint::ativa(&e.sprints).map(|s| s.numero), Some(1));
    }

    #[test]
    fn encerrar_move_as_pendentes_e_grava_a_sprint_por_ultimo() {
        let (_d, v) = vault_com_projeto();
        let a = handle_criar_sprint(v.clone(), "saas".into(), "Um".into(), String::new(), None, None, "hoje".into()).unwrap();
        handle_criar_sprint(v.clone(), "saas".into(), "Dois".into(), String::new(), None, None, "hoje".into()).unwrap();
        handle_iniciar_sprint(v.clone(), a.clone()).unwrap();
        let t1 = nova_tarefa(&v, "Feita");
        let t2 = nova_tarefa(&v, "Pendente");
        for t in [&t1, &t2] {
            handle_mudar_sprint_da_tarefa(v.clone(), t.clone(), Some(1)).unwrap();
        }
        // A 1 feita (à mão, como faz quem fecha sem executar).
        let txt = handle_read_page(v.clone(), t1.clone()).unwrap();
        handle_write_page(v.clone(), t1.clone(), anotadinho_core::MarkdownCodec::set_frontmatter_field(&txt, "status", "feita").unwrap()).unwrap();
        let r = handle_encerrar_sprint(v.clone(), a.clone(), DestinoDasPendentes::ProximaSprint).unwrap();
        assert_eq!(r.feitas, 1);
        assert_eq!(sprint_da(&v, &t1), Some(1), "a feita fica no histórico da sprint");
        assert_eq!(sprint_da(&v, &t2), Some(2), "a pendente vai pra próxima");
        let (s, _, _) = sprint_por_caminho(&v, &a).unwrap();
        assert_eq!(s.estado, EstadoSprint::Encerrada);
        // Encerrada não recebe tarefa nova.
        assert!(handle_mudar_sprint_da_tarefa(v, t2, Some(1)).unwrap_err().contains("encerrada"));
    }

    #[test]
    fn encerrar_sem_planejada_pra_receber_nao_grava_nada() {
        let (_d, v) = vault_com_projeto();
        let a = handle_criar_sprint(v.clone(), "saas".into(), "Um".into(), String::new(), None, None, "hoje".into()).unwrap();
        handle_iniciar_sprint(v.clone(), a.clone()).unwrap();
        let t = nova_tarefa(&v, "Pendente");
        handle_mudar_sprint_da_tarefa(v.clone(), t.clone(), Some(1)).unwrap();
        assert!(handle_encerrar_sprint(v.clone(), a.clone(), DestinoDasPendentes::ProximaSprint).is_err());
        assert_eq!(sprint_da(&v, &t), Some(1));
        assert_eq!(sprint_por_caminho(&v, &a).unwrap().0.estado, EstadoSprint::Ativa);
    }

    #[test]
    fn plano_num_vault_vazio_vira_um_lote_e_nao_grava() {
        let d = tempfile::TempDir::new().unwrap();
        let v = d.path().to_string_lossy().to_string();
        let r = handle_propor_plano(v.clone(), plano::MODELO_DE_PLANO.into(), AGORA.into(), "teste".into()).unwrap();
        assert_eq!(r.slug, "agenda");
        // projeto, arquitetura, conversa, 2 sprints, 2 tarefas.
        assert_eq!(r.ids.len(), 7, "{:?}", r.ids);
        let props = crate::handle_listar_propostas(v.clone()).unwrap();
        assert!(props.iter().all(|p| p.lote.as_deref() == Some(r.lote.as_str())));
        assert!(!Path::new(&v).join("pages/projetos/agenda/projeto.md").exists(), "gravou em vez de propor");
    }

    #[test]
    fn plano_num_projeto_que_existe_numera_depois() {
        let (_d, v) = vault_com_projeto();
        nova_tarefa(&v, "Já existia");
        handle_criar_sprint(v.clone(), "saas".into(), "Um".into(), String::new(), None, None, "hoje".into()).unwrap();
        let yaml = "projeto:\n  titulo: SaaS\n  slug: saas\nsprints:\n  - nome: Dois\ntarefas:\n  - titulo: Nova\n    sprint: Um\n";
        handle_propor_plano(v.clone(), yaml.into(), AGORA.into(), "teste".into()).unwrap();
        let mut alvos: Vec<String> = crate::handle_listar_propostas(v).unwrap().into_iter().map(|p| p.alvo).collect();
        alvos.sort();
        assert_eq!(alvos, ["pages/projetos/saas/sprints/02-dois.md", "pages/projetos/saas/tarefas/002-nova.md"]);
    }

    #[test]
    fn propor_tarefa_acha_a_sprint_pelo_nome_ou_diz_quais_existem() {
        let (_d, v) = vault_com_projeto();
        handle_criar_sprint(v.clone(), "saas".into(), "Fundação".into(), String::new(), None, None, "hoje".into()).unwrap();
        handle_propor_tarefa(v.clone(), "saas".into(), "Login".into(), String::new(), Some("fundação".into()), vec!["auth".into()], Some("2026-11-05".into()), AGORA.into(), None, "teste".into()).unwrap();
        let p = crate::handle_listar_propostas(v.clone()).unwrap().remove(0);
        assert_eq!(Tarefa::ler(&p.alvo, &p.conteudo).unwrap().sprint, Some(1));
        let erro = handle_propor_tarefa(v.clone(), "saas".into(), "X".into(), String::new(), Some("Nada".into()), vec![], None, AGORA.into(), None, "teste".into()).unwrap_err();
        // Prazo que não é data é recusado antes de virar proposta.
        assert!(handle_propor_tarefa(v, "saas".into(), "Y".into(), String::new(), None, vec![], Some("5/11".into()), AGORA.into(), None, "teste".into()).is_err());
        assert!(erro.contains("Fundação (1)"), "{erro}");
    }

    #[test]
    fn propor_encerramento_vem_num_lote_com_a_sprint_por_ultimo() {
        let (_d, v) = vault_com_projeto();
        let a = handle_criar_sprint(v.clone(), "saas".into(), "Um".into(), String::new(), None, None, "hoje".into()).unwrap();
        handle_iniciar_sprint(v.clone(), a.clone()).unwrap();
        let t = nova_tarefa(&v, "Pendente");
        handle_mudar_sprint_da_tarefa(v.clone(), t.clone(), Some(1)).unwrap();
        let ids = handle_propor_encerramento(v.clone(), a.clone(), DestinoDasPendentes::Backlog, AGORA.into(), "teste".into()).unwrap();
        assert_eq!(ids.len(), 2);
        let props = crate::handle_listar_propostas(v.clone()).unwrap();
        let tarefa = props.iter().find(|p| p.alvo == t).unwrap();
        assert_eq!(Tarefa::ler(&tarefa.alvo, &tarefa.conteudo).unwrap().sprint, None);
        assert_eq!(sprint_da(&v, &t), Some(1), "propor não grava");
    }
}
