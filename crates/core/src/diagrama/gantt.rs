//! Gantt: seções e tarefas com início, fim ou duração e dependência.

use super::{desescapar, id_valido, sem_posicoes, ForaDoModelo, Instrucao};
use crate::detalhes::data_valida;

/// O único formato de data do construtor: é o que o seletor de data da
/// tela produz, e o que o resto do vault usa.
pub const FORMATO_DATA: &str = "YYYY-MM-DD";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marca {
    Feita,
    Ativa,
    Critica,
    Marco,
}

impl Marca {
    pub const TODAS: [Marca; 4] = [Marca::Feita, Marca::Ativa, Marca::Critica, Marca::Marco];

    pub fn token(self) -> &'static str {
        match self {
            Marca::Feita => "done",
            Marca::Ativa => "active",
            Marca::Critica => "crit",
            Marca::Marco => "milestone",
        }
    }

    fn de(s: &str) -> Option<Self> {
        Self::TODAS.into_iter().find(|m| m.token() == s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Inicio {
    /// Sem início escrito: logo depois da tarefa anterior.
    Anterior,
    Data(String),
    /// `after a1 a2`: depois que todas terminarem.
    Depois(Vec<String>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Fim {
    Data(String),
    /// `3d`, `2w`, `12h`.
    Duracao(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tarefa {
    pub nome: String,
    pub marcas: Vec<Marca>,
    pub id: Option<String>,
    pub inicio: Inicio,
    pub fim: Fim,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Secao {
    /// `None` só na primeira: tarefas antes de qualquer `section`.
    pub nome: Option<String>,
    pub tarefas: Vec<Tarefa>,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Gantt {
    pub titulo: Option<String>,
    /// `true` quando o código traz `dateFormat YYYY-MM-DD` (o padrão do
    /// mermaid já é esse; guardar só evita diff ao reaplicar).
    pub com_formato: bool,
    pub secoes: Vec<Secao>,
}

/// `3d`, `1.5w`, `12h`: número e unidade.
pub fn duracao_valida(s: &str) -> bool {
    let Some(unidade) = s.chars().last() else {
        return false;
    };
    let numero = &s[..s.len() - unidade.len_utf8()];
    matches!(unidade, 'd' | 'w' | 'h') && !numero.is_empty() && numero.parse::<f64>().map(|n| n > 0.0).unwrap_or(false)
}

/// O nome de tarefa no código: `:` encerraria o nome, `;` a instrução.
fn escapar_nome(t: &str) -> String {
    // `;` primeiro: as entidades também terminam em `;`.
    t.replace(';', "#59;").replace(':', "#58;").replace('"', "#quot;")
}

impl Gantt {
    pub fn escrever(&self) -> String {
        let mut s = String::from("gantt\n");
        if let Some(t) = &self.titulo {
            s.push_str(&format!("  title {}\n", escapar_nome(t)));
        }
        if self.com_formato {
            s.push_str(&format!("  dateFormat {FORMATO_DATA}\n"));
        }
        for sec in &self.secoes {
            let recuo = if let Some(n) = &sec.nome {
                s.push_str(&format!("  section {}\n", escapar_nome(n)));
                "    "
            } else {
                "  "
            };
            for t in &sec.tarefas {
                let mut campos: Vec<String> = t.marcas.iter().map(|m| m.token().to_string()).collect();
                if let Some(id) = &t.id {
                    campos.push(id.clone());
                }
                match &t.inicio {
                    Inicio::Anterior => {}
                    Inicio::Data(d) => campos.push(d.clone()),
                    Inicio::Depois(ids) => campos.push(format!("after {}", ids.join(" "))),
                }
                campos.push(match &t.fim {
                    Fim::Data(d) | Fim::Duracao(d) => d.clone(),
                });
                s.push_str(&format!("{recuo}{} :{}\n", escapar_nome(&t.nome), campos.join(", ")));
            }
        }
        s
    }
}

pub(crate) fn ler(linha: usize, cabecalho: &str, resto: &[Instrucao]) -> Result<Gantt, ForaDoModelo> {
    if cabecalho.trim() != "gantt" {
        return Err(ForaDoModelo::na(linha, "cabeçalho com mais coisa além de `gantt`"));
    }
    sem_posicoes(resto)?;
    let mut g = Gantt::default();
    for inst in resto {
        let Instrucao::Texto { linha, texto } = inst else {
            continue;
        };
        let linha = *linha;
        let (palavra, depois) = texto.split_once(char::is_whitespace).unwrap_or((texto.as_str(), ""));
        match palavra {
            "title" => g.titulo = Some(desescapar(depois.trim())),
            "dateFormat" => {
                if depois.trim() != FORMATO_DATA {
                    return Err(ForaDoModelo::na(linha, format!("`dateFormat {}` — o construtor usa {FORMATO_DATA}", depois.trim())));
                }
                g.com_formato = true;
            }
            "section" => g.secoes.push(Secao { nome: Some(desescapar(depois.trim())), tarefas: Vec::new() }),
            "axisFormat" | "tickInterval" | "excludes" | "includes" | "todayMarker" | "weekday" | "click" | "inclusiveEndDates"
            | "topAxis" | "displayMode" | "accTitle" | "accDescr" => {
                return Err(ForaDoModelo::na(linha, format!("`{palavra}` não é montado pelo construtor")));
            }
            _ => {
                let tarefa = ler_tarefa(linha, texto)?;
                if g.secoes.is_empty() {
                    g.secoes.push(Secao { nome: None, tarefas: Vec::new() });
                }
                g.secoes.last_mut().expect("acabou de entrar").tarefas.push(tarefa);
            }
        }
    }
    Ok(g)
}

fn ler_tarefa(linha: usize, texto: &str) -> Result<Tarefa, ForaDoModelo> {
    let (nome, dados) = texto
        .split_once(':')
        .ok_or_else(|| ForaDoModelo::na(linha, format!("não entendi `{}`", texto.chars().take(30).collect::<String>())))?;
    let mut campos: Vec<&str> = dados.split(',').map(str::trim).collect();
    let mut marcas = Vec::new();
    while let Some(m) = campos.first().and_then(|c| Marca::de(c)) {
        marcas.push(m);
        campos.remove(0);
    }
    let fora = |motivo: &str| ForaDoModelo::na(linha, motivo.to_string());
    let fim = |s: &str| -> Result<Fim, ForaDoModelo> {
        if data_valida(s) {
            Ok(Fim::Data(s.to_string()))
        } else if duracao_valida(s) {
            Ok(Fim::Duracao(s.to_string()))
        } else {
            Err(ForaDoModelo::na(linha, format!("`{s}` não é data nem duração")))
        }
    };
    let inicio = |s: &str| -> Result<Inicio, ForaDoModelo> {
        if data_valida(s) {
            Ok(Inicio::Data(s.to_string()))
        } else if let Some(ids) = s.strip_prefix("after ") {
            let ids: Vec<String> = ids.split_whitespace().map(str::to_string).collect();
            if ids.is_empty() || ids.iter().any(|i| !id_valido(i)) {
                return Err(ForaDoModelo::na(linha, format!("`{s}` não é dependência que o construtor monta")));
            }
            Ok(Inicio::Depois(ids))
        } else {
            Err(ForaDoModelo::na(linha, format!("`{s}` não é início que o construtor monta")))
        }
    };
    let (id, ini, f) = match campos.as_slice() {
        [f] => (None, Inicio::Anterior, fim(f)?),
        [i, f] => (None, inicio(i)?, fim(f)?),
        [id, i, f] => {
            if !id_valido(id) {
                return Err(ForaDoModelo::na(linha, format!("`{id}` não serve de id de tarefa")));
            }
            (Some(id.to_string()), inicio(i)?, fim(f)?)
        }
        _ => return Err(fora("tarefa com campos que o construtor não monta")),
    };
    Ok(Tarefa { nome: desescapar(nome.trim()), marcas, id, inicio: ini, fim: f })
}

#[cfg(test)]
mod tests {
    use super::super::{escrever, ler, Diagrama};
    use super::*;

    fn gantt(codigo: &str) -> Gantt {
        match ler(codigo) {
            Ok(Diagrama::Gantt(g)) => g,
            outro => panic!("não leu como gantt: {outro:?}"),
        }
    }

    const CANONICO: &str = concat!(
        "gantt\n",
        "  title Lançamento\n",
        "  dateFormat YYYY-MM-DD\n",
        "  Preparar :2026-10-01, 2d\n",
        "  section Construção\n",
        "    Núcleo :done, a1, 2026-10-03, 3d\n",
        "    Tela :active, crit, a2, after a1, 2026-10-12\n",
        "    Revisão#58; final :1w\n",
        "  section Entrega\n",
        "    Publicar :milestone, after a1 a2, 0.5d\n",
    );

    #[test]
    fn canonico_volta_igual() {
        let g = gantt(CANONICO);
        assert_eq!(g.titulo.as_deref(), Some("Lançamento"));
        assert_eq!(g.secoes[0].nome, None);
        let t = &g.secoes[1].tarefas;
        assert_eq!(t[0].marcas, [Marca::Feita]);
        assert_eq!(t[1].inicio, Inicio::Depois(vec!["a1".into()]));
        assert_eq!(t[1].fim, Fim::Data("2026-10-12".into()));
        assert_eq!(t[2].nome, "Revisão: final");
        assert_eq!(t[2].inicio, Inicio::Anterior);
        assert_eq!(g.secoes[2].tarefas[0].inicio, Inicio::Depois(vec!["a1".into(), "a2".into()]));
        assert_eq!(escrever(&Diagrama::Gantt(g)), CANONICO);
    }

    #[test]
    fn espacamento_a_mao_reescreve_no_mesmo_modelo() {
        let a_mao = gantt("gantt\n    section A\n        x : a1,2026-01-01,3d\n        y : after a1 , 2d");
        assert_eq!(gantt(&escrever(&Diagrama::Gantt(a_mao.clone()))), a_mao);
    }

    #[test]
    fn duracao() {
        assert!(duracao_valida("3d") && duracao_valida("1.5w") && duracao_valida("12h"));
        assert!(!duracao_valida("d") && !duracao_valida("0d") && !duracao_valida("3m") && !duracao_valida(""));
    }

    #[test]
    fn o_que_nao_monta_diz_a_linha() {
        let casos = [
            ("gantt\n  dateFormat DD/MM/YYYY", 2, "dateFormat"),
            ("gantt\n  excludes weekends", 2, "excludes"),
            ("gantt\n  section A\n  x :a1, 2026-13-01, 3d", 3, "não é início"),
            ("gantt\n  x :a1, 2026-01-01, depois", 2, "não é data nem duração"),
            ("gantt\n  sem dois pontos", 2, "não entendi"),
        ];
        for (codigo, linha, trecho) in casos {
            let e = ler(codigo).expect_err(codigo);
            assert_eq!(e.linha, linha, "{codigo}: {e}");
            assert!(e.motivo.contains(trecho), "{codigo}: {e}");
        }
    }
}
