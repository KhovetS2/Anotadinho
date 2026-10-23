---
id: "477"
titulo: "Tarefa rica — na janela: selos no cartão e painel de detalhes"
status: done
criado: 2026-10-23
autor: agente
prioridade: alta
depende_de: ["476"]
estima_min: 180
---

# 477 — Tarefa rica na janela

Segunda etapa da proposta `pages/propostas/tarefa-rica-home-e-memoria.md`.

## Critérios de aceite

- [x] O cartão mostra os selos: critérios (2/5, verde quando completo),
      prazo (vermelho se vencido e não feito), comentários, anexos e
      tags (RF5).
- [x] `⋯` no cartão abre o painel ao lado do quadro. Ele edita tags e
      prazo, marca os critérios, comenta, anexa pelo diálogo nativo e
      abre o anexo (RF6). O botão fica fora do arrasto.
- [x] Abrir anexo só vale pra arquivo dentro de `assets/`.
- [x] DOM com `data-nav-*`, CSS com tokens e em BEM.
- [x] Harness: selos, painel (critério byte a byte, comentário, tag e
      prazo), anexo sem sobrescrever e a recusa fora de `assets/`.
