---
title: Ciclo 140 — Sidebar não devolve nav-mode ao navegar dentro dela
type: ciclo
ciclo: "140"
status: concluida
date: 2026-08-09
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 140 — Sidebar não devolve nav-mode ao navegar dentro dela

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Sidebar não devolve nav-mode ao navegar dentro dela

## Objetivo

Bug real reportado pelo usuário: ao delegar do nav-mode pra sidebar
(Enter em "sidebar") e tentar navegar com seta DENTRO dela, a sessão
de navegação por REGIÕES reativava sozinha no meio do caminho.

## Critérios de aceite

- [x] `sidebar.rs::on_nav_keydown` ganha `e.stop_propagation()` em
      TODOS os ramos (ArrowDown/Up/Right/Left/Enter — já existia só
      no Escape) — sem isso, a tecla continuava borbulhando até
      `.app-root` mesmo depois de `prevent_default()`, e como o
      delegate só desativa `nav_mode_active` (não `nav_mode_enabled`),
      o `.app-root` reagia à MESMA seta como "primeira seta, inicia
      sessão nova"
- [x] `app.rs`: guarda `focus_is_nav_tracked` (ciclo 136, antes só em
      Enter/Backspace/Escape) estendida TAMBÉM pras setas — proteção
      adicional pro mesmo tipo de bug em qualquer outro delegate/menu
      que não isole a própria tecla (ex: os 3 menus dropdown locais,
      que usam listener de `window` — `stop_propagation` ali não
      adiantaria, porque `.app-root` é ANCESTRAL e já disparou antes
      do listener de `window` rodar); a recuperação de foco perdido em
      `<body>` já é feita separadamente pelo polling do ciclo 138, então
      o "autocuro" antigo das setas (voltar pro item 0) não fazia mais
      falta
- [x] `cd ui && cargo test --lib`, `cargo test --workspace`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
- [x] Validação ao vivo via MCP `tauri`: delegate pra sidebar,
      múltiplos ArrowDown seguidos — badge continua ausente em todos,
      item da sidebar avança corretamente a cada seta; Enter na
      sidebar abre a página certa, mesmo sem o nav-mode interferir

## Comandos de validação

```bash
cd ui && cargo test --lib
cargo test --workspace
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Mover o listener de teclado dos 3 menus dropdown de `window` pro
  elemento do próprio menu (fix estrutural mais correto pro caso
  deles, mas mais invasivo) — a guarda `focus_is_nav_tracked` já
  cobre o sintoma pra esse caso sem precisar dessa refatoração

## Notas

### Causa raiz

`e.prevent_default()` bloqueia o comportamento PADRÃO do navegador
pra aquela tecla (ex: rolar a página), mas NÃO impede o evento de
continuar borbulhando pros elementos ancestrais — só
`e.stop_propagation()` faz isso. O comentário já existente no ramo
"Escape" de `sidebar.rs` (ciclo 106) já explicava exatamente esse
mecanismo, mas o fix só tinha sido aplicado ali, não nos outros ramos
— antes do nav-mode existir, não tinha nada mais no `.app-root`
escutando seta pura (sem Ctrl), então a lacuna era inofensiva. O
nav-mode (ciclo 133) mudou isso: agora QUALQUER seta que chegue em
`.app-root` com a capacidade ligada tenta iniciar uma sessão.

### Por que a guarda em `app.rs` também foi estendida

Ainda que o fix em `sidebar.rs` resolva o caso relatado, o MESMO
padrão de bug existe pros 3 menus dropdown locais (⚙, popover de git,
"⋯" do editor) — a diferença é que o listener de seta deles vive no
`window` (`EventListener::new(&window, "keydown", ...)`), não no
elemento do menu. Como `.app-root` é ANCESTRAL do alvo do evento e
`window` é o destino final do bubbling, `.app-root` SEMPRE dispara
ANTES do listener de `window` — um `stop_propagation()` dentro do
listener de `window` não teria efeito nenhum sobre um listener de
ANCESTRAL que já rodou. A guarda `focus_is_nav_tracked` em `app.rs`
resolve isso de um jeito mais geral, sem precisar mexer na estrutura
dos 3 menus.

## Resultado

# Ciclo 140 - done

## Resumo

Bug real reportado pelo usuário: navegar dentro da sidebar (depois de
delegar do nav-mode) reativava a navegação entre regiões no meio do
caminho. Causa raiz: `sidebar.rs` só chamava `stop_propagation()` no
Escape, não nos outros ramos — a tecla continuava borbulhando até
`.app-root`, que reagia à mesma seta como "primeira seta, inicia
sessão nova" (o delegate só desativa a sessão, não a capacidade).
Corrigido no ponto de origem (sidebar) e reforçado de forma mais
geral em `app.rs` (guarda `focus_is_nav_tracked` estendida pras
setas), cobrindo também os 3 menus dropdown locais, que têm o mesmo
padrão de bug por um motivo estrutural diferente (listener em
`window`, onde `stop_propagation` não teria efeito sobre o ancestral
`.app-root` que já disparou antes).

## Arquivos criados/modificados

- `ui/src/components/sidebar.rs` — `stop_propagation()` em todos os
  ramos de `on_nav_keydown`
- `ui/src/app.rs` — guarda `focus_is_nav_tracked` estendida pras setas

## Testes

`cd ui && cargo test --lib`: 84. `cargo test --workspace`: 116.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: delegate pra sidebar, múltiplos
ArrowDown seguidos sem o badge reaparecer em nenhum, item avançando
corretamente a cada seta; Enter abrindo a página certa.

## Notas

Ver Notas completas no arquivo de task sobre a causa raiz
(`prevent_default` não impede bubbling, só `stop_propagation` faz) e
por que a guarda em `app.rs` foi estendida além do fix pontual da
sidebar (cobre os 3 menus dropdown locais também, por um motivo
estrutural diferente).
