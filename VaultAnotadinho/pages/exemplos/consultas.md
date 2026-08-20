---
title: Consultas — listas que se mantêm sozinhas
tags: [demo, embed, consulta]
---
# Consultas vivas

Uma consulta é um RECORTE declarado do vault. Ela não guarda dados:
lê o frontmatter e as properties `chave:: valor` de todas as páginas e
mostra quem bate com o filtro. Mudou a página, mudou a lista.

O mesmo recorte roda no terminal:

```
anotadinho-cli --vault . query --from-embed pages/exemplos/consultas.md:0
```

## Lista simples

Tudo que tem `date` — jornais e páginas datadas.

{{ type: "query" }}
where:
- field: date
  op: exists
sort:
  field: date
  desc: true
limit: 6
view: list
columns:
- date
{{ /query }}

## Tabela com campos escolhidos

`columns` define o que aparece; cada célula é editável na própria linha
(ciclo 168) e a lista reavalia depois de gravar.

{{ type: "query" }}
from: pages
where:
- field: type
  op: exists
view: table
columns:
- type
- tags
{{ /query }}

## Cartões

{{ type: "query" }}
from: pages/produto
view: cards
columns:
- type
{{ /query }}

## Agrupada, com contagem

Substitui ter uma consulta por valor. O grupo "sem campo" fica sempre
no fim, e cada cabeçalho recolhe no clique — o estado fica no YAML.

{{ type: "query" }}
from: pages
group_by: type
aggregate:
- op: count
view: list
{{ /query }}

## Operadores

| Escrito | Significa |
|---|---|
| `op: eq` | igual (ignora caixa) |
| `op: neq` | diferente — **inclui quem não tem o campo** |
| `op: contains` | contém o texto |
| `op: exists` | campo presente e não vazio |
| `op: gt` / `lt` | maior/menor; numérico quando os dois lados são número |

> `neq` incluir quem não tem o campo é decisão consciente: "specs que
> não estão em `done`" precisa mostrar a spec sem `status`, que é
> justamente o trabalho não classificado. ^neq
