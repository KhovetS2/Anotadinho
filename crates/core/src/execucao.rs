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
// `Eq` sai com o custo (ciclo 422): float não tem igualdade total.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    /// O que consumiu, quando o agente conta (ciclo 422).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uso: Option<crate::agente::Uso>,
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

/// O que o dia consumiu (ciclo 422): soma o uso das execuções daquele
/// dia, e diz quantas não contaram.
///
/// Execução sem uso não vira zero na conta — vira "de N". Um total que
/// esconde o que não foi medido convida a confiar demais nele.
pub fn total_do_dia(execucoes: &[Execucao], dia: &str) -> Option<(crate::agente::Uso, usize, usize)> {
    let do_dia: Vec<&Execucao> = execucoes.iter().filter(|x| x.quando.starts_with(dia)).collect();
    if do_dia.is_empty() {
        return None;
    }
    let com_uso: Vec<&crate::agente::Uso> = do_dia.iter().filter_map(|x| x.uso.as_ref()).collect();
    if com_uso.is_empty() {
        return None;
    }
    let total = com_uso.iter().fold(crate::agente::Uso::default(), |acc, u| acc.somar(u));
    Some((total, com_uso.len(), do_dia.len()))
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
            uso: None,
        };
        let lidas = ler(&format!("{}\nlixo\n", linha(&x).trim_end()));
        assert_eq!(lidas, vec![x]);
        assert_eq!(lidas[0].fim.rotulo(), "falhou: timeout");
        assert_eq!(Fim::Interrompida.rotulo(), "interrompida");
    }

    // --- Ciclo 422: o que o dia consumiu ---------------------------------------------

    fn com_uso(quando: &str, entrada: u64, custo: Option<f64>) -> Execucao {
        Execucao {
            quando: quando.into(),
            conversa: "pages/conversas/c.md".into(),
            agente: "claude".into(),
            binario: "claude".into(),
            anexos: 0,
            prompt: 10,
            segundos: 5,
            fim: Fim::Respondeu,
            uso: Some(crate::agente::Uso { entrada, saida: 10, custo_usd: custo }),
        }
    }

    #[test]
    fn o_total_do_dia_soma_e_diz_quantas_contaram() {
        let mut sem = com_uso("2026-09-17 11:00", 0, None);
        sem.uso = None;
        let lista = vec![
            com_uso("2026-09-17 10:00", 1000, Some(0.01)),
            com_uso("2026-09-17 10:30", 2000, Some(0.02)),
            sem,
            com_uso("2026-09-16 10:00", 9999, Some(9.99)),
        ];
        let (total, contaram, quantas) = total_do_dia(&lista, "2026-09-17").expect("tem o que somar");
        assert_eq!(total.entrada, 3000, "só as do dia");
        assert_eq!(total.custo_usd, Some(0.03));
        assert_eq!((contaram, quantas), (2, 3), "duas contaram, de três execuções");
        // Dia sem execução, ou dia em que ninguém contou: nada a mostrar.
        assert!(total_do_dia(&lista, "2026-09-15").is_none());
    }

    #[test]
    fn execucao_antiga_sem_uso_continua_lendo() {
        let antiga = r#"{"quando":"2026-09-17 10:00","conversa":"c.md","agente":"a","binario":"b","anexos":0,"prompt":1,"segundos":2,"fim":"respondeu"}"#;
        let x: Execucao = serde_json::from_str(antiga).expect("json de antes do 422");
        assert_eq!(x.uso, None);
        assert!(!linha(&x).contains("uso"), "quem não mediu não ganha campo");
    }
}
