---
id: "256"
titulo: "A cor da consulta identifica o valor, não a célula"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["217"]
estima_min: 45
---

# 256 — Cor por valor na consulta

## Estado ao começar

O RF2 da spec `leitura-de-consultas` parecia pronto: existe paleta,
existe `indice_cor_consulta`, existem os `--cor-N` no CSS. Os RF1, RF3,
RF4 e RF5 estavam mesmo fechados (alinhamento, altura rolável,
`max_height` por consulta, aviso de mais conteúdo).

Mas a chave da cor era `(coluna, campo, valor)`, e o efeito era o
contrário do pedido: numa tabela do vault real, `ciclo` na coluna `type`
e `ciclo` na coluna `tags` saíam de cores diferentes — 39 conflitos numa
página só. O cenário pendente media exatamente isso e reprovava.

E as duas chamadas nem concordavam entre si: a tabela passava a coluna, o
cartão passava `""`. O mesmo valor mudava de cor ao trocar a visão.

## Critérios de aceite

- [x] O mesmo valor tem a mesma cor em qualquer coluna, visão ou consulta
- [x] Valores diferentes se espalham pela paleta
- [x] Os três cenários de `consultas` saem de `pendentes.mjs` para
      `interacoes.mjs`

## A troca assumida

`alta` numa escala de prioridade e `alta` em qualquer outra escala agora
dividem a cor. É o preço, e é menor que o problema: duas escalas
dividindo a mesma palavra é raro, a mesma palavra mudando de cor de
coluna em coluna era o tempo todo.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
