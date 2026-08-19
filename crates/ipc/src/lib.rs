//! Anotadinho IPC: comandos Tauri expostos pro Yew frontend.
//!
//! Os comandos IPC são a única ponte entre o Yew (no WebView) e o
//! backend Rust (Tauri core). Tudo que o UI faz passa por aqui.

#![warn(missing_docs)]

use anotadinho_core::PageIndexEntry;
use anotadinho_search::SearchIndex;
use anotadinho_vault::{GitFileEntry, GitLogEntry, VaultIo};
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

/// Metadados de uma página listada.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageMeta {
    /// Path relativo ao vault.
    pub path: String,
    /// Nome do arquivo (sem extensão).
    pub title: String,
    /// Seção (`pages` ou `journals`).
    pub section: String,
}

/// Handler de list_pages: retorna todas as páginas `.md` do vault.
pub fn handle_list_pages(vault_path: String) -> Result<Vec<PageMeta>, String> {
    let vault = VaultIo::open(&vault_path);
    let pages = vault.list_pages().map_err(|e| e.to_string())?;
    Ok(pages
        .into_iter()
        .map(|p| PageMeta {
            path: p.path,
            title: p.title,
            section: p.section,
        })
        .collect())
}

/// Handler de scan_vault: lê TODAS as páginas uma vez e devolve só os
/// metadados (frontmatter + properties do corpo + tags + wikilinks).
///
/// Substitui o padrão N+1 que o frontend usava — `list_pages()` seguido
/// de um `read_page()` por página, cada um atravessando a ponte com o
/// arquivo inteiro. Página que não puder ser lida é pulada em silêncio:
/// uma varredura do vault não deveria falhar inteira por causa de um
/// arquivo com permissão errada.
pub fn handle_scan_vault(vault_path: String) -> Result<Vec<PageIndexEntry>, String> {
    let vault = VaultIo::open(&vault_path);
    let pages = vault.list_pages().map_err(|e| e.to_string())?;
    Ok(pages
        .into_iter()
        .filter_map(|p| {
            let content = vault.read_page(&p.path).ok()?;
            Some(PageIndexEntry::from_content(
                &p.path, &p.title, &p.section, &content,
            ))
        })
        .collect())
}

/// Handler de read_page: retorna o conteúdo Markdown bruto.
pub fn handle_read_page(vault_path: String, page_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.read_page(&page_path).map_err(|e| e.to_string())
}

/// Handler de write_page: grava conteúdo Markdown no disco.
pub fn handle_write_page(
    vault_path: String,
    page_path: String,
    content: String,
) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault
        .write_page(&page_path, &content)
        .map_err(|e| e.to_string())
}

/// Handler de create_page: cria nova página em pages/.
pub fn handle_create_page(vault_path: String, title: String) -> Result<PageMeta, String> {
    handle_create_page_typed(vault_path, title, "md".to_string())
}

/// Handler de create_page com tipo.
pub fn handle_create_page_typed(
    vault_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault
        .create_page_with_type(&title, &page_type)
        .map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de open_today_journal: abre ou cria journal do dia.
pub fn handle_open_today_journal(vault_path: String) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault.open_today_journal().map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de delete_page: remove arquivo .md do vault.
pub fn handle_delete_page(vault_path: String, page_path: String) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault.delete_page(&page_path).map_err(|e| e.to_string())
}

/// Handler de create_folder: cria pasta (subdiretório) no vault.
pub fn handle_create_folder(vault_path: String, folder_path: String) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault.create_folder(&folder_path).map_err(|e| e.to_string())
}

/// Handler de list_folders: lista pastas (incluindo vazias) sob `pages/`.
pub fn handle_list_folders(vault_path: String) -> Result<Vec<String>, String> {
    let vault = VaultIo::open(&vault_path);
    vault.list_folders().map_err(|e| e.to_string())
}

/// Handler de move_page: move/renomeia uma página pra organizá-la em pastas.
pub fn handle_move_page(
    vault_path: String,
    from_path: String,
    to_path: String,
) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault
        .move_page(&from_path, &to_path)
        .map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de create_page_in_folder: cria página dentro de uma pasta.
