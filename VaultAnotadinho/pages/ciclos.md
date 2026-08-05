---
title: Sistema de Ciclos
tags: [docs, processo]
created: 2026-08-04
---

# Sistema de Ciclos

O Anotadinho evolui por **ciclos**, não por commits avulsos.

## Ciclos concluídos

| Ciclo | Feature | Status |
|---|---|---|
| 001 | Bootstrap Tauri + Yew com tema dark | ✅ done |
| 002 | Vault picker (dialog nativo de seleção de pasta) | ✅ done |
| 003 | Sidebar com lista de páginas (Pages + Journals) | ✅ done |

## Próximos ciclos

- 004: Editor Markdown básico
- 005: Salvamento automático
- 006: Parser de blocos e properties
- 009: Watcher de arquivos (notify)
- 011: Busca full-text (FTS5)

## Garantias

1. **Isolamento**: cada task em área bem definida
2. **Não-regressão**: `cargo test` roda TODOS os testes
3. **Histórico**: status files em `cycles/status/`
4. **Dependências**: task tem campo `depende_de`

## Comandos

```bash
./cycles/orchestrator.sh run      # próxima task pendente
./cycles/orchestrator.sh list     # lista todas as tasks
./cycles/orchestrator.sh status   # status geral
./cycles/orchestrator.sh history  # histórico completo
```
