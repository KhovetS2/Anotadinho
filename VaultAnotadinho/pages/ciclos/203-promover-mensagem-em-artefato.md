---
title: Ciclo 203 — Promover mensagem da conversa em spec ou proposta
type: ciclo
ciclo: "203"
status: concluida
date: 2026-08-22
prioridade: alta
depende_de: [201, 202]
tags:
- ciclo
---

# Ciclo 203 — Promover mensagem da conversa em spec ou proposta

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Promover mensagem em artefato

## Objetivo

A ponte entre a conversa solta e o trabalho estruturado. Sem ela o fluxo
morre no copiar-e-colar — que é onde a maioria das integrações de chat
com "criar tarefa" para.

## Critérios de aceite

- [x] `fluxo::montar_pagina` monta a página com frontmatter, o embed de
      fluxo em rascunho e o rastro da origem.
- [x] `slug_de_titulo` e `titulo_sugerido`.
- [x] Botões "virar spec" / "virar proposta" na resposta do agente, só no
      hover.
- [x] A página criada abre, e o embed de fluxo já responde nela.
- [x] Teste garantindo que a página gerada VOLTA a parsear como fluxo —
      se o wrapper saísse errado, ela nasceria sem máquina de estados.
- [x] 2 cenários de harness, que limpam o que criam.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
node scripts/uitest/run.mjs
```

## Não-objetivos

- Promover um TRECHO selecionado da resposta (a mensagem inteira basta
  por ora).

## Resultado

# 203 — Promover mensagem em artefato

## O que mudou

- `crates/core/src/fluxo.rs`: `montar_pagina`, `slug_de_titulo`,
  `titulo_sugerido`. 6 testes novos (15 no módulo).
- `ui/src/components/conversa_view.rs`: botões de promover na resposta
  do agente, com a origem apontando pra conversa.
- `scripts/uitest/fluxo.mjs`: 2 cenários.

## O teste que importa

`pagina_montada_parseia_de_volta_no_fluxo` monta a página e reparseia
procurando o embed. É o guarda contra o erro silencioso: um wrapper
malformado geraria uma página bonita, sem máquina de estados nenhuma, e
ninguém perceberia até tentar aprovar.

## Validação

- `cargo test --workspace`: 0 falhas.
- `node scripts/uitest/run.mjs promover:`: 2/2.
