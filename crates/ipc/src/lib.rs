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

    // Cache em disco (ciclo 171): só relê e reparseia o que mudou desde
    // a última varredura. Cache é conveniência — se falhar em qualquer
    // ponto, a varredura completa acontece igual.
    let mut cache = anotadinho_vault::IndexCache::carregar(vault.root());
    let mut entradas = Vec::with_capacity(pages.len());
    let mut paths = Vec::with_capacity(pages.len());

    for p in pages {
        paths.push(p.path.clone());
        let versao = vault.page_version(&p.path);
        if let Some(versao) = &versao {
            if let Some(cacheada) = cache.obter(&p.path, versao) {
                entradas.push(cacheada.clone());
                continue;
            }
        }
        let Ok(content) = vault.read_page(&p.path) else {
            continue;
        };
        let entry = PageIndexEntry::from_content(&p.path, &p.title, &p.section, &content);
        if let Some(versao) = versao {
            cache.guardar(&p.path, versao, entry.clone());
        }
        entradas.push(entry);
    }

    cache.manter_apenas(&paths);
    cache.salvar();
    Ok(entradas)
}

/// Handler de read_page: retorna o conteúdo Markdown bruto.
pub fn handle_read_page(vault_path: String, page_path: String) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    vault.read_page(&page_path).map_err(|e| e.to_string())
}

/// Conteúdo de uma página junto da marca de versão do arquivo
/// (ciclo 173) — o par que o frontend guarda pra detectar escrita
/// concorrente na hora de salvar.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedPage {
    /// Markdown cru.
    pub content: String,
    /// Marca de versão (`None` = arquivo não existe).
    pub version: Option<String>,
}

/// Handler de read_page_versioned: conteúdo + versão numa leitura só.
pub fn handle_read_page_versioned(
    vault_path: String,
    page_path: String,
) -> Result<VersionedPage, String> {
    let vault = VaultIo::open(&vault_path);
    let (content, version) = vault
        .read_page_versioned(&page_path)
        .map_err(|e| e.to_string())?;
    Ok(VersionedPage { content, version })
}

/// Handler de write_page_checked: grava só se o arquivo ainda estiver
/// na versão `expected_version`. Devolve a versão nova.
///
/// `expected_version` vazio/ausente = gravação incondicional (mesmo
/// comportamento de `handle_write_page`, pra criar arquivo novo).
/// Recusa gravar VAZIO por cima de uma página que tem conteúdo.
///
/// Duas propostas do vault foram zeradas (0 bytes) logo depois de um
/// pedido de execução, e a causa não foi reproduzida — nem pelo editor,
/// nem pelo agente sandboxado, que é bloqueado antes de truncar.
///
/// Sem saber quem escreveu, a trava fica no ponto por onde TODOS passam.
/// Apagar uma nota inteira nunca é o resultado certo de um save: quem
/// quer esvaziar uma página apaga a página. Uma página nova (que ainda
/// não existe no disco) continua podendo nascer vazia.
fn recusar_esvaziamento(vault: &VaultIo, page_path: &str, content: &str) -> Result<(), String> {
    if !content.trim().is_empty() {
        return Ok(());
    }
    match vault.read_page(page_path) {
        Ok(atual) if !atual.trim().is_empty() => Err(format!(
            "gravação recusada: isso apagaria as {} letras de \"{}\". \
             Pra esvaziar de propósito, apague a página.",
            atual.trim().chars().count(),
            page_path
        )),
        _ => Ok(()),
    }
}

pub fn handle_write_page_checked(
    vault_path: String,
    page_path: String,
    content: String,
    expected_version: Option<String>,
) -> Result<String, String> {
    let vault = VaultIo::open(&vault_path);
    recusar_esvaziamento(&vault, &page_path, &content)?;
    vault
        .write_page_checked(&page_path, &content, expected_version.as_deref())
        .map_err(|e| e.to_string())
}

