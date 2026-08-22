---
title: Ciclo 007 — Criar nova página (botão + na sidebar)
type: ciclo
ciclo: "007"
status: concluida
date: 2026-08-04
prioridade: alta
depende_de: ["006"]
tags:
- ciclo
---

# Ciclo 007 — Criar nova página (botão + na sidebar)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Criar nova página

## Objetivo

Botão "+" na seção Pages da sidebar cria uma nova página `.md`
em `pages/` com frontmatter básico e abre no editor.

## Critérios de aceite

- [x] Botão "+" na seção Pages
- [x] Cria arquivo `pages/<slug>.md` com frontmatter title
- [x] Sidebar atualiza e seleciona a nova página
- [x] IPC `create_page(vault_path, title) -> PageMeta`
- [x] Teste `VaultIo::create_page`
- [x] `cargo test --workspace` exit 0
- [x] App continua abrindo

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Criar journals (futuro / journal de hoje)
- Dialog de nome (usa prompt simples ou título gerado)
- Delete page

## Notas

Slug: lowercase, espaços → hífens, só alfanuméricos e hífen.
Se arquivo existe, acrescenta `-2`, `-3`, etc.

## Resultado

## Resumo
Ciclo 007: Criar nova página via botão + na sidebar.
