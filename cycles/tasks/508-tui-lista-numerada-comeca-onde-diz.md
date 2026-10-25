---
id: "508"
titulo: "TUI: a lista numerada começa no número que a fonte diz"
status: done
criado: 2026-10-24
autor: agente
prioridade: baixa
depende_de: ["507"]
estima_min: 20
---

# 508 — A numerada começa onde diz

O que o 507 deixou de fora: `3. três` saía `1. três`.

## Critérios de aceite

- [x] O primeiro item tem o número da primeira linha da fonte da lista
      (`3.` ou `3)`), e os outros seguem dele.
- [x] Sem número legível, conta do 1.

## Comandos de validação

```bash
cargo test -p anotadinho-tui
```
