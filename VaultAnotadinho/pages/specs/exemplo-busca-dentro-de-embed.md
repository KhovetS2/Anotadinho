---
title: Exemplo — Busca dentro de embed
date: 2026-08-18
start: 2026-08-18
end: 2026-08-26
status: in-progress
priority: alta
owner: ''
depends_on: []
related_decision: ''
tags:
- spec
- exemplo
---
# Exemplo — Busca dentro de embed

> Status: `backlog` → `in-progress` → `in-review` → `done` (ou `blocked`).

Esta spec existe pra o vault de exemplo mostrar o fluxo COMPLETO do
esquema. Com uma spec só, e em `backlog`, o bloco "Em andamento" do
[[Painel]] aparecia vazio — e um painel vazio não ensina nada sobre o
que ele faz.

Ela também é a única página com `start` e `end` dentro da janela padrão
do cronograma, então é ela que desenha a barra do
`{{ type: "timeline" }}` com `source: vault` no painel.

## Problema

Procurar por um card de kanban achava o YAML cru: a página certa abria,
mas sem dizer que aquilo era um card nem em que coluna estava.

## Proposta

Indexar cada registro de dentro dos embeds como um documento próprio,
com o tipo e uma âncora que leve até ele.

## Como saber que funcionou

- Buscar o título de um card devolve "Kanban · coluna X".
- Clicar no resultado rola até o embed e o destaca.
- `anotadinho-cli search` mostra a mesma origem no terminal.

## Relacionado

- [[Guia do Agent OS]] — o esquema que esta spec exercita
- [[Consultas — listas que se mantêm sozinhas]] — como o painel filtra
