//! Anotadinho UI: Yew frontend.
//!
//! Este crate compila pra WASM e roda dentro do WebView do Tauri.
//! É o frontend visual do app.

#![warn(missing_docs)]

pub mod api;
pub mod app;
pub mod components;
pub mod html_to_md;
pub mod markdown_render;
pub mod state;
pub mod theme;

pub use app::App;
