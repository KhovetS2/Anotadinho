---
title: Ciclo 125 — Navegacao por teclado nos menus dropdown
type: ciclo
ciclo: "125"
status: concluida
date: 2026-08-09
prioridade: media
depende_de: ["123"]
tags:
- ciclo
---

# Ciclo 125 — Navegacao por teclado nos menus dropdown

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Navegação por teclado nos menus dropdown (⚙, git status, ⋯ do editor)

## Objetivo

Auditoria encontrou 3 menus dropdown próprios (não usam `Modal`, são
`<div>` popover com fechar-ao-clicar-fora + Escape já implementados):
menu "⚙" do app (`header_bar.rs:233-298`), popover de git status
(`header_bar.rs:198-225`), e menu "⋯" do editor (`editor.rs:1642-1677`).
Todos já fecham com Escape/clique fora, mas nenhum foca o primeiro
item ao abrir nem tem navegação por seta — usuário precisa saber Tab
às cegas pra alcançar os itens.

## Critérios de aceite

- [x] Os 3 menus (⚙, git status, ⋯ do editor) focam automaticamente o
      primeiro item (`<button>`) assim que abrem — via
      `crate::menu_keyboard::focus_first_item`, chamado no mesmo
      `use_effect_with` que já cuidava de clique-fora/Escape
- [x] Seta pra baixo/cima move entre os itens do menu (wrap-around) —
      `crate::menu_keyboard::move_item_focus`; Enter ativa o item
      focado (nativo do `<button>`, não precisou de código novo)
- [x] Escape e clique-fora continuam fechando (sem regressão)
- [x] `cd ui && cargo test --lib`, `trunk build`,
      `cargo build --manifest-path src-tauri/Cargo.toml` passam
- [x] Validação ao vivo via MCP `tauri`: os 3 menus testados —
      auto-foco no primeiro item confirmado nos 3, `ArrowDown`
      movendo pro próximo item confirmado nos 3, wrap-around
      confirmado no popover de git (Pull → Commit+Push → Pull de
      novo), Escape fechando confirmado nos 3

## Comandos de validação

```bash
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Extrair um componente `DropdownMenu` genérico reaproveitável pelos
  3 — decisão tomada: NÃO extrair o componente inteiro (os 3 têm JSX
  diferente demais — botões de ação extras, separador, lista
  condicional). Em vez disso, extraiu-se só a PARTE realmente
  idêntica (focar primeiro item / mover foco entre itens) pra um
  módulo `ui/src/menu_keyboard.rs` compartilhado — meio-termo entre
  "zero abstração" e "componente genérico", evita duplicar a lógica
  de DOM/foco 3 vezes sem forçar os 3 menus a caber num molde único
- Atalho de teclado dedicado pra ABRIR cada menu (ex: uma tecla só pra
  abrir o menu ⚙) — os botões que abrem já são focáveis/Tab-áveis
  (ciclo 123 já dá o indicador visual); atalho dedicado é ciclo
  futuro se pedirem

## Notas

Reaproveita a técnica já validada em `command_palette.rs` — não é
território novo, é replicar um padrão que já funciona bem em três
lugares que ainda não o usam.

**Armadilha real encontrada e corrigida durante a implementação**: os
3 menus usam um `NodeRef` no WRAPPER (que contém tanto o botão que
abre/fecha quanto o conteúdo do menu). Se `focus_first_item` tivesse
usado esse mesmo ref, acharia o PRÓPRIO botão de abrir (estruturalmente
primeiro no DOM), não o primeiro item de verdade — bug sutil que só
apareceria em teste ao vivo. Corrigido usando um SEGUNDO `NodeRef`
específico pro conteúdo do menu (`*_content_ref`), mantendo o ref do
wrapper só pra detecção de clique-fora. Validado explicitamente: o
popover de git foca "Pull" (não o indicador "⎇ N" que abre o popover).

## Resultado

# Ciclo 125 - done

## Resumo

Os 3 menus dropdown próprios do app (⚙, popover de git status, ⋯ do
editor) ganham foco automático no primeiro item ao abrir e navegação
por seta (↑/↓, wrap-around) — mesmo padrão que a paleta de comandos já
tinha desde o ciclo 091. Escape e clique-fora, que já existiam,
continuam funcionando.

## Arquivos criados/modificados

- `ui/src/menu_keyboard.rs` (novo) — `focus_first_item`,
  `move_item_focus`, compartilhados pelos 3 menus
- `ui/src/lib.rs` — registra o módulo
- `ui/src/components/header_bar.rs` — menu ⚙ e popover de git ganham
  ref de conteúdo separado + navegação por seta
- `ui/src/components/editor.rs` — menu ⋯ ganha o mesmo tratamento

## Testes

`cd ui && cargo test --lib`: 79. `trunk build` +
`cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: os 3 menus testados individualmente
— auto-foco no primeiro item real (não no botão que abre o menu),
`ArrowDown` movendo corretamente, wrap-around confirmado no popover de
git, Escape fechando em todos.

## Notas

Armadilha real encontrada e corrigida: usar o ref do WRAPPER (que
inclui o botão de abrir) pro auto-foco acharia o próprio botão de
abrir, não o primeiro item — corrigido com um segundo `NodeRef`
específico pro conteúdo do menu. Detalhes no arquivo de task.

Próximo: grafo navegável por teclado (126).
