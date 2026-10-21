---
title: Módulo Projeto — construir um projeto com a LLM num lugar só
tags:
- spec
- agent-os
- projeto
type: spec
date: 2026-10-21
prioridade: alta
status: em-revisao
---
# Módulo Projeto — construir um projeto com a LLM num lugar só

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao

{{ /fluxo }}

## Contexto

Montar um projeto (um SaaS, por exemplo) com a LLM hoje exige costurar
peças espalhadas: a conversa, a spec, a proposta, a execução, um kanban
que só lê e não tem `status:`, os gatilhos e a fila. E o código do
projeto mora fora do vault, num repositório próprio, que o agente não
sabe onde fica.

A pessoa quer um lugar único onde conversa com a LLM sobre o projeto, vê
a arquitetura e as tarefas detalhadas nascerem dessa conversa, e manda
executar uma tarefa só mudando o status dela.

## Requisitos funcionais

- **RF1.** A pessoa cria um projeto pela GUI, pela TUI ou pelo CLI. Ele
  nasce como uma pasta própria no vault com: página do projeto (visão,
  objetivo, stack), página de arquitetura, uma pasta de tarefas e uma
  conversa geral.
- **RF2.** Cada projeto aponta para UM repositório externo. O caminho é
  escolhido pela pessoa e não fica gravado numa página do vault.
- **RF3.** A página do projeto abre uma tela de Projeto que reúne, sem
  trocar de página: as tarefas agrupadas por etapa, a conversa com a LLM
  e a arquitetura.
- **RF4.** A conversa do projeto leva automaticamente como contexto a
  página do projeto, a arquitetura e um índice das tarefas, com opção de
  desligar.
- **RF5.** Uma resposta da LLM pode virar uma ou várias tarefas, ou uma
  mudança na arquitetura. Isso sempre entra como proposta para revisão,
  nunca é gravado direto.
- **RF6.** Uma tarefa tem: objetivo, critérios de aceite verificáveis,
  abordagem, fora de escopo, prioridade, dependências e, opcionalmente, um
  vínculo com uma spec ou proposta que já existe.
- **RF7.** Uma tarefa passa por estas etapas: ideia, detalhando, pronta,
  executar, executando, revisão, feita e bloqueada. Cada etapa diz quem
  pode levar a tarefa até ela.
- **RF8.** Quando a pessoa põe uma tarefa em **executar**, ela entra
  numa fila do projeto. Quando chega a vez, o agente configurado trabalha
  nela dentro do repositório do projeto.
- **RF9.** Ao terminar, a tarefa vai para **revisão**, com o relato do
  agente registrado numa conversa da própria tarefa. Se o agente não
  conseguiu, ela vai para **bloqueada**, com o motivo.
- **RF10.** A pessoa pode pausar e retomar a fila de um projeto, e vê o
  que está rodando e o que está esperando.
- **RF11.** A tarefa entra na fila uma única vez por mudança de etapa,
  não importa por qual canal a etapa mudou: GUI, TUI, CLI ou edição do
  arquivo.
- **RF12.** Tudo isso funciona na GUI e na TUI.

## Requisitos não funcionais

- **RNF1.** Só humano leva uma tarefa a ideia, detalhando, pronta,
  executar e feita. O sistema só registra executando, revisão e bloqueada.
  O agente nunca grava etapa.
- **RNF2.** O agente continua sem escrever direto no vault: tudo passa
  por `propor` e pelas permissões por pasta.
- **RNF3.** Uma tarefa nunca é executada duas vezes por um único pedido,
  nem com GUI e TUI abertas ao mesmo tempo.
- **RNF4.** Por padrão, um projeto executa uma tarefa por vez.
- **RNF5.** Cada execução aparece no registro de execuções e respeita o
  freio do dia.
- **RNF6.** Com 1000 tarefas no projeto, a checagem do que executar
  continua com custo linear.

## Critérios de aceite

- [ ] `projeto novo` cria a pasta com as quatro páginas e não sobrescreve
  uma que já exista.
- [ ] Mudar uma tarefa para executar, por qualquer canal, faz ela passar
  para executando uma única vez, e o processo do agente roda com o
  diretório de trabalho no repositório do projeto.
- [ ] Uma resposta sem a linha de resultado deixa a tarefa em bloqueada,
  e não em revisão.
- [ ] Com a fila pausada, nenhuma tarefa sai de executar.
- [ ] Uma resposta com três tarefas vira um lote de três propostas.
- [ ] A tela de Projeto existe na GUI e na TUI e só oferece as transições
  válidas.

## Fora de escopo

- Ordenar a fila automaticamente pelo grafo de dependências. Uma tarefa
  com dependência pendente só espera.
- Detalhar automaticamente toda tarefa nova.
- O app commitar ou abrir PR no repositório do projeto.
- Rodar mais de um agente diferente no mesmo projeto.
