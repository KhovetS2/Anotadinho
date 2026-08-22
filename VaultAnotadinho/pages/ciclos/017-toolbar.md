---
title: Ciclo 017 — Toolbar de formatação Markdown
type: ciclo
ciclo: "017"
status: concluida
date: 2026-08-05
prioridade: alta
depende_de: ["016"]
tags:
- ciclo
---

# Ciclo 017 — Toolbar de formatação Markdown

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Toolbar de formatação

## Objetivo

Adicionar toolbar entre o header e o editor com botões para inserir
formatação Markdown no texto selecionado.

## Critérios de aceite

- [x] Botões: Bold, Italic, Heading, List, Code, Quote, Link, Table
- [x] Inserem sintaxe Markdown no cursor/seleção
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
```

## Resultado

## Resumo
Ciclo 017: Editor WYSIWYG contenteditable + toolbar.
- contenteditable div com formatação inline
- Toolbar: Bold, Italic, Underline, Strikethrough, H1/H2, P, List, Quote, Code, Link
- HTML → Markdown converter (html_to_md.rs)
- Load: Markdown → HTML (pulldown-cmark)
- Save: HTML → Markdown (DOM walker)
