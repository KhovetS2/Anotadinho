//! Avaliação do agente: tarefas com resultado esperado (ciclo 413).
//!
//! Todo o resto do projeto testa o Anotadinho. Nada testava o ARRANJO —
//! prompt, contrato de ferramentas, permissões, formato de proposta —
//! que faz um agente de verdade acertar ou errar. Trocar o prompt padrão
//! ou mexer no contrato (ciclo 407) era mudar às cegas: dava pra "ficar
//! melhor" e ninguém saber.
//!
//! Uma tarefa descreve um vault de partida, o que se pede, e o que se
//! espera ver depois. Rodar a suíte com o agente falso verifica o
//! ARRANJO (o prompt chega, a proposta aparece, a permissão barra);
//! rodar com um agente real mede o agente.
//!
//! # O que este módulo faz e o que não faz
//!
//! Aqui está a DEFINIÇÃO e o JULGAMENTO — puros, testáveis sem processo
//! filho. Criar o vault temporário e chamar o binário é do CLI (`avaliar`).
//! Assim o julgamento não depende de máquina, e a mesma tarefa pode ser
//! julgada de novo sobre um resultado guardado.

use serde::{Deserialize, Serialize};

/// O que se espera depois da execução.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "tipo", rename_all = "snake_case")]
pub enum Espera {
    /// Existe proposta pra alguma página sob este prefixo.
    PropoeEm {
        prefixo: String,
    },
    /// NENHUMA proposta foi criada — é o esperado quando a tarefa pede
    /// algo fora das permissões, ou só uma leitura.
    NaoPropoe,
    /// O conteúdo de alguma proposta contém este texto.
    ConteudoContem {
        texto: String,
    },
    /// A resposta do agente (o que ele escreveu na saída) contém este
    /// texto.
    RespostaContem {
        texto: String,
    },
    /// A execução falhou — e a mensagem contém este texto. Serve pra
    /// fixar o comportamento de recusa (permissão, timeout).
    FalhaContem {
        texto: String,
    },
}

impl Espera {
    /// Como se lê no relatório.
    pub fn rotulo(&self) -> String {
        match self {
            Self::PropoeEm { prefixo } => format!("propõe em {prefixo}"),
            Self::NaoPropoe => "não propõe nada".into(),
            Self::ConteudoContem { texto } => format!("conteúdo contém \"{texto}\""),
            Self::RespostaContem { texto } => format!("resposta contém \"{texto}\""),
            Self::FalhaContem { texto } => format!("falha contém \"{texto}\""),
        }
    }
}

/// Uma tarefa de avaliação.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tarefa {
    /// Nome curto, que aparece no relatório.
    pub nome: String,
    /// O que se manda pro agente.
    pub prompt: String,
    /// O vault de partida: caminho relativo → conteúdo.
    #[serde(default)]
    pub vault: Vec<(String, String)>,
    /// As permissões de escrita do vault de partida (ciclo 405).
    /// Ausente usa o padrão.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub permissoes: Option<crate::permissoes::Permissoes>,
    /// Tudo isto tem que valer pra tarefa passar.
    pub espera: Vec<Espera>,
}

/// O que se observou depois de rodar.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Observado {
    /// As propostas que sobraram no vault: (alvo, conteúdo).
    pub propostas: Vec<(String, String)>,
    /// O que o agente escreveu na saída.
    pub resposta: String,
    /// A mensagem de falha, se falhou.
    pub falha: Option<String>,
    /// Quanto tempo levou.
    pub segundos: u64,
}

/// O veredito de uma tarefa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Veredito {
    pub tarefa: String,
    pub passou: bool,
    /// Uma linha por expectativa que falhou. Vazio quando passou.
    pub falhas: Vec<String>,
    pub segundos: u64,
}

/// Julga uma tarefa sobre o que se observou.
pub fn julgar(t: &Tarefa, o: &Observado) -> Veredito {
    let mut falhas = Vec::new();
    for espera in &t.espera {
        let ok = match espera {
            Espera::PropoeEm { prefixo } => o.propostas.iter().any(|(alvo, _)| sob(alvo, prefixo)),
            Espera::NaoPropoe => o.propostas.is_empty(),
            Espera::ConteudoContem { texto } => {
                o.propostas.iter().any(|(_, conteudo)| conteudo.contains(texto))
            }
            Espera::RespostaContem { texto } => o.resposta.contains(texto),
            Espera::FalhaContem { texto } => {
                o.falha.as_ref().is_some_and(|f| f.contains(texto))
            }
        };
        if !ok {
            falhas.push(format!("esperava que {}", espera.rotulo()));
        }
    }
    // Falha não prevista é falha da tarefa: um agente que morreu não
    // "passou" só porque a expectativa era não propor nada.
    if let Some(f) = &o.falha {
        let previa = t.espera.iter().any(|e| matches!(e, Espera::FalhaContem { .. }));
        if !previa {
            falhas.push(format!("o agente falhou: {f}"));
        }
    }
    Veredito { tarefa: t.nome.clone(), passou: falhas.is_empty(), falhas, segundos: o.segundos }
}

