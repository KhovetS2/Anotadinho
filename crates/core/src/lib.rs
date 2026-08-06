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

pub use block::{Block, BlockId, BlockKind};
pub use markdown::MarkdownCodec;
pub use page::{Frontmatter, Page, PageId};
pub use property::Property;
