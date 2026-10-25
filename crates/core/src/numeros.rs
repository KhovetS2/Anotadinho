//! Os números de um projeto por papel (ciclo 524).
//!
//! O arranjo PM / tech lead / júnior só vale se o revisor caro custar
//! menos do que custaria o executor caro, e se a revisão convergir. Isto
//! junta o registro de execuções (`.anotadinho/execucoes.jsonl`) com o
//! estado das tarefas pra responder às duas perguntas com número.

use serde::{Deserialize, Serialize};

use crate::agente::Uso;
use crate::execucao::{Execucao, Fim};
use crate::projeto::{Tarefa, PASTA};
use crate::revisor::estado;

/// De quem é uma execução, pela conversa de onde saiu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quem {
    /// A conversa de uma tarefa (`conversas/tarefa-NNN.md`): execução e
    /// detalhe moram no mesmo fio.
    Executor,
    /// A conversa do revisor (`conversas/revisao-NNN.md`).
    Revisor,
    /// O resto do projeto: a conversa geral, a pessoa planejando.
    Conversas,
}

impl Quem {
    /// O papel da execução, se ela é deste projeto.
    pub fn da_conversa(conversa: &str, slug: &str) -> Option<Quem> {
        let resto = conversa.strip_prefix(&format!("{PASTA}/{slug}/"))?;
        let nome = resto.strip_prefix("conversas/").unwrap_or(resto);
        Some(if nome.starts_with("revisao-") {
            Quem::Revisor
        } else if nome.starts_with("tarefa-") {
            Quem::Executor
        } else {
            Quem::Conversas
        })
    }

    /// Como aparece na tela.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Executor => "executor",
            Self::Revisor => "revisor",
            Self::Conversas => "conversas",
        }
    }
}

/// O que um papel rodou.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DoPapel {
    /// Quantas execuções.
    pub execucoes: usize,
    /// Quantas falharam ou foram interrompidas.
    pub falhas: usize,
    /// O tempo somado.
    pub segundos: u64,
    /// Os tokens somados, e o custo das que contaram custo.
    pub uso: Uso,
    /// Quantas execuções disseram o custo: o resto (um modelo local) não
    /// entra no total, e a tela precisa dizer isso em vez de mostrar zero.
    pub com_custo: usize,
    /// Os agentes que rodaram neste papel, na ordem em que apareceram.
    pub agentes: Vec<String>,
}

/// Como a revisão pelo agente andou.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DaRevisao {
    /// Tarefas que o revisor terminou de revisar (aprovou, esgotou ou
    /// falhou).
    pub revisadas: usize,
    /// Aprovadas sem pedir ajuste nenhum.
    pub de_primeira: usize,
    /// Aprovadas depois de ajustes.
    pub depois_de_ajuste: usize,
    /// Chegaram no teto ainda pedindo ajuste.
    pub esgotaram: usize,
    /// O revisor não rodou ou não deu veredito.
    pub falharam: usize,
    /// Revisões por tarefa revisada, em média.
    pub media_de_revisoes: f64,
}

/// Os números de um projeto.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct DoProjeto {
    /// O executor.
    pub executor: DoPapel,
    /// O revisor.
    pub revisor: DoPapel,
    /// As outras conversas do projeto.
    pub conversas: DoPapel,
    /// A revisão.
    pub revisao: DaRevisao,
}

fn somar(p: &mut DoPapel, x: &Execucao) {
    p.execucoes += 1;
    if !matches!(x.fim, Fim::Respondeu) {
        p.falhas += 1;
    }
    p.segundos += x.segundos;
    if let Some(u) = &x.uso {
        p.uso = p.uso.somar(u);
        if u.custo_usd.is_some() {
            p.com_custo += 1;
        }
    }
    if !p.agentes.contains(&x.agente) {
        p.agentes.push(x.agente.clone());
    }
}

/// Os números de `slug`, das execuções do vault e das tarefas do projeto.
pub fn do_projeto(execucoes: &[Execucao], tarefas: &[Tarefa], slug: &str) -> DoProjeto {
    let mut n = DoProjeto::default();
    for x in execucoes {
        match Quem::da_conversa(&x.conversa, slug) {
            Some(Quem::Executor) => somar(&mut n.executor, x),
            Some(Quem::Revisor) => somar(&mut n.revisor, x),
            Some(Quem::Conversas) => somar(&mut n.conversas, x),
            None => {}
        }
    }
    let mut revisoes = 0u32;
    for t in tarefas.iter().filter(|t| t.projeto == slug) {
        // `rodada` conta as revisões que pediram ajuste; a que terminou o
        // laço (aprovando, esgotando) é mais uma — exceto no teto, onde a
        // última JÁ foi contada como ajuste.
        match t.revisao_ia.as_deref() {
            Some(estado::APROVADA) => {
                n.revisao.revisadas += 1;
                revisoes += t.rodada + 1;
                if t.rodada == 0 {
                    n.revisao.de_primeira += 1;
                } else {
                    n.revisao.depois_de_ajuste += 1;
                }
            }
            Some(estado::ESGOTOU) => {
                n.revisao.revisadas += 1;
                n.revisao.esgotaram += 1;
                revisoes += t.rodada.max(1);
            }
            Some(estado::FALHOU) => {
                n.revisao.revisadas += 1;
                n.revisao.falharam += 1;
                revisoes += t.rodada + 1;
            }
            _ => {}
        }
    }
    if n.revisao.revisadas > 0 {
        n.revisao.media_de_revisoes = f64::from(revisoes) / n.revisao.revisadas as f64;
    }
    n
}

