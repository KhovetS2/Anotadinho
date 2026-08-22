---
title: Ciclo 021 — Markdown live formatting (character shortcuts as-you-type)
type: ciclo
ciclo: "021"
status: concluida
date: 2026-08-05
prioridade: alta
depende_de: ["020"]
tags:
- ciclo
---

# Ciclo 021 — Markdown live formatting (character shortcuts as-you-type)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Markdown live formatting

## Objetivo

Ao digitar markdown syntax, a formatação é aplicada inline:
- `# ` → heading
- `- ` ou `* ` → lista
- `> ` → citação  
- `**text**` → bold (remove os `**` e aplica bold)
- `*text*` → italic
- `` `code` `` → código inline

## Critérios de aceite

- [ ] `# ` + espaço converte linha para heading
- [ ] `- ` item vira lista
- [ ] `**texto**` detecta na digitação e aplica bold inline
- [ ] App continua compilando e abrindo

## Resultado

## Resumo
Ciclo 021: Markdown live formatting - character shortcuts.
- # ao inicio da linha + espaço → heading (h1-h6)
- -  ou *  + enter → lista
- >  + enter → citação
- 1.  + enter → lista numerada
