---
title: "Ciclo 184 — Atalho `n`: funciona sobre embed e Escape cancela sem deixar lixo"
type: ciclo
ciclo: "184"
status: concluida
date: ""
prioridade: media
depende_de: [181]
tags:
- ciclo
---

# Ciclo 184 — Atalho `n`: funciona sobre embed e Escape cancela sem deixar lixo

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

## Objetivo

Fechar os dois furos que apareceram no uso real do atalho `n` (ciclo 181):

1. Cancelar o menu `/` com `Escape` deixava a barra `/` digitada no texto,
   virando uma linha solta no `.md` na próxima gravação.
2. O atalho não respondia quando o foco estava num controle de **embed** —
   só funcionava com um bloco de texto focado.

## Critérios de aceite

- [x] `Escape` no menu `/` aberto pelo `n` apaga o `/` digitado e não deixa
      linha solta no arquivo salvo.
- [x] `n` com um controle de embed focado insere um segmento de markdown
      logo DEPOIS do embed e abre o menu `/` nele.
- [x] O embed e o texto ao redor sobrevivem ao round-trip de gravação.
- [x] Cenário novo no harness de UI cobrindo os dois casos.

## Validação

- `cargo build --workspace`, `cargo test --workspace`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cd ui && trunk build`
- `node scripts/uitest/run.mjs`

## Não-objetivos

- Edição estruturada por bloco (ciclo 175, adiado).
- Mudar a lista de itens do menu `/`.

## Notas

O motivo do (2) não é o `segmento_do_embed_focado()` e sim ONDE o handler
estava: os controles de um embed ficam FORA de qualquer `contenteditable`,
então a tecla nunca subia até o `onkeydown` do editor. O tratamento desse
caso desceu pro contêiner `.editor__wysiwyg-segments`, que é ancestral tanto
dos blocos de texto quanto dos embeds.

Também ficou de fora do escopo, mas corrigido junto: o cenário 180 do harness
conferia `window_is_maximized` no mesmo tick do `window_toggle_maximize` e
falhava de forma intermitente — o comando volta quando PEDE ao gerenciador de
janelas, não quando o estado muda. Virou polling curto.

## Resultado

# 184 — Atalho `n` sobre embed e cancelamento limpo

## O que mudou

- `ui/src/components/editor.rs`
  - Ramo `Escape` do menu `/`: agora apaga o contexto do `/` digitado
    (`delete_slash_context_and_collapse`), recompõe o markdown a partir do DOM
    e marca como editado, antes de fechar o menu.
  - Novo `on_segments_keydown` no contêiner `.editor__wysiwyg-segments`, que
    trata `n` quando o foco está num embed. Antes esse tratamento estava no
    `onkeydown` do `contenteditable` e nunca era alcançado: os controles do
    embed vivem fora dele.
  - Helpers `segmento_do_embed_focado()` (lê o índice de
    `data-nav-group="embed-<i>"`) e `inserir_segmento_e_abrir_menu()` (insere
    `DocSegment::Markdown(BLANK_SEGMENT)`, espera o re-render, foca o segmento
    novo e digita o `/`).
- `scripts/uitest/cenarios.mjs`
  - Cenário novo do 184 cobrindo os dois casos + round-trip do arquivo.
  - Cenário 180 deixou de ser intermitente (polling no estado de maximizado).

## Validação

- `cargo test --workspace`: 293 testes, 0 falhas.
- `cargo build --manifest-path src-tauri/Cargo.toml`: ok.
- `cd ui && trunk build`: ok.
- `node scripts/uitest/run.mjs`: **19/19 em 105.8s**.
- Conferido na janela viva pelo bridge MCP: `n` sobre o callout abriu o menu
  com 2 segmentos no DOM; `Escape` fechou o menu e o arquivo salvo voltou sem
  nenhuma linha `/` solta, com o texto e o embed intactos.
