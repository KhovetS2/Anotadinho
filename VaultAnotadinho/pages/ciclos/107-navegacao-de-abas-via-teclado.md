---
title: Ciclo 107 — Navegacao de abas via teclado
type: ciclo
ciclo: "107"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: ["105"]
tags:
- ciclo
---

# Ciclo 107 — Navegacao de abas via teclado

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Navegação de abas via teclado

## Objetivo

Quarto ciclo do tema "navegação 100% via teclado". Hoje só existe
`Ctrl+W` (próxima aba, cíclico). Este ciclo adiciona "aba anterior"
(já é uma ação do `GlobalKeymap` do ciclo 105, faltando implementação)
e "pular direto pra aba N" (`Ctrl+1`..`Ctrl+9`, fixo — não faz sentido
customizar 9 binds separados no `GlobalKeymap`).

## Critérios de aceite

- [x] "Próxima aba"/"Aba anterior" (`GlobalKeymap`) navegam
      ciclicamente pra frente/trás em `open_tabs` (próxima já existe via
      `Ctrl+W`; anterior é nova)
- [x] `Ctrl+1` a `Ctrl+9` pulam direto pra aba de índice 0-8 (se
      existir aquele índice; sem efeito se não tiver aba suficiente) —
      fixo, não faz parte do `GlobalKeymap` customizável (mesmo padrão
      de navegador/editor de código)
- [x] `ui/src/components/tab_bar.rs` ganha um indicador visual sutil do
      número de cada aba (ex: tooltip ou badge pequeno) pra descobrir
      qual `Ctrl+N` pula pra qual
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

- Reordenar abas por teclado (drag continua sendo só mouse, se existir
  — confirmar se já existe reorder de abas antes de decidir se entra
  aqui)
- `Ctrl+1..9` customizável — fixo de propósito (convenção já
  estabelecida em outras ferramentas, menos uma coisa pra configurar)

## Notas

Depende do ciclo 105 pro dispatcher de `GlobalKeymap` já existir (esse
ciclo só adiciona as ações que faltam + a lógica fixa de `Ctrl+1..9`,
que fica FORA do keymap customizável, direto no dispatcher do
`app.rs`).

"Próxima aba"/"Aba anterior" já tinham sido implementadas no próprio
ciclo 105 (fazia sentido construir o dispatcher completo de uma vez,
já que a estrutura de match já estava lá) — esse ciclo só faltava
`Ctrl+1..9` + o indicador visual.

`Ctrl+1..9` checado ANTES do match de `GlobalKeymap` (não depois) —
garante que fica reservado mesmo se o usuário custimizar alguma ação
pra um dígito por engano.

Validado ao vivo via MCP `tauri`: 3 abas abertas mostram badges 1/2/3;
Ctrl+2 pula pra "arquitetura", Ctrl+1 pula pra 🏠; Ctrl+9 (sem 9ª aba)
não faz nada, sem erro; tooltip de cada aba mostra "(Ctrl+N)".

## Resultado

# Ciclo 107 - done

## Resumo

Quarto ciclo do tema "navegação 100% via teclado". `Ctrl+1`..`Ctrl+9`
pulam direto pra aba de índice 0-8 (fixo, fora do `GlobalKeymap`
customizável, mesma convenção de navegador/editor de código). `TabBar`
ganha um badge numérico sutil + tooltip com o atalho. "Próxima
aba"/"Aba anterior" já existiam desde o ciclo 105.

## Arquivos criados/modificados

- `ui/src/app.rs` — `Ctrl+1..9` no dispatcher, checado antes do
  `GlobalKeymap`
- `ui/src/components/tab_bar.rs` — badge numérico + tooltip
- `ui/src/styles/main.css` — `.tab-bar__tab-num`

## Testes

`cargo test --workspace`: 82 (inalterado). `cd ui && cargo test --lib`:
75 (inalterado — sem lógica pura nova pra testar). Total 157.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: 3 abas com badges 1/2/3; Ctrl+2/
Ctrl+1 pulam corretamente; Ctrl+9 sem 9ª aba não faz nada; tooltip
mostra o atalho.

## Notas

"Próxima aba"/"Aba anterior" foram implementadas junto com o
dispatcher no ciclo 105 (antecipado, fazia sentido construir tudo de
uma vez) — detalhes no arquivo de task.

Próximo: cheatsheet de atalhos (108) — fecha o tema navegação 100% via
teclado.
