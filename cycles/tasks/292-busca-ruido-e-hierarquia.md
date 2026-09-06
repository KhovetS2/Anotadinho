---
id: "292"
titulo: "Busca, o ruído da contagem e a hierarquia dos títulos"
status: done
criado: 2026-09-06
autor: agente
prioridade: alta
depende_de: ["291"]
estima_min: 180
---

# 292 — Busca, ruído e hierarquia

Três itens da lista, mais um achado por captura de tela.

## Critérios de aceite

- [x] `/` filtra o painel com foco, nos dois
- [x] Digitar na busca não dispara comando de vim
- [x] `Enter` mantém o filtro; `Escape` limpa
- [x] O cursor nunca aponta pro que sumiu
- [x] Com filtro, o movimento anda pela lista visível
- [x] A contagem só aparece com o nível fechado
- [x] `▾`/`▸` dizem que dá pra dobrar
- [x] h1 com faixa, h2 com régua, sem gastar linha

## Comandos de validação

```bash
cargo test -p anotadinho-tui
cargo clippy -p anotadinho-tui --all-targets
```
