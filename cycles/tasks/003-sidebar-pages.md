---
id: "003"
titulo: "Sidebar com lista de páginas (pages/ e journals/)"
status: done
criado: 2026-08-04
autor: humano
prioridade: alta
depende_de: ["002"]
estima_min: 60
agente_alvo: claude-sonnet
finalizado: 2026-08-04T22:15:00
---

# Sidebar com lista de páginas

## Objetivo

Quando um vault está aberto, a sidebar mostra duas seções:
- **Pages**: lista de `.md` em `vault/pages/`
- **Journals**: lista de `.md` em `vault/journals/`

Click em um item deve carregar a página no editor (placeholder por enquanto).

## Critérios de aceite

- [x] Sidebar renderiza com 2 seções (Pages, Journals)
- [x] Cada seção lista os `.md` do diretório correspondente
- [x] Click num item seleciona (highlight visual)
- [x] Ordem alfabética
- [x] Empty state ("Nenhuma página ainda") se diretório vazio
- [x] Teste unitário em `vault::io::VaultIo::list_pages` que cria diretório
      temp, coloca 3 arquivos `.md`, valida retorno
- [x] `cargo test --workspace` exit 0
- [x] `cargo clippy --workspace --all-targets -- -D warnings` exit 0

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## Não-objetivos

- Não abrir/editar página (ciclos 004, 005)
- Não fazer busca (ciclo 011)
- Não fazer watcher (ciclo 009)

## Notas

- Usar `walkdir` (já em deps) ou `std::fs::read_dir`
- Filtrar apenas `.md`
- Path relativo ao vault (não absoluto) na UI
- IPC command novo: `list_pages(vault_path) -> Vec<PageMeta>`
