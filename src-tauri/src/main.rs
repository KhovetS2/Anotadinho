//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use anotadinho_ipc::{handle_ping, PingArgs, PingResult};

#[tauri::command]
fn ping(args: PingArgs) -> PingResult {
    handle_ping(args)
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![ping])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
