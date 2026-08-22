---
title: Ciclo 108 — Cheatsheet de atalhos
type: ciclo
ciclo: "108"
status: concluida
date: 2026-08-08
prioridade: baixa
depende_de: ["105"]
tags:
- ciclo
---

# Ciclo 108 — Cheatsheet de atalhos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Cheatsheet de atalhos

## Objetivo

Quinto e último ciclo do tema "navegação 100% via teclado" — capstone
de usabilidade. Com `GlobalKeymap` (105) e `VimKeymap` (092) sendo os
dois mapas de teclas customizáveis do app, este ciclo dá um jeito
rápido de VER todos os binds atuais de uma vez, sem precisar abrir os
dois modais de configuração separadamente.

## Critérios de aceite

- [x] Tecla `?` (fora de qualquer campo de texto/contenteditable) abre
      um overlay listando as ações + teclas atuais dos dois keymaps
      (`GlobalKeymap` sempre; `VimKeymap` só se o vim mode estiver
      ativado), lado a lado ou em duas seções
- [x] Também acessível via item "Atalhos (?)" no menu ⚙ e via comando
      na paleta ("Ver atalhos")
- [x] `Escape` fecha o overlay
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

- Editar atalhos direto do cheatsheet — é só leitura; pra mudar, ainda
  usa os modais de configuração dedicados (`GlobalKeymap`/`VimKeymap`)
- Busca/filtro dentro do cheatsheet — lista curta o bastante pra não
  precisar nesta v1

## Notas

Puramente de leitura sobre dado que já existe (`labeled_fields()` dos
dois keymaps) — não deveria precisar de nenhuma mudança de modelo de
dados, só um componente novo de apresentação. Bom ciclo pra fechar o
tema com baixo risco depois dos ciclos mais estruturais (104-107).

`Modal` genérico não trata Escape sozinho (só clique fora/✕) — o
`CheatsheetModal` adiciona seu próprio listener de `keydown` (mesmo
padrão dos popovers do menu ⚙/git status).

`?` sem Ctrl, filtrado por `is_text_input_target` (novo helper em
`app.rs`) — checa tag `input`/`textarea`/`select` e
`contenteditable="true"` no alvo do evento, pra não abrir o overlay
enquanto o usuário digita um "?" de verdade numa nota ou na busca da
sidebar.

Validado ao vivo via MCP `tauri`: `?` fora de campo de texto abre o
overlay com as 17 ações do `GlobalKeymap`; `?` dentro do campo de busca
da sidebar NÃO abre (filtro funcionando); Escape fecha; item do menu ⚙
e comando "Ver atalhos" da paleta também abrem; com vim mode ativado,
a segunda seção (19 ações do modo Normal) aparece.

## Resultado

# Ciclo 108 - done

## Resumo

Quinto e último ciclo do tema "navegação 100% via teclado" — capstone.
Tecla `?` (fora de campo de texto) abre um overlay somente leitura com
os binds atuais do `GlobalKeymap` (sempre) e do `VimKeymap` (só com
vim mode ativado), lado a lado. Também acessível via menu ⚙ e comando
"Ver atalhos" da paleta. Escape fecha.

## Arquivos criados/modificados

- `ui/src/components/cheatsheet_modal.rs` (novo) — overlay de leitura
  sobre `GlobalKeymap`/`VimKeymap::labeled_fields()`
- `ui/src/components/mod.rs` — registra o módulo
- `ui/src/components/command_palette.rs` — comando "Ver atalhos"
- `ui/src/components/header_bar.rs` — item "Atalhos (?)" no menu
- `ui/src/app.rs` — `is_text_input_target` (filtro do `?`), estado
  `cheatsheet_open`, wiring dos 3 pontos de entrada
- `ui/src/styles/components.css` — `.cheatsheet*`

## Testes

`cargo test --workspace`: 82 (inalterado). `cd ui && cargo test --lib`:
75 (inalterado — componente de apresentação puro, sem lógica nova
testável isoladamente). Total 157.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: `?` abre o overlay fora de campo de
texto, NÃO abre dentro da busca da sidebar; Escape fecha; menu ⚙ e
paleta também abrem; seção do vim mode aparece só com vim mode
ativado.

## Notas

Fecha o tema "navegação 100% via teclado" (104-108): modal de captura
genérico + GlobalKeymap + navegação da sidebar + navegação de abas +
cheatsheet. Detalhes no arquivo de task.

Com isso, todos os 11 ciclos planejados (098-108) + o ciclo extra 109
(pedido direto do usuário) estão completos.
