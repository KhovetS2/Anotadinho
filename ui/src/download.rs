//! Dispara o download de um arquivo de texto no browser — Blob +
//! `<a download>` sintético, sem passar pelo backend (não precisa
//! escrever nada em disco fora do vault).

use wasm_bindgen::{JsCast, JsValue};

/// Dispara o download de `content` como um arquivo `filename`.
pub fn download_text_file(filename: &str, mime: &str, content: &str) {
    let Some(window) = web_sys::window() else { return };
    let Some(document) = window.document() else { return };

    let arr = js_sys::Array::new();
    arr.push(&JsValue::from_str(content));
    let mut opts = web_sys::BlobPropertyBag::new();
    opts.type_(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&arr, &opts) else { return };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else { return };

    let Ok(anchor) = document.create_element("a") else { return };
    let Ok(anchor) = anchor.dyn_into::<web_sys::HtmlAnchorElement>() else { return };
    anchor.set_href(&url);
    anchor.set_download(filename);
    anchor.click();
    let _ = web_sys::Url::revoke_object_url(&url);
}
