---
id: "165"
titulo: "Nav-mode alcança os embeds inline"
status: done
criado: 2026-08-20
autor: humano
prioridade: alta
depende_de: []
estima_min: 120
agente_alvo: claude-opus-5
---

# Nav-mode alcança os embeds inline

## Objetivo

Pedido do usuário: poder operar o app inteiro só pelo teclado. Hoje dá
pra chegar nos controles de um embed **por Tab** (são `<button>` de
verdade, com Enter/Espaço e foco visível), mas o nav-mode — a navegação
por setas dos ciclos 133-140 — **não alcança embed nenhum**.

Achado ao investigar: pra o motor descer num grupo, o MESMO elemento
precisa de `data-nav-item` + `data-nav-parent` + `data-nav-group`
(ver `nav_mode::items_in_group` e `group_of`). Os embeds novos
(151-156) declaram `data-nav-group` na raiz e
`data-nav-item`/`data-nav-parent` nos controles, mas nada liga essa
raiz ao nível de cima — e o item de topo `editor` é um **delegate**
(`app.rs` l.1168), que foca o contenteditable e sai da frente. Ou seja:
os grupos dos embeds existem e são inalcançáveis. Os 3 embeds antigos
(kanban/calendário/tabela inline) não têm atributo de nav nenhum — o
ciclo 135 cobriu as PÁGINAS tipadas (`kanban.rs`, `calendar.rs`,
`task_table.rs`), não os embeds.

## Critérios de aceite

- [x] Com o cursor no editor, uma tecla leva o foco pro próximo embed
      da página e outra pro anterior (sem mouse, sem Tab às cegas
      atravessando todos os botões do embed anterior)
- [x] Com um embed focado, as setas andam pelos controles dele
      (reaproveitando `nav_mode::items_in_group` sobre os
      `data-nav-item`/`data-nav-parent` que os embeds novos já têm)
- [x] Enter/Espaço ativa o controle focado; Escape devolve o foco pro
      texto do editor, no segmento mais próximo do embed
- [x] O indicador visual do item focado (ciclo 139) aparece nos
      controles do embed igual aparece no resto do app
- [x] Vale pros 9 tipos: os 3 embeds antigos ganharam os atributos que
      nunca tiveram — kanban (12 itens: colunas, excluir coluna, editar
      e excluir card), calendário (5: navegação, hoje, criar evento com
      e sem data) e tabela (5: nova coluna, nova linha, excluir linha).
      Célula a célula da tabela e card a card do kanban ficam pra
      quando o arraste por teclado entrar (task 167)
- [x] Escape dentro do embed NÃO desseleciona a página (depende da
      task 161 estar resolvida, ou de resolver junto)
- [x] Cheatsheet (`cheatsheet_modal.rs`) e `GlobalKeymap` atualizados
      com as teclas novas, customizáveis como as demais (ciclo 105)
- [x] Validação ao vivo (MCP `tauri`): abrir `pages/produto/painel.md`
      e operar callout → actions → query → timeline → columns **sem
      encostar no mouse**

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mover card de kanban / barra de timeline pelas setas — é a task 167
  (aqui é NAVEGAR até o controle, não OPERAR arraste)
- Trocar o delegate do editor por um grupo genérico: o caminho é
  entrar no embed A PARTIR do editor, preservando o comportamento de
  texto que os ciclos 133-140 estabeleceram

## Notas

`cd ui && cargo test --lib`: 26. `cargo test --workspace`: 255.
`trunk build` e `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Achado que mudou o desenho no meio do caminho: **todo embed de um tipo
usava o mesmo `data-nav-group`** (`"embed-callout"` cravado no
componente). No `painel.md`, com 3 consultas, as setas andariam pelos
controles das três de uma vez. O id passou a vir do EDITOR
(`format!("embed-{i}")`, com `i` = índice do segmento) por prop, o que
também dá identidade estável pro salto entre embeds.

A entrada no embed reaproveita o motor inteiro do nav-mode: `Ctrl+.`
foca o primeiro controle e empilha o grupo em `nav_stack`, e daí em
diante setas/Enter/Backspace/Escape já funcionavam desde o ciclo 133.
O que precisou de código novo foi só o salto (`adjacent_embed_group`,
que usa `compare_document_position` pra respeitar onde o cursor está no
texto) e a saída devolvendo o foco pro `contenteditable` em vez de pro
topo do app.

Validação ao vivo (MCP `tauri`): no `painel.md`, `Ctrl+.` entrou no
callout, seta desceu pro título, `Ctrl+.` pulou pro embed de ações,
Enter abriu o prompt de "Nova spec", Escape fechou só o modal, e Escape
de novo devolveu o cursor pro texto. Em `exemplos-embeds.md` o mesmo
percurso passou por kanban → calendário → tabela, com `Ctrl+,`
voltando. O indicador de foco do ciclo 139 aparece nos controles.

O trabalho de atributo já está feito nos 6 embeds novos — o que falta é
o caminho de entrada e o loop de setas dentro do embed. Começar por
`inline_callout` (o mais simples: 3 controles) e só então generalizar.
