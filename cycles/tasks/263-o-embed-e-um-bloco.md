---
id: "263"
titulo: "O embed é um bloco: a lista única"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["261", "262"]
estima_min: 120
---

# 263 — O embed é um bloco

## Objetivo

O RF1 da spec: uma lista de blocos só, a mesma pro modo de navegação,
pro vim e pra seleção. É o primeiro ciclo desta série com efeito
visível — o `j` deixa de pular por cima de uma tabela.

## Critérios de aceite

- [x] O embed entra em `[data-nav-block]`, com valor `"embed"`
- [x] `j`/`k` pousam no embed
- [x] Pousar num embed é foco e realce, não caret (RF3)
- [x] `blocos_de_texto()` continua sem embed, pra quem precisa de cursor
- [x] Cenários que reprovavam antes

## O que NÃO entra aqui

Copiar e apagar embed (ciclo 264). O markdown de um embed não está no
DOM — o DOM tem a tabela desenhada, não o `{{ type: "table" }}` que a
gerou. Até o 264, uma seleção com embed no meio copia os blocos de texto
e PULA o embed, que é menos errado do que colar HTML achatado.

## Comandos de validação

```bash
cd ui && cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
