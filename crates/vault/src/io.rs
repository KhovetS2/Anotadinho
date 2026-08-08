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

    /// Remove uma página do vault (arquivo `.md`).
    pub fn delete_page(&self, relative_path: &str) -> Result<()> {
        let full = self.resolve_safe(relative_path)?;
        if full.extension().map_or(true, |e| e != "md") {
            anyhow::bail!("só é permitido excluir arquivos .md");
        }
        std::fs::remove_file(&full)
            .map_err(|e| anyhow::anyhow!("erro ao excluir {}: {}", relative_path, e))?;
        Ok(())
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

    /// Abre ou cria o journal do dia (`journals/YYYY-MM-DD.md`).
    pub fn open_today_journal(&self) -> Result<PageMeta> {
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let relative = format!("journals/{}.md", today);
        let full = self.root.join(&relative);
        if !full.exists() {
            let content = format!(
                "---\ntitle: {}\n---\n\n- \n",
                today
            );
            self.write_page(&relative, &content)?;
        }
        Ok(PageMeta {
            path: relative,
            title: today,
            section: "journals".to_string(),
        })
    }

    /// Cria uma nova página em `pages/` com frontmatter básico.
    ///
    /// Retorna metadados da página criada. Gera slug único se colidir.
    pub fn create_page(&self, title: &str) -> Result<PageMeta> {
        self.create_page_with_type(title, "md")
    }

    /// Cria nova página com tipo específico (md, kanban, calendar, table).
    pub fn create_page_with_type(&self, title: &str, page_type: &str) -> Result<PageMeta> {
        self.create_page_in(None, title, page_type)
    }

    /// Cria nova página dentro de uma pasta (`folder_relative` é o path
    /// relativo ao vault, ex: `"pages/trabalho"`).
    pub fn create_page_in_folder(
        &self,
        folder_relative: &str,
        title: &str,
        page_type: &str,
    ) -> Result<PageMeta> {
        self.create_page_in(Some(folder_relative), title, page_type)
    }

    fn create_page_in(
        &self,
        folder_relative: Option<&str>,
        title: &str,
        page_type: &str,
    ) -> Result<PageMeta> {
        let base_slug = slugify(title);
        let mut slug = base_slug.clone();
        let mut n = 2u32;
        let dir_prefix = match folder_relative {
            Some(f) => format!("{}/", f.trim_end_matches('/')),
            None => "pages/".to_string(),
        };
        loop {
            let relative = format!("{}{}.md", dir_prefix, slug);
            let full = self.root.join(&relative);
            if !full.exists() {
                let type_line = if page_type != "md" {
                    format!("type: {}\n", page_type)
                } else {
                    String::new()
                };
                let content = format!(
                    "---\ntitle: {}\n{}---\n\n- \n",
                    title.replace(':', " -"),
                    type_line
                );
                self.write_page(&relative, &content)?;
                return Ok(PageMeta {
                    path: relative,
                    title: slug,
                    section: "pages".to_string(),
                });
            }
            slug = format!("{}-{}", base_slug, n);
            n += 1;
            if n > 1000 {
                anyhow::bail!("não foi possível gerar slug único para {}", title);
            }
        }
    }

    /// Cria uma pasta (subdiretório) dentro do vault, geralmente sob
    /// `pages/`. Idempotente — não falha se a pasta já existir.
    pub fn create_folder(&self, relative_path: &str) -> Result<()> {
        validate_relative_path(relative_path)?;
        let full = self.root.join(relative_path);
        if full.is_file() {
            anyhow::bail!("já existe um arquivo com esse nome: {}", relative_path);
        }
        std::fs::create_dir_all(&full)
            .map_err(|e| anyhow::anyhow!("erro ao criar pasta {}: {}", relative_path, e))?;
        Ok(())
    }

    /// Lista todas as pastas (subdiretórios) sob `pages/`, incluindo
    /// pastas vazias — necessário porque `list_pages` só enxerga
    /// arquivos, então uma pasta recém-criada sem páginas dentro
    /// desapareceria da árvore sem isso.
    pub fn list_folders(&self) -> Result<Vec<String>> {
        let dir = self.root.join("pages");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut folders = Vec::new();
        for entry in WalkDir::new(&dir)
            .max_depth(3)
            .min_depth(1)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.path().is_dir() {
                let relative = entry
                    .path()
                    .strip_prefix(&self.root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                folders.push(relative);
            }
        }
        folders.sort();
        Ok(folders)
    }

    /// Move (renomeia) uma página pra um novo path relativo — usado pra
    /// organizar páginas em pastas. Recusa se o destino já existir.
    pub fn move_page(&self, from_relative: &str, to_relative: &str) -> Result<PageMeta> {
        let from_full = self.resolve_safe(from_relative)?;
        validate_relative_path(to_relative)?;
        let to_full = self.root.join(to_relative);
        if to_full.exists() {
            anyhow::bail!("já existe um arquivo em {}", to_relative);
        }
        if let Some(parent) = to_full.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("erro ao criar dirs: {}", e))?;
        }
        std::fs::rename(&from_full, &to_full).map_err(|e| {
            anyhow::anyhow!("erro ao mover {} -> {}: {}", from_relative, to_relative, e)
        })?;
        let section = if to_relative.starts_with("journals/") {
            "journals"
        } else {
            "pages"
        };
        let title = Path::new(to_relative)
            .file_stem()
            .map(|s| s.to_string_lossy().to_string())
            .unwrap_or_default();
        Ok(PageMeta {
            path: to_relative.to_string(),
            title,
            section: section.to_string(),
        })
    }

    /// Lista arquivos no diretório `assets/` do vault.
    pub fn list_assets(&self) -> Result<Vec<String>> {
        let dir = self.root.join("assets");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut files = Vec::new();
        for entry in WalkDir::new(&dir).max_depth(3).into_iter().filter_map(|e| e.ok()) {
            if entry.path().is_file() {
                let relative = entry.path()
                    .strip_prefix(&self.root)
                    .unwrap_or(entry.path())
                    .to_string_lossy()
                    .to_string();
                files.push(relative);
            }
        }
        files.sort();
        Ok(files)
    }

    /// Copia um arquivo externo para `assets/` e retorna o path relativo.
    pub fn copy_to_assets(&self, source_path: &str) -> Result<String> {
        let src = std::path::Path::new(source_path);
        if !src.is_file() {
            anyhow::bail!("arquivo fonte não existe: {}", source_path);
        }
        let file_name = src.file_name()
            .ok_or_else(|| anyhow::anyhow!("nome de arquivo inválido"))?
            .to_string_lossy()
            .to_string();
        let dest_dir = self.root.join("assets");
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| anyhow::anyhow!("erro ao criar assets/: {}", e))?;
        let dest = dest_dir.join(&file_name);
        std::fs::copy(src, &dest)
            .map_err(|e| anyhow::anyhow!("erro ao copiar {}: {}", source_path, e))?;
        let relative = dest
            .strip_prefix(&self.root)
            .unwrap_or(&dest)
            .to_string_lossy()
            .to_string();
        Ok(relative)
    }

    /// Busca texto no conteúdo de todas as páginas.
    /// Retorna pares (page_path, excerpt com 50 chars ao redor do match).
    pub fn search_content(&self, query: &str) -> Result<Vec<(String, String)>> {
        let pages = self.list_pages()?;
        let mut results = Vec::new();
        let q = query.to_lowercase();
        for page in &pages {
            let content = self.read_page(&page.path).unwrap_or_default();
            let lower = content.to_lowercase();
            if let Some(pos) = lower.find(&q) {
                let start = pos.saturating_sub(20);
                let end = (pos + q.len() + 30).min(content.len());
                let excerpt = content[start..end].replace('\n', " ");
                results.push((page.path.clone(), format!("...{}...", excerpt)));
            }
        }
        Ok(results)
    }

    /// Escreve conteúdo UTF-8 numa página pelo path relativo ao vault.
    ///
    /// Cria diretórios pais se necessário. Rejeita path traversal.
    pub fn write_page(&self, relative_path: &str, content: &str) -> Result<()> {
        let full = self.resolve_safe_for_write(relative_path)?;
        if let Some(parent) = full.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("erro ao criar dirs: {}", e))?;
        }
        std::fs::write(&full, content)
            .map_err(|e| anyhow::anyhow!("erro ao escrever {}: {}", relative_path, e))?;
        Ok(())
    }

    /// Resolve path relativo garantindo que fica dentro do vault (arquivo deve existir).
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

    /// Resolve path para escrita: valida que o path normalizado fica no vault
    /// mesmo se o arquivo ainda não existir.
    fn resolve_safe_for_write(&self, relative_path: &str) -> Result<PathBuf> {
        validate_relative_path(relative_path)?;
        let joined = self.root.join(relative_path);
        let root_canonical = self
            .root
            .canonicalize()
            .map_err(|e| anyhow::anyhow!("vault root inválido: {}", e))?;
        // Normaliza sem exigir que o arquivo exista
        let parent = joined.parent().unwrap_or(&self.root);
        let parent_canonical = if parent.exists() {
            parent
                .canonicalize()
                .map_err(|e| anyhow::anyhow!("parent inválido: {}", e))?
        } else {
            // Cria parent e canonicaliza
            std::fs::create_dir_all(parent)
                .map_err(|e| anyhow::anyhow!("erro ao criar parent: {}", e))?;
            parent
                .canonicalize()
                .map_err(|e| anyhow::anyhow!("parent inválido: {}", e))?
        };
        if !parent_canonical.starts_with(&root_canonical) {
            anyhow::bail!("path fora do vault: {}", relative_path);
        }
        let file_name = joined
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("path sem nome de arquivo"))?;
        Ok(parent_canonical.join(file_name))
    }
}

