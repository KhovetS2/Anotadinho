---
id: "240"
titulo: "A barra de formatação continua servindo depois de cada clique"
status: done
criado: 2026-08-30
autor: agente
prioridade: alta
depende_de: ["234", "235"]
estima_min: 90
---

# 240 — A barra continua servindo

## Objetivo

Três relatos do uso real: a seleção se perde ao aplicar ou tirar uma
marca; a paleta de cor reaparece aberta na seleção seguinte; e marcar,
desselecionar e reselecionar não tira a marca — aninha outra, e depois
não sai mais.

## Critérios de aceite

- [x] A seleção sobrevive a aplicar e a tirar
- [x] Reselecionar e clicar de novo TIRA a marca
- [x] Marca parcial não vira marca aninhada
- [x] A paleta abre fechada em cada seleção nova
- [x] Dois cenários do harness deixam de digitar antes de o bloco existir

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs
```
