---
title: Ciclo 064 — Wrapper type embed cards ricos e drag and drop robusto
type: ciclo
ciclo: "064"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: ["063"]
tags:
- ciclo
---

# Ciclo 064 — Wrapper type embed cards ricos e drag and drop robusto

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Wrapper `{{ type: "..." }}`, cards ricos no kanban, drag-and-drop robusto

## Objetivo

Depois do ciclo 063 (embeds dinâmicos + modal), o usuário testou de novo e
trouxe 3 pontos: a fence ` ```kanban ``` ` colide semanticamente com blocos
de código de verdade; os cards do kanban precisavam ser mais ricos
(descrição, tags, vencimento, checklist, comentários, anexos, num modal
próprio com abas); e o drag-and-drop ainda não tinha feedback visual,
reordenação dentro da mesma coluna, nem confiabilidade quando o mouse é
solto fora do board. Este ciclo entrega os três.

## Critérios de aceite

- [x] Wrapper novo `{{ type: "kanban" }}` ... `{{ /kanban }}` (corpo YAML)
      substitui a fence ` ```kanban ``` ` — `ui/src/embed.rs::segment`
      reescrito como scanner de linhas por offset de byte, sem depender do
      parser de `CodeBlock` do pulldown-cmark
- [x] `KanbanCard` rico: `description`, `tags`, `due`, `checklist`,
      `comments`, `attachments`, todos opcionais via `#[serde(default)]`
      (card simples continua simples no YAML)
- [x] `KanbanEmbedData`/`CalendarEmbedData` migrados pra
      `#[derive(Serialize, Deserialize)]` + `serde_yaml` direto na struct —
      elimina a classe inteira do bug do ciclo 063 (`#` no título truncando
      o card), não só o caso pontual
- [x] `move_card(from_idx, to_column, before_card_idx)` unifica trocar de
      coluna e reordenar dentro da mesma coluna numa única operação
- [x] `CardDetailModal` novo (abas Detalhes/Comentários/Anexos), abre ao
      clicar num card; reaproveita `Modal` com uma variante `wide`
- [x] Drag-and-drop reescrito em `inline_kanban.rs`: classe
      `kanban__card--dragging` no card sendo arrastado, reordenação dentro
      da mesma coluna via `onmouseup` por card, e um listener de `mouseup`
      no `window` inteiro (via `gloo_events::EventListener`) que sempre
      zera o estado de arraste, mesmo se o mouse for solto fora do board
- [x] `markdown_render.rs`: removida a lógica morta de fence especial
      (`in_kanban`/`embed-*`) — fences ` ```kanban ``` ` voltam a ser
      blocos de código normais, sem tratamento especial
- [x] Slash commands (Kanban/Calendário/Tabela de Tarefas) reescritos pra
      inserir `<div data-embed-insert="X">` (convertido pro wrapper novo
      por `html_to_md.rs`) em vez do `<pre><code class="language-X">`
      antigo, que não seria mais reconhecido como embed
- [x] `VaultAnotadinho/pages/exemplos-embeds.md` migrada pro wrapper novo,
      com um card rico de exemplo (descrição/tags/vencimento/checklist)

## Comandos de validação

```bash
cd ui && cargo test --lib embed::
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
cargo test --workspace  # crates/core, vault, search, ipc (ui/ tem Cargo.lock próprio)
```

## Não-objetivos

- Anexos ainda usam `PendingDialog::Prompt` pedindo o caminho do arquivo
  (sem um seletor de arquivo nativo) — igual ao slash command de imagem já
  existente, consistente com o resto do editor
- Não adicionei "desfazer" pro drag-and-drop nem pra edições do card —
  mesma decisão do ciclo 063, fica pra um ciclo futuro se precisar

## Notas

Validação ao vivo via MCP `tauri` (mesmo fluxo dos ciclos 062/063) pegou um
detalhe de timing que não é bug do app, mas importante pra testes futuros:
disparar `mousedown` e `mouseup` sintéticos no MESMO script (mesmo tick)
faz o `mouseup` ler o valor ANTIGO do `use_state` de `dragging` — Yew só
re-renderiza (e rebinda os closures dos eventos) de forma assíncrona, então
dois eventos síncronos back-to-back nunca veem a atualização um do outro.
Cliques reais de mouse não têm esse problema (mousedown/mouseup são tasks
separadas do event loop, com tempo de sobra pra Yew re-renderizar entre
os dois). Testes de drag precisam disparar mousedown e mouseup em
chamadas de ferramenta SEPARADAS, não no mesmo `webview_execute_js`.

