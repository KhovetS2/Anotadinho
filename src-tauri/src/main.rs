//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

use anotadinho_ipc::{
    handle_create_page, handle_delete_page, handle_list_pages, handle_open_today_journal,
    handle_ping, handle_read_page, handle_write_page, PageMeta, PingArgs, PingResult, VaultInfo,
};
use anotadinho_vault::{VaultIo, VaultWatcher};
use tauri_plugin_dialog::DialogExt;

struct AppWatchers(Mutex<HashMap<String, VaultWatcher>>);

#[tauri::command]
fn check_changes(
    vault_path: String,
    state: tauri::State<'_, AppWatchers>,
) -> Result<bool, String> {
    let mut map = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(watcher) = map.get_mut(&vault_path) {
        return Ok(watcher.has_changes());
    }
    let watcher = VaultWatcher::start(vault_path.clone().into()).map_err(|e| e.to_string())?;
    let changed = watcher.has_changes();
    let _ = map.insert(vault_path, watcher);
    Ok(changed)
}

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
fn delete_page(vault_path: String, page_path: String) -> Result<(), String> {
    handle_delete_page(vault_path, page_path)
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
        .manage(AppWatchers(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            ping,
            get_vault_info,
            list_pages,
            read_page,
            write_page,
            create_page,
            open_today_journal,
            delete_page,
            check_changes,
            open_vault_dialog
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
