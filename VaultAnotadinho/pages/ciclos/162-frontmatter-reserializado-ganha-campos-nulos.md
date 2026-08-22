---
title: Ciclo 162 — Frontmatter reserializado ganha campos nulos
type: ciclo
ciclo: "162"
status: concluida
date: 2026-08-19
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 162 — Frontmatter reserializado ganha campos nulos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Frontmatter reserializado ganha campos nulos

## Objetivo

Bug pré-existente, encontrado na validação ao vivo do ciclo 156.
Toda vez que uma página passa pelo caminho TIPADO de reserialização de
frontmatter — `MarkdownCodec::set_frontmatter_field` (usado pelo
`anotadinho-cli set-property`, ciclo 116, e agora pela ação
`set-property` do embed de ações) — os campos opcionais não
preenchidos aparecem no arquivo como `null`:

```yaml
title: Spec de Teste
created: null      # ← não estava lá antes
updated: null      # ← não estava lá antes
type: null         # ← não estava lá antes
status: done
```

Causa: `Frontmatter` (`crates/core/src/page.rs`) declara `title`,
`created`, `updated` e `page_type` como `Option<String>` SEM
`skip_serializing_if`, então `serde_yaml` escreve `null` pra cada um.

Além de poluir o arquivo e sujar o `git diff`, `type: null` é uma
chave que o `PageIndexEntry` e o motor de consulta (ciclo 154) veem
como campo presente com valor vazio.

## Critérios de aceite

- [x] `Frontmatter` ganha `#[serde(skip_serializing_if =
      "Option::is_none")]` em `title`, `created`, `updated` e
      `page_type`
- [x] `tags` vazio também não é escrito (`Vec::is_empty`)
- [x] Teste em `crates/core`: `set_frontmatter_field` numa página que
      só tem `title` e `status` devolve frontmatter com só esses dois
      campos + o alterado — nenhum `null`
- [x] Teste no CLI (`crates/cli/tests/cli.rs`): `set-property` não
      introduz chave nova além da alterada
- [x] Conferir o painel de propriedades (`typed_page_header.rs`), que
      usa o mesmo `serde_yaml::to_string(&Frontmatter)` — deve parar de
      escrever nulos também
- [x] Rodar em cima de uma página real do vault e comparar o `git diff`

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo test -p anotadinho-cli
cd ui && trunk build
```

## Não-objetivos

- Mudar o formato do frontmatter (ordem das chaves, estilo de lista)
- Migrar páginas que já ficaram com `null` gravado — some sozinho na
  próxima escrita, depois deste conserto

## Notas

`cargo test -p anotadinho-core`: 148 (+1). `cargo test -p
anotadinho-cli`: 33 (+1).

Conferido num arquivo real do vault (`pages/specs/exemplo-exportar-
nota-em-pdf.md`): depois do conserto, o `git diff` de um `set-property`
não tem mais nenhum `null` nem campo inventado.

**Fica um resíduo conhecido**: o caminho tipado reordena o frontmatter
(campos `extra` saem em ordem alfabética, depois dos tipados), então o
diff mostra as linhas movendo de lugar mesmo quando só um valor mudou.
Some quem quiser resolver isso vai precisar de edição em nível de
linha em vez de round-trip tipado — é outro ciclo, e o incômodo é
bem menor que o dos nulos.

Achado na validação do ciclo 156 (ação `set-property` do embed de
ações), mas é anterior: vale pra todo caminho tipado desde o ciclo
116. Virou task própria pela regra de isolamento do `cycles/README.md`.

## Resultado

# Ciclo 162 - done

## Resumo

`Frontmatter` deixou de escrever `created: null`, `updated: null` e
`type: null` em toda página que passa pelo caminho tipado — o
`set-property` do CLI e a ação `set-property` do embed de ações. Além
de sujar o arquivo, `type: null` virava chave presente no índice de
consulta.

## Arquivos criados/modificados

- `crates/core/src/page.rs` — `skip_serializing_if` em `title`,
  `created`, `updated`, `page_type` e `tags`
- `crates/core/src/markdown.rs` — teste novo
- `crates/cli/tests/cli.rs` — teste novo

## Testes adicionados

- `set_frontmatter_field` não inventa campo nulo nem perde o corpo
- `set-property` pelo CLI não introduz chave nova

## Problemas encontrados

- Resíduo conhecido: o round-trip tipado REORDENA o frontmatter (campos
  livres saem alfabéticos, depois dos tipados). O diff mostra linhas
  mudando de lugar; resolver exigiria edição por linha, não round-trip.

## Notas para próximos ciclos

- Se o reordenamento incomodar, vira task própria.
