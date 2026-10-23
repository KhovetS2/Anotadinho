---
id: "472"
titulo: "Sprints — na janela: seletor no quadro, aba Sprints, encerrar com a pergunta, sprint no cartão"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["471"]
estima_min: 240
---

# 472 — As sprints na janela

Quarto ciclo da proposta `pages/propostas/sprints-no-projeto.md`.

## Critérios de aceite

- [x] O cabeçalho do projeto tem um seletor: a sprint ativa (padrão), as
      planejadas, o backlog e "todas". As encerradas não aparecem: são
      histórico (RF5). Com uma sprint ativa, o cabeçalho mostra o
      objetivo e o fim dela.
- [x] O quadro mostra só o que o filtro deixa (`sprint::tarefas_do_quadro`).
      As dependências continuam olhando o projeto INTEIRO.
- [x] O cartão tem um seletor de sprint: backlog, a ativa e as
      planejadas. Ele não dispara o arrasto.
- [x] Aba **Sprints**:
  - [x] criar uma sprint (nome, objetivo, início e fim);
  - [x] iniciar uma planejada (o erro de "já há uma ativa" aparece);
  - [x] encerrar a ativa: um painel pergunta se as N pendentes vão pra
        próxima sprint ou pro backlog, e "próxima" fica desligado quando
        não há sprint planejada;
  - [x] o histórico das encerradas: datas, objetivo, feitas/total e os
        títulos das feitas.
- [x] Projeto sem sprint: o seletor diz "Todas (nenhuma sprint ativa)", e
      o quadro é o de antes (RNF4).
- [x] DOM com `data-nav-*`, CSS com tokens e em BEM.
- [x] Harness: o seletor filtra; mover de sprint pelo cartão; criar,
      encerrar (pergunta → próxima) e iniciar pela aba; o histórico.

## Comandos de validação

```bash
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs "(472)"
node scripts/uitest/run.mjs
```
