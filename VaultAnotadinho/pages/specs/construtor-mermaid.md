---
title: Construtor de diagramas mermaid
tags:
- spec
- editor
- mermaid
type: spec
date: 2026-10-23
prioridade: media
status: aprovada
---
# Construtor de diagramas mermaid

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada
{{ /fluxo }}

## Contexto

Desde o ciclo 485, um bloco ` ```mermaid ` salvo aparece desenhado. Mas
montar um diagrama ainda exige saber a sintaxe do mermaid: o `/mermaid`
abre um campo de texto de uma linha, e editar é mexer no código do bloco.

Além disso, a fonte do código (JetBrains Mono) junta `->>` e `-->` em
ligaduras, que aparecem como `—»`. O arquivo está certo, mas quem edita o
código vê caracteres que não digitou.

## Requisitos funcionais

### A. Abrir o construtor

- **RF1.** O `/mermaid` abre um modal com o construtor, no lugar do campo
  de uma linha. "Inserir" põe o diagrama na página como um bloco
  ` ```mermaid `.
- **RF2.** Um bloco mermaid que já existe tem uma ação "Editar no
  construtor". Ela abre o mesmo modal com o diagrama carregado, e
  "Aplicar" troca o código do bloco.
- **RF3.** O modal tem duas abas: **Construtor** e **Código**. As duas
  editam o mesmo diagrama, e há uma prévia desenhada ao vivo.

### B. O que o construtor monta

- **RF4.** Quatro tipos:
  - **Fluxograma:** nós com formato (caixa, arredondado, decisão,
    círculo), ligações com e sem rótulo e a direção (de cima pra baixo
    ou da esquerda pra direita);
  - **Sequência:** participantes, mensagens em ordem (síncrona, resposta,
    assíncrona) e notas;
  - **Estados:** estados, transições com rótulo, início e fim;
  - **Gantt / cronograma:** seções, tarefas com início, duração ou fim e
    dependência.
- **RF5.** A forma de montar mistura formulário e canvas:
  - **Fluxograma e Estados:** canvas onde se arrastam os nós e se puxam
    ligações entre eles, mais um painel lateral para editar o
    selecionado;
  - **Sequência e Gantt:** formulário, com listas para adicionar, mover e
    apagar itens. Nesses dois, a ordem É o conteúdo.
- **RF6.** O construtor funciona inteiro por teclado: adicionar, selecionar,
  ligar, mover na lista e apagar.

### C. Abrir o que o construtor não entende

- **RF7.** Se o código de um bloco tiver algo fora do modelo do construtor
  (outro tipo de diagrama, estilos, subgrafos, diretivas), o modal abre
  na aba **Código** e diz o motivo. A aba Construtor fica indisponível
  para esse diagrama.
- **RF8.** Abrir e aplicar sem mudar nada devolve o código igual.

### D. Ler o código

- **RF9.** O código mermaid aparece sem ligaduras, no bloco e na aba
  Código.

## Requisitos não funcionais

- **RNF1.** O arquivo guarda só o código mermaid. Nenhum formato próprio
  fica fora do bloco.
- **RNF2.** A posição dos nós no canvas não muda o desenho: o mermaid
  recalcula o layout. Se a posição for guardada, ela vai dentro do bloco,
  num comentário `%%` (que o mermaid ignora), e some sem prejuízo quando
  alguém apaga o comentário.
- **RNF3.** O mermaid continua sendo lido só quando é preciso (ciclo 486):
  abrir o modal lê a biblioteca, e a página sem diagrama continua sem
  ela.
- **RNF4.** O DOM novo leva `data-nav-*`, o CSS usa tokens e BEM, e os
  ícones novos entram em `icon.rs`.

## Critérios de aceite

- [ ] O `/mermaid` abre o construtor, e "Inserir" põe um bloco que
      aparece desenhado.
- [ ] Os quatro tipos montam um diagrama válido, conferido pela prévia
      sem erro.
- [ ] Fluxograma e Estados: arrastar um nó e puxar uma ligação pelo canvas
      gera a ligação no código.
- [ ] "Editar no construtor" abre o diagrama do bloco, e "Aplicar" troca
      só aquele bloco.
- [ ] Um diagrama fora do modelo abre na aba Código, com o motivo.
- [ ] Abrir e aplicar sem mexer devolve o arquivo igual.
- [ ] Nenhuma ligadura no código mermaid.

## Fora de escopo

- Outros tipos do mermaid (classes, ER, pizza, jornada, mapa mental).
- Estilos por nó, cores e subgrafos no construtor. Eles continuam
  editáveis pelo código.
- ~~O construtor na TUI.~~ *(Saiu do "fora de escopo" em 2026-10-24, a
  pedido da pessoa: o ciclo 500 trouxe o construtor pra TUI, com
  formulários por tipo, a prévia desenhada em caracteres (496–499) e a aba
  Código; abrir e aplicar sem mexer devolve o código igual, como o RF8
  daqui.)*

## Ambiguidade apontada

O pedido foi "um misto" de formulário com prévia e de canvas arrastável.
O RF5 lê assim: **canvas onde a estrutura é um grafo** (fluxograma e
estados) e **formulário onde a estrutura é uma lista ordenada**
(sequência e gantt). Num canvas, a posição de uma mensagem ou de uma
tarefa do gantt não diria nada, porque a ordem é que diz.

Se a ideia for outra, por exemplo canvas nos quatro tipos ou formulário
com um canvas só de visualização, o RF5 muda antes da proposta valer.