pub fn handle_create_page_in_folder(
    vault_path: String,
    folder_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault
        .create_page_in_folder(&folder_path, &title, &page_type)
        .map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de git_status: lista arquivos modificados/não rastreados no
/// vault via `git status --porcelain` (somente leitura). `None` se o
/// vault não for um repositório git ou `git` não estiver instalado —
/// nunca um erro, pra degradar silenciosamente na UI.
pub fn handle_git_status(vault_path: String) -> Option<Vec<GitFileEntry>> {
    anotadinho_vault::git_status(std::path::Path::new(&vault_path))
}

/// Handler de git_log: histórico de commits de uma página específica
/// (somente leitura), via `git log --follow`. `None` nas mesmas
/// condições de `handle_git_status`.
pub fn handle_git_log(vault_path: String, page_path: String) -> Option<Vec<GitLogEntry>> {
    anotadinho_vault::git_log(std::path::Path::new(&vault_path), &page_path)
}

/// Handler de git_pull: `git pull`, ação explícita do usuário
/// (ciclo 119). Erro (conflito, sem remote, etc) vira `Err` com a
/// mensagem do git tal qual — diferente de `handle_git_status`, aqui
/// o usuário precisa saber se falhou.
pub fn handle_git_pull(vault_path: String) -> Result<String, String> {
    anotadinho_vault::git_pull(std::path::Path::new(&vault_path)).map_err(|e| e.to_string())
}

/// Handler de git_commit_and_push: `git add -A && commit && push`,
/// ação explícita do usuário (ciclo 119).
pub fn handle_git_commit_and_push(vault_path: String, message: String) -> Result<String, String> {
    anotadinho_vault::git_commit_and_push(std::path::Path::new(&vault_path), &message)
        .map_err(|e| e.to_string())
}

/// Handler de export_folder: concatena o markdown fonte de todas as
/// páginas dentro de uma pasta (recursivo) num dump único.
/// `folder_path` vazio exporta o vault inteiro.
pub fn handle_export_folder(vault_path: String, folder_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.export_folder(&folder_path).map_err(|e| e.to_string())
}

/// Handler de list_templates: lista templates em `templates/`.
pub fn handle_list_templates(vault_path: String) -> Result<Vec<PageMeta>, String> {
    let vault = VaultIo::open(&vault_path);
    let templates = vault.list_templates().map_err(|e| e.to_string())?;
    Ok(templates
        .into_iter()
        .map(|t| PageMeta { path: t.path, title: t.title, section: t.section })
        .collect())
}

/// Handler de create_page_from_template: cria página a partir de um
/// template em `templates/`, substituindo `{{title}}` pelo título.
pub fn handle_create_page_from_template(
    vault_path: String,
    template_path: String,
    title: String,
) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault
        .create_page_from_template(&template_path, &title, None)
        .map_err(|e| e.to_string())?;
    Ok(PageMeta {
        path: meta.path,
        title: meta.title,
        section: meta.section,
    })
}

/// Handler de list_assets: lista arquivos em assets/.
pub fn handle_list_assets(vault_path: String) -> Result<Vec<String>, String> {
    let vault = VaultIo::open(&vault_path);
    vault.list_assets().map_err(|e| e.to_string())
}

/// Metadados de um arquivo em `assets/`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Path relativo ao vault.
    pub path: String,
    /// Tamanho em bytes.
    pub size: u64,
}

/// Handler de list_assets_info: lista arquivos em assets/ com tamanho.
pub fn handle_list_assets_info(vault_path: String) -> Result<Vec<AssetInfo>, String> {
    let vault = VaultIo::open(&vault_path);
    let assets = vault.list_assets_info().map_err(|e| e.to_string())?;
    Ok(assets.into_iter().map(|a| AssetInfo { path: a.path, size: a.size }).collect())
}

/// Handler de delete_asset: remove um arquivo de assets/.
pub fn handle_delete_asset(vault_path: String, asset_path: String) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    vault.delete_asset(&asset_path).map_err(|e| e.to_string())
}

/// Handler de save_pasted_asset: decodifica base64 (dado que veio da
/// área de transferência, sem arquivo de origem no disco) e grava em
/// `assets/` com nome único (ciclo 118).
pub fn handle_save_pasted_asset(
    vault_path: String,
    extension: String,
    base64_data: String,
) -> Result<String, String> {
    use base64::Engine;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(base64_data)
        .map_err(|e| format!("base64 inválido: {}", e))?;
    let vault = VaultIo::open(&vault_path);
    vault.save_asset_bytes(&extension, &bytes).map_err(|e| e.to_string())
}

