//! As guardas do trabalho autônomo (ciclo 434).
//!
//! O ciclo 412 soltou o agente sozinho — ele dispara por mudança, por
//! consulta e por hora — e o 422 deu o medidor de custo. Faltava o
//! freio: nada impedia um gatilho mal escrito de rodar cinquenta vezes
//! num dia, nem de trabalhar às três da manhã enquanto a conta cresce.
//!
//! Aqui estão os três limites que resolvem isso sem tirar a autonomia:
//! quantas execuções automáticas por dia, quanto custo por dia, e em que
//! janela de horas ele deve ficar quieto.
//!
//! # O que as guardas NÃO fazem
//!
//! Elas não travam o que VOCÊ manda. Enviar uma pergunta é decisão
//! humana com a pessoa na frente da tela; passar do teto ali é escolha
//! dela, não um acidente. As guardas existem pro que roda sem ninguém
//! olhando — que é onde o desastre é silencioso.

use serde::{Deserialize, Serialize};

/// Arquivo das guardas, relativo à raiz do vault.
pub const ARQUIVO: &str = ".anotadinho/guardas.json";

/// Os limites do trabalho automático.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Guardas {
    /// Máximo de disparos automáticos por dia. `0` = sem teto.
    #[serde(default = "execucoes_padrao")]
    pub teto_execucoes: usize,
    /// Máximo de custo por dia, em dólares. `0` = sem teto.
    ///
    /// Só conta o que o agente informou (ciclo 422): execução sem medida
    /// não entra na soma — e por isso o teto de execuções existe também,
    /// como rede pra quem usa agente que não conta.
    #[serde(default = "custo_padrao")]
    pub teto_custo_usd: f64,
    /// Começo da janela de silêncio, `"HH:MM"`. Vazio = sem silêncio.
    #[serde(default = "silencio_de_padrao")]
    pub silencio_de: String,
    /// Fim da janela de silêncio, `"HH:MM"`.
    #[serde(default = "silencio_ate_padrao")]
    pub silencio_ate: String,
}

fn execucoes_padrao() -> usize {
    20
}

fn custo_padrao() -> f64 {
    // Alto o bastante pra não atrapalhar um dia de trabalho de verdade,
    // baixo o bastante pra um gatilho em laço não virar surpresa.
    5.0
}

fn silencio_de_padrao() -> String {
    "23:00".into()
}

fn silencio_ate_padrao() -> String {
    "07:00".into()
}

impl Default for Guardas {
    fn default() -> Self {
        Self {
            teto_execucoes: execucoes_padrao(),
            teto_custo_usd: custo_padrao(),
            silencio_de: silencio_de_padrao(),
            silencio_ate: silencio_ate_padrao(),
        }
    }
}

/// Por que o disparo automático não vai acontecer agora.
///
/// Serializa porque a janela pergunta isto ao backend pra mostrar na
/// tela — e a serialização carrega o limite junto, pra a mensagem poder
/// dizer QUANTO faltava.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Freio {
    /// Pode disparar.
    Nenhum,
    /// Está dentro da janela de silêncio.
    Silencio {
        /// Até quando (`"HH:MM"`).
        ate: String,
    },
    /// Já rodou o bastante hoje.
    Execucoes {
        hoje: usize,
        teto: usize,
    },
    /// Já gastou o bastante hoje.
    Custo {
        hoje: f64,
        teto: f64,
    },
}

impl Freio {
    pub fn parou(&self) -> bool {
        !matches!(self, Self::Nenhum)
    }

    /// Como se lê na tela — dizendo o limite, não só que parou.
    pub fn motivo(&self) -> String {
        match self {
            Self::Nenhum => String::new(),
            Self::Silencio { ate } => format!("janela de silêncio até {ate}"),
            Self::Execucoes { hoje, teto } => {
                format!("já foram {hoje} disparos automáticos hoje (teto {teto})")
            }
            Self::Custo { hoje, teto } => {
                format!("o dia já custou US$ {hoje:.2} (teto US$ {teto:.2})")
            }
        }
    }
}

/// Alguma guarda impede o disparo automático agora?
///
/// - `agora`: `"AAAA-MM-DD HH:MM"`.
/// - `execucoes_hoje`: quantas já rodaram hoje (automáticas e manuais —
///   o que pesa na conta é o total do dia).
/// - `custo_hoje`: o que já foi medido hoje, em dólares.
pub fn freio(g: &Guardas, agora: &str, execucoes_hoje: usize, custo_hoje: f64) -> Freio {
    let hora = agora.split_whitespace().nth(1).unwrap_or("");
    if em_silencio(g, hora) {
        return Freio::Silencio { ate: g.silencio_ate.clone() };
    }
    if g.teto_execucoes > 0 && execucoes_hoje >= g.teto_execucoes {
        return Freio::Execucoes { hoje: execucoes_hoje, teto: g.teto_execucoes };
    }
    if g.teto_custo_usd > 0.0 && custo_hoje >= g.teto_custo_usd {
        return Freio::Custo { hoje: custo_hoje, teto: g.teto_custo_usd };
    }
    Freio::Nenhum
}

