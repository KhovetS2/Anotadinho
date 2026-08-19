//! Ponte da UI pra `anotadinho_core::embed` + as varreduras do vault que
//! dependem de IPC.
//!
//! O parsing/serialização de embed (`segment`, `join`, `EmbedKind`,
//! `EmbedData` e todas as structs de dados) foi pro `core` no ciclo 149,
//! pro `anotadinho-cli` alcançar. O que sobra aqui precisa de
//! `crate::api` (ponte WASM↔Tauri), que não existe fora do navegador.
//! Todo mundo continua importando `crate::embed::*` como antes.

pub use anotadinho_core::embed::*;

/// Escaneia o vault inteiro por páginas com `date::` (e opcionalmente
/// `time::`) no frontmatter — mesma fonte de dados da página inteira
/// `type: calendar` (`components/calendar.rs`), reaproveitada aqui pra
/// alimentar o embed em modo Vault. Cada página com `date::` vira uma
/// `CalendarEntry` sintética com `page_path` preenchido (nunca
/// persistida — recalculada toda vez que o modo Vault é exibido).
pub async fn scan_vault_calendar_entries(vault_path: &str) -> Vec<CalendarEntry> {
    let mut out = Vec::new();
    let Ok(pages) = crate::api::list_pages(vault_path).await else {
        return out;
    };
    for page in &pages {
        let Ok(content) = crate::api::read_page(vault_path, &page.path).await else {
            continue;
        };
        let mut date: Option<String> = None;
        let mut time: Option<String> = None;
        for line in content.lines() {
            let t = line.trim();
            if let Some(v) = t.strip_prefix("date:: ") {
                date = Some(v.trim().to_string());
            } else if let Some(v) = t.strip_prefix("time:: ") {
                time = Some(v.trim().to_string());
            }
        }
        if let Some(date) = date {
            out.push(CalendarEntry {
                date: Some(date),
                title: page.title.clone(),
                end_date: None,
                tags: Vec::new(),
                legacy_tag: None,
                start_time: time,
                end_time: None,
                page_path: Some(page.path.clone()),
            });
        }
    }
    out
}

/// Escaneia o vault inteiro por tags usadas em embeds inline (cards de
/// kanban, eventos de calendário) e agrega por tag → páginas onde
/// aparece. Usado pela página `type: tags`. Tabelas (colunas
/// Select/MultiSelect) ficam de fora nesta v1 — ver Não-objetivos do
/// ciclo que introduziu isso.
pub async fn scan_vault_tags(vault_path: &str) -> std::collections::BTreeMap<String, Vec<(String, String)>> {
    let mut out: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    let Ok(pages) = crate::api::list_pages(vault_path).await else {
        return out;
    };
    for page in &pages {
        let Ok(content) = crate::api::read_page(vault_path, &page.path).await else {
            continue;
        };
        let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&content);
        let segments = segment(body);
        let mut page_tags: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        for seg in &segments {
            if let DocSegment::Embed(data) = seg {
                match data {
                    EmbedData::Kanban(k) => {
                        for card in &k.items {
                            for t in &card.tags {
                                page_tags.insert(t.clone());
                            }
                        }
                    }
                    EmbedData::Calendar(c) => {
                        for entry in &c.entries {
                            for t in entry.all_tags() {
                                page_tags.insert(t);
                            }
                        }
                    }
                    EmbedData::Table(_) => {}
                }
            }
        }
        for tag in page_tags {
            out.entry(tag).or_default().push((page.path.clone(), page.title.clone()));
        }
    }
    out
}
