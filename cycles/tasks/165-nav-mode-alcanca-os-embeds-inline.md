---
id: "165"
titulo: "Nav-mode alcança os embeds inline"
status: pending
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

- [ ] Com o cursor no editor, uma tecla leva o foco pro próximo embed
      da página e outra pro anterior (sem mouse, sem Tab às cegas
      atravessando todos os botões do embed anterior)
- [ ] Com um embed focado, as setas andam pelos controles dele
      (reaproveitando `nav_mode::items_in_group` sobre os
      `data-nav-item`/`data-nav-parent` que os embeds novos já têm)
- [ ] Enter/Espaço ativa o controle focado; Escape devolve o foco pro
      texto do editor, no segmento mais próximo do embed
- [ ] O indicador visual do item focado (ciclo 139) aparece nos
      controles do embed igual aparece no resto do app
- [ ] Vale pros 9 tipos: os 3 embeds antigos (`inline_kanban`,
      `inline_calendar`, `inline_table`) ganham os atributos de nav que
      nunca tiveram
- [ ] Escape dentro do embed NÃO desseleciona a página (depende da
      task 161 estar resolvida, ou de resolver junto)
- [ ] Cheatsheet (`cheatsheet_modal.rs`) e `GlobalKeymap` atualizados
      com as teclas novas, customizáveis como as demais (ciclo 105)
- [ ] Validação ao vivo (MCP `tauri`): abrir `pages/produto/painel.md`
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

O trabalho de atributo já está feito nos 6 embeds novos — o que falta é
o caminho de entrada e o loop de setas dentro do embed. Começar por
`inline_callout` (o mais simples: 3 controles) e só então generalizar.
