//! Anotadinho vault: I/O de arquivos, watcher, locks.
//!
//! Este crate gerencia o filesystem do vault: listar, ler, escrever
//! páginas Markdown, watcher de mudanças e locks entre instâncias.

#![warn(missing_docs)]

pub mod git_status;
pub mod git_sync;
pub mod io;
pub mod watcher;

pub use git_status::{git_log, git_status, GitFileEntry, GitLogEntry};
pub use git_sync::{git_commit_and_push, git_pull};
pub use io::VaultIo;
pub use watcher::VaultWatcher;
