---
title: Ciclo 102 — Busca full-text real na paleta de comandos
type: ciclo
ciclo: "102"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 102 — Busca full-text real na paleta de comandos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Busca full-text real na paleta de comandos

## Objetivo

Quinto ciclo do tema "agent-os readiness" — fecha um gap encontrado na
auditoria: a paleta (`Ctrl+K`, ciclo 091) filtra páginas só por
SUBSTRING NO TÍTULO, sem usar o `SearchIndex` (SQLite FTS5) que já
existe desde o ciclo 094 e já alimenta a busca da sidebar. Uma paleta
que só acha pelo título é bem mais fraca que a busca lateral — deveria
ser pelo menos igual.

## Critérios de aceite

- [x] `ui/src/components/command_palette.rs`: quando a query tem 3+
      caracteres, busca também no CONTEÚDO das páginas via
      `api::search_content` (mesma função já usada pela sidebar,
      ciclo 094), debounced, mostrando resultado com trecho destacado
      (reaproveita `render_excerpt_highlight` de `sidebar.rs`, extraído
      pra um lugar comum se fizer sentido)
- [x] Resultados de conteúdo aparecem numa seção separada dos títulos
      (que continuam com match instantâneo, sem esperar a busca
      assíncrona) — não bloqueia a navegação rápida por título já
      existente
- [x] Query com menos de 3 caracteres continua só filtrando por
      título/comando (mesmo comportamento de hoje, sem busca de
      conteúdo — evita disparar buscas caras a cada tecla no início)
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Busca em comandos nomeados por conteúdo (comandos continuam sendo
  filtrados só pelo rótulo — não faz sentido "buscar dentro" de um
  comando)
- Unificar a busca da sidebar E da paleta num componente só — mantém os
  dois, só faz a paleta USAR a mesma função de busca por baixo

## Notas

Reaproveita 100% infraestrutura já pronta (`api::search_content` +
`SearchIndex` FTS5 do ciclo 094, `render_excerpt_highlight` do ciclo
094/sidebar) — esse ciclo é praticamente só de "fiação" (wiring), não
precisa de infraestrutura nova.

`render_excerpt_highlight` virou `pub(crate)` em `sidebar.rs` e é
importada por `command_palette.rs` — sem duplicar a função nem criar
um módulo novo só pra isso.

Páginas que já batem por TÍTULO não aparecem de novo na seção de
conteúdo mesmo se o termo também aparecer no corpo (dedupe por
`path`) — evita listar a mesma página duas vezes.

Validado ao vivo via MCP `tauri`: buscar "Tauri" (não é título de
nenhuma página) retornou 6 resultados de conteúdo com o cabeçalho "No
conteúdo" e trechos com `<strong>` no termo destacado; buscar "ta" (2
chars) mostrou só comandos/títulos, sem seção de conteúdo — confirma
o gate de tamanho mínimo.

## Resultado

# Ciclo 102 - done

## Resumo

Quinto ciclo do tema "agent-os readiness". A paleta de comandos
(Ctrl+K) ganha busca de CONTEÚDO (não só título), reusando
`api::search_content`/`SearchIndex` FTS5 (ciclo 094) — mesma
infraestrutura que já alimenta a busca da sidebar. Query com 3+
caracteres dispara a busca assíncrona; resultados de conteúdo aparecem
numa seção "No conteúdo" separada, sem bloquear o match instantâneo
por título.

## Arquivos criados/modificados

- `ui/src/components/command_palette.rs` — `Item::ContentResult`,
  efeito de busca debounced por tamanho mínimo, dedupe contra títulos
  já encontrados, seção "No conteúdo"
- `ui/src/components/sidebar.rs` — `render_excerpt_highlight` virou
  `pub(crate)` (reusada, não duplicada)
- `ui/src/styles/components.css` — `.command-palette__section`,
  `.command-palette__item-result`, `.command-palette__item-excerpt`

## Testes

`cargo test --workspace`: 74 (inalterado, ciclo é só de UI).
`cd ui && cargo test --lib`: 66. Total 140.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: busca "Tauri" (não é título de
nenhuma página) retorna 6 resultados de conteúdo com trecho destacado;
busca "ta" (2 chars) não dispara busca de conteúdo.

## Notas

Ciclo praticamente só de wiring — infraestrutura de busca já existia
inteira desde o ciclo 094. Detalhes no arquivo de task.

Próximo: visibilidade de git read-only (103), fecha o tema "agent-os
readiness" (098-103).
