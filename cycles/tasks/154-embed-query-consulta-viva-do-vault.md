---
id: "154"
titulo: "Embed query: consulta viva do vault"
status: done
criado: 2026-08-19
autor: humano
prioridade: alta
depende_de: ["148", "150"]
estima_min: 150
agente_alvo: claude-sonnet
---

# Embed query: consulta viva do vault

## Objetivo

A peça central da série pro agent-os. O esquema do
`guia-agent-os.md` (produto / specs / decisões / padrões, com
`status`, `priority`, `dominio` no frontmatter) só é navegável hoje na
mão ou pelo CLI — a única visão agregada dentro do app é o
[[Roadmap]], um kanban MANUAL que alguém precisa lembrar de mover.
Este embed torna qualquer recorte do vault uma view viva: "specs em
backlog ordenadas por prioridade", "decisões aceitas deste mês",
"páginas sem tag".

O motor de filtro nasce em `crates/core` porque o ciclo 158 expõe
exatamente ele no CLI — o agente headless enxerga o mesmo recorte que
o humano vê na página.

## Critérios de aceite

- [x] `crates/core/src/query.rs` (novo):
      `Query { from: Option<String>, tags: Vec<String>, conditions:
      Vec<Condition>, sort: Option<Sort { field, dir }>, limit:
      Option<usize>, view: QueryView (List|Table|Cards), columns:
      Vec<String> }`
      com `Condition { field, op: Eq|Neq|Contains|Exists|Gt|Lt, value }`
      e `fn run<'a>(&self, entries: &'a [PageIndexEntry]) -> Vec<&'a PageIndexEntry>`
- [x] `from` filtra por prefixo de path; `tags` é AND; `conditions`
      leem tanto os campos fixos (`title`, `path`, `type`) quanto
      qualquer chave do frontmatter; comparação de `Gt`/`Lt` é
      numérica quando os dois lados parseiam como número, senão
      lexicográfica (datas `YYYY-MM-DD` funcionam nos dois casos)
- [x] `EmbedKind::Query` + `{{ type: "query" }}`, serializando a
      `Query` direto (o YAML do embed É a consulta)
- [x] Componente `embeds/inline_query.rs`: consome `api::scan_vault`
      (ciclo 150) — UMA chamada, não N — e renderiza conforme `view`:
      `list` (título + subtítulo com os campos de `columns`), `table`
      (uma coluna por campo de `columns`) e `cards` (grade)
- [x] Clique numa linha/card abre a página (`on_page_selected`);
      teclado idem (`keyboard_activate`)
- [x] Botão de configuração abre modal no padrão de
      `embeds/column_settings_modal.rs`: pasta, tags, condições
      (adicionar/remover linha campo-operador-valor), ordenação,
      limite, modo de exibição e colunas
- [x] Rodapé mostra a contagem de resultados; estado vazio usa
      `components/empty_state.rs`
- [x] Somente leitura: editar a página continua sendo na página
- [x] Testes de `query.rs` no core (sem WASM): filtro por pasta, tag
      AND, cada operador, ordenação asc/desc com campo ausente indo
      pro fim, limite, e consulta vazia devolvendo tudo

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Escrita a partir da query (mudar `status` direto da lista) — o
  ciclo 156 (`actions`) cobre a parte de escrita por botão
- Agrupamento (`group by`) e agregados (soma/contagem por grupo)
- Expressões booleanas com OR/parênteses — as condições são AND, que
  é o que o esquema do agent-os precisa; OR fica pra depois se pedirem
- Reindexação incremental/cache

## Notas

`cargo test -p anotadinho-core`: 132 (119 + 12 de `query.rs` + 1 do
embed). `cargo test --workspace`, `cd ui && cargo test --lib` (26),
`trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Decisão de comportamento com teste dedicado: `!=` casa também com
página que NÃO TEM o campo. "Specs que não estão em done" tem que
incluir a spec sem `status` — é justamente o trabalho não
classificado, o que mais precisa aparecer. Ordenação faz o oposto:
página sem o campo vai pro fim nos dois sentidos (ela não é "a menor",
ela não participa).

Os campos oferecidos no modal saem do próprio vault (toda chave de
frontmatter/property vista no `scan_vault`), então dá pra configurar
sem decorar nome de campo.

**Bug pré-existente encontrado na validação:** Escape dentro de
qualquer modal fecha o modal E desseleciona a página aberta
(`Modal` não faz `stop_propagation`, o Escape sobe pro listener global
do `app.rs`). Virou a task 161 em vez de remendo aqui.

Validação ao vivo (MCP `tauri`): inserida por `/consulta`, nasce
mostrando o vault inteiro (10 de 24, limite 10); pasta `pages/specs`
reduziu pra 1; condição `status é backlog` manteve 1 e a descrição
virou "em pages/specs · status é backlog"; campos `status, priority` e
visão Tabela desenharam as colunas certas; salvo e RECARREGADO do
disco — consulta idêntica.

`PageIndexEntry` vem do ciclo 150 e mora em `crates/ipc`; se o `core`
não puder depender dele, mover a struct pro `core` como parte deste
ciclo (o `ipc` re-exporta) — o motor de query tem que ser testável sem
Tauri.
