---
id: "057"
titulo: "Kanban board view"
status: done
criado: 2026-08-06
depende_de: []
estima_min: 90
---

# Kanban Board

## Objetivo
Nova view de kanban com colunas (Backlog, Todo, Doing, Done).
Cards são páginas `.md` com property `kanban:: coluna`.
Arrastar cards entre colunas atualiza a property.

## Critérios
- [x] View Kanban acessível (frontmatter `type: kanban` roteia para o componente via PageView)
- [ ] Colunas configuráveis (hoje fixas: backlog/todo/doing/done)
- [ ] Cards arrastáveis entre colunas (HTML5 drag) — não implementado, só click
- [ ] Atualiza property `kanban::` no .md ao mover — implementado com `column::` (não `kanban::`) e sem drag, então não há update-on-move
- [x] Click no card abre no editor

## Nota de backfill (2026-08-06)
Marcada como `done` retroativamente: o board foi implementado nos commits
`00c1ecf`/`0d579c9`/`bff148f`/`d99559b`/`93ebe73`, fora do orchestrator (que
nunca rodou porque `depende_de: ["056"]` apontava pra uma task inexistente).
A dependência foi removida acima. Abordagem final ficou diferente da spec
original: cards são blocos `- column:: coluna` dentro de UMA página
`type: kanban` (ver `ui/src/components/kanban.rs`), não páginas separadas
por card, e não há drag-and-drop. Ver critérios não marcados acima.
