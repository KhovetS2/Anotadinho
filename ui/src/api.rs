//! Ponte IPC entre o Yew (WASM) e o backend Tauri.
//!
//! Fornece funções async que chamam comandos Tauri via
//! `window.__TAURI_INTERNALS__.invoke()`.

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
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("vaultPath"),
        &JsValue::from_str(vault_path),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("title"),
        &JsValue::from_str(title),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("create_page", &args)
        .await
        .map_err(|e| format!("create_page error: {:?}", e))?;

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
