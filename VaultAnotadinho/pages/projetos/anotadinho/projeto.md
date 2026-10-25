---
title: Anotadinho
tags:
- projeto
type: projeto
date: 2026-10-22
projeto: anotadinho
---
# Anotadinho

## Visão

Um editor de notas markdown que também é o painel do próprio
desenvolvimento. As notas moram em `VaultAnotadinho/`, dentro do
repositório, e o agente trabalha nelas por proposta.

## Objetivo

Construir o Anotadinho com o próprio módulo Projeto: o que está em
aberto vira tarefa aqui, com o histórico de antes preservado em
`cycles/` e `pages/ciclos`.

## Stack

| Camada | Tecnologia |
|---|---|
| Runtime | Tauri 2.x |
| Backend | Rust (workspace em `crates/`) |
| UI | Yew 0.21 + WASM, servido por `trunk` |
| TUI | ratatui (`crates/tui`) |
| Testes | `cargo test --workspace` + harness de DOM (`scripts/uitest/`) |

## Combinados com o agente

Valem pra TODA tarefa deste projeto:

- **Siga o `AGENTS.md` da raiz.** Ele é a regra, e as páginas de
  `pages/padroes/` são as cicatrizes. Anexe as do assunto.
- **Nada de escrever em `VaultAnotadinho/` pelo disco.** O vault muda só
  por `anotadinho-cli propor`, e isso vale pra `.anotadinho/` também
  (propostas, `projetos.json`, permissões).
- **Decisão sobre o vault (tarefa 001, 2026-10-21): aceitar o risco.**
  O repositório deste projeto contém o vault, e o agente com
  `acceptEdits` CONSEGUE escrever nele pelo disco. A regra acima é
  instrução, não impedimento. Não há conferência automática depois da
  execução: ela traria mais problema do que resolve neste projeto.
  - A revisão de uma escrita direta é pelo `git diff` de
    `VaultAnotadinho/pages/`.
  - `.anotadinho/` está no `.gitignore`, então uma mudança ali NÃO
    aparece no diff. Na dúvida, confira `projetos.json` e
    `permissoes.json` à mão.
  - O risco não é da fila: conversas comuns já rodavam o agente na raiz
    deste repositório desde o ciclo 216.
- **Valide antes de dizer que terminou:** `cargo test --workspace`,
  `trunk build` e o build do Tauri. Se mexeu em UI, o harness; se mexeu
  em caminho do sistema, o `cargo check` do Windows.
- **Não abra o app.** Quem deixa o app de pé é a pessoa.
- **Commit** segue o histórico: `feat({id}): …` ou `fix({id}): …`.
