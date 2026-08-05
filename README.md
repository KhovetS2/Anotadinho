# Anotadinho

Editor de notas Markdown nativo, construído com **Tauri + Rust + Yew**.

Inspirado em [Obsidian](https://obsidian.md/) (vinhetas, backlinks, pastas) e [Logseq](https://logseq.com/) (outliner, block IDs, properties inline). Multiplataforma, offline-first, dados em `.md` legíveis.

## Stack

| Camada | Tecnologia |
|---|---|
| Runtime | Tauri 2.x |
| Backend | Rust puro |
| UI | Yew 0.21 (WASM) |
| Storage | Markdown + YAML frontmatter |
| Sync | Manual (pendrive / qualquer cópia) |
| Build | Cargo workspace |

## Estrutura

```
Anotadinho/
├── crates/
│   ├── core/         # block model, parser MD, properties, IDs
│   ├── vault/        # I/O de arquivos, watcher, locks
│   ├── search/       # full-text + embeddings
│   └── ipc/          # commands Tauri expostos pro Yew
├── ui/               # Yew frontend (WASM)
├── src-tauri/        # entry Tauri
├── cycles/           # sistema de implementação cíclica
├── docs/             # arquitetura, decisões, etc
└── tests/            # unit + integration + fixtures
```

## Desenvolvimento

```bash
# Clonar
git clone git@github.com:KhovetS2/Anotadinho.git
cd Anotadinho

# Rodar em dev
cargo tauri dev

# Build release
cargo tauri build
```

## Sistema de ciclos

O projeto evolui por **ciclos** (não por commits avulsos). Cada ciclo:
1. Lê uma task em `cycles/tasks/`
2. Implementa
3. Roda testes
4. Valida critérios
5. Salva status em `cycles/status/`
6. Próximo ciclo

Detalhes em [`docs/cycles.md`](docs/cycles.md).

## Licença

MIT — ver [`LICENSE`](LICENSE).
