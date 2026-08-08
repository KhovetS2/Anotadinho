//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

use anotadinho_ipc::{
    handle_copy_to_assets, handle_create_folder, handle_create_page, handle_create_page_from_template,
    handle_create_page_in_folder, handle_create_page_typed, handle_delete_asset, handle_delete_page,
    handle_export_folder, handle_list_assets, handle_list_assets_info, handle_list_folders,
    handle_list_pages, handle_list_templates, handle_move_page, handle_open_today_journal, handle_ping,
    handle_read_page, handle_search_content, handle_write_page, AssetInfo, PageMeta, PingArgs,
    PingResult, VaultInfo,
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
fn create_page_with_type(
    vault_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    handle_create_page_typed(vault_path, title, page_type)
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
fn create_folder(vault_path: String, folder_path: String) -> Result<(), String> {
    handle_create_folder(vault_path, folder_path)
}

#[tauri::command]
fn list_folders(vault_path: String) -> Result<Vec<String>, String> {
    handle_list_folders(vault_path)
}

#[tauri::command]
fn move_page(vault_path: String, from_path: String, to_path: String) -> Result<PageMeta, String> {
    handle_move_page(vault_path, from_path, to_path)
}

#[tauri::command]
fn create_page_in_folder(
    vault_path: String,
    folder_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    handle_create_page_in_folder(vault_path, folder_path, title, page_type)
}

#[tauri::command]
fn export_folder(vault_path: String, folder_path: String) -> Result<String, String> {
    handle_export_folder(vault_path, folder_path)
}

#[tauri::command]
fn list_templates(vault_path: String) -> Result<Vec<PageMeta>, String> {
    handle_list_templates(vault_path)
}

#[tauri::command]
fn create_page_from_template(
    vault_path: String,
    template_path: String,
    title: String,
) -> Result<PageMeta, String> {
    handle_create_page_from_template(vault_path, template_path, title)
}

#[tauri::command]
fn list_assets(vault_path: String) -> Result<Vec<String>, String> {
    handle_list_assets(vault_path)
}

#[tauri::command]
fn copy_to_assets(vault_path: String, source_path: String) -> Result<String, String> {
    handle_copy_to_assets(vault_path, source_path)
}

#[tauri::command]
fn list_assets_info(vault_path: String) -> Result<Vec<AssetInfo>, String> {
    handle_list_assets_info(vault_path)
}

#[tauri::command]
fn delete_asset(vault_path: String, asset_path: String) -> Result<(), String> {
    handle_delete_asset(vault_path, asset_path)
}

#[tauri::command]
fn search_content(vault_path: String, query: String) -> Result<Vec<(String, String)>, String> {
    handle_search_content(vault_path, query)
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
        .plugin(tauri_plugin_mcp_bridge::init())
        .manage(AppWatchers(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            ping,
            get_vault_info,
            list_pages,
            read_page,
            write_page,
            create_page,
            create_page_with_type,
            open_today_journal,
            delete_page,
            create_folder,
            list_folders,
            move_page,
            create_page_in_folder,
            export_folder,
            list_templates,
            create_page_from_template,
            list_assets,
            copy_to_assets,
            list_assets_info,
            delete_asset,
            search_content,
            check_changes,
            open_vault_dialog
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
