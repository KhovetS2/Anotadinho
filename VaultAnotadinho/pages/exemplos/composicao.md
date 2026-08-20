---
title: Composição — destaque, colunas e galeria
tags: [demo, embed]
---
# Compondo uma página

Os três embeds desta página não guardam registros: eles servem pra
MONTAR a nota — o que o markdown puro, sendo linear, não dá.

## Destaque (`callout`)

Cinco variantes, cada uma com cor e ícone próprios. O corpo é markdown
de verdade: aceita lista, código, ênfase, `[[wikilink]]`.

{{ type: "callout" }}
variant: info
title: Para que serve
body: |
  Contexto que o leitor precisa antes de continuar — sem virar mais um
  parágrafo perdido no meio do texto.

{{ /callout }}

{{ type: "callout" }}
variant: warning
title: Cuidado conhecido
body: |
  Editar o `.md` por fora com o app aberto é seguro: sem edição
  pendente a página recarrega sozinha (ciclo 173); com edição pendente
  aparece uma barra pra você ver a diferença e escolher entre manter o
  seu e recarregar (ciclo 190).

{{ /callout }}

{{ type: "callout" }}
variant: tip
title: Atalho
body: |
  No modo de navegação, `n` com um bloco focado abre um bloco novo
  já com o menu `/` — dá pra montar a página inteira sem mouse.

{{ /callout }}

## Colunas (`columns`)

Larguras em unidades de fração (`1fr`, `2fr`...), ajustáveis pelos
botões que aparecem no hover de cada painel. Abaixo de 700px empilha
sozinho.

{{ type: "columns" }}
columns:
- width: 2
  body: |
    ### O que cabe aqui

    Texto longo, tabela, citação — cada painel é markdown independente.
    Útil pra "antes e depois", "prós e contras", ou uma referência ao
    lado da explicação.
- width: 1
  body: |
    ### Dica

    Se um painel ficar apertado, use os botões `‹` e `›` pra mudar a
    proporção em vez de encolher o texto.

{{ /columns }}

## Galeria (`gallery`)

Imagens de `assets/` numa grade, com legenda e lightbox no clique.
Mudam de tamanho (P/M/G) e de número de colunas pela própria barra.

{{ type: "gallery" }}
columns: 3
size: lg
items:
- path: assets/exemplo-grafo.png
  caption: 'Grafo: conexões entre páginas'
- path: assets/exemplo-fluxo.png
  caption: 'Fluxo: da spec ao commit'
- path: assets/exemplo-quadro.png
  caption: 'Quadro: o que está em andamento'

{{ /gallery }}
