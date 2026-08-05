---
title: Sobre o Anotadinho
tags: [projeto, docs]
created: 2026-08-04
---

# Sobre o Anotadinho

Anotadinho é um editor de notas Markdown nativo, construído com **Tauri + Rust + Yew**.

## Stack

| Camada | Tecnologia |
|---|---|
| Runtime | Tauri 2.x |
| Backend | Rust puro |
| UI | Yew 0.21 (WASM) |
| Storage | Markdown + YAML frontmatter |

## Funcionalidades

- Vault local: suas notas são arquivos `.md` comuns
- Sidebar com seções Pages e Journals
- Persistência em localStorage (último vault aberto)
- Tema dark com acentos azul/roxo
- Interface nativa via WebView

## Desenvolvimento

O projeto evolui por ciclos. Cada ciclo implementa exatamente uma feature.
Veja a página "Ciclos" para detalhes.
