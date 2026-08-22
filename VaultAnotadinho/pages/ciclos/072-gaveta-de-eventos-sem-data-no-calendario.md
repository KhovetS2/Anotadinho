---
title: Ciclo 072 — Gaveta de eventos sem data no calendario
type: ciclo
ciclo: "072"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["071"]
tags:
- ciclo
---

# Ciclo 072 — Gaveta de eventos sem data no calendario

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Gaveta de eventos sem data no calendário

## Objetivo

Quinto item do backlog dos ciclos 066/067: `CalendarEntry.date` vira
`Option<String>`, e uma gaveta recolhível no rodapé do embed do
calendário lista os eventos sem data. Dá pra criar um evento sem data
direto, arrastar da gaveta pra um dia (mesmo mecanismo de `dragging` já
usado pelas barras/blocos da grade) ou abrir o modal e definir a data
pelo `DatePicker`.

## Critérios de aceite

- [x] `CalendarEntry.date: Option<String>` — `None` = sem data, não
      serializa a chave `date:` no YAML quando `None`
- [x] `CalendarEmbedData::add_unscheduled_entry(title)`
- [x] `move_entry`/`move_entry_time` funcionam pra atribuir data a um
      evento sem data (não só pra mover um já agendado) — sem duração
      antiga pra preservar nesse caso
- [x] `pack_days` e o filtro de blocos com horário ignoram eventos sem
      data (não aparecem na grade)
- [x] Gaveta no rodapé: botão de recolher/expandir com contagem, "+
      evento sem data", lista de pills arrastáveis
- [x] Arrastar um item da gaveta pra um dia (mês) ou coluna (semana/dia)
      atribui a data, some da gaveta, aparece na grade
- [x] `EventDetailModal`: evento sem data mostra só o campo "Início"
      (funciona como "definir data"); "Vários dias"/"Horário específico"
      só aparecem depois que a data existe
- [x] Retrocompatível: entradas antigas (sempre tinham `date:`) continuam
      parseando igual, sem mudança de comportamento

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Visões Semana/Dia na página de calendário inteira (`type: calendar`)
  — fica pra retomar depois (task #59 no tracker)

## Notas

Escopo do `Option<String>` ficou menor do que parecia — o componente de
calendário de página inteira (`ui/src/components/calendar.rs`, usado no
frontmatter `type: calendar`) usa um modelo de dados totalmente separado
(`DayItem`, raspado de `date::` nas páginas), não `CalendarEntry`, então
não precisou de nenhuma mudança.

Validado ao vivo via MCP `tauri`: criar evento sem data via botão da
gaveta (Prompt), abrir a gaveta e ver a contagem/pill, arrastar o pill
pra um dia do mês — some da gaveta, aparece na grade naquele dia,
contagem de "N eventos" no cabeçalho soma certo. Testado também o outro
caminho: criar sem data, clicar (não arrastar) pra abrir o modal, ver
"Sem data — clique para definir" no lugar do chip de data, clicar nele
pra abrir o `DatePicker` e escolher uma data — campo atualiza pra
`2026-08-15`, os campos "Vários dias"/"Horário específico" aparecem, e o
evento passa a aparecer na grade no dia escolhido.

`exemplos-embeds.md` ganhou um evento sem data (`Ligar pro fornecedor`,
sem chave `date:`) pra documentar a sintaxe e servir de fixture de
regressão (`exemplos_embeds_vault_file_parses` atualizado pra 5 entradas).

## Resultado

# Ciclo 072 - done

## Resumo

`CalendarEntry.date` vira `Option<String>` + gaveta recolhível de eventos
sem data no rodapé do embed do calendário (criar, arrastar pra um dia,
ou definir a data pelo modal).

## Arquivos criados/modificados

- `ui/src/embed.rs` — `CalendarEntry.date: Option<String>`,
  `add_unscheduled_entry`, `move_entry`/`move_entry_time` ajustados, 3
  testes novos + fixture do vault atualizada pra 5 entradas
- `ui/src/components/embeds/inline_calendar.rs` — `pack_days`/filtro de
  blocos com horário ignoram eventos sem data, gaveta (estado
  `drawer_open`, callback de criar sem data, lista de pills arrastáveis)
- `ui/src/components/embeds/event_detail_modal.rs` — campo "Início"
  mostra "Sem data — clique para definir" quando `entry.date` é `None`;
  "Vários dias"/"Horário específico" só aparecem com data definida
- `ui/src/styles/main.css` — `.calendar-grid__drawer*`
- `VaultAnotadinho/pages/exemplos-embeds.md` — 1 evento sem data de
  exemplo

## Testes

`cargo test --lib`: 52 passaram (3 novos: `calendar_add_unscheduled_*`,
`calendar_move_entry_assigns_date_to_unscheduled_entry`,
`calendar_unscheduled_entry_roundtrips_without_date_key`; mais o fixture
de regressão atualizado).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Criar evento sem data pela gaveta, abrir a gaveta, arrastar o pill pra um
dia do mês — confirmado que some da gaveta e aparece na grade naquele
dia. Testado também o caminho pelo modal: clicar (não arrastar) abre o
modal com "Sem data — clique para definir", clicar no chip abre o
`DatePicker`, escolher uma data assina ela, campos de "Vários
dias"/"Horário específico" aparecem depois.

## Notas

Nenhuma edição de teste vazou pro vault (`git diff --stat VaultAnotadinho/`
vazio antes de commitar, exceto a fixture `exemplos-embeds.md` que foi
intencionalmente editada como parte do ciclo).
