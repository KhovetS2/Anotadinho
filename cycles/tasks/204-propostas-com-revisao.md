---
id: "204"
titulo: "Propostas de escrita com revisão humana"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [189, 190, 202]
estima_min: 240
agente_alvo: claude-opus
---

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
