---
title: Código e diagramas na TUI — como
tags:
- proposta
- tui
- mermaid
type: proposta
date: 2026-10-23
spec: '[[Código e diagramas na TUI]]'
status: aprovada
---
# Código e diagramas na TUI — como
{{ type: "fluxo" }}
artefato: proposta
etapa: aprovada
{{ /fluxo }}
Atende `pages/specs/tui-codigo-e-diagramas.md`.

## Abordagem

### Bloco em várias linhas, sem virar várias paradas

Hoje `crates/tui/src/tela.rs` faz uma `Linha` por unidade, e o `corpo()`
troca toda quebra por espaço. A peça que resolve isso já existe: a linha
de **enfeite** (ciclo 293), que ocupa tela e não é destino do cursor. É
com ela que o embed ganhou caixa.

- O bloco de código vira **uma linha de destino** (a borda de cima, com a
linguagem) e **linhas de enfeite** (o miolo e a borda de baixo). O
cursor, a rolagem e a dobra continuam contando por linha visível, e o
cursor pousa uma vez por bloco (RF9).
- O miolo é cortado num limite de linhas (RF4), com a última linha
dizendo quantas faltam. O limite é uma constante, e depende da altura
do terminal só se for barato.
- A linha larga demais é cortada com `…` na hora de desenhar (RF5), que
é quando a largura é conhecida.
- `Linha` ganha os trechos com estilo por linha do miolo; o tipo que já
carrega trechos (ciclo 287) recebe um realce novo por categoria de
token.

### As cores do código: `syntect`

- **`syntect`** com `default-fancy` (regex em Rust puro, sem
oniguruma): compila no Windows (RNF2) e já traz as gramáticas do
Sublime pras linguagens do RF2.
- **As cores não vêm do tema do syntect.** O escopo de cada token
(`keyword`, `string`, `comment`, `constant.numeric`, `entity.name.type`)
vira uma categoria nossa, e a categoria vira cor pelo `tema.rs` da TUI,
nos dois temas. Assim o código combina com o resto da tela.
- **O custo:** as gramáticas embutidas pesam alguns MB no binário, e
carregá-las leva dezenas de ms. Elas carregam **uma vez, na primeira
página com bloco de código**, e não na partida (o mesmo princípio do
ciclo 486 na janela).
- **Alternativa descartada:** um colorizador nosso (palavras-chave,
strings, comentários). Leve, mas cada linguagem nova seria um ciclo, e
string com aspa escapada ou comentário de bloco erram fácil.
- **Alternativa descartada:** `tree-sitter`. Mais preciso, mas uma crate
de gramática por linguagem, com código C a compilar em cada uma, o que
pesa no build do Windows.

### O desenho do diagrama mora no núcleo

- **`crates/core/src/diagrama/texto.rs`:** `desenhar(&Diagrama, largura) -> Result<Vec<Vec<Trecho>>, NaoCabe>`. São linhas de caracteres com a
categoria de cada trecho (caixa, seta, rótulo), e quem desenha pinta.
- Testa-se com `cargo test`, comparando o texto desenhado, sem
terminal.
- Lê o diagrama pelo `diagrama::ler` do construtor (RNF3). Fora do
modelo, a TUI cai no código com `ForaDoModelo::mensagem()` (RF7).


- **Sequência:** colunas de largura do maior nome de participante,
linhas da vida com `│`, cada mensagem uma linha com `──▶` ou `╌╌▶` e o
rótulo em cima. Nota numa caixa. É o caso mais fácil, e vai primeiro.
- **Gantt:** resolve início e fim de cada tarefa (data, duração, `after`)
e desenha barras `█` num eixo de dias, por seção. A régua do eixo
segue a do embed de cronograma da TUI (ciclo 305).
- **Fluxograma e estados:** a arrumação em camadas do canvas
(`ui/.../canvas.rs::completar`, ciclo 490) **desce pro núcleo** e passa
a servir os dois: o canvas e a TUI. Cada camada é uma linha de caixas
com moldura (`┌─┐`), e as ligações são traçadas numa grade de
caracteres:
- entre camadas vizinhas, linha reta ou em cotovelo (`│`, `└─┐`);
- ligação que pula camadas ou que volta (a "voltou" do exemplo) corre
por uma pista lateral, por fora das caixas;
- o rótulo vai no meio do trecho vertical.
- A posição salva pelo canvas (`%% anotadinho-posicoes`) não é usada:
as coordenadas do canvas não cabem numa grade de terminal.


- **Não cabe (RF8):** largura maior que a tela devolve `NaoCabe`, e a
TUI mostra o código com o aviso. Cortar o desenho no meio de uma caixa
confunde mais do que ajuda.

## Etapas (ciclos)

1. **Bloco de código em várias linhas.** Linha de destino mais enfeites,
corte por altura e largura, linguagem no topo. Sem cor ainda. Teste
de unidade em `tela.rs` e conferência num pty.
1. **Cor no código.** `syntect` carregado sob demanda, categorias,
cores do tema. Bateria de escala: página com 200 blocos de código.
`cargo check` no alvo do Windows.
1. **Sequência e gantt desenhados.** `diagrama::texto` no núcleo,
desenho testado por texto, fora do modelo cai no código com o motivo.
1. **Fluxograma e estados desenhados.** A arrumação em camadas desce pro
núcleo (o canvas passa a usá-la de lá, e os cenários 490 e 491
continuam passando), e o traçado das ligações na grade. O diagrama de
exemplo e o do print do ciclo 491 entram como casos de teste.
1. **Entrar no bloco e rolar** (498, pedido de 2026-10-24). O núcleo
desenha na largura que o diagrama pede (`desenhar_inteiro`), e a TUI
recorta numa vista que desliza. Enter entra, `hjkl` e `g G 0 $` rolam,
`i` edita, `Esc` sai. O diagrama largo deixa de cair no código: sai
recortado, com o aviso. Isso muda o RF8 e o RF9 da spec, e a revisão
dela está na fila.
1. **Os outros tipos de mermaid** (499). Um leitor tolerante só pra ver
(`diagrama::so_ver`), separado do leitor estrito do construtor, porque
nada daqui volta pro arquivo. Pizza, mapa mental, linha do tempo,
jornada, classes e ER. Classes e ER reaproveitam o grafo do 497, com
caixa de várias linhas.
1. **O construtor na TUI** (500). Formulários por tipo e prévia em
caracteres ao lado. O texto do código é a fonte da verdade, como na
janela.

## Riscos

- **O traçado do fluxograma é a parte incerta.** Com cruzamentos e
voltas, um desenho em caracteres pode ficar mais confuso que o código.
Se no ciclo 4 o resultado não for legível pros casos de teste, a saída
é desenhar só as caixas em camadas e listar as ligações embaixo
("Pronta → Executar?"). Isso eu trago como decisão antes de fazer, não
troco por conta.
- **Largura do terminal.** Um fluxograma com quatro caixas lado a lado
não cabe em 80 colunas. *(Resolvido no 498: ele sai recortado e se
navega por dentro.)*
- **Caracteres de caixa no Windows.** O terminal do Windows desenha
`┌─┐` e `▶` bem; o console antigo (conhost) pode não desenhar. A TUI já
usa esses caracteres nos embeds, então o risco não é novo.
- **Tamanho do binário.** O `syntect` com as gramáticas soma alguns MB à
TUI. Se pesar, dá pra embutir só as linguagens do RF2.
