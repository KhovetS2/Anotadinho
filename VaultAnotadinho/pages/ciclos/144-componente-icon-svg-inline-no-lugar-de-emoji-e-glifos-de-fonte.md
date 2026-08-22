---
title: Ciclo 144 — Componente Icon SVG inline no lugar de emoji e glifos de fonte
type: ciclo
ciclo: "144"
status: concluida
date: 2026-08-14
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 144 — Componente Icon SVG inline no lugar de emoji e glifos de fonte

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Componente Icon SVG inline no lugar de emoji e glifos de fonte

## Objetivo

Pedido do usuário: os ícones da interface hoje são emoji/glifos Unicode
(⚙ ⋯ ✕ 🏠 📁 📅 🔍 📄 🕸 🔗 🕐 ⬇ ⎇ ⚡ 💬 📎 ↗ ☀ 🌙 ◀ ▶ ✎ ✓ ☑ ☐), que
dependem de fonte de emoji/símbolo instalada no SO — em algumas
distros Linux sem `noto-emoji`, ou no Windows sem fallback de fonte
correto, viram caixas vazias ("tofu") ou ficam visualmente
inconsistentes com o resto do app. Este ciclo cria o componente base
(`Icon`) com o conjunto de ícones SVG inline necessário — os próximos
3 ciclos (145-147) trocam os usos por área da UI.

## Critérios de aceite

- [x] `ui/src/components/icon.rs` (novo): componente `Icon` — prop
      `name: &'static str`, renderiza um `<svg>` inline (`viewBox="0 0
      24 24"`, `stroke="currentColor"`, `fill="none"`,
      `stroke-width="2"`, `stroke-linecap="round"`,
      `stroke-linejoin="round"`, sem cor fixa — herda a cor do texto
      do elemento pai via `currentColor`, então funciona igual em
      hover/tema claro/escuro sem código extra) via `match` sobre o
      nome
- [x] Conjunto de ícones cobrindo todos os glifos hoje em uso:
      `settings`, `more-horizontal`, `x`, `edit`, `check`, `square`,
      `check-square`, `home`, `folder`, `calendar`, `search`,
      `file-text`, `network`, `link`, `clock`, `download`,
      `git-branch`, `zap`, `message-circle`, `paperclip`,
      `external-link`, `sun`, `moon`, `chevron-left`, `chevron-right`
- [x] Registrado em `ui/src/components/mod.rs`
- [x] CSS mínimo em `main.css` (`.icon { width: 1em; height: 1em;
      vertical-align: -0.125em; }`)
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`, `cd ui &&
      trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: injetei o markup SVG exato de
      10 dos 25 ícones (settings/home/moon/folder/link/edit/network/
      zap/message-circle/paperclip) como overlay na janela rodando —
      todos legíveis e proporcionais no tamanho real de botão

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Trocar os usos existentes de emoji pelos ícones novos — feito nos
  ciclos 145-147, este ciclo só cria a peça reutilizável
- Ícones dos atalhos de teclado do cheatsheet (`↑ ↓ ← →` dentro de
  `<kbd>`) — representam teclas físicas do teclado, não "ícones" de
  interface, e são setas Unicode simples (`U+2190`-`U+2193`) amplamente
  suportadas mesmo sem fonte de emoji, diferente dos pictogramas
  (`U+1F300+`) e dingbats que motivaram o pedido
- Sistema de carregamento de ícone externo (arquivo `.svg` avulso,
  sprite sheet) — os SVGs ficam inline no Rust, mesmo padrão zero-
  dependência do resto do projeto

## Notas

`cd ui && cargo test --lib`: 84 passados (sem testes novos — componente
puramente de apresentação, sem lógica testável). `cargo test
--workspace`, `trunk build`, `cargo build --manifest-path
src-tauri/Cargo.toml`: OK.

25 ícones desenhados à mão com primitivas SVG simples (circle/line/
rect/path/polygon/polyline), sem copiar path data de nenhuma
biblioteca de ícones existente — estilo consistente (linha, 2px,
cantos arredondados). `settings` usa 3 sliders em vez de uma
engrenagem de verdade (mais fácil de desenhar com precisão e ainda
reconhecível). `edit`/`link` usam `<g transform="rotate(45 ...)">`
sobre retângulos simples em vez de path complexo. `moon` usa a fórmula
clássica de crescente (círculo grande menos círculo deslocado).

Ícones das setas do cheatsheet (`↑ ↓ ← →` dentro de `<kbd>`) ficaram
de fora de propósito (ver Não-objetivos) — representam teclas físicas
do teclado, não ícone de interface.

## Resultado

# Ciclo 144 - done

## Resumo

Pedido do usuário: trocar os ícones de emoji/glifo Unicode (que
dependem de fonte de emoji/símbolo instalada no SO e podem virar
caixa vazia em outros sistemas) por SVG inline. Este ciclo cria o
componente base `Icon` com os 25 ícones necessários — os próximos 3
ciclos trocam os usos por área da UI.

## Arquivos criados/modificados

- `ui/src/components/icon.rs` (novo) — componente `Icon` + 25 ícones
- `ui/src/components/mod.rs` — registra `pub mod icon;`
- `ui/src/styles/main.css` — classe `.icon` (tamanho/alinhamento base)

## Testes adicionados

- Nenhum — componente puramente de apresentação

## Problemas encontrados

- Nenhum

## Notas para próximos ciclos

- Ícones prontos: `settings`, `more-horizontal`, `x`, `edit`, `check`,
  `square`, `check-square`, `home`, `folder`, `calendar`, `search`,
  `file-text`, `network`, `link`, `clock`, `download`, `git-branch`,
  `zap`, `message-circle`, `paperclip`, `external-link`, `sun`,
  `moon`, `chevron-left`, `chevron-right`
- Uso: `<Icon name="home" />` (opcional `class={classes!("...")}`)
