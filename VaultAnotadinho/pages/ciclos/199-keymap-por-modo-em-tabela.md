---
title: Ciclo 199 — Keymap por modo em tabela
type: ciclo
ciclo: "199"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: [197]
tags:
- ciclo
---

# Ciclo 199 — Keymap por modo em tabela

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Keymap por modo em tabela

## Objetivo

Os bugs dos ciclos 194, 195 e 197 foram o MESMO defeito estrutural: uma
tecla tratada no modo errado. Cada atalho carregava sua própria condição
solta dentro de um `on_keydown` de centenas de linhas, e nada obrigava a
responder "isto vale em qual modo?".

O harness pega a ocorrência; a tabela mata a espécie.

## Critérios de aceite

- [x] `ATALHOS`: tabela de `Atalho { tecla, alt, modo, descricao }`.
- [x] `comando_vale()` — um lugar só respondendo "é comando aqui?".
- [x] Os handlers consultam a tabela em vez de repetir a condição.
- [x] Testes DERIVADOS da tabela: sem duplicata, todo atalho de bloco é
      de navegação, letra comum não é atalho, atalho só vale no próprio
      modo, seta só move bloco com Alt, precedência de modo, e toda
      entrada com descrição.
- [x] Comportamento inalterado — suíte inteira verde.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Mover os atalhos GLOBAIS (`app.rs`/`state.rs`) pra esta tabela: eles
  já têm um keymap configurável próprio, e misturar os dois seria outro
  ciclo.

## Notas

O teste `todo_atalho_de_bloco_e_de_navegacao` é o que fecha o buraco do
194: se alguém marcar um atalho de bloco como `Edicao`, ele volta a
disparar durante a digitação — e agora o `cargo test` reprova antes de
chegar no app.

## Resultado

# 199 — Keymap por modo em tabela

## O que mudou

- `ui/src/components/editor.rs`: `Atalho`, `ATALHOS`, `atalho_de()`,
  `comando_vale()`; `Modo` virou público com `Debug`. Os handlers de
  `n`, `c`, `d`, `y`, `K`, `J` e `Alt+setas` consultam a tabela, e o
  guard de tecla imprimível também.
- 7 testes derivados da tabela (46 no total na UI).

## Por que isto e não mais cenários

Três ciclos seguidos corrigiram o mesmo defeito estrutural. Cenário pega
a ocorrência depois que ela existe; a tabela impede a espécie. O teste
`todo_atalho_de_bloco_e_de_navegacao` reprova no `cargo test` se alguém
marcar um atalho de bloco como de edição — o bug do 194 não tem mais
como chegar no app.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 46 testes.
- `trunk build`: `✅ success`; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **85/85 em 237.9s** — comportamento
  inalterado, que era o objetivo de um refactor.
