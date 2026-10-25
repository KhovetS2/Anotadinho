---
title: Proposta de implementação — Módulo Projeto
tags:
- proposta
- agent-os
- projeto
type: proposta
date: 2026-10-21
status: concluida
---
# Proposta de implementação — Módulo Projeto
{{ type: "fluxo" }}
artefato: proposta
etapa: concluida
origem: pages/specs/modulo-projeto.md
nota: 'Executada nos ciclos 457–462 (+ 463, o Anotadinho como projeto). Núcleo `core::projeto` (etapas, fila por nível com reserva, prompts, índice de dependências); `ipc` compartilhado por janela, TUI e CLI; execução no repositório do projeto pelo `cwd` do adaptador; tela de Projeto e barra de etapa na janela; tela de Projeto na TUI; guia `como-usar-projetos` e regras 11–12 do padrão de agente. Desvios registrados e aprovados: pasta em `pages/projetos/` e fila por nível (sem `projetos-visto.json`). Validado com `cargo test --workspace`, harness da janela (269/271 no 460; 9/9 `projeto:` no 462), pty da TUI e bateria de escala linear até 2000 tarefas. Fora: arrastar cartão, barra de etapa dentro do editor da TUI, ordenar a fila pelo grafo de dependências.'
{{ /fluxo }}
## Abordagem

Montar o módulo sobre o que já existe, sem reescrever nada: conversa,
`envio::montar`, `fila::Fila`, `execucao`, `proposta` e `permissoes`, e os
dois executores do agente (o do Tauri e o `Trabalho` da TUI).

**Modelo no vault.** O projeto é uma pasta
`pages/projetos/<slug>/` (sob `pages/`, como todo o resto do vault —
emenda do ciclo 457) com estas páginas:

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

**Fila por nível, com reserva** (emenda do ciclo 457). A fila de um
projeto é simplesmente o que está em `executar`, com as dependências
`feita`, por prioridade e número (`core::projeto::fila`).
`ipc::handle_tarefas_a_executar` é compartilhado pela GUI e pela TUI. Ele
grava `executando` como reserva antes de devolver o trabalho e respeita a
pausa, a concorrência, as permissões e o freio. A checagem acontece na
hora, logo após uma mudança de etapa feita pela UI, e a cada 5 s pelo
mtime de `pages/projetos/*/tarefas/`.

A versão anterior desta proposta previa uma memória por borda
(`projetos-visto.json`). Escrevendo a função, apareceu que ela perde
pedido: uma tarefa posta em `executar` com a fila pausada ficava "vista"
e não rodava ao retomar. O que impede rodar duas vezes é a reserva.
Depois dela a tarefa sai da consulta, então a objeção ao
`Quando::Consulta` (disparar a cada minuto) não se aplica.

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
1. **ipc e CLI.** Os `handle_*` do projeto, a config, a fila por borda e
o fechamento. O CLI ganha `projeto novo|listar|tarefas`,
`tarefa nova` (que propõe) e `tarefa etapa`, e o `contexto` passa a
listar os projetos.
1. **Executar no repositório.** `cwd` por parâmetro nos dois executores,
uma fila por projeto e a avaliação na hora e a cada 5 s. Inclui o
`cargo check` no alvo Windows.
1. **GUI.** `ProjetoView` com cabeçalho, quadro por etapa e abas
Conversa, Arquitetura e Execuções. Ganha também a barra de etapa da
tarefa, os botões "virar tarefas" e "atualizar arquitetura" (que
propõem), a paleta e o harness.
1. **TUI.** `TipoEspecial::Projeto`, as teclas de etapa e de pausa, o
indicador de execução e a validação em pty.
1. **Docs e escala.** O guia de uso, o padrão atualizado e o estresse
com 1000 tarefas, com o teto visto reprovando antes de valer.

## Alternativas descartadas

- **Usar `gatilho::Quando::Consulta status=executar`.** O gatilho não
reserva a tarefa, então dispararia a cada minuto enquanto ela
estivesse em executar. A fila própria usa o mesmo nível, mas com a
reserva.
- **Memória por borda (`projetos-visto.json`).** Perde pedido quando a
fila está pausada (ciclo 457).
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
