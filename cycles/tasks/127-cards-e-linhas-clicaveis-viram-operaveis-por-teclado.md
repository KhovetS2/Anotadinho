---
id: "127"
titulo: "Cards e linhas clicaveis viram operaveis por teclado"
status: pending
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: ["123"]
estima_min: 90
agente_alvo: claude-sonnet
---

# Cards e linhas clicáveis viram operáveis por teclado

## Objetivo

Auditoria encontrou 4 lugares com itens clicáveis que são elementos
"bobos" (`<div>`/`<span>`/`<tr>` com só `onclick`, sem `tabindex` nem
handler de teclado) — mouse-only, invisíveis pro Tab:

- `kanban.rs:99` — `<div class="kanban__card" onclick>`
- `calendar.rs:179-180`/`:210-211` — itens de dia/lista
  (`page-calendar__cell-item`/`calendar__item`)
- `task_table.rs:112-113` — `<tr class="task-table__row" onclick>`
  (e os `<th onclick>` de ordenar, linhas 86/90/94)
- `tags_page.rs:75` — `<span class="tags-page__page-chip" onclick>`

## Critérios de aceite

- [ ] Cada um dos 4 ganha `tabindex="0"` + handler de `onkeydown` que
      trata Enter/Espaço como equivalente ao clique (reaproveita o
      mesmo callback já usado no `onclick`, só adiciona o gatilho de
      teclado)
- [ ] `<th onclick>` de ordenar tabela (`task_table.rs`) recebe o
      mesmo tratamento — focável, Enter/Espaço ordena
- [ ] Nenhuma regressão no comportamento de clique de mouse existente
- [ ] Indicador de foco visível em cada um (regra genérica do ciclo
      123 deve cobrir, já que são elementos HTML normais — só
      confirmar visualmente, sem CSS extra esperado)
- [ ] `cd ui && cargo test --lib`, `trunk build` passam
- [ ] Validação ao vivo via MCP `tauri`: Tab até um card do kanban, um
      evento do calendário, uma linha da tabela, e um chip de tag —
      confirma foco visível e Enter/Espaço ativando cada um

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Trocar a tag HTML de `<div>`/`<tr>` pra `<button>` de verdade — um
  `<tr>` não pode virar `<button>` sem quebrar semântica de tabela;
  `tabindex` + handler de teclado é a abordagem correta pra esses
  casos (padrão ARIA `role="button"` implícito o suficiente pro
  escopo aqui, sem precisar formalizar `role`/`aria-*` completo)
- Navegação por seta ENTRE cards/linhas dentro dessas views (ex: seta
  pra baixo pula pro próximo card do kanban) — Tab sequencial já
  resolve o mínimo; navegação por seta dedicada é extensão futura se
  pedirem, uma view de cada vez

## Notas

Escopo deliberadamente maior que os outros ciclos deste tema (4
componentes em vez de 1) porque o fix é o MESMO padrão pequeno
repetido 4 vezes (`tabindex` + `onkeydown` chamando o callback que já
existe) — não há lógica nova por componente, só aplicação repetida.
