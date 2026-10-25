---
title: Proposta de implementação — revisar proposta vendo a mudança e a página final
tags:
- proposta
- propostas
type: proposta
date: 2026-10-07
status: aprovada
---
# Proposta de implementação — revisão de proposta

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
origem: pages/specs/revisao-de-proposta.md
{{ /fluxo }}

## Abordagem

### Janela (RF1, RF2, RF3, RF5)

A Visualização passa pelo MESMO caminho do editor, em vez de imitá-lo.

- Do editor sai uma função `preparar_leitura(div, vault, pagina)`, que
  faz o que o editor faz depois do `set_inner_html`: imagens do vault,
  transclusões, classe `editor__bloco` nos blocos (com a mesma regra da
  lista como grupo), realce e mermaid. Ela deixa de fora o que é edição:
  `contenteditable`, `tabindex` e os atributos de navegação. Por isso
  nenhuma ferramenta de edição acorda: a barra de seleção exige
  `contenteditable="true"` dentro de `.editor`.
- O `PaginaPreview` monta a mesma árvore do editor:
  `.editor__wysiwyg-segments` com um `.editor__wysiwyg` por trecho de
  markdown. É isso que aplica as ~60 regras de tipografia sem copiar
  nenhuma. O CSS tira do contêiner só o que é de painel (padding,
  rolagem, `flex`).
- A Visualização recebe o conteúdo PARCIAL (`aplicar_trechos` com os
  escolhidos), o mesmo que o botão "Aplicar N de M" grava.
- `.propostas__l` passa a `white-space: pre-wrap` com
  `overflow-wrap: anywhere`.

### TUI (RF2, RF3, RF4)

- Cada linha do diff é quebrada na largura do cartão. A continuação
  mantém o fundo da linha e o realce das palavras, e leva um recuo no
  lugar do sinal, pra não parecer outra linha do diff.
- A Visualização recebe o conteúdo parcial, como na janela.
- O cabeçalho quebra o resumo e os modos em duas linhas quando não
  cabem em uma. A linha de atalhos quebra.

## Alternativas descartadas

- **Copiar o CSS do editor pra `.pagina-preview`.** Seriam ~60 regras
  em dobro, e elas divergiriam no primeiro ciclo que mexesse em uma só.
  É o que já aconteceu: a Visualização nasceu certa no 230, e o editor
  mudou depois.
- **Abrir o editor de verdade em modo leitura.** O editor carrega
  estado, atalhos e salvamento. Desligar tudo isso é mais arriscado do
  que reusar só o caminho de render.

## Riscos

- Código global que procura `.editor__wysiwyg` no DOM pode passar a
  achar o preview. A varredura achou cinco lugares no `editor.rs`, todos
  ativos só com o editor montado ou com uma seleção dentro de
  `contenteditable`. Um cenário de harness confere que selecionar texto
  na Visualização não abre a barra de formatação.
- A Visualização pode ficar MAIS alta que antes (o espaçamento do
  editor é maior). A caixa já rola.

## Etapas

1. Janela: `preparar_leitura`, `PaginaPreview`, conteúdo parcial e CSS
   do diff, com cenários de harness. É o ciclo 452.
2. TUI: quebra do diff, conteúdo parcial e cabeçalho, com testes de
   desenho. É o ciclo 453.
