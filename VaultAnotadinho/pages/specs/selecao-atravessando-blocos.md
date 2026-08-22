---
title: "Seleção e cópia atravessando blocos"
type: spec
date: 2026-08-22
status: em-revisao
prioridade: media
tags:
- spec
- editor
---
# Seleção e cópia atravessando blocos

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao
origem: pages/ciclos/175-edicao-estruturada-por-bloco.md
{{ /fluxo }}

## Problema

É a única pendência do ciclo 175, e foi deixada de fora **de propósito**
lá — não é esquecimento, é uma troca conhecida.

Com um `contenteditable` por bloco, o navegador não estende seleção
entre blocos. Na prática:

- Arrastar o mouse do meio de um parágrafo até o seguinte não seleciona
  os dois.
- `Ctrl+A` seleciona só o bloco onde o cursor está.
- Copiar dois parágrafos de uma vez e colar noutro editor não funciona.

Selecionar e copiar **dentro** de um bloco funciona normalmente.

## Por que ainda não foi feito

O modelo de um editável por bloco é o mesmo do Notion e do Logseq, e os
dois resolvem isto **reimplementando seleção do zero**: rastrear âncora
e foco em coordenadas próprias, desenhar o realce por conta, e
interceptar copiar/recortar/colar pra montar o markdown na mão.

É trabalho grande, mexe no caminho mais usado do app, e ninguém pediu
até agora. O ciclo 175 registrou a decisão em vez de fingir que não
existia.

## Proposta

Seleção de BLOCOS, não de caracteres entre blocos — que é o que o
modelo comporta sem reescrever o motor de seleção:

1. No modo de navegação, `Shift+↑/↓` estende a seleção pra blocos
   vizinhos, com realce visível.
2. `Ctrl+C` com blocos selecionados monta o markdown deles, na ordem, e
   põe na área de transferência.
3. `d` apaga todos os selecionados; `K`/`J` movem o conjunto.
4. Arrastar o mouse entre blocos passa a selecionar blocos inteiros, em
   vez de não fazer nada.

Isso cobre o caso real ("quero levar estes três parágrafos pra outro
lugar") sem reimplementar seleção de texto.

## Como saber que funcionou

- Selecionar três blocos e copiar produz markdown legível ao colar num
  editor de texto comum.
- Selecionar e apagar remove exatamente os blocos realçados.
- A bateria de digitação continua verde: seleção DENTRO de um bloco não
  pode mudar de comportamento.
- Cenário novo no harness cobrindo seleção múltipla.

## Não-objetivos

- Seleção parcial atravessando blocos (metade de um parágrafo até a
  metade do próximo). É o que exigiria o motor próprio.
- Arrastar bloco com o mouse pra reordenar.

## Relacionado

- [[Ciclo 175 — Edição estruturada por bloco]] — onde a decisão foi tomada
- [[Ciclos]]
