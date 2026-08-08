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

/// Metadados de um arquivo em `assets/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Path relativo ao vault.
    pub path: String,
    /// Tamanho em bytes.
    pub size: u64,
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
        let dir_prefix = match folder_relative {
            Some(f) => format!("{}/", f.trim_end_matches('/')),
            None => "pages/".to_string(),
        };
        let (slug, relative) = self.find_unique_relative_path(&dir_prefix, title)?;
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
        Ok(PageMeta {
            path: relative,
            title: slug,
            section: "pages".to_string(),
        })
    }

    /// Encontra um path relativo único sob `dir_prefix` a partir do
    /// slug de `title`, gerando `-2`, `-3`, ... em caso de colisão.
    /// Compartilhado entre `create_page_in` e `create_page_from_template`.
    fn find_unique_relative_path(&self, dir_prefix: &str, title: &str) -> Result<(String, String)> {
        let base_slug = slugify(title);
        let mut slug = base_slug.clone();
        let mut n = 2u32;
        loop {
            let relative = format!("{}{}.md", dir_prefix, slug);
            let full = self.root.join(&relative);
            if !full.exists() {
                return Ok((slug, relative));
            }
            slug = format!("{}-{}", base_slug, n);
            n += 1;
            if n > 1000 {
                anyhow::bail!("não foi possível gerar slug único para {}", title);
            }
        }
    }

    /// Concatena o markdown fonte de todas as páginas dentro de
    /// `folder_relative` (recursivo, via `list_pages` filtrado por
    /// prefixo de path) num dump único, separado por um cabeçalho com
    /// o título de cada página. `folder_relative` vazio ou `"pages"`/
    /// `"journals"` exporta a seção inteira; `None` (via
    /// `export_vault`) exporta tudo.
    pub fn export_folder(&self, folder_relative: &str) -> Result<String> {
        let pages = self.list_pages()?;
        let prefix = format!("{}/", folder_relative.trim_end_matches('/'));
        let mut matching: Vec<_> = pages
            .into_iter()
            .filter(|p| folder_relative.is_empty() || p.path.starts_with(&prefix))
            .collect();
        matching.sort_by(|a, b| a.path.cmp(&b.path));

        let mut out = String::new();
        for page in &matching {
            let content = self.read_page(&page.path)?;
            if !out.is_empty() {
                out.push_str("\n\n---\n\n");
            }
            out.push_str(&format!("## {}\n\n", page.title));
            out.push_str(&content);
        }
        Ok(out)
    }

    /// Concatena o markdown fonte de TODAS as páginas do vault
    /// (`pages/` e `journals/`) — mesmo formato de `export_folder`.
    pub fn export_vault(&self) -> Result<String> {
        self.export_folder("")
    }

    /// Lista templates em `templates/` (mesmo padrão de `list_pages`,
    /// restrito a essa pasta). `templates/` fica fora de `pages/` e
    /// `journals/` de propósito — não é uma "página" (não aparece na
    /// sidebar normal, tags, busca ou backlinks).
    pub fn list_templates(&self) -> Result<Vec<PageMeta>> {
        let dir = self.root.join("templates");
        if !dir.is_dir() {
            return Ok(Vec::new());
        }
        let mut templates = Vec::new();
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
                templates.push(PageMeta {
                    path: relative,
                    title,
                    section: "templates".to_string(),
                });
            }
        }
        templates.sort_by(|a, b| a.title.cmp(&b.title));
        Ok(templates)
    }

    /// Cria uma página nova em `pages/` a partir de um template em
    /// `templates/`, substituindo `{{title}}` e `{{date}}` (corpo E
    /// frontmatter) pelo título escolhido e a data de hoje
    /// (`YYYY-MM-DD`, mesmo formato de `open_today_journal`/
    /// `created`/`updated`). Mesma lógica de slug único de
    /// `create_page_in`. Placeholders desconhecidos (`{{outracoisa}}`)
    /// não são tocados — só esses dois são suportados.
    pub fn create_page_from_template(
        &self,
        template_relative: &str,
        title: &str,
        folder_relative: Option<&str>,
    ) -> Result<PageMeta> {
        let template_full = self.resolve_safe(template_relative)?;
        let template_content = std::fs::read_to_string(&template_full)
            .map_err(|e| anyhow::anyhow!("erro ao ler template {}: {}", template_relative, e))?;
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        let content = template_content.replace("{{title}}", title).replace("{{date}}", &today);

        let dir_prefix = match folder_relative {
            Some(f) => format!("{}/", f.trim_end_matches('/')),
            None => "pages/".to_string(),
        };
        let (slug, relative) = self.find_unique_relative_path(&dir_prefix, title)?;
        self.write_page(&relative, &content)?;
        Ok(PageMeta {
            path: relative,
            title: slug,
            section: "pages".to_string(),
        })
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

    /// Lista arquivos em `assets/` com tamanho — usado pela página
    /// `type: assets` (gestão de anexos). `list_assets` (só paths)
    /// continua existindo pro autocomplete do editor, que não precisa
    /// de tamanho.
    pub fn list_assets_info(&self) -> Result<Vec<AssetInfo>> {
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
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                files.push(AssetInfo { path: relative, size });
            }
        }
        files.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(files)
    }

    /// Remove um arquivo de `assets/`. Recusa qualquer path fora de
    /// `assets/` (não é uma exclusão de página).
    pub fn delete_asset(&self, relative_path: &str) -> Result<()> {
        if !relative_path.starts_with("assets/") {
            anyhow::bail!("só é permitido excluir arquivos dentro de assets/");
        }
        let full = self.resolve_safe(relative_path)?;
        std::fs::remove_file(&full)
            .map_err(|e| anyhow::anyhow!("erro ao excluir {}: {}", relative_path, e))?;
        Ok(())
    }

    /// Grava bytes arbitrários em `assets/`, gerando um nome único
    /// (`colado-N.ext`) — usado pelo paste de imagem no editor
    /// (ciclo 118), onde não existe um arquivo de origem com nome
    /// (o dado vem direto da área de transferência). `extension` sem
    /// o ponto (ex: `"png"`).
    pub fn save_asset_bytes(&self, extension: &str, bytes: &[u8]) -> Result<String> {
        let dest_dir = self.root.join("assets");
        std::fs::create_dir_all(&dest_dir)
            .map_err(|e| anyhow::anyhow!("erro ao criar assets/: {}", e))?;
        let ext = extension.trim_start_matches('.');
        let mut n = 1u32;
        loop {
            let filename = format!("colado-{}.{}", n, ext);
            let dest = dest_dir.join(&filename);
            if !dest.exists() {
                std::fs::write(&dest, bytes)
                    .map_err(|e| anyhow::anyhow!("erro ao gravar {}: {}", filename, e))?;
                let relative = dest
                    .strip_prefix(&self.root)
                    .unwrap_or(&dest)
                    .to_string_lossy()
                    .to_string();
                return Ok(relative);
            }
            n += 1;
            if n > 10_000 {
                anyhow::bail!("não foi possível gerar nome único pro asset colado");
            }
        }
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
    fn list_assets_info_returns_size() {
        let (dir, io) = setup_vault();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/foto.png"), b"12345").unwrap();
        let assets = io.list_assets_info().unwrap();
        assert_eq!(assets.len(), 1);
        assert_eq!(assets[0].path, "assets/foto.png");
        assert_eq!(assets[0].size, 5);
    }

    #[test]
    fn list_assets_info_empty_without_dir() {
        let (_dir, io) = setup_vault();
        assert!(io.list_assets_info().unwrap().is_empty());
    }

    #[test]
    fn delete_asset_removes_file() {
        let (dir, io) = setup_vault();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/foto.png"), b"x").unwrap();
        io.delete_asset("assets/foto.png").unwrap();
        assert!(io.list_assets_info().unwrap().is_empty());
    }

    #[test]
    fn delete_asset_rejects_outside_assets_dir() {
        let (_dir, io) = setup_vault();
        assert!(io.delete_asset("pages/alpha.md").is_err());
    }

    #[test]
    fn delete_asset_rejects_escape() {
        let (_dir, io) = setup_vault();
        assert!(io.delete_asset("assets/../../etc/passwd").is_err());
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

    #[test]
    fn export_vault_concatenates_all_pages_with_headers() {
        let (_dir, io) = setup_vault();
        let dump = io.export_vault().unwrap();
        assert!(dump.contains("## alpha"));
        assert!(dump.contains("# Alpha"));
        assert!(dump.contains("## beta"));
        assert!(dump.contains("## gamma"));
        assert!(dump.contains("## 2026-01-01"));
        assert!(dump.contains("\n\n---\n\n"));
    }

    #[test]
    fn export_folder_filters_by_prefix() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("pages/trabalho")).unwrap();
        fs::write(dir.path().join("pages/trabalho/tarefa.md"), "---\ntitle: Tarefa\n---\n# Tarefa\n").unwrap();
        let dump = io.export_folder("pages/trabalho").unwrap();
        assert!(dump.contains("## tarefa"));
        assert!(!dump.contains("## alpha"));
        assert!(!dump.contains("## beta"));
    }

    #[test]
    fn export_folder_empty_folder_returns_empty_string() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("pages/vazia")).unwrap();
        let dump = io.export_folder("pages/vazia").unwrap();
        assert_eq!(dump, "");
    }

    #[test]
    fn list_templates_returns_empty_without_folder() {
        let (_dir, io) = setup_vault();
        assert!(io.list_templates().unwrap().is_empty());
    }

    #[test]
    fn list_templates_finds_md_files_sorted() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(dir.path().join("templates/spec.md"), "---\ntitle: {{title}}\n---\n").unwrap();
        fs::write(dir.path().join("templates/decisao.md"), "---\ntitle: {{title}}\n---\n").unwrap();
        let templates = io.list_templates().unwrap();
        let titles: Vec<&str> = templates.iter().map(|t| t.title.as_str()).collect();
        assert_eq!(titles, vec!["decisao", "spec"]);
        for t in &templates {
            assert_eq!(t.section, "templates");
        }
    }

    #[test]
    fn list_templates_does_not_leak_into_list_pages() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(dir.path().join("templates/spec.md"), "---\ntitle: {{title}}\n---\n").unwrap();
        let pages = io.list_pages().unwrap();
        assert!(!pages.iter().any(|p| p.title == "spec"));
    }

    #[test]
    fn create_page_from_template_substitutes_title_in_body_and_frontmatter() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(
            dir.path().join("templates/spec.md"),
            "---\ntitle: {{title}}\nstatus: draft\n---\n# {{title}}\n\nConteúdo.\n",
        )
        .unwrap();
        let meta = io
            .create_page_from_template("templates/spec.md", "Minha Spec", None)
            .unwrap();
        assert_eq!(meta.path, "pages/minha-spec.md");
        let content = io.read_page(&meta.path).unwrap();
        assert!(content.contains("title: Minha Spec"));
        assert!(content.contains("# Minha Spec"));
        assert!(content.contains("status: draft"));
        assert!(!content.contains("{{title}}"));
    }

    #[test]
    fn create_page_from_template_substitutes_date_in_body_and_frontmatter() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(
            dir.path().join("templates/decisao.md"),
            "---\ntitle: {{title}}\ndate: {{date}}\n---\n# {{title}}\n\nDecidido em {{date}}.\n",
        )
        .unwrap();
        let meta = io
            .create_page_from_template("templates/decisao.md", "Minha Decisão", None)
            .unwrap();
        let content = io.read_page(&meta.path).unwrap();
        let today = chrono::Local::now().format("%Y-%m-%d").to_string();
        assert!(content.contains(&format!("date: {}", today)));
        assert!(content.contains(&format!("Decidido em {}.", today)));
        assert!(!content.contains("{{date}}"));
    }

    #[test]
    fn create_page_from_template_leaves_unknown_placeholders_untouched() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(
            dir.path().join("templates/spec.md"),
            "---\ntitle: {{title}}\nowner: {{owner}}\n---\n# {{title}}\n",
        )
        .unwrap();
        let meta = io
            .create_page_from_template("templates/spec.md", "X", None)
            .unwrap();
        let content = io.read_page(&meta.path).unwrap();
        assert!(content.contains("owner: {{owner}}"));
    }

    #[test]
    fn create_page_from_template_generates_unique_slug_on_collision() {
        let (dir, io) = setup_vault();
        fs::create_dir_all(dir.path().join("templates")).unwrap();
        fs::write(dir.path().join("templates/spec.md"), "---\ntitle: {{title}}\n---\n").unwrap();
        io.create_page_from_template("templates/spec.md", "Nova Spec", None).unwrap();
        let meta = io.create_page_from_template("templates/spec.md", "Nova Spec", None).unwrap();
        assert_eq!(meta.path, "pages/nova-spec-2.md");
    }

    #[test]
    fn create_page_from_template_rejects_missing_template() {
        let (_dir, io) = setup_vault();
        assert!(io.create_page_from_template("templates/nope.md", "X", None).is_err());
    }

    #[test]
    fn save_asset_bytes_creates_assets_dir_and_writes_file() {
        let (dir, io) = setup_vault();
        let relative = io.save_asset_bytes("png", b"fake-png-bytes").unwrap();
        assert_eq!(relative, "assets/colado-1.png");
        let content = fs::read(dir.path().join(&relative)).unwrap();
        assert_eq!(content, b"fake-png-bytes");
    }

    #[test]
    fn save_asset_bytes_generates_unique_name_on_collision() {
        let (_dir, io) = setup_vault();
        let first = io.save_asset_bytes("png", b"a").unwrap();
        let second = io.save_asset_bytes("png", b"b").unwrap();
        assert_eq!(first, "assets/colado-1.png");
        assert_eq!(second, "assets/colado-2.png");
    }

    #[test]
    fn save_asset_bytes_strips_leading_dot_from_extension() {
        let (_dir, io) = setup_vault();
        let relative = io.save_asset_bytes(".jpg", b"x").unwrap();
        assert_eq!(relative, "assets/colado-1.jpg");
    }
}
