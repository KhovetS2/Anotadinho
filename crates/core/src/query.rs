//! Motor de consulta sobre o índice do vault (`crate::index`).
//!
//! O esquema de agent-os (produto / specs / decisões / padrões, com
//! `status`, `priority`, `dominio` no frontmatter) só era navegável na
//! mão ou pelo CLI: a única visão agregada dentro do app era um kanban
//! MANUAL que alguém precisava lembrar de mover. Uma `Query` é o
//! recorte declarado — "specs em backlog ordenadas por prioridade" —
//! que o embed `{{ type: "query" }}` renderiza e o `anotadinho-cli`
//! executa no terminal. Um motor só, duas superfícies: o que o humano
//! vê na página é exatamente o que o agente lê no terminal.
//!
//! Condições são combinadas com E (AND). Sem OR nem parênteses de
//! propósito: é o que o esquema precisa, e a alternativa seria uma
//! linguagem de expressão inteira dentro do YAML.

use serde::{Deserialize, Serialize};

use crate::index::PageIndexEntry;

/// Comparação de uma condição.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryOp {
    /// Igual (ignora caixa).
    #[default]
    Eq,
    /// Diferente (ignora caixa).
    Neq,
    /// Contém o texto (ignora caixa).
    Contains,
    /// Campo existe e não está vazio.
    Exists,
    /// Maior que — numérico se os dois lados forem número, senão
    /// alfabético (datas `YYYY-MM-DD` ordenam certo dos dois jeitos).
    Gt,
    /// Menor que, mesma regra do `Gt`.
    Lt,
}

impl QueryOp {
    /// Todos os operadores, na ordem em que aparecem no seletor.
    pub fn all() -> &'static [QueryOp] {
        &[Self::Eq, Self::Neq, Self::Contains, Self::Exists, Self::Gt, Self::Lt]
    }

    /// Símbolo usado no CLI (`campo=valor`, `campo!=valor`, ...).
    pub fn symbol(&self) -> &'static str {
        match self {
            Self::Eq => "=",
            Self::Neq => "!=",
            Self::Contains => "~",
            Self::Exists => "?",
            Self::Gt => ">",
            Self::Lt => "<",
        }
    }

    /// Nome de exibição.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Eq => "é",
            Self::Neq => "não é",
            Self::Contains => "contém",
            Self::Exists => "existe",
            Self::Gt => "maior que",
            Self::Lt => "menor que",
        }
    }
}

/// Uma condição sobre um campo (fixo ou de frontmatter/property).
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Condition {
    /// Nome do campo (`status`, `priority`, `title`, `type`...).
    pub field: String,
    /// Comparação.
    #[serde(default)]
    pub op: QueryOp,
    /// Valor comparado. Ignorado por `Exists`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub value: String,
}

impl Condition {
    /// Avalia a condição contra uma entrada.
    pub fn matches(&self, entry: &PageIndexEntry) -> bool {
        let actual = entry.field(&self.field);
        match self.op {
            QueryOp::Exists => actual.map(|v| !v.trim().is_empty()).unwrap_or(false),
            QueryOp::Eq => actual
                .map(|v| v.eq_ignore_ascii_case(&self.value))
                .unwrap_or(false),
            // Campo ausente É diferente do valor procurado — senão
            // "specs que não estão em done" perderia toda página que
            // nem tem `status`, que é justamente onde mora o trabalho
            // não classificado.
            QueryOp::Neq => actual
                .map(|v| !v.eq_ignore_ascii_case(&self.value))
                .unwrap_or(true),
            QueryOp::Contains => actual
                .map(|v| v.to_lowercase().contains(&self.value.to_lowercase()))
                .unwrap_or(false),
            QueryOp::Gt => compare(actual.as_deref(), &self.value)
                .map(|o| o == std::cmp::Ordering::Greater)
                .unwrap_or(false),
            QueryOp::Lt => compare(actual.as_deref(), &self.value)
                .map(|o| o == std::cmp::Ordering::Less)
                .unwrap_or(false),
        }
    }
}

