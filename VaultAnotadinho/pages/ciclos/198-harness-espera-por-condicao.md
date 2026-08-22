---
title: "Ciclo 198 — Harness: espera por condição em vez de relógio"
type: ciclo
ciclo: "198"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: [197]
tags:
- ciclo
---

# Ciclo 198 — Harness: espera por condição em vez de relógio

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Harness: espera por condição

## Objetivo

A suíte levava 7min43s, e 115 desses segundos eram esperas FIXAS
(`PAUSA(1500)` depois do reload, `PAUSA(2200)` depois de abrir a
página). Tempo dimensionado pro pior caso e desperdiçado em todos os
outros. Suíte lenta é rodada com menos frequência, e suíte que não roda
não vale nada.

## Critérios de aceite

- [x] `recarregarEstavel` — recarrega e espera o documento ser TROCADO,
      não um tempo fixo.
- [x] `abrirPaginaEstavel` — abre e espera o conteúdo PARAR de mudar.
- [x] `esperarEstavel` — duas leituras iguais seguidas.
- [x] Os quatro arquivos de cenário usando os novos helpers.
- [x] Suíte inteira verde, e mais rápida.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
```

## Resultado

**236.1s contra 462.8s** — 49% mais rápido, com os mesmos 85 cenários.
A bateria de digitação sozinha caiu de 95s pra 51s.

## Notas

**A corrida que quase passou despercebida:** a primeira versão do
`recarregarEstavel` esperava "a sidebar tem itens" logo depois de pedir
o reload. Isso passa NA HORA, porque o DOM antigo ainda está lá — e o
cenário clicava num nó prestes a ser destruído. Um marcador em `window`
resolve: ele só some quando o documento é realmente trocado.

O sintoma foi feio: a suíte inteira travou sem imprimir uma linha. A
lição é que espera por condição só é mais rápida se a condição
distinguir o estado NOVO do VELHO — senão ela não espera nada.

## Resultado

# 198 — Harness: espera por condição

## O que mudou

- `scripts/uitest/bridge.mjs`: `esperarEstavel`, `recarregarEstavel` e
  `abrirPaginaEstavel`.
- `cenarios.mjs`, `digitacao.mjs`, `blocos.mjs`, `interacoes.mjs`: os
  helpers de setup passaram a esperar por condição.

## Resultado medido

| | Antes | Depois |
|---|---|---|
| Suíte inteira (85 cenários) | 462.8s | **236.1s** |
| Bateria de digitação (17) | 95.5s | **51.2s** |
| Um cenário típico | ~5.5s | ~2.9s |

## O erro no caminho

A primeira versão esperava "a sidebar tem itens" logo após pedir o
reload — condição que passa instantaneamente contra o DOM ANTIGO. O
cenário clicava num nó prestes a ser destruído, e a suíte travou sem
imprimir nada. Um marcador em `window`, que só desaparece quando o
documento é trocado, resolve.

## Validação

- `node scripts/uitest/run.mjs`: **85/85 em 236.1s**.
