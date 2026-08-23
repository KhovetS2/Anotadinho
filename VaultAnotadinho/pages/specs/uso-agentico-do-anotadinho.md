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

## Requisitos funcionais

- **RF1.** Começar uma conversa com o agente em um passo, sem criar
  arquivo à mão nem lembrar de pôr `type: conversa`.
- **RF2.** A conversa registra de onde nasceu e quais páginas o modelo
  deve consultar, de forma que sobreviva a fechar o app.
- **RF3.** Uma proposta pendente de revisão é visível de qualquer
  página.
- **RF4.** A partir de uma proposta aprovada, é possível disparar a
  execução e registrar o que aconteceu.
- **RF5.** Existe documentação de operação que não dependa de memória.

## Requisitos não funcionais

- **RNF1.** Nenhuma escrita do agente chega ao vault sem revisão humana.
- **RNF2.** A configuração do agente vive fora do vault.
- **RNF3.** O `.md` continua legível e editável fora do app.

## Critérios de aceite

- [x] Ir de "quero fazer X" até uma spec aprovada sem criar arquivo na mão.
- [x] Uma proposta pendente é notada sem abrir a página de propostas.
- [x] Cenários de harness cobrindo o caminho inteiro.

## Notas de escopo

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
