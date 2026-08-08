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
const KEY_VIM_MODE_ENABLED: &str = "anotadinho.vim_mode_enabled";
const KEY_VIM_KEYMAP: &str = "anotadinho.vim_keymap";

/// Mapa de teclas do modo Normal do vim mode — cada ação tem UMA tecla
/// configurável. `delete_line`/`yank_line` são especiais: pressionar a
/// tecla configurada DUAS vezes seguidas confirma a ação (mesmo padrão
/// mnemônico do vim `dd`/`yy`, mas sobre a tecla que o usuário escolher,
/// não fixo em "d"/"y").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VimKeymap {
    pub left: String,
    pub down: String,
    pub up: String,
    pub right: String,
    pub word_forward: String,
    pub word_backward: String,
    pub line_start: String,
    pub line_end: String,
    pub doc_start: String,
    pub doc_end: String,
    pub insert_before: String,
    pub insert_after: String,
    pub open_below: String,
    pub open_above: String,
    pub delete_char: String,
    pub delete_line: String,
    pub yank_line: String,
    pub paste: String,
    pub undo: String,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            left: "h".into(),
            down: "j".into(),
            up: "k".into(),
            right: "l".into(),
            word_forward: "w".into(),
            word_backward: "b".into(),
            line_start: "0".into(),
            line_end: "$".into(),
            doc_start: "g".into(),
            doc_end: "G".into(),
            insert_before: "i".into(),
            insert_after: "a".into(),
            open_below: "o".into(),
            open_above: "O".into(),
            delete_char: "x".into(),
            delete_line: "d".into(),
            yank_line: "y".into(),
            paste: "p".into(),
            undo: "u".into(),
        }
    }
}

impl VimKeymap {
    /// Lista `(rótulo, campo)` — usada pela tela de configuração de
    /// atalhos pra iterar todas as ações sem repetir os nomes na UI.
    pub fn labeled_fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("Esquerda", &self.left),
            ("Baixo", &self.down),
            ("Cima", &self.up),
            ("Direita", &self.right),
            ("Palavra seguinte", &self.word_forward),
            ("Palavra anterior", &self.word_backward),
            ("Início da linha", &self.line_start),
            ("Fim da linha", &self.line_end),
            ("Início do documento", &self.doc_start),
            ("Fim do documento", &self.doc_end),
            ("Inserir antes do cursor", &self.insert_before),
            ("Inserir depois do cursor", &self.insert_after),
            ("Nova linha abaixo", &self.open_below),
            ("Nova linha acima", &self.open_above),
            ("Apagar caractere", &self.delete_char),
            ("Apagar linha (2x)", &self.delete_line),
            ("Copiar linha (2x)", &self.yank_line),
            ("Colar", &self.paste),
            ("Desfazer", &self.undo),
        ]
    }

    /// Atualiza o campo correspondente ao rótulo (mesmos rótulos de
    /// `labeled_fields`). Não faz nada se o rótulo não existir.
    pub fn set_by_label(&mut self, label: &str, key: String) {
        match label {
            "Esquerda" => self.left = key,
            "Baixo" => self.down = key,
            "Cima" => self.up = key,
            "Direita" => self.right = key,
            "Palavra seguinte" => self.word_forward = key,
            "Palavra anterior" => self.word_backward = key,
            "Início da linha" => self.line_start = key,
            "Fim da linha" => self.line_end = key,
            "Início do documento" => self.doc_start = key,
            "Fim do documento" => self.doc_end = key,
            "Inserir antes do cursor" => self.insert_before = key,
            "Inserir depois do cursor" => self.insert_after = key,
            "Nova linha abaixo" => self.open_below = key,
            "Nova linha acima" => self.open_above = key,
            "Apagar caractere" => self.delete_char = key,
            "Apagar linha (2x)" => self.delete_line = key,
            "Copiar linha (2x)" => self.yank_line = key,
            "Colar" => self.paste = key,
            "Desfazer" => self.undo = key,
            _ => {}
        }
    }
}

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

/// Salva se o vim mode está ativado.
pub fn save_vim_mode_enabled(enabled: bool) {
    let _ = gloo_storage::LocalStorage::set(KEY_VIM_MODE_ENABLED, enabled);
}

/// Carrega se o vim mode está ativado (padrão: desativado).
pub fn load_vim_mode_enabled() -> bool {
    gloo_storage::LocalStorage::get(KEY_VIM_MODE_ENABLED).unwrap_or(false)
}

/// Salva o mapa de teclas do vim mode.
pub fn save_vim_keymap(keymap: &VimKeymap) {
    let _ = gloo_storage::LocalStorage::set(KEY_VIM_KEYMAP, keymap);
}

/// Carrega o mapa de teclas do vim mode (padrão: teclas clássicas do vim).
pub fn load_vim_keymap() -> VimKeymap {
    gloo_storage::LocalStorage::get(KEY_VIM_KEYMAP).unwrap_or_default()
}
