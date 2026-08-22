---
title: Ciclo 059 — Task tables com filtro e ordenação
type: ciclo
ciclo: "059"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: ["058"]
tags:
- ciclo
---

# Ciclo 059 — Task tables com filtro e ordenação

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Task Tables

## Objetivo
View de tabela listando todas as páginas com property `status::`.
Colunas: título, status, data, prioridade. Ordenável e filtrável.
Integrado com kanban e calendário.

## Critérios
- [x] Tabela com colunas título/status/prioridade (coluna "data" não incluída)
- [x] Ordenação por coluna (click no header — título/status/prioridade)
- [ ] Filtro por status/prioridade — não implementado (só ordenação)
- [x] Click na linha abre no editor
- [ ] Sincronizado com kanban/calendário — não implementado

## Nota de backfill (2026-08-06)
Marcada como `done` retroativamente (implementada nos commits
`00c1ecf`…`93ebe73`, fora do orchestrator). `ui/src/components/task_table.rs`
escaneia `status::`/`priority::` no vault inteiro. Ver critérios não marcados.

## Resultado

## Resumo
Backfill retroativo. Task table implementada fora do orchestrator, junto com
o mesmo bloco de commits do 057/058 (`00c1ecf`…`93ebe73`). Ver
`cycles/tasks/059-task-table.md` para os critérios não atendidos (sem filtro,
sem sincronização com kanban/calendário).
