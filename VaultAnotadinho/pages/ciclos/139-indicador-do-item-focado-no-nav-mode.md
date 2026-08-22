---
title: Ciclo 139 — Indicador do item focado no nav-mode
type: ciclo
ciclo: "139"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 139 — Indicador do item focado no nav-mode

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Indicador do item focado no nav-mode

## Objetivo

Pedido do usuário: ao entrar no modo de navegação, não dava pra ver
claramente qual item está com foco pra selecionar — o `:focus-visible`
genérico (ciclo 123, só contorno com offset) nem sempre chama atenção
o bastante em itens grandes (ex: o `<header>` inteiro é um item de
nível raiz) ou pode ficar cortado por um ancestral com `overflow`.
Pedido explícito: usar bgcolor diferente ou borda interna em vez de
depender só do contorno.

## Critérios de aceite

- [x] `nav_mode::focus_item` marca o item com a classe
      `nav-mode__item-active` (fundo + `box-shadow` inset) sempre que
      move o foco, removendo a marca do item anterior antes — mesmo
      padrão de "consultar e substituir" já usado pro destaque de
      região (ciclo 133/136)
- [x] Nova `nav_mode::clear_item_highlight()` — limpa a marca quando a
      sessão inteira termina (delegate ou Escape na raiz), chamada no
      mesmo efeito que já limpa o destaque de região
- [x] CSS: `.nav-mode__item-active` com `background-color` +
      `box-shadow: inset` usando `--nav-mode-depth-color` (herda do
      `.nav-mode__region-active` ancestral via cascata normal de
      custom property; item de nível raiz, sem ancestral com a
      variável, cai no fallback azul)
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: item "header" marcado
      corretamente (fundo + box-shadow computados confirmados);
      descer um nível transfere a marca pro novo item e limpa do
      anterior; Escape duplo (sair da sessão) limpa a marca por
      completo; delegate pra sidebar também limpa

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Tratamento especial pros nós do grafo (SVG `<g>`, ciclo 126) —
  `background-color`/`box-shadow` não têm efeito em elementos SVG
  (mesma limitação já documentada pro `:focus-visible` deles, que usa
  `stroke` num `<circle>` em vez de `outline`). Não é regressão nova
  (o `:focus-visible` de contorno já não funcionava bem lá antes
  também) e o nav-mode só toca um nó do grafo de passagem, no
  delegate pro editor — não fica ali navegando entre nós ainda (isso
  precisaria de um grupo de verdade, fora de escopo aqui)

## Notas

Reaproveita o mesmo mecanismo do destaque de região (ciclo 133/136,
`--nav-mode-depth-color`) em vez de inventar um esquema de cor
separado — a cor do item focado automaticamente bate com a cor do
"wrapper" em que ele está, por herança normal de CSS custom property
(o item é descendente do container que tem `.nav-mode__region-active`
com a variável setada).

## Resultado

# Ciclo 139 - done

## Resumo

Pedido do usuário: o item focado no nav-mode não era visível o
bastante (só o contorno genérico de `:focus-visible`). Adiciona um
indicador dedicado (fundo + borda interna) via `.nav-mode__item-active`,
gerenciado por `nav_mode::focus_item`/`clear_item_highlight`, herdando
a cor de profundidade do `.nav-mode__region-active` ancestral.

## Arquivos criados/modificados

- `ui/src/nav_mode.rs` — `focus_item` marca/desmarca a classe,
  `clear_item_highlight` novo
- `ui/src/app.rs` — chama `clear_item_highlight` quando a sessão
  termina
- `ui/src/styles/main.css` — `.nav-mode__item-active`

## Testes

`cd ui && cargo test --lib`: 84. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: fundo + box-shadow confirmados no
item "header"; marca transferida corretamente ao descer nível; limpa
por completo ao sair da sessão (Escape duplo) e ao delegar (sidebar).

## Notas

Reaproveita o mecanismo de cor por profundidade já existente
(`--nav-mode-depth-color`, ciclos 133/136) via herança de CSS custom
property, sem precisar de lógica de cor nova.
