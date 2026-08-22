//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::collections::HashMap;
use std::sync::Mutex;

use anotadinho_ipc::{
    handle_copy_to_assets, handle_create_folder, handle_create_page, handle_create_page_from_template,
    handle_read_asset_data_url,
    handle_create_page_in_folder, handle_create_page_typed, handle_delete_asset, handle_delete_page,
    handle_export_folder, handle_git_commit_and_push, handle_git_log, handle_git_pull, handle_git_status,
    handle_list_assets, handle_list_assets_info,
    handle_list_folders, handle_list_pages, handle_list_templates, handle_move_page,
    handle_open_today_journal, handle_ping, handle_read_page, handle_save_pasted_asset,
    handle_read_page_versioned, handle_scan_vault, handle_search_content, handle_write_page,
    handle_write_page_checked, VersionedPage,
    AssetInfo, PageMeta, PingArgs, PingResult, VaultInfo,
};
use anotadinho_core::PageIndexEntry;
use anotadinho_vault::{GitFileEntry, GitLogEntry, VaultIo, VaultWatcher};
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

/// Controles da janela (ciclo 180). Com `decorations: false` a barra do
/// sistema some, então minimizar/maximizar/fechar passam a ser botões do
/// próprio header do Anotadinho.
#[tauri::command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// Alterna maximizado/restaurado e devolve o estado NOVO — o botão
/// precisa saber pra trocar o ícone.
#[tauri::command]
fn window_toggle_maximize(window: tauri::Window) -> Result<bool, String> {
    let maximizada = window.is_maximized().map_err(|e| e.to_string())?;
    if maximizada {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(!maximizada)
}

#[tauri::command]
fn window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// Começa a redimensionar a janela pela borda indicada (ciclo 180).
///
/// Sem a decoração do sistema não existe borda de arraste no WM, então
/// as faixas invisíveis do `.window-resize` no frontend chamam isso no
/// `mousedown` e o próprio compositor assume o arraste dali em diante.
#[tauri::command]
fn window_start_resize(window: tauri::Window, direcao: String) -> Result<(), String> {
    use tauri_runtime::ResizeDirection as D;
    let direcao = match direcao.as_str() {
        "n" => D::North,
        "s" => D::South,
        "w" => D::West,
        "e" => D::East,
        "nw" => D::NorthWest,
        "ne" => D::NorthEast,
        "sw" => D::SouthWest,
        "se" => D::SouthEast,
        outro => return Err(format!("direção desconhecida: {outro}")),
    };
    window.start_resize_dragging(direcao).map_err(|e| e.to_string())
}

/// Estado inicial do botão de maximizar (a janela pode abrir já
/// maximizada pelo gerenciador de janelas).
#[tauri::command]
fn window_is_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
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

/// Varredura única do vault: metadados de todas as páginas numa
/// chamada só (ciclo 150), no lugar de `list_pages` + N `read_page`.
#[tauri::command]
fn scan_vault(vault_path: String) -> Result<Vec<PageIndexEntry>, String> {
    handle_scan_vault(vault_path)
}

#[tauri::command]
fn read_page(vault_path: String, page_path: String) -> Result<String, String> {
    handle_read_page(vault_path, page_path)
}

/// Leitura com marca de versão (ciclo 173) — o editor guarda a marca e
/// devolve ela ao salvar, pra escrita concorrente não passar batida.
#[tauri::command]
fn read_page_versioned(vault_path: String, page_path: String) -> Result<VersionedPage, String> {
    handle_read_page_versioned(vault_path, page_path)
}

/// Gravação condicionada à versão lida. Devolve a versão nova.
#[tauri::command]
fn write_page_checked(
    vault_path: String,
    page_path: String,
    content: String,
    expected_version: Option<String>,
) -> Result<String, String> {
    handle_write_page_checked(vault_path, page_path, content, expected_version)
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
fn git_status(vault_path: String) -> Option<Vec<GitFileEntry>> {
    handle_git_status(vault_path)
}

#[tauri::command]
fn git_log(vault_path: String, page_path: String) -> Option<Vec<GitLogEntry>> {
    handle_git_log(vault_path, page_path)
}

#[tauri::command]
fn git_pull(vault_path: String) -> Result<String, String> {
    handle_git_pull(vault_path)
}

#[tauri::command]
fn git_commit_and_push(vault_path: String, message: String) -> Result<String, String> {
    handle_git_commit_and_push(vault_path, message)
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
    folder_path: Option<String>,
) -> Result<PageMeta, String> {
    handle_create_page_from_template(vault_path, template_path, title, folder_path)
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
fn read_asset_data_url(vault_path: String, asset_path: String) -> Result<String, String> {
    handle_read_asset_data_url(vault_path, asset_path)
}

#[tauri::command]
fn save_pasted_asset(vault_path: String, extension: String, base64_data: String) -> Result<String, String> {
    handle_save_pasted_asset(vault_path, extension, base64_data)
}

#[tauri::command]
fn list_assets_info(vault_path: String) -> Result<Vec<AssetInfo>, String> {
    handle_list_assets_info(vault_path)
}

#[tauri::command]
fn delete_asset(vault_path: String, asset_path: String) -> Result<(), String> {
    handle_delete_asset(vault_path, asset_path)
}


/// Executa o agente configurado e devolve a saída (ciclo 202).
///
/// Deliberadamente SEM shell: `Command::new(binario).args(...)`, com o
/// prompt entrando como um argumento. Aspas, quebras de linha e
/// `$(...)` dentro do prompt são texto — não há interpretador no
/// caminho pra transformá-los em comando.
///
/// A configuração vem das preferências do app, nunca do conteúdo de uma
/// página. É a mesma invariante que mantém a lista de ações do embed
/// `actions` fechada.
///
/// Roda em `spawn_blocking` porque `std::process` é bloqueante e travaria
/// o runtime; o timeout mata o processo em vez de deixá-lo pendurado.
#[tauri::command]
async fn rodar_agente(
    adaptador: anotadinho_core::agente::Adaptador,
    prompt: String,
    vault_path: String,
) -> Result<String, String> {
    if let Some(problema) = adaptador.validar() {
        return Err(format!("configuração do agente inválida: {}", problema.mensagem()));
    }
    let args = adaptador.montar_args(&prompt);
    let cwd = if adaptador.cwd.trim().is_empty() { vault_path } else { adaptador.cwd.clone() };
    let binario = adaptador.binario.clone();
    let limite = adaptador.timeout_s;

    tokio::task::spawn_blocking(move || {
        use std::process::{Command, Stdio};

        let mut filho = Command::new(&binario)
            .args(&args)
            .current_dir(&cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("não consegui executar \"{binario}\": {e}"))?;

        // Espera com limite. Sem isso, um agente que trava deixa o
        // processo pendurado e a UI esperando pra sempre.
        let inicio = std::time::Instant::now();
        loop {
            match filho.try_wait() {
                Ok(Some(_)) => break,
                Ok(None) => {
                    if limite > 0 && inicio.elapsed().as_secs() >= limite {
                        let _ = filho.kill();
                        let _ = filho.wait();
                        return Err(format!("o agente passou de {limite}s e foi interrompido"));
                    }
                    std::thread::sleep(std::time::Duration::from_millis(120));
                }
                Err(e) => return Err(format!("erro esperando o agente: {e}")),
            }
        }

        let saida = filho
            .wait_with_output()
            .map_err(|e| format!("erro lendo a saída do agente: {e}"))?;
        let stdout = String::from_utf8_lossy(&saida.stdout).trim().to_string();
        if saida.status.success() {
            if stdout.is_empty() {
                return Err("o agente terminou sem escrever nada na saída".to_string());
            }
            Ok(stdout)
        } else {
            let stderr = String::from_utf8_lossy(&saida.stderr).trim().to_string();
            let detalhe = if stderr.is_empty() { stdout } else { stderr };
            Err(format!("o agente falhou: {detalhe}"))
        }
    })
    .await
    .map_err(|e| format!("tarefa do agente não completou: {e}"))?
}

#[tauri::command]
fn search_content(
    vault_path: String,
    query: String,
) -> Result<Vec<anotadinho_core::embed::SearchHit>, String> {
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
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
            window_start_resize,
            get_vault_info,
            list_pages,
            scan_vault,
            read_page,
            read_page_versioned,
            write_page,
            write_page_checked,
            create_page,
            create_page_with_type,
            open_today_journal,
            delete_page,
            create_folder,
            list_folders,
            move_page,
            create_page_in_folder,
            export_folder,
            git_status,
            git_log,
            git_pull,
            git_commit_and_push,
            list_templates,
            create_page_from_template,
            list_assets,
            copy_to_assets,
            read_asset_data_url,
            save_pasted_asset,
            list_assets_info,
            delete_asset,
            search_content,
            rodar_agente,
            check_changes,
            open_vault_dialog
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
