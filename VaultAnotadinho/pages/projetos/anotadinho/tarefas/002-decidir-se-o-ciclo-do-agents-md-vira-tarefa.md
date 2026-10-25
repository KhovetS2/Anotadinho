---
title: Decidir se o ciclo do AGENTS.md vira tarefa
tags:
- tarefa
type: tarefa
date: 2026-10-22
projeto: anotadinho
status: pronta
---
# Decidir se o ciclo do AGENTS.md vira tarefa

## Objetivo

Decidir como o ciclo do `AGENTS.md` (task em `cycles/tasks`, validação,
status em `cycles/status`, commit `feat({id})`) convive com as tarefas
deste projeto.

## Critérios de aceite

- [ ] Está escrito qual é o registro de uma feature: a task do ciclo, a
      tarefa do projeto, ou as duas (e quem aponta pra quem).
- [ ] O `AGENTS.md` diz isso a qualquer agente que chegar no repositório.

## Abordagem

Hoje o `AGENTS.md` manda abrir `cycles/tasks/{id}-{slug}.md` pra cada
feature. Com a fila, a tarefa já guarda objetivo, critérios e relato. As
saídas possíveis: a execução da tarefa também gera a task e o status do
ciclo (os dois registros), ou a tarefa passa a ser o registro, e
`cycles/` vira histórico.

## Fora de escopo

- Converter os ciclos antigos em tarefas (ciclo 463, ambiguidade 2).
