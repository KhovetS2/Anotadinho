# Arquitetura

```
┌─────────────────────────────────────────────────────────────────┐
│                        Anotadinho                              │
│                                                                 │
│  ┌─────────────┐    IPC commands     ┌─────────────────────┐   │
│  │  Yew UI     │ ◄────────────────► │  src-tauri (shell)  │   │
│  │  (WASM)     │     tauri::command  │  Rust + Tauri 2     │   │
│  └─────────────┘                     └──────────┬──────────┘   │
│       │                                         │              │
│       │ renderiza                                │ importa      │
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

## Camadas

### 1. `ui/` (Yew/WASM)
- Componentes Yew (React-like)
- Compila pra WASM, roda no WebView do Tauri
- Chama backend via `tauri::command`
- Tema dark com acentos azul/roxo (CSS variables)

### 2. `src-tauri/` (Tauri shell)
- Entry point do app
- Configura janela (1024x720, mínimo 800x600)
- Define comandos IPC (chama `anotadinho-ipc`)
- Bundle com `cargo tauri build`

### 3. `anotadinho-ipc` (commands)
- Ponte entre Yew e crates de domínio
- Define structs `Args` e `Result` por comando
- Stubs atualmente, implementações vêm nos ciclos

### 4. `anotadinho-core` (block model)
- `Block`, `BlockId`, `BlockKind` (Nota, Tarefa, Heading, etc)
- `Page`, `PageId`, `Frontmatter`
- `Property` (parser de `key:: value`)
- `MarkdownCodec` (parser/serializer - ciclo 006)

### 5. `anotadinho-vault` (I/O)
- `VaultIo` (ler/escrever/listar arquivos)
- `LockManager` (locks entre instâncias - ciclo 010)
- `VaultWatcher` (notify - ciclo 009)

### 6. `anotadinho-search` (busca)
- `SearchIndex` (FTS5 - ciclo 011)
- Embeddings opcionais (futuro)

## Fluxo de dados

**Ler página:**
```
Yew (click em page) → tauri::command("read_page")
  → src-tauri → anotadinho-ipc::read_page
  → anotadinho-vault::VaultIo::read_page
  → anotadinho-core::MarkdownCodec::parse
  → Page → JSON → Yew (renderiza)
```

**Escrever página:**
```
Yew (edita + save) → tauri::command("write_page")
  → anotadinho-vault::VaultIo::write_page
  → anotadinho-core::MarkdownCodec::serialize
  → escreve .md no disco
  → retorna OK pro Yew
```

## Vault format

`vault/pages/minha-nota.md`:
```markdown
---
title: Minha Nota
tags: [projeto]
created: 2026-08-04T10:00:00
---

- id:: 671c0a3e-...
  tipo:: nota
  Conteúdo do bloco com properties inline.
```

`vault/journals/2026_08_04.md`:
```markdown
---
title: 2026-08-04
---

- id:: ...
  Registro do dia.
```

`vault/.anotadinho/locks/<block-id>.lock`:
```
owner: hostname:pid
acquired: 2026-08-04T10:00:00
ttl: 300
```
