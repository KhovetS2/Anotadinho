---
id: "231"
titulo: "Gravar uma página com embed não muda a página"
status: done
criado: 2026-08-29
autor: agente
prioridade: media
depende_de: []
estima_min: 30
---

# 231 — Cerca de embed estável

## Objetivo

Abrir e salvar uma página com embed acrescentava uma linha em branco antes
do `{{ /... }}`. O arquivo mudava sem ninguém ter escrito nada, e o diff
do git enchia de ruído.

## Critérios de aceite

- [x] `to_fence_text` não duplica a quebra de linha final do corpo
- [x] Gravar duas vezes seguidas produz o mesmo arquivo, nos quatro
      formatos de corpo (YAML puro, YAML + tabela markdown, campo de
      texto, fluxo)
- [x] `pages/incio.md` normalizado, mantendo a alteração real de quem
      recolheu os grupos da consulta

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
node scripts/uitest/run.mjs
```
