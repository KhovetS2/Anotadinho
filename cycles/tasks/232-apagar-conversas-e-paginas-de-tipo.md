---
id: "232"
titulo: "Apagar conversas e páginas de tipo"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: []
estima_min: 60
---

# 232 — Apagar conversas e páginas de tipo

## Objetivo

`delete_page` existe ponta a ponta desde sempre, mas o único botão morava
no menu do cabeçalho do Editor. Conversa, kanban, calendário, tabela,
tags, assets e grafo não passam pelo Editor — nenhuma delas tinha como ser
apagada pela interface. Conversa criada por engano ficava para sempre.

## Critérios de aceite

- [x] Cada item da sidebar tem excluir, aparecendo no hover como o mover
- [x] Pergunta antes, e cancelar não apaga nada
- [x] Comando na paleta apaga a página aberta
- [x] Apagar a página aberta fecha a aba dela
- [x] Apagar outra página não tira a pessoa de onde ela está

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Lixeira com restauração (o vault está no git)
- Apagar pasta inteira
