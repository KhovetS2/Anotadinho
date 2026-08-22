---
title: Ciclo 070 — TimePicker customizado para eventos do calendario
type: ciclo
ciclo: "070"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["069"]
tags:
- ciclo
---

# Ciclo 070 — TimePicker customizado para eventos do calendario

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# `TimePicker` customizado (fecha o ciclo de identidade visual do `DatePicker`)

## Objetivo

Terceiro item do backlog dos ciclos 066/067: substituir o
`input[type=time]` nativo do `EventDetailModal` por um popover próprio,
mesmo padrão visual do `DatePicker` já existente. Diferente do date, o
horário nativo não tinha o bug de popup-não-fecha (é um spinner inline,
não overlay separado), mas ainda destoava do resto da UI.

## Critérios de aceite

- [x] `ui/src/components/time_picker.rs` (novo): popover com lista
      rolável de horários de 15 em 15 minutos, já aberto rolado até o
      horário selecionado (ou o mais próximo de agora, se nenhum)
- [x] Fecha em outside-click/Escape — mesmo padrão dos outros dropdowns
- [x] `EventDetailModal` troca os dois `input[type=time]` (início/fim)
      por chips clicáveis que abrem o `TimePicker`

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redimensionar duração arrastando a borda do bloco — próximo ciclo
- Painel de eventos sem data, visões Semana/Dia na página de calendário
  inteira — ambos ficam pra retomar depois (pausados a pedido do
  usuário, não descartados)

## Notas

Validado ao vivo via MCP `tauri`: popover abre já rolado pro horário
atual do evento (14:30), escolher um horário da lista (16:00) atualiza o
campo e fecha o popover sozinho.

Ciclo interrompido a pedido do usuário depois deste item — os itens 4
(redimensionar), 5 (painel de eventos sem data) e 6 (visões Semana/Dia na
página de calendário inteira) do backlog dos ciclos 066/067 ficam
pendentes pra retomar em outro momento (tasks #57/#58/#59 já criadas no
tracker, deixadas como `pending`).

## Resultado

# Ciclo 070 - done

## Resumo

`TimePicker` (novo componente) substitui o `input[type=time]` nativo no
`EventDetailModal` — popover com lista rolável de horários de 15 em 15
min, mesmo padrão visual/de interação do `DatePicker` (fecha em
outside-click/Escape, abre já posicionado no horário atual).

## Arquivos criados/modificados

- `ui/src/components/time_picker.rs` (novo)
- `ui/src/components/mod.rs` — registro do módulo
- `ui/src/components/embeds/event_detail_modal.rs` — chips + popover em
  vez dos `input[type=time]`
- `ui/src/styles/main.css`, `ui/src/styles/components.css` — CSS do
  `.time-picker` e `.event-modal__time-chip`

## Testes

`cargo test --lib`: 45 passaram (sem testes novos — componente de UI
pura, sem lógica de dados nova).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Popover abre com a lista já rolada até o horário atual do campo (14:30);
clicar num horário (16:00) atualiza o campo e fecha o popover sozinho.

## Notas

Ciclo interrompido a pedido do usuário depois deste item — os itens 4
(redimensionar duração), 5 (painel de eventos sem data) e 6 (visões
Semana/Dia na página de calendário inteira) ficam pendentes pra retomar
depois.
