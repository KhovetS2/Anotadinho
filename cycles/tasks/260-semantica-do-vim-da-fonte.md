---
id: "260"
titulo: "A semântica do vim, tirada da fonte"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["252", "254"]
estima_min: 180
---

# 260 — A semântica do vim, tirada da fonte

## Por que existe

O ciclo 254 implementou o vocabulário do vim a partir de um cheat sheet.
O vocabulário ficou certo; a **semântica**, não. Um cheat sheet diz que
`de` apaga até o fim da palavra — não diz que `e` é inclusivo e `w` é
exclusivo, nem que o registrador carrega o tipo do yank, nem que `cw`
tem um caso especial condicionado a o cursor não estar sobre espaço.

Nada disso é dedução: está em `runtime/doc/motion.txt` e em
`src/nvim/normal.c`. Com o projeto sob GPL-3 (compatível com a licença do
Vim e com a Apache-2.0 do Neovim), consultar é opção legítima.

## Critérios de aceite

- [x] Tabela inclusivo/exclusivo/por-linha, com cada linha citável no
      `motion.txt`, e teste que a percorre inteira
- [x] Registrador tipado: `yy`/`dd` por linha, `yw`/`x`/`d$` por
      caractere, e `p` respeitando os dois
- [x] `cw` age como `ce`, mas só com o cursor fora de espaço
- [x] Cenários de harness que reprovam com o comportamento antigo
- [x] `w`, `e` e `b` concordam sobre o que é uma palavra

## O que cresceu no caminho, e por quê

Comecei corrigindo só o `e`. O cenário do harness mostrou que o `w`
estava errado também, para o outro lado — e deixar um certo e o outro
errado seria pior que os dois errados, porque aí eles discordariam sobre
onde uma palavra termina. Os três movimentos de palavra passaram a sair
do texto, não da granularidade `word` do navegador.

## Comandos de validação

```bash
cd ui && cargo test --target x86_64-unknown-linux-gnu
node scripts/uitest/run.mjs teclado
node scripts/uitest/run.mjs
```
