---
title: "Ciclo 152 — Embed columns: layout multi-coluna"
type: ciclo
ciclo: "152"
status: concluida
date: 2026-08-19
prioridade: media
depende_de: ["151"]
tags:
- ciclo
---

# Ciclo 152 — Embed columns: layout multi-coluna

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Embed columns: layout multi-coluna

## Objetivo

Markdown é linear: tudo empilha numa coluna só. Pra montar uma landing
page ou um painel (o caso do ciclo 160) é preciso colocar conteúdo lado
a lado. Este embed dá isso sem sair do arquivo `.md`: N colunas, cada
uma com seu próprio corpo markdown editável, reusando o
`EmbedMarkdownField` do ciclo 151.

## Critérios de aceite

- [x] `EmbedKind::Columns` + `{{ type: "columns" }}`
- [x] `ColumnsEmbedData { columns: Vec<ColumnPane { width: u8, body:
      String }> }`, com `width` em unidades de fração (default 1)
- [x] Componente `embeds/inline_columns.rs`: `display: grid` com
      `grid-template-columns` montado a partir dos `width` (`1fr 2fr`
      etc), cada painel com um `EmbedMarkdownField`
- [x] Botões de adicionar coluna (até 4) e remover coluna (mínimo 1);
      remover coluna com conteúdo pede confirmação via `PendingDialog::Confirm`
- [x] Ajustar a largura relativa de uma coluna (+/- no header do painel)
- [x] Empilha em coluna única abaixo de 700px (media query)
- [x] `data-nav-item`/`data-nav-group` nos painéis e controles
- [x] Testes: round-trip com 1/2/4 colunas, coluna vazia, e larguras
      assimétricas; `parse` de `columns: []` cai num default de 2
      colunas em vez de renderizar nada

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar a divisória pra redimensionar (o +/- resolve; o drag entra
  em conflito com o drag de seleção de texto dentro dos painéis — ver
  ciclo 068)
- Colunas aninhadas
- Embeds dentro de coluna

## Notas

`cargo test -p anotadinho-core`: 115 (110 + 5 novos). `cargo test
--workspace`, `cd ui && cargo test --lib` (26), `trunk build`,
`cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo (MCP `tauri`): inserido por `/colunas` com 2 painéis
1fr/1fr; alargar o primeiro deu `2fr 1fr`; adicionar deu `2fr 1fr 1fr`;
escrito `### Terceira` + `[[Missão]]` + `` `codigo: aqui` `` no painel
novo; salvo e RECARREGADO do disco — larguras, heading, wikilink
clicável e código inline voltaram idênticos.

Ícone novo em `icon.rs`: `layout` (dois painéis com divisória) — o
`columns` (3 barras iguais) já estava em uso pelo kanban.

O `width` fica em unidades inteiras de `fr` de propósito: mantém o
YAML legível pro agente (`width: 2`) e evita percentual que não fecha
em 100.

## Resultado

# Ciclo 152 - done

## Resumo

`{{ type: "columns" }}` — até 4 painéis markdown lado a lado, com
largura relativa em unidades de fração inteiras, cada painel usando o
`EmbedMarkdownField` do ciclo 151. É o que permite montar landing page
e painel (ciclo 160) sem sair do arquivo `.md`.

## Arquivos criados/modificados

- `crates/core/src/embed.rs` — `EmbedKind::Columns`, `ColumnPane`,
  `ColumnsEmbedData` (com `grid_template()`) + 5 testes
- `crates/core/src/index.rs` — braço do novo tipo
- `ui/src/components/embeds/inline_columns.rs` (novo)
- `ui/src/components/embeds/mod.rs` — registro + dispatcher
- `ui/src/components/icon.rs` — `layout`
- `ui/src/styles/main.css` — `.columns-embed*` + media query de 700px

## Testes adicionados

- round-trip com larguras assimétricas (e `grid_template()` = "2fr 1fr")
- `columns: []` cai em 2 painéis (embed sem painel some da tela)
- máximo de 4 painéis e mínimo de 1 respeitados
- largura limitada entre 1 e 6
- arquivo com painéis a mais é truncado no parse

## Problemas encontrados

- Nenhum novo. O `EmbedMarkdownField` do ciclo anterior entrou sem
  precisar de ajuste, que era a aposta de tê-lo extraído.

## Notas para próximos ciclos

- Divisória arrastável ficou fora de propósito (conflita com seleção de
  texto — ciclo 068). Se pedirem, vira task própria.
- Falta pra série de composição: `gallery` (153).
