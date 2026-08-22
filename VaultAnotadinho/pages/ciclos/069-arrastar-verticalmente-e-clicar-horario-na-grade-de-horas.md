---
title: Ciclo 069 — Arrastar verticalmente e clicar horario na grade de horas
type: ciclo
ciclo: "069"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["068"]
tags:
- ciclo
---

# Ciclo 069 — Arrastar verticalmente e clicar horario na grade de horas

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Arrastar verticalmente + clicar horário na grade de horas (calendário)

## Objetivo

Primeiros dois itens do backlog acumulado dos ciclos 066/067: arrastar um
evento com horário verticalmente na grade de Semana/Dia muda o horário
(preservando a duração), e clicar num ponto específico da grade cria o
evento já com aquele horário em vez de sempre dia inteiro. Os dois
compartilham a mesma matemática pixel↔horário, por isso entraram juntos
num ciclo só.

## Critérios de aceite

- [x] `CalendarEmbedData::add_entry_timed` — cria evento já com
      `start_time`/`end_time`
- [x] `CalendarEmbedData::move_entry_time` — muda `date`+`start_time`
      preservando a duração em minutos (se o evento já tinha horário);
      evento sem horário não ganha um do nada
- [x] Clicar numa célula vazia da grade de horas (Semana/Dia) cria evento
      com o horário do clique arredondado pro quarto de hora mais
      próximo (`y_to_snapped_minutes`, snap de 15 em 15 min)
- [x] Arrastar um bloco de horário verticalmente (ou entre dias, na
      visão Semana) solta com o novo horário/dia, preservando a duração
- [x] Evento sem horário que por algum motivo for solto na grade de
      horas só muda a data (não inventa um horário)

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redimensionar a duração arrastando a borda do bloco — próximo ciclo
- `TimePicker` customizado — ciclo separado
- Snap configurável (fixo em 15 min por enquanto)

## Notas

Validado ao vivo via MCP `tauri`: clicar num ponto ~100px do topo da
coluna (2h05 brutos) criou o evento em 02:00 (snap de 15min) com 1h de
duração padrão; arrastar esse mesmo evento 240px pra baixo (5h) moveu
pra 07:00 preservando a duração de 1h exatamente (altura do bloco
inalterada, 48px).

Decisão de implementação: o handler de `onmouseup` da COLUNA usa
`e.offset_y()`, que só é confiável quando `e.target()` é a própria
coluna — os blocos de horário já chamam `stop_propagation()` no próprio
`onmouseup`, então o handler da coluna nunca recebe um evento cujo alvo
original foi um bloco (evitando o problema de `offsetY` ser relativo ao
alvo errado quando bubbling).

## Resultado

# Ciclo 069 - done

## Resumo

Grade de horas (Semana/Dia) ganha as duas interações que faltavam:
arrastar um bloco de horário verticalmente muda o horário do evento
preservando a duração, e clicar num ponto vazio da grade cria o evento
já com o horário daquele ponto (snap de 15 em 15 min) em vez de sempre
dia inteiro.

## Arquivos modificados

- `ui/src/embed.rs` — `add_entry_timed`, `move_entry_time`
- `ui/src/components/embeds/inline_calendar.rs` — `y_to_snapped_minutes`,
  onclick da coluna cria evento com horário, onmouseup da coluna
  reagenda horário/dia de um bloco arrastado

## Testes

`cargo test --lib`: 49 passaram (4 novos: `add_entry_timed`,
`move_entry_time` preservando duração no mesmo dia e entre dias,
`move_entry_time` sem inventar horário pra evento de dia inteiro).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

- Clique a ~100px do topo da coluna → evento criado às 02:00 (snap de
  15min) com 1h de duração padrão
- Arrastar esse evento 240px pra baixo → moveu pra 07:00, duração
  preservada exatamente (altura do bloco inalterada)

## Notas

`e.offset_y()` no handler de drop da coluna só é confiável quando
`e.target()` é a própria coluna — os blocos de horário já chamam
`stop_propagation()` no próprio `onmouseup`, evitando que o handler da
coluna receba um evento cujo alvo original foi um bloco (o que faria
`offsetY` vir relativo ao alvo errado).