/// Handler de write_page: grava conteúdo Markdown no disco.
pub fn handle_write_page(
    vault_path: String,
    page_path: String,
    content: String,
) -> Result<(), String> {
    let vault = VaultIo::open(&vault_path);
    recusar_esvaziamento(&vault, &page_path, &content)?;
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

/// Handler de criar_vault: prepara uma pasta para ser um vault (ciclo 233).
///
/// Semeia estrutura, templates, padrões, prompts e a página inicial.
/// **Nunca sobrescreve**: um arquivo que já existe é deixado como está, e
/// devolvido na lista de ignorados. Assim "preparar" uma pasta que já
/// tem coisa dentro é seguro — completa o que falta sem tocar no resto.
///
/// Devolve os caminhos criados.
pub fn handle_criar_vault(vault_path: String) -> Result<Vec<String>, String> {
    let raiz = std::path::Path::new(&vault_path);
    if raiz.exists() && !raiz.is_dir() {
        return Err(format!("{vault_path} existe e não é uma pasta"));
    }
    std::fs::create_dir_all(raiz).map_err(|e| format!("não consegui criar {vault_path}: {e}"))?;

    for pasta in anotadinho_core::semente::PASTAS {
        std::fs::create_dir_all(raiz.join(pasta))
            .map_err(|e| format!("não consegui criar {pasta}: {e}"))?;
    }

    let mut criados = Vec::new();
    for arquivo in anotadinho_core::semente::arquivos() {
        let destino = raiz.join(arquivo.caminho);
        if destino.exists() {
            continue;
        }
        if let Some(pai) = destino.parent() {
            std::fs::create_dir_all(pai)
                .map_err(|e| format!("não consegui criar {}: {e}", pai.display()))?;
        }
        std::fs::write(&destino, arquivo.conteudo)
            .map_err(|e| format!("não consegui escrever {}: {e}", arquivo.caminho))?;
        criados.push(arquivo.caminho.to_string());
    }
    Ok(criados)
}

/// A pasta já parece um vault com conteúdo? (ciclo 233)
///
/// Serve pra decidir entre "abrir" e "preparar": uma pasta sem nenhuma
/// página é uma pasta vazia com cara de vault quebrado, e a pessoa não
/// tem como saber disso olhando a tela.
pub fn handle_vault_esta_vazio(vault_path: String) -> Result<bool, String> {
    let raiz = std::path::Path::new(&vault_path);
    for pasta in ["pages", "journals"] {
        let dir = raiz.join(pasta);
        let Ok(entradas) = std::fs::read_dir(&dir) else { continue };
        for e in entradas.flatten() {
            if e.path().extension().is_some_and(|x| x == "md") {
                return Ok(false);
            }
            if e.path().is_dir()
                && std::fs::read_dir(e.path())
                    .map(|mut i| i.any(|f| f.is_ok_and(|f| f.path().extension().is_some_and(|x| x == "md"))))
                    .unwrap_or(false)
            {
                return Ok(false);
            }
        }
    }
    Ok(true)
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
        .map(|t| PageMeta {
            path: t.path,
            title: t.title,
            section: t.section,
        })
        .collect())
}

/// Handler de create_page_from_template: cria página a partir de um
/// template em `templates/`, substituindo `{{title}}` pelo título.
pub fn handle_create_page_from_template(
    vault_path: String,
    template_path: String,
    title: String,
    folder_path: Option<String>,
) -> Result<PageMeta, String> {
    let vault = VaultIo::open(&vault_path);
    let meta = vault
        .create_page_from_template(&template_path, &title, folder_path.as_deref())
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
    Ok(assets
        .into_iter()
        .map(|a| AssetInfo {
            path: a.path,
            size: a.size,
        })
        .collect())
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
    vault
        .save_asset_bytes(&extension, &bytes)
        .map_err(|e| e.to_string())
}

/// Imagem enviada pelo frontend para uma gravação em lote.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAssetPayload {
    /// Nome original, usado apenas para preencher a interface.
    #[serde(default)]
    pub name: String,
    /// Extensão sem ponto.
    pub extension: String,
    /// Conteúdo codificado em base64.
    pub base64_data: String,
}

