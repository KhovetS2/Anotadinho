---
title: Ciclo 204 — Propostas de escrita com revisão humana
type: ciclo
ciclo: "204"
status: concluida
date: 2026-08-22
prioridade: alta
depende_de: [189, 190, 202]
tags:
- ciclo
---

# Ciclo 204 — Propostas de escrita com revisão humana

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Propostas com revisão

## Objetivo

O agente deixa de escrever DIRETO no vault: ele propõe, e a mudança só
vira arquivo depois de alguém ver o diff e aprovar.

Esta é a defesa que sustenta todo o acoplamento com modelos. As outras
— não ter shell, blindar o contexto no prompt — reduzem a CHANCE de o
agente ser enganado. Esta continua valendo mesmo se ele for: o estrago
para na tela de revisão.

## Critérios de aceite

- [x] `crates/core/src/proposta.rs`: `Proposta`, validação e diff.
- [x] Recusa caminho que escapa do vault (`..`, absoluto, `C:\`).
- [x] Recusa quando o estado mudou entre propor e aplicar — o agente
      decidiu com uma foto velha.
- [x] Recusa conteúdo com embed inválido, pela MESMA validação do
      `embed check` (ciclo 189).
- [x] Diff pelo motor do ciclo 190 — a pessoa lê no formato que já
      conhece da barra de conflito.
- [x] `anotadinho-cli propor / propostas / aplicar / recusar`.
- [x] Tela `type: propostas` com diff, Aplicar e Recusar.
- [x] 4 cenários de harness + 6 testes no `crates/ipc`.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Aplicar parte de uma proposta (só linha X): é tudo ou nada por ora.
- Fila de propostas encadeadas.

## Detalhes que importam

As propostas vivem em `.anotadinho/propostas/`, FORA de `pages/` — se
ficassem dentro, apareceriam como página do vault e entrariam em
consulta.

`aplicar` **revalida antes de escrever**: entre propor e aprovar o vault
pode ter mudado, e há teste provando que aplicar não sobrescreve o
trabalho de outra pessoa.

Proposta ilegível é PULADA na listagem, não fatal — uma sozinha
corrompida não pode esconder as outras da revisão.

## Resultado

# 204 — Propostas com revisão

## O que mudou

- `crates/core/src/proposta.rs` (novo): `Proposta`, `Operacao`, `Recusa`,
  validação e `diff` reusando o ciclo 190. 9 testes.
- `crates/ipc`: `handle_propor`, `handle_listar_propostas`,
  `handle_aplicar_proposta`, `handle_recusar_proposta`. 6 testes.
- `crates/cli`: `propor`, `propostas`, `aplicar`, `recusar`.
- `src-tauri`: os quatro comandos.
- `ui/src/components/propostas_view.rs` (novo): tela de revisão.
- `scripts/uitest/fluxo.mjs`: 4 cenários.

## O laço, pelo terminal

```
$ echo "..." | anotadinho-cli propor pages/nova.md --motivo "..." --autor claude
pages-nova-md-1787419155
$ anotadinho-cli propostas
pages-nova-md-...  pages/nova.md  Criar  claude  ...
$ anotadinho-cli aplicar pages-nova-md-...
pages/nova.md
```

Entre o primeiro e o terceiro comando, `pages/nova.md` **não existe**.

## Validação

- `cargo test --workspace`: 0 falhas.
- `node scripts/uitest/run.mjs`: **117/117 em 279.3s**, vault limpo.
