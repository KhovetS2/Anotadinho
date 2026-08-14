---
id: "146"
titulo: "Migra ícones do editor e painéis para SVG"
status: done
criado: 2026-08-14
autor: humano
prioridade: media
depende_de: ["144"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Migra ícones do editor e painéis para SVG

## Objetivo

Continuação do ciclo 145 — troca os emoji/glifos de `editor.rs`,
`typed_page_header.rs`, `properties_panel.rs` e `command_palette.rs`
pelo componente `Icon` (ciclo 144).

## Critérios de aceite

- [x] `editor.rs`: `⋯` (menu) → `more-horizontal`; `🏠` (definir/
      remover início) → `home`; `⬇ Exportar HTML` → `download`;
      `🕐 Histórico` → `clock`; `✕` (remover embed) → `x`;
      `📄` (item wikilink) → `file-text`; `🔗 Backlinks` → `link`
- [x] `typed_page_header.rs`: `⚙ Propriedades` → `settings`
- [x] `properties_panel.rs`: `✕` (remover tag/propriedade, 2 lugares)
      → `x`
- [x] `command_palette.rs`: `⚡` (comando) → `zap`; `📄` (página, 2
      lugares) → `file-text`
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: abri o menu "⋯" do editor
      numa página real — ícones de início/exportar/histórico nítidos;
      seção "Backlinks" com ícone de link visível no rodapé

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Ícones dos embeds (kanban/tabela/grafo) — ciclo 147

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`,
`trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`: OK,
sem erros de sintaxe desta vez (todas as trocas foram só `{ "texto" }`
→ `<Icon .../>{ " texto" }` inline, sem `if`/`else` novo).

Validação ao vivo via MCP `tauri`: abri a página "arquitetura" (tem
grafo mermaid + backlinks) e o menu "⋯" — "Definir como início",
"Exportar HTML" e "Histórico" todos com ícone nítido antes do texto;
"Backlinks (2)" no rodapé com ícone de corrente antes do texto.
