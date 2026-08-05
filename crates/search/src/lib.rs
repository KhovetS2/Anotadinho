//! Anotadinho search: full-text e embeddings opcionais.
//!
//! Stub: implementação concreta virá no ciclo 011.

#![warn(missing_docs)]

pub mod fulltext;

pub use fulltext::SearchIndex;

/// Tipo de resultado de busca.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// ID do bloco encontrado.
    pub block_id: String,
    /// Path da página.
    pub page_path: String,
    /// Snippet com highlight (futuro).
    pub snippet: String,
    /// Score de relevância.
    pub score: f32,
}
