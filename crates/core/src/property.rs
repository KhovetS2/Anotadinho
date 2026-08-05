//! Properties inline (estilo Logseq).
//!
//! Formato: `key:: value` numa linha de bloco.

use serde::{Deserialize, Serialize};

/// Uma property `chave:: valor`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Property {
    /// Chave (sem o `::`).
    pub key: String,
    /// Valor.
    pub value: String,
}

impl Property {
    /// Cria uma property nova.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }

    /// Formata como `key:: value`.
    pub fn to_inline(&self) -> String {
        format!("{}:: {}", self.key, self.value)
    }

    /// Parseia uma linha no formato `key:: value`.
    pub fn parse(line: &str) -> Option<Self> {
        let idx = line.find("::")?;
        let key = line[..idx].trim();
        let value = line[idx + 2..].trim();
        if key.is_empty() || value.is_empty() {
            return None;
        }
        // Chave não pode ter espaços (estilo Logseq)
        if key.contains(char::is_whitespace) {
            return None;
        }
        Some(Self {
            key: key.to_string(),
            value: value.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_simple() {
        let p = Property::parse("status:: em-andamento").unwrap();
        assert_eq!(p.key, "status");
        assert_eq!(p.value, "em-andamento");
    }

    #[test]
    fn parse_with_spaces_in_value() {
        let p = Property::parse("titulo:: Minha nota importante").unwrap();
        assert_eq!(p.value, "Minha nota importante");
    }

    #[test]
    fn reject_empty_value() {
        assert!(Property::parse("key::").is_none());
        assert!(Property::parse(":: value").is_none());
    }

    #[test]
    fn reject_space_in_key() {
        assert!(Property::parse("my key:: value").is_none());
    }

    #[test]
    fn to_inline_roundtrip() {
        let p = Property::new("status", "done");
        assert_eq!(p.to_inline(), "status:: done");
    }
}
