---
id: "277"
titulo: "Clicar no que já existe: as 27 corridas latentes da suíte"
status: aberto
criado: 2026-09-06
autor: agente
prioridade: media
depende_de: ["276"]
estima_min: 90
---

# 277 — Clicar no que já existe

## O que motivou

No ciclo 276, três cenários reprovaram com cara de defeito do app
(`null is not an object`) e eram corrida do próprio cenário: eles
esperavam por um sinal PRÓXIMO — a aba entrou na barra — e clicavam no
mesmo instante em OUTRO elemento, que ainda não tinha renderizado.

Custou caro achar, porque a reprovação era determinística na suíte e
sumia quando eu refazia os passos na mão (onde eu, sem pensar, tinha
posto uma pausa).

## O tamanho do problema, medido

Varredura por `querySelector('X').click()` sem `esperar` pelo MESMO
seletor nas 10 linhas anteriores:

```
27 ocorrências
  fluxo.mjs:    17
  cenarios.mjs:  6
  telas.mjs:     4
```

Cada uma reprova com a mesma assinatura enganosa quando o tempo muda —
máquina carregada, embed mais pesado, ordem diferente de cenários.

## O que fazer

Um auxiliar em `bridge.mjs`:

```js
await clicar(bridge, '.seletor', "o que se espera aparecer");
```

que espera o seletor e SÓ ENTÃO clica, com mensagem de falha dizendo o
que não apareceu — em vez de `null is not an object`.

Migrar as 27, uma por vez, rodando o cenário afetado a cada troca.

## Critérios de aceite

- [ ] `clicar()` existe e falha com mensagem que nomeia o elemento
- [ ] As 27 ocorrências migradas
- [ ] Nenhum cenário que passava passou a reprovar
- [ ] A varredura volta a zero (o script está no status do 276)

## Cuidado registrado

No ciclo 270 eu "arrumei" higiene de abas na suíte e quebrei um cenário
que passava. Mudança em massa no harness se valida cenário a cenário,
não só pelo total no fim.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
```
