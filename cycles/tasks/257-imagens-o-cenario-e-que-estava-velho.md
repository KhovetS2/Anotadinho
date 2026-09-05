---
id: "257"
titulo: "Imagens: o cenário é que estava velho"
status: done
criado: 2026-09-05
autor: agente
prioridade: alta
depende_de: ["118", "226", "242", "245", "246"]
estima_min: 60
---

# 257 — Imagens: o cenário é que estava velho

## O que a bateria pendente dizia

Quatro cenários vermelhos da spec `imagens-coladas-e-arrastadas`:
arrastar não grava no acervo, a arrastada não sobrevive ao recarregar,
colar não deixa referência na nota.

## O que estava acontecendo

Nenhum dos três era bug.

**Arrastar.** Os cenários esperavam gravação imediata. O app abre o
modal de personalização (ciclo 242) e grava na confirmação. Isso não foi
desvio: é a resposta que a própria spec traz na seção "Perguntas em
aberto", escrita à mão por quem pediu a spec. O cenário nasceu do RF1 e
nunca leu a resposta que veio depois, no mesmo arquivo.

**Colar.** O cenário exigia `![](…)` por regex. O app grava
`<figure class="inserted-image">` — escolha do ciclo 226, para carregar
alinhamento, tamanho, proporção e legenda, que a sintaxe curta não
comporta. E o critério de aceite da spec não pede sintaxe: pede
"referência válida, nenhum `blob:` chega ao arquivo". A `<figure>`
atende. O regex é que era mais estreito que o critério.

A prova de que não era bug estava na própria mensagem de falha, que
imprimia o `.md` — e nele, a referência `assets/colado-4.png` gravada
direitinho.

## Critérios de aceite

- [x] Os cenários cobram o critério da spec (referência válida, sem
      `blob:`), não uma sintaxe
- [x] O de arraste dirige o modal, que é o fluxo escolhido
- [x] Cenário novo pro RF4 (soltar um `.txt` não mexe na nota)
- [x] Os cinco migram pra `interacoes.mjs`
- [x] As perguntas em aberto da spec ficam respondidas no arquivo
- [x] `pendentes.mjs` esvazia

## A lição, escrita no `pendentes.mjs`

Um cenário pendente envelhece. Ele nasce de uma spec e passa a valer como
se fosse a spec, mas o produto pode ter respondido a mesma pergunta de
outro jeito no caminho — inclusive respondendo a uma pergunta que a spec
deixou explicitamente em aberto. Antes de tratar um vermelho de lá como
bug, conferir se não é só desatualização.

## Comandos de validação

```bash
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --pendentes
```
