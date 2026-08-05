---
id: "008"
titulo: "Journal do dia (botão Hoje)"
status: done
criado: 2026-08-04
autor: humano
prioridade: media
depende_de: ["007"]
estima_min: 30
agente_alvo: claude-sonnet
---

# Journal do dia

## Objetivo

Botão "Hoje" na seção Journals abre (ou cria) o journal do dia atual
em `journals/YYYY-MM-DD.md`.

## Critérios de aceite

- [x] Botão "Hoje" na seção Journals
- [x] Cria journal se não existir
- [x] Abre no editor se já existir
- [x] IPC `open_today_journal(vault_path) -> PageMeta`
- [x] Teste unitário
- [x] App continua compilando e abrindo

## Comandos de validação

```bash
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
```

## Não-objetivos

- Calendário de journals
- Templates avançados
