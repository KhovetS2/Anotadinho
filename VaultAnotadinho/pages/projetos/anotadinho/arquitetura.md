---
title: Arquitetura — Anotadinho
tags:
- arquitetura
type: docs
date: 2026-10-22
projeto: anotadinho
---
# Arquitetura — Anotadinho

A fonte é `docs/architecture.md`, no repositório. Esta página é o mapa
curto pra conversa do projeto, e mudanças de verdade vão lá.

## Componentes

| Onde | O quê |
|---|---|
| `crates/core/` | modelos puros: blocos, markdown, embeds, consulta, fluxo, agente, projeto |
| `crates/vault/` | I/O dos arquivos e watcher |
| `crates/ipc/` | os `handle_*` que a janela, a TUI e o CLI compartilham |
| `crates/cli/` | `anotadinho-cli`, o canal do agente com o vault (inclui o servidor MCP) |
| `crates/tui/` | a interface de terminal |
| `crates/search/` | busca full-text |
| `ui/src/` | a janela, em Yew/WASM |
| `src-tauri/` | o shell Tauri, que roda o agente da janela |

## Dados

O vault é markdown com frontmatter YAML e embeds `{{ type: "..." }}`.
O estado do app que não é página mora em `.anotadinho/`: propostas,
execuções, gatilhos, permissões e projetos.

## Integrações

- Agente externo (`claude`, `codex`, `opencode`) por processo, sem
  shell, com o prompt pelo stdin.
- MCP do próprio Anotadinho (`anotadinho-cli mcp`): só `propor` escreve.

## Decisões

- A regra mora no `core`, e o IO compartilhado no `ipc`: duas
  implementações são duas UIs que divergem (ciclo 423).
- Só humano avança etapa. O app registra o que a decisão causou, e o
  agente propõe (ciclos 201 e 457).
