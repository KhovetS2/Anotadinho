---
id: "247"
titulo: "O padrão da fronteira com o sistema operacional"
status: done
criado: 2026-08-30
autor: agente
prioridade: media
depende_de: ["246"]
estima_min: 30
---

# 247 — O padrão da fronteira

## Objetivo

Registrar como padrão a lição que custou três ciclos no arrasto de
imagem, para não ser reaprendida no próximo gesto que venha de fora.

## Critérios de aceite

- [x] `pages/padroes/fronteira-do-sistema.md` escrito
- [x] Entra na semente, então um vault novo já nasce com ele
- [x] `AGENTS.md` manda medir antes de consertar e confirmar à mão

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
