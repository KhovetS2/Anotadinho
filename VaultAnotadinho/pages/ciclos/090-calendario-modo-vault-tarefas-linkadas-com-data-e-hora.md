---
title: Ciclo 090 — Calendario modo Vault tarefas linkadas com data e hora
type: ciclo
ciclo: "090"
status: concluida
date: 2026-08-07
prioridade: alta
depende_de: ["078"]
tags:
- ciclo
---

# Ciclo 090 — Calendario modo Vault tarefas linkadas com data e hora

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Calendário modo Vault (tarefas linkadas com data/hora de entrega)

## Objetivo

Quinto ciclo do conjunto grande — a peça que conecta landing page
(ciclo 089) ao exemplo original do usuário: um calendário na landing
page mostrando tarefas (páginas com `date::`) com data e horário de
entrega, clicando abre a tarefa de verdade. O embed
`{{ type: "calendar" }}` ganha `mode: Manual | Vault`; Vault escaneia o
vault inteiro (mesma fonte da página `type: calendar`), somente
leitura.

## Critérios de aceite

- [x] `ui/src/embed.rs`: `CalendarEntry.page_path` (síntético, nunca
      serializado), `CalendarSource` (`Manual`/`Vault`),
      `CalendarEmbedData.mode`, `scan_vault_calendar_entries` (escaneia
      `date::`/`time::` de todas as páginas do vault) — com testes de
      round-trip/backward-compat
- [x] `ui/src/components/calendar.rs` (página inteira `type: calendar`,
      ciclo 085) refatorada pra reusar `scan_vault_calendar_entries` em
      vez de duplicar o loop de scan — ganha suporte a `time::` de
      graça, exibido na Lista e nas células da grade
- [x] `ui/src/components/embeds/inline_calendar.rs`: seletor "Manual /
      Vault" no cabeçalho (persistido em `data.mode` via `on_change`);
      modo Vault busca `vault_entries` via `scan_vault_calendar_entries`
      e renderiza com `render_vault_month_grid`/`render_vault_agenda`
      (funções NOVAS, somente leitura — não retrofita interatividade nas
      funções `render_month_grid`/`render_day_columns` existentes, zero
      risco de regressão no modo Manual)
- [x] Clicar um evento em modo Vault navega pra página de origem
      (`on_page_selected`); controles de criar/arrastar/gaveta ficam
      ocultos em modo Vault
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Sincronizar bidirecionalmente ao arrastar um evento em modo Vault —
  Vault é só leitura nesta v1; editar data/hora continua sendo feito na
  página de origem (`date::`/`time::` no frontmatter)
- Grade de horas posicionada por horário nas visões Semana/Dia do modo
  Vault — usa uma agenda simples (lista por dia, hora como texto), mais
  barato de implementar corretamente do que replicar o posicionamento
  por pixel do modo Manual; ainda mostra data E hora, que é o que o
  pedido original pedia
- Filtrar o modo Vault por tag/pasta/propriedade customizável — mostra
  TODAS as páginas com `date::` do vault, sem filtro

## Notas

Decisão de arquitetura (feita durante a implementação, não estava 100%
fechada no plano): em vez de adicionar um parâmetro `read_only` nas
funções `render_month_grid`/`render_day_columns` já existentes (900+
linhas, MUITO acopladas a `props.data.entries` por índice em cada
handler de arraste/resize/edição), criei funções irmãs SEPARADAS
(`render_vault_month_grid`/`render_vault_agenda`) que só reaproveitam as
partes puras (`pack_days`, cálculo de células do mês). Isso manteve o
modo Manual completamente intocado (zero risco de regressão numa parte
do código já grande e testada) ao custo de um pouco de duplicação
estrutural (grade de mês) — troca que valeu a pena dado o tamanho e
acoplamento do código existente.

`components/calendar.rs` (página inteira) foi atualizada pra chamar o
MESMO `scan_vault_calendar_entries` em vez de manter sua própria cópia
do loop de scan — ganha `time::` de graça, uma implementação só pra
manter.

Validado ao vivo via MCP `tauri`: inserir `{{ type: "calendar" }}` via
slash na página `teste`, trocar pra "Vault" → mostra as 3 páginas do
vault com `date::` (2 journals sem hora, 1 com `time:: 14:30` setado
temporariamente pro teste); clicar o evento com hora navega pra
`2026-08-06` de verdade; trocar visão pra Semana mostra a agenda com
"14:30 · 2026-08-06" no dia certo e "—" nos dias sem evento; reabrir a
página `teste` confirma que o modo Vault persistiu (`mode: vault` no
YAML do embed). Mudanças de teste revertidas em `VaultAnotadinho/`
antes de fechar o ciclo.

Achado incidental (comportamento pré-existente, não uma regressão deste
ciclo): inserir o embed calendário via slash já semeava um evento de
exemplo ("Novo evento" na data do relógio do sistema) — `editor.rs`
linha ~716, de um ciclo anterior. Confirmado que não é afetado pelas
mudanças deste ciclo.

## Resultado

# Ciclo 090 - done

## Resumo

Quinto ciclo do conjunto grande — a peça que conecta wikilinks/landing
page ao pedido original (calendário de tarefas linkadas com data/hora
de entrega). Embed `{{ type: "calendar" }}` ganha `mode: Manual|Vault`.

## Arquivos criados/modificados

- `ui/src/embed.rs` — `CalendarEntry.page_path`, `CalendarSource`,
  `CalendarEmbedData.mode`, `scan_vault_calendar_entries`, 4 testes
- `ui/src/components/calendar.rs` — reusa o scanner compartilhado,
  ganha `time::`
- `ui/src/components/embeds/inline_calendar.rs` — seletor de fonte,
  `render_vault_month_grid`/`render_vault_agenda` (novas, read-only)
- `ui/src/components/embeds/mod.rs` — thread `vault_path`/
  `on_page_selected` pro `InlineCalendar`
- `ui/src/styles/main.css` — `.calendar-grid__bar--readonly`,
  `.calendar-grid__vault-*`, `.calendar__item-time`

## Testes

`cargo test --workspace`: 48. `cd ui && cargo test --lib`: 66 (62 + 4
novos de `embed.rs`). Total 91 combinando os dois runs (workspace já
inclui parte do total do lib da ui separadamente, contabilizado à
parte por rodar em processos distintos).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Modo Vault mostra as páginas com `date::` do vault; clique navega pra
página real; `time::` aparece como prefixo; modo persiste ao salvar.
Detalhes no arquivo de task.

## Notas

Próximo: paleta de comandos (Ctrl+K).
