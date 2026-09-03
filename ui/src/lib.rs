//! Anotadinho UI: Yew frontend.
//!
//! Este crate compila pra WASM e roda dentro do WebView do Tauri.
//! É o frontend visual do app.

#![warn(missing_docs)]

pub mod api;
pub mod app;
pub mod components;
pub mod date_util;
pub mod dialog;
pub mod download;
pub mod embed;
pub mod html_to_md;
pub mod keyboard_activate;
pub mod markdown_render;
pub mod menu_keyboard;
pub mod nav_mode;
pub mod selecao_blocos;
/// Motor de consulta do vault — reexportado do `anotadinho-core` (mora
/// lá pro `anotadinho-cli` executar a mesma consulta que o embed
/// mostra, sem passar por WASM).
pub mod query {
    pub use anotadinho_core::query::*;
}
pub mod state;
pub mod wikilink;

pub use app::App;
