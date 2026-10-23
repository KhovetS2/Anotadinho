---
id: "478"
titulo: "Tarefa rica — na TUI: selos no cartão e `v` pros detalhes"
status: done
criado: 2026-10-23
autor: agente
prioridade: alta
depende_de: ["477"]
estima_min: 150
---

# 478 — Tarefa rica na TUI

Terceira etapa da proposta `pages/propostas/tarefa-rica-home-e-memoria.md`.

## Critérios de aceite

- [x] A terceira linha do cartão vira a dos selos: dependência,
      critérios, prazo (`⚠` vencido), comentários, anexos e tags.
      Cortada na borda da coluna.
- [x] `v` abre os detalhes. Enter num critério marca sem fechar o modal;
      tags e prazo vêm preenchidos (vazio tira); comentar; anexar por
      caminho (com `~`); abrir o anexo (só de `assets/`).
- [x] Os mesmos `handle_*` da janela.
- [x] Testes de unidade, foto de tela e pty.
