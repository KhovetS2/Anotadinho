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
- ~~**O que `yy` num embed põe no registrador?**~~ **DECIDIDO:** uma
  forma serializada, com a colagem parseando de volta. Ver a seção do
  registrador acima.
- ~~**Um embed deve responder a comandos vim?**~~ **DECIDIDO: sim, no
  que for cabível**, e "cabível" deixa de ser julgamento nosso pra ser
  declaração de cada unidade, resolvida pela cadeia de
  responsabilidade. A proteção de hoje (digitar `dd` num campo de texto
  não pode apagar a linha) continua valendo: um campo em edição trata a
  tecla e ela não sobe.

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

## O tmux aninhado, e o padrão que o descreve

O modelo pedido é: painéis dentro de painéis, foco que desce e sobe, e a
tecla vai pro painel mais interno que souber o que fazer com ela. Não se
"desliga o vim pra usar o calendário" — o calendário simplesmente
responde às teclas que fazem sentido nele.

Isso tem nome, e é o **Chain of Responsibility**: *"lets you pass
requests along a chain of handlers"*, aplicável *"quando o programa
precisa processar tipos diferentes de requisição de várias formas, mas
os tipos exatos e sua sequência não são conhecidos de antemão"*.

E não é invenção nossa juntá-lo ao Composite — a página do Composite diz
que os dois *"são frequentemente usados em conjunto"*, com os
componentes-folha passando requisições pelos pais até a raiz. É
exatamente o desenho.

Na prática, dentro de um embed de calendário:

| tecla | quem trata | resultado |
|---|---|---|
| `j` | o calendário | próximo dia |
| `w` | ninguém no calendário → sobe | próximo bloco |
| `dd` | ninguém no calendário → sobe | apaga o bloco do calendário |
| `i` | o calendário, se tiver campo | edita; senão sobe e é ignorado |

O que hoje é uma decisão binária ("vim ligado ou desligado") vira uma
propriedade de cada unidade: **o que EU sei tratar**. Um embed que não
declara nada continua funcionando — tudo sobe, e o comportamento é o de
hoje. É o que torna a migração dos dez embeds incremental de verdade.

## Os outros padrões, e onde cada um encaixa

O Composite sozinho não resolve; o catálogo aponta os companheiros dele.

**Iterator** — *"você pode usar Iterators pra percorrer árvores
Composite"*. É o RF1: uma travessia só, com variantes (só visíveis, só
os que aceitam texto, profundidade primeiro) em vez dos
`query_selector_all` + filtro espalhados de hoje.

**Visitor** — *"pra executar uma operação sobre uma árvore Composite
inteira"*. Serializar pra markdown, pintar no DOM, pintar no terminal.

**Adapter** — o padrão da migração. Os dez embeds não viram blocos no
mesmo commit (RNF3); um adaptador deixa um embed ainda não migrado
apresentar a interface nova enquanto por dentro continua como está.

**Command** e/ou **Memento** — o desfazer. É o que hoje obriga a
confirmação no `dd` sobre embed.

**State** — os modos (Normal, Inserção, Visual, Navegação). Já existe
como `VimModo` + `match`, e em Rust isso costuma ser melhor que objetos
de estado: o compilador confere que todo modo trata toda tecla.

**Rejeitados, com motivo:**

- **Decorator.** A própria página o distingue: *"Decorator acrescenta
  responsabilidades ao objeto embrulhado, enquanto Composite apenas soma
  os resultados dos filhos"*. Um card de kanban soma seus blocos; não
  embrulha um.
- **Flyweight, Proxy, Singleton.** Resolvem problemas que não temos.

## O eixo CLI/GUI: Bridge, e não Strategy

A intuição de separar o comportamento base atrás de uma interface
genérica está certa, e o ganho apontado — testar os dois lados — é o
ganho real. Sobre o nome, o catálogo é específico, e a diferença ajuda a
decidir ONDE cortar.

