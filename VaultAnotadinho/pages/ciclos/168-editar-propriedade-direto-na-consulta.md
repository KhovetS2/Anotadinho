---
title: Ciclo 168 — Editar propriedade direto na consulta
type: ciclo
ciclo: "168"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: ["154"]
tags:
- ciclo
---

# Ciclo 168 — Editar propriedade direto na consulta

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Editar propriedade direto na consulta

## Objetivo

O embed de consulta (154) é somente leitura por decisão daquele ciclo.
Na prática isso quebra o fluxo do painel: você vê a spec em `backlog`,
decide começar, e precisa abrir a página, achar o painel de
propriedades e voltar. "Ver" e "agir" ficam em lugares diferentes.

Este ciclo deixa editar, NA PRÓPRIA LINHA, os campos que a consulta já
mostra em `columns`.

## Critérios de aceite

- [x] Célula de um campo listado em `columns` vira editável no clique
      (e no Enter, pelo teclado), com o mesmo visual de célula da
      tabela embedada
- [x] A escrita passa por `MarkdownCodec::set_frontmatter_field` — o
      mesmo caminho do `anotadinho-cli set-property` e do embed de
      ações, sem um terceiro jeito de gravar frontmatter
- [x] Depois de gravar, a consulta reavalia: se a página deixou de bater
      com o filtro, ela sai da lista na hora (é o feedback de que a
      ação funcionou)
- [x] Os valores já usados naquele campo viram sugestão (`<datalist>`)
      no campo de edição — em vez de um seletor fechado, que impediria
      criar um valor novo (`blocked`, por exemplo) sem sair da lista
- [x] A gravação passa pela checagem de versão do ciclo 173, então
      página alterada por fora no meio da edição não é sobrescrita: o
      embed avisa e manda abrir a página
- [x] Erro de escrita aparece pro usuário, não em silêncio
- [x] Testes do motor: reavaliação depois da edição, e edição de campo
      que não está em `columns` não é oferecida

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Editar título/corpo da página pela consulta (só frontmatter)
- Criar página pela consulta (é o embed de ações)

## Notas

`cargo test --workspace`: 262. Harness (177): 10/10, com cenário novo
que cria uma spec em `backlog`, muda o status pela linha da consulta e
confere que ela SAI do recorte e que o `.md` no disco tem
`status: done`.

Uma armadilha do teste virou lição: a primeira versão do cenário
procurava o título "Spec de teste" em `document.body`, que casa com
coisa fora da lista. Assertion agora olha só as linhas da consulta.

Fecha o par com a task 163 (modal de configuração de botão): as duas
juntas tiram o painel do estágio "mostra e manda abrir" pra "resolve
ali".

## Resultado

# Ciclo 168 - done

## Resumo

A consulta deixou de ser só leitura: os campos listados em `columns`
viram célula editável na própria linha. Ver a spec em backlog e mudar o
status agora é um clique, sem abrir a página — e, ao gravar, a consulta
reavalia: se a página deixou de bater com o filtro, ela sai da lista na
hora.

## Arquivos criados/modificados

- `ui/src/components/embeds/inline_query.rs` — célula editável,
  gravação com checagem de versão (173), sugestões por `<datalist>`,
  linha de erro
- `ui/src/styles/main.css` — `.query-embed__editavel/__editar/__erro`
- `scripts/uitest/cenarios.mjs` — cenário novo

## Testes adicionados

- Cenário de harness: spec em backlog → editar status pela linha → sai
  do recorte e o `.md` fica com `status: done`

## Problemas encontrados

- A gravação reusa `MarkdownCodec::set_frontmatter_field` — o mesmo
  caminho do CLI e do embed de ações, sem um terceiro jeito de escrever
  frontmatter.
- Primeira versão do cenário procurava o título no `document.body`
  inteiro e dava falso negativo; passou a olhar só as linhas.

## Notas para próximos ciclos

- 163 (modal de botão do embed de ações) fecha o par "ver e agir".