/// Compara dois textos: numericamente quando os dois parseiam como
/// número, alfabeticamente caso contrário.
fn compare(a: Option<&str>, b: &str) -> Option<std::cmp::Ordering> {
    let a = a?;
    match (a.trim().parse::<f64>(), b.trim().parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y),
        _ => Some(a.to_lowercase().cmp(&b.to_lowercase())),
    }
}

/// Como os resultados são desenhados.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum QueryView {
    /// Uma linha por página, com os campos extras em subtítulo.
    #[default]
    List,
    /// Uma coluna por campo de `columns`.
    Table,
    /// Grade de cartões.
    Cards,
}

impl QueryView {
    /// Todas as visões, na ordem do seletor.
    pub fn all() -> &'static [QueryView] {
        &[Self::List, Self::Table, Self::Cards]
    }

    /// Nome no YAML e no modificador BEM.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Table => "table",
            Self::Cards => "cards",
        }
    }

    /// Nome de exibição.
    pub fn label(&self) -> &'static str {
        match self {
            Self::List => "Lista",
            Self::Table => "Tabela",
            Self::Cards => "Cartões",
        }
    }
}

/// Ordenação dos resultados.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Sort {
    /// Campo pelo qual ordenar.
    pub field: String,
    /// Decrescente.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub desc: bool,
}

/// Operação de agregado sobre um campo do grupo (ciclo 169).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AggregateOp {
    /// Quantas páginas (ignora o campo).
    #[default]
    Count,
    /// Soma dos valores numéricos.
    Sum,
    /// Média dos valores numéricos.
    Avg,
    /// Menor valor (numérico se der, senão alfabético).
    Min,
    /// Maior valor, mesma regra do `Min`.
    Max,
}

impl AggregateOp {
    /// Todas as operações, na ordem do seletor.
    pub fn all() -> &'static [AggregateOp] {
        &[Self::Count, Self::Sum, Self::Avg, Self::Min, Self::Max]
    }

    /// Nome no YAML.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Count => "count",
            Self::Sum => "sum",
            Self::Avg => "avg",
            Self::Min => "min",
            Self::Max => "max",
        }
    }

    /// Rótulo curto mostrado no rodapé do grupo.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Count => "total",
            Self::Sum => "soma",
            Self::Avg => "média",
            Self::Min => "mín",
            Self::Max => "máx",
        }
    }
}

/// Um agregado declarado na consulta.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Aggregate {
    /// Campo agregado (ignorado por `count`).
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub field: String,
    /// Operação.
    #[serde(default)]
    pub op: AggregateOp,
}

impl Aggregate {
    /// Calcula sobre as páginas do grupo. Valor não-numérico é IGNORADO
    /// em `sum`/`avg` (somar texto daria lixo silencioso); `min`/`max`
    /// caem pra comparação alfabética quando não são números.
    pub fn calcular(&self, itens: &[&PageIndexEntry]) -> String {
        if self.op == AggregateOp::Count {
            return itens.len().to_string();
        }
        let valores: Vec<String> = itens
            .iter()
            .filter_map(|e| e.field(&self.field))
            .filter(|v| !v.trim().is_empty())
            .collect();
        if valores.is_empty() {
            return "—".to_string();
        }
        let numeros: Vec<f64> = valores.iter().filter_map(|v| v.trim().parse::<f64>().ok()).collect();
        match self.op {
            AggregateOp::Count => unreachable!("tratado acima"),
            AggregateOp::Sum | AggregateOp::Avg => {
                if numeros.is_empty() {
                    return "—".to_string();
                }
                let soma: f64 = numeros.iter().sum();
                let valor = if self.op == AggregateOp::Sum {
                    soma
                } else {
                    soma / numeros.len() as f64
                };
                formatar_numero(valor)
            }
            AggregateOp::Min | AggregateOp::Max => {
                if numeros.len() == valores.len() {
                    let escolhido = if self.op == AggregateOp::Min {
                        numeros.iter().cloned().fold(f64::INFINITY, f64::min)
                    } else {
                        numeros.iter().cloned().fold(f64::NEG_INFINITY, f64::max)
                    };
                    formatar_numero(escolhido)
                } else {
                    let mut ordenados = valores.clone();
                    ordenados.sort();
                    if self.op == AggregateOp::Min {
                        ordenados.first().cloned().unwrap_or_default()
                    } else {
                        ordenados.last().cloned().unwrap_or_default()
                    }
                }
            }
        }
    }
}

