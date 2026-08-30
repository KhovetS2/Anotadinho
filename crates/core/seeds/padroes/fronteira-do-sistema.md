---
title: A fronteira com o sistema operacional
date: 2026-08-30
dominio: teste
tags:
- padrao
---
# A fronteira com o sistema operacional

## Quando se aplica

Todo gesto que **nasce fora do app** e entra nele: arrastar um arquivo do
gerenciador, colar da área de transferência do sistema, abrir por diálogo
nativo, receber um atalho global, um evento de janela.

## A regra

Um cenário sintético prova o caminho **depois** da fronteira. Nunca a
fronteira.

Quando o harness monta o evento, quem escolhe o formato é ele mesmo — e
ele escolhe o formato que quem escreveu o teste imaginou. O sistema
operacional não consultou ninguém. Se os dois discordarem, o cenário fica
verde e o gesto falha, sem contradição: eles testam coisas diferentes.

Então, para código que atravessa a fronteira:

1. **Meça primeiro.** Instale uma sonda na janela de verdade, peça o
   gesto de verdade, e grave o que chegou — inteiro, não os primeiros
   caracteres.
2. **Só então escreva o cenário**, reproduzindo o formato medido.
3. **Confirme à mão.** Verde no harness não conclui um ciclo que mexe na
   fronteira; a confirmação é alguém fazendo o gesto.

## O caso que criou este padrão

Arrastar imagem do gerenciador de arquivos não inseria nada. Três ciclos
disseram que estava resolvido, com a suíte inteira verde:

- Os cenários montavam o drop com `File`. O WebKitGTK **nunca** entrega
  `File` nesse caminho.
- Depois passaram a montar com `text/uri-list` preenchido. O gerenciador
  **anuncia** esse tipo e o entrega **vazio**.
- O caminho estava no `text/html`, como texto de uma âncora:
  `<a style="…">file:///home/eu/foto.png</a>`.

O que resolveu não foi teste: foi uma sonda gravando o payload inteiro
durante um arrasto real. Uma sonda anterior tinha cortado em 200
caracteres e escondido justamente o trecho que importava.

## Sinais de alerta

- "O cenário passa, mas no uso não funciona" — quase sempre é fronteira.
- Um teste que **constrói** o evento que deveria estar testando.
- Sonda que trunca o que grava.
- Um formato "óbvio pela especificação". A especificação diz o que é
  permitido, não o que aquele programa faz.

## O que NÃO fazer

Concluir da suíte verde que a fronteira funciona. Ela prova que o resto do
caminho está de pé — o que é útil, e não é a mesma pergunta.
