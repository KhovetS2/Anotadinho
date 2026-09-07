---
id: "297"
titulo: "O galho diz o arranjo: linha ou coluna"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["296"]
estima_min: 180
---

# 297 — O galho diz o arranjo

## A ideia não foi minha, e é melhor que a minha

Eu tinha dito que botão navegável exigiria o embed deixar de ser
atômico, e ofereci duas saídas ruins: um botão por linha, ou um cursor
horizontal dentro da linha.

Estava errado nas duas pontas. Primeiro, `navegacao::mover` **não
consulta `atomica`** — `Passo::Entrar` só pergunta se há filhos, e desde
o ciclo 283 os embeds têm. Entrar num embed já funcionava.

Segundo, a saída certa era outra: **o galho declara o arranjo**. Em
linha, os filhos ficam lado a lado e `h`/`l` andam entre eles; em
coluna, empilhados e `j`/`k` andam. Aninhar os dois dá layout, do jeito
que uma GUI compõe caixa dentro de caixa.

## O que isso resolve de barato

A geometria some. Sem o arranjo declarado, "quem está ao lado de quem"
teria que ser adivinhado pela tela — e o `espacial.rs` existe justamente
pra isso na janela. No terminal a informação já estava no modelo; faltava
alguém dizê-la.

## Critérios de aceite

- [x] `Arranjo { Folha, Coluna, Linha }` no núcleo
- [x] Os botões de um embed são uma fileira declarada
- [x] A fileira é desenhada numa linha só
- [x] `h`/`l` andam dentro da fileira; `j`/`k` saem dela
- [x] O desenho não adivinha mais pelo nome da parte
- [x] A suíte da janela não muda de resultado (258/259; a reprovada
      passa isolada, e é a instável conhecida)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