/// A hora está dentro da janela de silêncio?
///
/// A janela ATRAVESSA a meia-noite quando o começo é maior que o fim
/// (`23:00`–`07:00`) — que é justamente o caso comum, e o que uma
/// comparação ingênua erraria.
fn em_silencio(g: &Guardas, hora: &str) -> bool {
    let (de, ate) = (g.silencio_de.trim(), g.silencio_ate.trim());
    if de.is_empty() || ate.is_empty() || hora.is_empty() || de == ate {
        return false;
    }
    if de < ate {
        hora >= de && hora < ate
    } else {
        hora >= de || hora < ate
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn sem_nada_no_caminho_o_gatilho_dispara() {
        let g = Guardas::default();
        assert_eq!(freio(&g, "2026-09-19 10:00", 3, 0.40), Freio::Nenhum);
        assert!(!freio(&g, "2026-09-19 10:00", 3, 0.40).parou());
    }

    #[test]
    fn a_janela_de_silencio_atravessa_a_meia_noite() {
        let g = Guardas::default(); // 23:00 → 07:00
        assert!(freio(&g, "2026-09-19 23:30", 0, 0.0).parou(), "depois das 23h");
        assert!(freio(&g, "2026-09-19 03:00", 0, 0.0).parou(), "madrugada");
        assert!(!freio(&g, "2026-09-19 07:00", 0, 0.0).parou(), "às 7 já pode");
        assert!(!freio(&g, "2026-09-19 22:59", 0, 0.0).parou());
        // Janela que não atravessa (almoço, digamos).
        let almoco = Guardas { silencio_de: "12:00".into(), silencio_ate: "13:00".into(), ..Guardas::default() };
        assert!(freio(&almoco, "2026-09-19 12:30", 0, 0.0).parou());
        assert!(!freio(&almoco, "2026-09-19 14:00", 0, 0.0).parou());
        // Vazio desliga o silêncio.
        let sem = Guardas { silencio_de: String::new(), silencio_ate: String::new(), ..Guardas::default() };
        assert!(!freio(&sem, "2026-09-19 03:00", 0, 0.0).parou());
    }

    #[test]
    fn o_teto_de_execucoes_segura_o_gatilho_em_laco() {
        let g = Guardas { teto_execucoes: 5, ..Guardas::default() };
        assert!(!freio(&g, "2026-09-19 10:00", 4, 0.0).parou());
        let f = freio(&g, "2026-09-19 10:00", 5, 0.0);
        assert!(f.parou());
        assert!(f.motivo().contains("5 disparos") && f.motivo().contains("teto 5"), "{}", f.motivo());
        // Zero desliga.
        let sem = Guardas { teto_execucoes: 0, ..Guardas::default() };
        assert!(!freio(&sem, "2026-09-19 10:00", 900, 0.0).parou());
    }

    #[test]
    fn o_teto_de_custo_para_antes_da_conta_crescer() {
        let g = Guardas { teto_custo_usd: 2.0, ..Guardas::default() };
        assert!(!freio(&g, "2026-09-19 10:00", 0, 1.99).parou());
        let f = freio(&g, "2026-09-19 10:00", 0, 2.0);
        assert!(f.parou());
        assert!(f.motivo().contains("US$ 2.00"), "{}", f.motivo());
        let sem = Guardas { teto_custo_usd: 0.0, ..Guardas::default() };
        assert!(!freio(&sem, "2026-09-19 10:00", 0, 999.0).parou());
    }

    #[test]
    fn o_silencio_vem_antes_dos_tetos_na_explicacao() {
        // Três razões ao mesmo tempo: a que se conta é a que a pessoa
        // resolve primeiro — esperar amanhecer.
        let g = Guardas { teto_execucoes: 1, teto_custo_usd: 0.01, ..Guardas::default() };
        let f = freio(&g, "2026-09-19 02:00", 50, 99.0);
        assert!(matches!(f, Freio::Silencio { .. }), "{f:?}");
    }

    #[test]
    fn guardas_antigas_ou_parciais_leem_com_padrao() {
        let g: Guardas = serde_json::from_str("{}").expect("arquivo vazio vale");
        assert_eq!(g, Guardas::default());
        let so_custo: Guardas = serde_json::from_str(r#"{"teto_custo_usd": 1.5}"#).unwrap();
        assert_eq!(so_custo.teto_custo_usd, 1.5);
        assert_eq!(so_custo.teto_execucoes, execucoes_padrao());
    }
}
