---
id: "238"
titulo: "A barra de formatação não aparecia em página com embed"
status: done
criado: 2026-08-29
autor: agente
prioridade: alta
depende_de: ["234"]
estima_min: 30
---

# 238 — A barra não aparecia em página com embed

## O bug

Reportado com print: texto selecionado numa spec, nenhuma barra na tela.

Numa página com embed o editor não tem raiz única — cada segmento de
markdown vira seu próprio `.editor__wysiwyg`, e o `editor_ref` não é
fixado em lugar nenhum. `medir_selecao` pedia essa raiz, não achava, e
desistia. Ou seja: a barra não funcionava em **nenhuma** página com embed,
que é onde mora quase todo o conteúdo do vault.

Meus três cenários do ciclo 234 usaram página sem embed — o outro caminho
de render. Passaram todos.

## Critérios de aceite

- [x] A barra aparece em página com embed
- [x] Marcar texto num segmento não danifica o embed vizinho
- [x] O cenário usa o evento `selectionchange` nativo, sem disparar na mão
- [x] Campos de texto dentro de embeds também ganham a barra

## Comandos de validação

```bash
cd ui && trunk build
node scripts/uitest/run.mjs
```