/// Grava um lote como uma única operação lógica: tudo é validado antes
/// e qualquer arquivo já publicado é removido se uma etapa posterior falhar.
pub fn handle_save_image_assets(
    vault_path: String,
    images: Vec<ImageAssetPayload>,
) -> Result<Vec<String>, String> {
    use base64::Engine;
    if images.is_empty() {
        return Err("o lote de imagens está vazio".into());
    }
    let mut decoded = Vec::with_capacity(images.len());
    for image in images {
        let ext = image.extension.trim_start_matches('.').to_ascii_lowercase();
        if !matches!(
            ext.as_str(),
            "png" | "jpg" | "jpeg" | "gif" | "webp" | "svg"
        ) {
            return Err(format!("tipo de imagem não aceito: {ext}"));
        }
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(image.base64_data)
            .map_err(|e| format!("base64 inválido: {e}"))?;
        validate_image_bytes(&ext, &bytes)?;
        decoded.push((ext, bytes));
    }
    let vault = VaultIo::open(&vault_path);
    let mut paths = Vec::with_capacity(decoded.len());
    for (ext, bytes) in decoded {
        match vault.save_asset_bytes(&ext, &bytes) {
            Ok(path) => paths.push(path),
            Err(error) => {
                for path in &paths {
                    let _ = vault.delete_asset(path);
                }
                return Err(error.to_string());
            }
        }
    }
    Ok(paths)
}

fn validate_image_bytes(ext: &str, bytes: &[u8]) -> Result<(), String> {
    let valid = match ext {
        "png" => bytes.starts_with(b"\x89PNG\r\n\x1a\n"),
        "jpg" | "jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        "webp" => bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP",
        "svg" => std::str::from_utf8(bytes)
            .map(|s| s.contains("<svg"))
            .unwrap_or(false),
        _ => false,
    };
    valid
        .then_some(())
        .ok_or_else(|| format!("conteúdo não corresponde a uma imagem {ext}"))
}

/// Handler de read_asset_data_url: lê um arquivo de `assets/` (ou
/// qualquer path dentro do vault) e devolve como `data:` URL
/// (ciclo 121) — um `src` relativo cru (`assets/x.png`) resolve
/// contra a origem do webview, não contra a pasta real do vault no
/// disco, então imagens/PDFs embutidos precisam desse passo pra
/// aparecer de verdade.
pub fn handle_read_asset_data_url(
    vault_path: String,
    asset_path: String,
) -> Result<String, String> {
    use base64::Engine;
    let vault = VaultIo::open(&vault_path);
    let bytes = vault
        .read_asset_bytes(&asset_path)
        .map_err(|e| e.to_string())?;
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
    vault
        .copy_to_assets(&source_path)
        .map_err(|e| e.to_string())
}

