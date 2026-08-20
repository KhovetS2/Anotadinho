---
title: Início
type: landing
---
# Anotadinho

{{ type: "callout" }}
variant: info
title: Este vault é a demonstração do app
body: |
  Cada página aqui existe pra mostrar um recurso funcionando com
  conteúdo real. Comece pelos exemplos abaixo, ou vá direto pro
  [[Painel]] se você já conhece o esquema de trabalho.
{{ /callout }}

## Exemplos

{{ type: "actions" }}
layout: grid
buttons:
- label: Embeds de dados
  icon: table
  variant: primary
  action: open-page
  path: pages/exemplos-embeds.md
- label: Composição da página
  icon: layout
  action: open-page
  path: pages/exemplos/composicao.md
- label: Consultas vivas
  icon: search
  action: open-page
  path: pages/exemplos/consultas.md
- label: Referências e transclusão
  icon: link
  action: open-page
  path: pages/exemplos/referencias.md
- label: Painel do Agent OS
  icon: home
  action: open-page
  path: pages/produto/painel.md
- label: Guia do Agent OS
  icon: file-text
  action: open-page
  path: pages/produto/guia-agent-os.md
{{ /actions }}

## O que está acontecendo no vault

{{ type: "query" }}
where:
- field: date
  op: exists
sort:
  field: date
  desc: true
limit: 5
view: list
columns:
- date
{{ /query }}

## Agenda

{{ type: "calendar" }}
mode: vault
{{ /calendar }}
