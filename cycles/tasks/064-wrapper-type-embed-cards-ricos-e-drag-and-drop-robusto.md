---
id: "064"
titulo: "Wrapper type embed cards ricos e drag and drop robusto"
status: done
criado: 2026-08-06
autor: humano
prioridade: media
depende_de: ["063"]
estima_min: 90
agente_alvo: claude-sonnet
---

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
