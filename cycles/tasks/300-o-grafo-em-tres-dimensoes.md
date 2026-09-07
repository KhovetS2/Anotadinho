---
id: "300"
titulo: "O grafo em três dimensões, com física"
status: done
criado: 2026-09-07
autor: agente
prioridade: alta
depende_de: ["299"]
estima_min: 180
---

# 300 — O grafo em três dimensões

## O problema, medido

O grafo desenhava todos os nós num CÍRCULO (`2πi/n` por índice), sem
física. O vault tem **239 páginas e 162 links**: todo nó fica na borda,
e por isso **toda aresta é uma corda cruzando o meio**. O desenho vira
uma bola de linhas que não mostra estrutura nenhuma.

O próprio ciclo 120 previu: "reavaliar layout melhor se um vault muito
grande deixar o círculo ilegível".

## O que muda de verdade

O que conserta a legibilidade é a FÍSICA, não a dimensão: força
dirigida agrupa o que se liga e afasta o que não se liga. A terceira
dimensão soma em cima disso — com rotação, some a oclusão que um layout
plano tem em grafo denso.

## Onde mora

No núcleo (`grafo.rs`): entra grafo, sai posição. É matemática, não
desenho, e assim dá pra AFIRMAR que dois nós ligados terminam mais perto
que dois que não se ligam — coisa que nenhum teste de tela consegue.

A janela projeta com perspectiva, dá tamanho e opacidade por
profundidade, e gira no arraste.

## Critérios de aceite

- [x] Quem se liga termina mais perto
- [x] O resultado é o mesmo toda vez (determinístico)
- [x] Usa as três dimensões, sem achatar num plano
- [x] Grafo vazio e aresta inválida não quebram
- [x] Teto de ordem medido no tamanho real do vault
- [x] Um cenário que reprova com o layout de círculo — visto
      reprovando: "média 185, desvio 18, 242 nós"
- [x] A suíte não muda de resultado (259/261; as duas passam isoladas
      e são as instáveis conhecidas)

## Comandos de validação

```bash
cargo test -p anotadinho-core --release grafo -- --ignored
node scripts/uitest/run.mjs
```