/// Handler de search_content: busca texto no conteúdo das páginas.
///
/// Cada página entra no índice duas vezes por natureza (ciclo 188): o
/// markdown solto como um documento, e cada REGISTRO de dentro dos
/// embeds como um documento próprio. Sem isso, procurar "Tarefa 2"
/// casava com a linha `- title: Tarefa 2` do YAML cru — abria a página
/// certa, mas sem dizer que aquilo era um card nem em que coluna.
pub fn handle_search_content(
    vault_path: String,
    query: String,
) -> Result<Vec<anotadinho_core::embed::SearchHit>, String> {
    let vault = VaultIo::open(&vault_path);
    let pages = vault.list_pages().map_err(|e| e.to_string())?;
    let mut index = SearchIndex::new().map_err(|e| e.to_string())?;
    for page in &pages {
        let Ok(content) = vault.read_page(&page.path) else {
            continue;
        };
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&content);
        let segmentos = anotadinho_core::embed::segment(corpo);

        // O markdown solto — sem o YAML dos embeds, que agora é indexado
        // de forma estruturada. Deixar os dois faria cada card aparecer
        // duas vezes na mesma busca.
        let texto_solto: String = segmentos
            .iter()
            .filter_map(|seg| match seg {
                anotadinho_core::embed::DocSegment::Markdown(md) => Some(md.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n");
        index
            .index_page(&page.path, &page.title, &texto_solto)
            .map_err(|e| e.to_string())?;

        for (i, seg) in segmentos.iter().enumerate() {
            let anotadinho_core::embed::DocSegment::Embed(data) = seg else {
                continue;
            };
            for hit in data.search_entries() {
                let origem = format!("{} · {}", hit.kind.label(), hit.contexto);
                index
                    .index_embed_entry(
                        &page.path,
                        &page.title,
                        &hit.texto,
                        &origem,
                        &format!("{i}:{}", hit.indice),
                    )
                    .map_err(|e| e.to_string())?;
            }
        }
    }
    let results = index.search(&query, 20).map_err(|e| e.to_string())?;
    Ok(results
        .into_iter()
        .map(|r| anotadinho_core::embed::SearchHit {
            path: r.page_path,
            snippet: r.snippet,
            origem: r.origem,
            ancora: r.ancora,
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── trava contra esvaziamento (ciclo 215) ─────────────────────

    #[test]
    fn gravar_vazio_por_cima_de_pagina_com_conteudo_e_recusado() {
        let dir = vault_temp();
        let vault = dir.path().to_string_lossy().to_string();
        let pagina = "pages/importante.md".to_string();
        handle_write_page(vault.clone(), pagina.clone(), "texto que importa".into()).unwrap();

        let erro = handle_write_page(vault.clone(), pagina.clone(), String::new()).unwrap_err();
        assert!(erro.contains("recusada"), "erro não explica: {erro}");

        // E o arquivo continua lá, inteiro.
        assert_eq!(
            handle_read_page(vault, pagina).unwrap().trim(),
            "texto que importa"
        );
    }

    #[test]
    fn so_espaco_em_branco_tambem_conta_como_vazio() {
        let dir = vault_temp();
        let vault = dir.path().to_string_lossy().to_string();
        let pagina = "pages/importante.md".to_string();
        handle_write_page(vault.clone(), pagina.clone(), "texto".into()).unwrap();
        assert!(handle_write_page(vault, pagina, "  \n\n  ".into()).is_err());
    }

    #[test]
    fn pagina_nova_ainda_pode_nascer_vazia() {
        // A trava é contra APAGAR o que existe, não contra criar.
        let dir = vault_temp();
        let vault = dir.path().to_string_lossy().to_string();
        handle_write_page(vault, "pages/nova.md".into(), String::new()).unwrap();
    }

    #[test]
    fn a_trava_vale_para_o_caminho_com_versao() {
        let dir = vault_temp();
        let vault = dir.path().to_string_lossy().to_string();
        let pagina = "pages/importante.md".to_string();
        let v =
            handle_write_page_checked(vault.clone(), pagina.clone(), "texto".into(), None).unwrap();
        assert!(
            handle_write_page_checked(vault, pagina, String::new(), Some(v)).is_err(),
            "o caminho com versão passou por cima da trava"
        );
    }

    // ── propostas (ciclo 204) ─────────────────────────────────────

    fn vault_temp() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        dir
    }

    fn png_payload(name: &str) -> ImageAssetPayload {
        use base64::Engine;
        ImageAssetPayload {
            name: name.into(),
            extension: "png".into(),
            base64_data: base64::engine::general_purpose::STANDARD
                .encode(b"\x89PNG\r\n\x1a\nresto"),
        }
    }

    #[test]
    fn lote_cria_um_asset_novo_por_insercao_identica() {
        let dir = vault_temp();
        let paths = handle_save_image_assets(
            dir.path().to_string_lossy().to_string(),
            vec![png_payload("a.png"), png_payload("a.png")],
        )
        .unwrap();
        assert_eq!(paths, vec!["assets/colado-1.png", "assets/colado-2.png"]);
        assert!(paths.iter().all(|p| dir.path().join(p).exists()));
    }

    #[test]
    fn lote_invalido_nao_publica_nada() {
        let dir = vault_temp();
        let mut invalid = png_payload("quebrada.png");
        invalid.base64_data = "bmFvLWUtdW1hLWltYWdlbQ==".into();
        assert!(handle_save_image_assets(
            dir.path().to_string_lossy().to_string(),
            vec![png_payload("ok.png"), invalid]
        )
        .is_err());
        assert!(
            !dir.path().join("assets").exists(),
            "validou parcialmente antes de gravar"
        );
    }

    fn proposta_de(
        alvo: &str,
        op: anotadinho_core::proposta::Operacao,
        conteudo: &str,
    ) -> anotadinho_core::proposta::Proposta {
        anotadinho_core::proposta::Proposta {
            id: "p1".into(),
            autor: "teste".into(),
            quando: "2026-08-22 10:00".into(),
            motivo: "teste".into(),
            alvo: alvo.into(),
            operacao: op,
            conteudo: conteudo.into(),
        }
    }

    #[test]
    fn propor_grava_fora_de_pages_e_nao_toca_no_alvo() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        let p = proposta_de(
            "pages/nova.md",
            Operacao::Criar,
            "---\ntitle: N\n---\ncorpo\n",
        );

        handle_propor(v.clone(), p).unwrap();

        // A página NÃO foi escrita — é o ponto inteiro.
        assert!(
            !dir.path().join("pages/nova.md").exists(),
            "escreveu sem aprovação"
        );
        assert!(dir.path().join(".anotadinho/propostas/p1.json").exists());
        assert_eq!(handle_listar_propostas(v).unwrap().len(), 1);
    }

    #[test]
    fn aplicar_escreve_a_pagina_e_some_a_proposta() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        handle_propor(
            v.clone(),
            proposta_de(
                "pages/nova.md",
                Operacao::Criar,
                "---\ntitle: N\n---\ncorpo\n",
            ),
        )
        .unwrap();

        handle_aplicar_proposta(v.clone(), "p1".into()).unwrap();

        let escrito = std::fs::read_to_string(dir.path().join("pages/nova.md")).unwrap();
        assert!(escrito.contains("corpo"), "{escrito}");
        assert!(
            handle_listar_propostas(v).unwrap().is_empty(),
            "a proposta ficou pendurada"
        );
    }

    #[test]
    fn recusar_descarta_sem_escrever() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        handle_propor(
            v.clone(),
            proposta_de("pages/nova.md", Operacao::Criar, "---\ntitle: N\n---\nx\n"),
        )
        .unwrap();

        handle_recusar_proposta(v.clone(), "p1".into()).unwrap();

        assert!(!dir.path().join("pages/nova.md").exists());
        assert!(handle_listar_propostas(v).unwrap().is_empty());
    }

    #[test]
    fn propor_recusa_caminho_fora_do_vault() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        let erro = handle_propor(v, proposta_de("../fora.md", Operacao::Criar, "x")).unwrap_err();
        assert!(erro.contains("fora do vault"), "{erro}");
    }

    #[test]
    fn aplicar_revalida_e_recusa_se_o_vault_mudou() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        handle_propor(
            v.clone(),
            proposta_de("pages/nova.md", Operacao::Criar, "---\ntitle: N\n---\nx\n"),
        )
        .unwrap();

        // Alguém criou a página no intervalo entre propor e aprovar.
        std::fs::write(dir.path().join("pages/nova.md"), "escrito por outra pessoa").unwrap();

        let erro = handle_aplicar_proposta(v, "p1".into()).unwrap_err();
        assert!(erro.contains("mudou"), "{erro}");
        let atual = std::fs::read_to_string(dir.path().join("pages/nova.md")).unwrap();
        assert_eq!(
            atual, "escrito por outra pessoa",
            "sobrescreveu o trabalho de outro"
        );
    }

    #[test]
    fn proposta_ilegivel_nao_esconde_as_outras() {
        use anotadinho_core::proposta::Operacao;
        let dir = vault_temp();
        let v = dir.path().to_string_lossy().to_string();
        handle_propor(
            v.clone(),
            proposta_de("pages/a.md", Operacao::Criar, "---\ntitle: A\n---\nx\n"),
        )
        .unwrap();
        std::fs::write(
            dir.path().join(".anotadinho/propostas/corrompida.json"),
            "{ nao json",
        )
        .unwrap();

        assert_eq!(handle_listar_propostas(v).unwrap().len(), 1);
    }

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

        let spec = entries
            .iter()
            .find(|e| e.path.ends_with("uma-spec.md"))
            .unwrap();
        assert_eq!(spec.title, "Uma Spec");
        assert_eq!(spec.section, "pages");
        assert_eq!(spec.tags, vec!["spec"]);
        assert_eq!(spec.field("status").as_deref(), Some("backlog"));
        assert_eq!(spec.wikilinks, vec!["Missão"]);

        let journal = entries.iter().find(|e| e.section == "journals").unwrap();
        assert_eq!(journal.field("date").as_deref(), Some("2026-08-19"));
    }

    #[test]
    fn write_checked_via_ipc_recusa_versao_velha() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        std::fs::write(dir.path().join("pages/a.md"), "original\n").unwrap();
        let vault = dir.path().to_string_lossy().to_string();

        let page = handle_read_page_versioned(vault.clone(), "pages/a.md".into()).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        handle_write_page(vault.clone(), "pages/a.md".into(), "de fora\n".into()).unwrap();

        let err = handle_write_page_checked(
            vault.clone(),
            "pages/a.md".into(),
            "do editor\n".into(),
            page.version.clone(),
        )
        .unwrap_err();
        assert!(err.contains("CONFLITO"), "{err}");

        // Com a versão atual, passa.
        let atual = handle_read_page_versioned(vault.clone(), "pages/a.md".into()).unwrap();
        assert!(handle_write_page_checked(
            vault,
            "pages/a.md".into(),
            "do editor\n".into(),
            atual.version
        )
        .is_ok());
    }

    #[test]
    fn segunda_varredura_usa_o_cache_e_enxerga_mudanca() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        std::fs::write(
            dir.path().join("pages/a.md"),
            "---\ntitle: A\nstatus: backlog\n---\n",
        )
        .unwrap();
        let vault = dir.path().to_string_lossy().to_string();

        let primeira = handle_scan_vault(vault.clone()).unwrap();
        assert_eq!(primeira.len(), 1);
        assert!(
            dir.path().join(".anotadinho/index.json").exists(),
            "o cache devia ter sido gravado"
        );

        // Segunda varredura sem mudança: mesmo resultado.
        let segunda = handle_scan_vault(vault.clone()).unwrap();
        assert_eq!(segunda, primeira);

        // Arquivo muda → o cache não pode devolver o valor velho.
        std::thread::sleep(std::time::Duration::from_millis(10));
        std::fs::write(
            dir.path().join("pages/a.md"),
            "---\ntitle: A\nstatus: done\n---\n",
        )
        .unwrap();
        let terceira = handle_scan_vault(vault.clone()).unwrap();
        assert_eq!(terceira[0].field("status").as_deref(), Some("done"));

        // Página apagada some do resultado.
        std::fs::remove_file(dir.path().join("pages/a.md")).unwrap();
        assert!(handle_scan_vault(vault).unwrap().is_empty());
    }

    #[test]
    fn handle_scan_vault_de_vault_vazio_devolve_lista_vazia() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        let entries = handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap();
        assert!(entries.is_empty());
    }

    #[test]
    fn busca_acha_card_de_embed_com_origem_e_ancora() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        std::fs::write(
            dir.path().join("pages/board.md"),
            "---\ntitle: Board\n---\n\ntexto solto\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: Zarabatana\n  column: Backlog\n{{ /kanban }}\n",
        )
        .unwrap();

        let r = handle_search_content(
            dir.path().to_string_lossy().to_string(),
            "Zarabatana".to_string(),
        )
        .unwrap();

        assert_eq!(
            r.len(),
            1,
            "devia achar UMA vez, não uma pelo YAML e outra pelo registro: {r:?}"
        );
        assert_eq!(r[0].path, "pages/board.md");
        assert_eq!(r[0].origem.as_deref(), Some("Kanban · coluna Backlog"));
        assert_eq!(r[0].ancora.as_deref(), Some("1:0"));
    }

    #[test]
    fn busca_em_texto_solto_nao_ganha_origem() {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        std::fs::write(
            dir.path().join("pages/nota.md"),
            "---\ntitle: Nota\n---\n\numa palavra rarissima aqui\n",
        )
        .unwrap();
        let r = handle_search_content(
            dir.path().to_string_lossy().to_string(),
            "rarissima".to_string(),
        )
        .unwrap();
        assert_eq!(r.len(), 1);
        assert!(r[0].origem.is_none(), "texto solto não vem de embed nenhum");
        assert!(r[0].ancora.is_none());
    }

    #[test]
    fn nome_de_campo_do_yaml_nao_casa_mais() {
        // O YAML cru saiu do índice: buscar "column" não pode achar
        // todo board do vault.
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages")).unwrap();
        std::fs::write(
            dir.path().join("pages/board.md"),
            "---\ntitle: Board\n---\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: Um card\n  column: Backlog\n{{ /kanban }}\n",
        )
        .unwrap();
        let r = handle_search_content(
            dir.path().to_string_lossy().to_string(),
            "column".to_string(),
        )
        .unwrap();
        assert!(r.is_empty(), "nome de campo do YAML não é conteúdo: {r:?}");
    }
}

