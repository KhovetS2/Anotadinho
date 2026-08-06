---
id: "059"
titulo: "Task tables com filtro e ordenação"
status: pending
criado: 2026-08-06
depende_de: ["058"]
estima_min: 90
---

# Task Tables

## Objetivo
View de tabela listando todas as páginas com property `status::`.
Colunas: título, status, data, prioridade. Ordenável e filtrável.
Integrado com kanban e calendário.

## Critérios
- [ ] Tabela com colunas extraídas das properties
- [ ] Ordenação por coluna (click no header)
- [ ] Filtro por status/prioridade
- [ ] Click na linha abre no editor
- [ ] Sincronizado: mudar status na tabela atualiza kanban e vice-versa
