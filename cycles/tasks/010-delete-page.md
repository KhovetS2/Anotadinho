---
id: "010"
titulo: "Excluir página selecionada"
status: done
criado: 2026-08-04
autor: humano
prioridade: media
depende_de: ["009"]
estima_min: 30
agente_alvo: claude-sonnet
---

# Excluir página

## Objetivo

Botão "Excluir" no editor remove o arquivo `.md` do disco (com confirmação)
e atualiza a sidebar.

## Critérios de aceite

- [x] Botão Excluir no header do editor
- [x] Confirmação via dialog
- [x] IPC `delete_page`
- [x] Teste VaultIo::delete_page
- [x] Sidebar refresh + deseleciona página
- [x] App continua abrindo

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```

## Não-objetivos

- Lixeira / undo
- Excluir pastas