// ── propostas de escrita (ciclo 204) ─────────────────────────────────

/// Grava uma proposta pra revisão humana, em vez de escrever a página.
///
/// É o que separa "o agente mexeu no meu vault" de "o agente sugeriu e
/// eu aprovei". A proposta chega já validada — caminho dentro do vault,
/// estado coerente e embeds conferidos — pra a revisão ser sobre o
/// CONTEÚDO, não sobre se aquilo sequer é aplicável.
pub fn handle_propor(
    vault_path: String,
    proposta: anotadinho_core::proposta::Proposta,
) -> Result<String, String> {
    let raiz = std::path::Path::new(&vault_path);
    let existe = raiz.join(&proposta.alvo).exists();
    if let Some(r) = proposta.validar(existe) {
        return Err(r.mensagem());
    }
    let pasta = raiz.join(anotadinho_core::proposta::PASTA);
    std::fs::create_dir_all(&pasta)
        .map_err(|e| format!("erro criando {}: {e}", pasta.display()))?;
    let arquivo = raiz.join(proposta.arquivo());
    let json = serde_json::to_string_pretty(&proposta).map_err(|e| e.to_string())?;
    std::fs::write(&arquivo, json).map_err(|e| format!("erro gravando proposta: {e}"))?;
    Ok(proposta.id)
}

