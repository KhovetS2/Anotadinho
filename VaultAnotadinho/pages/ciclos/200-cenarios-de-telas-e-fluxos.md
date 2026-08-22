---
title: Ciclo 200 — Cenários das telas e fluxos sem cobertura
type: ciclo
ciclo: "200"
status: concluida
date: 2026-08-21
prioridade: media
depende_de: [198, 199]
tags:
- ciclo
---

# Ciclo 200 — Cenários das telas e fluxos sem cobertura

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Cenários das telas e fluxos sem cobertura

## Objetivo

Fechar as áreas que nenhum arquivo do harness tocava: journals,
páginas-de-TIPO (grafo, tags, assets, kanban — diferentes dos embeds de
mesmo nome), templates, exportação, git, histórico e cheatsheet.

Feito DEPOIS do 198 e do 199 de propósito: os cenários nascem já com
espera por condição, e não herdam as esperas lentas.

## Critérios de aceite

- [x] `scripts/uitest/telas.mjs` com 11 cenários.
- [x] Nenhum tempo fixo de setup.
- [x] Os cenários que CRIAM página no vault apagam o que criaram.
- [x] Suíte inteira verde, e `git status` do vault limpo ao fim.

## Comandos de validação

```bash
node scripts/uitest/run.mjs tela:
node scripts/uitest/run.mjs
```

## Não-objetivos

- Profundidade nessas telas: aqui é "abre e mostra o esperado". Quem
  precisar de detalhe ganha arquivo próprio depois.

## Notas

**"Nova página" é um fluxo de DOIS passos** — escolhe o template antes
de pedir o nome. O cenário original supunha um só e falhava esperando o
campo de título. Aproveitado pra cobrir templates, que era outra lacuna:
o teste confere que os templates do vault aparecem na lista.

**O título exibido é o SLUG**: digitar `__uitest_nova` abre
`uitest-nova`.

**Três comandos CRIAM página**: "Ir pra Hoje", "Ver Tags" e "Ver
Assets". Rodar a suíte sujava o vault com um journal e duas páginas de
índice. O helper `semSujarOVault` guarda o que existia antes e apaga só
o que o cenário criou.

## Resultado

# 200 — Cenários das telas e fluxos

## Estado final da suíte

**96 cenários, todos verdes, em 256s.**

| Arquivo | Cenários | Foco |
|---|---|---|
| `cenarios.mjs` | 28 | regressões nomeadas por ciclo |
| `blocos.mjs` | 23 | navegação, movimentação, transições |
| `interacoes.mjs` | 21 | varredura em largura |
| `digitacao.mjs` | 17 | texto e teclas por modo |
| `telas.mjs` | 11 | telas e fluxos (novo) |
| snapshot | 1 | impressão digital visual dos 9 embeds |

Comparado com três ciclos atrás: 85 cenários em 462s → **96 em 256s**.
Mais cobertura, quase metade do tempo.

## Achados

- **"Nova página" tem dois passos** (template, depois nome) — o cenário
  supunha um. Virou cobertura de templates de quebra.
- **Três comandos criam página no vault** e sujavam o repositório a cada
  execução. `semSujarOVault` apaga só o que o cenário criou.

## Validação

- `node scripts/uitest/run.mjs`: **96/96 em 256.0s**, com
  `git status VaultAnotadinho` limpo ao fim.
- `cargo test --workspace`: 0 falhas; `ui`: 46 testes.
- Tauri: 0 erros.
