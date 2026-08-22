---
title: Ciclo 068 — Corrige selecao de texto durante drag no kanban e calendario
type: ciclo
ciclo: "068"
status: concluida
date: 2026-08-07
prioridade: alta
depende_de: ["067"]
tags:
- ciclo
---

# Ciclo 068 — Corrige selecao de texto durante drag no kanban e calendario

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Corrige seleção de texto durante drag no kanban e calendário

## Objetivo

Bug relatado pelo usuário: arrastando rápido (principalmente da direita
pra esquerda), o `mousedown` + mover o mouse selecionava o texto por
baixo do cursor, e o navegador tentava iniciar um drag nativo de conteúdo
por cima do nosso drag por mouse próprio — visualmente parecia "o ghost
de um container" em vez do card/evento sendo arrastado. Causa
identificada pelo próprio usuário.

## Critérios de aceite

- [x] `e.prevent_default()` nos handlers de `onmousedown` que iniciam o
      drag (kanban: card; calendário: barra de evento e bloco de horário)
- [x] `user-select: none` em `.kanban__card`, `.kanban__board`,
      `.calendar-grid__weeks` e `.calendar-grid__hour-scroll` (escopo
      restrito às áreas de drag — não no `.calendar-grid` inteiro, que
      também contém o `EventDetailModal`, que precisa continuar editável)
- [x] Validado ao vivo: arraste rápido de direita pra esquerda no kanban
      e no calendário, `window.getSelection().toString()` vazio durante
      todo o arraste, sem nenhum artefato visual estranho

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum — ciclo pequeno e focado só nesse bug

## Notas

Validação ao vivo simulou o arraste com uma sequência real de eventos
`mousemove` incrementais (8 passos, não só início+fim) pra reproduzir a
velocidade real do bug — confirmado `window.getSelection().toString()`
vazio do início ao fim do gesto, em ambos os componentes, e o card/evento
efetivamente moveu pra posição solta corretamente.

## Resultado

# Ciclo 068 - done

## Resumo

`e.prevent_default()` nos handlers de `onmousedown` que iniciam um drag
(kanban e calendário) + `user-select: none` escopado nas áreas de
arraste — corrige o bug onde arrastar rápido selecionava o texto de
fundo e o navegador tentava um drag nativo de conteúdo por cima do drag
por mouse próprio do app.

## Arquivos modificados

- `ui/src/components/embeds/inline_kanban.rs`
- `ui/src/components/embeds/inline_calendar.rs`
- `ui/src/styles/main.css`

## Testes

`cargo test --lib`: 41 passaram (nenhum teste novo — bug de interação de
mouse, não de lógica de dados).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Simulado arraste rápido de direita pra esquerda no kanban (card entre
colunas) e no calendário (evento entre dias) com sequência real de 8
`mousemove` incrementais. `window.getSelection().toString()` vazio do
início ao fim em ambos, sem nenhum ghost estranho, e o item moveu
corretamente pra onde foi solto.