/// Valida que um path relativo não escapa do vault (sem `..`, vazio ou
/// com byte nulo) — compartilhado entre escrita, criação de pasta e
/// move.
fn validate_relative_path(relative_path: &str) -> Result<()> {
    if relative_path.is_empty()
        || relative_path.contains('\0')
        || Path::new(relative_path)
            .components()
            .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        anyhow::bail!("path inválido: {}", relative_path);
    }
    Ok(())
}

/// Converte título em slug de arquivo seguro.
fn slugify(title: &str) -> String {
    let s: String = title
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else if c.is_whitespace() || c == '-' || c == '_' {
                '-'
            } else {
                '\0'
            }
        })
        .filter(|c| *c != '\0')
        .collect();
    let s = s
        .split('-')
        .filter(|p| !p.is_empty())
        .collect::<Vec<_>>()
        .join("-");
    if s.is_empty() {
        "untitled".to_string()
    } else {
        s
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

    #[test]
    fn write_page_overwrites_content() {
        let (_dir, io) = setup_vault();
        io.write_page("pages/alpha.md", "# Novo Alpha\n").unwrap();
        let content = io.read_page("pages/alpha.md").unwrap();
        assert_eq!(content, "# Novo Alpha\n");
    }

    #[test]
    fn write_page_creates_new_file() {
        let (_dir, io) = setup_vault();
        io.write_page("pages/nova.md", "conteudo novo\n").unwrap();
        let content = io.read_page("pages/nova.md").unwrap();
        assert_eq!(content, "conteudo novo\n");
    }

    #[test]
    fn write_page_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.write_page("../escape.md", "x").is_err());
    }

    #[test]
    fn create_page_writes_file() {
        let (_dir, io) = setup_vault();
        let meta = io.create_page("Minha Nota").unwrap();
        assert_eq!(meta.path, "pages/minha-nota.md");
        assert_eq!(meta.section, "pages");
        let content = io.read_page(&meta.path).unwrap();
        assert!(content.contains("title: Minha Nota"));
    }

    #[test]
    fn create_page_unique_slug_on_collision() {
        let (_dir, io) = setup_vault();
        let a = io.create_page("Dup").unwrap();
        let b = io.create_page("Dup").unwrap();
        assert_eq!(a.path, "pages/dup.md");
        assert_eq!(b.path, "pages/dup-2.md");
    }

    #[test]
    fn slugify_basic() {
        assert_eq!(slugify("Hello World"), "hello-world");
        assert_eq!(slugify("  "), "untitled");
    }

    #[test]
    fn delete_page_removes_file() {
        let (_dir, io) = setup_vault();
        assert!(io.read_page("pages/alpha.md").is_ok());
        io.delete_page("pages/alpha.md").unwrap();
        assert!(io.read_page("pages/alpha.md").is_err());
    }

    #[test]
    fn delete_page_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.delete_page("../secret.md").is_err());
    }

    #[test]
    fn create_folder_makes_empty_dir() {
        let (_dir, io) = setup_vault();
        io.create_folder("pages/trabalho").unwrap();
        assert!(io.list_folders().unwrap().contains(&"pages/trabalho".to_string()));
    }

    #[test]
    fn create_folder_is_idempotent() {
        let (_dir, io) = setup_vault();
        io.create_folder("pages/trabalho").unwrap();
        assert!(io.create_folder("pages/trabalho").is_ok());
    }

    #[test]
    fn create_folder_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.create_folder("../escape").is_err());
    }

    #[test]
    fn create_folder_rejects_existing_file() {
        let (_dir, io) = setup_vault();
        assert!(io.create_folder("pages/alpha.md").is_err());
    }

    #[test]
    fn list_folders_includes_folders_with_and_without_pages() {
        let (_dir, io) = setup_vault();
        io.create_folder("pages/vazia").unwrap();
        io.write_page("pages/cheia/nota.md", "conteudo\n").unwrap();
        let folders = io.list_folders().unwrap();
        assert!(folders.contains(&"pages/vazia".to_string()));
        assert!(folders.contains(&"pages/cheia".to_string()));
    }

    #[test]
    fn list_pages_finds_files_inside_folders() {
        let (_dir, io) = setup_vault();
        io.write_page("pages/trabalho/tarefa.md", "# Tarefa\n").unwrap();
        let pages = io.list_pages().unwrap();
        let found = pages.iter().find(|p| p.path == "pages/trabalho/tarefa.md");
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "tarefa");
    }

    #[test]
    fn create_page_in_folder_writes_nested_file() {
        let (_dir, io) = setup_vault();
        let meta = io.create_page_in_folder("pages/trabalho", "Minha Tarefa", "md").unwrap();
        assert_eq!(meta.path, "pages/trabalho/minha-tarefa.md");
        assert!(io.read_page(&meta.path).is_ok());
    }

    #[test]
    fn move_page_renames_file() {
        let (_dir, io) = setup_vault();
        let meta = io.move_page("pages/alpha.md", "pages/trabalho/alpha.md").unwrap();
        assert_eq!(meta.path, "pages/trabalho/alpha.md");
        assert!(io.read_page("pages/trabalho/alpha.md").is_ok());
        assert!(io.read_page("pages/alpha.md").is_err());
    }

    #[test]
    fn move_page_rejects_existing_destination() {
        let (_dir, io) = setup_vault();
        assert!(io.move_page("pages/alpha.md", "pages/beta.md").is_err());
    }

    #[test]
    fn move_page_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.move_page("pages/alpha.md", "../escape.md").is_err());
    }

    #[test]
    fn open_today_journal_creates_file() {
        let (_dir, io) = setup_vault();
        let meta = io.open_today_journal().unwrap();
        assert_eq!(meta.section, "journals");
        assert!(meta.path.starts_with("journals/"));
        assert!(meta.path.ends_with(".md"));
        let content = io.read_page(&meta.path).unwrap();
        assert!(content.contains("title:"));
        // second call returns same file
        let meta2 = io.open_today_journal().unwrap();
        assert_eq!(meta.path, meta2.path);
    }
}
