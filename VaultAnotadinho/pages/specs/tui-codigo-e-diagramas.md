---
title: Código e diagramas na TUI
tags:
- spec
- tui
- mermaid
type: spec
date: 2026-10-23
prioridade: media
status: aprovada
---
# Código e diagramas na TUI

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada
{{ /fluxo }}

## Contexto

Na janela, um bloco de código aparece com as linhas, a indentação e as
cores da linguagem, e um bloco ` ```mermaid ` aparece desenhado (ciclo
485). Na TUI, os dois viram **uma linha só**: as quebras viram espaço, a
indentação some, e o que se lê é "```" seguido do código inteiro
emendado. Um diagrama na TUI é ilegível, e um trecho de código com mais
de três linhas também.

Esta spec trata só de **ver**. Editar continua como hoje: o bloco de
código abre no modal de texto.

## Requisitos funcionais

### A. Bloco de código

- **RF1.** O bloco de código aparece com as suas linhas e a indentação,
  numa caixa com a linguagem no topo (quando a cerca diz qual é).
- **RF2.** As linguagens comuns aparecem com cor: palavra-chave, string,
  comentário, número e tipo. As cores vêm do tema da TUI, claro e
  escuro. A lista mínima: Rust, JavaScript/TypeScript, Python, shell,
  JSON, YAML, TOML, HTML, CSS, SQL e Markdown.
- **RF3.** Linguagem desconhecida, ou sem linguagem, aparece sem cor, mas
  com as linhas e a indentação.
- **RF4.** Um bloco mais alto que um limite aparece cortado, com a
  contagem do que falta ("… mais 40 linhas"). Dobrar e desdobrar o bloco
  segue a regra de dobra que a TUI já tem.
- **RF5.** Linha mais larga que a tela é cortada no fim com uma marca, e
  não quebra: código quebrado em duas linhas parece duas instruções.

### B. Diagrama mermaid

- **RF6.** Um bloco mermaid dentro do modelo do construtor (fluxograma,
  sequência, estados e gantt) aparece **desenhado em caracteres**:
  - **sequência:** uma coluna por participante, linha da vida, mensagens
    em ordem com seta e rótulo, e notas;
  - **gantt:** as tarefas como barras num eixo de datas, por seção;
  - **fluxograma e estados:** caixas com o texto, ligadas por linhas com
    seta e rótulo, na direção do diagrama.
- **RF7.** Um bloco fora do modelo (outro tipo, subgrafo, estilos) aparece
  como código (RF1–RF3), com uma linha dizendo o motivo, o mesmo que o
  construtor mostra.
- **RF8.** Um diagrama mais largo que a tela não sai torto: ele sai
  desenhado e recortado na borda, com o aviso de quantas colunas tem e de
  como ver o resto (RF10). Nunca uma linha que dobra no meio do desenho.
  *(Revisto no ciclo 498: antes, caía no código com "não cabe".)*
- **RF8a.** Pra VER, comentário (`%%`), diretiva (`%%{…}%%`) e o
  cabeçalho `---` do mermaid não impedem o desenho; o título do cabeçalho
  vai pro topo. *(Ciclo 499.)*
- **RF8b.** Os tipos que o construtor não monta, mas que a janela desenha,
  também aparecem desenhados: pizza (barras com porcentagem), mapa mental
  (árvore), linha do tempo, jornada (nota em estrelas), classes (caixa com
  membros e relações) e entidade-relação (atributos e cardinalidade). O
  leitor deles é tolerante e só serve pra ver: uma linha que ele não
  entende fica fora do desenho. Tipo sem desenho diz qual é. *(Ciclo 499.)*

### C. Navegação

- **RF9.** O cursor pousa no bloco como hoje, uma parada por bloco: as
  linhas do código e do desenho não viram paradas. `i`/`A` num bloco de
  código abrem o modal de edição. *(Revisto no ciclo 498: Enter passou a
  ENTRAR no bloco, RF10; Enter de novo, lá dentro, edita.)*
- **RF10.** Enter num bloco de código ou diagrama entra nele: a moldura
  acende, o topo diz onde se está (linhas e colunas à vista de quantas
  há), e as teclas rolam a vista nas duas direções (`hjkl`, setas, meia
  tela, `g G 0 $`) sem passar dos limites. `Esc` sai. Dentro, o código
  não é cortado com marca: a vista desliza pela linha inteira.
  *(Pedido da pessoa em 2026-10-24; ciclo 498.)*
- **RF11.** O construtor de diagramas da janela tem par na TUI:
  formulários por tipo, prévia desenhada em caracteres ao lado e a aba
  Código; abrir e aplicar sem mexer devolve o código igual. *(Ciclo 500.)*

## Requisitos não funcionais

- **RNF1.** O arquivo não muda: tudo isso é só desenho.
- **RNF2.** A TUI continua funcionando no Windows (ciclo 448): nada de
  dependência que não compile lá.
- **RNF3.** O desenho do diagrama sai do mesmo modelo do construtor
  (`anotadinho_core::diagrama`), e não de um segundo leitor de mermaid.
  O que o construtor entende é o que a TUI desenha.
- **RNF4.** Abrir uma página com muitos blocos de código não fica
  perceptivelmente mais lento. A bateria de escala ganha um caso pra
  isso.

## Critérios de aceite

- [ ] Um bloco Rust de 20 linhas aparece em 20 linhas, com indentação e
      cor, numa caixa com "rust" no topo.
- [ ] Um bloco sem linguagem aparece em várias linhas, sem cor.
- [ ] Um bloco de 200 linhas aparece cortado, com a contagem do resto.
- [ ] O diagrama de exemplo (`pages/exemplos/diagrama-mermaid.md`) e um
      de cada um dos quatro tipos aparecem desenhados na TUI.
- [ ] Um diagrama com `subgraph` aparece como código, com o motivo.
- [ ] O cursor pula o bloco inteiro de uma vez, e Enter num bloco de
      código abre o modal.
- [ ] Nada disso muda o arquivo.
- [ ] Conferido num pty, com teclas dirigidas por script, como manda o
      AGENTS.md pra TUI.

## Fora de escopo

- Desenhar o diagrama como imagem (kitty, sixel): exige o mermaid de
  verdade, que é JavaScript, e um navegador sem janela.
- Os tipos de mermaid que nem RF8b cobre (gitGraph, quadrant, xychart,
  sankey, C4, requirement, block, packet, architecture): aparecem como
  código, com o motivo.
- Números de linha no bloco de código.

## Ambiguidade apontada

O pedido foi "diagramas mermaid, trechos de código **e etc.**". Esta spec
lê o "etc." como nada além dos dois. Outras coisas que a janela mostra e a
TUI não — **imagens** (dá pra mostrar nos terminais que aceitam imagem e
cair no texto alternativo nos outros), **tabelas** fora de embed e
**fórmulas** — ficam de fora até alguém dizer quais entram. Se entrarem,
é aqui que mudam os requisitos, antes da proposta valer.