#[cfg(test)]
mod testes {
    use super::*;

    fn exec(conversa: &str, agente: &str, uso: Option<Uso>, fim: Fim) -> Execucao {
        Execucao {
            quando: "2026-10-25 10:00".into(),
            conversa: conversa.into(),
            agente: agente.into(),
            binario: "x".into(),
            anexos: 0,
            prompt: 0,
            segundos: 10,
            uso,
            fim,
        }
    }

    fn tarefa(n: u32, revisao: Option<&str>, rodada: u32) -> Tarefa {
        let mut fm = format!("---\ntitle: T{n}\ntype: tarefa\nstatus: revisao\nrodada: {rodada}\n");
        if let Some(r) = revisao {
            fm.push_str(&format!("revisao_ia: {r}\n"));
        }
        fm.push_str("---\n");
        Tarefa::ler(&format!("{PASTA}/saas/tarefas/{n:03}-t.md"), &fm).unwrap()
    }

    #[test]
    fn o_papel_sai_da_conversa_e_o_projeto_filtra() {
        assert_eq!(Quem::da_conversa(&format!("{PASTA}/saas/conversas/tarefa-003.md"), "saas"), Some(Quem::Executor));
        assert_eq!(Quem::da_conversa(&format!("{PASTA}/saas/conversas/revisao-003.md"), "saas"), Some(Quem::Revisor));
        assert_eq!(Quem::da_conversa(&format!("{PASTA}/saas/conversas/geral.md"), "saas"), Some(Quem::Conversas));
        assert_eq!(Quem::da_conversa(&format!("{PASTA}/outro/conversas/tarefa-003.md"), "saas"), None);
        assert_eq!(Quem::da_conversa("pages/conversas/x.md", "saas"), None);
    }

    #[test]
    fn soma_por_papel_e_diz_quantas_contaram_custo() {
        let c = |x: &str| format!("{PASTA}/saas/conversas/{x}.md");
        let xs = vec![
            exec(&c("tarefa-001"), "aider + Ollama (local)", Some(Uso { entrada: 800, saida: 60, custo_usd: None }), Fim::Respondeu),
            exec(&c("tarefa-001"), "aider + Ollama (local)", None, Fim::Falhou("x".into())),
            exec(&c("revisao-001"), "Claude Code (revisor)", Some(Uso { entrada: 12_000, saida: 800, custo_usd: Some(0.04) }), Fim::Respondeu),
            exec(&c("geral"), "Claude Code", None, Fim::Respondeu),
            exec(&format!("{PASTA}/outro/conversas/tarefa-001.md"), "x", None, Fim::Respondeu),
        ];
        let n = do_projeto(&xs, &[], "saas");
        assert_eq!((n.executor.execucoes, n.executor.falhas, n.executor.segundos), (2, 1, 20));
        assert_eq!((n.executor.uso.entrada, n.executor.com_custo), (800, 0));
        assert_eq!(n.executor.agentes, ["aider + Ollama (local)"]);
        assert_eq!((n.revisor.execucoes, n.revisor.com_custo, n.revisor.uso.custo_usd), (1, 1, Some(0.04)));
        assert_eq!(n.conversas.execucoes, 1, "a do outro projeto não conta");
    }

    #[test]
    fn a_revisao_conta_de_primeira_ajustes_e_teto() {
        let ts = vec![
            tarefa(1, Some(estado::APROVADA), 0),
            tarefa(2, Some(estado::APROVADA), 2),
            tarefa(3, Some(estado::ESGOTOU), 3),
            tarefa(4, Some(estado::FALHOU), 0),
            tarefa(5, None, 0),
            tarefa(6, Some(estado::RODANDO), 1),
        ];
        let r = do_projeto(&[], &ts, "saas").revisao;
        assert_eq!((r.revisadas, r.de_primeira, r.depois_de_ajuste, r.esgotaram, r.falharam), (4, 1, 1, 1, 1));
        // 1 + 3 + 3 + 1 revisões em 4 tarefas.
        assert!((r.media_de_revisoes - 2.0).abs() < 1e-9, "{}", r.media_de_revisoes);
    }
}
