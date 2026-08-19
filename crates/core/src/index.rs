//! Índice do vault: metadados de todas as páginas numa estrutura só.
//!
//! Antes deste módulo (ciclo 150), todo consumidor que precisava olhar o
//! vault inteiro — grafo de backlinks, calendário em modo vault, página
//! de tags — fazia `list_pages()` e depois um `read_page()` POR PÁGINA,
//! cada um atravessando a ponte WASM↔Tauri com o conteúdo completo do
//! arquivo. Num vault de 200 páginas isso é 201 round-trips só pra
//! desenhar um grafo.
//!
//! Aqui a leitura acontece uma vez, no backend, e atravessa a ponte já
//! reduzida ao que interessa: frontmatter, properties, tags e alvos de
//! wikilink.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::markdown::MarkdownCodec;
use crate::property::Property;

/// Metadados de uma página, prontos pra filtrar/agrupar sem reler o
/// arquivo.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct PageIndexEntry {
    /// Path relativo ao vault (ex: `pages/specs/minha-spec.md`).
    pub path: String,
    /// Título da página (frontmatter `title`, ou nome do arquivo).
    pub title: String,
    /// Seção (`pages` ou `journals`).
    pub section: String,
    /// Tipo efetivo (`md`, `kanban`, `calendar`, `table`, `graph`,
    /// `tags`, `assets`, `landing`).
    pub page_type: String,
    /// Tags do frontmatter.
    pub tags: Vec<String>,
    /// Frontmatter YAML + properties `chave:: valor` do corpo, achatados
    /// em texto. O vault usa as duas formas (o esquema de agent-os
    /// escreve YAML; o calendário lê `date::` do corpo), e quem consulta
    /// não deveria precisar saber de qual das duas o valor veio.
    /// Em caso de conflito o YAML ganha — é o que o painel de
    /// propriedades edita.
    pub properties: BTreeMap<String, String>,
    /// Alvos de `[[wikilink]]` no corpo, únicos e sem alias/âncora.
    pub wikilinks: Vec<String>,
    /// Tags usadas DENTRO dos embeds inline da página (labels de card de
    /// kanban, tags de evento de calendário), únicas e ordenadas. Ficam
    /// aqui porque a página de tags precisava delas e era o último
    /// consumidor a reler o vault inteiro página por página só pra
    /// parsear embed.
    pub embed_tags: Vec<String>,
}

impl PageIndexEntry {
    /// Monta a entrada a partir do conteúdo cru de uma página.
    /// `fallback_title` é o título vindo da listagem (nome do arquivo),
    /// usado quando o frontmatter não traz `title`.
    pub fn from_content(path: &str, fallback_title: &str, section: &str, content: &str) -> Self {
        let frontmatter = MarkdownCodec::split_frontmatter(content)
            .map(|(fm, _)| fm)
            .unwrap_or_default();
        let (_, body) = MarkdownCodec::split_frontmatter_text(content);

        let mut properties = BTreeMap::new();
        // Properties do corpo primeiro: o YAML sobrescreve em caso de
        // conflito (ver doc do campo).
        for line in body.lines() {
            if let Some(p) = Property::parse(line.trim()) {
                properties.insert(p.key, p.value);
            }
        }
        for (key, value) in &frontmatter.extra {
            properties.insert(key.clone(), yaml_to_string(value));
        }
        if let Some(created) = &frontmatter.created {
            properties.insert("created".to_string(), created.clone());
        }
        if let Some(updated) = &frontmatter.updated {
            properties.insert("updated".to_string(), updated.clone());
        }

        let title = frontmatter
            .title
            .clone()
            .filter(|t| !t.trim().is_empty())
            .unwrap_or_else(|| fallback_title.to_string());

        Self {
            path: path.to_string(),
            title,
            section: section.to_string(),
            page_type: frontmatter.effective_type().to_string(),
            tags: frontmatter.tags.clone(),
            properties,
            wikilinks: crate::links::extract_wikilink_targets(body),
            embed_tags: collect_embed_tags(body),
        }
    }

    /// Valor de um campo pelo nome, unificando os campos fixos e as
    /// properties. É o acesso que o motor de consulta usa pra não
    /// precisar saber se `status` é YAML, `key:: value` ou coluna
    /// interna.
    pub fn field(&self, name: &str) -> Option<String> {
        match name {
            "path" => Some(self.path.clone()),
            "title" => Some(self.title.clone()),
            "section" => Some(self.section.clone()),
            "type" => Some(self.page_type.clone()),
            "tags" => Some(self.tags.join(", ")),
            other => self.properties.get(other).cloned(),
        }
    }
}

