---
title: Sprints no projeto, e rotas pra LLM montar o projeto
tags:
- spec
- agent-os
- projeto
type: spec
date: 2026-10-22
prioridade: alta
status: aprovada
---
# Sprints no projeto, e rotas pra LLM montar o projeto

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada
{{ /fluxo }}

## Contexto

O quadro do projeto mostra TODAS as tarefas. Num projeto que cresce,
"Feita" acumula centenas de cartões, e o que está sendo atacado AGORA se
perde no meio. Ferramentas como o Jira resolvem isso com sprints: um
recorte com começo, fim e objetivo, e o quadro mostra só a sprint ativa.

Além disso, quem trabalha pelo Claude Code ou pelo opencode quer pedir à
LLM: "monte a base deste projeto" (projeto, arquitetura, sprints e
tarefas detalhadas), e hoje ela só tem o `propor` genérico, página a
página, adivinhando o formato de cada uma.

## Requisitos funcionais

- **RF1.** Uma sprint pertence a um projeto e tem nome, objetivo, data
  de início, data de fim e estado: planejada, ativa ou encerrada.
- **RF2.** Só uma sprint fica ativa por vez em cada projeto.
- **RF3.** Uma tarefa pertence a no máximo uma sprint. Tarefa sem
  sprint está no backlog.
- **RF4.** O quadro mostra por padrão a sprint ativa. Um seletor troca
  pro backlog, pra outra sprint ou pra "todas".
- **RF5.** As tarefas feitas de uma sprint encerrada nunca aparecem no
  quadro. Elas ficam no histórico da sprint.
- **RF6.** A pessoa cria, inicia e encerra sprints, e põe ou tira
  tarefas de uma sprint, pela janela e pela TUI.
- **RF7.** Ao encerrar uma sprint com tarefas não feitas, o app pergunta
  se elas vão pra próxima sprint planejada ou voltam pro backlog.
- **RF8.** O histórico mostra as sprints encerradas, cada uma com o
  objetivo, as datas e o que foi feito.
- **RF9.** Uma LLM consegue, pelo CLI e pelo MCP:
  - ver o estado de um projeto;
  - propor um projeto inteiro a partir de um plano (projeto,
    arquitetura, sprints e tarefas);
  - propor uma sprint;
  - propor uma tarefa, já numa sprint.
- **RF10.** O formato do plano é documentado e descobrível pela própria
  LLM: um comando mostra o modelo.

## Requisitos não funcionais

- **RNF1.** Tudo o que a LLM cria PROPÕE: entra como lote na revisão
  humana, como o resto do que o agente escreve. O plano inteiro é UM
  lote.
- **RNF2.** Mudar sprint de uma tarefa, ou o estado de uma sprint, é
  ação humana, como avançar etapa.
- **RNF3.** Com 300 tarefas feitas em sprints encerradas, o quadro da
  sprint ativa desenha só o que é dela, e o custo continua linear.
- **RNF4.** Um projeto sem nenhuma sprint funciona como hoje: o quadro
  mostra tudo.

## Critérios de aceite

- [ ] Iniciar uma sprint com outra já ativa é recusado, e o erro diz
      qual está ativa.
- [ ] Com uma sprint ativa, o quadro mostra só as tarefas dela. O
      seletor mostra o backlog e as outras.
- [ ] Encerrar pergunta o destino das pendentes e as move de uma vez. As
      feitas somem do quadro e aparecem no histórico.
- [ ] `anotadinho-cli projeto modelo-de-plano` imprime um plano de
      exemplo, e `projeto montar --file plano.yaml` propõe o projeto
      inteiro num lote só.
- [ ] O MCP expõe `ver_projeto`, `propor_plano_de_projeto`,
      `propor_sprint` e `propor_tarefa`. As três `propor_*` passam pela
      revisão.
- [ ] Um projeto sem sprints continua com o quadro de hoje.

## Fora de escopo

- Story points, velocidade e gráfico de burndown.
- Sprints de vários projetos ao mesmo tempo.
- Mudar a fila de execução: uma tarefa em `executar` roda esteja ou
  não na sprint ativa.
