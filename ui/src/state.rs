//! Estado global da aplicação.
//!
//! Mantém o vault atual, nome do vault, e persiste no localStorage
//! via `gloo-storage` para reabrir automaticamente na próxima sessão.

use gloo_storage::Storage;
use serde::{Deserialize, Serialize};

const KEY_VAULT_PATH: &str = "anotadinho.vault_path";
const KEY_VAULT_NAME: &str = "anotadinho.vault_name";

/// Estado da aplicação.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// Vault aberto (None = nenhum).
    pub vault_path: Option<String>,
    /// Nome do vault (nome do diretório).
    pub vault_name: Option<String>,
}

impl AppState {
    /// Cria um estado novo vazio.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Salva o path do vault no localStorage.
pub fn save_vault_path(path: &str) {
    let _ = gloo_storage::LocalStorage::set(KEY_VAULT_PATH, path);
}

/// Carrega o path do vault do localStorage.
pub fn load_vault_path() -> Option<String> {
    gloo_storage::LocalStorage::get(KEY_VAULT_PATH).ok()
}

/// Salva o nome do vault no localStorage.
pub fn save_vault_name(name: &str) {
    let _ = gloo_storage::LocalStorage::set(KEY_VAULT_NAME, name);
}

/// Carrega o nome do vault do localStorage.
pub fn load_vault_name() -> Option<String> {
    gloo_storage::LocalStorage::get(KEY_VAULT_NAME).ok()
}

/// Extrai o nome do diretório de um path.
pub fn extract_name_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string())
}

/// Remove vault path/name do localStorage.
pub fn clear_vault() {
    let _ = gloo_storage::LocalStorage::delete(KEY_VAULT_PATH);
    let _ = gloo_storage::LocalStorage::delete(KEY_VAULT_NAME);
}