/// Lista as propostas pendentes, das mais novas pras mais velhas.
pub fn handle_listar_propostas(
    vault_path: String,
) -> Result<Vec<anotadinho_core::proposta::Proposta>, String> {
    let pasta = std::path::Path::new(&vault_path).join(anotadinho_core::proposta::PASTA);
    if !pasta.exists() {
        return Ok(Vec::new());
    }
    let mut out = Vec::new();
    let entradas = std::fs::read_dir(&pasta).map_err(|e| e.to_string())?;
    for e in entradas.flatten() {
        if e.path().extension().and_then(|x| x.to_str()) != Some("json") {
            continue;
        }
        let Ok(texto) = std::fs::read_to_string(e.path()) else {
            continue;
        };
        // Proposta ilegível é PULADA, não fatal: uma sozinha corrompida
        // não pode esconder as outras da revisão.
        if let Ok(p) = serde_json::from_str::<anotadinho_core::proposta::Proposta>(&texto) {
            out.push(p);
        }
    }
    out.sort_by(|a, b| b.quando.cmp(&a.quando));
    Ok(out)
}

/// Aplica uma proposta: escreve a página e apaga a proposta.
///
/// Revalida ANTES de escrever — entre propor e aprovar o vault pode ter
/// mudado, e aplicar às cegas escreveria por cima do que ninguém viu.
pub fn handle_aplicar_proposta(vault_path: String, id: String) -> Result<String, String> {
    let raiz = std::path::Path::new(&vault_path);
    let arquivo = raiz.join(format!("{}/{id}.json", anotadinho_core::proposta::PASTA));
    let texto =
        std::fs::read_to_string(&arquivo).map_err(|_| format!("proposta {id} não existe"))?;
    let proposta: anotadinho_core::proposta::Proposta =
        serde_json::from_str(&texto).map_err(|e| format!("proposta ilegível: {e}"))?;

    let existe = raiz.join(&proposta.alvo).exists();
    if let Some(r) = proposta.validar(existe) {
        return Err(r.mensagem());
    }
    let vault = VaultIo::open(&vault_path);
    vault
        .write_page(&proposta.alvo, &proposta.conteudo)
        .map_err(|e| e.to_string())?;
    let _ = std::fs::remove_file(&arquivo);
    Ok(proposta.alvo)
}

