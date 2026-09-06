---
id: "272"
titulo: "A árvore rodando em paralelo ao editor"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["271"]
estima_min: 150
---

# 272 — A árvore em paralelo

## Objetivo

Passo 3 da unificação. A `Unidade` já lê e escreve markdown (ciclo 271),
mas o editor continua derivando tudo do DOM. Antes de INVERTER — fazer a
árvore virar a fonte da verdade — é preciso saber onde os dois discordam
hoje.

Aqui os dois rodam em paralelo e são comparados. Cada divergência é um
lugar onde a inversão quebraria alguma coisa, e é muito mais barato
achar agora.

## Critérios de aceite

- [x] Um comando devolve a visão do MODELO sobre os blocos de uma página
- [x] Uma bateria compara modelo contra tela, incluindo páginas REAIS
- [x] Divergência conhecida é declarada, não tolerada em silêncio
- [x] Cada divergência conhecida tem um cenário que AFIRMA que ela ainda
      existe — pra o dia em que ela sumir avisar
- [x] Custo zero no render: a árvore só é construída quando alguém
      pergunta

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs --arvore
node scripts/uitest/run.mjs
```
