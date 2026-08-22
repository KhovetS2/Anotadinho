---
title: "Ciclo 185 — Atalho `n`: entrar em digitação encerra a sessão de nav-mode"
type: ciclo
ciclo: "185"
status: concluida
date: ""
prioridade: media
depende_de: [181, 184]
tags:
- ciclo
---

# Ciclo 185 — Atalho `n`: entrar em digitação encerra a sessão de nav-mode

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

## Objetivo

Depois do `n`, o bloco de ORIGEM continuava com o retângulo azul do
nav-mode aceso e o indicador `-- NAV: editor-blocos --` continuava na
tela — mesmo com o cursor já no bloco novo, digitando. As setas também
seguiam navegando entre blocos em vez de andar no texto.

## Critérios de aceite

- [x] `n` num bloco de texto apaga o destaque do nav-mode e derruba a
      sessão (indicador some, setas voltam a andar no texto).
- [x] `n` sobre um embed faz o mesmo.
- [x] Os dois cenários de harness (181 e 184) conferem
      `.nav-mode__item-active` zerado depois do atalho.

## Validação

- `cargo build --workspace`, `cargo test --workspace`
- `cargo build --manifest-path src-tauri/Cargo.toml`
- `cd ui && trunk build`
- `node scripts/uitest/run.mjs`

## Não-objetivos

- Mudar quando o nav-mode COMEÇA (isso é `on_enter_block_nav`, ciclo 174).
- Edição estruturada por bloco (ciclo 175, adiado).

## Notas

São dois estados independentes e os dois precisavam cair: a classe
`nav-mode__item-active` (vive no DOM, gerenciada por
`nav_mode::focus_item`) e o `nav_mode_active`/`nav_stack` do `app.rs`
(que decide para onde vão as setas). O `Enter` num bloco já fazia as
duas coisas desde o ciclo 174; o `n` do 181 nasceu sem elas.

## Resultado

# 185 — `n` encerra a sessão de nav-mode

## O que mudou

- `ui/src/components/editor.rs`
  - Prop nova `on_leave_block_nav`, o inverso do `on_enter_block_nav`.
  - Helper `sair_do_nav_mode()`: limpa `nav-mode__item-active` no DOM e
    emite o callback que derruba o estado no `app.rs`.
  - Os dois ramos do `n` (bloco de texto e embed) chamam esse helper
    antes de abrir o menu `/`.
- `ui/src/components/page_view.rs`: repassa a prop.
- `ui/src/app.rs`: `on_leave_block_nav` zera `nav_mode_active` e `nav_stack`.
- `scripts/uitest/cenarios.mjs`: os cenários 181 e 184 passaram a conferir
  que nenhum elemento fica com `.nav-mode__item-active` depois do atalho.

## Validação

- `cargo test --workspace`: 10 suítes, 0 falhas.
- `cargo build --manifest-path src-tauri/Cargo.toml`: ok.
- `cd ui && trunk build`: ok.
- `node scripts/uitest/run.mjs`: **19/19 em 106.4s**.
- Na janela viva, pelo bridge: antes do `n`, 1 item com destaque; depois,
  0 itens, menu `/` aberto, nenhum indicador `NAV:` na tela e nenhum
  destaque de região.
