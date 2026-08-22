---
title: Ciclo 063 — Embeds dinamicos kanban calendar table e modal proprio
type: ciclo
ciclo: "063"
status: concluida
date: 2026-08-06
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 063 — Embeds dinamicos kanban calendar table e modal proprio

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embeds dinâmicos kanban/calendar/table + modal próprio

## Objetivo

Depois de validar o fix dos embeds (ciclo 062), o usuário testou o kanban de
verdade e reportou que era estático (sem criar/editar/excluir card ou
coluna), que o drag-and-drop não funcionava de verdade, que faltavam
comandos de slash pra inserir novos embeds, e que toda edição usava
`window.prompt`/`confirm` nativos em vez de algo com a identidade visual do
Anotadinho. Pediu também tipos de célula configuráveis na tabela (estilo
Notion). Este ciclo entrega tudo isso.

## Critérios de aceite

- [x] Modal próprio (`ui/src/dialog.rs` + `ui/src/components/dialog_host.rs`)
      substitui os ~11 usos de `gloo_dialogs::prompt/confirm/alert` em
      `app.rs`/`sidebar.rs`/`editor.rs`/`inline_calendar.rs`
- [x] Kanban: criar/editar/excluir card, criar/renomear/excluir coluna,
      drag-and-drop reescrito com eventos de mouse (não mais HTML5 DnD
      nativo, instável no WebKitGTK)
- [x] Calendar: criar/editar/excluir evento (data + título, dois prompts encadeados)
- [x] Table: colunas tipadas (Texto/Seleção/Checkbox) via preâmbulo YAML
      opcional na fence (retrocompatível — tabela sem preâmbulo continua
      funcionando igual), criar/excluir linha e coluna, célula de Seleção
      vira badge colorido clicável
- [x] 3 comandos de slash novos (Kanban/Calendário/Tabela de Tarefas)
      inserindo um embed novo, dependendo do fix em `html_to_md.rs` (fence
      perdia a linguagem ao converter `<pre><code class="language-X">`)
- [x] Fix no guard do effect de render do editor (`editor.rs`) pra
      repovoar os trechos de markdown quando um embed é adicionado/removido
      na mesma sessão, não só ao trocar de página

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Excluir card/coluna/evento/linha tem confirmação (pedido do usuário), mas
  não tem "desfazer" — fica pra um ciclo futuro se precisar
- Tipos de coluna da tabela ficaram em Texto/Seleção/Checkbox (não incluí
  Número/Data como tipos dedicados — não pareciam agregar o suficiente sobre
  Texto sem validação/formatação real, que não foi pedida)

## Notas

Dois bugs sérios só apareceram na validação ao vivo via MCP (nenhum teste
unitário pegava):

1. `KanbanEmbedData::to_fence_body` não escapava os itens antes de
   serializar como YAML — um card com "#" no título (ex: "Revisar PR #42")
   virava comentário YAML e cortava o resto da linha, truncando o título E
   jogando o card pra coluna errada no reparse seguinte. `items` agora usa
   o mesmo `yaml_scalar()` já usado pela tabela.
2. `Modal` (`ui/src/components/modal.rs`) fechava sozinho em QUALQUER clique
   dentro dele, porque o clique borbulhava até o `onclick` de fechar do
   overlay — inofensivo pra diálogos simples (o fechamento duplicado não
   muda nada), mas destruía qualquer fluxo de diálogos encadeados (ex:
   nome → tipo → opções da coluna da tabela), já que o fechamento do
   overlay rodava DEPOIS e apagava o diálogo novo que tinha acabado de
   abrir. Fix: `stop_propagation` no conteúdo do modal.

Reforça a mesma lição do ciclo 062: `cargo test` sozinho não substitui
testar no app rodando de verdade.

## Resultado

# Ciclo 063 - done

## Resumo

Kanban/calendar/table inline deixam de ser estáticos: criar/editar/excluir
card, coluna, evento e linha, com um modal próprio (substitui os diálogos
nativos do navegador) e drag-and-drop reescrito com eventos de mouse em vez
do Drag and Drop HTML5 nativo (instável no WebKitGTK, era o motivo real do
"não funciona pra outras sessões" relatado). Tabela ganha colunas tipadas
(Texto/Seleção/Checkbox) com preâmbulo YAML opcional, retrocompatível. Três
comandos de slash novos pra inserir embeds.

## Arquivos criados/modificados

- `ui/src/dialog.rs` (novo) — enum `PendingDialog`
- `ui/src/components/dialog_host.rs` (novo) — renderiza o modal certo
- `ui/src/components/modal.rs` — fix de stop_propagation (bug real, ver Notas)
- `ui/src/styles/components.css` — CSS do modal
- `ui/src/styles/main.css` — CSS novo pra ações de kanban/calendar/table (add/edit/delete)
- `ui/src/embed.rs` — CRUD em `KanbanEmbedData`/`CalendarEmbedData`, `TableEmbedData`
  remodelada com `ColumnKind`/`TableColumn`, fix de escaping YAML nos itens do kanban
- `ui/src/components/embeds/inline_{kanban,calendar,table}.rs` — UI dinâmica completa
- `ui/src/components/editor.rs` — migração de `gloo_dialogs`, 3 slash items novos,
  fix do guard do effect de re-segmentação
- `ui/src/html_to_md.rs` — preserva linguagem da fence em `<pre><code class="language-X">`
- `ui/src/app.rs`, `ui/src/components/sidebar.rs`, `ui/src/components/page_view.rs` —
  prop `open_dialog` threading

## Testes adicionados

- `crates/core`: nenhum (mudanças todas em `ui`)
- `ui`: 5 testes novos em `embed::tests` — CRUD de kanban/calendar/table,
  round-trip de colunas tipadas, regressão do bug de `#` no YAML

## Problemas encontrados

Dois bugs sérios encontrados e corrigidos durante validação ao vivo via MCP
(nenhum teste unitário cobria nenhum dos dois) — detalhados em
`cycles/tasks/063-*.md`: escaping YAML faltando nos itens do kanban, e
`Modal` fechando diálogos encadeados por bubbling de clique.

## Notas para próximos ciclos

- MCP `tauri` (screenshot/DOM/JS real na janela) continua sendo o único
  jeito confiável de pegar esses bugs — `cargo test` não cobre interação
  entre componentes Yew e o DOM real.
- Reformatação cosmética de embeds não editados a cada save continua sendo
  um comportamento conhecido, não corrigido (mesma nota do ciclo 062).
