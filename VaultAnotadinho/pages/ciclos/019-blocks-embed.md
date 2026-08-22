---
title: "Ciclo 019 — Blocos embed: PDF, imagem, Mermaid (diagramas)"
type: ciclo
ciclo: "019"
status: concluida
date: 2026-08-05
prioridade: media
depende_de: ["018"]
tags:
- ciclo
---

# Ciclo 019 — Blocos embed: PDF, imagem, Mermaid (diagramas)

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Blocos Embed

## Objetivo

Suporte a blocos especiais inseridos via comando `/`:
- `/pdf` → input file que mostra o PDF
- `/img` → input file que mostra a imagem
- `/mermaid` → bloco de código com preview de diagrama

Assets salvos em `vault/assets/` e referenciados no Markdown.

## Critérios de aceite

- [x] `/pdf` seleciona PDF e mostra embed
- [x] `/img` seleciona imagem e mostra preview
- [x] Arquivos copiados para `vault/assets/`
- [x] Markdown gerado com referência `![alt](assets/file)`
- [x] `/mermaid` renderiza diagrama (Mermaid.js via CDN)
- [x] App continua compilando

## Resultado

## Resumo
Ciclo 019: Blocos embed (/img e /mermaid).
- /img → prompt URL, insere <img> com preview
- /mermaid → prompt código, insere diagrama (Mermaid.js CDN)
- HTML↔Markdown para imagens e mermaid
