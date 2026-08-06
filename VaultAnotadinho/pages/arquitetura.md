---

## title: Arquitetura

tags: [docs, tech]
created: 2026-08-04


# Arquitetura


```
```


┌─────────────────────────────────────────────────────────────────┐
│                        Anotadinho                              │
│                                                                 │
│  ┌─────────────┐    IPC commands     ┌─────────────────────┐   │
│  │  Yew UI     │ ◄────────────────► │  src-tauri (shell)  │   │
│  │  (WASM)     │     tauri::command  │  Rust + Tauri 2     │   │
│  └─────────────┘                     └──────────┬──────────┘   │
│       │                                         │              │
│       ▼                                         ▼              │
│  ┌─────────────┐                     ┌─────────────────────┐   │
│  │   styles    │                     │  anotadinho-ipc     │   │
│  │  (dark)     │                     │  (commands)         │   │
│  └─────────────┘                     └──────────┬──────────┘   │
│                                                  │              │
│                              ┌───────────────────┼────────┐     │
│                              ▼                   ▼        ▼     │
│                       ┌────────────┐    ┌────────────┐  ┌─────┐ │
│                       │  core      │    │  vault     │  │search│ │
│                       │  block     │    │  io,watch  │  │ fts  │ │
│                       │  model,    │    │  lock      │  │      │ │
│                       │  parser    │    │            │  │      │ │
│                       └────────────┘    └────────────┘  └─────┘ │
└─────────────────────────────────────────────────────────────────┘


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
