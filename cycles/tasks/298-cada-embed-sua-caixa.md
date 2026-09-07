---
id: "298"
titulo: "Cada embed sua caixa, e o botão que é destino de verdade"
status: done
criado: 2026-09-07
autor: agente
prioridade: alta
depende_de: ["297"]
estima_min: 150
---

# 298 — Cada embed sua caixa

Quatro defeitos achados numa captura de tela e usando.

## Seis embeds numa caixa só

A região da moldura era identificada pela COR. Embeds vizinhos pedem a
mesma cor, então seis seguidos viravam um retângulo magenta. O DONO
distingue.

## O botão não tinha linha

A fileira é desenhada numa linha só, e cada botão é um destino. Entrar
num deles movia o cursor pra um caminho sem linha na tela: o Enter
parecia não fazer nada, e `h`/`l` não tinham onde acender.

Os filhos da fileira viraram SEGMENTOS, cada um com o caminho dele. A
linha é uma, os destinos são vários — e o que muda é qual segmento
acende.

## `G` deixava a tela em branco

A janela é contada em linhas REAIS e o desenho insere molduras. Caber na
conta não é caber na tela.

A correção monta e confere em vez de estimar; e recua o máximo que ainda
mantém o cursor visível, porque começar a janela NELE o põe na primeira
linha e deixa o resto vazio.

## O cartão se partia no meio

Com o cursor dentro de um embed, o foco abria caixa própria: a moldura
do fluxo fechava antes da linha do cursor e outra abria depois. Um
cartão partido não é cartão.

Dentro de um embed a caixa é dele; quem marca o foco é a LATERAL da
linha, que acende.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
```
