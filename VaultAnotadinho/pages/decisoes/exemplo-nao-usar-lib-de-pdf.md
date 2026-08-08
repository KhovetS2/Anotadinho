---
title: Exemplo — Não usar lib de geração de PDF no backend
date: 2026-08-08
status: aceita
related_spec: pages/specs/exemplo-exportar-nota-em-pdf.md
tags:
- decisao
- exemplo
---
# Exemplo — Não usar lib de geração de PDF no backend

_Decisão registrada em 2026-08-08._

> Decisão de EXEMPLO, ligada à spec de exemplo "Exportar nota em PDF".
> Ao registrar uma decisão de verdade, use o template "decisão".

## Contexto

Ao planejar a [[Exemplo — Exportar nota em PDF]], surgiu a dúvida:
gerar o PDF no backend Rust (com uma crate tipo `printpdf`) ou usar o
motor de impressão nativo do WebView (`window.print()` + CSS
`@media print`)?

## Decisão

Usar `window.print()` no frontend, não uma crate de geração de PDF no
backend.

## Alternativas consideradas

- **`printpdf` no backend**: mais controle sobre o layout final, mas
  exige reimplementar a renderização markdown→PDF do zero (a
  renderização HTML já existe via `markdown_render.rs`) e adiciona uma
  dependência pesada só pra isso.
- **Lib JS de PDF no frontend (ex: `jsPDF`)**: adiciona dependência
  WASM/JS nova só pra reimplementar o que o motor de impressão do
  próprio WebView já faz de graça.

## Consequências

- Formatação do PDF fica sujeita ao motor de impressão do SO — menos
  controle fino de layout, mas zero dependência nova
- Precisa de CSS `@media print` dedicado e testado por plataforma
