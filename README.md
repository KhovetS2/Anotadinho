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

## Embeds inline

Qualquer página `.md` pode conter blocos interativos, escritos como
`{{ type: "X" }} ... {{ /X }}` — sintaxe que não existe em CommonMark,
então nunca colide com markdown normal. São 9 tipos, todos inseridos
pelo menu `/` do editor:

| Tipo | O que é |
|---|---|
| `kanban` | board com colunas, cards, checklist, anexos |
| `calendar` | eventos por data, com visões de mês/semana/dia |
| `table` | tabela com colunas tipadas (select, data, número, link de página...) |
| `callout` | caixa de destaque colapsável |
| `columns` | painéis de markdown lado a lado |
| `gallery` | grade de imagens do vault com lightbox |
| `query` | lista viva de páginas filtradas por pasta/tag/propriedade |
| `timeline` | cronograma de barras por intervalo de datas |
| `actions` | botões que criam páginas de template, abrem páginas, gravam propriedades |

O conteúdo de cada um fica em YAML legível dentro do próprio `.md`, e o
`anotadinho-cli` lê e escreve os mesmos blocos pelo terminal
(`anotadinho-cli embed --help`).

## Estrutura

```
Anotadinho/
├── crates/
│   ├── core/         # block model, parser MD, embeds, índice, consulta
│   ├── vault/        # I/O de arquivos, watcher, locks
│   ├── search/       # full-text (FTS5)
│   ├── ipc/          # commands Tauri expostos pro Yew
│   └── cli/          # anotadinho-cli (acesso headless ao vault)
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

GPL-3.0-or-later — ver [`LICENSE`](LICENSE).
