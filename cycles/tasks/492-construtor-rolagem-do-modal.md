---
id: "492"
titulo: "Construtor mermaid: a rolagem do modal não trava em cima do canvas"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["491"]
estima_min: 45
---

# 492 — A rolagem do modal

Relato depois do 491: "o scroll do modal está meio travado".

Duas causas:

1. O 491 fez a roda sozinha andar a vista do canvas, com
   `prevent_default`. O canvas ocupa quase a coluna inteira: rolar o modal
   parava sempre que o ponteiro passava por cima dele.
2. Três rolagens uma dentro da outra — o modal, a prévia (`max-height:
   70vh`) e o formulário de sequência e gantt (`max-height: 60vh`). A roda
   ia pra uma ou pra outra conforme o ponteiro.

## Critérios de aceite

- [x] A roda sem Ctrl passa direto pelo canvas; Ctrl+roda (e a pinça)
      continua aproximando. Andar a vista é pelo arraste do fundo.
- [x] O modal tem altura fixa e não rola; a edição e a prévia rolam cada
      uma no seu painel; Cancelar e Aplicar ficam sempre à vista.
- [x] O cenário 491 confere as duas coisas, e reprova com a roda segurada.

## Comandos de validação

```bash
cd ui && cargo test --lib && trunk build
node scripts/uitest/run.mjs "(48"
node scripts/uitest/run.mjs "(49"
node scripts/uitest/run.mjs
```

## Não-objetivos

- A rolagem de verdade no WebKitGTK (roda, touchpad) fica pra conferência
  à mão: o cenário usa `WheelEvent` sintético.
