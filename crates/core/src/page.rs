//! Page model: uma página é um arquivo .md com frontmatter + blocos.

use std::collections::BTreeMap;

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
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct Frontmatter {
    /// Título da página.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Tags (lista).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tags: Vec<String>,
    /// Data de criação. Formato livre (o vault usa `YYYY-MM-DD` sem horário,
    /// que não é um `DateTime` RFC3339 válido) — mantido como string pra não
    /// quebrar o parse de frontmatter de páginas reais do vault.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,
    /// Data de última atualização. Mesmo formato livre de `created`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,
    /// Tipo de página: "md" (default), "kanban", "calendar", "table".
    #[serde(rename = "type", default, skip_serializing_if = "Option::is_none")]
    pub page_type: Option<String>,
    /// Qualquer propriedade YAML além das reconhecidas acima — ex:
    /// `status:: doing`, `owner:: elis`, `spec-id:: 42`. Sem isso,
    /// `serde_yaml::from_str` nesta struct simplesmente IGNORA (não dá
    /// erro, só descarta) qualquer chave desconhecida, então uma página
    /// que passasse pelo caminho de round-trip TIPADO
    /// (`MarkdownCodec::serialize`, hoje sem nenhum caller fora dos
    /// testes do próprio crate — a UI usa `split_frontmatter_text`, que
    /// preserva o texto cru e por isso nunca perdeu nada) perderia essas
    /// propriedades. Necessário pro painel de propriedades (ciclo 099)
    /// ter um modelo de dados genérico pra ler/escrever QUALQUER
    /// propriedade, não só as 5 fixas.
    #[serde(flatten)]
    pub extra: BTreeMap<String, serde_yaml::Value>,
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

    #[test]
    fn frontmatter_extra_captures_unknown_keys() {
        let yaml = "title: Spec\nstatus: draft\nowner: elis\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(fm.title.as_deref(), Some("Spec"));
        assert_eq!(fm.extra.get("status").and_then(|v| v.as_str()), Some("draft"));
        assert_eq!(fm.extra.get("owner").and_then(|v| v.as_str()), Some("elis"));
        // campos conhecidos não vazam pra `extra`
        assert!(!fm.extra.contains_key("title"));
    }

    #[test]
    fn frontmatter_extra_roundtrips_through_serialize() {
        let yaml = "title: Spec\nstatus: draft\npriority: 2\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        let out = serde_yaml::to_string(&fm).unwrap();
        let fm2: Frontmatter = serde_yaml::from_str(&out).unwrap();
        assert_eq!(fm2.title, fm.title);
        assert_eq!(fm2.extra, fm.extra);
        assert_eq!(fm2.extra.get("priority").and_then(|v| v.as_i64()), Some(2));
    }

    #[test]
    fn frontmatter_without_extra_keys_has_empty_map() {
        let yaml = "title: Simples\ntags: [a, b]\n";
        let fm: Frontmatter = serde_yaml::from_str(yaml).unwrap();
        assert!(fm.extra.is_empty());
    }
}
