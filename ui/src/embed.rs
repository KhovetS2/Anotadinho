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
/// `time::`) — mesma fonte de dados da página inteira `type: calendar`
/// (`components/calendar.rs`), reaproveitada aqui pra alimentar o embed
/// em modo Vault. Cada página com `date::` vira uma `CalendarEntry`
/// sintética com `page_path` preenchido (nunca persistida — recalculada
/// toda vez que o modo Vault é exibido).
///
/// Uma chamada de IPC só desde o ciclo 150 (`api::scan_vault`): antes
/// lia o vault inteiro arquivo por arquivo pra procurar duas linhas.
pub async fn scan_vault_calendar_entries(vault_path: &str) -> Vec<CalendarEntry> {
    let Ok(pages) = crate::api::scan_vault(vault_path).await else {
        return Vec::new();
    };
    pages
        .iter()
        .filter_map(|page| {
            let date = page.properties.get("date")?.clone();
            Some(CalendarEntry {
                date: Some(date),
                title: page.title.clone(),
                end_date: page.properties.get("end_date").cloned(),
                tags: Vec::new(),
                legacy_tag: None,
                start_time: page.properties.get("time").cloned(),
                end_time: None,
                page_path: Some(page.path.clone()),
            })
        })
        .collect()
}

/// Escaneia o vault inteiro por tags usadas em embeds inline (cards de
/// kanban, eventos de calendário) e agrega por tag → páginas onde
/// aparece. Usado pela página `type: tags`. Tabelas (colunas
/// Select/MultiSelect) ficam de fora nesta v1 — ver Não-objetivos do
/// ciclo que introduziu isso.
///
/// O parse dos embeds acontece no backend, dentro da varredura
/// (`PageIndexEntry::embed_tags`) — aqui só resta agregar.
pub async fn scan_vault_tags(vault_path: &str) -> std::collections::BTreeMap<String, Vec<(String, String)>> {
    let mut out: std::collections::BTreeMap<String, Vec<(String, String)>> = std::collections::BTreeMap::new();
    let Ok(pages) = crate::api::scan_vault(vault_path).await else {
        return out;
    };
    for page in &pages {
        for tag in &page.embed_tags {
            out.entry(tag.clone())
                .or_default()
                .push((page.path.clone(), page.title.clone()));
        }
    }
    out
}
