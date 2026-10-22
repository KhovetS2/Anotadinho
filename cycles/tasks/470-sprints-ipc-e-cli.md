---
id: "470"
titulo: "Sprints — ipc e CLI: criar, iniciar, encerrar, mover de sprint, e montar o projeto por plano"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["469"]
estima_min: 210
---

# 470 — As rotas das sprints e do plano, no `ipc` e no CLI

Segundo ciclo da proposta `pages/propostas/sprints-no-projeto.md`.

## Critérios de aceite

- [x] `ipc`, gravando PELA PESSOA (janela e TUI chamam estes):
  - [x] `handle_sprints_do_projeto` lê só `sprints/`;
  - [x] `handle_criar_sprint` numera sem colidir;
  - [x] `handle_iniciar_sprint` recusa a segunda ativa;
  - [x] `handle_encerrar_sprint(destino)` grava a sprint e as pendentes;
  - [x] `handle_mudar_sprint_da_tarefa`.
- [x] `EstadoDoProjeto` ganha as sprints, a ativa e os avisos de sprint.
- [x] `ipc`, PROPONDO (o canal do agente; o MCP do 471 chama estes):
  - [x] `handle_propor_plano` gera o projeto inteiro num lote só,
        numerado a partir do que existe;
  - [x] `handle_propor_sprint`;
  - [x] `handle_propor_tarefa`, com sprint pelo nome ou pelo número.
- [x] CLI (o que escreve, propõe):
  - [x] `projeto modelo-de-plano` imprime `MODELO_DE_PLANO`;
  - [x] `projeto montar --file plano.yaml` (ou `-` pelo stdin);
  - [x] `projeto ver <slug>` mostra sprints e tarefas com a sprint de
        cada uma (`--json` também);
  - [x] `sprint nova|listar|iniciar|encerrar --pendentes proxima|backlog`.
        Encerrar vai num lote: a sprint e as tarefas.
  - [x] `tarefa nova --sprint <nome|n>` e
        `tarefa sprint <caminho> <n|backlog>`.
- [x] `contexto` mostra a sprint ativa de cada projeto.
- [x] Testes no `ipc` e de integração no CLI, com o ciclo
      montar → aplicar → ver.

## Comandos de validação

```bash
cargo test -p anotadinho-ipc projeto
cargo test -p anotadinho-cli
cargo test --workspace
cargo check -p anotadinho-cli --target x86_64-pc-windows-gnu
```

## Não-objetivos

- O MCP (471) e as telas (472 e 473)
