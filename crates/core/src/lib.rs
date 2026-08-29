//! Anotadinho core: block model, Markdown parser, properties.
//!
//! Este crate é o coração do Anotadinho. Define o modelo de blocos,
//! parser de Markdown com suporte a block IDs e properties inline,
//! e operações sobre o grafo de páginas.
//!
//! Implementação concreta virá nos próximos ciclos.

#![warn(missing_docs)]

pub mod block;
pub mod page;
pub mod property;
pub mod markdown;
// Movidos da UI no ciclo 149: são lógica pura e o `anotadinho-cli`
// precisa deles pra operar embeds sem passar por WASM.
pub mod date_util;
pub mod embed;
pub mod agente;
pub mod conversa;
pub mod diff;
pub mod fluxo;
pub mod sanitize;
pub mod semente;
pub mod history;
pub mod index;
pub mod inserted_image;
pub mod links;
pub mod proposta;
pub mod prompt_padrao;
pub mod query;

pub use block::{Block, BlockId, BlockKind};
pub use markdown::MarkdownCodec;
pub use page::{Frontmatter, Page, PageId};
pub use property::Property;
pub use index::PageIndexEntry;
pub use inserted_image::{ImageAlignment, InsertedImage};
pub use query::Query;
