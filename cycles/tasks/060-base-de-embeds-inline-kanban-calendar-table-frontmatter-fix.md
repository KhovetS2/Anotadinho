---
id: "060"
titulo: "Base de embeds inline kanban calendar table + frontmatter fix"
status: done
criado: 2026-08-06
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
- [ ] `exemplos-embeds.md` renderiza corretamente no app rodando (validado via MCP) —
      **não verificado ao vivo nesta rodada**: os MCP servers `tauri`/`playwright`
      foram registrados (`claude mcp add`) só durante esta sessão, e MCP
      registrado a meio de sessão só fica disponível numa sessão nova. Validado
      via `cargo test --workspace` (20 testes em `anotadinho-core`, incluindo
      round-trip de fence, e 7 testes de segmentação em `anotadinho-ui`) +
      `trunk build`/`cargo build` limpos. Verificação visual fica pendente pra
      quando o skill `ui-check` puder rodar com os MCP tools carregados.

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
