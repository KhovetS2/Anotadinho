---
id: "206"
titulo: "Histórico de implementação dentro do vault"
status: done
criado: 2026-08-22
autor: humano
prioridade: alta
depende_de: [201, 205]
estima_min: 150
agente_alvo: claude-opus
---

# Histórico de implementação dentro do vault

## Objetivo

Trazer pra dentro do vault o que só o repositório enxergava, pra o
próprio Anotadinho virar a ferramenta de acompanhar o Anotadinho.

## Critérios de aceite

- [x] `scripts/migrar-ciclos.py` — idempotente, gera uma página por
      ciclo a partir de `cycles/tasks/` + `cycles/status/`.
- [x] 168 ciclos como páginas `type: ciclo`, com o embed de fluxo e o
      `status` no vocabulário do ciclo 201.
- [x] `pages/ciclos.md` virou painel com consultas vivas, no lugar da
      tabela estática que parou no ciclo 003.
- [x] `pages/produto/agent-os-capacidades.md` documentando adaptador,
      MCP, propostas e os limites.
- [x] `arquitetura.md` e `guia-agent-os.md` atualizados.
- [x] 2 cenários de harness travando a migração.

## Comandos de validação

```bash
python3 scripts/migrar-ciclos.py
cargo test --workspace
node scripts/uitest/run.mjs
```

## Bug encontrado, e ele era meu

**Título com `: ` derruba o frontmatter inteiro, em silêncio.**

`title: Ciclo 006 — MarkdownCodec: parse e serialize` é YAML inválido.
O parser descarta o bloco todo: a página perde título, tipo e tags de
uma vez, cai pro nome do arquivo e SOME das consultas — sem erro nenhum.
Apareceu como "25 dos 168 ciclos não aparecem na consulta".

O mesmo defeito estava no `fluxo::montar_pagina` (ciclo 203): uma
resposta do agente cujo título tivesse dois-pontos geraria uma spec com
frontmatter quebrado. Corrigido com `markdown::escapar_escalar_yaml`,
que só põe aspas quando precisa — o vault é editado à mão, e aspas em
tudo pioraria a leitura.

O cenário de harness conta as páginas em vez de olhar uma amostra,
justamente porque este defeito é invisível numa inspeção por amostragem.
