---
id: "169"
titulo: "Consulta com agrupamento e agregados"
status: done
criado: 2026-08-20
autor: humano
prioridade: media
depende_de: ["154"]
estima_min: 120
agente_alvo: claude-opus-5
---

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
