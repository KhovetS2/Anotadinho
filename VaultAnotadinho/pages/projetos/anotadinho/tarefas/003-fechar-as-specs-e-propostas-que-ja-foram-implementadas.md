---
title: Fechar as specs e propostas que já foram implementadas
tags:
- tarefa
type: tarefa
date: 2026-10-22
prioridade: alta
projeto: anotadinho
status: pronta
---
# Fechar as specs e propostas que já foram implementadas

## Objetivo

Fechar a etapa das specs e propostas cujo trabalho JÁ foi feito, mas que
ficaram paradas no vault. O `contexto` mostra trabalho em aberto que não
existe, e a próxima pessoa (ou agente) pode refazê-lo.

## Critérios de aceite

- [ ] Cada par abaixo está conferido contra o ciclo que o implementou e,
      se estiver mesmo pronto, com a etapa avançada pela pessoa:
  - [ ] `pages/specs/cli-junto-do-app.md` e `pages/propostas/cli-junto-do-app.md` (ciclo 450)
  - [ ] `pages/specs/prompt-de-qualquer-tamanho.md` e `pages/propostas/prompt-pelo-stdin.md` (ciclo 449)
  - [ ] `pages/specs/tui-no-windows.md` e `pages/propostas/tui-no-windows.md` (ciclo 448)
  - [ ] `pages/specs/revisao-de-proposta.md` e `pages/propostas/revisao-de-proposta.md` (ciclos 452 e 453)
- [ ] `cli contexto` não lista mais nenhuma delas como rascunho ou em
      revisão.

## Abordagem

Para cada uma: ler a spec, ler o status do ciclo, conferir os critérios
de aceite um a um. Pelo que ficar de fora, abrir tarefa nova.

## Fora de escopo

- Reescrever as specs.
