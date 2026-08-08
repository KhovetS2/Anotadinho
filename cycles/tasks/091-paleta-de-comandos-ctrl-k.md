---
id: "091"
titulo: "Paleta de comandos Ctrl+K"
status: done
criado: 2026-08-07
autor: humano
prioridade: media
depende_de: []
estima_min: 60
agente_alvo: claude-sonnet
---

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
