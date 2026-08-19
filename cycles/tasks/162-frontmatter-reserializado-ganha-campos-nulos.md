---
id: "162"
titulo: "Frontmatter reserializado ganha campos nulos"
status: pending
criado: 2026-08-19
autor: agente
prioridade: media
depende_de: []
estima_min: 30
agente_alvo: claude-sonnet
---

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

- [ ] `Frontmatter` ganha `#[serde(skip_serializing_if =
      "Option::is_none")]` em `title`, `created`, `updated` e
      `page_type`
- [ ] `tags` vazio também não é escrito (`Vec::is_empty`)
- [ ] Teste em `crates/core`: `set_frontmatter_field` numa página que
      só tem `title` e `status` devolve frontmatter com só esses dois
      campos + o alterado — nenhum `null`
- [ ] Teste no CLI (`crates/cli/tests/cli.rs`): `set-property` não
      introduz chave nova além da alterada
- [ ] Conferir o painel de propriedades (`typed_page_header.rs`), que
      usa o mesmo `serde_yaml::to_string(&Frontmatter)` — deve parar de
      escrever nulos também
- [ ] Rodar em cima de uma página real do vault e comparar o `git diff`

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

Achado na validação do ciclo 156 (ação `set-property` do embed de
ações), mas é anterior: vale pra todo caminho tipado desde o ciclo
116. Virou task própria pela regra de isolamento do `cycles/README.md`.
