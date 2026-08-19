//! Ponte IPC entre o Yew (WASM) e o backend Tauri.
//!
//! Fornece funções async que chamam comandos Tauri via
//! `window.__TAURI_INTERNALS__.invoke()`.

pub use anotadinho_core::PageIndexEntry;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use serde::{Deserialize, Serialize};

/// Informações de um vault (retornadas pelo comando `get_vault_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    /// Path absoluto do vault.
    pub path: String,
    /// Nome do diretório.
    pub name: String,
}

/// Metadados de uma página listada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageMeta {
    /// Path relativo ao vault.
    pub path: String,
    /// Nome do arquivo (sem extensão).
    pub title: String,
    /// Seção (`pages` ou `journals`).
    pub section: String,
}

/// Uma linha de `git status --porcelain` (ciclo 103, somente leitura).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitFileEntry {
    /// Path relativo ao vault.
    pub path: String,
    /// Status resumido: `M`/`A`/`D`/`R`/`??`.
    pub status: String,
}

/// Um commit do histórico de uma página (ciclo 117, somente leitura).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitLogEntry {
    /// Hash curto do commit.
    pub hash: String,
    /// Data do commit (`YYYY-MM-DD`).
    pub date: String,
    /// Mensagem do commit.
    pub message: String,
}

fn get_invoke_fn() -> Result<js_sys::Function, JsValue> {
    let w = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let ipc = js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI_INTERNALS__"))?;
    if ipc.is_undefined() {
        return Err(JsValue::from_str("__TAURI_INTERNALS__ not available"));
    }
    let invoke = js_sys::Reflect::get(&ipc, &JsValue::from_str("invoke"))?;
    invoke.dyn_into::<js_sys::Function>()
}

async fn tauri_invoke(cmd: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let invoke = get_invoke_fn()?;
    let promise_js = invoke.call2(&JsValue::null(), &JsValue::from_str(cmd), args)?;
    let promise: js_sys::Promise = promise_js.unchecked_into();
    JsFuture::from(promise).await
}

/// Abre o dialog nativo de seleção de pasta via comando Tauri.
pub async fn open_folder_dialog() -> Result<Option<String>, String> {
    let args = JsValue::from(js_sys::Object::new());

    let result = tauri_invoke("open_vault_dialog", &args)
        .await
        .map_err(|e| format!("dialog error: {:?}", e))?;

    if result.is_null() || result.is_undefined() {
        return Ok(None);
    }

    let path = result
        .as_string()
        .ok_or_else(|| "caminho retornado não é string".to_string())?;
    Ok(Some(path))
}

/// Obtém informações do vault a partir do path.
pub async fn get_vault_info(path: &str) -> Result<VaultInfo, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("path"), &JsValue::from_str(path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("get_vault_info", &args)
        .await
        .map_err(|e| format!("get_vault_info error: {:?}", e))?;

    let info: VaultInfo =
        serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))?;
    Ok(info)
}

/// Lista todas as páginas `.md` do vault.
pub async fn list_pages(vault_path: &str) -> Result<Vec<PageMeta>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("list_pages", &args)
        .await
        .map_err(|e| format!("list_pages error: {:?}", e))?;

    let pages: Vec<PageMeta> =
        serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))?;
    Ok(pages)
}

/// Varredura única do vault (ciclo 150): metadados de TODAS as páginas
/// numa chamada só — frontmatter, properties `chave:: valor` do corpo,
/// tags e alvos de wikilink.
///
/// Use isto no lugar de `list_pages()` + `read_page()` em laço sempre
/// que a informação necessária for metadado: o laço faz uma travessia
/// WASM↔Tauri por página, carregando o arquivo inteiro em cada uma.
pub async fn scan_vault(vault_path: &str) -> Result<Vec<PageIndexEntry>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("scan_vault", &args)
        .await
        .map_err(|e| format!("scan_vault error: {:?}", e))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Lê o conteúdo bruto de uma página.
