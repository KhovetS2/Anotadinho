---
title: Início
type: landing
tags:
- guia
---

# Início

{{ type: "callout" }}
variant: tip
title: Por onde começar
body: |
  `Ctrl+K` abre a paleta: é de lá que se cria página, projeto e conversa
  com o agente. `Ctrl+/` mostra os atalhos. O que cada tipo de página e
  cada embed fazem está no [[Guia do vault]]. O trabalho com o agente
  está em [[Como usar projetos]] e [[Como usar o modo agêntico]].
{{ /callout }}

Este painel é feito de consultas: cada bloco abaixo lê o vault e se
atualiza sozinho. Edite à vontade, porque é uma página como outra
qualquer.

## Projetos

{{ type: "query" }}
from: pages/projetos
where:
- field: type
  op: eq
  value: projeto
view: cards
{{ /query }}

## Sprints ativas

{{ type: "query" }}
from: pages/projetos
where:
- field: type
  op: eq
  value: sprint
- field: estado
  op: eq
  value: ativa
sort:
  field: fim
view: table
columns:
- projeto
- objetivo
- fim
{{ /query }}

## Esperando você

As tarefas que o agente terminou e esperam a sua revisão. As propostas
do agente ficam na **Revisão**: o ícone no topo da janela acende quando
há alguma.

{{ type: "query" }}
from: pages/projetos
where:
- field: type
  op: eq
  value: tarefa
- field: status
  op: eq
  value: revisao
view: table
columns:
- projeto
- prioridade
{{ /query }}

## Com prazo

As tarefas abertas que têm prazo, as mais urgentes primeiro.

{{ type: "query" }}
from: pages/projetos
where:
- field: type
  op: eq
  value: tarefa
- field: prazo
  op: exists
  value: ''
- field: status
  op: neq
  value: feita
sort:
  field: prazo
view: table
columns:
- projeto
- prazo
- status
{{ /query }}

## Specs

{{ type: "query" }}
from: pages/specs
view: list
group_by: status
aggregate:
- op: count
{{ /query }}

## Propostas

{{ type: "query" }}
from: pages/propostas
view: list
group_by: status
aggregate:
- op: count
{{ /query }}

## Execuções recentes

{{ type: "query" }}
from: pages/execucoes
sort:
  field: date
  desc: true
limit: 5
view: list
{{ /query }}