/// Descarta uma proposta sem aplicar.
pub fn handle_recusar_proposta(vault_path: String, id: String) -> Result<(), String> {
    let arquivo = std::path::Path::new(&vault_path)
        .join(format!("{}/{id}.json", anotadinho_core::proposta::PASTA));
    std::fs::remove_file(&arquivo).map_err(|_| format!("proposta {id} não existe"))?;
    Ok(())
}

#[cfg(test)]
mod testes_semente {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn vault_novo_nasce_navegavel() {
        let dir = TempDir::new().unwrap();
        let raiz = dir.path().to_string_lossy().to_string();
        let criados = handle_criar_vault(raiz.clone()).expect("criar");

        assert!(criados.contains(&anotadinho_core::semente::PAGINA_INICIAL.to_string()));
        // As pastas que o código espera existir têm que existir mesmo
        // vazias — é delas que dependem o fluxo e o seletor de prompt.
        for pasta in anotadinho_core::semente::PASTAS {
            assert!(dir.path().join(pasta).is_dir(), "faltou a pasta {pasta}");
        }
        // E o vault recém-criado já se enxerga: a varredura tem que
        // achar a página inicial e os prompts.
        let paginas = handle_scan_vault(raiz).expect("varrer");
        assert!(paginas.iter().any(|p| p.path == "pages/inicio.md"));
        assert!(
            !anotadinho_core::prompt_padrao::descobrir(paginas).is_empty(),
            "o seletor de prompt nasceria vazio"
        );
    }

