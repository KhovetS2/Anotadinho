//! O registro das decisões sobre propostas do agente (ciclo 404).
//!
//! Aplicar ou recusar apagava a proposta e não deixava rastro: depois
//! ninguém sabia o que o agente propôs, o que entrou e o que foi
//! recusado — e por quê. Aqui cada decisão vira uma linha JSON num
//! arquivo só de acrescentar, dentro de `.anotadinho/`, fora de `pages/`.
//!
//! Uma linha por decisão, e não um arquivo por decisão, porque o que se
//! quer ler é a SEQUÊNCIA; e acrescentar linha é a escrita mais difícil
//! de corromper que existe.

use serde::{Deserialize, Serialize};

/// Arquivo do registro, relativo à raiz do vault.
pub const ARQUIVO: &str = ".anotadinho/decisoes.jsonl";

/// O que foi decidido.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Acao {
    /// A proposta entrou inteira.
    Aplicada,
    /// Só parte dos trechos entrou.
    Parcial {
        /// Quantos trechos entraram.
        aceitos: usize,
        /// De quantos.
        de: usize,
    },
    /// A pessoa editou o conteúdo proposto antes de gravar (ciclo 411).
    /// O que entrou não é o que o agente escreveu — e o registro precisa
    /// dizer isso, senão a auditoria credita ao agente texto humano.
    Editada,
    /// Nada entrou.
    Recusada,
}

impl Acao {
    /// Como se lê na tela.
    pub fn rotulo(&self) -> String {
        match self {
            Self::Aplicada => "aplicada".into(),
            Self::Parcial { aceitos, de } => format!("parcial ({aceitos}/{de})"),
            Self::Editada => "editada".into(),
            Self::Recusada => "recusada".into(),
        }
    }
}

/// Uma decisão tomada sobre uma proposta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decisao {
    /// `"AAAA-MM-DD HH:MM"`.
    pub quando: String,
    /// O id da proposta.
    pub proposta: String,
    /// A página alvo.
    pub alvo: String,
    /// Quem propôs.
    pub autor: String,
    /// O que foi decidido.
    pub acao: Acao,
    /// Por que — o que a pessoa escreveu ao recusar, quando escreveu.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub motivo: String,
}

/// A linha JSON de uma decisão, pronta pra acrescentar (já com `\n`).
pub fn linha(d: &Decisao) -> String {
    match serde_json::to_string(d) {
        Ok(j) => format!("{j}\n"),
        Err(_) => String::new(),
    }
}

/// O que contar ao agente sobre as decisões desde `desde` (ciclo 421).
///
/// Fechar o laço: o agente propôs, a pessoa decidiu, e o único jeito de
/// ele saber era alguém digitar de novo. Aqui sai o texto pronto —
/// aplicadas, parciais e recusadas COM o motivo, que é a parte que
/// ensina.
///
/// `desde` é `"AAAA-MM-DD HH:MM"`; comparação de string basta porque o
/// formato é ordenável. Vazio pega tudo.
pub fn resumo_para_agente(decisoes: &[Decisao], desde: &str) -> String {
    let recentes: Vec<&Decisao> = decisoes.iter().filter(|d| d.quando.as_str() >= desde).collect();
    if recentes.is_empty() {
        return String::new();
    }
    let mut linhas: Vec<String> = Vec::new();
    for d in &recentes {
        let motivo = if d.motivo.trim().is_empty() {
            String::new()
        } else {
            format!(" — motivo: {}", d.motivo.trim())
        };
        linhas.push(format!("- {}: {}{motivo}", d.alvo, d.acao.rotulo()));
    }
    format!(
        "O que eu decidi sobre o que você propôs:\n\n{}\n\nLeve isso em conta na próxima proposta.",
        linhas.join("\n")
    )
}

/// Lê o registro. Linha ilegível é PULADA: uma decisão corrompida não
/// pode esconder as outras.
pub fn ler(texto: &str) -> Vec<Decisao> {
    texto.lines().filter(|l| !l.trim().is_empty()).filter_map(|l| serde_json::from_str(l).ok()).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn escreve_e_le_a_sequencia_pulando_o_ilegivel() {
        let d = Decisao {
            quando: "2026-09-17 10:00".into(),
            proposta: "p1".into(),
            alvo: "pages/a.md".into(),
            autor: "claude".into(),
            acao: Acao::Parcial { aceitos: 1, de: 3 },
            motivo: String::new(),
        };
        let registro = format!("{}{{quebrado\n{}", linha(&d), linha(&Decisao { acao: Acao::Recusada, motivo: "fora do escopo".into(), ..d.clone() }));
        let lidas = ler(&registro);
        assert_eq!(lidas.len(), 2);
        assert_eq!(lidas[0], d);
        assert_eq!(lidas[1].acao.rotulo(), "recusada");
        assert_eq!(lidas[0].acao.rotulo(), "parcial (1/3)");
    }

    // --- Ciclo 421: contar ao agente -------------------------------------------------

    fn dec(quando: &str, alvo: &str, acao: Acao, motivo: &str) -> Decisao {
        Decisao {
            quando: quando.into(),
            proposta: "p".into(),
            alvo: alvo.into(),
            autor: "claude".into(),
            acao,
            motivo: motivo.into(),
        }
    }

    #[test]
    fn o_resumo_conta_o_que_entrou_e_por_que_o_resto_nao() {
        let lista = vec![
            dec("2026-09-17 09:00", "pages/velha.md", Acao::Aplicada, ""),
            dec("2026-09-17 10:05", "pages/a.md", Acao::Aplicada, ""),
            dec("2026-09-17 10:06", "pages/b.md", Acao::Recusada, "fora do escopo"),
            dec("2026-09-17 10:07", "pages/c.md", Acao::Parcial { aceitos: 1, de: 3 }, ""),
        ];
        let texto = resumo_para_agente(&lista, "2026-09-17 10:00");
        assert!(!texto.contains("velha.md"), "o que é de antes da conversa não entra:\n{texto}");
        assert!(texto.contains("pages/a.md: aplicada"), "{texto}");
        assert!(texto.contains("pages/b.md: recusada — motivo: fora do escopo"), "{texto}");
        assert!(texto.contains("pages/c.md: parcial (1/3)"), "{texto}");
    }

    #[test]
    fn sem_decisao_nova_o_resumo_e_vazio() {
        let lista = vec![dec("2026-09-17 09:00", "pages/a.md", Acao::Aplicada, "")];
        assert!(resumo_para_agente(&lista, "2026-09-17 10:00").is_empty());
        assert!(resumo_para_agente(&[], "").is_empty());
    }
}
