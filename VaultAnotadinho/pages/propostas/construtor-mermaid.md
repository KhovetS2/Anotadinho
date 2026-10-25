---
title: Construtor de diagramas mermaid — como
tags:
- proposta
- editor
- mermaid
type: proposta
date: 2026-10-23
spec: '[[Construtor de diagramas mermaid]]'
status: aprovada
---
# Construtor de diagramas mermaid — como

{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
{{ /fluxo }}

Atende `pages/specs/construtor-mermaid.md`.

## Abordagem

### O modelo mora no núcleo (`crates/core/src/diagrama/`)

- **O modelo:** um tipo por diagrama (`Fluxograma`, `Sequencia`,
  `Estados`, `Gantt`), com `ler(codigo) -> Result<Diagrama, ForaDoModelo>`
  e `escrever(&Diagrama) -> String`.
- **Por que no núcleo:** o round-trip é o risco principal do pedido
  (RF8, RNF1), e lá ele se testa com `cargo test`, sem DOM.
- **`ForaDoModelo` carrega o motivo** que o modal mostra (RF7). Por
  exemplo: "subgrafo na linha 4" ou "tipo `classDiagram`".
- **O leitor é estrito de propósito:** o que ele não reconhece vira
  `ForaDoModelo`, e não é ignorado. Ignorar seria apagar no "Aplicar".
- **Teste de propriedade:** `escrever(ler(x)) == x` para o que o próprio
  construtor gera. Para código escrito à mão dentro do modelo, o teste
  confere a equivalência por `ler`, que é a garantia que importa (o
  espaçamento pode normalizar).
- **As posições do canvas (RNF2)** vão num comentário
  `%% anotadinho-posicoes: {"A":[x,y],...}`, serializado com
  `serde_json`. O leitor aceita o código sem esse comentário.

### A UI

- **`ui/src/components/construtor_mermaid/`** traz o modal (abas
  Construtor e Código), a prévia e um editor por tipo:
  - **a prévia** usa o mesmo caminho do 485: `mermaid.render` e a imagem.
    Ela redesenha com atraso curto depois de cada mudança e mostra o
    erro, como no bloco;
  - **o canvas** (fluxograma e estados) é SVG próprio: nós como
    retângulos arrastáveis por ponteiro e ligação puxada de uma alça do
    nó. Não usa biblioteca nova. Pelo teclado, Tab seleciona os nós, `L`
    liga ao próximo selecionado e as setas movem o nó;
  - **o formulário** (sequência e gantt) são listas com adicionar, subir,
    descer e apagar, e campos por item.
- **A ponte** entre o modal (WASM) e o núcleo: o núcleo é Rust e a UI é
  Rust, então o modelo é usado direto, sem comando Tauri.
- **`/mermaid`** troca o `PendingDialog::Prompt` pelo modal novo.
- **O botão "Editar no construtor"** aparece no bloco mermaid quando ele
  está em edição (classe `--editando` do 485). Ele fica fora do
  `contenteditable`, num elemento flutuante da página e não do bloco,
  pelo mesmo motivo do `::after`: não pode entrar no `text_content` nem
  na fila de irmãos. "Aplicar" troca o `<code>` daquele `<pre>` e passa
  por `mark_edited_estrutural`, então o Ctrl+Z desfaz a troca inteira.
- **As ligaduras (RF9):** `font-variant-ligatures: none` no bloco
  mermaid e na aba Código.

## Etapas (ciclos)

1. **487 — núcleo.** O modelo, o leitor e o escritor dos quatro tipos,
   `ForaDoModelo` com o motivo, as posições em comentário e os testes de
   round-trip. As ligaduras saem do bloco, uma linha de CSS que vai junto.
2. **488 — modal, código e prévia.** O `/mermaid` abre o modal, com a aba
   Código e a prévia ao vivo. "Editar no construtor" abre o bloco
   existente, e um diagrama fora do modelo abre no código. Cenário de UI
   para inserir, editar, aplicar e o round-trip do arquivo.
3. **489 — formulário de sequência e gantt.** Cenário de UI que monta
   um de cada e confere a prévia sem erro.
4. **490 — canvas de fluxograma e estados.** Arrastar, ligar, painel
   lateral e o caminho por teclado. O cenário arrasta e liga por eventos
   de ponteiro sintéticos. **Isso não prova o gesto de verdade** (padrão
   "fronteira-do-sistema"), então o arraste fica para você conferir à
   mão.

## Alternativas descartadas

- **Biblioteca de canvas de diagramas (por exemplo, um editor visual
  pronto):**
  - seria mais um script pesado, depois de o 486 tirar 51 MiB do
    repouso;
  - o formato dela não é mermaid, e a conversão seria outra fonte de
    perda.
- **Guardar o diagrama num formato próprio e gerar o mermaid a partir
  dele:** viola o RNF1. O arquivo deixaria de ser legível por qualquer
  outro programa que entende mermaid.
- **Canvas para os quatro tipos:** a posição numa sequência ou num gantt
  não significa nada, porque a ordem é que significa (ver a ambiguidade
  na spec).

## Riscos

- **Round-trip de código escrito à mão:** a normalização do espaçamento
  pode criar diff num "Aplicar" sem mudança. Mitigação: se nada mudou no
  modelo, "Aplicar" devolve o texto original, sem regenerar.
- **O desenho do canvas e o do mermaid são diferentes:** o canvas mostra
  a posição que a pessoa escolheu, e o mermaid desenha o layout dele.
  A prévia ao lado deixa isso visível.
- **O tamanho do ciclo 490:** o canvas é a maior parte do trabalho. Se
  passar do razoável, ele se divide em arrastar e ligar, num ciclo, e o
  painel lateral e o teclado, noutro.
