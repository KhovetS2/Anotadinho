---
id: "193"
titulo: "Bateria de digitação no harness"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: [177]
estima_min: 120
agente_alvo: claude-opus
---

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
