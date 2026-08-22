---
title: Ciclo 071 — Redimensionar duracao do evento arrastando a borda do bloco
type: ciclo
ciclo: "071"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: ["070"]
tags:
- ciclo
---

# Ciclo 071 — Redimensionar duracao do evento arrastando a borda do bloco

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Redimensionar duração do evento arrastando a borda do bloco

## Objetivo

Quarto item do backlog dos ciclos 066/067: na grade de horas (Semana/Dia),
permitir arrastar a borda de cima ou de baixo de um bloco com horário pra
mudar `start_time`/`end_time` diretamente, sem precisar abrir o modal.
Também corrige um bug real encontrado no meio do ciclo: o
`onmousedown` do bloco com horário (`inline_calendar.rs`) não tinha
`e.prevent_default()`, diferente de todos os outros pontos de início de
drag (card do kanban, barra do mês, barra de dia inteiro) — o mesmo bug
de seleção de texto/drag nativo do ciclo 068 podia voltar a acontecer
especificamente ao arrastar um bloco com horário.

## Critérios de aceite

- [x] `CalendarEmbedData::resize_entry_time(idx, is_start_edge, new_minutes)`
      em `embed.rs` — redimensiona início ou fim, com duração mínima de
      15min (trava no limite em vez de inverter início/fim)
- [x] Alças de redimensionar (`.calendar-grid__resize-handle--top/bottom`)
      no topo/base de cada bloco com horário, `cursor: ns-resize`
- [x] Prévia visual em tempo real enquanto arrasta (bloco cresce/encolhe
      seguindo o cursor, com destaque de contorno) antes de soltar
- [x] `e.prevent_default()` adicionado no `onmousedown` do bloco com
      horário (bug encontrado durante este ciclo, consistente com os
      outros 3 pontos de início de drag que já tinham)
- [x] Testes: início, fim, trava de duração mínima nos dois sentidos, e
      no-op em evento sem horário

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Painel de eventos sem data, visões Semana/Dia na página de calendário
  inteira — ficam pra retomar depois (tasks #58/#59 no tracker)

## Notas

Bug real encontrado e corrigido durante a implementação (não durante
validação ao vivo, dessa vez direto na leitura do código): o commit do
resize inicialmente lia `resize_preview_min` (um `use_state`) dentro do
listener de `mouseup`, mas esse listener é criado UMA VEZ (efeito com
`use_effect_with(*resizing, ...)`, só recria quando o resize começa/termina)
— o clone do handle que ele capturou fica congelado no valor daquele
instante (`None`), então `.set()` chamado depois pelo listener de
`mousemove` (outra instância do mesmo handle, mas lida por uma closure
diferente) nunca era visto pelo `mouseup`. Resultado: o resize nunca
persistia, revertia pro tamanho original ao soltar o mouse — confirmado
ao vivo via MCP antes da correção (screenshot mostrou o bloco voltando a
36px depois do "solto"). Corrigido calculando o minuto final direto da
posição do mouse via `NodeRef` (que sempre reflete o DOM ao vivo, sem
esse problema de handle congelado) dentro do próprio listener de
`mouseup`, em vez de depender do estado de prévia.

Validado ao vivo via MCP `tauri` depois da correção: arrastar a borda de
baixo de "Deploy produção" (14:30–15:15) até ~16:45 e soltar — modal
confirmou `14:30 – 16:45`. Arrastar a borda de cima confirmou o mesmo pro
início. Testado também que mover o bloco inteiro (drag horizontal entre
dias) continua funcionando sem regressão depois de adicionar as alças de
redimensionar como filhos do bloco.

Lição de metodologia de teste: `window.dispatchEvent(...)` não faz
hit-testing por coordenada — só notifica listeners registrados no
`window` (por isso o `mouseup` de resize, que É um listener de window,
funcionou nos testes), mas não aciona handlers `onmouseup` inline de um
elemento específico (como o drop no dia-coluna do calendário). Pra testar
esse tipo de handler é preciso `element.dispatchEvent(...)` no elemento
real sob o cursor (via `document.elementFromPoint`), não `window`.

## Resultado

# Ciclo 071 - done

## Resumo

Redimensionar a duração de um evento com horário arrastando a borda de
cima/baixo do bloco na grade de horas (Semana/Dia). Também corrige um bug
de `prevent_default()` faltando no `onmousedown` do bloco com horário
(mesma classe do bug do ciclo 068).

## Arquivos criados/modificados

- `ui/src/embed.rs` — `CalendarEmbedData::resize_entry_time` + 4 testes
  novos
- `ui/src/components/embeds/inline_calendar.rs` — alças de resize, estado
  `resizing`/`resize_preview_min`, efeito de mousemove/mouseup, prévia
  visual, correção do `prevent_default()` faltando
- `ui/src/styles/main.css` — `.calendar-grid__resize-handle`,
  `.calendar-grid__timed-block--resizing`
- `ui/Cargo.toml` — feature `DomRect` do `web-sys` (necessária pra
  `Element::get_bounding_client_rect()`)

## Testes

`cargo test --lib`: 49 passaram (4 novos: `calendar_resize_entry_time_*`).

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Bug real encontrado e corrigido no meio da implementação: primeira versão
do commit do resize lia um `use_state` (`resize_preview_min`) dentro de
um listener de `mouseup` criado uma única vez no início do resize — esse
handle ficava congelado no valor de quando foi criado (`None`), então o
resize nunca persistia (confirmado ao vivo: bloco voltava pro tamanho
original ao soltar). Corrigido lendo a posição do mouse direto via
`NodeRef` (sempre reflete o DOM atual) dentro do próprio `mouseup`, sem
depender do estado de prévia pra commit.

Depois da correção: arrastar borda de baixo de "Deploy produção"
(14:30–15:15) até a prévia mostrar 16:45 e soltar — modal confirma
`14:30 – 16:45`. Arrastar borda de cima confirma o mesmo pro início.
Mover o bloco inteiro (drag horizontal entre dias) testado sem regressão
depois de adicionar as alças como filhas do bloco.

## Notas

Nenhuma edição de teste vazou pro vault (`git diff --stat VaultAnotadinho/`
vazio antes de commitar).
