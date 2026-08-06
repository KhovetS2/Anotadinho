---
title: Exemplos de Embeds
tags: [demo, embed]
---

# Exemplos de blocos embedados

Você pode usar blocos especiais dentro de qualquer página `.md`, delimitados
por `{{ type: "..." }}` ... `{{ /... }}` (não usa fence de código markdown,
pra não colidir com blocos de código de verdade):

## Kanban Embed

{{ type: "kanban" }}
columns:
- Backlog
- Todo
- Done
items:
- title: Tarefa 1
  column: Backlog
  description: Descrição da tarefa 1, com mais detalhes do que cabe no título.
  tags:
  - urgente
  - bug
  due: '2026-08-10'
  checklist:
  - text: Sub-item 1
    done: false
  - text: Sub-item 2
    done: true
- title: Tarefa 2
  column: Done
- title: Tarefa 3
  column: Done
{{ /kanban }}

## Calendar Embed

{{ type: "calendar" }}
entries:
- date: '2026-08-06'
  title: Revisão de código
- date: '2026-08-07'
  title: Deploy produção
- date: '2026-08-08'
  title: Retrospectiva sprint
{{ /calendar }}

## Table Embed

{{ type: "table" }}
| Tarefa | Status | Prioridade |
| ------ | ------ | ---------- |
| API    | done   | alta       |
| UI     | doing  | media      |
| Testes | todo   | alta       |
{{ /table }}

Acima do embed você pode ter texto normal. Abaixo também.
