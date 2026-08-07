---
id: "067"
titulo: "Visoes semana e dia no calendario e animacao de drag"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["066"]
estima_min: 150
agente_alvo: claude-sonnet
---

# Visões Semana/Dia no calendário + animação de drag-and-drop

## Objetivo

Dois pontos trazidos pelo usuário depois de validar a grade mensal do
ciclo anterior. Primeiro: faltavam as visões Semana e Dia (grade de
horas, referência visual do Google Calendar). Segundo: o drag-and-drop
(kanban e calendário) não tinha retorno visual — só uma opacidade
reduzida no item, sem "seguir o cursor" nem indicar onde vai cair. Este
ciclo entrega os dois. Arrastar verticalmente na grade de horas pra mudar
o HORÁRIO fica pro próximo ciclo (confirmado com o usuário).

## Critérios de aceite

- [x] Ghost seguindo o cursor durante o arraste (kanban + calendário,
      todas as visões) via listener global de `mousemove`, `position: fixed`
      + `pointer-events: none`
- [x] Indicador de destino: linha de inserção antes do card sob o cursor
      (kanban) + destaque de fundo mais forte na coluna/dia sob o cursor
      (kanban e calendário)
- [x] `CalendarEntry.start_time`/`end_time` (retrocompatível — sem eles
      continua sendo evento de dia inteiro)
- [x] `EventDetailModal`: toggle "Horário específico" revela
      `input[type=time]` nativo pra início/fim
- [x] Seletor de visão Mês/Semana/Dia no cabeçalho do calendário
- [x] Grade de horas (Semana: 7 colunas, Dia: 1 coluna) — 0h-23h,
      scroll inicial ~2h antes do horário atual, linha vermelha do
      horário atual (só no dia que é hoje), eventos com horário
      posicionados por `top`/`height`, faixa de dia inteiro/intervalo no
      topo reaproveitando o mesmo algoritmo de lanes da grade mensal
- [x] Arrastar entre dias na visão Semana funciona igual à visão Mês

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Arrastar verticalmente na grade de horas pra mudar o horário — fica pro
  próximo ciclo (já no pipeline), junto com clicar num horário específico
  da grade pra criar evento já com aquele horário
- `TimePicker` customizado — usa `input[type=time]` nativo (sem o bug de
  popup do date, é um spinner inline)
- Redimensionar duração arrastando a borda do bloco
- Fuso horário
- Visão Semana/Dia na página de calendário inteira (`type: calendar` no
  frontmatter) — só o embed inline ganhou as visões novas

## Notas

Bug real encontrado e corrigido durante a validação ao vivo: eventos com
horário apareciam DUPLICADOS — uma vez na faixa de dia inteiro (porque
`pack_days`, reaproveitado do algoritmo de barras da grade mensal, não
filtrava por horário) e outra vez como bloco posicionado na grade de
horas. Corrigido com um parâmetro `exclude_timed` em `pack_days`: `false`
na visão Mês (sem grade de horas, todo evento vira barra ali mesmo),
`true` na faixa de dia inteiro de Semana/Dia (evento com horário já tem
seu bloco próprio).

Confirmado ao vivo: visão Dia bate visualmente com a referência do Google
Calendar (cabeçalho "7 de agosto de 2026", círculo azul no dia, linha
vermelha do horário atual na posição matematicamente correta — conferido
comparando `now_minutes()` com o `top` renderizado). Arrastar um card do
kanban mostra o ghost seguindo o cursor e a linha de inserção antes do
card certo; soltar move o card exatamente pra onde a linha indicava.
