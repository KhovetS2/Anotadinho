---
id: "139"
titulo: "Indicador do item focado no nav-mode"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: []
estima_min: 45
agente_alvo: claude-sonnet
---

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
