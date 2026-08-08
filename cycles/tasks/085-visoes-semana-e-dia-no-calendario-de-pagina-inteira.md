---
id: "085"
titulo: "Visões Semana e Dia no calendário de página inteira"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
