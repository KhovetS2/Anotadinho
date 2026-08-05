---
id: "009"
titulo: "Fechar vault (voltar à tela inicial)"
status: done
criado: 2026-08-04
autor: humano
prioridade: media
depende_de: ["008"]
estima_min: 15
agente_alvo: claude-sonnet
---

# Fechar vault

## Objetivo

Botão no header permite fechar o vault atual, limpar localStorage
e voltar à tela EmptyState.

## Critérios de aceite

- [x] Botão "Fechar" no header
- [x] Limpa vault_path/name do localStorage
- [x] Volta ao EmptyState
- [x] App continua compilando

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
```

## Não-objetivos

- Confirmar dirty state não salvo (futuro)