/// Handler de read_asset_data_url: lê um arquivo de `assets/` (ou
/// qualquer path dentro do vault) e devolve como `data:` URL
/// (ciclo 121) — um `src` relativo cru (`assets/x.png`) resolve
/// contra a origem do webview, não contra a pasta real do vault no
/// disco, então imagens/PDFs embutidos precisam desse passo pra
/// aparecer de verdade.
pub fn handle_read_asset_data_url(vault_path: String, asset_path: String) -> Result<String, String> {
    use base64::Engine;
    let vault = VaultIo::open(&vault_path);
    let bytes = vault.read_asset_bytes(&asset_path).map_err(|e| e.to_string())?;
    let mime = guess_mime(&asset_path);
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
    Ok(format!("data:{};base64,{}", mime, b64))
}

fn guess_mime(path: &str) -> &'static str {
    let ext = std::path::Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "svg" => "image/svg+xml",
        "webp" => "image/webp",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

/// Handler de copy_to_assets: copia arquivo para assets/.
pub fn handle_copy_to_assets(vault_path: String, source_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.copy_to_assets(&source_path).map_err(|e| e.to_string())
}

/// Handler de search_content: busca texto no conteúdo das páginas.
pub fn handle_search_content(
    vault_path: String,
    query: String,
) -> Result<Vec<(String, String)>, String> {
    let vault = VaultIo::open(&vault_path);
    let pages = vault.list_pages().map_err(|e| e.to_string())?;
    let mut index = SearchIndex::new().map_err(|e| e.to_string())?;
    for page in &pages {
        if let Ok(content) = vault.read_page(&page.path) {
            index
                .index_page(&page.path, &page.title, &content)
                .map_err(|e| e.to_string())?;
        }
    }
    let results = index.search(&query, 20).map_err(|e| e.to_string())?;
    Ok(results.into_iter().map(|r| (r.page_path, r.snippet)).collect())
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

    #[test]
    fn guess_mime_recognizes_known_extensions() {
        assert_eq!(guess_mime("assets/x.png"), "image/png");
        assert_eq!(guess_mime("assets/x.PDF"), "application/pdf");
        assert_eq!(guess_mime("assets/x.jpeg"), "image/jpeg");
        assert_eq!(guess_mime("assets/x.unknown"), "application/octet-stream");
    }

    #[test]
    fn handle_read_asset_data_url_roundtrips_bytes() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("assets")).unwrap();
        std::fs::write(dir.path().join("assets/x.pdf"), b"%PDF-1.4 fake").unwrap();
        let url = handle_read_asset_data_url(
            dir.path().to_string_lossy().to_string(),
            "assets/x.pdf".to_string(),
        )
        .unwrap();
        assert!(url.starts_with("data:application/pdf;base64,"));
    }

    #[test]
    fn handle_scan_vault_le_metadados_de_todas_as_paginas() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages/specs")).unwrap();
        std::fs::create_dir_all(dir.path().join("journals")).unwrap();
        std::fs::write(
            dir.path().join("pages/specs/uma-spec.md"),
            "---\ntitle: Uma Spec\ntags: [spec]\nstatus: backlog\n---\n\nliga em [[Missão]]\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("journals/2026-08-19.md"),
            "---\ntitle: Diário\n---\n\ndate:: 2026-08-19\n",
        )
        .unwrap();

        let entries = handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap();
        assert_eq!(entries.len(), 2);

        let spec = entries.iter().find(|e| e.path.ends_with("uma-spec.md")).unwrap();
        assert_eq!(spec.title, "Uma Spec");
        assert_eq!(spec.section, "pages");
        assert_eq!(spec.tags, vec!["spec"]);
        assert_eq!(spec.field("status").as_deref(), Some("backlog"));
        assert_eq!(spec.wikilinks, vec!["Missão"]);

        let journal = entries.iter().find(|e| e.section == "journals").unwrap();
        assert_eq!(journal.field("date").as_deref(), Some("2026-08-19"));
    }

    #[test]
    fn handle_scan_vault_de_vault_vazio_devolve_lista_vazia() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        let entries = handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(entries.is_empty());
    }
}
