---
title: Ciclo 004 — Editor Markdown básico (ler e exibir página)
type: ciclo
ciclo: "004"
status: concluida
date: 2026-08-04
prioridade: alta
depende_de: ["003"]
tags:
- ciclo
---

# Ciclo 004 — Editor Markdown básico (ler e exibir página)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Editor Markdown básico

## Objetivo

Click em uma página na sidebar carrega o conteúdo `.md` bruto no painel
principal, exibido num textarea editável. O conteúdo é só leitura do disco
neste ciclo (salvar fica pro 005).

## Critérios de aceite

- [x] Click na sidebar carrega o conteúdo da página no editor
- [x] Editor mostra o Markdown bruto (textarea monoespaçado)
- [x] Header do editor mostra o título da página selecionada
- [x] Se nenhuma página está selecionada, mostra placeholder
- [x] IPC `read_page(vault_path, page_path) -> String`
- [x] Teste unitário em `VaultIo::read_page` (cria temp file, lê conteúdo)
- [x] `cargo test --workspace` exit 0
- [x] `cargo build --workspace` e `trunk build` exit 0
- [x] App continua abrindo com vault picker + sidebar funcionando

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Não salvar edições (ciclo 005)
- Não parsear blocos/properties (ciclo 006)
- Não syntax highlight
- Não preview rendered

## Notas

- `VaultIo::read_page` retorna `String` com conteúdo UTF-8
- Path relativo ao vault (ex: `pages/sobre.md`)
- UI: componente `Editor` em `ui/src/components/editor.rs`

## Resultado

## Resumo
Ciclo 004: Editor Markdown básico (read-only).
- VaultIo::read_page + path traversal protection
- IPC read_page
- Componente Editor com textarea monoespaçado
- 17 testes passando
