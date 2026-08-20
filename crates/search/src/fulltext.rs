//! Full-text search index (SQLite FTS5).
//!
//! Reconstruído do zero a cada busca (não é um índice persistido nem
//! mantido incrementalmente) — o custo é o mesmo do scanner ingênuo que
//! isso substitui (ler todas as páginas do vault), mas a QUALIDADE do
//! resultado é muito melhor: ranking de verdade (BM25), múltiplos
//! trechos possíveis por página (via `snippet()` do FTS5) em vez de só
//! a primeira ocorrência, e múltiplos termos de busca (`OR` entre
//! palavras da query, com prefix-match) em vez de um substring literal
//! só. `rusqlite` com feature `bundled` já era uma dependência declarada
//! (não usada até este ciclo) — o SQLite embarcado já compila com
//! `-DSQLITE_ENABLE_FTS5`, então não precisa de nenhuma dependência de
//! sistema nova.

use anyhow::{Context, Result};
use rusqlite::{params, Connection};

use crate::SearchResult;

/// Index de busca full-text — vive só durante uma consulta (`new()` +
/// `index_page()` várias vezes + `search()`), não é reaproveitado entre
/// consultas.
pub struct SearchIndex {
    conn: Connection,
}

impl SearchIndex {
    /// Cria um índice novo, vazio, em memória.
    pub fn new() -> Result<Self> {
        let conn = Connection::open_in_memory().context("erro ao abrir SQLite em memória")?;
        // `origem`/`ancora` são UNINDEXED: entram no resultado, mas não
        // participam do casamento — sem isso, buscar "card" acharia
        // todo card do vault (ciclo 188).
        conn.execute_batch(
            "CREATE VIRTUAL TABLE docs USING fts5(\
             path, title, content, origem UNINDEXED, ancora UNINDEXED);",
        )
            .context("erro ao criar tabela FTS5")?;
        Ok(Self { conn })
    }

    /// Adiciona uma página ao índice.
    pub fn index_page(&mut self, path: &str, title: &str, content: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO docs (path, title, content, origem, ancora) \
                 VALUES (?1, ?2, ?3, NULL, NULL)",
                params![path, title, content],
            )
            .context("erro ao indexar página")?;
        Ok(())
    }

    /// Adiciona UM registro de dentro de um embed (ciclo 188): um card,
    /// uma linha de tabela, um evento. Vira um documento próprio no
    /// índice, então o resultado sabe dizer o que é e pra onde levar.
    pub fn index_embed_entry(
        &mut self,
        path: &str,
        title: &str,
        texto: &str,
        origem: &str,
        ancora: &str,
    ) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO docs (path, title, content, origem, ancora) \
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![path, title, texto, origem, ancora],
            )
            .context("erro ao indexar registro de embed")?;
        Ok(())
    }

    /// Busca no índice — cada palavra da query casa como prefixo,
    /// unidas por `OR` (mais tolerante que exigir todas as palavras);
    /// ranking por BM25 (SQLite retorna score onde MENOR é MELHOR —
    /// invertido aqui pra `score` seguir a convenção usual de "maior é
    /// melhor").
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let fts_query = build_fts_query(query);
        if fts_query.is_empty() {
            return Ok(Vec::new());
        }
        let mut stmt = self
            .conn
            .prepare(
                "SELECT path, snippet(docs, 2, '**', '**', '...', 10), bm25(docs), origem, ancora \
                 FROM docs WHERE docs MATCH ?1 ORDER BY bm25(docs) LIMIT ?2",
            )
            .context("erro ao preparar busca")?;
        let rows = stmt
            .query_map(params![fts_query, limit as i64], |row| {
                let path: String = row.get(0)?;
                let snippet: String = row.get(1)?;
                let bm25: f64 = row.get(2)?;
                let origem: Option<String> = row.get(3)?;
                let ancora: Option<String> = row.get(4)?;
                Ok(SearchResult {
                    block_id: path.clone(),
                    page_path: path,
                    snippet,
                    score: -bm25 as f32,
                    origem,
                    ancora,
                })
            })
            .context("erro ao executar busca")?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r.context("erro ao ler resultado")?);
        }
        Ok(out)
    }
}

/// Monta a query FTS5 a partir do texto digitado pelo usuário — cada
/// palavra vira um termo de prefixo (`"palavra"*`), entre aspas pra
/// blindar contra sintaxe de operador do FTS5 injetada sem querer
/// (`AND`/`OR`/`NOT`/`-`/parênteses digitados pelo usuário viram texto
/// literal, não comandos).
fn build_fts_query(query: &str) -> String {
    query
        .split_whitespace()
        .map(|t| t.replace('"', "\"\""))
        .filter(|t| !t.is_empty())
        .map(|t| format!("\"{}\"*", t))
        .collect::<Vec<_>>()
        .join(" OR ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_index() -> SearchIndex {
        let mut idx = SearchIndex::new().unwrap();
        idx.index_page("pages/alpha.md", "alpha", "Isto é uma página sobre kanban e produtividade.").unwrap();
        idx.index_page("pages/beta.md", "beta", "Beta fala sobre calendário e tarefas do dia a dia.").unwrap();
        idx.index_page("pages/gamma.md", "gamma", "Gamma não tem nada relacionado às outras duas.").unwrap();
        idx
    }

    #[test]
    fn search_finds_matching_page() {
        let idx = sample_index();
        let results = idx.search("kanban", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_path, "pages/alpha.md");
    }

    #[test]
    fn search_prefix_match() {
        let idx = sample_index();
        let results = idx.search("produt", 10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].page_path, "pages/alpha.md");
    }

    #[test]
    fn search_multi_word_or_matches_either() {
        let idx = sample_index();
        let results = idx.search("kanban calendário", 10).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn search_no_match_returns_empty() {
        let idx = sample_index();
        let results = idx.search("inexistente_xyz", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_empty_query_returns_empty() {
        let idx = sample_index();
        let results = idx.search("   ", 10).unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn search_respects_limit() {
        let idx = sample_index();
        let results = idx.search("a", 1).unwrap();
        assert_eq!(results.len(), 1);
    }

    #[test]
    fn search_snippet_highlights_match() {
        let idx = sample_index();
        let results = idx.search("kanban", 10).unwrap();
        assert!(results[0].snippet.contains("**kanban**"));
    }

    #[test]
    fn search_rejects_operator_injection() {
        // Sem as aspas de blindagem, "NOT" seria interpretado como
        // operador do FTS5 (exclui resultados) em vez de texto literal.
        let idx = sample_index();
        let results = idx.search("NOT kanban", 10).unwrap();
        assert_eq!(results.len(), 1, "\"NOT\" deveria ser tratado como termo literal, não operador");
    }
}
