---
title: Ciclo 012 — Watcher de arquivos (auto-refresh via polling)
type: ciclo
ciclo: "012"
status: concluida
date: 2026-08-05
prioridade: media
depende_de: ["011"]
tags:
- ciclo
---

# Ciclo 012 — Watcher de arquivos (auto-refresh via polling)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Watcher de arquivos

## Objetivo

VaultWatcher monitora mudanças no vault. Frontend faz polling
a cada 3s e recarrega sidebar + editor se houver alterações.

## Critérios de aceite

- [x] VaultWatcher com notify no backend
- [x] IPC `check_changes(vault_path) -> bool`
- [x] Frontend poll 3s com gloo_timers
- [x] Sidebar recarrega se houver mudanças
- [x] Editor recarrega se página atual foi modificada
- [x] `cargo test --workspace` exit 0
- [x] App continua abrindo

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```

## Resultado

## Resumo
Ciclo 012: Watcher de arquivos (polling 3s).
- VaultWatcher com notify (inotify)
- IPC check_changes
- App faz polling a cada 3s e recarrega sidebar
