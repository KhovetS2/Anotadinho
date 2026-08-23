---
title: Início
type: landing
tags: [inicio]
---
# Anotadinho

Editor de notas markdown que também é o painel do próprio
desenvolvimento. Tudo abaixo se atualiza sozinho a partir do vault.

{{ type: "actions" }}
layout: row
buttons:
- label: Nova conversa
  icon: zap
  variant: primary
  action: run-search
  query: Nova conversa com o agente
- label: Como usar o modo agêntico
  icon: file-text
  action: open-page
  path: pages/produto/como-usar-modo-agentico.md
- label: Propostas pendentes
  icon: check-square
  action: open-page
  path: pages/propostas.md
- label: Ciclos
  icon: clock
  action: open-page
  path: pages/ciclos.md
- label: Painel do produto
  icon: home
  action: open-page
  path: pages/produto/painel.md
- label: Nova nota de reunião
  icon: file-text
  action: new-from-template
  template: templates/nota-de-reuniao.md
{{ /actions }}

## Esperando você

O que está pronto pra alguém ler e decidir. Nada aqui avança sozinho.

{{ type: "query" }}
where:
- field: status
  op: eq
  value: em-revisao
sort:
  field: date
  desc: true
view: table
columns:
- type
- prioridade
{{ /query }}

## Specs esperando decisão

{{ type: "query" }}
from: pages/specs
where:
- field: status
  op: neq
  value: concluida
sort:
  field: prioridade
view: table
columns:
- status
- prioridade
- date
{{ /query }}

## O que o vault tem

{{ type: "query" }}
group_by: type
aggregate:
- op: count
view: list
collapsed: true
{{ /query }}

## Trabalho recente

{{ type: "query" }}
from: pages/ciclos
sort:
  field: ciclo
  desc: true
limit: 8
view: cards
columns:
- prioridade
{{ /query }}

## Por onde começar

- [[Como usar o modo agêntico]] — operar o app com um modelo
- [[Capacidades de agente]] — o que ele pode e o que é barrado
- [[Guia do Agent OS]] — o esquema de specs, decisões e padrões
- [[Exemplos de Embeds]] — os 10 blocos que dá pra usar numa nota
- [[Arquitetura]] — como o código está organizado
