---
title: "Ciclo 191 — Wikilink: exemplo em código inline e clique por título do frontmatter"
type: ciclo
ciclo: "191"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: [170, 182]
tags:
- ciclo
---

# Ciclo 191 — Wikilink: exemplo em código inline e clique por título do frontmatter

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Wikilink: exemplo em código inline e clique por título

## Objetivo

Dois defeitos achados pelo usuário na página `pages/exemplos/referencias.md`:

1. Os títulos que EXPLICAM a sintaxe (`## \`[[Página]]\` — link`) apareciam
   como `[Página](anotadinho://page/P%C3%A1gina)` — o exemplo virava um
   link de verdade e a URL percent-encoded ia pra tela.
2. Clicar em `[[Grafo do Vault]]` não abria nada.

## Critérios de aceite

- [x] `linkify` não converte `[[...]]` dentro de crase.
- [x] `extract_titles` também ignora — senão o grafo ganha aresta pra
      uma página que só existe no exemplo.
- [x] Clicar num wikilink resolve pelo título do FRONTMATTER, com
      fallback pro nome do arquivo.
- [x] Cenário de harness cobrindo os dois.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Mudar a sintaxe ou a resolução por título (continua por título, não
  por path).

## Notas

Os dois são a MESMA falha já corrigida noutro caminho, que não foi
propagada:

- O ciclo 182 pôs o guard de crase em `markdown_render::marcar_linha`
  (transclusão) e não em `wikilink::linkify_line` (link).
- O ciclo 170 trocou `list_pages` por `scan_vault` em
  `upgrade_transclusions_at`, pelo motivo exato de `list_pages` devolver
  o nome do ARQUIVO como título — o handler de clique ficou pra trás.

Isso explica por que o bug parecia intermitente: `[[Nomenclatura]]`
funcionava (título igual ao nome do arquivo) e `[[Grafo do Vault]]` não
(`grafo.md`).

## Resultado

# 191 — Wikilink em código inline e clique por título

## O que mudou

- `ui/src/wikilink.rs`: `linkify_line` e `extract_titles_line` passam a
  rastrear crase e pular código inline. 3 testes novos.
- `ui/src/components/editor.rs`: o clique em wikilink resolve por
  `scan_vault` (título do frontmatter) com fallback pro nome do arquivo,
  em vez de `list_pages` (que devolve o nome do arquivo como título).
- `scripts/uitest/cenarios.mjs`: cenário cobrindo os dois.

## Por que passou despercebido

Os dois são a mesma falha já corrigida noutro caminho, sem propagação:
o guard de crase entrou no 182 só pra transclusão, e a troca de
`list_pages` por `scan_vault` entrou no 170 só pra transclusão.

O segundo parecia intermitente porque `[[Nomenclatura]]` abria
(`nomenclatura.md` — título igual ao arquivo) e `[[Grafo do Vault]]` não
(`grafo.md`).

## Validação

- `cargo test --workspace`: 0 falhas.
- `cd ui && cargo test`: 34 passaram.
- `trunk build` (com `PATH` do cargo): `✅ success`.
- `cargo build --manifest-path src-tauri/Cargo.toml`: 0 erros.
- `node scripts/uitest/run.mjs`: **25/25 em 140.4s**.
- Na janela: os títulos de `referencias.md` voltaram a mostrar
  `[[Página]]` literal, e clicar em "Grafo do Vault" e em "Missão" abre
  as páginas certas.
