//! Markdown parser/serializer com suporte a block IDs e properties.
//!
//! Implementação concreta virá no ciclo 006 (parser MD básico).
//! Por enquanto, apenas o esqueleto.

use anyhow::Result;

use crate::page::Page;

/// Parser/serializer de Markdown para o modelo de página.
pub struct MarkdownCodec;

impl MarkdownCodec {
    /// Converte texto Markdown em uma `Page`.
    ///
    /// TODO(ciclo 006): implementar parsing real com block IDs e properties.
    pub fn parse(_text: &str) -> Result<Page> {
        anyhow::bail!("MarkdownCodec::parse ainda nao implementado (vai no ciclo 006)")
    }

    /// Converte uma `Page` de volta em texto Markdown.
    ///
    /// TODO(ciclo 006): implementar serialização real.
    pub fn serialize(_page: &Page) -> Result<String> {
        anyhow::bail!("MarkdownCodec::serialize ainda nao implementado (vai no ciclo 006)")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_stub_returns_error() {
        let r = MarkdownCodec::parse("# hello");
        assert!(r.is_err());
    }
}