pub async fn read_page(vault_path: &str, page_path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("pagePath"),
        &JsValue::from_str(page_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("read_page", &args)
        .await
        .map_err(|e| format!("read_page error: {:?}", e))?;

    result
        .as_string()
        .ok_or_else(|| "conteúdo retornado não é string".to_string())
}

/// Exclui uma página do vault.
pub async fn delete_page(vault_path: &str, page_path: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("pagePath"),
        &JsValue::from_str(page_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    tauri_invoke("delete_page", &args)
        .await
        .map_err(|e| format!("delete_page error: {:?}", e))?;
    Ok(())
}

/// Cria uma pasta (subdiretório) sob `pages/`.
pub async fn create_folder(vault_path: &str, folder_path: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("folderPath"), &JsValue::from_str(folder_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    tauri_invoke("create_folder", &args).await.map_err(|e| format!("create_folder error: {:?}", e))?;
    Ok(())
}

/// Lista pastas (incluindo vazias) sob `pages/`.
pub async fn list_folders(vault_path: &str) -> Result<Vec<String>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("list_folders", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Move (renomeia) uma página — usado pra organizar em pastas.
pub async fn move_page(vault_path: &str, from_path: &str, to_path: &str) -> Result<PageMeta, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("fromPath"), &JsValue::from_str(from_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("toPath"), &JsValue::from_str(to_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("move_page", &args).await.map_err(|e| format!("move_page error: {:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Cria página dentro de uma pasta.
pub async fn create_page_in_folder(
    vault_path: &str, folder_path: &str, title: &str, page_type: &str,
) -> Result<PageMeta, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("folderPath"), &JsValue::from_str(folder_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("title"), &JsValue::from_str(title))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("pageType"), &JsValue::from_str(page_type))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("create_page_in_folder", &args)
        .await
        .map_err(|e| format!("create_page_in_folder error: {:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Lista arquivos modificados/não rastreados via `git status
/// --porcelain` (somente leitura). `None` se o vault não for um
/// repositório git ou `git` não estiver instalado — a UI deve tratar
/// isso como "não mostrar indicador", não como erro.
pub async fn git_status(vault_path: &str) -> Result<Option<Vec<GitFileEntry>>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("git_status", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Histórico de commits que tocaram uma página específica (ciclo 117,
/// somente leitura). `None` nas mesmas condições de `git_status`.
pub async fn git_log(vault_path: &str, page_path: &str) -> Result<Option<Vec<GitLogEntry>>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("pagePath"), &JsValue::from_str(page_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("git_log", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// `git pull` — ação explícita do usuário (ciclo 119). Retorna a
/// saída do git em sucesso, ou `Err` com a mensagem de erro (conflito,
/// sem remote, etc) tal qual.
pub async fn git_pull(vault_path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("git_pull", &args).await.map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;
    result.as_string().ok_or_else(|| "resposta inválida".to_string())
}

/// `git add -A && commit -m <message> && push` — ação explícita do
/// usuário (ciclo 119).
pub async fn git_commit_and_push(vault_path: &str, message: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("message"), &JsValue::from_str(message))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("git_commit_and_push", &args).await.map_err(|e| e.as_string().unwrap_or_else(|| format!("{:?}", e)))?;
    result.as_string().ok_or_else(|| "resposta inválida".to_string())
}

/// Concatena o markdown fonte de todas as páginas dentro de uma pasta
/// (recursivo) num dump único. `folder_path` vazio exporta o vault
/// inteiro (`pages/` + `journals/`).
pub async fn export_folder(vault_path: &str, folder_path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("folderPath"), &JsValue::from_str(folder_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("export_folder", &args)
        .await
        .map_err(|e| format!("export_folder error: {:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Lista templates em `templates/`.
pub async fn list_templates(vault_path: &str) -> Result<Vec<PageMeta>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("list_templates", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Cria página a partir de um template, substituindo `{{title}}`.
/// `folder_path` escolhe a pasta de destino (`None` = `pages/`) — usado
/// pelo embed de ações (ciclo 156) pra criar spec/decisão já na pasta
/// certa do esquema de agent-os.
pub async fn create_page_from_template(
    vault_path: &str, template_path: &str, title: &str, folder_path: Option<&str>,
) -> Result<PageMeta, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("templatePath"), &JsValue::from_str(template_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("title"), &JsValue::from_str(title))
        .map_err(|e| format!("{:?}", e))?;
    if let Some(folder) = folder_path {
        js_sys::Reflect::set(&args, &JsValue::from_str("folderPath"), &JsValue::from_str(folder))
            .map_err(|e| format!("{:?}", e))?;
    }
    let args = JsValue::from(args);
    let result = tauri_invoke("create_page_from_template", &args)
        .await
        .map_err(|e| format!("create_page_from_template error: {:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Lista arquivos no diretório assets/.
pub async fn list_assets(vault_path: &str) -> Result<Vec<String>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("list_assets", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Copia um arquivo para assets/ e retorna o path relativo.
pub async fn copy_to_assets(vault_path: &str, source_path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("sourcePath"), &JsValue::from_str(source_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("copy_to_assets", &args).await.map_err(|e| format!("{:?}", e))?;
    result.as_string().ok_or_else(|| "path inválido".to_string())
}

/// Grava bytes (já em base64) em `assets/` com nome único — usado pelo
/// paste de imagem no editor (ciclo 118), sem arquivo de origem no
/// disco. `extension` sem o ponto (ex: `"png"`). Retorna o path
/// relativo do asset criado.
pub async fn save_pasted_asset(vault_path: &str, extension: &str, base64_data: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("extension"), &JsValue::from_str(extension))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("base64Data"), &JsValue::from_str(base64_data))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("save_pasted_asset", &args).await.map_err(|e| format!("{:?}", e))?;
    result.as_string().ok_or_else(|| "path inválido".to_string())
}

/// Lê um arquivo do vault (ex: `assets/x.png`, `assets/x.pdf`) como
/// uma `data:` URL (ciclo 121) — necessário porque um `src`/`href`
/// relativo cru resolve contra a origem do webview, não contra a
/// pasta real do vault no disco.
pub async fn read_asset_data_url(vault_path: &str, asset_path: &str) -> Result<String, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("assetPath"), &JsValue::from_str(asset_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("read_asset_data_url", &args).await.map_err(|e| format!("{:?}", e))?;
    result.as_string().ok_or_else(|| "resposta inválida".to_string())
}

/// Metadados de um arquivo em `assets/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Path relativo ao vault.
    pub path: String,
    /// Tamanho em bytes.
    pub size: u64,
}

/// Lista arquivos em assets/ com tamanho.
pub async fn list_assets_info(vault_path: &str) -> Result<Vec<AssetInfo>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("list_assets_info", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Remove um arquivo de assets/.
pub async fn delete_asset(vault_path: &str, asset_path: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("assetPath"), &JsValue::from_str(asset_path))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    tauri_invoke("delete_asset", &args).await.map_err(|e| format!("{:?}", e))?;
    Ok(())
}

/// Busca texto no conteúdo de todas as páginas.
pub async fn search_content(vault_path: &str, query: &str) -> Result<Vec<(String, String)>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("query"), &JsValue::from_str(query))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);
    let result = tauri_invoke("search_content", &args).await.map_err(|e| format!("{:?}", e))?;
    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Verifica se houve mudanças no vault desde a última verificação.
pub async fn check_changes(vault_path: &str) -> Result<bool, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("check_changes", &args)
        .await
        .map_err(|e| format!("check_changes error: {:?}", e))?;

    Ok(result.as_bool().unwrap_or(false))
}

/// Abre ou cria o journal do dia.
pub async fn open_today_journal(vault_path: &str) -> Result<PageMeta, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("open_today_journal", &args)
        .await
        .map_err(|e| format!("open_today_journal error: {:?}", e))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Cria uma nova página em pages/.
pub async fn create_page(vault_path: &str, title: &str) -> Result<PageMeta, String> {
    create_page_with_type(vault_path, title, "md").await
}

/// Cria pagina com tipo especifico (md, kanban, calendar, table).
pub async fn create_page_with_type(
    vault_path: &str, title: &str, page_type: &str,
) -> Result<PageMeta, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(&args, &JsValue::from_str("vaultPath"), &JsValue::from_str(vault_path))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("title"), &JsValue::from_str(title))
        .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(&args, &JsValue::from_str("pageType"), &JsValue::from_str(page_type))
        .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("create_page_with_type", &args)
        .await
        .map_err(|e| format!("create_page_with_type error: {:?}", e))?;

    serde_wasm_bindgen::from_value(result).map_err(|e| format!("deserialize: {}", e))
}

/// Grava o conteúdo de uma página no disco.
pub async fn write_page(vault_path: &str, page_path: &str, content: &str) -> Result<(), String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("pagePath"),
        &JsValue::from_str(page_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("content"),
        &JsValue::from_str(content),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    tauri_invoke("write_page", &args)
        .await
        .map_err(|e| format!("write_page error: {:?}", e))?;
    Ok(())
}
