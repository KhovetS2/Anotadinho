---
id: "452"
titulo: "Revisão na janela: a Visualização é a página final, e o diff quebra a linha"
status: done
criado: 2026-10-07
autor: agente
prioridade: alta
depende_de: ["451"]
estima_min: 120
---

# 452 — A Visualização é a tela final

Spec: `pages/specs/revisao-de-proposta.md`. Proposta:
`pages/propostas/revisao-de-proposta.md`. As duas estão na fila de
revisão. A execução começou a pedido direto da pessoa.

## Objetivo

A Visualização da revisão usava o renderizador de markdown fora da
árvore do editor e sem os passos que o editor aplica depois do render.
O resultado não era a página que ia ficar. Além disso, ela ignorava os
trechos tirados, e o diff não quebrava a linha longa.

## Critérios de aceite

- [x] `editor::preparar_leitura` é o pipeline do editor sem a edição:
      render, imagens do vault, transclusões, `editor__bloco`, mermaid e
      realce de código
- [x] `PaginaPreview` monta `.editor__wysiwyg-segments` >
      `.editor__wysiwyg`. Título, parágrafo, lista, item, negrito e código
      têm o estilo calculado do editor (harness)
- [x] Nenhum bloco `contenteditable="true"` na Visualização (harness)
- [x] Com um trecho tirado, a Visualização mostra a página sem ele
      (harness)
- [x] A linha longa do diff quebra, sem rolagem horizontal (harness)
- [x] O cenário reprova com cada peça antiga: diff (rolava 354px),
      árvore do preview e conteúdo parcial

## Comandos de validação

```bash
node scripts/uitest/run.mjs "estilo da página"
node scripts/uitest/run.mjs "revisão"
node scripts/uitest/run.mjs
```

## Não-objetivos

- Marcar na Visualização o que mudou (ambiguidade apontada na spec)
- TUI: fica no ciclo 453
