---
id: "489"
titulo: "Construtor mermaid: formulários de sequência e gantt"
status: done
criado: 2026-10-23
autor: agente
prioridade: media
depende_de: ["488"]
estima_min: 300
---

# 489 — Sequência e gantt

Terceira etapa da proposta `pages/propostas/construtor-mermaid.md`.

## Critérios de aceite

- [x] **Sequência:**
  - participantes com nome e "pessoa" (`actor`);
  - mensagens com de, seta, para e texto;
  - notas à direita, à esquerda ou sobre um ou dois;
  - em todas as listas: adicionar, subir, descer e apagar.
- [x] **Gantt:**
  - título e seções;
  - tarefas com nome, marcas (feita, em andamento, crítica, marco),
    início (logo depois da anterior, numa data ou depois de outra),
    duração ou data de fim;
  - adicionar, subir, descer e apagar.
- [x] Nada do que se faz no formulário deixa o diagrama ilegível: data só
      pelo seletor, e duração e id só entram quando válidos. Apagar o que
      outro item cita fica bloqueado, com o motivo.
- [x] Dependência escolhida pelo nome da tarefa. A tarefa sem id ganha um.
- [x] O "Aplicar" leva a última edição, mesmo sem render entre ela e o
      clique.
- [x] Cenários de UI que montam um de cada e conferem a prévia sem erro.

## Comandos de validação

```bash
cd ui && cargo test --lib && trunk build
node scripts/uitest/run.mjs "(489)"
node scripts/uitest/run.mjs
```
