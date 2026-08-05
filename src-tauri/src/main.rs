//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anotadinho_ipc::{
    handle_create_page, handle_list_pages, handle_open_today_journal, handle_ping,
    handle_read_page, handle_write_page, PageMeta, PingArgs, PingResult, VaultInfo,
};
use anotadinho_vault::VaultIo;
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
fn ping(args: PingArgs) -> PingResult {
    handle_ping(args)
}

#[tauri::command]
fn get_vault_info(path: String) -> Result<VaultInfo, String> {
    let vault = VaultIo::open(&path);
    let name = vault
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string());
    Ok(VaultInfo {
        path: path.to_string(),
        name,
    })
}

#[tauri::command]
fn list_pages(vault_path: String) -> Result<Vec<PageMeta>, String> {
    handle_list_pages(vault_path)
}

#[tauri::command]
fn read_page(vault_path: String, page_path: String) -> Result<String, String> {
    handle_read_page(vault_path, page_path)
}

#[tauri::command]
fn write_page(vault_path: String, page_path: String, content: String) -> Result<(), String> {
    handle_write_page(vault_path, page_path, content)
}

#[tauri::command]
fn create_page(vault_path: String, title: String) -> Result<PageMeta, String> {
    handle_create_page(vault_path, title)
}

#[tauri::command]
fn open_today_journal(vault_path: String) -> Result<PageMeta, String> {
    handle_open_today_journal(vault_path)
}

#[tauri::command]
async fn open_vault_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .pick_folder(move |file_path| {
            let _ = tx.send(file_path.map(|p| p.to_string()));
        });
    rx.await.map_err(|e| e.to_string())
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            ping,
            get_vault_info,
            list_pages,
            read_page,
            write_page,
            create_page,
            open_today_journal,
            open_vault_dialog
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
