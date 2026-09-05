---
id: "262"
titulo: "A cadeia de responsabilidade das teclas"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["261"]
estima_min: 90
---

# 262 — A cadeia de responsabilidade

## Objetivo

A regra do tmux aninhado como função pura: a tecla vai pra unidade mais
interna que declara interesse nela, e sobe até o documento se ninguém
declarar.

## Critérios de aceite

- [x] `rotear(linhagem, interesse, declara)` resolve de dentro pra fora
- [x] Um embed que declara `Movimento` trata o `j` e deixa o `d` subir
- [x] Um embed que declara NADA se comporta como hoje (tudo sobe) — é o
      que torna a migração dos dez incremental
- [x] Texto em edição consome tudo: `dd` num campo não apaga o bloco
- [x] A raiz não é elo da cadeia: o documento é fallback explícito
- [x] Zero `web_sys`

## Por que interesse por CATEGORIA e não por tecla

Declarar tecla a tecla faria cada embed listar dezenas, e o vim ganha
comando a cada ciclo. Categoria é estável: um calendário quer
`Movimento` e não quer `Operador` — e é exatamente isso que faz `dd`
apagar o calendário em vez de não fazer nada.

## Comandos de validação

```bash
cargo test --workspace
```