Com isso corrigido, validado ao vivo: clique no card abre o modal com
descrição/tags/vencimento/checklist corretos; adicionar comentário
persiste e atualiza o badge do card; mover card pra outra coluna e
reordenar dentro da mesma coluna funcionam; salvar e recarregar a página
preserva tudo; o slash command de kanban gera um wrapper
`{{ type: "kanban" }}` válido que reparseia corretamente.

## Resultado

# Ciclo 064 - done

## Resumo

Substitui a fence ` ```kanban ``` ` (colidia com blocos de código de
verdade) por um wrapper próprio `{{ type: "kanban" }}` ... `{{ /kanban }}`
com corpo YAML, migrando `KanbanEmbedData`/`CalendarEmbedData` pra
`serde_yaml` derive puro (elimina a classe de bug do `#`-como-comentário do
ciclo 063). Kanban ganha cards ricos (descrição, tags, vencimento,
checklist, comentários, anexos) com um modal próprio de abas
(Detalhes/Comentários/Anexos). Drag-and-drop reescrito: feedback visual no
card sendo arrastado, reordenação dentro da mesma coluna, e um listener
global de `mouseup` no `window` pra nunca deixar o estado de arraste preso.

## Arquivos criados/modificados

- `ui/src/embed.rs` — `segment()` reescrito como scanner de linhas (não
  mais fence-based), `KanbanCard` rico com `ChecklistItem`/`Comment`/
  `Attachment`, `move_card`, `update_card`, migração de Kanban/Calendar pra
  `serde_yaml` derive direto
- `ui/src/components/embeds/card_detail_modal.rs` (novo) — modal de
  detalhes do card com abas
- `ui/src/components/embeds/inline_kanban.rs` — reescrito: usa `KanbanCard`,
  abre o modal de detalhes ao clicar no card, drag-and-drop com feedback
  visual + reordenação + listener global de `mouseup`
- `ui/src/components/embeds/mod.rs` — `InlineEmbedProps` ganha `vault_path`
- `ui/src/components/editor.rs` — slash items de embed reescritos pra
  inserir `data-embed-insert` em vez de `<pre><code class="language-X">`
- `ui/src/html_to_md.rs` — reconhece `data-embed-insert` e emite o wrapper
  `{{ type: "X" }}` direto
- `ui/src/markdown_render.rs` — remove tratamento morto de fence especial
- `ui/src/components/modal.rs` — prop `wide` pro modal de detalhes do card
- `ui/src/styles/main.css`, `ui/src/styles/components.css` — CSS novo pro
  card (`kanban__card-main/meta/badge/tags`, `kanban__card--dragging`) e
  pro modal de detalhes (`card-modal__*`, `modal--wide`)
- `VaultAnotadinho/pages/exemplos-embeds.md` — migrada pro wrapper novo

## Testes

`cargo test --lib embed::` (ui/): 24 passaram, 0 falharam — inclui um teste
novo (`exemplos_embeds_vault_file_parses`) que carrega a página de demo do
vault direto via `include_str!` pra garantir que a sintaxe escrita à mão
não diverge do parser.

`cargo test --workspace` (core/vault/search/ipc): todos passaram (sem
mudança nesses crates neste ciclo).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

- Card com descrição/tags/vencimento/checklist renderiza os badges certos
  no board e abre o modal de detalhes correto ao clicar
- Adicionar comentário via modal persiste e atualiza o badge de comentários
  no card
- Mover card pra outra coluna (drag) e reordenar dentro da mesma coluna
  (drag) funcionam
- Salvar e recarregar a página preserva tudo, incluindo o card rico
- Slash command "Kanban" insere um wrapper `{{ type: "kanban" }}` válido
  que reparseia corretamente após salvar
- `+ coluna` abre o diálogo "Nova coluna" corretamente (cancelado sem
  efeito colateral, pra não sujar a página de demo)

## Notas

Lição de teste (não é bug do app): disparar `mousedown` e `mouseup`
sintéticos no MESMO script faz o handler de `mouseup` ler o valor ANTIGO
do `use_state` de `dragging`, porque Yew só re-renderiza (e rebinda os
closures dos eventos) de forma assíncrona — os dois eventos síncronos
nunca veem a atualização um do outro. Cliques reais de mouse não têm esse
problema, já que mousedown/mouseup são despachados como tasks separadas do
event loop do browser, com tempo de sobra pra Yew re-renderizar entre os
dois. Testes de drag via MCP precisam disparar mousedown e mouseup em
chamadas de ferramenta SEPARADAS.
