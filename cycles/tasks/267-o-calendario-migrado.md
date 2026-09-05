---
id: "267"
titulo: "O calendário migrado: o primeiro embed a declarar"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["262", "265"]
estima_min: 120
---

# 267 — O calendário migrado

## Objetivo

O critério que ficou marcado `[~]` no ciclo 265: o calendário anda entre
dias com `j`/`k`. É o primeiro embed a DECLARAR o que consome, e a prova
de que o mecanismo dos ciclos 262/265 serve pra migrar os outros.

## Critérios de aceite

- [x] `Interesse::da_tecla` no núcleo, testado, mapeando tecla→categoria
- [x] O calendário declara `Movimento` e nada mais
- [x] `j`/`k` andam uma LINHA da grade (7 dias no mês, 1 na semana/dia)
- [x] `h`/`l` andam um dia
- [x] O dia sob o cursor tem realce próprio
- [x] `dd` de dentro do calendário SOBE e não apaga
- [x] Cenários pros dois lados da declaração
- [x] Baseline do snapshot regravada, com o diff conferido

## Por que `j` anda 7 dias no mês

A grade do mês é semana × dia. `j` desce uma LINHA e `l` anda uma
COLUNA — é o que a grade desenha, e é o que qualquer pessoa que usa vim
espera de uma grade. Na semana e no dia não há linha, então tudo é um
dia.

## O realce era pré-requisito, não enfeite

Sem o `--cursor`, andar dentro do mesmo mês não mudava NADA na tela: o
`anchor` só era visível quando a virada trocava de mês. O movimento
funcionaria e pareceria quebrado.

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs
```
