---
id: "244"
titulo: "Tirar a marca só do trecho selecionado"
status: done
criado: 2026-08-30
autor: agente
prioridade: alta
depende_de: ["240"]
estima_min: 120
---

# 244 — Marcação parcial

## Objetivo

Selecionar uma palavra de uma frase inteira em negrito e clicar em negrito
tirava o negrito da **frase toda**. O pedido é sobre a palavra.

## Critérios de aceite

- [x] Tirar a marca de um trecho preserva as bordas marcadas
- [x] Vale para negrito, itálico, tachado, código e link
- [x] Vale para cor e realce, e cada borda volta com a cor que tinha
- [x] Aplicar sobre uma marca parcial ESTENDE, não fragmenta
- [x] A marca nunca começa nem termina em espaço
- [x] Pintar um eixo não apaga o outro, mesmo em trecho parcial
- [x] A paleta não se fecha por causa da nossa própria cirurgia

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs
```
