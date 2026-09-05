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

## O achado que muda o desenho

Existem **dois modelos de bloco no projeto, e eles não se conhecem.**

`crates/core/src/block.rs` define `Block { id, content, kind,
properties, depth }`, e `Page` tem `blocks: Vec<Block>`. O `markdown.rs`
parseia pra ele. É um modelo de dados, testável, sem DOM.

O editor **não usa nada disso**. A única ocorrência de "Block" em
`ui/src` é a string `"formatBlock"` de um `execCommand`. Para o editor,
um bloco é um `web_sys::Element` que recebeu `data-nav-block`,
`contenteditable` e uma classe — e "ser um bloco" é o efeito de
`marcar_blocos()` ter passado por ali.

Daí saem duas consequências que já custaram ciclos:

- **O bloco pode nascer sem ser bloco.** Foi o ciclo 249: o menu `/`
  inseria o elemento e ninguém carimbava os atributos, então ele nascia
  morto. Num modelo, um bloco inserido É um bloco; não há estado
  intermediário em que ele existe mas não conta.
- **A verdade do documento é o DOM.** `recompute_markdown_from_dom` lê a
  árvore e produz markdown na hora de salvar. O ciclo 248 precisou de uma
  trava no backend contra gravar vazio justamente porque o DOM pode estar
  transitoriamente vazio e ninguém tem um modelo pra confrontar.

E é também por isso que o `depth: u8` do `Block` do core não resolve
sozinho: profundidade é um número ao lado, não uma árvore. Um card de
kanban que contém blocos não se expressa com `depth`.

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

- ~~**`dd` num embed pede confirmação?**~~ **DECIDIDO: sim**, quando o
  alvo é um embed INTEIRO. Enquanto o desfazer não alcançar o modelo, a
  confirmação é a única rede. Quando o `Command`/desfazer entrar, a
  pergunta volta à mesa.
- **O que `yy` num embed põe no registrador?** O markdown do embed
  (colável em qualquer editor) ou uma referência interna? O markdown
  parece certo, e é o que `markdown_dos_selecionados` já faz.
- **Um embed com texto editável dentro (célula de tabela, título de
  card) deve responder a comandos vim?** Hoje não responde, porque o
  handler está preso ao `.editor__wysiwyg` — e isso é uma proteção, não
  um acidente: digitar `dd` num campo não pode apagar a linha.

## O modelo, e os padrões que ele instancia

A ideia é a do **Composite** (GoF), e a intenção original do padrão
descreve o pedido com precisão: *"compose objects into tree structures to
represent part-whole hierarchies; Composite lets clients treat individual
objects and compositions of objects uniformly"*. `dd` tratando um
parágrafo e um card de kanban do mesmo jeito é literalmente isso.

Um bloco é a unidade; um embed é um bloco cujo tipo carrega estrutura e
cujos filhos são blocos. Um card de kanban vira um bloco com filhos
(título, descrição), e aí a composição responde sozinha a pergunta que
motivou esta spec: `dd` no título apaga o bloco do título, `dd` com o
card selecionado apaga o card. Não são dois comandos — é o mesmo comando
em dois níveis.

**Onde o Composite clássico atrapalha, e como Rust escapa.** O GoF
discute o dilema entre a versão "transparente" (pôr `add`/`remove` na
base, e leaf herda operação sem sentido) e a "segura" (só o composto
tem, e o cliente precisa testar tipo). Num `enum` com `filhos:
Vec<Bloco>` o dilema não existe: o teste de tipo é exaustivo, verificado
pelo compilador, e "este bloco aceita filhos?" vira dado, não herança.

**Visitor**, para as operações. Serializar pra markdown, renderizar pra
DOM, aplicar um operador do vim e — o ponto que interessa — renderizar
pra terminal viram travessias da MESMA árvore. Em Rust isso costuma ser
um `match` sobre o enum, não a maquinaria de duplo despacho do livro; o
que vale do padrão é a separação, não a implementação.

**Command**, para o desfazer. Hoje o desfazer é o do navegador e ele não
alcança um componente Yew — é por isso que `dd` num embed PRECISA de
confirmação. Com os operadores como comandos sobre o modelo, `u` passa a
funcionar de verdade, e a confirmação vira escolha em vez de obrigação.

**Prior art direta:** o `NodeSpec` do ProseMirror (MIT). Cada tipo de nó
declara `content` (o que ele pode conter), `atom` (não tem conteúdo
editável diretamente — que é exatamente o que um embed é), `selectable`
e `isolating` (a seleção não atravessa esta fronteira). É o mesmo
desenho de "unidade base + camadas que trazem a intenção", já resolvido
por um editor que vive no DOM.

## O teste de fogo: e se o Anotadinho fosse um CLI?

A pergunta serve de régua porque separa o que é modelo do que é pintura.

**O backend já passa.** `crates/` — core, vault, ipc, search, cli, ~16
mil linhas — tem ZERO referências a `web_sys` ou `wasm_bindgen`. Um
Anotadinho de terminal reusaria tudo isso sem tocar numa linha.

**O que reprova é o modelo de movimento.** Os quatro arquivos que
definem o que é um bloco e como se anda entre eles — `nav_mode` (20
referências a `web_sys`), `selection_toolbar` (16), `vim_visual` (12),
`selecao_blocos` (10) — são inteiramente tipados contra
`web_sys::Element`. Não é lógica que *usa* o DOM: é lógica *escrita em
termos* de DOM. Portar significaria reescrevê-la, inclusive as partes
que são raciocínio puro.

Ou seja: o que impede o Anotadinho de virar um lazygit não é a
renderização — é o modelo morar dentro dela.

E há prova de que a direção funciona, feita sem querer: `vim_comandos.rs`
(a gramática de comandos) e as funções de palavra do ciclo 260 têm ZERO
`web_sys`. Foram extraídas pra serem testáveis, e o efeito colateral é
que portariam sem alteração. `markdown_render.rs` também está em zero.

## Não-objetivos

- Reescrever os dez embeds. A base entra e eles migram um a um.
- Seleção PARCIAL atravessando blocos — segue não-objetivo, herdado da
  spec de seleção.
- Vim dentro dos campos de texto de um embed.

## Relacionado

- [[Seleção atravessando blocos]] — de onde vem a definição atual
- [[Navegação por teclado e vim]] — o segundo sistema que herdou a lista
- [[Ciclo 259 — Revisão de padrões e estresse]] — o custo medido por tecla
