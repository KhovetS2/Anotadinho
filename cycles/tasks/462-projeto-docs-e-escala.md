---
id: "462"
titulo: "Módulo Projeto — guia de uso, padrão de agente e execução, e escala"
status: done
criado: 2026-10-21
autor: agente
prioridade: media
depende_de: ["461"]
estima_min: 120
---

# 462 — Documentação e escala do módulo Projeto

Último ciclo da proposta `pages/propostas/modulo-projeto.md`.

## Critérios de aceite

- [x] `pages/produto/como-usar-projetos.md` mostra o fluxo pela visão
      de quem usa, na janela e na TUI. Entra no vault por `propor`.
- [x] O padrão `agente-e-execucao` ganha a regra da fila de tarefas: por
      nível, com a reserva ANTES de iniciar, e o sistema só registrando
      o que a decisão humana causou. A mudança vai na semente (direto) e
      na página do vault (por `propor`).
- [x] O guia do modo agêntico aponta pro de projetos.
- [x] Bateria de escala: um projeto com até 2000 tarefas.
  - [x] Mede `handle_tarefas_a_executar` e `handle_estado_do_projeto`,
        e afirma que o custo por tarefa não cresce com o tamanho do
        projeto.
  - [x] O teto foi VISTO reprovando antes de valer (ciclo 259).

## Comandos de validação

```bash
cargo test -p anotadinho-ipc --test estresse -- --ignored --nocapture
cargo test --workspace
```

## Não-objetivos

- O cenário de estresse da UI com o quadro: a tela desenha a lista que o
  `ipc` entrega, e a ordem do custo está no `ipc`
