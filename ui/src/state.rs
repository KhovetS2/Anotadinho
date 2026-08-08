//! Estado global da aplicação.
//!
//! Mantém o vault atual, nome do vault, e persiste no localStorage
//! via `gloo-storage` para reabrir automaticamente na próxima sessão.

use gloo_storage::Storage;
use serde::{Deserialize, Serialize};

const KEY_VAULT_PATH: &str = "anotadinho.vault_path";
const KEY_VAULT_NAME: &str = "anotadinho.vault_name";
const KEY_AUTOSAVE_ENABLED: &str = "anotadinho.autosave_enabled";
const KEY_HOME_PAGE_PREFIX: &str = "anotadinho.home_page::";

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

/// Salva a preferência de salvamento automático no localStorage.
pub fn save_autosave_enabled(enabled: bool) {
    let _ = gloo_storage::LocalStorage::set(KEY_AUTOSAVE_ENABLED, enabled);
}

/// Carrega a preferência de salvamento automático (padrão: ativado — sem
/// isso o usuário perdia edições ao trocar de página sem salvar antes).
pub fn load_autosave_enabled() -> bool {
    gloo_storage::LocalStorage::get(KEY_AUTOSAVE_ENABLED).unwrap_or(true)
}

/// Chave de storage da página inicial — por vault (cada vault tem a sua
/// própria página de início, guardadas separadamente pelo path do vault).
fn key_home_page(vault_path: &str) -> String {
    format!("{}{}", KEY_HOME_PAGE_PREFIX, vault_path)
}

/// Marca `page_path` como a página inicial deste vault — aberta
/// automaticamente ao abrir o vault (ver `App`).
pub fn save_home_page(vault_path: &str, page_path: &str) {
    let _ = gloo_storage::LocalStorage::set(key_home_page(vault_path), page_path);
}

/// Path da página inicial deste vault, se alguma tiver sido definida.
pub fn load_home_page(vault_path: &str) -> Option<String> {
    gloo_storage::LocalStorage::get(key_home_page(vault_path)).ok()
}

/// Remove a página inicial deste vault.
pub fn clear_home_page(vault_path: &str) {
    let _ = gloo_storage::LocalStorage::delete(key_home_page(vault_path));
}
