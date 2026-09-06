---
id: "290"
titulo: "Esperar o que se vai TOCAR, não só o que se vai clicar"
status: aberto
criado: 2026-09-06
autor: agente
prioridade: media
depende_de: ["289"]
estima_min: 120
---

# 290 — Esperar o que se vai tocar

## O que o ciclo 277 não pegou

O 277 consertou 27 cliques diretos e escreveu a regra no AGENTS.md:
**espere o elemento que você vai clicar, não um parente dele.**

A varredura de lá procurava `querySelector(...).click()` — o uso
ENCADEADO. Não pega esta forma, que é a mesma coisa em duas instruções:

```js
const alvo = document.querySelector('.editor__bloco');
alvo.focus();   // ← estoura em "null is not an object"
```

Foi assim que três cenários de imagem reprovaram na suíte do ciclo 289,
com a assinatura de sempre, e passaram isolados.

## O tamanho, medido

Varredura por `const x = document.querySelector(...)` sem espera pelo
seletor e sem checagem de nulo: **60 ocorrências**. Quatro já foram
consertadas no 289 (os cenários de imagem que estavam reprovando).

Nem todas são corrida de verdade — muitas pegam `.app-root`, que existe
desde sempre. A varredura precisa ser lida antes de aplicada.

## Cuidado registrado

No ciclo 270 eu "arrumei" higiene de abas na suíte e quebrei um cenário
que passava. No 277 a lição virou método: mapear quais cenários contêm
as linhas mudadas e rodar os grupos ANTES da suíte inteira. Vale igual
aqui.

## Critérios de aceite

- [ ] A varredura é lida e separada: corrida de verdade × falso positivo
- [ ] As de verdade esperam pelo elemento
- [ ] Nenhum cenário que passava passou a reprovar
- [ ] A regra no AGENTS.md passa a cobrir as DUAS formas

## Comandos de validação

```bash
node scripts/uitest/run.mjs
```
