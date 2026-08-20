//! Anotadinho search: full-text (SQLite FTS5) e embeddings opcionais
//! (futuro).

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
    /// De que tipo de embed o resultado veio, já em texto pra mostrar
    /// ("card em Backlog", "linha de tabela"). `None` = texto solto da
    /// página (ciclo 188).
    pub origem: Option<String>,
    /// Índice do embed na página e do registro dentro dele
    /// (`"<segmento>:<registro>"`), pra o resultado poder levar até lá.
    pub ancora: Option<String>,
}
