---
id: "469"
titulo: "Sprints — núcleo: sprint, filtro do quadro, encerramento e o plano de projeto"
status: done
criado: 2026-10-22
autor: agente
prioridade: alta
depende_de: ["468"]
estima_min: 210
---

# 469 — O núcleo das sprints e do plano de projeto

Primeiro ciclo da proposta `pages/propostas/sprints-no-projeto.md` (spec
`pages/specs/sprints-no-projeto.md`), aprovada em 2026-10-22. Só modelo
puro, sem IO.

## Critérios de aceite

- [x] `core::sprint`:
  - [x] `Sprint` (página `type: sprint` em `<projeto>/sprints/`) e
        `EstadoSprint` (planejada → ativa → encerrada).
  - [x] `mudar_estado` recusa uma segunda sprint ativa e diz qual está
        ativa.
  - [x] `Filtro { SprintAtiva, Backlog, Sprint(n), Todas }` e
        `tarefas_do_quadro`. "Todas" exclui as feitas de sprints
        encerradas. Sem sprint ativa, `SprintAtiva` cai em "todas"
        (RNF4).
  - [x] `plano_de_encerramento`: as tarefas não feitas vão pra próxima
        sprint planejada ou pro backlog. Sem sprint planejada, "próxima"
        é erro.
  - [x] `mudar_sprint_da_tarefa` grava ou TIRA o `sprint:` pelo
        `serde_yaml`.
- [x] `Tarefa` ganha `sprint: Option<u32>`. O número vem do nome do
      arquivo da sprint, então renomear a sprint não solta as tarefas.
- [x] `core::plano`:
  - [x] `PlanoDeProjeto` em YAML, com `deny_unknown_fields`: um campo
        errado da LLM vira erro com o nome do campo, não silêncio.
  - [x] `montar_plano` numera a partir do que já existe e resolve
        `sprint` e `depende` pelo título do plano.
  - [x] `MODELO_DE_PLANO`: um teste garante que o modelo é lido e vira
        páginas que `Tarefa::ler` e `Sprint::ler` entendem.
- [x] O índice das tarefas (contexto da conversa) diz a sprint de cada
      tarefa.

## Comandos de validação

```bash
cargo test -p anotadinho-core sprint
cargo test -p anotadinho-core plano
cargo test --workspace
```

## Não-objetivos

- IO, CLI e MCP (470 e 471), telas (472 e 473)
