---
title: Ciclo 148 — Registro genérico de tipos de embed
type: ciclo
ciclo: "148"
status: concluida
date: 2026-08-19
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 148 — Registro genérico de tipos de embed

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Registro genérico de tipos de embed

## Objetivo

Primeiro ciclo da série 148-160 (novos embeds inline + interface de
agent-os). Hoje cada tipo de embed aparece hardcoded em 3 lugares
diferentes do editor: uma entrada em `SLASH_ITEMS` com uma sentinela
mágica (`__EMBED_KANBAN__`), um braço de `match` em `select_slash` com
o corpo YAML default cravado como string literal, e o `match` do
dispatcher. Antes de adicionar 6 tipos novos, esse custo por tipo tem
que cair: `EmbedKind` vira o registro único de metadados (label,
descrição, ícone, corpo default) e o menu `/` passa a se gerar a
partir dele.

## Critérios de aceite

- [x] `EmbedKind` (em `ui/src/embed.rs`) ganha:
      `all() -> &'static [EmbedKind]`, `label() -> &'static str`,
      `desc() -> &'static str`, `icon() -> &'static str` (nome de
      `components/icon.rs`) e `default_body() -> String` (o YAML/
      markdown inicial que hoje está literal em `select_slash`)
- [x] `default_body(today)` recebe a data de hoje de fora (`today_iso()`
      no editor) em vez de consultar o relógio: o corpo do calendário é
      o único não-constante, e a função precisa continuar pura pro
      ciclo 149 poder movê-la pro `anotadinho-core` (onde não existe
      `js_sys::Date`)
- [x] `SLASH_ITEMS` deixa de ter as 3 entradas de embed; o menu `/`
      concatena os itens estáticos com os gerados por
      `EmbedKind::all()` e a filtragem por texto continua funcionando
      sobre a lista concatenada
- [x] `select_slash` troca os 3 braços `__EMBED_*__` por um único
      braço que reconhece o prefixo `__EMBED__:<type>`, resolve o
      `EmbedKind` por `from_type_name` e chama
      `insert_embed_marker_at_cursor(kind.type_name(), &kind.default_body())`
- [x] O item do menu mostra o ícone do tipo (`<Icon name={kind.icon()} />`)
- [x] Testes: `from_type_name(k.type_name()) == Some(k)` pra todo `k`
      em `all()`; `EmbedData::parse(k, &k.default_body())` não entra em
      pânico e devolve dados não-vazios pra todo `k`
- [x] Inserir kanban, calendário e tabela pelo `/` continua funcionando
      igual (validado ao vivo via MCP `tauri`)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Adicionar tipo de embed novo — este ciclo só prepara o terreno
- Mexer em `segment`/`join`/formato de arquivo
- Mover `embed.rs` pra `crates/core` — é o ciclo 149

## Notas

Além do combinado: o menu `/` passou a mostrar um ícone por item (não só
nos embeds — ficaria inconsistente ter metade da lista com ícone). 8
ícones novos em `icon.rs`: `heading`, `list`, `quote`, `code`, `table`,
`minus`, `image`, `columns`.

`cd ui && cargo test --lib`: 88 passados (84 + 4 novos). `cargo test
--workspace`, `trunk build`, `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Validação ao vivo (MCP `tauri`): menu `/` com os 15 itens e ícone em
todos; os 3 itens de embed gerados de `EmbedKind::all()`; inserir
Kanban monta o board na hora; `/cal` filtra pra 1 resultado e insere o
calendário com "Novo evento" em 2026-08-19 (data de hoje resolvida por
`today_iso()`).

Depois deste ciclo, um embed novo custa: 1 variante em `EmbedKind` +
1 braço em cada `match` de metadado + 1 variante em `EmbedData` com
parse/serialize + 1 componente em `components/embeds/` + 1 braço no
dispatcher `InlineEmbed`. Nada no editor.

## Resultado

# Ciclo 148 - done

## Resumo

Primeiro ciclo da série 148-160 (novos embeds inline + interface de
agent-os). `EmbedKind` vira o registro único de metadados de embed
(`all`, `label`, `desc`, `icon`, `default_body`) e o menu `/` passa a
se gerar a partir dele: os 3 itens de embed saíram do `SLASH_ITEMS`
estático e as 3 sentinelas hardcoded (`__EMBED_KANBAN__` etc, cada uma
com o YAML inicial cravado literal no `select_slash`) viraram um braço
só, `__EMBED__:<type>`. Um tipo de embed novo agora não toca no
`editor.rs`.

De quebra o menu ganhou ícone por item (8 SVGs novos em `icon.rs`).

## Arquivos criados/modificados

- `ui/src/embed.rs` — metadados de `EmbedKind` + 4 testes
- `ui/src/components/editor.rs` — `slash_items()`, `SLASH_BLOCKS`,
  `EMBED_PREFIX`, `today_iso()`, braço único de embed
- `ui/src/components/icon.rs` — `heading`, `list`, `quote`, `code`,
  `table`, `minus`, `image`, `columns`
- `ui/src/styles/main.css` — `.slash-menu__item-icon`

## Testes adicionados

- `all_kinds_round_trip_pelo_nome_do_tipo`
- `all_kinds_tem_metadados_preenchidos`
- `default_body_de_todo_kind_parseia_em_dados_nao_vazios`
- `embed_com_default_body_sobrevive_a_segment_e_join`

## Problemas encontrados

- `default_body()` não podia consultar o relógio: o ciclo 149 move o
  módulo pro `anotadinho-core`, onde `js_sys::Date` não existe. Ficou
  `default_body(today: &str)`, com o editor passando `today_iso()` —
  função pura, atravessa a fronteira WASM sem mudança.
- `SlashItem` precisou deixar de ser `&'static` (o corpo do calendário
  é montado em runtime). A lista é reconstruída por render — 15
  structs pequenas, custo irrelevante perto de um re-render de Yew.

## Notas para próximos ciclos

- Custo de um embed novo agora: 1 variante em `EmbedKind` + 1 braço em
  cada `match` de metadado + parse/serialize em `EmbedData` + 1
  componente + 1 braço no dispatcher `InlineEmbed`.
- Próximo: 149 (move `embed.rs` pro core, destrava o CLI dos ciclos
  157/158).
