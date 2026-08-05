//! Anotadinho vault: I/O de arquivos, watcher, locks.
//!
//! Este crate gerencia o filesystem do vault: listar, ler, escrever
//! páginas Markdown, watcher de mudanças e locks entre instâncias.

#![warn(missing_docs)]

pub mod io;
pub mod watcher;

pub use io::VaultIo;
pub use watcher::VaultWatcher;
