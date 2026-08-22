---
title: "Ciclo 131 — Vim mode: scroll acompanha o cursor"
type: ciclo
ciclo: "131"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 131 — Vim mode: scroll acompanha o cursor

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Vim mode: scroll acompanha o cursor

## Objetivo

Bug reportado: no vim mode, navegar pra baixo (`j`, ou qualquer motion
via `vim_move`) até uma linha fora da área visível do editor não rola
o container — o cursor "some" da tela. `Selection.modify` (usado por
`vim_move`) move o caret mas, diferente do comportamento nativo de
seta do navegador numa página comum, não rola a viewport sozinho.

## Critérios de aceite

- [x] Nova função `vim_scroll_caret_into_view` (`editor.rs`) — acha o
      elemento mais próximo do container da seleção atual e chama
      `scroll_into_view_with_scroll_into_view_options` com
      `block: Nearest` (mesmo critério já usado pra manter o item
      destacado visível na sidebar/paleta — rola o mínimo, sem
      centralizar à toa a cada tecla)
- [x] Chamada automaticamente ao fim de `vim_move` — cobre TODAS as
      motions que passam por ali (h/j/k/l, word forward/backward,
      line start/end, doc start/end) sem precisar repetir a chamada
      em cada `if` do dispatcher de teclas
- [x] `cd ui && cargo test --lib` passa
- [x] Validação ao vivo via MCP `tauri`: página de teste com 60
      parágrafos curtos (bastante scroll), vim mode ativado, cursor no
      primeiro parágrafo, 24 `j` seguidos — `scrollTop` foi de 0 pra
      371 (antes do fix, ficaria travado em 0) e o parágrafo com o
      caret ficou exatamente na borda inferior visível do container
      (colado, não centralizado — confirma `Nearest` funcionando).
      Repetido na direção inversa (`k`, de baixo pra cima) com o mesmo
      resultado simétrico.

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
```

## Não-objetivos

- Aplicar o mesmo scroll-into-view em `o`/`O` (abrir linha)/`p`
  (colar)/`dd` (apagar linha) — essas ações raramente saem da área
  visível numa única operação (diferente de navegar várias linhas
  seguidas) e não foram reportadas; se virar problema real, é um fix
  igualmente pequeno de adicionar depois

## Notas

Fix pequeno e cirúrgico: um helper novo + uma chamada no fim de
`vim_move`, sem tocar em mais nada. `Selection.modify` é a mesma API
usada por Ctrl+seta/Shift+seta nativos do navegador — o navegador
rola sozinho quando o USUÁRIO aperta seta numa página comum, mas isso
é comportamento do NAVEGADOR reagindo à tecla física, não da API
`Selection.modify` em si; como `vim_move` chama a API diretamente
(sem passar pela tecla física de seta), o navegador nunca teve a
chance de rolar sozinho — daí precisar do `scrollIntoView` manual.

## Resultado

# Ciclo 131 - done

## Resumo

No vim mode, `Selection.modify` (usado por `vim_move` pras motions
h/j/k/l/w/b/0/$/gg/G) move o caret mas não rola o container sozinho —
navegar pra baixo até sair da área visível deixava o cursor invisível.
Corrigido com um `scrollIntoView(block: "nearest")` chamado
automaticamente ao fim de `vim_move`.

## Arquivos criados/modificados

- `ui/src/components/editor.rs` — `vim_scroll_caret_into_view` (novo),
  chamada no fim de `vim_move`

## Testes

`cd ui && cargo test --lib`: 80. `cargo test --workspace`: 117.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: página de teste com 60 parágrafos,
vim mode ativado, 24 `j` seguidos moveu `scrollTop` de 0 pra 371 (antes
ficaria travado em 0), cursor confirmado exatamente na borda inferior
visível (não centralizado — `Nearest` funcionando certo). Mesmo
resultado simétrico testado com `k` (pra cima).

## Notas

Fix cirúrgico — um helper + uma chamada, sem tocar em mais nada.
Página de teste (`_teste-scroll-vim.md`) e um journal de hoje criado
sem querer durante a validação foram removidos do vault depois.
