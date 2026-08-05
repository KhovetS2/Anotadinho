---
id: "005"
titulo: "Salvar página editada (Ctrl+S / botão Salvar)"
status: done
criado: 2026-08-04
autor: humano
prioridade: alta
depende_de: ["004"]
estima_min: 40
agente_alvo: claude-sonnet
---

# Salvar página editada

## Objetivo

O editor deixa de ser readonly. O usuário edita o Markdown e salva
com botão "Salvar" ou Ctrl+S. O conteúdo é escrito de volta no arquivo.

## Critérios de aceite

- [x] Textarea é editável (não readonly)
- [x] Botão "Salvar" no header do editor
- [x] Ctrl+S salva
- [x] IPC `write_page(vault_path, page_path, content)`
- [x] Teste unitário `VaultIo::write_page`
- [x] Indicador visual dirty (não salvo) e saved
- [x] `cargo test --workspace` exit 0
- [x] trunk build + src-tauri build exit 0
- [x] App continua abrindo (picker + sidebar + editor)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Autosave com debounce (futuro)
- Parser de blocos (ciclo 006)
- Watcher (ciclo 009)

## Notas

- write_page sobrescreve o arquivo inteiro
- Mesma proteção path traversal do read_page
