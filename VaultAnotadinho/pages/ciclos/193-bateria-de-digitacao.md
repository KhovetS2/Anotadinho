---
title: Ciclo 193 — Bateria de digitação no harness
type: ciclo
ciclo: "193"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: [177]
tags:
- ciclo
---

# Ciclo 193 — Bateria de digitação no harness

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Bateria de digitação no harness

## Objetivo

Rede de segurança PERMANENTE pro caminho mais usado do app, escrita
ANTES da reescrita do editor por bloco (ciclo 175). Sem ela, a reescrita
é uma aposta: digitação é a origem de quase todo bug do editor
(076, 078, 079, 082, 111, 141-143).

## Critérios de aceite

- [x] Arquivo próprio (`scripts/uitest/digitacao.mjs`), separado dos
      cenários de regressão, com a regra de mudança escrita no cabeçalho.
- [x] Cobre: digitar, Enter no fim, Enter no meio, Backspace fundindo,
      prefixos `#`/`##`/`-`/`>`, lista, checkbox, código em bloco e
      inline, ênfase, colar multilinha, Ctrl+Z.
- [x] Round-trip byte-idêntico de uma página não editada.
- [x] Roda dentro do `run.mjs`.

## Comandos de validação

```bash
node scripts/uitest/run.mjs digitação
node scripts/uitest/run.mjs
```

## Não-objetivos

- Testar o que ainda não existe: a bateria trava o comportamento ATUAL.

## Notas

**Dois bugs achados na primeira execução**, os dois corrigidos aqui:

1. Checkbox de lista de tarefas vinha `disabled` do pulldown-cmark, então
   `- [ ] x` escrito em markdown era somente leitura no editor — só o
   checkbox inserido pelo menu `/` funcionava.
2. Abrir e salvar sem editar nada somava uma linha em branco: o
   `inline_children` tratava como conteúdo o `\n` de formatação que o
   `set_inner_html` deixa entre blocos. Não acumulava, mas sujava o
   primeiro diff. Ao corrigir, apareceu que o heading emitia só um `\n`
   e dependia justamente desse espaço acidental.

**Duas lacunas registradas, não corrigidas** (não são regressão, são
funcionalidade que nunca existiu):

- Não há como digitar DEPOIS de um bloco de código que é o último
  elemento da página — o Enter cai dentro do `<pre>`.
- `ClipboardEvent` sintético não carrega `clipboardData` neste WebView,
  então colar de verdade não é testável por aqui; o cenário testa o
  caminho de inserção de texto multilinha, que é o que o `onpaste`
  acaba usando.

## Resultado

# 193 — Bateria de digitação

## O que mudou

- `scripts/uitest/digitacao.mjs` (novo): 15 cenários travando o
  comportamento atual de digitação, com um helper `caso()` que padroniza
  "parte deste markdown, faça isto, confira o arquivo salvo".
- `scripts/uitest/run.mjs`: a bateria entra junto dos cenários.
- `ui/src/markdown_render.rs`: `habilitar_checkboxes` + teste.
- `ui/src/html_to_md.rs`: espaço de formatação deixa de ser conteúdo;
  headings passam a emitir a linha em branco explicitamente.

## Bugs achados pela bateria (primeira execução)

1. **Checkbox somente leitura.** `- [ ] tarefa` vinha `disabled` do
   pulldown-cmark. O checkbox do menu `/` nasce sem, então a mesma coisa
   na tela tinha dois comportamentos.
2. **Linha em branco somada a cada abertura.** Abrir e salvar sem editar
   nada mudava o arquivo. Não acumulava (estabilizava em uma), mas o
   primeiro diff de qualquer página ficava sujo. A correção revelou que
   o heading dependia desse espaço acidental pra se separar do parágrafo
   seguinte.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 39 testes.
- `cargo build --manifest-path src-tauri/Cargo.toml`: 0 erros.
- `node scripts/uitest/run.mjs`: **43/43 em 242.6s** (28 cenários + 15
  da bateria).
