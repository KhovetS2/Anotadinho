---
id: "476"
titulo: "Tarefa rica — núcleo: tags, prazo, checklist, comentários e anexos"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["475"]
estima_min: 180
---

# 476 — Tarefa rica: núcleo

Primeira etapa da proposta `pages/propostas/tarefa-rica-home-e-memoria.md`.

## Critérios de aceite

- [x] `Tarefa` lê `tags` (sem a `tarefa`), `prazo`, `checklist` (os itens
      de **Critérios de aceite**), `comentarios` e `anexos`.
- [x] Funções puras que mexem só no trecho delas: `alternar_item`,
      `acrescentar_comentario`, `acrescentar_anexo`, `com_tags` e
      `com_prazo` (recusa o que não é `AAAA-MM-DD`). Mais `vencida` e
      `progresso`.
- [x] `handle_*` pela pessoa. O anexo é copiado SEM sobrescrever.
- [x] O plano, o `propor_tarefa` (MCP) e o `tarefa nova` (CLI) aceitam
      tags e prazo. O prazo inválido é recusado antes de virar proposta.

## Comandos de validação

```bash
cargo test --workspace
```
