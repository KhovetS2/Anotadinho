---
title: Proposta de implementação — Módulo Projeto
tags:
- proposta
- agent-os
- projeto
type: proposta
date: 2026-10-21
status: em-revisao
---
# Proposta de implementação — Módulo Projeto

{{ type: "fluxo" }}
artefato: proposta
etapa: em-revisao
origem: pages/specs/modulo-projeto.md

{{ /fluxo }}

## Abordagem

Montar o módulo sobre o que já existe, sem reescrever nada: conversa,
`envio::montar`, `fila::Fila`, `execucao`, `proposta` e `permissoes`, e os
dois executores do agente (o do Tauri e o `Trabalho` da TUI).

**Modelo no vault.** O projeto é uma pasta
`projetos/<slug>/` com estas páginas:

- `projeto.md`, com `type: projeto`;
- `arquitetura.md`, com `type: docs`;
- `tarefas/<n>-<slug>.md`, com `type: tarefa`;
- `conversas/geral.md` e `conversas/tarefa-<n>.md`.

A etapa da tarefa fica no `status:` do frontmatter, então `query`,
`list-pages` e `TaskTable` a enxergam sem nada novo. O corpo tem quatro
seções fixas: Objetivo e Critérios de aceite (a parte do "o quê"),
Abordagem e Fora de escopo (a parte do "como").

**Config fora do vault.** `.anotadinho/projetos.json` guarda o
`repositorio`, `pausado` e `concorrencia` de cada projeto. Uma página não
pode decidir onde um agente com `acceptEdits` escreve.

**Disparo por borda.** `core::projeto::transicoes(visto, agora)` é pura e
compara com `.anotadinho/projetos-visto.json`.
`ipc::handle_tarefas_a_executar` é compartilhado pela GUI e pela TUI. Ele
grava `executando` como reserva antes de devolver o trabalho e respeita a
pausa, a concorrência, as permissões e o freio. A checagem acontece na
hora, logo após uma mudança de etapa feita pela UI, e a cada 5 s pelo
mtime de `projetos/*/tarefas/`.

**Execução.** O `cwd` entra por parâmetro em `disparar` e em
`Trabalho::iniciar`. O `mcp.json` continua apontando pro vault. O prompt
de execução trava a abordagem e exige no fim uma linha
`RESULTADO: concluida|bloqueada`. `handle_fechar_tarefa` grava o relato
na conversa da tarefa, registra a execução e move a tarefa para revisao
ou bloqueada.

## Etapas (um ciclo cada)

1. **Núcleo.** `crates/core/src/projeto.rs`: tipos, etapas e quem pode
   mover para cada uma, as páginas, o contexto do projeto, os prompts de
   detalhar e de executar, `ler_resultado` e `transicoes`. A semente
   ganha `projetos/` e o prompt "quebrar em tarefas".
2. **ipc e CLI.** Os `handle_*` do projeto, a config, a fila por borda e
   o fechamento. O CLI ganha `projeto novo|listar|tarefas`,
   `tarefa nova` (que propõe) e `tarefa etapa`, e o `contexto` passa a
   listar os projetos.
3. **Executar no repositório.** `cwd` por parâmetro nos dois executores,
   uma fila por projeto e a avaliação na hora e a cada 5 s. Inclui o
   `cargo check` no alvo Windows.
4. **GUI.** `ProjetoView` com cabeçalho, quadro por etapa e abas
   Conversa, Arquitetura e Execuções. Ganha também a barra de etapa da
   tarefa, os botões "virar tarefas" e "atualizar arquitetura" (que
   propõem), a paleta e o harness.
5. **TUI.** `TipoEspecial::Projeto`, as teclas de etapa e de pausa, o
   indicador de execução e a validação em pty.
6. **Docs e escala.** O guia de uso, o padrão atualizado e o estresse
   com 1000 tarefas, com o teto visto reprovando antes de valer.

## Alternativas descartadas

- **Usar `gatilho::Quando::Consulta status=executar`.** Ele é por nível,
  então dispararia a cada minuto enquanto a tarefa estivesse em executar.
- **Guardar o caminho do repositório em `projeto.md`.** Seria config
  dentro de uma página, o que o padrão de agente e execução proíbe.
- **Pôr tudo numa página só com embeds.** A página cresce sem limite e
  cada tarefa deixa de ser consultável por `status:`.
- **Exigir spec e proposta para toda tarefa.** Fica pesado demais para
  tarefas pequenas. O vínculo opcional cobre as grandes.

## Riscos

- **GUI e TUI no mesmo vault.** A reserva via `executando` e a releitura
  antes de iniciar protegem a fila, mas não o resto do app.
- **Mudança só no núcleo não chega à UI.** O `trunk serve` não observa
  `crates/`. Nos ciclos de núcleo, forçar o rebuild antes do harness.
- **Execução longa.** Vale o timeout padrão de 30 min e o botão de parar.
