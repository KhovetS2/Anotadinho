---
id: "159"
titulo: "Toolbar do embed: mover e duplicar"
status: done
criado: 2026-08-19
autor: humano
prioridade: baixa
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
