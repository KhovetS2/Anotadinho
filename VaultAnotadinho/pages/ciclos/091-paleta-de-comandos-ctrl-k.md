---
title: Ciclo 091 — Paleta de comandos Ctrl+K
type: ciclo
ciclo: "091"
status: concluida
date: 2026-08-07
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 091 — Paleta de comandos Ctrl+K

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Paleta de comandos (Ctrl+K)

## Objetivo

Sexto ciclo do conjunto grande. Navegar pra qualquer página do vault ou
disparar um comando nomeado (nova página, nova pasta, alternar tema,
alternar sidebar, ir pra hoje) sem precisar do mouse — pedido explícito
do usuário junto com vim mode.

## Critérios de aceite

- [x] `ui/src/components/command_palette.rs` novo: lista filtrada
      (comandos + títulos de página), ArrowUp/Down/Enter/Escape, fecha
      ao clicar fora, mesmo padrão visual/mecânico do menu `/`
- [x] `Ctrl+K` (e `Ctrl+P`, como alias) abre a paleta — substitui o
      protótipo cru que já existia em `Ctrl+P` (um `Prompt` com a lista
      inteira de títulos jogada no texto do título do modal, sem busca
      de verdade nem navegação por teclado numa lista real)
      `ui/src/app.rs`
- [x] Ações "Nova página"/"Nova pasta"/"Ir pra Hoje" extraídas em
      callbacks reaproveitáveis (`new_page_action`/`new_folder_action`/
      `today_action`) — usadas tanto pelos atalhos diretos quanto pelos
      comandos da paleta, uma implementação só de cada
      `ui/src/app.rs`
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

- Fuzzy-match de verdade (score por proximidade de caracteres) — usa
  substring case-insensitive, mesmo critério já usado pelo menu `/` e
  pela busca da sidebar; suficiente pro tamanho de vault atual
- Comandos extensíveis por plugin/config — lista fixa em `COMMANDS`,
  hardcoded

## Notas

Removido o `Ctrl+P` cru que já existia (`app.rs`, dumping a lista de
títulos dentro do TEXTO do título de um `PendingDialog::Prompt` — sem
busca, sem lista visual, exigia digitar o título exato) — virou alias
de `Ctrl+K` pra o componente novo.

Achado de metodologia de teste: os atalhos globais de `app.rs` só
disparam se `.app-root` (o `<div tabindex="0">` raiz) estiver com foco
— o mesmo já valia pra `Ctrl+N`/`Ctrl+B` antes deste ciclo, não é
regressão nova. Ao testar via MCP `tauri`, precisei `document.querySelector('.app-root').focus()`
explicitamente antes de simular `Ctrl+K` (o clique inicial na janela não
necessariamente foca esse elemento específico).

Validado ao vivo via MCP `tauri`: `Ctrl+K` abre a paleta com comandos +
páginas; digitar filtra ambos (ex: "kan" → kanban/kanban-projeto; "tema"
→ "Alternar tema" + página "tema-design"); clicar uma página navega e
fecha a paleta; clicar "Alternar tema" executa o toggle e fecha; Escape
fecha sem executar nada.

## Resultado

# Ciclo 091 - done

## Resumo

Sexto ciclo do conjunto grande. Paleta de comandos (Ctrl+K/Ctrl+P):
navegar pra qualquer página ou disparar um comando nomeado sem mouse.

## Arquivos criados/modificados

- `ui/src/components/command_palette.rs` — novo
- `ui/src/components/mod.rs` — registra o módulo
- `ui/src/app.rs` — `palette_open`, ações extraídas
  (`new_page_action`/`new_folder_action`/`today_action`), remove o
  protótipo cru de `Ctrl+P`, renderiza `<CommandPalette>`
- `ui/src/styles/components.css` — `.command-palette*`

## Testes

Sem testes novos (componente puramente de UI/composição, validado ao
vivo). `cargo test --workspace`: 48. `cd ui && cargo test --lib`: 66.

`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

## Validação ao vivo (MCP tauri)

Ctrl+K abre, filtra comandos+páginas, clique navega/executa e fecha,
Escape fecha sem efeito. Detalhes no arquivo de task.

## Notas

Próximo: vim mode real + atalhos customizáveis (pedido junto com a
paleta na mesma mensagem do usuário).
