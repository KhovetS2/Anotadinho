---
title: Ciclo 111 — Corrige achatamento de tabela ao salvar
type: ciclo
ciclo: "111"
status: concluida
date: 2026-08-08
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 111 — Corrige achatamento de tabela ao salvar

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Corrige achatamento de tabela ao salvar

## Objetivo

Bug de perda de dado documentado desde o ciclo 099 e nunca corrigido:
salvar qualquer página com uma tabela Markdown (corpo solto, fora de
embed) achata a tabela pra texto corrido sem `|`, mesmo sem tocar em
nada relacionado a frontmatter — reproduzido com uma edição de corpo
comum + Salvar. Rastreado até o round-trip DOM→Markdown em
`recompute_markdown_from_dom`/`ui/src/html_to_md.rs`, que não trata
`<table>` corretamente ao reconverter a árvore do contenteditable de
volta pra markdown.

## Critérios de aceite

- [x] `ui/src/html_to_md.rs`: caso `"table"` serializa
      `<table><thead>...<tbody>...` de volta pro formato
      `| a | b |\n|---|---|\n| ... |`, preservando cabeçalho e todas as
      linhas
- [x] Teste automatizado NÃO escrito — ver Notas (sem infra de teste
      pra código que toca `web_sys::Element`/DOM neste crate); validado
      ao vivo via MCP `tauri` em vez disso, reproduzindo o cenário exato
      documentado no ciclo 099
- [x] Reprodução manual documentada no ciclo 099
      (`cycles/tasks/099-*.md`, seção Notas) deixa de reproduzir:
      abrir uma página com tabela, editar o corpo (fora da tabela),
      salvar, e o `.md` no disco continua com `|` intactos
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Suporte a tabelas com alinhamento de coluna (`:---:`) ou células com
  markdown inline complexo — só precisa preservar o que já existia
  antes do round-trip quebrar (cabeçalho + linhas simples)
- Editar tabelas fora de embed pela UI WYSIWYG (adicionar linha/coluna
  clicando) — isso já existe pro embed `{{ type: "table" }}`; tabela
  markdown solta no corpo é só texto, edição é digitando markdown
  mesmo

## Notas

Causa raiz confirmada: `walk()` não tinha nenhum case pra `"table"`,
então caía no branch `_` genérico, que recursa em `tr`/`th`/`td` como
elementos quaisquer — cada célula vira texto puro concatenado sem `|`
nem quebra de linha. Corrigido interceptando o `<table>` inteiro (sem
recursar via `walk` nos filhos): monta o cabeçalho via
`thead tr` e as linhas via `tbody tr` com `query_selector`/
`query_selector_all`, formatando cada linha como
`| célula | célula |`. Preserva formatação inline dentro de célula
(`text_of` ainda chama `walk` pros filhos de cada `<td>`/`<th>`) e
escapa `|` literal dentro de uma célula pra não quebrar a sintaxe.

Sem infraestrutura de teste automatizado pra esse arquivo (nenhum
código que toca `web_sys::Element` no crate `ui` tem teste — não há
`wasm-bindgen-test` configurado, e `cargo test --lib` roda nativo, sem
DOM disponível). Validação foi ao vivo via MCP `tauri`: abri
`VaultAnotadinho/pages/sobre.md` (tem uma tabela real em "## Stack"),
editei o parágrafo antes da tabela, salvei, e conferi o `.md` no disco
— a tabela veio de volta com todos os `|` intactos (antes da correção,
virava texto corrido). Mudança de teste revertida
(`git checkout -- VaultAnotadinho/pages/sobre.md`) antes de fechar o
ciclo.

## Resultado

# Ciclo 111 - done

## Resumo

Corrige bug de perda de dado documentado desde o ciclo 099 e nunca
corrigido: salvar qualquer página com uma tabela Markdown no corpo
achatava a tabela pra texto corrido sem `|`. `ui/src/html_to_md.rs`
não tinha nenhum case pra `"table"` — caía no branch genérico, que
concatenava o texto de `tr`/`th`/`td` sem separador. Novo case
`"table"` monta a tabela markdown direto a partir de `thead`/`tbody`
via `query_selector`, preservando cabeçalho, linhas e formatação
inline dentro de cada célula.

## Arquivos criados/modificados

- `ui/src/html_to_md.rs` — case `"table"` + helper `table_cell_texts`

## Testes

`cargo test --workspace`: 90. `cd ui && cargo test --lib`: 75. Total 165
(sem teste novo pra este arquivo — sem infra de teste DOM no crate `ui`,
ver Notas do arquivo de task).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: reproduzi o cenário exato do bug
(abrir `pages/sobre.md`, que tem uma tabela em "## Stack", editar o
parágrafo antes dela, salvar) — antes da correção isso achatava a
tabela; depois da correção, o `.md` no disco manteve todos os `|`
intactos.

## Notas

Mudança de teste no vault revertida
(`git checkout -- VaultAnotadinho/pages/sobre.md`) antes de fechar o
ciclo.

Próximo: placeholders de template além de `{{title}}` (112).
