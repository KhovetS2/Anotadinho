---
title: 'O que é um bloco: base comum de bloco e embed'
type: spec
date: 2026-09-05
status: rascunho
prioridade: alta
tags:
- spec
- editor
- arquitetura
---
{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}
# O que é um bloco

## Problema

Hoje existem **duas respostas diferentes** pra "o que é um bloco", e elas
discordam na mesma tela.

O modo de navegação diz: bloco é tudo que tem `data-nav-item` com
`data-nav-parent="editor-blocos"`. Isso inclui os dez embeds — as setas
passam por uma tabela como passam por um parágrafo.

O vim e a seleção múltipla dizem outra coisa: bloco é o que tem
`data-nav-block`, atributo que só os filhos de um `.editor__wysiwyg`
recebem. Nenhum embed tem. Consequências medidas:

- `j` em modo normal **pula por cima** de uma tabela como se ela não
  existisse;
- não há como fazer `dd` num embed, `yy` nele, nem movê-lo com `>`/`<`;
- `V` seguido de `j` seleciona os parágrafos ao redor e ignora o embed no
  meio — o conjunto copiado sai sem ele.

Isso **não foi descuido**: `selecao_blocos.rs` documenta a escolha
("embed é componente Yew, não markdown num contêiner, então não entra
numa seleção que existe pra ser serializada, apagada e movida"). A
decisão fazia sentido isolada. O que ela não previu é que um segundo
sistema de movimento (o vim, ciclo 252) herdaria a mesma lista e a
divergência viraria uma inconsistência visível pra quem usa.

Há ainda um terceiro custo, mais caro a prazo: **cada embed reimplementa
navegação por conta própria.** São dez arquivos `inline_*.rs`, cada um
espalhando `data-nav-item`/`data-nav-parent` à mão nos seus controles,
com convenções parecidas mas não idênticas. Um embed novo não herda nada
— copia de um vizinho, e as diferenças acumulam por cópia. É o mesmo
padrão que a ponte IPC tinha antes do ciclo 259.

## Objetivo

Uma definição só de bloco, com uma base comum que bloco de texto e embed
herdam, aplicando por padrão o que os dois precisam pra navegação e vim.
Cada embed refina a partir dali em vez de começar do zero.

## Requisitos funcionais

- **RF1.** Existe UMA lista ordenada dos blocos da página, e ela é a
  mesma pro modo de navegação, pro vim e pra seleção múltipla.
- **RF2.** Um embed é um bloco: `j`/`k` pousam nele, `dd` o apaga, `yy` o
  copia como markdown, e ele entra numa seleção de `V`.
- **RF3.** Pousar num embed é um REALCE, não um cursor de texto — porque
  ele não comporta caret. Sair dele com `j`/`k` continua de onde parou.
- **RF4.** Um bloco declara o que aceita: recebe texto? aceita cursor?
  pode ser dividido? pode ser mesclado? O que age sobre blocos consulta
  essa declaração em vez de testar a tag ou a classe.
- **RF5.** Um embed pode APROFUNDAR a interação (entrar nos controles com
  Enter, como já faz hoje) sem reescrever a navegação básica.
- **RF6.** Escrever um embed novo não exige saber de navegação: ele
  ganha o comportamento padrão por herdar a base.

## Requisitos não funcionais

- **RNF1.** O `.md` não muda. Isto é modelo interno e apresentação.
- **RNF2.** A base não pode custar render: a lista de blocos é lida a
  cada tecla de movimento (medido no ciclo 259: ~6,4ms por tecla numa
  página de 1200 blocos, dominado por mutação de classe).
- **RNF3.** Migração incremental. Dez embeds não migram num commit, e a
  suíte precisa ficar verde entre eles.
- **RNF4.** Nada de estado Rust espelhando a estrutura do DOM — o
  re-render o invalida, e é a razão documentada de a seleção morar em
  atributos hoje.

## Perguntas a decidir ANTES da proposta

- **`dd` num embed pede confirmação?** Apagar uma tabela de 600 linhas
  com duas teclas é destrutivo. No vim, `u` desfaz e ninguém pergunta;
  aqui o desfazer é o do navegador, e ele não alcança um componente Yew.
  Se não houver desfazer confiável, a confirmação deixa de ser opcional.
- **O que `yy` num embed põe no registrador?** O markdown do embed
  (colável em qualquer editor) ou uma referência interna? O markdown
  parece certo, e é o que `markdown_dos_selecionados` já faz.
- **Um embed com texto editável dentro (célula de tabela, título de
  card) deve responder a comandos vim?** Hoje não responde, porque o
  handler está preso ao `.editor__wysiwyg` — e isso é uma proteção, não
  um acidente: digitar `dd` num campo não pode apagar a linha.

## Não-objetivos

- Reescrever os dez embeds. A base entra e eles migram um a um.
- Seleção PARCIAL atravessando blocos — segue não-objetivo, herdado da
  spec de seleção.
- Vim dentro dos campos de texto de um embed.

## Relacionado

- [[Seleção atravessando blocos]] — de onde vem a definição atual
- [[Navegação por teclado e vim]] — o segundo sistema que herdou a lista
- [[Ciclo 259 — Revisão de padrões e estresse]] — o custo medido por tecla
