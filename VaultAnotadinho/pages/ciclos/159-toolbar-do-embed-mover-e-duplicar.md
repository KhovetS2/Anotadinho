---
title: "Ciclo 159 — Toolbar do embed: mover e duplicar"
type: ciclo
ciclo: "159"
status: concluida
date: 2026-08-19
prioridade: baixa
depende_de: []
tags:
- ciclo
---

# Ciclo 159 — Toolbar do embed: mover e duplicar

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Toolbar do embed: mover e duplicar

## Objetivo

O `.embed-hover-wrapper` (ciclos 075 e 083) só sabe inserir uma linha
acima/abaixo e remover o embed. Com 9 tipos de embed depois desta
série, montar uma página vira um exercício de ordenação — e hoje
reordenar significa recortar e colar YAML na mão. Como o wrapper age
no nível do `DocSegment`, mover e duplicar valem pra todos os tipos de
uma vez, com o mesmo código.

## Critérios de aceite

- [x] Botões novos no wrapper: mover pra cima, mover pra baixo,
      duplicar
- [x] Mover troca o embed de posição com o segmento vizinho do mesmo
      nível no `Vec<DocSegment>` e re-`join`; o markdown entre os dois
      é preservado (o embed passa pro outro lado do trecho, o texto
      não some nem duplica)
- [x] Botão desabilitado quando não há pra onde mover (primeiro/último
      segmento)
- [x] Duplicar insere uma cópia idêntica logo abaixo, com um segmento
      separador entre as duas (`embed::BLANK_SEGMENT`). O separador NÃO
      sobrevive ao salvamento — ver Notas — mas isso não afeta o parse:
      os dois embeds continuam sendo dois
- [x] Ações refletem no editor ao vivo, sem precisar salvar (mesma
      regra do ciclo 079)
- [x] Undo (ciclo 095) desfaz mover e duplicar (as duas ações passam
      por `mark_edited`, que é onde o histórico de snapshots é gravado)
- [x] Os 4 botões viraram uma toolbar única (`.embed-hover-wrapper__
      toolbar`) no canto do embed, em vez do "×" solto que existia
      desde o ciclo 083 — as quatro ações agem no mesmo nível
      (segmento), então pertencem juntas. Foco visível em cada um
- [x] Testes puros sobre `Vec<DocSegment>`: mover primeiro pra cima é
      no-op, mover último pra baixo é no-op, mover no meio preserva
      todo o texto ao redor, duplicar não altera o original

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar o embed pra reordenar (drag-and-drop dentro do
  contenteditable conflita com seleção de texto — ver ciclo 068)
- Recortar/colar embed entre páginas diferentes

## Notas

`cargo test -p anotadinho-core`: 147 (143 + 4 novos). `cargo test
--workspace`: 255. `trunk build` e `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

Mover é uma TROCA com o segmento vizinho, não recorte-e-cola: por isso
nenhum texto se perde nem duplica, e mover e voltar devolve o
documento idêntico (tem teste). Efeito colateral esperado: ao mover um
embed pra fora do meio de dois blocos de texto, os dois blocos viram um
só na próxima leitura — é o mesmo markdown, com o embed noutro lugar.

O separador em branco criado pelo "duplicar" não sobrevive ao
salvamento: um segmento de markdown que só tem `\n` renderiza como
`<div>` vazio, e o recompute a partir do DOM devolve string vazia, que
`join` não escreve (comportamento de antes deste ciclo, documentado no
075). Os dois embeds ficam colados no arquivo mas continuam sendo dois
— o parser reconhece o fechamento seguido da abertura. Pra escrever
entre eles, os botões "+" de hover continuam sendo o caminho.

Validação ao vivo (MCP `tauri`) com dois callouts e três blocos de
texto: mover o primeiro pra baixo passou o embed pro outro lado do
texto do meio; mover de novo trocou a ordem dos dois embeds; duplicar
gerou um terceiro idêntico; salvo e RECARREGADO do disco com os 3
embeds e os 3 trechos de texto intactos.

Ícones novos em `icon.rs`: `arrow-up`, `arrow-down`, `copy`.

## Resultado

# Ciclo 159 - done

## Resumo

A barra de hover do embed (que desde o ciclo 083 só tinha "remover" e
os "+" de inserir linha) virou uma toolbar com mover pra cima, mover
pra baixo, duplicar e remover. Como as quatro agem no nível do
`DocSegment`, valem pros 9 tipos de embed com o mesmo código.

A aritmética mora no core, testada: `embed::move_segment` (troca com o
vizinho — por isso nada de texto se perde) e
`embed::duplicate_segment`.

## Arquivos criados/modificados

- `crates/core/src/embed.rs` — `BLANK_SEGMENT`, `move_segment`,
  `duplicate_segment` + 4 testes
- `ui/src/components/editor.rs` — `reorder_embed`, `duplicate_embed`,
  toolbar no lugar do botão solto
- `ui/src/components/icon.rs` — `arrow-up`, `arrow-down`, `copy`
- `ui/src/styles/main.css` — `.embed-hover-wrapper__toolbar` e `__btn`

## Testes adicionados

- mover troca com o vizinho preservando todo o texto (e mover + voltar
  devolve o documento idêntico)
- mover nas pontas e com índice inválido é no-op
- duplicar sobrevive a um round-trip pelo texto com DOIS embeds
  distintos (é onde um separador ausente os faria virar um só)
- duplicar com índice inexistente é no-op

## Problemas encontrados

- O separador em branco do "duplicar" não sobrevive ao salvamento
  (segmento de markdown vazio some no recompute a partir do DOM —
  comportamento anterior a este ciclo, documentado no 075). Os dois
  embeds continuam sendo dois; só não sobra linha clicável entre eles,
  e os botões "+" resolvem.

## Notas para próximos ciclos

- Falta 160 (painel do agent-os), que fecha a série, e as tasks de bug
  161/162 + a 163.
