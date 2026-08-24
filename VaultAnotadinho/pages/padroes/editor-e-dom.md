---
title: Editor e DOM
date: 2026-08-24
dominio: ui
tags:
- padrao
---
# Editor e DOM

## Quando se aplica

Qualquer mudança em `ui/src/components/editor.rs`, nos embeds inline,
ou no caminho de digitação. É o código mais usado do app e a origem
histórica de quase todo bug.

## As regras

1. **Nada de `execCommand`.** Use `insert_embed_marker_at_cursor` ou
   `insert_element_at_cursor`. O `execCommand` corrompe o DOM do editor
   e ainda produz referência que não sobrevive ao save.
2. **Um `contenteditable` por bloco**, nunca aninhados. Editável dentro
   de editável dá comportamento errático de Enter.
3. **A fonte da verdade da digitação é o DOM**, e do embed é o YAML.
   Não misture: recompor markdown do DOM sobre um embed apaga o que o
   embed sabe.
4. **Inserir precisa de uma seleção real.** Sem `Range`, a inserção não
   tem onde acontecer e o efeito colateral (gravar o asset, por
   exemplo) já aconteceu.
5. **Decisão de apresentação que depende do documento mora no Rust**,
   não em CSS. `:only-child` conta filhos do SEGMENTO, não da página.

## Por que existe

- **076, 078, 079, 082, 111, 141-143** — bugs de digitação, todos de
  DOM, nenhum pego por `cargo test`.
- **175/194** — a reescrita por bloco deixou um caminho antigo com
  `contenteditable="true"`, produzindo editáveis aninhados.
- **194/195** — o placeholder aparecia no meio da escrita porque a
  decisão estava no CSS.
- **212** — lista numerada era o único bloco sem a margem negativa, e o
  realce caía por cima do marcador.

## Como conferir

Harness, sempre: `node scripts/uitest/run.mjs`. A bateria de digitação
(193) é rede permanente, não cenário pontual — se ela quebrar, pare.
