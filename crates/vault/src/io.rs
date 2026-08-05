//! I/O do vault: listar, ler e escrever páginas.
//!
//! Implementa operações de filesystem sobre o diretório do vault.
//! O vault é uma pasta que contém `pages/`, `journals/`, `assets/`
//! e um diretório oculto `.anotadinho/` para metadados.

use std::path::{Path, PathBuf};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

/// Metadados de uma página listada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    /// Path relativo ao vault.
    pub path: String,
    /// Nome do arquivo (sem extensão).
    pub title: String,
    /// Se é `pages/` ou `journals/`.
    pub section: String,
}

/// Interface de I/O do vault.
pub struct VaultIo {
    /// Path absoluto do diretório raiz do vault.
    root: PathBuf,
}

impl VaultIo {
    /// Abre um vault no path informado.
    pub fn open(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Caminho absoluto da raiz do vault.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Lista todas as páginas `.md` em `pages/` e `journals/`.
    ///
    /// Retorna metadados ordenados alfabeticamente por título.
    pub fn list_pages(&self) -> Result<Vec<PageMeta>> {
        let mut pages = Vec::new();

        let sections = ["pages", "journals"];
        for section in &sections {
            let dir = self.root.join(section);
            if !dir.is_dir() {
                continue;
            }
            for entry in WalkDir::new(&dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path();
                if path.extension().map_or(false, |ext| ext == "md") {
                    let relative = path
                        .strip_prefix(&self.root)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    let title = path
                        .file_stem()
                        .map(|s| s.to_string_lossy().to_string())
                        .unwrap_or_default();
                    pages.push(PageMeta {
                        path: relative,
                        title,
                        section: section.to_string(),
                    });
                }
            }
        }

        pages.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(pages)
    }

    /// Lê o conteúdo UTF-8 de uma página pelo path relativo ao vault.
    ///
    /// Rejeita paths que escapem da raiz do vault (`..`).
    pub fn read_page(&self, relative_path: &str) -> Result<String> {
        let full = self.resolve_safe(relative_path)?;
        let content = std::fs::read_to_string(&full)
            .map_err(|e| anyhow::anyhow!("erro ao ler {}: {}", relative_path, e))?;
        Ok(content)
    }

    /// Resolve path relativo garantindo que fica dentro do vault.
    fn resolve_safe(&self, relative_path: &str) -> Result<PathBuf> {
        let joined = self.root.join(relative_path);
        let canonical = joined
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("path inválido {}: {}", relative_path, e))?;
        let root_canonical = self
            .root
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("vault root inválido: {}", e))?;
        if !canonical.starts_with(&root_canonical) {
            anyhow::bail!("path fora do vault: {}", relative_path);
        }
        Ok(canonical)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn setup_vault() -> (TempDir, VaultIo) {
        let dir = TempDir::new().expect("cria temp dir");
        fs::create_dir_all(dir.path().join("pages")).unwrap();
        fs::create_dir_all(dir.path().join("journals")).unwrap();

        fs::write(dir.path().join("pages/alpha.md"), "# Alpha\n").unwrap();
        fs::write(dir.path().join("pages/beta.md"), "# Beta\n").unwrap();
        fs::write(dir.path().join("pages/gamma.md"), "# Gamma\n").unwrap();
        fs::write(dir.path().join("journals/2026-01-01.md"), "# Journal\n").unwrap();
        fs::write(dir.path().join("pages/nota.txt"), "ignorado").unwrap();

        let io = VaultIo::open(dir.path().to_path_buf());
        (dir, io)
    }

    #[test]
    fn list_pages_returns_md_only() {
        let (_dir, io) = setup_vault();
        let pages = io.list_pages().unwrap();
        assert_eq!(pages.len(), 4);
        for p in &pages {
            assert!(p.path.ends_with(".md"));
        }
    }

    #[test]
    fn list_pages_sorted_by_title() {
        let (_dir, io) = setup_vault();
        let pages = io.list_pages().unwrap();
        let titles: Vec<&str> = pages.iter().map(|p| p.title.as_str()).collect();
        assert_eq!(titles, vec!["2026-01-01", "alpha", "beta", "gamma"]);
    }

    #[test]
    fn list_pages_has_sections() {
        let (_dir, io) = setup_vault();
        let pages = io.list_pages().unwrap();
        let page_sections: Vec<&str> = pages
            .iter()
            .filter(|p| p.section == "pages")
            .map(|p| p.title.as_str())
            .collect();
        assert_eq!(page_sections.len(), 3);
        let journal_sections: Vec<&str> = pages
            .iter()
            .filter(|p| p.section == "journals")
            .map(|p| p.title.as_str())
            .collect();
        assert_eq!(journal_sections.len(), 1);
    }

    #[test]
    fn open_returns_root() {
        let (_dir, io) = setup_vault();
        assert!(io.root().is_dir());
    }

    #[test]
    fn read_page_returns_content() {
        let (_dir, io) = setup_vault();
        let content = io.read_page("pages/alpha.md").unwrap();
        assert_eq!(content, "# Alpha\n");
    }

    #[test]
    fn read_page_missing_returns_err() {
        let (_dir, io) = setup_vault();
        assert!(io.read_page("pages/nao-existe.md").is_err());
    }

    #[test]
    fn read_page_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.read_page("../etc/passwd").is_err());
    }
}
