---
id: "203"
titulo: "Promover mensagem da conversa em spec ou proposta"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [201, 202]
estima_min: 90
agente_alvo: claude-opus
---

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
