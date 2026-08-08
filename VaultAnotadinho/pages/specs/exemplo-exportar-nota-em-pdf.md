---
title: Exemplo — Exportar nota em PDF
date: 2026-08-08
status: backlog
priority: media
owner: ''
depends_on: []
related_decision: ''
tags:
- spec
- exemplo
---
# Exemplo — Exportar nota em PDF

> Esta é uma spec de EXEMPLO, preenchida de ponta a ponta, pra mostrar
> o nível de detalhe esperado. Ao criar uma spec de verdade, use o
> template "spec" (botão "Nova página" → escolher template) — ele já
> vem com esta estrutura vazia.

## Contexto

Hoje o Anotadinho só exporta HTML (por página) e markdown bruto (em
massa, ciclo 101). Usuários que compartilham notas fora do vault (ex:
mandar uma spec pra alguém que não usa o app) pedem PDF, que preserva
formatação sem depender de um visualizador de markdown.

## Objetivo

Usuário consegue exportar a página atual como um `.pdf` com a mesma
formatação visual do editor (headings, listas, tabelas, código),
disparado por um botão no menu "⋯" do editor.

## Escopo

### Dentro do escopo

- Exportar UMA página por vez, a que está aberta
- Preserva formatação básica: headings, negrito/itálico, listas,
  tabelas, blocos de código
- Botão no menu "⋯" do editor (mesmo menu do ciclo 109)

### Fora do escopo

- Exportação em massa (várias páginas num PDF só) — extensão futura
- Preservar embeds interativos (kanban/calendário) como imagem —
  primeira versão pode simplesmente pular ou renderizar como texto

## Requisitos funcionais

- [ ] Botão "Exportar PDF" no menu "⋯" do editor
- [ ] Gera um `.pdf` com o conteúdo renderizado da página atual
- [ ] Nome do arquivo baixado = slug do título da página

## Requisitos não-funcionais

- Não pode travar a UI pra páginas grandes (>5000 palavras) — gerar de
  forma assíncrona

## Design técnico

Provável abordagem: renderizar o HTML já produzido por
`markdown_render.rs` numa `<div>` fora da tela, e usar a API nativa de
impressão do WebView (`window.print()` com CSS `@media print`) em vez
de uma lib de geração de PDF — evita dependência nova pesada (ver
[[Stack Técnico]]).

## Plano de tarefas

- [ ] CSS `@media print` dedicado (esconder sidebar/header, só o
      conteúdo da página)
- [ ] Botão no menu "⋯" chamando `window.print()`
- [ ] Testar em página com tabela e bloco de código

## Critérios de aceite

- [ ] Abrir uma página com heading/lista/tabela, clicar "Exportar
      PDF", confirmar visualmente que a formatação é preservada no
      preview de impressão
- [ ] Sidebar e header NÃO aparecem no PDF gerado

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
```

## Riscos e dependências

`window.print()` dentro do WebView do Tauri pode ter comportamento
diferente por SO — validar em pelo menos uma plataforma antes de
considerar pronto.

## Não-objetivos

- Gerar PDF no backend Rust (ex: com `printpdf`) — versão 1 usa o
  motor de impressão nativo, mais simples e sem dependência nova

## Notas

Exemplo ilustrativo — esta feature não está implementada, é só
material de referência pro formato de spec.
