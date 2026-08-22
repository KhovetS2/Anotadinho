---
title: Ciclo 013 — Atalhos de teclado (Ctrl+N nova página)
type: ciclo
ciclo: "013"
status: concluida
date: 2026-08-05
prioridade: baixa
depende_de: ["012"]
tags:
- ciclo
---

# Ciclo 013 — Atalhos de teclado (Ctrl+N nova página)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Atalhos de teclado

## Objetivo

Ctrl+N cria nova página. Escape deseleciona página atual
(volta ao placeholder).

## Critérios de aceite

- [x] Ctrl+N abre prompt de nova página
- [x] Escape deseleciona página aberta
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
```

## Resultado

## Resumo
Ciclo 013: Ctrl+N nova página, Escape deseleciona.
