---
id: "264"
titulo: "Copiar e apagar um embed pelo vim"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["263"]
estima_min: 150
---

# 264 — Copiar e apagar embed

## Objetivo

O RF2 da spec: `yy` copia o embed, `dd` o apaga. Com a confirmação
decidida — apagar um embed inteiro pergunta antes.

## A correção estrutural que veio junto

O ciclo 263 expôs que o `on_keydown` do vim morava em CADA
`div.editor__wysiwyg`, e o embed é irmão desses divs — nenhuma tecla
com embed em foco chegava ao vim. Aquele ciclo remendou o `j` pelo
handler do contêiner; este arruma a causa: **um handler de teclado pro
documento**, no contêiner de segmentos.

Subir funciona porque `on_keydown` não consulta o alvo do evento — ele
decide pelo foco e pela seleção. Os outros handlers (`oninput`,
`onpaste`, `ondrop`) ficaram por segmento porque dependem do
`data-segment-index`: colar precisa saber ONDE; uma tecla não.

## Critérios de aceite

- [x] Um handler de teclado por documento, não por segmento
- [x] `yy` num embed copia o MARKDOWN dele, buscado no `content_md`
- [x] `dd` num embed pergunta antes de apagar
- [x] Confirmar apaga o segmento e preserva os vizinhos
- [x] Cenários que provam os dois

## Por que o markdown vem do `content_md`

O DOM tem a tabela DESENHADA, não o `{{ type: "table" }}` que a gerou.
Serializar da tela devolveria HTML achatado. O cenário confere as duas
metades: que veio o fence e que NÃO veio HTML.

## Comandos de validação

```bash
cd ui && cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
```