**Strategy** é *"uma família de algoritmos... intercambiáveis"*, pra
*"trocar de algoritmo em tempo de execução"*.

**Bridge** é *"separar uma classe grande em duas hierarquias —
abstração e implementação — que podem evoluir independentemente"*, e a
sua segunda aplicabilidade é literalmente *"quando você precisa estender
uma classe em várias dimensões ortogonais (independentes)"*.

Aqui há duas dimensões que crescem sozinhas: **os tipos de bloco**
(parágrafo, título, tabela, kanban, calendário...) × **os renderizadores**
(DOM, terminal). Toda vez que nasce um tipo de bloco ele precisa existir
nos dois; toda vez que nasce um renderizador ele precisa dar conta de
todos. Isso é Bridge pela definição.

O Strategy encaixa noutra junta, mais fina: a POLÍTICA de cada tipo de
bloco — o que ele trata, se é atômico, se aceita cursor. São
comportamentos plugáveis por tipo, não uma segunda hierarquia.

O catálogo avisa que os dois têm estrutura quase igual e que *"um padrão
é mais do que um jeito de estruturar classes; ele comunica intenção e o
problema que endereça"*. Na prática o que importa mais que o nome é a
**direção da dependência**: o modelo não pode conhecer o renderizador.
Com isso, os testes do modelo rodam sem DOM e sem terminal — que é
exatamente o que já aconteceu por acidente com `vim_comandos.rs`.

## O registrador: forma serializada + parser na colagem

Decidido: o registrador guarda uma forma serializada e a colagem
PARSEIA. Markdown é o candidato natural (colável fora do app, e
`markdown_dos_selecionados` já faz), mas a escolha fica pra
implementação — se o modelo de bloco pedir outra forma, ela ganha, desde
que a colagem saiba reconstruir a partir dela.

## O teste de fogo: e se o Anotadinho fosse um CLI?

A pergunta começou como régua — serve pra separar o que é modelo do que
é pintura — e virou objetivo declarado: uma versão de porte totalmente
CLI é desejada no futuro. Isso muda o peso do Bridge acima de
"organização elegante" pra "requisito".

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

## O que a implementação ensinou (ciclos 261-266)

**A cadeia não precisou de despacho próprio.** O ciclo 262 modelou o
roteamento como função pura esperando construir o roteador na UI. Não
precisou: o DOM já borbulha, e isso É a cadeia. Um evento nascido dentro
do componente já passou por ele antes de chegar ao documento. A regra
que faltava era de uma linha — com o foco DENTRO de um embed, o vim do
documento se cala. O `Interesses` segue valendo como a forma declarativa
e testável da mesma regra.

**"No bloco" e "dentro do bloco" são estados diferentes**, e é essa
distinção que separa `j` andar entre blocos de `j` andar entre dias de
um calendário. Sem ela, não há como escrever a regra acima.

**Entrar sem poder sair é o defeito recorrente desta série.** Aconteceu
com o `j` no ciclo 263 (entrava no embed e travava) e teria acontecido
com o Enter no 265 se o Escape não tivesse sido tratado como exceção
explícita. Todo gesto que DESCE precisa do gesto que SOBE no mesmo
commit.

**Um cenário que aperta a tecla uma vez não distingue "funcionou" de
"travou".** Foi assim que o 263 passou verde com um defeito pior que o
original.

## Não-objetivos

- Reescrever os dez embeds. A base entra e eles migram um a um.
- Seleção PARCIAL atravessando blocos — segue não-objetivo, herdado da
  spec de seleção.
- Vim dentro dos campos de TEXTO de um embed (célula em edição, título
  de card sendo digitado): ali a tecla é do campo e não sobe.

## Relacionado

- [[Seleção atravessando blocos]] — de onde vem a definição atual
- [[Navegação por teclado e vim]] — o segundo sistema que herdou a lista
- [[Ciclo 259 — Revisão de padrões e estresse]] — o custo medido por tecla
