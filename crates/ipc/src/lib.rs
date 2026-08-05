//! Anotadinho IPC: comandos Tauri expostos pro Yew frontend.
//!
//! Os comandos IPC são a única ponte entre o Yew (no WebView) e o
//! backend Rust (Tauri core). Tudo que o UI faz passa por aqui.

#![warn(missing_docs)]

use anotadinho_vault::VaultIo;
use serde::{Deserialize, Serialize};

/// Comando de exemplo: ping.
#[derive(Debug, Serialize, Deserialize)]
pub struct PingArgs {
    /// Mensagem a ecoar.
    pub message: String,
}

/// Resposta do ping.
#[derive(Debug, Serialize, Deserialize)]
pub struct PingResult {
    /// Eco da mensagem.
    pub echo: String,
    /// Versão do app.
    pub version: String,
}

/// Handler de ping.
pub fn handle_ping(args: PingArgs) -> PingResult {
    PingResult {
        echo: format!("pong: {}", args.message),
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

/// Informações de um vault aberto.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    /// Path absoluto do vault.
    pub path: String,
    /// Nome do diretório.
    pub name: String,
}

/// Metadados de uma página listada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    /// Path relativo ao vault.
    pub path: String,
    /// Nome do arquivo (sem extensão).
    pub title: String,
    /// Seção (`pages` ou `journals`).
    pub section: String,
}

/// Handler de list_pages: retorna todas as páginas `.md` do vault.
pub fn handle_list_pages(vault_path: String) -> Result<Vec<PageMeta>, String> {
    let vault = VaultIo::open(&vault_path);
    let pages = vault.list_pages().map_err(|e| e.to_string())?;
    Ok(pages
        .into_iter()
        .map(|p| PageMeta {
            path: p.path,
            title: p.title,
            section: p.section,
        })
        .collect())
}

/// Handler de read_page: retorna o conteúdo Markdown bruto.
pub fn handle_read_page(vault_path: String, page_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.read_page(&page_path).map_err(|e| e.to_string())
}

/// Handler de write_page: grava conteúdo Markdown no disco.
pub fn handle_write_page(
    vault_path: String,
    page_path: String,
    content: String,
) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault
        .write_page(&page_path, &content)
        .map_err(|e| e.to_string())
}

/// Handler de create_page: cria nova página em pages/.
pub fn handle_create_page(vault_path: String, title: String) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault.create_page(&title).map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de open_today_journal: abre ou cria journal do dia.
pub fn handle_open_today_journal(vault_path: String) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault.open_today_journal().map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de delete_page: remove arquivo .md do vault.
pub fn handle_delete_page(vault_path: String, page_path: String) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault.delete_page(&page_path).map_err(|e| e.to_string())
}

/// Handler de list_assets: lista arquivos em assets/.
pub fn handle_list_assets(vault_path: String) -> Result<Vec<String>, String> {
    let vault = VaultIo::open(&vault_path);
    vault.list_assets().map_err(|e| e.to_string())
}

/// Handler de copy_to_assets: copia arquivo para assets/.
pub fn handle_copy_to_assets(vault_path: String, source_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.copy_to_assets(&source_path).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_echo() {
        let r = handle_ping(PingArgs {
            message: "hello".to_string(),
        });
        assert_eq!(r.echo, "pong: hello");
        assert!(!r.version.is_empty());
    }
}
