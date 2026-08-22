---
title: Ciclo 187 — Snapshot visual dos embeds no harness
type: ciclo
ciclo: "187"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: [177]
tags:
- ciclo
---

# Ciclo 187 — Snapshot visual dos embeds no harness

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Snapshot visual dos embeds no harness

## Objetivo

Os 19 cenários do harness testam COMPORTAMENTO. Nada pega regressão
visual — CSS de um embed vazando pro outro, badge sem contraste, grade
quebrada — que é justamente o que só aparece abrindo a página. Este
ciclo acrescenta um passo de captura por tipo de embed, comparado com
uma baseline versionada.

## Critérios de aceite

- [x] `scripts/uitest/snapshot.mjs`: monta uma página com um embed de
      cada tipo e compara com `scripts/uitest/baseline/<tipo>.json`.
- [x] Comparação **não é de pixel** — ver a nota de mudança de rumo
      abaixo. É por classe → estilos computados + contagem.
- [x] `--atualizar` regrava a baseline; sem a flag, diferença reprova e o
      relatório diz qual propriedade mudou.
- [x] Roda dentro de `run.mjs` como um cenário a mais (`--sem-snapshot`
      pula), e pode rodar sozinho, com filtro por tipo.
- [x] Baseline dos 9 tipos versionada (~100KB de JSON no total).
- [x] Provado que reprova de verdade: com um CSS injetado mudando cor e
      raio do callout, o relatório apontou as duas propriedades.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
node scripts/uitest/snapshot.mjs --atualizar
```

## Não-objetivos

- Snapshot do app inteiro (chrome, sidebar) — o alvo é o embed, que é o
  que muda com frequência.
- Testar temas claro/escuro em separado neste ciclo.

## Notas

**Mudança de rumo, decidida durante a execução:** não dá pra capturar
pixel nesta plataforma. O `capture_native_screenshot` do bridge responde
`Native Linux screenshot not yet implemented`, e o harness fala com o app
só por esse canal.

A impressão digital que entrou no lugar pega a mesma classe de regressão
(CSS vazando entre embeds, cor sem contraste, grade quebrada, caixa
colapsada) e, na prática, é melhor pro que este repositório precisa: o
relatório diz QUAL propriedade mudou em vez de "3,2% dos pixels
diferem", não reprova por antialiasing, e a baseline é JSON de poucos KB
— não PNG, que já pesou 10MB aqui uma vez (ciclo 148).

O que ela NÃO pega, e vale saber: sobreposição de elementos, imagem
errada e fonte trocada por outra de mesma métrica.

Duas decisões de estabilidade: a janela é fixada em 1280×900 antes de
medir, e as datas da fixture são relativas ao mês corrente (data cravada
tiraria a barra do cronograma da área visível conforme o tempo passa).
Calendário e cronograma têm contagem de elementos variável por mês, então
neles só o estilo reprova.

## Resultado

# 187 — Snapshot visual dos embeds

## Mudança de rumo

A task pedia diff de pixel contra PNGs versionados. Não é possível aqui:
o bridge MCP é o único canal do harness com o app, e o
`capture_native_screenshot` dele responde `Native Linux screenshot not
yet implemented`.

O que entrou no lugar é uma impressão digital do DOM ao vivo: por embed,
cada combinação de classes que aparece na subárvore vira um registro com
os estilos computados que decidem o visual + quantos elementos têm
aquela combinação. Comparar por CLASSE (não por posição no DOM) é o que
deixa estável — reordenar dois cards não é regressão, mudar a cor deles é.

## O que mudou

- `scripts/uitest/snapshot.mjs` (novo): fixture com um embed de cada um
  dos 9 tipos, coleta, comparação e CLI (`--atualizar`, filtro por tipo).
- `scripts/uitest/bridge.mjs`: `comando()` cru e `redimensionar()` — a
  janela é fixada em 1280×900 antes de medir.
- `scripts/uitest/run.mjs`: o snapshot roda como cenário a mais;
  `--sem-snapshot` pula.
- `scripts/uitest/baseline/*.json`: 9 baselines, ~100KB no total.

## Validação

- `node scripts/uitest/snapshot.mjs --atualizar` grava; rodar de novo sem
  a flag passa nos 9 — a impressão digital é estável entre execuções.
- **Prova de que reprova:** com um CSS injetado mudando
  `background-color` e `border-radius` do callout, o relatório apontou
  exatamente as duas propriedades, com valor antigo e novo.
- `node scripts/uitest/run.mjs`: **21/21 em 116.6s**.
