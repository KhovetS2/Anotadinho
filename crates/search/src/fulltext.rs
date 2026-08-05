//! Full-text search index (SQLite FTS5).
//!
//! Stub: implementação concreta virá no ciclo 011.

use anyhow::Result;

use crate::SearchResult;

/// Index de busca full-text.
pub struct SearchIndex;

impl SearchIndex {
    /// Adiciona um bloco ao índice.
    ///
    /// TODO(ciclo 011): usar FTS5 virtual table.
    pub fn index_block(&mut self, _block_id: &str, _text: &str) -> Result<()> {
        anyhow::bail!("SearchIndex::index_block ainda nao implementado (vai no ciclo 011)")
    }

    /// Busca no índice.
    ///
    /// TODO(ciclo 011): query FTS5 com snippet.
    pub fn search(&self, _query: &str, _limit: usize) -> Result<Vec<SearchResult>> {
        anyhow::bail!("SearchIndex::search ainda nao implementado (vai no ciclo 011)")
    }
}
