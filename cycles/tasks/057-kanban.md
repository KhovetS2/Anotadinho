---
id: "057"
titulo: "Kanban board view"
status: pending
criado: 2026-08-06
depende_de: ["056"]
estima_min: 90
---

# Kanban Board

## Objetivo
Nova view de kanban com colunas (Backlog, Todo, Doing, Done).
Cards são páginas `.md` com property `kanban:: coluna`.
Arrastar cards entre colunas atualiza a property.

## Critérios
- [ ] View Kanban acessível via menu de views
- [ ] Colunas configuráveis
- [ ] Cards arrastáveis entre colunas (HTML5 drag)
- [ ] Atualiza property `kanban::` no .md ao mover
- [ ] Click no card abre no editor
