---
id: "102"
titulo: "Busca full-text real na paleta de comandos"
status: done
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
