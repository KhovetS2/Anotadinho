---
id: "182"
titulo: "Transclusão dentro de código inline"
status: done
criado: 2026-08-21
autor: agente
prioridade: media
depende_de: ["170"]
estima_min: 30
agente_alvo: claude-opus-5
---

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
