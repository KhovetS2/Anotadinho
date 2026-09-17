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
    /// Nada entrou.
    Recusada,
}

impl Acao {
    /// Como se lê na tela.
    pub fn rotulo(&self) -> String {
        match self {
            Self::Aplicada => "aplicada".into(),
            Self::Parcial { aceitos, de } => format!("parcial ({aceitos}/{de})"),
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
}
