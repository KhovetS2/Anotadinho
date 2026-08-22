---
title: Ciclo 182 — Transclusão dentro de código inline
type: ciclo
ciclo: "182"
status: concluida
date: 2026-08-21
prioridade: media
depende_de: ["170"]
tags:
- ciclo
---

# Ciclo 182 — Transclusão dentro de código inline

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Transclusão dentro de código inline

## Objetivo

Achado ao escrever as páginas de exemplo do ciclo 183: uma página que
EXPLICA a sintaxe de transclusão, escrevendo `` `![[Página]]` `` entre
crases, via o próprio exemplo virar uma transclusão de verdade — e
como não existe página chamada "Página", aparecia "Página não existe
ainda" no meio da explicação.

O marcador do ciclo 170 pulava fence de código (```` ``` ````) mas não
código INLINE.

## Critérios de aceite

- [x] `![[X]]` entre crases não vira transclusão
- [x] `![[X]]` solto na mesma linha continua virando
- [x] Fence de código continua protegido
- [x] Testes no `markdown_render` cobrindo os quatro casos

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Proteger `[[wikilink]]` comum dentro de código inline (o `linkify`
  tem o próprio comportamento, herdado de antes; se incomodar, vira
  task própria)

## Notas

`ui/src/markdown_render.rs` ganhou seu primeiro módulo de teste: a
função é string pura e roda no host, sem WASM. Já entrou com um teste
do id de bloco junto (ciclo 176), que estava sem cobertura direta.

## Resultado

# Ciclo 182 - done

## Resumo

`![[X]]` escrito entre crases virava transclusão de verdade. Quem
tentasse documentar a sintaxe via o exemplo sumir e virar um aviso de
"página não existe".

## Arquivos criados/modificados

- `ui/src/markdown_render.rs` — `marcar_linha` respeita código inline,
  + módulo de testes novo

## Testes adicionados

- transclusão solta vira marcador
- dentro de código inline NÃO vira
- dentro de fence NÃO vira
- mistura das duas na mesma linha
- id de bloco vira marca discreta (cobertura que faltava do 176)

## Problemas encontrados

- Nenhum além do próprio bug.

## Notas para próximos ciclos

- `[[wikilink]]` comum dentro de código inline ainda vira link (o
  `linkify` é anterior a isso). Não incomodou ainda; se incomodar, é o
  mesmo tipo de conserto.
