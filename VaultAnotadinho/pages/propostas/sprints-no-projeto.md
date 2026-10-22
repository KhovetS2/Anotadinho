---
title: Proposta de implementação — Sprints no projeto e rotas pra LLM
tags:
- proposta
- agent-os
- projeto
type: proposta
date: 2026-10-22
status: aprovada
---
# Proposta de implementação — Sprints no projeto e rotas pra LLM

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/specs/sprints-no-projeto.md
{{ /fluxo }}

## Abordagem

**A sprint é PÁGINA**, como a tarefa: `pages/projetos/<slug>/sprints/<n>-<slug>.md`,
com `type: sprint`, `projeto`, `estado` (`planejada|ativa|encerrada`),
`inicio`, `fim` (`AAAA-MM-DD`) e o objetivo no corpo. Assim ela aparece
em `query`, no `contexto` e na sidebar sem nada novo, e se edita como
qualquer nota.

**A tarefa ganha `sprint: <n>`** no frontmatter. Sem o campo, a tarefa
está no backlog. O número, e não o caminho, porque renomear a sprint não
pode soltar as tarefas.

**A regra fica no núcleo** (`core::projeto`), sem IO:
- `EstadoSprint` com as transições, e "uma ativa por projeto";
- `Filtro { SprintAtiva, Backlog, Sprint(n), Todas }` e
  `tarefas_do_quadro(tarefas, sprints, filtro)`. "Todas" exclui as
  feitas de sprints encerradas (RF5).
- `plano_de_encerramento(sprint, destino)`: as mudanças que encerrar
  causa, pra o `ipc` gravar de uma vez.
- `PlanoDeProjeto`, que é YAML com serde: `projeto`, `arquitetura`,
  `sprints[]`, `tarefas[]` (com `sprint`, `depende`, `prioridade` e as
  quatro seções). Junto vêm `montar_plano`, que produz as páginas com os
  números certos, e `MODELO_DE_PLANO`.

**IO no `ipc`**, compartilhado por janela, TUI e CLI:
- `handle_sprints_do_projeto` e `handle_criar_sprint`, que grava pela
  pessoa;
- `handle_iniciar_sprint` e `handle_encerrar_sprint(destino)`, que grava
  a sprint e as tarefas;
- `handle_mudar_sprint_da_tarefa`;
- `handle_propor_plano`, `handle_propor_sprint` e
  `handle_propor_tarefa`, os três por proposta;
- `EstadoDoProjeto` ganha as sprints.

**Rotas pra LLM:**
- No CLI:
  - `sprint nova|listar|iniciar|encerrar` (o que escreve, propõe);
  - `tarefa nova --sprint N`;
  - `tarefa sprint <caminho> N|backlog`;
  - `projeto modelo-de-plano`;
  - `projeto montar --file plano.yaml`.
- No MCP: `ver_projeto` (lê), `propor_plano_de_projeto`, `propor_sprint`
  e `propor_tarefa`. O `CONTRATO` cresce, e a invariante do ciclo 204
  continua valendo: só quem se chama `propor*` escreve, e declara
  revisão humana.
- `contexto` e `como_agir` mencionam as rotas, e um prompt padrão "Montar
  a base do projeto" pede o plano no formato do modelo.

**Telas:**
- **Janela:**
  - seletor de sprint no cabeçalho do projeto;
  - aba **Sprints**: as planejadas e a ativa, com criar, iniciar e
    encerrar (o diálogo pergunta o destino das pendentes), e o histórico
    das encerradas;
  - no cartão, "→ sprint" (mover pra sprint ou backlog).
- **TUI:**
  - `s` troca o filtro;
  - `S` move a tarefa selecionada de sprint;
  - a tela de sprints é uma lista na paleta, com iniciar e encerrar.

## Etapas (um ciclo cada)

1. **Núcleo:** sprint, filtro, encerramento e o plano com o modelo.
2. **ipc e CLI:** as rotas, incluindo `montar` e `modelo-de-plano`.
3. **MCP:** as quatro ferramentas, o contrato, `como_agir` e o prompt
   padrão.
4. **Janela:** seletor, aba Sprints, encerrar com pergunta e "→ sprint".
5. **TUI:** filtro, mover de sprint e a lista de sprints.
6. **Docs e escala:** o guia, e o estresse com 300 feitas em sprints
   encerradas (com o teto visto reprovando).

## Alternativas descartadas

- **Sprint dentro de `projeto.md`, como lista no frontmatter.** Seria
  uma página que cresce sem limite, editada por várias ações ao mesmo
  tempo, e a sprint não seria consultável sozinha.
- **Arquivar as feitas movendo arquivos.** Isso quebra links e caminhos
  de conversa (`tarefa-<n>`). Filtrar pela sprint encerrada dá o mesmo
  efeito sem mover nada.
- **Ferramenta MCP que grava direto.** Recusada na spec (RNF1).

## Riscos

- **Duas sprints ativas por edição à mão do arquivo.** O núcleo trata a
  mais recente como ativa e mostra aviso. Não trava.
- **O plano da LLM com números de tarefa colidindo com os existentes.**
  `montar_plano` renumera a partir de `proximo_numero`. As referências
  `depende` do plano são pelo título ou pela posição no plano, nunca
  pelo número.
