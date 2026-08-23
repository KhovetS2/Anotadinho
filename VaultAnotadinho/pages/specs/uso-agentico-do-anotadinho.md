---
title: Uso agêntico do Anotadinho no dia a dia
tags:
- spec
- agent-os
type: spec
date: 2026-08-22
prioridade: alta
status: aprovada
---
# Uso agêntico do Anotadinho no dia a dia
{{ type: "fluxo" }}
artefato: spec
etapa: aprovada

{{ /fluxo }}
## Problema

As peças existem — conversa, fluxo, propostas, MCP — mas o uso ainda
depende de saber onde cada uma está. Falta o caminho pronto: começar uma
conversa, virar spec, aprovar, executar, sem procurar nada.

Hoje é preciso criar a página de conversa na mão, lembrar de pôr
`type: conversa`, e saber que existe uma página `type: propostas`.

## Proposta

**1. Começar uma conversa em um passo.** Comando na paleta ("Nova
conversa") e botão no home. Cria em `pages/conversas/` com a data no
nome, já com `type: conversa`, e abre.

- Ponto a ser considerado, poder adicionar a conversa paginas como contexto para o modelo, por exemplo, se ao conversar para criar um spec relacionado a modo agentico do Anotadinho eu anexo de contexto que ele deve consultar tanto para me responder como para propor algo que não necessariamente entre em conflito ou que as vezes já existe fazendo com que ele perca menos tempo tendo que preocuparar aleatoriamente e ter um foco maior.

**2. A conversa lembra de onde veio.** Ao ser criada a partir de uma
página, gravar `origem:` no frontmatter — hoje o contexto é a página
anterior em memória e some ao reabrir o app.

**3. Aviso de proposta pendente.** Um indicador no cabeçalho quando há
proposta esperando revisão. Sem isso, o agente propõe e ninguém vê até
abrir a página certa.

**4. Executar a partir da proposta aprovada.** Botão que chama o agente
com a proposta como prompt, gerando a `execucao` — é a etapa que hoje
só existe como conceito.

**5. Passo a passo escrito.** Uma página só sobre como operar, pra não
depender de memória. Ver [[Como usar o modo agêntico]].

## Como saber que funcionou

- Dá pra ir de "quero fazer X" até uma spec aprovada sem criar arquivo
na mão.
- Uma proposta pendente é visível de qualquer página.
- Cenários de harness cobrindo o caminho inteiro.

## Não-objetivos

- Sessão persistente com o modelo (hoje é um disparo por vez).
- Streaming da resposta token a token.
- Executar sem aprovação — a revisão humana é a defesa que sustenta o
desenho, ver [[Capacidades de agente]].

## Relacionado

- [[Como usar o modo agêntico]] — o passo a passo
- [[Capacidades de agente]] — limites e configuração
- [[Guia do Agent OS]] — o esquema do vault
