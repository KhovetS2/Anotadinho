//! File watcher: detecta mudanças no vault via `notify`.
//!
//! Mantém um `modified` flag que é setado quando qualquer arquivo `.md`
//! é criado, modificado ou deletado dentro do vault.

use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::sync::Arc;

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

/// Uma mudança observada no vault (ciclo 172) — o que o
/// `anotadinho-cli watch` emite pra um agente reagir.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct VaultEvent {
    /// Path relativo ao vault.
    pub path: String,
    /// `created` | `modified` | `deleted`.
    pub kind: String,
}

/// Wrapper thread-safe para monitorar mudanças no vault.
pub struct VaultWatcher {
    modified: Arc<AtomicBool>,
    /// Fila de eventos com path e tipo (ciclo 172). O `modified` acima
    /// continua existindo pro polling do app, que só precisa do "mudou
    /// alguma coisa" — os dois consumidores querem granularidades
    /// diferentes.
    eventos: Arc<Mutex<Vec<VaultEvent>>>,
    _watcher: RecommendedWatcher,
}

impl VaultWatcher {
    /// Inicia o watcher no diretório raiz do vault.
    pub fn start(root: PathBuf) -> Result<Self, anyhow::Error> {
        let modified = Arc::new(AtomicBool::new(false));
        let modified_clone = modified.clone();
        let eventos: Arc<Mutex<Vec<VaultEvent>>> = Arc::new(Mutex::new(Vec::new()));
        let eventos_clone = eventos.clone();
        let raiz_para_evento = root.canonicalize()?;

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
                                let tipo = match event.kind {
                                    EventKind::Create(_) => "created",
                                    EventKind::Remove(_) => "deleted",
                                    _ => "modified",
                                };
                                if let Ok(mut fila) = eventos_clone.lock() {
                                    for p in &event.paths {
                                        if p.extension().map_or(true, |e| e != "md") {
                                            continue;
                                        }
                                        let relativo = p
                                            .strip_prefix(&raiz_para_evento)
                                            .unwrap_or(p)
                                            .to_string_lossy()
                                            .to_string();
                                        fila.push(VaultEvent {
                                            path: relativo,
                                            kind: tipo.to_string(),
                                        });
                                    }
                                }
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
            eventos,
            _watcher: watcher,
        })
    }

    /// Retorna `true` se houve mudanças desde a última verificação,
    /// e reseta o flag.
    pub fn has_changes(&self) -> bool {
        self.modified.swap(false, Ordering::SeqCst)
    }

    /// Esvazia a fila de eventos acumulados desde a última chamada
    /// (ciclo 172).
    ///
    /// Faz o "debounce" que o chamador esperaria: salvar um arquivo uma
    /// vez costuma gerar mais de um evento do sistema (gravar +
    /// renomear, dependendo da plataforma), então eventos repetidos do
    /// MESMO path e tipo, no mesmo lote, viram um só.
    pub fn drain_events(&self) -> Vec<VaultEvent> {
        let Ok(mut fila) = self.eventos.lock() else { return Vec::new() };
        let mut vistos: Vec<VaultEvent> = Vec::new();
        for e in fila.drain(..) {
            if !vistos.contains(&e) {
                vistos.push(e);
            }
        }
        vistos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Espera até `cond` virar true ou estourar — o `notify` entrega o
    /// evento em outra thread, então dormir um tempo fixo deixaria o
    /// teste instável.
    fn esperar(watcher: &VaultWatcher, limite_ms: u64) -> Vec<VaultEvent> {
        let inicio = std::time::Instant::now();
        loop {
            let eventos = watcher.drain_events();
            if !eventos.is_empty() || inicio.elapsed().as_millis() as u64 > limite_ms {
                return eventos;
            }
            std::thread::sleep(std::time::Duration::from_millis(50));
        }
    }

    #[test]
    fn drain_events_devolve_path_e_tipo() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        let watcher = VaultWatcher::start(dir.path().to_path_buf()).unwrap();

        std::fs::write(dir.path().join("pages/nova.md"), "conteúdo\n").unwrap();
        let eventos = esperar(&watcher, 3000);
        assert!(!eventos.is_empty(), "nenhum evento chegou");
        assert!(
            eventos.iter().any(|e| e.path == "pages/nova.md"),
            "path veio errado: {eventos:?}"
        );
        assert!(
            eventos.iter().all(|e| ["created", "modified", "deleted"].contains(&e.kind.as_str())),
            "tipo inesperado: {eventos:?}"
        );
    }

    #[test]
    fn drain_events_esvazia_a_fila() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        let watcher = VaultWatcher::start(dir.path().to_path_buf()).unwrap();
        std::fs::write(dir.path().join("pages/x.md"), "a\n").unwrap();
        let _ = esperar(&watcher, 3000);
        assert!(watcher.drain_events().is_empty(), "a fila devia ter sido esvaziada");
    }

    #[test]
    fn arquivo_que_nao_e_md_nao_gera_evento() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        let watcher = VaultWatcher::start(dir.path().to_path_buf()).unwrap();
        std::fs::write(dir.path().join("assets/x.png"), b"bin").unwrap();
        std::thread::sleep(std::time::Duration::from_millis(600));
        assert!(watcher.drain_events().is_empty());
    }
}

