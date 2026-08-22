---
title: Ciclos
type: landing
tags: [processo, docs]
---
# Ciclos

O Anotadinho evolui por **ciclos**: uma feature por vez, com task escrita
antes, validação rodada depois e um registro do que aconteceu.

O histórico inteiro vive no vault desde o ciclo 206 — cada ciclo é uma
página em `pages/ciclos/`, com o mesmo `status` que as consultas abaixo
filtram. Antes disso ele só existia em `cycles/`, que o produto não
enxergava.

{{ type: "callout" }}
variant: tip
title: Como acompanhar
body: |
  As listas abaixo se atualizam sozinhas. Para achar um ciclo específico,
  a busca acha pelo texto da task — inclusive dentro dos embeds, desde o
  ciclo 188.
{{ /callout }}

## Em execução

{{ type: "query" }}
from: pages/ciclos
where:
- field: status
  op: eq
  value: em-execucao
sort:
  field: ciclo
  desc: true
view: list
columns:
- ciclo
{{ /query }}

## Últimos concluídos

{{ type: "query" }}
from: pages/ciclos
where:
- field: status
  op: eq
  value: concluida
sort:
  field: ciclo
  desc: true
limit: 12
view: table
columns:
- ciclo
- prioridade
- date
{{ /query }}

## Por prioridade

{{ type: "query" }}
from: pages/ciclos
group_by: prioridade
aggregate:
- op: count
view: list
{{ /query }}

## O processo

Cada ciclo passa por: task escrita → implementação → validação
(`cargo test --workspace`, `trunk build`, build do Tauri e o harness de
UI) → registro do resultado → commit `feat({id}):`.

Regressão achada durante a validação vira cenário no harness, com o
número do ciclo onde apareceu. É por isso que a suíte cresceu de zero
para mais de cem cenários sem ninguém ter parado pra "escrever testes".

Ver também: [[Guia do Agent OS]], [[Capacidades de agente]],
[[Arquitetura]].
