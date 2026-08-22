---
title: Ciclo 127 — Cards e linhas clicaveis viram operaveis por teclado
type: ciclo
ciclo: "127"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: ["123"]
tags:
- ciclo
---

# Ciclo 127 — Cards e linhas clicaveis viram operaveis por teclado

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

- [x] Cada um dos 4 ganha `tabindex="0"` + handler de `onkeydown` que
      trata Enter/Espaço como equivalente ao clique (reaproveita o
      mesmo callback já usado no `onclick`, só adiciona o gatilho de
      teclado)
- [x] `<th onclick>` de ordenar tabela (`task_table.rs`) recebe o
      mesmo tratamento — focável, Enter/Espaço ordena
- [x] Nenhuma regressão no comportamento de clique de mouse existente
- [x] Indicador de foco visível em cada um (regra genérica do ciclo
      123 cobriu — são elementos HTML normais, confirmado sem CSS
      extra necessário)
- [x] `cd ui && cargo test --lib`, `trunk build` passam
- [x] Validação ao vivo via MCP `tauri`: card do kanban (Enter reabriu
      a página-container, mesmo comportamento do clique — cards de
      kanban vivem todos na mesma página, então "reabrir" é o
      resultado correto), linha de `tabela-tarefas` (Enter abriu a
      página da tarefa numa aba nova), cabeçalho `<th>Status</th>`
      (Espaço reordenou as linhas), item de `calendario` (tabindex
      confirmado), chip de tag (Enter navegou pra página de origem
      numa aba nova) — todos confirmados

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

Diferente do padrão dos ciclos anteriores (checagem de tecla inline em
cada componente), aqui a repetição justificou extrair um helper
compartilhado: `ui/src/keyboard_activate.rs::activate_on_enter_or_space`,
que recebe um `Callback<()>` de "ativar" e devolve o `onkeydown` pronto
(mesma checagem `.key() == "Enter" || .key() == " " || .code() ==
"Space"` reforçada no ciclo 126). Cada site de clique agora constrói um
`Callback<()>` de ativação, reusado tanto no `onclick` (via
`.reform`-like wrapper que ignora o `MouseEvent`) quanto no `onkeydown`
gerado pelo helper — evita duplicar a lógica de "o que fazer ao
ativar" entre mouse e teclado, não só a checagem de tecla.

Achado durante a validação: `kanban__card` não navega pra outra
página ao ativar — todos os cards de um board vivem na MESMA página
(são linhas `- column:: X title:: Y` dentro de um único arquivo
`.md`), então o callback de ativação sempre reabre a página atual.
Isso já era o comportamento do `onclick` original; Enter/Espaço só
reproduz o mesmo resultado, não é regressão nem bug novo.

## Resultado

# Ciclo 127 - done

## Resumo

Os 4 lugares com itens clicáveis "bobos" (kanban, calendário, tabela
de tarefas, chips de tag) ganham `tabindex="0"` + `onkeydown`
(Enter/Espaço equivale ao clique). Extraído um helper compartilhado
(`ui/src/keyboard_activate.rs`) em vez de repetir a checagem de tecla
inline 8 vezes.

## Arquivos criados/modificados

- `ui/src/keyboard_activate.rs` (novo) — `activate_on_enter_or_space`
- `ui/src/lib.rs` — registra o módulo
- `ui/src/components/kanban.rs` — card ganha tabindex/onkeydown
- `ui/src/components/calendar.rs` — item de célula e item de lista
- `ui/src/components/task_table.rs` — `<tr>` de linha e os 3 `<th>`
  de ordenar
- `ui/src/components/tags_page.rs` — chip de página

## Testes

`cd ui && cargo test --lib`: 79. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: card do kanban (Enter reabriu a
própria página, comportamento correto — ver Notas do arquivo de task),
linha de `tabela-tarefas` (Enter abriu a página da tarefa numa aba
nova), `<th>Status</th>` (Espaço reordenou as linhas visivelmente),
item de `calendario` (tabindex confirmado), chip de `tags` (Enter
navegou pra `exemplos-embeds` numa aba nova) — todos confirmados sem
regressão no clique de mouse.

## Notas

Ver Notas detalhadas no arquivo de task sobre o helper compartilhado e
o achado de que cards de kanban vivem todos na mesma página (não é
bug, é o modelo de dados existente desde antes deste ciclo).

Página de teste temporária `_teste-tags-temp.md` (`type: tags`) criada
e removida do vault — não existia página desse tipo no vault real.

Próximo: criação de página de tipo específico via paleta de comandos
(128).
