---
title: {{title}}
date: {{date}}
status: backlog
priority: media
owner: ''
depends_on: []
related_decision: ''
tags:
- spec
---
# {{title}}

> Status: `backlog` → `in-progress` → `in-review` → `done` (ou `blocked`).
> Atualize o campo `status` no frontmatter (painel de propriedades ou
> direto no `.md`) conforme a spec avança — é o campo que um agente
> deve checar antes de puxar a próxima tarefa.

## Contexto

Por que isto existe e que problema resolve — para quem, e o que motivou
priorizar agora. **Nada de solução aqui**: como fazer é assunto da
proposta de implementação.

## Requisitos funcionais

O que o sistema PRECISA fazer, em comportamento observável.

- **RF1.**
- **RF2.**

## Requisitos não funcionais

Desempenho, segurança, compatibilidade, acessibilidade — as restrições
que valem seja qual for a abordagem escolhida.

- **RNF1.**

## Critérios de aceite

Como saber, sem ambiguidade, que está pronto.

- [ ]
- [ ]

## Fora de escopo

O que esta spec explicitamente NÃO cobre.

## Por que separar spec de proposta

A spec é o **o quê** e sobrevive à troca de abordagem: se a proposta
ferir um padrão da casa, você descarta a proposta e escreve outra — o
requisito continua valendo. Quando esta spec for aprovada, use
"Planejar implementação" pra abrir a conversa que gera a proposta, e
anexe ali as páginas de padrão que ela deve respeitar.
