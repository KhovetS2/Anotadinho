//! Anotadinho UI: Yew frontend.
//!
//! Este crate compila pra WASM e roda dentro do WebView do Tauri.
//! É o frontend visual do app.
//!
//! Por enquanto, apenas um esqueleto com tema dark.
//! Features concretas virão nos próximos ciclos.

#![warn(missing_docs)]

pub mod app;
pub mod state;
pub mod theme;
pub mod components;

pub use app::App;
