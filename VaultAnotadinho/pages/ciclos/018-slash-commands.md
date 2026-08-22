---
title: Ciclo 018 — Comandos slash (/) para inserir blocos especiais
type: ciclo
ciclo: "018"
status: concluida
date: 2026-08-05
prioridade: alta
depende_de: ["017"]
tags:
- ciclo
---

# Ciclo 018 — Comandos slash (/) para inserir blocos especiais

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Comandos Slash (/)

## Objetivo

Ao digitar `/` no início de uma linha, abre um menu flutuante com
opções de blocos: Título, Lista, Citação, Código, Tabela, Checklist,
Divisor (hr). Selecionar um item insere o bloco correspondente.

## Critérios de aceite

- [x] Menu slash aparece ao digitar `/` no início de linha
- [x] Opções: H1, H2, Lista, Checklist, Quote, Code, Table, HR
- [x] Navegação com setas + Enter seleciona
- [x] Escape fecha o menu
- [x] App continua compilando e abrindo

## Resultado

## Resumo
Ciclo 018: Slash commands (/).
- Menu flutuante ao digitar /
- 9 opções: H1-H3, Lista, Checklist, Citação, Código, Tabela, Linha
- Navegação com setas + Enter seleciona
- Escape fecha, Backspace retorna da busca
- Filtro em tempo real por texto digitado
