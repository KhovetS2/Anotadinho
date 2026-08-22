---
title: Ciclo 085 — Visões Semana e Dia no calendário de página inteira
type: ciclo
ciclo: "085"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 085 — Visões Semana e Dia no calendário de página inteira

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Visões Semana e Dia no calendário de página inteira

## Objetivo

O componente `ui/src/components/calendar.rs` (a página inteira acessada
pela aba lateral "Calendário", diferente do embed `{{ type: "calendar" }}`
que já tinha grade mensal) só tinha a lista cronológica original. Adiciona
visões de grade Mês/Semana/Dia — só navegação/visualização de páginas com
`date::`, sem edição — reaproveitando o layout visual da grade mensal já
existente no embed.

## Critérios de aceite

- [x] Seletor de visão (Lista/Mês/Semana/Dia) no cabeçalho, junto com
      navegação prev/próximo/Hoje quando fora da Lista
- [x] Visão Mês: grade de 42 células (dias do mês anterior/seguinte
      esmaecidos), cada célula lista as páginas daquele dia
- [x] Visão Semana: 7 colunas a partir do domingo da semana do "âncora"
- [x] Visão Dia: 1 coluna só, o dia do "âncora"
- [x] Clicar um item em qualquer visão de grade abre a página
      correspondente, igual ao comportamento já existente da Lista
- [x] Navegação prev/próximo desloca o "âncora" por mês/semana/dia
      conforme a visão ativa; "Hoje" volta pro dia atual
- [x] `cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Edição de eventos nessa visão (isso é papel do embed
  `{{ type: "calendar" }}`, que já tem modal de evento, resize, drag,
  drawer de não-agendados etc.) — esta página é somente leitura/navegação
- Arrastar itens entre dias nesta visão
- Grade de horas (blocos com horário específico) — os itens aqui vêm só
  de `date::` no frontmatter da página, sem conceito de horário

## Notas

Reaproveita `week_start`/`add_months` (novos helpers locais) e
`date_util::{weekday_of, days_in_month, prev_month, next_month,
add_days, parse_date, format_date, month_name}` já existentes. A
construção da grade de 42 células é a mesma lógica já usada em
`inline_calendar.rs::render_month_grid`, só que sem os handlers de
drag/resize/click-to-create (view é somente leitura).

Classes CSS novas: `.page-calendar__*` (grade/células/itens), mais
`.calendar__header-spacer` pra acomodar os novos controles no cabeçalho
sem quebrar o layout. Reaproveita `.calendar-grid__nav-btn`,
`.calendar-grid__month-label`, `.calendar-grid__today-btn`,
`.calendar-grid__view-select` do embed pra manter consistência visual
dos controles.

Validado ao vivo via MCP `tauri`: troca entre as 4 visões renderiza
corretamente; navegação prev/next em Mês/Semana/Dia confirmada (um teste
inicial com 3 cliques em sequência rápida via chamadas de ferramenta
separadas pareceu mover 4 dias em vez de 3 — testado de novo com cliques
isolados e confirmado que a lógica está correta, foi artefato de
timing do teste, não bug real). Clique num item da visão Dia (página
"2026-08-04") abriu a página corretamente, mesmo comportamento da Lista.

## Resultado

# Ciclo 085 - done

## Resumo

Adiciona visões de grade Mês/Semana/Dia ao calendário de página inteira
(`ui/src/components/calendar.rs`), que antes só tinha a lista
cronológica. Reaproveita o layout visual da grade mensal já existente no
embed `{{ type: "calendar" }}`, mas como visão somente-leitura (sem
drag/resize/criação rápida) — o embed continua sendo o único lugar com
edição de eventos.

## Arquivos criados/modificados

- `ui/src/components/calendar.rs` — reescrito: `ViewMode`
  (List/Month/Week/Day), `week_start`/`add_months` helpers,
  `render_day_cell`, navegação prev/next/Hoje por âncora, seletor de
  visão no cabeçalho
- `ui/src/styles/main.css` — `.page-calendar__*` (grade/células/itens),
  `.calendar__header-spacer`

## Testes

`cargo test --lib`: 54 passaram (sem testes novos — view somente-leitura
sem lógica de parsing nova, validado via MCP ao vivo).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Troca entre as 4 visões renderiza corretamente. Navegação prev/next
confirmada em Mês/Semana/Dia (isolando cliques um por vez). Clique num
item da visão Dia ("2026-08-04") abriu a página correspondente, mesmo
comportamento já existente da Lista.

## Notas

Um teste inicial de 3 cliques em "prev" (visão Dia) em sequência rápida
via chamadas de ferramenta separadas pareceu mover o âncora 4 dias em
vez de 3 — reteste com cliques isolados (cada um verificado antes do
próximo) confirmou que a lógica de navegação está correta (7→6→5→4).
Consistente com artefatos de timing já observados em ciclos anteriores
ao clicar rápido demais via MCP; não é um bug de código.
