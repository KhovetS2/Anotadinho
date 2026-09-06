---
id: "284"
titulo: "Ver a página sem janela"
status: done
criado: 2026-09-06
autor: agente
prioridade: media
depende_de: ["283"]
estima_min: 60
---

# 284 — Ver a página sem janela

## Objetivo

O `render::Terminal` existe desde o ciclo 266 e o ciclo 283 deu a ele o
que desenhar dentro de cada embed. Nada fora dos testes o usava.

`anotadinho-cli ver <página>` desenha a estrutura de uma página — a
MESMA árvore que a janela navega — sem DOM nenhum. É o primeiro pedaço
do porte que dá pra usar, e é o que transforma "o núcleo dá conta" de
afirmação em coisa que se roda.

## Critérios de aceite

- [x] O comando desenha bloco a bloco, e dentro de cada embed o conteúdo
- [x] Um teste cobre uma página com embed de ponta a ponta
- [x] O desenho não despeja o fence do embed

## Comandos de validação

```bash
cargo test -p anotadinho-cli
anotadinho-cli --vault <vault> ver pages/incio.md
```