/// Tags de dentro dos embeds inline do corpo. Tabela fica de fora (as
/// colunas Select/MultiSelect não são "tags da página" — ver
/// Não-objetivos do ciclo que introduziu a página de tags).
fn collect_embed_tags(body: &str) -> Vec<String> {
    let mut out: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    for seg in crate::embed::segment(body) {
        let crate::embed::DocSegment::Embed(data) = seg else { continue };
        match data {
            crate::embed::EmbedData::Kanban(k) => {
                for card in &k.items {
                    out.extend(card.tags.iter().cloned());
                }
            }
            crate::embed::EmbedData::Calendar(c) => {
                for entry in &c.entries {
                    out.extend(entry.all_tags());
                }
            }
            crate::embed::EmbedData::Table(_) | crate::embed::EmbedData::Callout(_) => {}
        }
    }
    out.into_iter().collect()
}

/// Achata um valor YAML em texto: escalar vira ele mesmo, sequência vira
/// os itens separados por `, `, mapa vira as chaves. Perde estrutura de
/// propósito — quem consulta compara texto.
fn yaml_to_string(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Number(n) => n.to_string(),
        serde_yaml::Value::Bool(b) => b.to_string(),
        serde_yaml::Value::Null => String::new(),
        serde_yaml::Value::Sequence(items) => items
            .iter()
            .map(yaml_to_string)
            .collect::<Vec<_>>()
            .join(", "),
        serde_yaml::Value::Mapping(map) => map
            .keys()
            .map(yaml_to_string)
            .collect::<Vec<_>>()
            .join(", "),
        serde_yaml::Value::Tagged(t) => yaml_to_string(&t.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_frontmatter_tipado_e_extra() {
        let content = "---\ntitle: Minha Spec\ntype: md\ntags: [spec, api]\nstatus: backlog\npriority: alta\n---\n\ncorpo\n";
        let e = PageIndexEntry::from_content("pages/specs/x.md", "x", "pages", content);
        assert_eq!(e.title, "Minha Spec");
        assert_eq!(e.tags, vec!["spec", "api"]);
        assert_eq!(e.field("status").as_deref(), Some("backlog"));
        assert_eq!(e.field("priority").as_deref(), Some("alta"));
        assert_eq!(e.field("type").as_deref(), Some("md"));
    }

    #[test]
    fn le_properties_do_corpo() {
        let content = "---\ntitle: Reunião\n---\n\ndate:: 2026-08-19\ntime:: 14:30\n";
        let e = PageIndexEntry::from_content("pages/r.md", "r", "pages", content);
        assert_eq!(e.field("date").as_deref(), Some("2026-08-19"));
        assert_eq!(e.field("time").as_deref(), Some("14:30"));
    }

    #[test]
    fn yaml_ganha_de_property_do_corpo_no_conflito() {
        let content = "---\nstatus: done\n---\n\nstatus:: backlog\n";
        let e = PageIndexEntry::from_content("pages/x.md", "x", "pages", content);
        assert_eq!(e.field("status").as_deref(), Some("done"));
    }

    #[test]
    fn titulo_cai_pro_nome_do_arquivo_sem_frontmatter() {
        let e = PageIndexEntry::from_content("pages/sem-fm.md", "sem-fm", "pages", "só corpo\n");
        assert_eq!(e.title, "sem-fm");
        assert_eq!(e.page_type, "md");
        assert!(e.tags.is_empty());
    }

    #[test]
    fn frontmatter_invalido_nao_derruba_a_entrada() {
        let content = "---\ntitle: [ isto: nao fecha\n---\n\n[[Missão]]\n";
        let e = PageIndexEntry::from_content("pages/x.md", "x", "pages", content);
        assert_eq!(e.title, "x");
        assert_eq!(e.wikilinks, vec!["Missão"]);
    }

    #[test]
    fn coleta_wikilinks_unicos_do_corpo() {
        let content = "---\ntitle: T\n---\n\n[[A]] [[A|alias]] [[B#sec]]\n";
        let e = PageIndexEntry::from_content("pages/t.md", "t", "pages", content);
        assert_eq!(e.wikilinks, vec!["A", "B"]);
    }

    #[test]
    fn coleta_tags_de_dentro_dos_embeds() {
        let content = concat!(
            "---\ntitle: T\n---\n\n",
            "{{ type: \"kanban\" }}\ncolumns:\n- Todo\nitems:\n- title: C\n  column: Todo\n  tags: [urgente]\n{{ /kanban }}\n\n",
            "{{ type: \"calendar\" }}\nentries:\n- date: '2026-08-19'\n  title: E\n  tags: [reuniao]\n{{ /calendar }}\n",
        );
        let e = PageIndexEntry::from_content("pages/t.md", "t", "pages", content);
        assert_eq!(e.embed_tags, vec!["reuniao", "urgente"]);
    }

    #[test]
    fn field_de_campo_inexistente_e_none() {
        let e = PageIndexEntry::from_content("pages/t.md", "t", "pages", "corpo");
        assert!(e.field("inexistente").is_none());
    }
}
