---
title: Diagramas no terminal
tags:
- exemplo
- mermaid
---
# Diagramas no terminal

Os tipos que a TUI desenha desde o ciclo 499, e um fluxograma largo pra entrar no bloco (Enter) e rolar de lado (ciclo 498).

## Fluxograma largo

```mermaid
graph LR
  A[Receber o pedido do cliente] --> B{Pedido tem estoque?}
  B -->|sim| C[Separar e embalar os itens]
  B -->|não| D[Avisar o cliente do atraso]
  C --> E[Emitir nota e despachar]
  D --> E
```

## Pizza

```mermaid
pie showData
  title Pets adotados
  "Cães" : 386
  "Gatos" : 85.9
  "Ratos" : 15
```

## Mapa mental

```mermaid
---
title: Plano do projeto
---
mindmap
  root((Projeto))
    Pesquisa
      Usuários
      Mercado
    Construção
      id1[Protótipo]
        Testes
```

## Linha do tempo

```mermaid
timeline
  title Redes
  section Começo
    2002 : LinkedIn
    2004 : Facebook : Google
  section Depois
    2005 : YouTube
```

## Jornada

```mermaid
journey
  title Dia de trabalho
  section Manhã
    Fazer café: 5: Eu
    Trabalhar: 2: Eu, Gato
```

## Classes

```mermaid
classDiagram
  class Animal {
    <<abstract>>
    +String nome
    +comer() void
  }
  Animal <|-- Pato
  Animal <|-- Peixe
  Pato : +nadar()
  Pato "1" --> "*" Ovo : bota
```

## Entidade-relação

```mermaid
erDiagram
  CLIENTE ||--o{ PEDIDO : faz
  PEDIDO ||--|{ ITEM : contém
  CLIENTE {
    string nome
    string email PK
  }
  PEDIDO {
    int numero PK
    date data
  }
```
