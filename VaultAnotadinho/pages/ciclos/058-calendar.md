---
title: Ciclo 058 — Calendar view + integração com tarefas
type: ciclo
ciclo: "058"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: ["057"]
tags:
- ciclo
---

# Ciclo 058 — Calendar view + integração com tarefas

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Calendar View

## Objetivo
View de calendário mensal mostrando páginas com property `date:: YYYY-MM-DD`.
Click na data mostra/abre a página. Navegação entre meses.

## Critérios
- [ ] Calendário mensal renderizado — implementado como lista agrupada por data, não grade de mês
- [x] Páginas com `date::` aparecem (escaneadas no vault inteiro, agrupadas e ordenadas por data)
- [x] Click no item abre a página (não há conceito de "dia" clicável, só os itens dentro do grupo)
- [ ] Navegação mês anterior/seguinte — não implementado
- [ ] Integração com kanban — não implementado

## Nota de backfill (2026-08-06)
Marcada como `done` retroativamente (implementada nos commits
`00c1ecf`…`93ebe73`, fora do orchestrator). Entrega funcional mas mais
simples que a spec: `ui/src/components/calendar.rs` é uma lista agrupada
por `date::`, não um grid mensal com navegação. Ver critérios não marcados.

## Resultado

## Resumo
Backfill retroativo. Calendar view implementada fora do orchestrator, junto
com o mesmo bloco de commits do 057/059 (`00c1ecf`…`93ebe73`). Ver
`cycles/tasks/058-calendar.md` para os critérios não atendidos (lista
agrupada por data em vez de grid mensal, sem navegação entre meses).
