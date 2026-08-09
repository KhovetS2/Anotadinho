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
pub mod state;
pub mod wikilink;

pub use app::App;
