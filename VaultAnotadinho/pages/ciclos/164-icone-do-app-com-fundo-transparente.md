---
title: Ciclo 164 — Ícone do app com fundo transparente
type: ciclo
ciclo: "164"
status: concluida
date: 2026-08-20
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 164 — Ícone do app com fundo transparente

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Ícone do app com fundo transparente

## Objetivo

Pedido do usuário, com print do lançador: o ícone aparece com "pontinhas
brancas" nos quatro cantos. A arte é um quadrado de cantos arredondados
que preenche a tela inteira, mas os PNGs eram 100% opacos — os cantos
FORA da moldura arredondada eram branco puro (`#FFFFFF`, alfa 255), e
qualquer fundo que não fosse branco deixava esses triângulos à mostra.

## Critérios de aceite

- [x] Os cantos dos masters (`anotadinho-icon.png`,
      `anotinho-icon-A.png`) ficam com alfa 0
- [x] Só o branco CONECTADO à borda some: o "R" branco, o texto
      "ANOTADINHO" e os pontos claros da arte continuam intactos
- [x] Sem franja clara na curva do canto — a borda anti-serrilhada
      recebe alfa parcial e tem a cor descomposta (inverso de compor
      sobre branco), conferido ampliando o canto sobre magenta
- [x] Conjunto de ícones do Tauri regerado do master limpo
      (`32x32`, `64x64`, `128x128`, `128x128@2x`, `icon.png`,
      `icon.ico`, `icon.icns` — os 6 declarados em `tauri.conf.json`)
- [x] `cargo build --manifest-path src-tauri/Cargo.toml` OK

## Comandos de validação

```bash
cargo tauri icon anotadinho-icon.png --output src-tauri/icons
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Redesenhar o ícone (a arte é a mesma, só o fundo saiu)
- Gerar os conjuntos de Android/iOS/Windows Store que o
  `cargo tauri icon` cria por padrão — o projeto não empacota pra
  nenhum dos três, e `tauri.conf.json` só declara 6 arquivos
- Trocar qual das duas artes é o ícone oficial

## Notas

O recorte não assume a geometria do canto (círculo? squircle?): parte
dos 4 cantos com um flood fill sobre o branco. Medindo, a curva não
batia nem com círculo nem com superelipse n=4/5, então qualquer máscara
analítica deixaria erro de um ou dois pixels — o flood fill segue a
arte, seja ela qual for.

O anti-serrilhado exigiu o passo extra: os pixels da curva já vinham
misturados com o branco do fundo, então alfa 0/1 deixaria uma franja
clara. Num anel de 2px o alfa sai do quanto o pixel puxa pro branco e a
cor é descomposta — `C = (C_obs - (1-a)·255) / a`.

Script do recorte: `/tmp/.../transparentize.py` (não versionado; a
operação é de uma vez só, e o resultado é que importa).

O ícone do lançador do sistema só muda depois de um
`./scripts/build.sh` + reinstalar o pacote — o que está instalado hoje
foi gerado antes desta correção.

## Resultado

# Ciclo 164 - done

## Resumo

Os cantos brancos do ícone (visíveis como "pontinhas" no lançador)
viraram transparência. A arte é a mesma; o que saiu foi o branco opaco
que ficava FORA da moldura arredondada.

## Arquivos criados/modificados

- `anotadinho-icon.png`, `anotinho-icon-A.png` — masters com alfa nos
  cantos (os originais opacos seguem no histórico do git)
- `src-tauri/icons/{32x32,64x64,128x128,128x128@2x,icon}.png`,
  `icon.ico`, `icon.icns` — regerados do master limpo

## Testes adicionados

- Nenhum: mudança de asset. Conferência foi por inspeção do alfa
  (cantos = alfa 0 em todos os tamanhos) e visual, compondo o ícone
  sobre magenta/branco/escuro.

## Problemas encontrados

- A curva do canto não bate com círculo nem com superelipse n=4/5 (medi
  os três) — máscara analítica erraria por 1-2px. Flood fill a partir
  dos cantos resolve sem assumir geometria.
- Recorte binário deixava franja clara na curva anti-serrilhada;
  resolvido com alfa parcial + descomposição da cor num anel de 2px.
- `cargo tauri icon` gera também Android/iOS/Windows Store; removidos,
  porque `tauri.conf.json` declara só 6 arquivos e o projeto não
  empacota pra nenhuma das três plataformas.

## Notas para próximos ciclos

- O ícone do lançador só troca depois de `./scripts/build.sh` +
  reinstalar: o instalado hoje é de antes desta correção.
