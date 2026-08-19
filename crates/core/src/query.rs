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
        };
        let yaml = serde_yaml::to_string(&q).unwrap();
        let back: Query = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back, q);
        assert!(yaml.contains("where:"), "o campo de condições sai como `where` no YAML: {yaml}");
    }
}
