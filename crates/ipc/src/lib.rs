//! Anotadinho IPC: comandos Tauri expostos pro Yew frontend.
//!
//! Os comandos IPC são a única ponte entre o Yew (no WebView) e o
//! backend Rust (Tauri core). Tudo que o UI faz passa por aqui.

#![warn(missing_docs)]

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
