---
id: "280"
titulo: "Passo 4d: entrar na lista é um ato de verdade"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["279"]
estima_min: 120
---

# 280 — Entrar na lista é um ato de verdade

## Como apareceu

Fui sondar o nível dos ITENS pra ver se valia a pena o ciclo 281 (a
árvore mandando na navegação). A sonda: página com lista, entrar no
editor, `j` até a lista, `Enter`, e então `j`.

```
bloco 1 (lista): ul:"um dois tres"
dentro da lista: ul:"um dois tres"   ← não desceu
  j1: ul:"um dois tres"              ← e não anda
  j2: ul:"um dois tres"
```

A lista é um beco. Entra-se nela e não se sai nem se anda — exatamente o
"entrei e fiquei preso" que a pessoa relatou ao pedir a árvore n-ária.

## O cenário que devia ter pego isso

O ciclo 273 tem um cenário chamado **"os itens da lista são blocos, e
entrar neles é um ato"**. Ele nunca aperta Enter: confere marcação
(atributos presentes, `contenteditable` ligado) e passa.

O nome promete o que ele não mede. É a quarta vez neste projeto que essa
armadilha esconde alguma coisa, e a única defesa continua sendo ver o
cenário reprovando pelo motivo certo.

## A causa

`marcar_itens_da_lista` põe `data-nav-parent="bloco-N"` nos itens, mas o
`<ul>` nunca declarou `data-nav-group="bloco-N"`. O Enter decide entre
descer e ativar por `group_of()`, que lê esse atributo — sem ele a lista
é folha. Todo outro grupo do app declara o seu; a lista dependia de um
lado só da relação.

## O que veio junto

O id do bloco era `bloco-{i}` com `i` contado DENTRO do segmento.
`marcar_blocos` roda uma vez por segmento, então numa página com embeds
dois segmentos produziam dois `bloco-1`. Enquanto o id só nomeava um
destino isso passava batido — a navegação anda por ordem de documento,
não por nome. Vira defeito no instante em que o id também nomeia um
GRUPO, porque o Escape acha o grupo por seletor e acharia o `<ul>` do
segmento errado.

O id passou a levar o segmento junto.

## Critérios de aceite

- [x] Um cenário foi visto REPROVANDO antes, com o defeito na mensagem
- [x] Enter na lista desce pro primeiro item
- [x] `j` anda entre itens
- [x] Escape volta pro nível da lista
- [x] O id do grupo é único no documento
- [x] A suíte não muda de resultado (258/258)

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
