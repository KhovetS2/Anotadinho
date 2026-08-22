---
title: Ciclo 169 — Consulta com agrupamento e agregados
type: ciclo
ciclo: "169"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: ["154"]
tags:
- ciclo
---

# Ciclo 169 — Consulta com agrupamento e agregados

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Consulta com agrupamento e agregados

## Objetivo

Hoje, pra ver specs por status, o painel precisa de uma consulta POR
status (o `painel.md` tem duas, e ficaria com cinco se cobrisse o ciclo
inteiro). Agrupar resolve numa visão só — e é o que falta pra a
consulta aposentar de vez o kanban manual do [[Roadmap]].

## Critérios de aceite

- [x] `group_by: <campo>` no YAML da consulta, com um cabeçalho por
      valor e as páginas embaixo, na ordem do `sort`
- [x] Página sem o campo cai num grupo "sem <campo>" no fim — mesma
      regra do `sort` (ausente não é o menor, é ausente)
- [x] Contagem por grupo no cabeçalho
- [x] `aggregate: [{ field, op: count|sum|avg|min|max }]` no rodapé de
      cada grupo e do total
- [x] Com agrupamento a saída é sempre em lista sob o cabeçalho do
      grupo, inclusive quando `view` é `cards`/`table`: misturar grade
      dentro de grupo recolhível dobrava a complexidade visual sem
      ganho claro. Fica anotado como possível refinamento
- [x] Grupo colapsável, com o estado guardado no YAML (`collapsed`),
      pra o painel abrir do jeito que ficou
- [x] Motor no core, testado sem WASM: agrupamento, ordem dos grupos,
      grupo vazio, cada operador de agregado, campo não-numérico em
      `sum`/`avg` (ignora em vez de somar lixo)
- [x] `anotadinho-cli query --group-by` imprime a mesma coisa

## Comandos de validação

```bash
cargo test -p anotadinho-core
cargo test -p anotadinho-cli
cargo test --workspace
cd ui && trunk build
```

## Não-objetivos

- Arrastar entre grupos (isso é kanban, e o kanban existe)
- Agrupar por mais de um campo

## Notas

`cargo test -p anotadinho-core`: 154 (+6). `cargo test -p
anotadinho-cli`: 35 (+2). Harness (177): 11/11, com cenário que confere
cabeçalho, contagem, recolher e a persistência do recolhido no YAML.

`run_grouped` é irmã de `run` (não substituta), como previsto — o CLI e
o embed continuam usando `run` no caminho sem agrupamento, e nada do
que já existia mudou de assinatura.

`Query::run` devolve `Vec<&PageIndexEntry>`; o agrupamento provavelmente
quer uma função irmã (`run_grouped`) em vez de mudar a assinatura
existente, que o CLI e o embed já usam.

## Resultado

# Ciclo 169 - done

## Resumo

`group_by` e `aggregate` na consulta: "specs por status, com contagem"
numa visão só, em vez de uma consulta por status (o painel tinha duas e
precisaria de cinco pra cobrir o ciclo inteiro). Grupos são
recolhíveis, e o estado fica no YAML.

## Arquivos criados/modificados

- `crates/core/src/query.rs` — `AggregateOp`, `Aggregate`, `Grupo`,
  `run_grouped`, `recolhido`/`alternar_recolhido` + 6 testes
- `ui/src/components/embeds/inline_query.rs` — cabeçalho de grupo,
  agregados e recolher
- `ui/src/styles/main.css` — `.query-embed__grupo*`
- `crates/cli/src/main.rs` — `--group-by` e `--aggregate` + 2 testes
- `scripts/uitest/cenarios.mjs` — cenário novo

## Testes adicionados

- agrupa com ausentes no fim; sem `group_by` devolve um grupo só
- count/sum/avg/max; campo não-numérico em `sum` devolve "—" em vez de
  somar lixo; min/max caem pra alfabético quando não são números
- recolhido faz round-trip no YAML
- CLI: saída agrupada e agregado inválido com mensagem útil

## Problemas encontrados

- `view: cards`/`table` com agrupamento ficou fora: grade dentro de
  grupo recolhível dobra a complexidade visual sem ganho claro.
  Anotado na task.

## Notas para próximos ciclos

- O [[Roadmap]] (kanban manual) agora tem substituto real, se o usuário
  quiser: uma consulta agrupada por status.
