---
title: Ciclo 003 — Sidebar com lista de páginas (pages/ e journals/)
type: ciclo
ciclo: "003"
status: concluida
date: 2026-08-04
prioridade: alta
depende_de: ["002"]
tags:
- ciclo
---

# Ciclo 003 — Sidebar com lista de páginas (pages/ e journals/)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

## Resumo

Ciclo 003: Sidebar com lista de páginas implementado.

### O que foi feito

- Comando IPC `list_pages` no backend (crates/ipc + src-tauri)
- Componente `Sidebar` no Yew UI com 2 seções: Pages e Journals
- Cada seção lista arquivos `.md` do diretório correspondente
- Click em item: destaque visual (azul) + callback `on_page_selected`
- Ordem alfabética por título
- Empty state "Nenhuma página ainda" se diretório vazio
- Loading state "Carregando..." durante fetch
- Ícones por seção (📄 pages, 📅 journals)
- CSS completo para sidebar: seções, itens, hover, selected

### Validação

- `cargo build --workspace`: OK (0 warnings)
- `cargo test --workspace`: 14/14 passed
- `cargo build --manifest-path src-tauri/Cargo.toml`: OK
- `trunk build` (Yew/WASM): OK (0 warnings)

### Arquivos modificados/criados

Modificados:
- crates/ipc/src/lib.rs (PageMeta, handle_list_pages)
- src-tauri/src/main.rs (comando list_pages)
- ui/src/api.rs (PageMeta, list_pages())
- ui/src/app.rs (Sidebar no layout, selected_page state)
- ui/src/components/mod.rs (sidebar module)
- ui/src/styles/main.css (sidebar styles)

Novos:
- ui/src/components/sidebar.rs
