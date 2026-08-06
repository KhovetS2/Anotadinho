//! Page model: uma página é um arquivo .md com frontmatter + blocos.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::block::Block;

/// Identificador único de uma página (path do arquivo).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PageId(pub String);

impl PageId {
    /// Cria um PageId a partir do path relativo ao vault.
    pub fn from_path(path: impl Into<String>) -> Self {
        Self(path.into())
    }
}

impl std::fmt::Display for PageId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Frontmatter YAML no topo de uma página.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Título da página.
    pub title: Option<String>,
    /// Tags (lista).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Data de criação (ISO 8601).
    pub created: Option<chrono::DateTime<chrono::Utc>>,
    /// Data de última atualização.
    pub updated: Option<chrono::DateTime<chrono::Utc>>,
    /// Tipo de página: "md" (default), "kanban", "calendar", "table".
    #[serde(rename = "type", default)]
    pub page_type: Option<String>,
}

impl Frontmatter {
    /// Retorna o tipo efetivo da página (md se não definido).
    pub fn effective_type(&self) -> &str {
        self.page_type.as_deref().unwrap_or("md")
    }
}

/// Uma página do vault: frontmatter + lista ordenada de blocos.
#[derive(Debug, Clone)]
pub struct Page {
    /// ID da página (path).
    pub id: PageId,
    /// Frontmatter.
    pub frontmatter: Frontmatter,
    /// Blocos em ordem.
    pub blocks: Vec<Block>,
}

impl Page {
    /// Cria uma página vazia com ID e título.
    pub fn new(id: PageId, title: impl Into<String>) -> Self {
        Self {
            id,
            frontmatter: Frontmatter {
                title: Some(title.into()),
                ..Default::default()
            },
            blocks: Vec::new(),
        }
    }
}

/// Gera um ID único (helper usado em operações internas).
pub fn new_page_uuid() -> Uuid {
    Uuid::new_v4()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_new_has_title() {
        let p = Page::new(PageId::from_path("pages/test.md"), "Test");
        assert_eq!(p.frontmatter.title.as_deref(), Some("Test"));
        assert!(p.blocks.is_empty());
    }
}
