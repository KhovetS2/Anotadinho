---
id: "011"
titulo: "Preview Markdown renderizado (toggle Editar/Visualizar)"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["010"]
estima_min: 40
agente_alvo: claude-sonnet
---

# Preview Markdown renderizado

## Objetivo

Botão "Visualizar" no editor alterna entre textarea (edição) e preview
HTML renderizado do Markdown, usando pulldown-cmark no frontend WASM.

## Critérios de aceite

- [x] Botão Visualizar/Editar no header do editor
- [x] Preview renderiza headings, bold, italic, listas, código
- [x] Pulldown-cmark compila no WASM
- [x] `cargo test --workspace` exit 0
- [x] trunk build + src-tauri build exit 0
- [x] App continua abrindo

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```
