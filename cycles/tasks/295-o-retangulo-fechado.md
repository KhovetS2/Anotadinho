---
id: "295"
titulo: "O retângulo fechado, e o fundo que saiu"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["294"]
estima_min: 60
---

# 295 — O retângulo fechado

Três ajustes pedidos depois de olhar o ciclo 294 na tela.

## O fundo saiu

A linha em foco tinha fundo de destaque. Com a moldura fazendo o mesmo
trabalho, viraram dois destaques competindo — e o texto colorido por
cima do fundo perde a cor que o TIPO dele tinha.

Quem diz onde se está agora é só a moldura.

## As quatro bordas

O retângulo tinha topo e fundo, e o texto escapava pelos lados. Sem as
laterais ele não lê como retângulo: lê como dois traços soltos.

## O texto cabe dentro

Linha mais longa que a caixa é cortada com `…`. Deixar transbordar seria
pior do que cortar: o `│` da direita sumiria e a moldura quebraria justo
na linha mais longa — que é onde ela mais precisa estar inteira.

## Critérios de aceite

- [x] O conteúdo em foco não ganha fundo
- [x] O retângulo tem as quatro bordas
- [x] Texto longo é cortado com `…` e não transborda
- [x] O realce de fundo continua só na lista de páginas, que não tem
      moldura

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```
