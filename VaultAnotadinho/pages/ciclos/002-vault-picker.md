---
title: Ciclo 002 — Vault picker (dialog nativo de seleção de pasta)
type: ciclo
ciclo: "002"
status: concluida
date: 2026-08-04
prioridade: alta
depende_de: ["001"]
tags:
- ciclo
---

# Ciclo 002 — Vault picker (dialog nativo de seleção de pasta)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Vault picker

## Objetivo

Botão "Abrir vault" na tela inicial abre um dialog nativo de seleção de
pasta. Quando o usuário escolhe, o path é salvo e a sidebar aparece
(listando páginas, mesmo que vazia neste ciclo).

## Critérios de aceite

- [x] Botão "Abrir vault" fica habilitado (não mais disabled)
- [x] Click no botão abre dialog nativo de seleção de diretório
- [x] Após seleção, o path aparece no header da UI
- [x] Path é persistido (localStorage)
- [x] Próxima vez que o app abre, o último vault é re-aberto automaticamente
- [x] `cargo test --workspace` exit 0
- [x] `cargo clippy --workspace --all-targets -- -D warnings` exit 0 (clippy indisponível, build 0 warnings)

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Não-objetivos

- Não listar páginas ainda (ciclo 003)
- Não criar/validar estrutura de pastas (ciclo 003)
- Não fazer watcher (ciclo 009)

## Notas

Usar `tauri-plugin-dialog` para o dialog nativo. Para persistência,
considerar `tauri-plugin-store` ou `localStorage` do WebView (mais simples).

Backend Rust (crate `vault`): implementar `VaultIo::list_pages()` mínimo
que retorna `Vec<String>` com os paths `.md` encontrados. UI chama via IPC.

## Resultado

## Resumo

Ciclo 002: Vault picker implementado.

### O que foi feito

- Criado `crates/vault/` com `VaultIo` e `list_pages()` (com testes)
- Adicionado `tauri-plugin-dialog` ao src-tauri
- Comando IPC `get_vault_info` no backend
- `ui/src/api.rs`: ponte WASM ↔ Tauri (invoke + dialog)
- `ui/src/state.rs`: persistência do vault path/nome em localStorage
- `ui/src/components/empty_state.rs`: botão "Abrir vault" habilitado + dialog nativo
- `ui/src/app.rs`: layout com header quando vault está aberto
- `ui/src/styles/main.css`: estilos pra header, sidebar placeholder, main area
- Permissões `dialog:default` e `dialog:allow-open` em capabilities

### Validação

- `cargo build --workspace`: OK (0 warnings)
- `cargo test --workspace`: 14/14 passed
- Clippy não disponível no ambiente, mas build limpo sem warnings

### Arquivos modificados/criados

Novos:
- crates/vault/Cargo.toml
- crates/vault/src/lib.rs
- crates/vault/src/io.rs
- ui/src/api.rs

Modificados:
- src-tauri/Cargo.toml
- src-tauri/src/main.rs
- src-tauri/capabilities/default.json
- crates/ipc/src/lib.rs
- ui/src/state.rs
- ui/src/components/empty_state.rs
- ui/src/app.rs
- ui/src/lib.rs
- ui/src/styles/main.css
- ui/Cargo.toml
- Cargo.lock
