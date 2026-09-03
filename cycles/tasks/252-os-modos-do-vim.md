---
id: "252"
titulo: "Os modos do vim, e o vim que acompanha o editor por bloco"
status: done
criado: 2026-09-03
autor: humano
prioridade: alta
depende_de: ["250", "251"]
estima_min: 210
agente_alvo: claude-opus
---

# Os modos do vim, e o vim que acompanha o editor por bloco

## Objetivo

Fecha a segunda metade da spec [[Navegação por teclado consistente e modo
vim completo]]: RF4 a RF7 e o RNF1. A primeira metade (o bug da pilha)
saiu no ciclo 250; a seleção de blocos, que a spec diz ser pré-requisito
do modo visual, saiu no 251.

## O que a spec descreve, e o que se achou medindo

A spec diz que o vim "foi feito muitos ciclos atrás e ficou para trás: só
existe o par normal/inserção, e mesmo esse não acompanha o editor por
bloco (ciclo 175)". A segunda parte era mais concreta do que parecia:

Medido na janela, no modo normal, com o cursor na coluna 2 de um
parágrafo — `j` levava o cursor pro **fim daquele parágrafo** e parava ali
para sempre. `Selection.modify` não sai do host de edição, e desde o
ciclo 175 cada bloco é seu próprio `contenteditable`. Na prática o modo
normal não conseguia percorrer a página.

E o RNF1 estava quebrado do outro lado: com o vim ligado, o ramo do vim
rodava **antes** dos atalhos de bloco e engolia `hjkl`, `d`, `y`, `v` —
o modo de navegação virava letra morta justamente pra quem usa vim.

## Decisões

**`vim_insert: bool` vira `VimModo`.** Um booleano não comporta cinco
estados sem virar uma coleção de flags que podem se contradizer.

**Três visuais, e a diferença entre eles é o que conta como unidade:**

| modo | tecla | unidade |
|---|---|---|
| Visual | `v` | caractere, dentro do bloco |
| Visual linha | `V` | o bloco inteiro (reusa o ciclo 251) |
| Visual bloco | `Ctrl+V` | retângulo: as mesmas colunas em vários blocos |

O retângulo é **retângulo de verdade**: âncora e foco guardam bloco E
coluna, e copiar/apagar agem sobre a fatia `[coluna_ini, coluna_fim)` de
cada bloco no intervalo. Foi possível sem o motor de seleção próprio que
a spec irmã pôs como não-objetivo porque ele não precisa de um REALCE
retangular, precisa de uma OPERAÇÃO retangular. As coordenadas são
offsets de caractere no bloco — a mesma régua que a barra de formatação
usa desde o ciclo 244.

O realce do visual bloco é por bloco, não pinta as colunas. Pintar
exigiria desenhar a seleção à mão, que é o motor declarado não-objetivo.
A pessoa vê quais blocos estão em jogo; a coluna ela vê pelo cursor.

**Visual comum não atravessa blocos.** É a limitação do ciclo 175, e a
própria spec previu: "ou a visual nasce só funcionando dentro de um
bloco". Pra atravessar existe `V`, que trabalha com blocos inteiros.

**`/` abre a paleta de comandos que o app já tem.** Inventar uma segunda
caixa de comando, com vocabulário próprio, seria um produto pior.

**`Alt+N` é a porta pra navegação.** Com o vim ligado, Escape pertence ao
vim (Inserção→Normal) e nunca chegava no caminho que entra em navegação:
quem usa vim ficava sem porta. Lido pelo `code` e não pelo `key`, porque
com Alt o layout pode devolver outro caractere.

## Critérios de aceite

- [x] Os quatro modos principais existem (cinco, com visual linha)
- [x] Visual seleciona por caractere; visual em bloco por retângulo, e
      copiar produz o recorte por coluna
- [x] `/` fora da edição abre a busca de comando
- [x] Com o vim ligado há atalho dedicado pro modo de navegação, e ele
      está no cheatsheet
- [x] A barra de modo mostra qual modo do vim está ativo
- [x] As teclas de um modo não disparam ações de outro (RNF1)
- [x] `j`/`k` atravessam blocos mantendo a coluna
- [x] Com o vim desligado, nada muda (RNF2) — a suíte inteira é a prova
- [x] A bateria de digitação continua verde (RNF3)

## Comandos de validação

```bash
cargo test --workspace
cd ui && PATH="$HOME/.cargo/bin:$PATH" cargo check --target wasm32-unknown-unknown
node scripts/uitest/run.mjs
node scripts/uitest/run.mjs --pendentes
```
