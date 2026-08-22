---

## title: Arquitetura

tags: [docs, tech]
created: 2026-08-04


# Arquitetura

```
```
┌─────────────────────────────────────────────────────────────────┐ │ Anotadinho │ │ │ │ ┌─────────────┐ IPC commands ┌─────────────────────┐ │ │ │ Yew UI │ ◄────────────────► │ src-tauri (shell) │ │ │ │ (WASM) │ tauri::command │ Rust + Tauri 2 │ │ │ └─────────────┘ └──────────┬──────────┘ │ │ │ │ │ │ ▼ ▼ │ │ ┌─────────────┐ ┌─────────────────────┐ │ │ │ styles │ │ anotadinho-ipc │ │ │ │ (dark) │ │ (commands) │ │ │ └─────────────┘ └──────────┬──────────┘ │ │ │ │ │ ┌───────────────────┼────────┐ │ │ ▼ ▼ ▼ │ │ ┌────────────┐ ┌────────────┐ ┌─────┐ │ │ │ core │ │ vault │ │search│ │ │ │ block │ │ io,watch │ │ fts │ │ │ │ model, │ │ lock │ │ │ │ │ │ parser │ │ │ │ │ │ │ └────────────┘ └────────────┘ └─────┘ │ └─────────────────────────────────────────────────────────────────┘
```
```






## Camadas

### ui/ (Yew/WASM)

Componentes Yew que compilam pra WASM.
Chama backend via `tauri::command`.


### src-tauri/ (Tauri shell)

Entry point do app. Define comandos IPC.


### crates/ipc

Ponte entre Yew e crates de domínio.
Structs Args/Result por comando.


### crates/core

Block model, Markdown parser, properties inline.


### crates/vault

I/O de arquivos, watcher, locks.


### crates/search

Full-text search com SQLite FTS5 (futuro).

### crates/core, além dos modelos

Módulos que nasceram depois da primeira versão desta página e carregam
regra de negócio, não só estrutura:

| Módulo | O que decide |
|---|---|
| `embed` | Os 10 tipos de embed, parse/serialize e validação semântica |
| `query` | O motor de consulta, compartilhado entre UI e terminal |
| `fluxo` | As etapas do trabalho e quais transições existem |
| `proposta` | O que um agente pode propor, e o que é recusado |
| `conversa` | Formato da conversa em markdown e montagem do prompt |
| `agente` | Configuração do agente externo, sem shell |
| `diff` | Comparação linha a linha (conflito e revisão de proposta) |
| `history` | Desfazer/refazer por snapshot |

O critério de o que vive aqui: **lógica que a UI, o CLI e o servidor MCP
precisam concordar**. Se só a UI usa, fica na UI; se o terminal também
precisa decidir aquilo, desce pro core.

Ver também: [[Capacidades de agente]], [[Ciclos]].
