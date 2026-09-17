//! Anotadinho core: block model, Markdown parser, properties.
//!
//! Este crate é o coração do Anotadinho. Define o modelo de blocos,
//! parser de Markdown com suporte a block IDs e properties inline,
//! e operações sobre o grafo de páginas.
//!
//! Implementação concreta virá nos próximos ciclos.

#![warn(missing_docs)]

pub mod block;
pub mod edicao;
pub mod inline;
pub mod unidade;
pub mod roteamento;
pub mod render;
pub mod espacial;
pub mod analise;
pub mod navegacao;
pub mod page;
pub mod permissoes;
pub mod property;
pub mod markdown;
// Movidos da UI no ciclo 149: são lógica pura e o `anotadinho-cli`
// precisa deles pra operar embeds sem passar por WASM.
pub mod date_util;
pub mod embed;
pub mod exportar;
pub mod tabela_md;
pub mod transclusao;
// A grade mensal do calendário saiu da UI no ciclo 306: o terminal
// desenha a mesma grade, e o algoritmo de faixas precisa ser um só.
pub mod calendario;
pub mod agente;
pub mod conversa;
pub mod decisao;
pub mod diff;
pub mod execucao;
pub mod ferramentas;
pub mod avaliacao;
pub mod gatilho;
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
pub mod vim;

pub use block::{Block, BlockId, BlockKind};
pub use markdown::MarkdownCodec;
pub use page::{Frontmatter, Page, PageId};
pub use property::Property;
pub use index::PageIndexEntry;
pub use inserted_image::{ImageAlignment, InsertedImage};
pub use query::Query;
