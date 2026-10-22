---
id: "463"
titulo: "Mover o processo do próprio Anotadinho para um projeto do módulo Projeto"
status: pending
criado: 2026-10-21
autor: agente
prioridade: media
depende_de: ["462"]
estima_min: 180
---

# 463 — O Anotadinho como projeto do próprio Anotadinho

Pedido direto da pessoa, durante o ciclo 461: "adicione também uma
tarefa pós finalização para mover o processo do anotadinho para um
projeto próprio". Fica pra DEPOIS do 462, quando o módulo Projeto estiver
completo: GUI, TUI, documentação e escala.

## Objetivo

Hoje, o desenvolvimento do Anotadinho mora espalhado:
- as specs e propostas ficam em `VaultAnotadinho/pages/specs` e
  `pages/propostas`;
- o histórico fica em `pages/ciclos` (168 páginas) e em `cycles/`, no
  git;
- a regra do processo fica em `AGENTS.md`.

O objetivo é que ele passe a ser um projeto em
`pages/projetos/anotadinho/`, com repositório = a raiz deste repo. O
trabalho novo nasceria como tarefa e iria pra execução pela fila, como
qualquer projeto.

## Critérios de aceite (a confirmar com a pessoa antes de executar)

- [ ] `pages/projetos/anotadinho/projeto.md` existe:
  - [ ] a visão e a stack vêm do `AGENTS.md`;
  - [ ] os "Combinados com o agente" trazem as regras que não se
        negociam e o ciclo de validação.
- [ ] `arquitetura.md` parte de `docs/architecture.md`.
- [ ] O que está em aberto vira tarefa, ligada à spec ou à proposta
      existente pelo `spec:`/`proposta:` da tarefa, SEM mover as
      páginas. Hoje são as 4 specs em rascunho e as 3 propostas em
      revisão.
- [ ] O repositório do projeto, em `.anotadinho/projetos.json`, aponta
      pra raiz do repo.
- [ ] `cli contexto` mostra o projeto.

## Ambiguidades (apontadas, não decididas)

1. **Mover ou ligar?** Mover `pages/specs` e `pages/propostas` pra
   dentro do projeto quebra os wikilinks e o `origem:` dos embeds de
   fluxo, que apontam pra esses caminhos. Ligar pela tarefa
   (`spec:`/`proposta:`) preserva tudo. A proposta deste ciclo deve
   escolher, e a recomendação é ligar.
2. **O que fazer com `cycles/` e `pages/ciclos`?** São histórico do que
   já foi feito. A leitura mais segura: ficam como estão, e só o trabalho
   NOVO nasce como tarefa. Se a ideia for converter o histórico em
   tarefas `feita`, isso é outro tamanho de trabalho (168 + 463 itens).
3. **O ciclo do `AGENTS.md` continua?** Hoje cada feature é uma
   task + status + commit `feat({id})`. Com a fila, a tarefa é o
   registro. É preciso decidir se a execução de uma tarefa ainda gera o
   par `cycles/tasks` + `cycles/status`, ou se o `AGENTS.md` passa a
   apontar pra tarefa. Isso muda o `AGENTS.md`, que é instrução de todo
   agente neste repo.
4. **O vault está DENTRO do repositório.** Com o `cwd` na raiz do repo, o
   agente da fila (Claude Code com `acceptEdits`) alcança
   `VaultAnotadinho/` direto pelo sistema de arquivos, sem passar por
   `propor` nem pelas permissões, que só valem pra quem propõe. Nos
   projetos de SaaS isso não acontece, porque o repositório é outro.
   Antes de ligar a fila neste projeto, é preciso decidir como proteger o
   vault: um `cwd` sem o vault (uma worktree?), uma regra no prompt de
   execução, ou aceitar o risco com revisão pelo git.

## Comandos de validação

```bash
./target/debug/anotadinho-cli --vault VaultAnotadinho contexto
./target/debug/anotadinho-cli --vault VaultAnotadinho projeto tarefas anotadinho
cargo test --workspace
```

## Não-objetivos

- Converter o histórico de ciclos em tarefas (ver ambiguidade 2)
- Mudar o fluxo de spec/proposta das páginas existentes
