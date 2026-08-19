---
id: "149"
titulo: "Move parsing de embed para anotadinho-core"
status: done
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["148"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Move parsing de embed para anotadinho-core

## Objetivo

`ui/src/embed.rs` (1820 linhas, 49 testes) é onde vive TODO o
conhecimento sobre o formato `{{ type: "X" }} ... {{ /X }}` — mas ele
está no crate WASM, então o `anotadinho-cli` (canal do agente
headless) não alcança nada disso. Hoje um agente que queira mexer num
board só tem a opção de reescrever o `.md` na mão, montando YAML por
concatenação de string — exatamente a origem do bug de corrupção do
ciclo 064. Este ciclo move a parte pura (parse/serialize/segment/join)
pra `crates/core`, sem mudar comportamento nenhum.

## Critérios de aceite

- [x] `crates/core/src/embed.rs` (novo) recebe: `EmbedKind`,
      `DocSegment`, `EmbedData`, `segment`, `join`, `BADGE_PALETTE`,
      `badge_class`, e todas as structs de dados (`KanbanCard`,
      `KanbanEmbedData`, `CalendarEntry`, `CalendarEmbedData`,
      `CalendarSource`, `ChecklistItem`, `Comment`, `Attachment`,
      `ColumnKind`, `TableColumn`, `TableEmbedData`) com seus `impl`
- [x] Os 53 testes de embed migram junto, sem edição semântica (só o
      caminho do `include_str!` e a troca de `anotadinho_core::` por
      `crate::` dentro do próprio crate)
- [x] `ui/src/embed.rs` fica só com o que depende de WASM
      (`scan_vault_calendar_entries`, que chama `crate::api`) e faz
      `pub use anotadinho_core::embed::*;` — nenhum arquivo da UI
      precisa mudar de import
- [x] `crates/core/src/lib.rs` exporta o módulo
- [x] Nada de `wasm-bindgen`/`web-sys`/`js-sys` entra em `core`
- [x] `cargo test -p anotadinho-core` passa com os testes migrados
      contabilizados

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mudar formato de arquivo, nomes de campo YAML ou comportamento de
  parse — refactor puro, os testes existentes são o contrato
- Expor os subcomandos de embed no CLI — é o ciclo 157
- Mexer nos componentes Yew

## Notas

`ui/src/date_util.rs` teve que ser dividido junto: `embed.rs` usa a
aritmética de data dele (`days_between`, `add_days`, `parse_time`...).
A parte pura (e os 9 testes) foi pro `crates/core/src/date_util.rs`; o
que lê o RELÓGIO (`today`, `today_string`, `now_minutes`, todas via
`js_sys::Date`) ficou na UI, que re-exporta o resto. Nenhum import
mudou em nenhum componente.

Contagem de testes: UI 88 → 26, core 29 → 91. Total 117 antes e
depois — nada perdido, só mudou de lado.

`core` já tem `serde_yaml` e `pulldown-cmark` no `Cargo.toml`, que são
as duas dependências que `embed.rs` usa — a mudança não acrescenta
dependência nenhuma ao workspace.
