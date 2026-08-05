//! Estado global da aplicação.
//!
//! Por enquanto, apenas um stub. State real será adicionado nos
//! próximos ciclos (vault atual, lista de páginas, etc).

use serde::{Deserialize, Serialize};

/// Estado da aplicação.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// Vault aberto (None = nenhum).
    pub vault_path: Option<String>,
}

impl AppState {
    /// Cria um estado novo vazio.
    pub fn new() -> Self {
        Self::default()
    }
}
