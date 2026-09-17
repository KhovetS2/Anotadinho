//! Gatilhos: quando o agente trabalha sem ninguém pedir (ciclo 412).
//!
//! Até aqui o agente só roda quando alguém escreve na conversa. Isso
//! basta pra conversar e é ruim pra trabalho de fundo: "toda vez que eu
//! mexer nas specs, revise o que ficou inconsistente" é um pedido que
//! ninguém quer repetir vinte vezes por semana.
//!
//! Um gatilho é uma regra guardada no vault: QUANDO acontece tal coisa,
//! mande tal prompt pro agente. O disparo continua passando por tudo o
//! que já protege o vault — o agente propõe, a pessoa aprova (ciclo
//! 204), dentro das permissões por pasta (ciclo 405), respeitando o
//! limite de paralelismo (ciclo 408) e deixando rastro no registro de
//! execuções (ciclo 406).
//!
//! # Por que a decisão mora aqui
//!
//! "Está na hora?" é lógica pura e cheia de canto: relógio, o que já
//! rodou, o que mudou no disco. Aqui dá pra testar sem tocar em arquivo
//! nem esperar um dia passar. Quem observa o disco e quem dispara
//! processo ficam fora.

use serde::{Deserialize, Serialize};

/// Arquivo dos gatilhos, relativo à raiz do vault.
pub const ARQUIVO: &str = ".anotadinho/gatilhos.json";

/// O que faz o gatilho disparar.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "lowercase")]
pub enum Quando {
    /// Alguma página sob este prefixo mudou no disco.
    Mudou {
        /// Pasta ou página (`pages/specs`, `pages/specs/a.md`).
        prefixo: String,
    },
    /// Uma consulta do vault voltou com resultado — quem avalia é quem
    /// tem o índice; aqui só se guarda a pergunta.
    Consulta {
        /// De onde (como no embed de consulta).
        de: String,
        /// Os filtros, no formato `campo=valor`.
        #[serde(default)]
        onde: Vec<String>,
    },
    /// Todo dia, a partir desta hora (`"07:30"`).
    Diario {
        /// `"HH:MM"`.
        hora: String,
    },
}

impl Quando {
    /// Como se lê na tela.
    pub fn rotulo(&self) -> String {
        match self {
            Self::Mudou { prefixo } => format!("quando mudar {prefixo}"),
            Self::Consulta { de, onde } if onde.is_empty() => format!("quando houver algo em {de}"),
            Self::Consulta { de, onde } => format!("quando {de} tiver {}", onde.join(", ")),
            Self::Diario { hora } => format!("todo dia às {hora}"),
        }
    }
}

/// Uma regra de disparo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Gatilho {
    /// Nome curto — identifica o gatilho e nomeia a conversa que ele
    /// abre.
    pub nome: String,
    pub quando: Quando,
    /// O prompt mandado pro agente.
    pub prompt: String,
    /// Desligado não dispara. Nasce ligado.
    #[serde(default = "sim")]
    pub ativo: bool,
    /// Quando disparou por último, `"AAAA-MM-DD HH:MM"`. Vazio = nunca.
    #[serde(default)]
    pub ultimo: String,
}

fn sim() -> bool {
    true
}

/// Minutos de descanso entre dois disparos do mesmo gatilho de mudança.
///
/// Salvar uma página dispara o watcher várias vezes, e um agente por
/// tecla digitada seria um desastre de custo. Um minuto é curto pro
/// trabalho de fundo e longo pra rajada de salvamento.
pub const DESCANSO_MIN: i64 = 1;

/// Quais gatilhos devem disparar agora.
///
/// - `agora`: `"AAAA-MM-DD HH:MM"`.
/// - `mudaram`: páginas que acabaram de mudar no disco.
/// - `com_resultado`: nomes dos gatilhos de consulta cuja pergunta tem
///   resposta agora (quem avalia é quem tem o índice).
///
/// Devolve os índices, na ordem da lista.
pub fn devidos(
    gatilhos: &[Gatilho],
    agora: &str,
    mudaram: &[String],
    com_resultado: &[String],
) -> Vec<usize> {
    gatilhos
        .iter()
        .enumerate()
        .filter(|(_, g)| g.ativo)
        .filter(|(_, g)| match &g.quando {
            Quando::Mudou { prefixo } => {
                descansou(&g.ultimo, agora) && mudaram.iter().any(|p| sob(p, prefixo))
            }
            Quando::Consulta { .. } => {
                descansou(&g.ultimo, agora) && com_resultado.iter().any(|n| n == &g.nome)
            }
            // Um por dia, e só depois da hora marcada.
            Quando::Diario { hora } => dia(&g.ultimo) != dia(agora) && hora_de(agora) >= hora.as_str(),
        })
        .map(|(i, _)| i)
        .collect()
}

/// A página está sob o prefixo? Compara por limite de pasta: `pages/spec`
/// não pega `pages/specs/a.md`.
fn sob(path: &str, prefixo: &str) -> bool {
    let prefixo = prefixo.trim_end_matches('/');
    if prefixo.is_empty() {
        return true;
    }
    path == prefixo || path.strip_prefix(prefixo).is_some_and(|resto| resto.starts_with('/'))
}

fn dia(quando: &str) -> &str {
    quando.split_whitespace().next().unwrap_or("")
}

fn hora_de(quando: &str) -> &str {
    quando.split_whitespace().nth(1).unwrap_or("")
}

