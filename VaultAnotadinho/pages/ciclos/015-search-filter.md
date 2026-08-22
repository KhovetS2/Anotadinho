---
title: Ciclo 015 — Busca de páginas por título (search bar na sidebar)
type: ciclo
ciclo: "015"
status: concluida
date: 2026-08-05
prioridade: alta
depende_de: ["014"]
tags:
- ciclo
---

# Ciclo 015 — Busca de páginas por título (search bar na sidebar)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Busca de páginas

## Objetivo

Campo de busca na sidebar filtra páginas por título. Se o termo
aparecer no título (case-insensitive), a página aparece nos resultados.

## Critérios de aceite

- [x] Search bar no topo da sidebar
- [x] Filtro case-insensitive por título
- [x] Limpa filtro com Escape ou botão X
- [x] Empty state "Nenhum resultado"
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
cargo test --workspace
```

## Resultado

## Resumo
Ciclo 015: Busca por título na sidebar.
- Search bar filtra case-insensitive
- Escape limpa busca
- Botao X para limpar
- Empty state "Nenhum resultado"
