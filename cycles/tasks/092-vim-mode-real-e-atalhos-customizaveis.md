---
id: "092"
titulo: "Vim mode real e atalhos customizaveis"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: []
estima_min: 150
agente_alvo: claude-sonnet
---

# Vim mode real e atalhos customizáveis

## Objetivo

Sétimo ciclo do conjunto grande. Substitui o flag `vim_mode` morto
(nunca lido em lugar nenhum, nunca setado como `true`) por um motor
modal Normal/Insert de verdade, com mapa de teclas configurável.

## Critérios de aceite

- [x] `ui/src/state.rs`: `VimKeymap` (uma tecla por ação, valores
      padrão = vim clássico), persistido no localStorage; toggle
      `vim_mode_enabled` persistido
- [x] `ui/src/components/editor.rs`: modo Normal (todo tecla é comando,
      nada digita) vs Insert (digitação normal de sempre) — motions
      `h/j/k/l/w/b/0/$/gg/G` via `Selection.modify` (API nativa do
      browser, mesma usada por Ctrl+seta — evita reimplementar
      navegação de palavra/linha na mão); `i/a/o/O` entram em Insert;
      `x` apaga caractere (`execCommand forwardDelete`); `dd`/`yy`
      (tecla configurada 2x seguidas) apaga/copia a linha (bloco
      `li`/`p`/heading/blockquote) pro registrador; `p` cola; `u`
      desfaz (`execCommand undo`, undo nativo do contenteditable)
- [x] `Esc` (só em Insert) volta pra Normal, com `stop_propagation`
      (mesma cautela do menu `/`/wikilink — sem isso vazava pro atalho
      global que desseleciona a página inteira)
- [x] `ui/src/components/vim_settings_modal.rs` novo: tela de
      configuração — clica numa tecla, pressiona a nova, Esc cancela
- [x] Toggle "Vim mode" + item "Atalhos do Vim mode..." no menu ⚙ do
      header (`ui/src/components/header_bar.rs`)
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Macros, registradores nomeados, modo Visual Block, `:comandos`
  estilo ex-command — escopo definido desde o plano original, v1 é só
  motions básicas + edição de linha
- `e` (fim de palavra) distinto de `w` (próxima palavra) —
  `Selection.modify` não tem uma granularidade nativa "fim da palavra
  atual" separada de "próxima palavra"; `e` não foi mapeado nesta v1
  (ver Notas)
- Motions horizontais (`h`/`l`/`a`) clampadas à linha atual — usar
  `Selection.modify` significa que mover no fim de uma linha pode
  cruzar pra o início da próxima (fluxo natural de texto do
  contenteditable), diferente do vim de verdade que trava na linha
  atual

## Notas

Descoberta de arquitetura que economizou MUITO trabalho:
`web_sys::Selection::modify(alter, direction, granularity)` (feature
`Selection`, já habilitada) expõe a mesma API que o browser usa
internamente pra Ctrl+seta/Shift+seta — granularidades
`character`/`word`/`line`/`lineboundary`/`documentboundary` cobrem
`hjkl`/`w`/`b`/`0`/`$`/`gg`/`G` sem precisar andar no DOM na mão feito
`find_slash_context` faz pra outras features. `dd`/`yy`/`p`/`o`/`O`
ainda precisam de manipulação de DOM direta (`vim_current_block`
via `closest("li, p, h1..h6, blockquote")`), mas puderam reaproveitar
os padrões já estabelecidos (`recompute_markdown_from_dom` +
`mark_edited` depois de mutação direta, igual as inserções do menu `/`).

Bug pego e corrigido DURANTE a validação ao vivo (não no primeiro
código escrito): `vim_paste_after`/`vim_open_line` sempre criavam um
`<p>`, então colar dentro de uma lista (`<li>`) produzia um `<p>` solto
no meio dos itens — visualmente quebrava a lista. Fix: `sibling_line_tag`
olha a tag do bloco atual (`li` continua `li`, resto vira `p`).

Validado ao vivo via MCP `tauri`: ativar vim mode pelo menu ⚙; `dd` com
cursor num item de lista remove exatamente aquele item; `yy` num item +
`p` noutro cola como novo `<li>` (não quebra a lista, confirmando o fix
acima); `a` entra em Insert (digitação normal confirmada via
`execCommand insertText`); `Esc` volta pra Normal sem desselecionar a
página; `Ctrl+S` continua funcionando em modo Normal; tela de atalhos
abre, clicar "Direita" + pressionar `;` reatribui e persiste em
`localStorage['anotadinho.vim_keymap']`. Mudanças de teste (conteúdo da
página `teste`, vim mode ativado, keymap customizado) revertidas antes
de fechar o ciclo.
