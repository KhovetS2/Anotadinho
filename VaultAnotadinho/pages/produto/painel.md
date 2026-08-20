---
title: Painel
type: landing
tags: [produto]
---
# Painel

{{ type: "callout" }}
variant: info
title: Comece por aqui
body: |
  Este painel é o ponto de entrada do esquema descrito no
  [[Guia do Agent OS]]: os botões abaixo criam as páginas nos lugares
  certos, e as listas se atualizam sozinhas conforme o `status` das
  specs muda. Nada aqui precisa ser mantido na mão.
{{ /callout }}

## Criar

{{ type: "actions" }}
layout: row
buttons:
- label: Nova spec
  icon: file-text
  variant: primary
  action: new-from-template
  template: templates/spec.md
  folder: pages/specs
- label: Nova decisão
  icon: check
  action: new-from-template
  template: templates/decisao.md
  folder: pages/decisoes
- label: Novo padrão
  icon: settings
  action: new-from-template
  template: templates/padrao-codigo.md
  folder: pages/padroes
- label: Sessão de trabalho
  icon: clock
  action: new-from-template
  template: templates/sessao-de-trabalho.md
- label: Abrir roadmap
  icon: home
  action: open-page
  path: pages/produto/roadmap.md
{{ /actions }}

## Em andamento

{{ type: "query" }}
from: pages/specs
where:
- field: status
  op: eq
  value: in-progress
view: list
columns:
- priority
{{ /query }}

## Fila (backlog, por prioridade)

{{ type: "query" }}
from: pages/specs
where:
- field: status
  op: eq
  value: backlog
sort:
  field: priority
view: list
columns:
- priority
- date
{{ /query }}

## Decisões

{{ type: "query" }}
from: pages/decisoes
sort:
  field: date
  desc: true
limit: 6
view: cards
columns:
- status
{{ /query }}

## Cronograma

{{ type: "timeline" }}
scale: month
source: vault
{{ /timeline }}

## Referência rápida

{{ type: "columns" }}
columns:
- width: 1
  body: |
    ### Status de spec

    `backlog` → `in-progress` → `in-review` → `done`

    (ou `blocked`, quando algo de fora trava)
- width: 1
  body: |
    ### Pelo terminal

    ```
    anotadinho-cli --vault . query \
      --from pages/specs --where 'status!=done'
    ```
{{ /columns }}
