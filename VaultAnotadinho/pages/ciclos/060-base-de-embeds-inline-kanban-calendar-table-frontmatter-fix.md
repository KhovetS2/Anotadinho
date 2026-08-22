---
title: Ciclo 060 — Base de embeds inline kanban calendar table + frontmatter fix
type: ciclo
ciclo: "060"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 060 — Base de embeds inline kanban calendar table + frontmatter fix

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Base de embeds inline kanban calendar table + frontmatter fix

## Objetivo

Corrigir o bug de `exemplos-embeds.md`: frontmatter YAML aparecendo como texto
solto e blocos ` ```kanban/calendar/table ` renderizando como texto cru. Ligar
`ui` ao `MarkdownCodec` já existente em `crates/core` (nunca foi usado pelo
frontend) para parsing de frontmatter correto, e construir uma camada base
de "embed" no `ui` que permita adicionar novos tipos de bloco inline sem
tocar no resto do sistema (1 variante de enum + 1 parser + 1 componente).

## Critérios de aceite

- [x] `crates/core::markdown::parse_blocks` trata fences multi-linha (` ``` `)
      como um único `Block`, com testes cobrindo `kanban`/`calendar`/`table`/sem linguagem
- [x] `ui` depende de `anotadinho-core` e compila para `wasm32-unknown-unknown` (`trunk build`)
- [x] `page_view.rs` usa `MarkdownCodec` para extrair `type:` do frontmatter (troca o `content.find("---")` na unha)
- [x] `markdown_render.rs` separa frontmatter do corpo antes de rodar `pulldown-cmark` — frontmatter nunca mais aparece como prosa em nenhuma página
- [x] Novo módulo `ui/src/embed.rs`: `EmbedKind`, segmentação por offset de fences via `pulldown-cmark`, parse/serialize por tipo
- [x] Componentes Yew interativos novos (`ui/src/components/embeds/`) para kanban/calendar/table, renderizados dentro de `editor.rs` como "ilhas" fora do fluxo `contenteditable`
- [x] Editar um embed (ex: mover card de coluna) regenera o texto daquele trecho e aciona o fluxo de salvar existente
- [x] `exemplos-embeds.md` renderiza corretamente no app rodando (validado via MCP) —
      confirmado numa sessão nova (ver ciclo 062): kanban/calendar/table renderizam
      como componentes reais, screenshot conferido. Essa mesma validação expôs 3 bugs
      que os testes unitários não cobriam (CSS faltando, Frontmatter.created,
      corrupção de newline no save) — corrigidos no ciclo 062.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cd ui && trunk build
```

## Não-objetivos

- Unificar os componentes inline com os componentes whole-page (`kanban.rs`/`calendar.rs`/`task_table.rs`, que leem `key:: value` do vault inteiro) — fontes de dados diferentes, fica pra depois
- Drag-and-drop nos embeds inline não é obrigatório nesta rodada se a arquitetura de componente Yew de verdade já estiver funcionando

## Notas

Ver `/home/elis/.claude/plans/jaunty-tinkering-beaver.md` (Workstream B) para
o desenho completo. `MarkdownCodec` (`crates/core/src/markdown.rs`) já é
testado e correto para frontmatter — reaproveitar, não reinventar.

## Resultado

# Ciclo 060 - done

## Resumo

Corrigido o bug de `exemplos-embeds.md` (frontmatter e blocos
```kanban/calendar/table``` renderizando como texto cru). `crates/core`
já tinha um `MarkdownCodec` correto e testado que o `ui` nunca usava — a
correção real foi ligar o frontend a ele (em vez de reinventar parsing de
frontmatter na unha, que existia em 2 lugares diferentes e de forma
incompleta) e construir uma camada nova (`ui/src/embed.rs`) que segmenta o
corpo de uma página em trechos de markdown comum + embeds reconhecidos,
renderizando os embeds como componentes Yew interativos de verdade (não
mais texto cru dentro de uma `<div>` inerte).

## Arquivos criados/modificados

- `crates/core/src/block.rs` — `BlockKind::Code` agora carrega a linguagem da fence (`Option<String>`)
- `crates/core/src/markdown.rs` — `parse_blocks` trata fences multi-linha como um bloco só;
  novo `MarkdownCodec::split_frontmatter_text` (frontmatter cru, sem parsear YAML, pra round-trip de save)
- `crates/core/src/lib.rs` — reexporta `MarkdownCodec`/`Frontmatter`
- `ui/Cargo.toml` — depende de `anotadinho-core`; `uuid` com feature `js` (RNG em wasm32); `serde_yaml`
- `ui/src/embed.rs` (novo) — `EmbedKind`, `DocSegment`, `EmbedData` (Kanban/Calendar/Table) com parse/serialize
- `ui/src/components/embeds/` (novo) — `InlineKanban`, `InlineCalendar`, `InlineTable`, dispatcher `InlineEmbed`
- `ui/src/components/editor.rs` — renderização segmentada quando a página tem embeds; save/export atualizados
- `ui/src/components/page_view.rs` — roteamento de `type:` via `MarkdownCodec` em vez de `content.find("---")`
- `ui/src/markdown_render.rs` — separa frontmatter antes de rodar pulldown-cmark
- `ui/src/lib.rs`, `ui/src/components/mod.rs` — registro dos módulos novos

## Testes adicionados

- `crates/core`: 6 testes novos (fences com/sem linguagem, conteúdo de fence não confundido com property,
  round-trip de serialize de fence, `split_frontmatter_text` com e sem frontmatter)
- `ui`: 7 testes novos em `embed::tests` (segmentação preserva texto ao redor, cada tipo de embed parseado
  corretamente, fence sem linguagem reconhecida não vira embed, round-trip de kanban)

## Problemas encontrados

- Verificação visual no app rodando não foi feita nesta rodada: os MCP servers `tauri`/`playwright` foram
  registrados via `claude mcp add` no meio desta sessão, e MCP registrado assim só fica disponível numa
  sessão nova. Fica pendente pro skill `ui-check`.

## Notas para próximos ciclos

- Os componentes whole-page (`kanban.rs`/`calendar.rs`/`task_table.rs`) continuam separados dos embeds
  inline de propósito (fontes de dado diferentes: página vs vault inteiro). Unificação é um possível
  ciclo futuro, não um requisito.
- Novo tipo de embed = 1 variante em `EmbedKind` + 1 parse/serialize em `EmbedData` + 1 componente em
  `components/embeds/` + 1 braço de match em `InlineEmbed` — nada mais precisa mudar.
