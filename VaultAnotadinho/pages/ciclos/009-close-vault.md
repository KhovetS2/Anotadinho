---
title: Ciclo 009 — Fechar vault (voltar à tela inicial)
type: ciclo
ciclo: "009"
status: concluida
date: 2026-08-04
prioridade: media
depende_de: ["008"]
tags:
- ciclo
---

# Ciclo 009 — Fechar vault (voltar à tela inicial)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

## Resumo
Ciclo 009: Fechar vault.
