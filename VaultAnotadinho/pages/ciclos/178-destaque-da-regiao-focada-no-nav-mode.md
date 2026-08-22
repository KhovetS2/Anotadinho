---
title: Ciclo 178 — Destaque da região focada no nav-mode
type: ciclo
ciclo: "178"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: []
tags:
- ciclo
---

# Ciclo 178 — Destaque da região focada no nav-mode

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Destaque da região focada no nav-mode

## Objetivo

Pedido do usuário, com print: no nível "Regiões" do nav-mode, o EDITOR
é a única região que não dá sinal nenhum de estar selecionada — não dá
pra saber o que se está prestes a abrir com Enter.

Causa: o destaque do ciclo 139 usa `background-color` + `box-shadow:
inset`, e os dois são pintados ATRÁS dos filhos. Header, sidebar e abas
têm espaço sobrando, então algo aparece; o painel do editor é preenchido
por dois filhos de fundo opaco (`.tab-bar` e `.editor`) que cobrem a
caixa inteira. O elemento tinha a classe, o CSS estava lá — e nada
aparecia.

## Critérios de aceite

- [x] A região focada mostra um destaque desenhado POR CIMA do
      conteúdo, não atrás
- [x] Vale pras quatro regiões (header, sidebar, abas, editor)
- [x] Itens de DENTRO das regiões continuam com o destaque de antes
- [x] Nenhum item que já é posicionado (`position: absolute`, como a
      barra do cronograma) tem o `position` mexido
- [x] Cenário no harness medindo o destaque de cada região

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs regiões
```

## Não-objetivos

- Redesenhar o indicador (cor, espessura) — o problema era ele não
  aparecer, não a aparência
- Mexer no `:focus-visible` genérico do ciclo 123

## Notas

`outline` foi considerado e descartado: os filhos também o cobrem. A
solução é um `::after` com `inset: 0` e `z-index`, restrito a
`[data-nav-parent="root"]` — as quatro regiões são contêineres de
posição estática, então criar contexto de posicionamento nelas é
inofensivo. Aplicar isso em TODO item quebraria a barra do cronograma,
que é `position: absolute`.

## Resultado

# Ciclo 178 - done

## Resumo

O editor era a única região do nav-mode sem destaque visível: os filhos
dele têm fundo opaco e cobriam o `background` e o `box-shadow: inset`
do indicador. Agora as regiões de topo ganham um overlay desenhado por
cima.

## Arquivos criados/modificados

- `ui/src/styles/main.css` — `[data-nav-parent="root"].nav-mode__item-active::after`
- `scripts/uitest/cenarios.mjs` — cenário que mede o destaque das 4
  regiões

## Testes adicionados

- Cenário de harness: percorre as regiões e exige overlay > 0 em todas

## Problemas encontrados

- `outline` não resolveria (filhos cobrem também). O overlay precisa de
  contexto de posicionamento, então a regra é restrita às regiões de
  topo: aplicar em todo item quebraria a barra do cronograma, que é
  `position: absolute`.

## Notas para próximos ciclos

- Achado no mesmo dia: 179 (foco perdido ao fechar modal).
