//! Ponte IPC entre o Yew (WASM) e o backend Tauri.
//!
//! Fornece funções async que chamam comandos Tauri via
//! `window.__TAURI_INTERNALS__.invoke()`.

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

/// Abre o dialog nativo de seleção de pasta (Tauri dialog plugin).
///
/// Retorna `Some(path)` se o usuário selecionou uma pasta, ou
/// `None` se cancelou.
pub async fn open_folder_dialog() -> Result<Option<String>, String> {
    let args = js_sys::Object::new();
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("directory"),
        &JsValue::from_bool(true),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("multiple"),
        &JsValue::from_bool(false),
    )
    .map_err(|e| format!("{:?}", e))?;
    js_sys::Reflect::set(
        &args,
        &JsValue::from_str("title"),
        &JsValue::from_str("Selecione a pasta do vault"),
    )
    .map_err(|e| format!("{:?}", e))?;
    let args = JsValue::from(args);

    let result = tauri_invoke("plugin:dialog|open", &args)
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
