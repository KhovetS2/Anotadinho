---
id: "474"
titulo: "Sprints — o guia e a escala: 300 feitas em sprints encerradas"
status: done
criado: 2026-10-22
autor: agente
prioridade: media
depende_de: ["473"]
estima_min: 120
---

# 474 — Docs e escala das sprints

Sexto e último ciclo da proposta `pages/propostas/sprints-no-projeto.md`.

## Critérios de aceite

- [x] Estresse (RNF3): um projeto com centenas de tarefas feitas em
      sprints encerradas e uma sprint ativa.
  - [x] O quadro da ativa tem SÓ as dela, e "todas" não traz as feitas
        das encerradas. Confere o efeito, não só o tempo.
  - [x] O custo por tarefa do estado do projeto e do recorte não muda de
        ordem entre 300 e 3000 tarefas.
  - [x] O teto foi visto reprovando.
- [x] O guia `pages/produto/como-usar-projetos.md` ganha a seção de
      sprints (janela, TUI, CLI e MCP), por `propor`.

## Comandos de validação

```bash
cargo test -p anotadinho-ipc --test estresse -- --ignored --nocapture sprints
cargo test --workspace
```
