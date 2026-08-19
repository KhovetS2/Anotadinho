---
id: "148"
titulo: "Registro genérico de tipos de embed"
status: done
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