/// Número sem casa decimal à toa (`3` em vez de `3.0`).
fn formatar_numero(v: f64) -> String {
    if (v - v.round()).abs() < f64::EPSILON {
        format!("{}", v.round() as i64)
    } else {
        format!("{v:.2}")
    }
}

/// Um grupo de resultados (ciclo 169).
#[derive(Debug, Clone, PartialEq)]
pub struct Grupo<'a> {
    /// Valor do campo de agrupamento (vazio = grupo dos "sem campo").
    pub valor: String,
    /// Rótulo pronto pra exibir.
    pub rotulo: String,
    /// Páginas do grupo, já ordenadas e limitadas.
    pub itens: Vec<&'a PageIndexEntry>,
    /// Agregados calculados, na ordem declarada.
    pub agregados: Vec<(String, String)>,
}

/// Um recorte do vault.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Query {
    /// Prefixo de path (`pages/specs`). Vazio = vault inteiro.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    /// Tags que a página precisa ter — todas (AND).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Condições sobre campos, todas precisam bater (AND).
    #[serde(default, rename = "where", skip_serializing_if = "Vec::is_empty")]
    pub conditions: Vec<Condition>,
    /// Ordenação.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sort: Option<Sort>,
    /// Máximo de resultados.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub limit: Option<usize>,
    /// Como desenhar.
    #[serde(default)]
    pub view: QueryView,
    /// Campos extras mostrados junto do título.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub columns: Vec<String>,
    /// Agrupa os resultados por este campo (ciclo 169). Sem isso, uma
    /// visão "por status" exigia uma consulta POR status.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub group_by: Option<String>,
    /// Agregados mostrados no rodapé de cada grupo e no total.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub aggregate: Vec<Aggregate>,
    /// Grupos recolhidos — guardado no YAML pra o painel abrir do jeito
    /// que ficou.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub collapsed: Vec<String>,
}