/// Passou o descanso desde o último disparo? Comparação por minutos do
/// mesmo dia; dia diferente já passou.
fn descansou(ultimo: &str, agora: &str) -> bool {
    if ultimo.trim().is_empty() {
        return true;
    }
    if dia(ultimo) != dia(agora) {
        return true;
    }
    match (minutos(hora_de(ultimo)), minutos(hora_de(agora))) {
        (Some(a), Some(b)) => b - a >= DESCANSO_MIN,
        // Relógio ilegível não pode travar o gatilho pra sempre.
        _ => true,
    }
}

fn minutos(hora: &str) -> Option<i64> {
    let (h, m) = hora.split_once(':')?;
    Some(h.trim().parse::<i64>().ok()? * 60 + m.trim().parse::<i64>().ok()?)
}

/// Marca o disparo. É o que impede repetição na rajada seguinte.
pub fn marcar(g: &mut Gatilho, agora: &str) {
    g.ultimo = agora.to_string();
}

#[cfg(test)]
mod testes {
    use super::*;

    fn mudou(nome: &str, prefixo: &str) -> Gatilho {
        Gatilho {
            nome: nome.into(),
            quando: Quando::Mudou { prefixo: prefixo.into() },
            prompt: "revise".into(),
            ativo: true,
            ultimo: String::new(),
        }
    }

    #[test]
    fn mudanca_sob_o_prefixo_dispara_e_fora_dele_nao() {
        let gs = vec![mudou("specs", "pages/specs")];
        assert_eq!(devidos(&gs, "2026-09-17 10:00", &["pages/specs/a.md".into()], &[]), [0]);
        assert!(devidos(&gs, "2026-09-17 10:00", &["pages/outra.md".into()], &[]).is_empty());
        // Limite de pasta: `pages/spec` não é `pages/specs`.
        assert!(devidos(&gs, "2026-09-17 10:00", &["pages/specsinho/a.md".into()], &[]).is_empty());
        // A própria página vale como prefixo.
        let gs = vec![mudou("uma", "pages/uma.md")];
        assert_eq!(devidos(&gs, "2026-09-17 10:00", &["pages/uma.md".into()], &[]), [0]);
    }

    #[test]
    fn o_descanso_evita_a_rajada_de_salvamento() {
        let mut g = mudou("specs", "pages/specs");
        marcar(&mut g, "2026-09-17 10:00");
        let gs = vec![g];
        let mudanca = vec!["pages/specs/a.md".to_string()];
        assert!(devidos(&gs, "2026-09-17 10:00", &mudanca, &[]).is_empty(), "no mesmo minuto, não");
        assert_eq!(devidos(&gs, "2026-09-17 10:01", &mudanca, &[]), [0]);
        // Dia seguinte sempre passou.
        assert_eq!(devidos(&gs, "2026-09-18 00:00", &mudanca, &[]), [0]);
    }

    #[test]
    fn desligado_nunca_dispara() {
        let mut g = mudou("specs", "pages/specs");
        g.ativo = false;
        assert!(devidos(&[g], "2026-09-17 10:00", &["pages/specs/a.md".into()], &[]).is_empty());
    }

    #[test]
    fn o_diario_dispara_uma_vez_por_dia_depois_da_hora() {
        let g = Gatilho {
            nome: "resumo".into(),
            quando: Quando::Diario { hora: "07:30".into() },
            prompt: "resuma o dia".into(),
            ativo: true,
            ultimo: String::new(),
        };
        let gs = vec![g.clone()];
        assert!(devidos(&gs, "2026-09-17 07:00", &[], &[]).is_empty(), "antes da hora, não");
        assert_eq!(devidos(&gs, "2026-09-17 07:30", &[], &[]), [0]);
        let mut ja = g;
        marcar(&mut ja, "2026-09-17 07:31");
        let gs = vec![ja];
        assert!(devidos(&gs, "2026-09-17 23:00", &[], &[]).is_empty(), "já rodou hoje");
        assert_eq!(devidos(&gs, "2026-09-18 08:00", &[], &[]), [0]);
    }

    #[test]
    fn a_consulta_dispara_quando_quem_tem_o_indice_diz_que_ha_resultado() {
        let gs = vec![Gatilho {
            nome: "pendentes".into(),
            quando: Quando::Consulta { de: "pages/specs".into(), onde: vec!["status=rascunho".into()] },
            prompt: "feche as pendências".into(),
            ativo: true,
            ultimo: String::new(),
        }];
        assert!(devidos(&gs, "2026-09-17 10:00", &[], &[]).is_empty());
        assert_eq!(devidos(&gs, "2026-09-17 10:00", &[], &["pendentes".to_string()]), [0]);
        // Nome de outro gatilho não serve.
        assert!(devidos(&gs, "2026-09-17 10:00", &[], &["outro".to_string()]).is_empty());
    }

    #[test]
    fn o_rotulo_explica_a_regra() {
        assert_eq!(mudou("x", "pages/specs").quando.rotulo(), "quando mudar pages/specs");
        assert_eq!(Quando::Diario { hora: "07:30".into() }.rotulo(), "todo dia às 07:30");
        assert_eq!(
            Quando::Consulta { de: "pages/specs".into(), onde: vec!["status=rascunho".into()] }.rotulo(),
            "quando pages/specs tiver status=rascunho"
        );
        assert_eq!(
            Quando::Consulta { de: "pages/specs".into(), onde: vec![] }.rotulo(),
            "quando houver algo em pages/specs"
        );
    }
}
