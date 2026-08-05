---
id: "012"
titulo: "Watcher de arquivos (auto-refresh via polling)"
status: done
criado: 2026-08-05
autor: humano
prioridade: media
depende_de: ["011"]
estima_min: 40
agente_alvo: claude-sonnet
---

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
