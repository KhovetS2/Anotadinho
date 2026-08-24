---
title: Escrita no vault
date: 2026-08-24
dominio: dados
tags:
- padrao
---
# Escrita no vault

## Quando se aplica

Qualquer código que grave `.md` no vault: editor, conversa, backend,
CLI, agente.

## As regras

1. **Um escritor por conteúdo.** A tela grava a pergunta, o backend
   grava a resposta, e nunca os dois no mesmo trecho. Onde dois
   escritores existiram, um apagou o outro.
2. **Nunca monte YAML na mão.** `serde_yaml` derive, sempre. O embed
   sai por `EmbedData::to_fence_text()` + `embed::join`.
3. **Escalar de frontmatter passa por `escapar_escalar_yaml`.** Um
   título com `: ` no meio derruba o frontmatter INTEIRO, em silêncio.
4. **Gravar vazio por cima de página com conteúdo é recusado.** Apagar
   uma nota inteira nunca é o resultado certo de um save.
5. **Agente propõe, não grava.** `anotadinho-cli propor` devolve um
   diff pra revisão humana.
6. **Parse tolerante:** um campo com tipo errado descarta só aquele
   campo, nunca o documento.

## Por que existe

- **064** — YAML montado à mão corrompeu embed.
- **206** — 25 de 168 ciclos migrados sumiram das consultas: o título
  tinha `: ` e o frontmatter inteiro virou inválido.
- **209** — dois escritores no mesmo arquivo; o último apagava o outro.
- **215** — duas propostas ficaram com 0 bytes. A causa nunca foi
  reproduzida; a trava impede o resultado.
- **215** — `unwrap_or_default()` no parse trocava um campo ruim pelo
  documento em branco, e o save seguinte gravava o branco por cima.
