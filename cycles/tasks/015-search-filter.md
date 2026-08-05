---
id: "015"
titulo: "Busca de páginas por título (search bar na sidebar)"
status: done
criado: 2026-08-05
autor: humano
prioridade: alta
depende_de: ["014"]
estima_min: 30
agente_alvo: claude-sonnet
---

# Busca de páginas

## Objetivo

Campo de busca na sidebar filtra páginas por título. Se o termo
aparecer no título (case-insensitive), a página aparece nos resultados.

## Critérios de aceite

- [x] Search bar no topo da sidebar
- [x] Filtro case-insensitive por título
- [x] Limpa filtro com Escape ou botão X
- [x] Empty state "Nenhum resultado"
- [x] App continua compilando

## Comandos de validação

```bash
cd ui && trunk build
cargo test --workspace
```
