---
id: "147"
titulo: "Migra ícones dos embeds (kanban, tabela, grafo) para SVG"
status: done
criado: 2026-08-14
autor: humano
prioridade: media
depende_de: ["144"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Migra ícones dos embeds (kanban, tabela, grafo) para SVG

## Objetivo

Último ciclo da série — troca os emoji/glifos de ícone dos embeds
(`inline_kanban.rs`, `inline_table.rs`, `card_detail_modal.rs`,
`column_settings_modal.rs`, `graph_view.rs`) pelo componente `Icon`
(ciclo 144), fechando o pedido do usuário de trocar todos os ícones de
fonte por SVG.

## Critérios de aceite

- [x] `inline_kanban.rs`: `✕` (excluir coluna/card) → `x`; `✎`
      (editar título) → `edit`; `☑ {done}/{total}` (checklist) →
      `check-square` + texto; `📅 {due}` → `calendar` + texto;
      `💬 {n}` → `message-circle` + texto; `📎 {n}` → `paperclip` +
      texto
- [x] `inline_table.rs`: `⚙` (config coluna) → `settings`; `✕`
      (excluir coluna/linha, 2 lugares) → `x`; `✎` (editar URL) →
      `edit`; `↗` (abrir página) → `external-link`; `📄 {título}`
      (célula PageLink) → `file-text` + texto; `☑`/`☐` (item de
      MultiSelect marcado/desmarcado) → `check-square`/`square`
- [x] `card_detail_modal.rs`: `✕` (remover tag, 4 lugares — 1 a mais
      do que o levantamento inicial pegou, achado durante a varredura
      final) → `x`
- [x] `column_settings_modal.rs`: `✕` (remover opção) → `x`
- [x] `graph_view.rs`: `🕸` (ícone do estado vazio) → `network`
- [x] Varredura final em todo `ui/src` confirmando zero glifo de
      ícone restante fora de comentário/`<kbd>`
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: página de teste com embed de
      kanban (card com checklist/due/comentário/anexo) e embed de
      tabela — badges do card e botão de excluir coluna nítidos

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Nenhum — fecha a série de 4 ciclos (144-147)

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`,
`trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Varredura final (`grep` por faixas Unicode de emoji/símbolo em toda
`ui/src`) achou 1 uso real que tinha escapado do levantamento inicial
do ciclo 144 — um 4º botão `{ "✕" }` em `card_detail_modal.rs` (seção
de anexos, não a de tags, indentação diferente o suficiente pra não
bater no `replace_all` da primeira passada). Corrigido. Os únicos
glifos restantes no repo inteiro depois desta varredura são comentários
de código (não renderizados) e as setas `↑ ↓ ← →` do cheatsheet
(excluídas de propósito desde o ciclo 144, representam teclas físicas).

Validação ao vivo via MCP: sessão do driver travou (timeout de script)
ao abrir uma página de exemplo pré-existente com embeds — não
relacionado às mudanças deste ciclo (reidentificada como instabilidade
de dev server já documentada em ciclos anteriores); reiniciei
`scripts/dev.sh` e reconectei numa porta nova, sem perda de trabalho.
Criei uma página de teste isolada com kanban (card com checklist 1/2,
due date, 1 comentário, 1 anexo) e tabela — todos os badges/ícones do
card renderizaram nítidos (check-square, calendar, message-circle,
paperclip) e o botão de excluir coluna (x) também. Página de teste
removida ao final.
