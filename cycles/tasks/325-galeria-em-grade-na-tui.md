---
id: "325"
titulo: "A galeria em grade na TUI"
status: done
criado: 2026-09-17
autor: agente
prioridade: media
depende_de: ["304"]
estima_min: 60
---

# 325 — A galeria em grade na TUI

Antes cada miniatura era um selo com a legenda, um embaixo do outro — nada
da grade da janela. Agora, como `.gallery`:

- Cabeçalho: "N imagens" à esquerda; à direita o seletor `P M G` com o
  tamanho ativo cheio na cor de destaque, e "N col".
- A grade: uma fileira por linha, `columns` miniaturas lado a lado. Cada
  uma é o QUADRO da imagem (fundo da página, borda de meio-bloco) na altura
  do tamanho (P, M, G), com o nome do arquivo no meio — o que a janela
  mostra quando a imagem falta — e a legenda embaixo ("Legenda" apagada
  quando não há). Sob o cursor, a borda e a legenda acendem.
- `h`/`l` andam na linha; `j`/`k` vão pra mesma coluna da linha de
  baixo/cima (nas pontas, saem da grade como sempre).
- Galeria vazia: o aviso da janela.

No núcleo, a galeria virou `cabecalho` (com `tamanho` e `colunas`
escondidos) + fileiras `fotos` de `miniatura` (com `indice` e `caminho`).
