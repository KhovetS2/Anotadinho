//! Block model: unidade fundamental do Anotadinho.
//!
//! Um bloco é uma linha ou grupo de linhas de Markdown com um ID
//! único e properties opcionais.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Identificador único e estável de um bloco.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockId(pub Uuid);

impl BlockId {
    /// Gera um novo BlockId aleatório (v4).
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for BlockId {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Display for BlockId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Tipo semântico de um bloco (nota, tarefa, heading, etc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BlockKind {
    /// Bloco de texto livre.
    #[default]
    Note,
    /// Tarefa com status.
    Task,
    /// Heading (h1-h6).
    Heading(u8),
    /// Citação.
    Quote,
    /// Código.
    Code,
    /// Lista.
    List,
    /// Tipo custom (definido por property `tipo::`).
    Custom,
}

/// Um bloco do vault.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    /// ID único.
    pub id: BlockId,
    /// Conteúdo textual (sem `id::` nem properties).
    pub content: String,
    /// Tipo semântico inferido.
    pub kind: BlockKind,
    /// Properties inline (ex: `status:: em-andamento`).
    pub properties: Vec<(String, String)>,
    /// Profundidade de indentação (0 = raiz).
    pub depth: u8,
}

impl Block {
    /// Cria um bloco novo com ID gerado.
    pub fn new(content: impl Into<String>) -> Self {
        Self {
            id: BlockId::new(),
            content: content.into(),
            kind: BlockKind::Note,
            properties: Vec::new(),
            depth: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_id_is_unique() {
        let a = BlockId::new();
        let b = BlockId::new();
        assert_ne!(a, b);
    }

    #[test]
    fn block_new_has_id() {
        let b = Block::new("hello");
        assert_eq!(b.content, "hello");
        assert_eq!(b.kind, BlockKind::Note);
        assert_eq!(b.depth, 0);
    }
}
