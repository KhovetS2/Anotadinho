---
title: "Navegação por teclado consistente e modo vim completo"
type: spec
date: 2026-08-23
status: em-revisao
prioridade: alta
tags:
- spec
- teclado
---
# Navegação por teclado consistente e modo vim completo

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao
{{ /fluxo }}

## Contexto

A navegação por teclado cresceu por ciclos (174, 194, 195, 197) e hoje
tem dois problemas distintos.

**O primeiro é um bug de estado.** Caminho relatado: na página inicial,
navegar até "Trabalho recente", Enter, escolher um card, a página abre —
e a partir daí as setas ficam presas na barra superior. Escape devolve
as setas ao editor, mas um segundo Escape **não** volta pra navegação
entre seções: só Backspace faz isso.

A hipótese, a confirmar na implementação: ao abrir uma página de dentro
do grupo de um embed, a pilha de navegação continua apontando pro grupo
ANTIGO, que não existe mais na página nova. O comportamento de Escape
depende dessa pilha, então ele responde pelo contexto errado.

**O segundo é o modo vim.** Ele foi feito muitos ciclos atrás e ficou
para trás: só existe o par normal/inserção, e mesmo esse não acompanha o
editor por bloco (ciclo 175). Falta metade do vocabulário que quem usa
vim espera, e não há separação clara entre as teclas do vim e as do modo
de navegação — o que é fonte de conflito.

## Requisitos funcionais

- **RF1.** Abrir uma página a partir de qualquer contexto de navegação
  deixa o teclado num estado previsível e documentado.
- **RF2.** Escape sobe UM nível por vez, sempre, sem depender de qual
  caminho levou até ali. Backspace continua fazendo o que faz hoje.
- **RF3.** `h`/`j`/`k`/`l` movem no modo de navegação, junto com as
  setas.
- **RF4.** O modo vim oferece os quatro modos principais: normal,
  inserção, **visual** e **visual em bloco**.
- **RF5.** Modo de comando por `/` fora do modo de edição.
- **RF6.** Com o vim ligado, existe um atalho dedicado pra entrar no
  modo de navegação, sem disputar tecla com os comandos do vim.
- **RF7.** A barra de modo mostra qual modo do vim está ativo.

## Requisitos não funcionais

- **RNF1.** As teclas de um modo não disparam ações de outro. É a regra
  do ciclo 199, e a tabela `ATALHOS` é onde ela vive.
- **RNF2.** Com o vim desligado, nada muda pra quem não usa.
- **RNF3.** A bateria de digitação continua verde: o caminho mais usado
  do editor não pode regredir.

## Critérios de aceite

- [ ] O caminho relatado (home → trabalho recente → card → página) deixa
      as setas navegando o conteúdo, não a barra superior.
- [ ] Dois Escapes seguidos sobem dois níveis, sem precisar de Backspace.
- [ ] `hjkl` movem onde as setas movem.
- [ ] Visual seleciona por caractere e visual em bloco por retângulo;
      copiar o selecionado produz markdown legível.
- [ ] `/` fora da edição abre a busca de comando.
- [ ] O atalho de navegação não colide com nenhum comando do vim.
- [ ] Cenários de harness pra cada modo e pra cada transição entre eles.

## Fora de escopo

- Macros, registradores nomeados e `.` (repetir).
- Emular o vim inteiro: o alvo é o vocabulário que se usa todo dia.

## Notas de escopo

A seleção visual esbarra na mesma limitação registrada em
[[Seleção e cópia atravessando blocos]] — com um editável por bloco, o
navegador não estende seleção entre blocos. As duas specs precisam ser
planejadas juntas, ou a visual nasce só funcionando dentro de um bloco.

## Relacionado

- [[Seleção e cópia atravessando blocos]]
- [[Ciclo 199 — Keymap por modo em tabela]]
