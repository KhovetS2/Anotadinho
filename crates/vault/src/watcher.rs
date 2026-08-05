//! File watcher: detecta mudanças no vault via `notify`.
//!
//! Mantém um `modified` flag que é setado quando qualquer arquivo `.md`
//! é criado, modificado ou deletado dentro do vault.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Wrapper thread-safe para monitorar mudanças no vault.
pub struct VaultWatcher {
    modified: Arc<AtomicBool>,
    _watcher: RecommendedWatcher,
}

impl VaultWatcher {
    /// Inicia o watcher no diretório raiz do vault.
    pub fn start(root: PathBuf) -> Result<Self, anyhow::Error> {
        let modified = Arc::new(AtomicBool::new(false));
        let modified_clone = modified.clone();

        let root_canonical = root.canonicalize()?;

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<Event, notify::Error>| {
                if let Ok(event) = res {
                    match event.kind {
                        EventKind::Create(_)
                        | EventKind::Modify(_)
                        | EventKind::Remove(_) => {
                            let has_md = event
                                .paths
                                .iter()
                                .any(|p| p.extension().map_or(false, |e| e == "md"));
                            if has_md {
                                modified_clone.store(true, Ordering::SeqCst);
                            }
                        }
                        _ => {}
                    }
                }
            },
            Config::default(),
        )?;

        watcher.watch(&root_canonical, RecursiveMode::Recursive)?;

        Ok(Self {
            modified,
            _watcher: watcher,
        })
    }

    /// Retorna `true` se houve mudanças desde a última verificação,
    /// e reseta o flag.
    pub fn has_changes(&self) -> bool {
        self.modified.swap(false, Ordering::SeqCst)
    }
}