impl Query {
    /// Aplica filtros, ordenação e limite sobre o índice.
    ///
    /// A ordem base (antes de `sort`) é a que veio do índice — a
    /// listagem do vault, que já é estável.
    pub fn run<'a>(&self, entries: &'a [PageIndexEntry]) -> Vec<&'a PageIndexEntry> {
        let mut out: Vec<&PageIndexEntry> = entries
            .iter()
            .filter(|e| self.matches(e))
            .collect();

        if let Some(sort) = &self.sort {
            out.sort_by(|a, b| {
                let va = a.field(&sort.field).filter(|v| !v.trim().is_empty());
                let vb = b.field(&sort.field).filter(|v| !v.trim().is_empty());
                // Página sem o campo vai pro FIM nos dois sentidos: ela
                // não é "a menor", ela não participa da ordenação.
                let ord = match (&va, &vb) {
                    (None, None) => std::cmp::Ordering::Equal,
                    (None, Some(_)) => return std::cmp::Ordering::Greater,
                    (Some(_), None) => return std::cmp::Ordering::Less,
                    (Some(x), Some(y)) => {
                        compare(Some(x), y).unwrap_or(std::cmp::Ordering::Equal)
                    }
                };
                if sort.desc { ord.reverse() } else { ord }
            });
        }

        if let Some(limit) = self.limit {
            out.truncate(limit);
        }
        out
    }

    /// Roda a consulta e agrupa por `group_by` (ciclo 169).
    ///
    /// Ordem dos grupos: alfabética pelo valor, com o grupo dos SEM
    /// CAMPO sempre no fim — mesma regra do `sort` (ausente não é o
    /// menor, é ausente). Sem `group_by`, devolve um grupo só, sem
    /// rótulo.
    pub fn run_grouped<'a>(&self, entries: &'a [PageIndexEntry]) -> Vec<Grupo<'a>> {
        let resultados = self.run(entries);
        let Some(campo) = self.group_by.as_ref().filter(|c| !c.trim().is_empty()) else {
            return vec![Grupo {
                valor: String::new(),
                rotulo: String::new(),
                agregados: self.agregar(&resultados),
                itens: resultados,
            }];
        };

        let mut ordem: Vec<String> = Vec::new();
        let mut por_valor: std::collections::BTreeMap<String, Vec<&PageIndexEntry>> = Default::default();
        for entry in resultados {
            let valor = entry
                .field(campo)
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_default();
            if !ordem.contains(&valor) {
                ordem.push(valor.clone());
            }
            por_valor.entry(valor).or_default().push(entry);
        }
        ordem.sort();
        // Sem valor vai pro fim.
        ordem.sort_by_key(|v| v.is_empty());

        ordem
            .into_iter()
            .map(|valor| {
                let itens = por_valor.remove(&valor).unwrap_or_default();
                Grupo {
                    rotulo: if valor.is_empty() {
                        format!("sem {campo}")
                    } else {
                        valor.clone()
                    },
                    agregados: self.agregar(&itens),
                    valor,
                    itens,
                }
            })
            .collect()
    }

    /// Se o grupo está recolhido.
    pub fn recolhido(&self, valor: &str) -> bool {
        self.collapsed.iter().any(|c| c == valor)
    }

    /// Alterna o estado de recolhido de um grupo.
    pub fn alternar_recolhido(&mut self, valor: &str) {
        if let Some(pos) = self.collapsed.iter().position(|c| c == valor) {
            self.collapsed.remove(pos);
        } else {
            self.collapsed.push(valor.to_string());
        }
    }

    fn agregar(&self, itens: &[&PageIndexEntry]) -> Vec<(String, String)> {
        self.aggregate
            .iter()
            .map(|a| {
                let rotulo = if a.op == AggregateOp::Count || a.field.is_empty() {
                    a.op.label().to_string()
                } else {
                    format!("{} {}", a.op.label(), a.field)
                };
                (rotulo, a.calcular(itens))
            })
            .collect()
    }

    /// Se a entrada passa por pasta, tags e condições.
    fn matches(&self, entry: &PageIndexEntry) -> bool {
        if let Some(from) = &self.from {
            let from = from.trim().trim_end_matches('/');
            if !from.is_empty() && !entry.path.starts_with(from) {
                return false;
            }
        }
        if !self
            .tags
            .iter()
            .filter(|t| !t.trim().is_empty())
            .all(|t| entry.tags.iter().any(|et| et.eq_ignore_ascii_case(t)))
        {
            return false;
        }
        self.conditions
            .iter()
            .filter(|c| !c.field.trim().is_empty())
            .all(|c| c.matches(entry))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(path: &str, title: &str, tags: &[&str], props: &[(&str, &str)]) -> PageIndexEntry {
        let mut e = PageIndexEntry {
            path: path.to_string(),
            title: title.to_string(),
            section: "pages".to_string(),
            page_type: "md".to_string(),
            tags: tags.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        };
        for (k, v) in props {
            e.properties.insert(k.to_string(), v.to_string());
        }
        e
    }

    fn sample() -> Vec<PageIndexEntry> {
        vec![
            entry("pages/specs/a.md", "Spec A", &["spec"], &[("status", "backlog"), ("priority", "alta"), ("peso", "3")]),
            entry("pages/specs/b.md", "Spec B", &["spec", "api"], &[("status", "done"), ("priority", "baixa"), ("peso", "10")]),
            entry("pages/decisoes/c.md", "Decisão C", &["decisao"], &[("status", "aceita")]),
            entry("pages/specs/d.md", "Spec D", &["spec"], &[]),
        ]
    }

    fn paths(result: Vec<&PageIndexEntry>) -> Vec<String> {
        result.iter().map(|e| e.path.clone()).collect()
    }

    #[test]
    fn consulta_vazia_devolve_tudo() {
        let all = sample();
        assert_eq!(Query::default().run(&all).len(), 4);
    }

    #[test]
    fn filtra_por_pasta() {
        let all = sample();
        let q = Query { from: Some("pages/specs".into()), ..Default::default() };
        assert_eq!(q.run(&all).len(), 3);
    }

    #[test]
    fn filtra_por_tags_em_and() {
        let all = sample();
        let q = Query { tags: vec!["spec".into(), "api".into()], ..Default::default() };
        assert_eq!(paths(q.run(&all)), vec!["pages/specs/b.md"]);
    }

    #[test]
    fn operador_eq_e_contains() {
        let all = sample();
        let eq = Query {
            conditions: vec![Condition { field: "status".into(), op: QueryOp::Eq, value: "BACKLOG".into() }],
            ..Default::default()
        };
        assert_eq!(paths(eq.run(&all)), vec!["pages/specs/a.md"]);

        let contains = Query {
            conditions: vec![Condition { field: "title".into(), op: QueryOp::Contains, value: "spec".into() }],
            ..Default::default()
        };
        assert_eq!(contains.run(&all).len(), 3);
    }

    #[test]
    fn neq_inclui_pagina_sem_o_campo() {
        let all = sample();
        let q = Query {
            from: Some("pages/specs".into()),
            conditions: vec![Condition { field: "status".into(), op: QueryOp::Neq, value: "done".into() }],
            ..Default::default()
        };
        // A (backlog) e D (sem status) — D é justamente o trabalho não
        // classificado, que não pode sumir do recorte.
        assert_eq!(paths(q.run(&all)), vec!["pages/specs/a.md", "pages/specs/d.md"]);
    }

    #[test]
    fn exists_pega_so_quem_tem_o_campo_preenchido() {
        let all = sample();
        let q = Query {
            conditions: vec![Condition { field: "priority".into(), op: QueryOp::Exists, value: String::new() }],
            ..Default::default()
        };
        assert_eq!(q.run(&all).len(), 2);
    }

    #[test]
    fn gt_e_lt_comparam_numero_como_numero() {
        let all = sample();
        let q = Query {
            conditions: vec![Condition { field: "peso".into(), op: QueryOp::Gt, value: "5".into() }],
            ..Default::default()
        };
        // Alfabeticamente "10" < "5"; numericamente 10 > 5.
        assert_eq!(paths(q.run(&all)), vec!["pages/specs/b.md"]);
    }

    #[test]
    fn gt_compara_data_como_texto() {
        let all = vec![
            entry("a.md", "A", &[], &[("date", "2026-08-01")]),
            entry("b.md", "B", &[], &[("date", "2026-09-15")]),
        ];
        let q = Query {
            conditions: vec![Condition { field: "date".into(), op: QueryOp::Gt, value: "2026-08-31".into() }],
            ..Default::default()
        };
        assert_eq!(paths(q.run(&all)), vec!["b.md"]);
    }

    #[test]
    fn ordena_asc_e_desc_com_ausentes_no_fim() {
        let all = sample();
        let asc = Query {
            from: Some("pages/specs".into()),
            sort: Some(Sort { field: "status".into(), desc: false }),
            ..Default::default()
        };
        assert_eq!(paths(asc.run(&all)), vec!["pages/specs/a.md", "pages/specs/b.md", "pages/specs/d.md"]);

        let desc = Query {
            from: Some("pages/specs".into()),
            sort: Some(Sort { field: "status".into(), desc: true }),
            ..Default::default()
        };
        assert_eq!(paths(desc.run(&all)), vec!["pages/specs/b.md", "pages/specs/a.md", "pages/specs/d.md"]);
    }

    #[test]
    fn limite_corta_o_resultado() {
        let all = sample();
        let q = Query { limit: Some(2), ..Default::default() };
        assert_eq!(q.run(&all).len(), 2);
    }

    #[test]
    fn condicao_com_campo_vazio_e_ignorada() {
        let all = sample();
        let q = Query {
            conditions: vec![Condition { field: "  ".into(), op: QueryOp::Eq, value: "x".into() }],
            ..Default::default()
        };
        assert_eq!(q.run(&all).len(), 4);
    }

    #[test]
    fn roundtrip_yaml_preserva_a_consulta() {
        let q = Query {
            from: Some("pages/specs".into()),
            tags: vec!["spec".into()],
            conditions: vec![Condition { field: "status".into(), op: QueryOp::Neq, value: "done".into() }],
            sort: Some(Sort { field: "priority".into(), desc: true }),
            limit: Some(5),
            view: QueryView::Cards,
            columns: vec!["status".into(), "priority".into()],
            group_by: Some("status".into()),
            aggregate: vec![Aggregate { field: "peso".into(), op: AggregateOp::Sum }],
            collapsed: vec!["done".into()],
        };
        let yaml = serde_yaml::to_string(&q).unwrap();
        let back: Query = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back, q);
        assert!(yaml.contains("where:"), "o campo de condições sai como `where` no YAML: {yaml}");
    }

    #[test]
    fn agrupa_por_campo_com_ausentes_no_fim() {
        let all = sample();
        let q = Query {
            from: Some("pages/specs".into()),
            group_by: Some("status".into()),
            ..Default::default()
        };
        let grupos = q.run_grouped(&all);
        let rotulos: Vec<&str> = grupos.iter().map(|g| g.rotulo.as_str()).collect();
        assert_eq!(rotulos, vec!["backlog", "done", "sem status"]);
        assert_eq!(grupos[2].itens.len(), 1, "a spec sem status é um grupo próprio");
    }

    #[test]
    fn sem_group_by_devolve_um_grupo_so() {
        let all = sample();
        let grupos = Query::default().run_grouped(&all);
        assert_eq!(grupos.len(), 1);
        assert!(grupos[0].rotulo.is_empty());
        assert_eq!(grupos[0].itens.len(), 4);
    }

    #[test]
    fn agregados_contam_somam_e_tiram_media() {
        let all = sample();
        let q = Query {
            from: Some("pages/specs".into()),
            aggregate: vec![
                Aggregate { field: String::new(), op: AggregateOp::Count },
                Aggregate { field: "peso".into(), op: AggregateOp::Sum },
                Aggregate { field: "peso".into(), op: AggregateOp::Avg },
                Aggregate { field: "peso".into(), op: AggregateOp::Max },
            ],
            ..Default::default()
        };
        let grupos = q.run_grouped(&all);
        let valores: Vec<&str> = grupos[0].agregados.iter().map(|(_, v)| v.as_str()).collect();
        // 3 specs; pesos 3 e 10 (a terceira não tem peso).
        assert_eq!(valores, vec!["3", "13", "6.50", "10"]);
    }

    #[test]
    fn agregado_de_campo_nao_numerico_nao_inventa_soma() {
        let all = sample();
        let q = Query {
            aggregate: vec![Aggregate { field: "status".into(), op: AggregateOp::Sum }],
            ..Default::default()
        };
        assert_eq!(q.run_grouped(&all)[0].agregados[0].1, "—");
    }

    #[test]
    fn min_e_max_caem_pra_alfabetico_quando_nao_sao_numeros() {
        let all = sample();
        let q = Query {
            from: Some("pages/specs".into()),
            aggregate: vec![
                Aggregate { field: "status".into(), op: AggregateOp::Min },
                Aggregate { field: "status".into(), op: AggregateOp::Max },
            ],
            ..Default::default()
        };
        let grupos = q.run_grouped(&all);
        let vals: Vec<&str> = grupos[0].agregados.iter().map(|(_, v)| v.as_str()).collect();
        assert_eq!(vals, vec!["backlog", "done"]);
    }

    #[test]
    fn grupo_recolhido_roundtrip_no_yaml() {
        let mut q = Query { group_by: Some("status".into()), ..Default::default() };
        assert!(!q.recolhido("done"));
        q.alternar_recolhido("done");
        assert!(q.recolhido("done"));
        let yaml = serde_yaml::to_string(&q).unwrap();
        let volta: Query = serde_yaml::from_str(&yaml).unwrap();
        assert!(volta.recolhido("done"));
        q.alternar_recolhido("done");
        assert!(!q.recolhido("done"));
    }
}
