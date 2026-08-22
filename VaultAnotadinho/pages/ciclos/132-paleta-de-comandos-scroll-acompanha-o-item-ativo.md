---
title: "Ciclo 132 — Paleta de comandos: scroll acompanha o item ativo"
type: ciclo
ciclo: "132"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 132 — Paleta de comandos: scroll acompanha o item ativo

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Paleta de comandos: scroll acompanha o item ativo

## Objetivo

Bug reportado: mesmo comportamento do ciclo 131 (vim mode), agora na
paleta de comandos (Ctrl+K) — navegar a lista com ArrowUp/Down até um
item fora da área visível não rola o container, o item destacado
"some" da tela. Diferente do vim mode (que usa `Selection.modify`
direto no DOM), aqui o destaque é controlado por um `idx: UseState`
que só troca a classe CSS do item — nunca existiu nenhum
`scrollIntoView`, mesmo bug de raiz que a sidebar (ciclo 106) e o menu
`/` do editor (ciclo 073/082) já corrigiram no passado.

## Critérios de aceite

- [x] `command_palette.rs` ganha `active_item_ref: NodeRef`,
      reatribuído a cada render pro item atualmente destacado (mesmo
      padrão de `nav_item_ref` da sidebar — ciclo 106), com `ref=` nos
      3 tipos de item (`Command`/`Page`/`ContentResult`)
- [x] `use_effect_with(*idx, ...)` chama
      `scroll_into_view_with_scroll_into_view_options` com
      `block: Nearest` sempre que `idx` muda
- [x] `cd ui && cargo test --lib` passa
- [x] Validação ao vivo via MCP `tauri`: paleta aberta sem filtro (37
      itens, lista com scroll de 1303px de altura vs 329px visíveis),
      20 ArrowDown seguidos moveu `scrollTop` de 0 pra 480 com o item
      ativo confirmado colado na borda inferior visível; ArrowUp de
      volta até o topo (incluindo o wrap-around de voltar do primeiro
      pro último item) também rolou corretamente, item sempre
      confirmado visível

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Nenhum — fix pequeno e isolado, mesmo padrão já usado em 2 outros
  lugares do app

## Notas

Mesma causa raiz que o ciclo 131, mecanismo de fix igual (`Nearest`
scroll-into-view), mas aplicado de um jeito ligeiramente diferente
porque aqui não existe um `Selection` do DOM pra ancorar — o "item
ativo" é só um índice (`idx: UseState<usize>`) que decide qual `<div>`
ganha a classe `--active`. Solução: um `NodeRef` só, reatribuído
dinamicamente ao item ativo a cada render (mesmíssimo padrão já usado
por `nav_item_ref` na sidebar desde o ciclo 106) — não precisou de
nada novo, só replicar um padrão já validado no projeto.

**Achado de teste, não bug**: ao validar, os primeiros disparos de
`ArrowDown` via `dispatchEvent` num laço síncrono não pareciam
surtir efeito (mesma classe ativa antes e depois). Causa: diferente do
`vim_move` (mutação direta e síncrona do DOM), aqui cada `ArrowDown`
dispara `idx.set(...)`, que só re-renderiza no PRÓXIMO microtask do
Yew — disparar 20 eventos sincronamente sem ceder o loop de eventos
faz todos lerem o mesmo `idx` "congelado" da closure capturada no
último render. Resolvido espaçando os disparos com
`await new Promise(r => setTimeout(r, 20))` entre cada um, dentro de
uma IIFE assíncrona — não é um problema do app, só do jeito de
simular teclado rápido via automação.

## Resultado

# Ciclo 132 - done

## Resumo

Mesmo bug de fundo do ciclo 131, agora na paleta de comandos: navegar
com ArrowUp/Down até um item fora da área visível não rolava o
container. Corrigido replicando o padrão já usado pela sidebar (ciclo
106) — um `NodeRef` reatribuído dinamicamente ao item ativo + um
`scrollIntoView(nearest)` disparado a cada mudança de `idx`.

## Arquivos criados/modificados

- `ui/src/components/command_palette.rs` — `active_item_ref`, efeito
  de scroll-into-view, `ref=` nos 3 tipos de item renderizados

## Testes

`cd ui && cargo test --lib`: 80. `cargo test --workspace`: 117.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: 20 ArrowDown numa lista de 37 itens
moveu o scroll de 0 pra 480 com o item ativo sempre visível; ArrowUp de
volta (incluindo wrap-around primeiro↔último item) confirmado
funcionando igual.

## Notas

Dev server (`trunk serve`/`cargo tauri dev`) tinha morrido durante o
build de release anterior (`./scripts/build.sh`) — precisou reiniciar
via `./scripts/dev.sh` antes de validar ao vivo.

Achado de teste (não bug do app) documentado no arquivo de task: laços
síncronos de `dispatchEvent` não compõem corretamente com
`use_state.set()` do Yew (re-render é assíncrono) — precisou espaçar
os disparos com um `setTimeout` entre cada um.
