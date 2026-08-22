---
title: Ciclo 011 — Preview Markdown renderizado (toggle Editar/Visualizar)
type: ciclo
ciclo: "011"
status: concluida
date: 2026-08-05
prioridade: alta
depende_de: ["010"]
tags:
- ciclo
---

# Ciclo 011 — Preview Markdown renderizado (toggle Editar/Visualizar)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

## Resumo
Ciclo 011: Preview Markdown renderizado.
- pulldown-cmark no frontend WASM
- Botao Visualizar/Editar no editor
- CSS completo para preview (headings, code, tables, etc)
