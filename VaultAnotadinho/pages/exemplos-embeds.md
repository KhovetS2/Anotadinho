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
  tag: urgente
- date: '2026-08-07'
  title: Deploy produção
  start_time: '14:30'
  end_time: '15:15'
- date: '2026-08-10'
  title: Sprint de agosto
  end_date: '2026-08-14'
  tag: infra
- date: '2026-08-08'
  title: Retrospectiva sprint
- title: Ligar pro fornecedor
{{ /calendar }}

## Table Embed

{{ type: "table" }}
columns:
- name: Tarefa
- name: Status
  type: select
  options: [todo, doing, done]
- name: Tags
  type: multiselect
  options: [urgente, bug, infra]
- name: Estimativa
  type: number
- name: Prazo
  type: date
- name: Referência
  type: url
- name: Relacionado
  type: page
---
| Tarefa | Status | Tags         | Estimativa | Prazo      | Referência          | Relacionado             |
| ------ | ------ | ------------ | ---------- | ---------- | ------------------- | ------------------------ |
| API    | done   | infra        | 8          | 2026-08-05 | https://example.com | pages/kanban-projeto.md |
| UI     | doing  | urgente, bug | 5          | 2026-08-10 |                      |                          |
| Testes | todo   |              | 3          |            |                      |                          |
{{ /table }}

Acima do embed você pode ter texto normal. Abaixo também.

## PDF Embed

Diferente dos blocos acima (`{{ type: "..." }}`, sintaxe de fence
explícita), embed de PDF é automático: qualquer link markdown normal
apontando pra um arquivo `.pdf` em `assets/` vira um frame com scroll
interno sozinho, sem precisar de sintaxe especial — só um link comum:

[Relatório de exemplo](assets/exemplo-relatorio.pdf)

Clicar/scrollar dentro do frame acima navega dentro do PDF (zoom, busca
e paginação do próprio visualizador do WebView), sem abrir outro
programa. Ao salvar a página, o link markdown original é preservado
tal qual — o frame é só uma forma de exibir, o arquivo `.md` continua
guardando `[texto](caminho.pdf)`.
