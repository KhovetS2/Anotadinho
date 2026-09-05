//! Anotadinho vault: I/O de arquivos, watcher, locks.
//!
//! Este crate gerencia o filesystem do vault: listar, ler, escrever
//! páginas Markdown, watcher de mudanças e locks entre instâncias.

#![warn(missing_docs)]

pub mod caminho;
pub mod git_status;
pub mod git_sync;
pub mod index_cache;
pub mod io;
pub mod watcher;

pub use caminho::{normalizar, relativo};
pub use git_status::{git_log, git_status, GitFileEntry, GitLogEntry};
pub use git_sync::{git_commit_and_push, git_pull};
pub use index_cache::IndexCache;
pub use io::{VaultIo, CONFLICT_PREFIX};
pub use watcher::{VaultEvent, VaultWatcher};

/// Monta um `Command` sem herdar console no Windows.
///
/// O app é `windows_subsystem = "windows"` (sem console próprio). Sem
/// essa flag, CADA `git status` do polling de 3s da UI (ciclo 103) faz
/// o Windows abrir e fechar uma janela de console pro processo filho —
/// visível como piscar contínuo enquanto o app está aberto.
pub(crate) fn comando(programa: &str) -> std::process::Command {
    #[cfg_attr(not(windows), allow(unused_mut))]
    let mut cmd = std::process::Command::new(programa);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}