    #[test]
    fn preparar_de_novo_nao_sobrescreve_nada() {
        let dir = TempDir::new().unwrap();
        let raiz = dir.path().to_string_lossy().to_string();
        handle_criar_vault(raiz.clone()).unwrap();

        let inicial = dir.path().join(anotadinho_core::semente::PAGINA_INICIAL);
        std::fs::write(&inicial, "---\ntitle: Meu\n---\nescrevi por cima\n").unwrap();

        let criados = handle_criar_vault(raiz).expect("preparar de novo");
        assert!(criados.is_empty(), "recriou arquivo que já existia: {criados:?}");
        assert!(
            std::fs::read_to_string(&inicial).unwrap().contains("escrevi por cima"),
            "a semente destruiu o que a pessoa escreveu"
        );
    }

    #[test]
    fn pasta_com_pagina_nao_e_considerada_vazia() {
        let dir = TempDir::new().unwrap();
        let raiz = dir.path().to_string_lossy().to_string();
        assert!(handle_vault_esta_vazio(raiz.clone()).unwrap(), "pasta vazia");

        std::fs::create_dir_all(dir.path().join("pages/specs")).unwrap();
        assert!(handle_vault_esta_vazio(raiz.clone()).unwrap(), "só pastas ainda é vazio");

        std::fs::write(dir.path().join("pages/specs/x.md"), "---\ntitle: X\n---\n").unwrap();
        assert!(!handle_vault_esta_vazio(raiz).unwrap(), "página em subpasta conta");
    }
}
