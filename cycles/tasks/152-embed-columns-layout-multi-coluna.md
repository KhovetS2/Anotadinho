---
id: "152"
titulo: "Embed columns: layout multi-coluna"
status: done
criado: 2026-08-19
autor: humano
prioridade: media
depende_de: ["151"]
estima_min: 75
agente_alvo: claude-sonnet
---

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
