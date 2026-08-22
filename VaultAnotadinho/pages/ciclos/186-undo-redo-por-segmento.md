---
title: Ciclo 186 — Desfazer/refazer que entende de blocos
type: ciclo
ciclo: "186"
status: concluida
date: 2026-08-20
prioridade: alta
depende_de: [149, 159]
tags:
- ciclo
---

# Ciclo 186 — Desfazer/refazer que entende de blocos

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Desfazer/refazer que entende de blocos

## Objetivo

**Correção da premissa original:** a pilha própria já existia desde o
ciclo 074 — o `Ctrl+Z` do editor NÃO é o nativo do `contenteditable`, é
um histórico de snapshots do markdown inteiro. O bug real, achado ao ler
o código, é outro: a decisão de agrupar snapshots era só temporal (janela
de 800ms), então uma mutação ESTRUTURAL disparada logo depois de digitar
caía dentro da janela, não virava ponto de desfazer, e o estado
pré-mutação sumia do histórico. Desfazer pulava para um estado bem mais
antigo, comendo a digitação junto.

Este ciclo separa as duas coisas: digitação agrupa, mutação estrutural
nunca agrupa. E o histórico, que era duas `Vec<String>` soltas dentro do
componente, vira um tipo testado no core.

## Critérios de aceite

- [x] `crates/core/src/history.rs` com `History` (`registrar`,
      `desfazer`, `refazer`, `pode_desfazer`, `pode_refazer`,
      `reiniciar`, limite de profundidade), testado fora do WASM.
- [x] Toda mutação estrutural do editor (inserir pelo menu `/`,
      inserir/remover/mover/duplicar segmento, mudança de dados de embed,
      gravar `^id` de bloco) cria um ponto de desfazer próprio,
      independente do relógio.
- [x] Digitação continua agrupando numa janela de 800ms — uma rajada é
      um passo só, não um passo por tecla.
- [x] `Ctrl+Z` depois de inserir um embed logo após digitar tira só o
      embed e mantém o texto.
- [x] Trocar de página e recarga externa reiniciam o histórico.
- [x] Cenário de harness cobrindo exatamente o caso do bug.

## Comandos de validação

```bash
cargo build --workspace
cargo test --workspace
cargo build --manifest-path src-tauri/Cargo.toml
cd ui && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Desfazer com granularidade de caractere sobre a pilha própria — isso é
  o desfazer nativo e continua sendo dele.
- Persistir histórico entre sessões.

## Notas

Guardar o markdown COMPLETO por entrada, não um diff: o documento cabe
folgado em memória, e um snapshot elimina a classe inteira de bug de
"patch aplicado fora de ordem". O limite de profundidade existe só pra
página gigante não crescer sem fim.

É pré-requisito de conforto pro ciclo 175: mover bloco pelo teclado sem
poder desfazer assusta.

## Resultado

# 186 — Desfazer que entende de blocos

## Correção de premissa

A task nasceu dizendo que o desfazer era o nativo do `contenteditable`.
Não era: existe uma pilha própria de snapshots desde o ciclo 074. Ao ler
o código apareceu o bug de verdade, e a task foi reescrita em cima dele
antes da implementação.

**O bug:** o agrupamento de snapshots era decidido só por tempo (janela
de 800ms). Uma mutação estrutural disparada dentro dessa janela — inserir
um embed pelo `/` logo depois de digitar, que é o caso comum — não virava
ponto de desfazer. O estado pré-inserção era descartado e `Ctrl+Z` voltava
para um estado bem mais antigo, comendo a digitação junto com o embed.

## O que mudou

- `crates/core/src/history.rs` (novo): `History` com snapshots do markdown
  inteiro, limite de profundidade e `registrar(novo, agrupar)`. A decisão
  de agrupar fica com quem chama, porque depende de relógio — é isso que
  mantém o tipo testável fora do WASM. 8 testes.
- `crates/core/src/lib.rs`: registra o módulo.
- `ui/src/components/editor.rs`:
  - As duas `Vec<String>` soltas (`undo_stack`/`redo_stack`) e o
    `last_content_ref` viraram um `History` só.
  - `mark_edited_com(md, estrutural)` como base, com dois wrappers:
    `mark_edited` (digitação, pode agrupar) e `mark_edited_estrutural`
    (nunca agrupa).
  - Passaram a usar o estrutural: todas as inserções do menu `/` (embeds,
    blocos markdown, imagem, mermaid, asset), inserir linha em branco,
    remover/mover/duplicar embed, mudança de dados de embed, o atalho `n`
    e a gravação de `^id` de bloco.
  - Recarga vinda do disco reinicia o histórico: desfazer depois dela
    regravaria o arquivo com o conteúdo velho.

## Validação

- `cargo test --workspace`: 0 falhas (8 testes novos em `history`).
- `cargo build --manifest-path src-tauri/Cargo.toml`: ok.
- `cd ui && trunk build`: ok.
- `node scripts/uitest/run.mjs`: **20/20 em 112.2s**.
- Antes da correção o cenário novo falhava com o arquivo salvo vazio —
  o desfazer voltava até o estado inicial do histórico. Depois, volta
  exatamente um passo.
