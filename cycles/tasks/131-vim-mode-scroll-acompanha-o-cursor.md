---
id: "131"
titulo: "Vim mode: scroll acompanha o cursor"
status: done
criado: 2026-08-09
autor: humano
prioridade: media
depende_de: []
estima_min: 30
agente_alvo: claude-sonnet
---

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
