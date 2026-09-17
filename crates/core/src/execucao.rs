//! O registro das execuções do agente (ciclo 406).
//!
//! A conversa guarda o que foi dito; a proposta, o que ele quis
//! escrever; o registro de decisões, o que você aceitou. Faltava o
//! MEIO: quando o agente rodou, com que configuração, quanto tempo
//! levou e como terminou. Sem isso não se responde "o que ele fez
//! ontem" nem "por que aquela resposta demorou".
//!
//! Mesma forma do registro de decisões: uma linha JSON por execução em
//! `.anotadinho/`, fora de `pages/`.

use serde::{Deserialize, Serialize};

/// Arquivo do registro, relativo à raiz do vault.
pub const ARQUIVO: &str = ".anotadinho/execucoes.jsonl";

/// Como a execução terminou.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Fim {
    /// O agente respondeu.
    Respondeu,
    /// Falhou, com a mensagem.
    Falhou(String),
    /// A pessoa interrompeu.
    Interrompida,
}

impl Fim {
    /// Como se lê na tela.
    pub fn rotulo(&self) -> String {
        match self {
            Self::Respondeu => "respondeu".into(),
            Self::Falhou(e) => format!("falhou: {}", e.lines().next().unwrap_or("")),
            Self::Interrompida => "interrompida".into(),
        }
    }
}

/// Uma execução do agente, do começo ao fim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Execucao {
    /// Quando começou, `"AAAA-MM-DD HH:MM"`.
    pub quando: String,
    /// A conversa de onde saiu.
    pub conversa: String,
    /// O nome do agente configurado.
    pub agente: String,
    /// O executável, pra saber depois o que rodou de fato.
    pub binario: String,
    /// Quantas páginas foram anexadas como contexto.
    pub anexos: usize,
    /// Tamanho do prompt em caracteres — o que foi mandado, sem guardar
    /// o conteúdo (ele já está na conversa).
    pub prompt: usize,
    /// Quanto durou.
    pub segundos: u64,
    /// Como terminou.
    pub fim: Fim,
}

/// A linha JSON de uma execução, pronta pra acrescentar (já com `\n`).
pub fn linha(x: &Execucao) -> String {
    match serde_json::to_string(x) {
        Ok(j) => format!("{j}\n"),
        Err(_) => String::new(),
    }
}

/// Lê o registro; linha ilegível é pulada.
pub fn ler(texto: &str) -> Vec<Execucao> {
    texto.lines().filter(|l| !l.trim().is_empty()).filter_map(|l| serde_json::from_str(l).ok()).collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn escreve_e_le_a_execucao() {
        let x = Execucao {
            quando: "2026-09-17 12:00".into(),
            conversa: "pages/conversas/c1.md".into(),
            agente: "claude".into(),
            binario: "claude".into(),
            anexos: 2,
            prompt: 1234,
            segundos: 17,
            fim: Fim::Falhou("timeout\ndetalhe".into()),
        };
        let lidas = ler(&format!("{}\nlixo\n", linha(&x).trim_end()));
        assert_eq!(lidas, vec![x]);
        assert_eq!(lidas[0].fim.rotulo(), "falhou: timeout");
        assert_eq!(Fim::Interrompida.rotulo(), "interrompida");
    }
}