/// Mesma regra de limite de pasta das permissões (ciclo 405).
fn sob(alvo: &str, prefixo: &str) -> bool {
    let prefixo = prefixo.trim_end_matches('/');
    if prefixo.is_empty() {
        return true;
    }
    alvo == prefixo || alvo.strip_prefix(prefixo).is_some_and(|resto| resto.starts_with('/'))
}

/// O resumo da suíte, pra uma linha final.
pub fn resumo(vereditos: &[Veredito]) -> String {
    let passaram = vereditos.iter().filter(|v| v.passou).count();
    let segundos: u64 = vereditos.iter().map(|v| v.segundos).sum();
    format!("{passaram}/{} tarefas passaram em {segundos}s", vereditos.len())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tarefa(espera: Vec<Espera>) -> Tarefa {
        Tarefa {
            nome: "uma".into(),
            prompt: "faça".into(),
            vault: vec![("pages/a.md".into(), "# A\n".into())],
            permissoes: None,
            espera,
        }
    }

    #[test]
    fn passa_quando_todas_as_expectativas_valem() {
        let t = tarefa(vec![
            Espera::PropoeEm { prefixo: "pages/specs".into() },
            Espera::ConteudoContem { texto: "resumo".into() },
            Espera::RespostaContem { texto: "pronto".into() },
        ]);
        let o = Observado {
            propostas: vec![("pages/specs/nova.md".into(), "# Nova\n\nresumo do dia\n".into())],
            resposta: "pronto, propus a página".into(),
            falha: None,
            segundos: 3,
        };
        let v = julgar(&t, &o);
        assert!(v.passou, "{:?}", v.falhas);
        assert_eq!(v.segundos, 3);
    }

    #[test]
    fn cada_expectativa_que_falha_aparece_no_relatorio() {
        let t = tarefa(vec![
            Espera::PropoeEm { prefixo: "pages/specs".into() },
            Espera::RespostaContem { texto: "pronto".into() },
        ]);
        let v = julgar(&t, &Observado::default());
        assert!(!v.passou);
        assert_eq!(v.falhas.len(), 2);
        assert!(v.falhas[0].contains("propõe em pages/specs"), "{:?}", v.falhas);
    }

    #[test]
    fn nao_propor_e_uma_expectativa_de_verdade() {
        let t = tarefa(vec![Espera::NaoPropoe]);
        assert!(julgar(&t, &Observado::default()).passou);
        let o = Observado {
            propostas: vec![("journals/2026-09-17.md".into(), "x".into())],
            ..Default::default()
        };
        assert!(!julgar(&t, &o).passou, "propôs onde não devia");
    }

    #[test]
    fn falha_nao_prevista_reprova_mesmo_com_expectativa_satisfeita() {
        let t = tarefa(vec![Espera::NaoPropoe]);
        let o = Observado { falha: Some("o agente falhou: saiu 1".into()), ..Default::default() };
        let v = julgar(&t, &o);
        assert!(!v.passou);
        assert!(v.falhas[0].contains("saiu 1"), "{:?}", v.falhas);
        // Prevista, ela passa a ser o que se espera.
        let t = tarefa(vec![Espera::FalhaContem { texto: "saiu 1".into() }]);
        assert!(julgar(&t, &o).passou);
    }

    #[test]
    fn o_prefixo_respeita_limite_de_pasta() {
        let t = tarefa(vec![Espera::PropoeEm { prefixo: "pages/spec".into() }]);
        let o = Observado {
            propostas: vec![("pages/specs/a.md".into(), String::new())],
            ..Default::default()
        };
        assert!(!julgar(&t, &o).passou, "pages/spec não é pages/specs");
    }

    #[test]
    fn o_resumo_conta_o_que_passou() {
        let v = vec![
            Veredito { tarefa: "a".into(), passou: true, falhas: vec![], segundos: 2 },
            Veredito { tarefa: "b".into(), passou: false, falhas: vec!["x".into()], segundos: 3 },
        ];
        assert_eq!(resumo(&v), "1/2 tarefas passaram em 5s");
    }

    #[test]
    fn a_tarefa_vai_e_volta_em_json() {
        let t = tarefa(vec![Espera::PropoeEm { prefixo: "pages/specs".into() }, Espera::NaoPropoe]);
        let json = serde_json::to_string(&t).unwrap();
        assert_eq!(serde_json::from_str::<Tarefa>(&json).unwrap(), t);
        // A forma no arquivo é a que a suíte lê à mão.
        assert!(json.contains("\"tipo\":\"propoe_em\""), "{json}");
    }
}
