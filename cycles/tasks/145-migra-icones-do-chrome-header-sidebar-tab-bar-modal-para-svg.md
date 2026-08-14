---
id: "145"
titulo: "Migra ícones do chrome (header, sidebar, tab-bar, modal) para SVG"
status: done
criado: 2026-08-14
autor: humano
prioridade: media
depende_de: ["144"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Migra ícones do chrome (header, sidebar, tab-bar, modal) para SVG

## Objetivo

Troca os emoji/glifos usados como ícone nos componentes de chrome
(sempre visíveis, fora do conteúdo de uma página) pelo componente
`Icon` (ciclo 144): `header_bar.rs`, `sidebar.rs`, `tab_bar.rs`,
`modal.rs`.

## Critérios de aceite

- [x] `header_bar.rs`: `◀`/`▶` → `chevron-left`/`chevron-right`;
      `⎇ {n}` → `git-branch` + texto; `☀`/`🌙` (botão + itens do menu)
      → `sun`/`moon`; `⚙` → `settings`; `✓` (autosave/vim mode) →
      `check` condicional
- [x] `sidebar.rs`: ícones da barra colapsada (`📄`/`📅`/`🔍`) →
      `file-text`/`calendar`/`search`; `✕` (limpar busca) → `x`;
      `🏠+`/`📁+` → `Icon` + `"+"` literal (ASCII, não é ícone de
      fonte); `📁` (mover/pasta) → `folder`; `⬇` (exportar pasta) →
      `download`; `page_icon()` passa a devolver o NOME do ícone em
      vez do glifo
- [x] `tab_bar.rs`: `🏠` (aba inicial) → `home`
- [x] `modal.rs`: `✕` (fechar) → `x`
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: screenshot do chrome completo
      (header/sidebar/tab-bar) confirmando ícones renderizando
      nítidos; abri o menu `⚙` e liguei "Salvamento automático" pra
      confirmar o ícone de check aparecendo condicionalmente

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Ícones do editor/painéis/embeds — ciclos 146/147

## Notas

`cd ui && cargo test --lib`: 84 passados. `cargo test --workspace`,
`trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Bug de sintaxe pego no build (não um bug de app, erro de compilação
mesmo): `if/else` dentro de `html!` não pode envolver os braços em
`html! { ... }` aninhado — precisa usar a sintaxe de bloco direta do
próprio `html!` (`if cond { <Elemento/> } else { {expr} }`), igual o
resto do arquivo já fazia pro `if let Some(n) = shortcut_num`.
Corrigido em `tab_bar.rs`.

Validação ao vivo confirmou visualmente: ícones de chevron (colapsar
sidebar), git-branch, sun/moon (tema), settings (engrenagem via
sliders), home (aba fixa + botão "nova página inicial"), folder
(pastas da árvore + botões), file-text/calendar/search (rail
colapsada), x (fechar busca/modal), download (exportar pasta) — todos
nítidos e proporcionais ao texto ao redor. Checkbox de "Salvamento
automático" testado ligando/desligando via JS: ícone `check` aparece/
some corretamente.
