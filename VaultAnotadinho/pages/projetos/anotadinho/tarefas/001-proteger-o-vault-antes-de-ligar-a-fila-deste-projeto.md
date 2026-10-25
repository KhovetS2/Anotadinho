---
title: Proteger o vault antes de ligar a fila deste projeto
tags:
- tarefa
type: tarefa
date: 2026-10-22
prioridade: alta
projeto: anotadinho
status: revisao
---
# Proteger o vault antes de ligar a fila deste projeto

## Objetivo

O agente da fila deste projeto roda com o `cwd` na raiz do repositório,
e o vault (`VaultAnotadinho/`) mora DENTRO dele. Com `acceptEdits`, ele
alcança as notas direto pelo disco, sem passar por `propor` nem pelas
permissões. Decidir o que fazer com isso.

## Decisão (2026-10-21)

**Aceitar o risco, sem conferência automática.** A pessoa decidiu que
uma conferência no fim de cada execução traria mais problema do que
resolve neste projeto. A regra "vault só por `propor`" fica como
instrução, e a revisão de uma escrita direta é pelo `git diff`.

A ressalva fica registrada: `.anotadinho/` está fora do git, então uma
mudança ali não aparece no diff.

## Critérios de aceite

- [x] A decisão está escrita nos combinados do projeto, com a ressalva
      do `.anotadinho/`.
- [x] A fila está retomada (`pausado: false` em `projetos.json`).

## Abordagem

As três opções levantadas no ciclo 463:

1. `cwd` numa worktree do git sem o vault: descartada.
2. Regra no prompt e conferência automática pelo `git status`:
   descartada a conferência; a regra já existe no prompt.
3. Aceitar o risco, com a revisão pelo diff: **escolhida**.

A primeira execução desta tarefa (conversa `tarefa-001`) terminou
`bloqueada` da forma certa: a tarefa era uma decisão, e o agente
apontou isso em vez de escolher.

## Fora de escopo

- Mudar o módulo Projeto pra outros projetos: lá o repositório é outro.
