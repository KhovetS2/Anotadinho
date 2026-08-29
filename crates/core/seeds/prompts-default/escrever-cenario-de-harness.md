---
title: Escrever cenário de harness
type: prompt
date: 2026-08-29
tags:
- prompt
---

Escreva um cenário de harness para o comportamento abaixo.

{{comportamento}}

Regras que valem aqui:

- O cenário cria a própria página de rascunho com `ctx.escrever` e nunca
  toca em página real do vault.
- Espere por condição com `ctx.esperar`, não por tempo fixo.
- Escopo o clique no elemento do próprio cenário. Clicar no "primeiro
  botão da tela" já aplicou uma proposta de verdade sem revisão.
- O nome termina com o número do ciclo entre parênteses.
- Limpe no `finally` o que criou fora da página de rascunho.

Diga em qual arquivo de `scripts/uitest/` ele entra e por quê.
