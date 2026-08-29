---
title: Estado capturado em closure
date: 2026-08-24
dominio: ui
tags:
- padrao
---
# Estado capturado em closure

## Quando se aplica

Toda vez que um valor é LIDO de dentro de um closure que roda depois:
efeito (`use_effect_with`), timer (`Interval`, `Timeout`), callback
assíncrono (`spawn_local`), ou handler registrado uma vez.

## A regra

Um handle de `use_state` capturado num closure fica **congelado** no
valor que tinha quando o closure foi criado. Ler dele ali devolve o
passado, não o presente.

- Para **escrever**, `use_state` serve: `handle.set(x)` funciona de
  qualquer lugar.
- Para **ler** um valor que muda entre a criação e a execução, use
  `use_mut_ref` (`Rc<RefCell<T>>`) e leia com `*r.borrow()`.

Sinal de alerta: se você está escrevendo `*alguma_coisa` dentro de um
`move ||` que vai rodar mais tarde, pare e confira de onde esse valor
vem.

## Por que existe

Este bug voltou cinco vezes, sempre com sintoma diferente:

- **155** — o `mouseup` do cronograma lia um handle congelado e
  commitava sempre 0 dias de arraste.
- **157** — mesma coisa noutra mutação de embed.
- **201** — o embed emitia `on_change` e `on_set_property` no mesmo
  tick; o segundo `set` apagava o primeiro e a etapa não avançava.
- **213** — a resposta do agente entrava a partir de uma lista velha e
  a pergunta recém-adicionada sumia.
- **218** — a trava do laço de acompanhamento lia `*ocupado` congelado
  em `false`; o laço parava pra sempre e a tela nunca mostrava
  "pensando".

## Como conferir

Um teste de unidade não pega: o defeito é de tempo, não de lógica. O
que pega é um cenário de harness que faça a coisa DUAS vezes seguidas,
ou que mude o estado entre a criação do closure e o disparo dele.
