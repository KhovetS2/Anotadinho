---
id: "070"
titulo: "TimePicker customizado para eventos do calendario"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: ["069"]
estima_min: 45
agente_alvo: claude-sonnet
---

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
