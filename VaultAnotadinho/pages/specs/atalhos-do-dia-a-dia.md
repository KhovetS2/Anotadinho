---
title: 'Atalhos do dia a dia: aba fixa e criação de conversa'
tags:
- spec
- ui
type: spec
date: 2026-08-23
prioridade: media
status: aprovada
---
# Atalhos do dia a dia: aba fixa e criação de conversa

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada

{{ /fluxo }}

## Contexto

Duas asperezas pequenas que aparecem toda sessão.

**A home some.** A página definida como inicial é uma aba igual às
outras: pode ser fechada, e vai pro fim da fila conforme se abre coisa.
Como ela é o ponto de partida — as consultas, os botões de ação, o
"trabalho recente" —, perdê-la custa uma navegação toda vez.

**Conversa não está entre os tipos de página.** A paleta lista "Nova
página: Kanban / Calendário / Tabela / Grafo" — a família de tipos. A
conversa é um tipo de página desde o ciclo 202, mas não está nessa
família: existe só como a ação separada "Nova conversa com o agente".
Quem procura por tipo não a encontra, e não dá pra criar uma numa pasta
escolhida como se faz com os outros tipos.

## Requisitos funcionais

- **RF1.** A página inicial é sempre a primeira aba.
- **RF2.** A aba inicial não pode ser fechada.
- **RF3.** Trocar qual página é a inicial move a fixação junto.
- **RF4.** Sem página inicial definida, a barra de abas se comporta como
  hoje.
- **RF5.** `conversa` aparece junto dos outros tipos de página no menu
  de criação, com o mesmo tratamento que kanban ou tabela.
- **RF6.** A conversa criada por esse caminho abre pronta pra uso, igual
  à criada pelo botão da home.

## Requisitos não funcionais

- **RNF1.** A aba fixa é visualmente distinta de uma aba comum, mas sem
  virar um elemento à parte da barra.
- **RNF2.** A navegação por teclado enxerga a aba fixa como enxerga as
  outras.

## Critérios de aceite

- [ ] Com uma home definida, ela abre na primeira posição e não oferece
      fechar.
- [ ] Trocar a home reordena as abas sem perder o que estava aberto.
- [ ] `conversa` aparece na família de tipos do menu de criação e
      produz uma página utilizável.
- [ ] Cenário de harness pra cada um dos dois.

## Fora de escopo

- Fixar abas quaisquer (só a home).
- Reordenar abas arrastando.

## Já entregue

O botão de "Nova conversa" nas ações da home foi ao ar no ciclo 208 —
foi conferido antes de escrever esta spec e está fora do escopo dela.

## Relacionado

- [[Início]]
- [[Ciclo 208 — Ações de agente na home]]
