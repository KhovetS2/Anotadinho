---
title: Exemplos de Embeds
tags: [demo, embed]
---

# Exemplos de blocos embedados

Você pode usar blocos especiais dentro de qualquer página `.md`, delimitados
por `{{ type: "..." }}` ... `{{ /... }}` (não usa fence de código markdown,
pra não colidir com blocos de código de verdade).

São dez tipos, e esta página tem **um de cada** — ela é a referência
completa, pra quando se quer ver todos lado a lado (desenhar uma tela,
conferir o que existe, comparar formatos).

As páginas abaixo continuam valendo, e são melhores pra APRENDER cada
assunto: aqui o exemplo é mínimo, lá vem com contexto.

- [[Composição — destaque, colunas e galeria]] — montar a página
- [[Consultas — listas que se mantêm sozinhas]] — recortes vivos do vault
- [[Referências — wikilink, transclusão e bloco]] — apontar pra outra página
- [[Painel]] — os embeds trabalhando juntos, com botões de ação

## Kanban Embed

{{ type: "kanban" }}
columns:
- Backlog
- Todo
- Done
items:
- title: Tarefa 1
  column: Backlog
  description: Descrição da tarefa 1, com mais detalhes do que cabe no título.
  tags:
  - urgente
  - bug
  due: 2026-08-10
  checklist:
  - text: Sub-item 1
    done: false
  - text: Sub-item 2
    done: true
- title: Tarefa 2
  column: Done
- title: Tarefa 3
  column: Done
{{ /kanban }}

## Calendar Embed

{{ type: "calendar" }}
entries:
- date: 2026-08-06
  title: Revisão de código
  tags:
  - urgente
- date: 2026-08-07
  title: Deploy produção
  start_time: 14:30
  end_time: 15:15
- date: 2026-08-10
  title: Sprint de agosto
  end_date: 2026-08-14
  tags:
  - infra
- date: 2026-08-08
  title: Retrospectiva sprint
- title: Ligar pro fornecedor
{{ /calendar }}

## Table Embed

{{ type: "table" }}
columns:
  - name: Tarefa
  - name: Status
    type: select
    options: [todo, doing, done]
  - name: Tags
    type: multiselect
    options: [urgente, bug, infra]
  - name: Estimativa
    type: number
  - name: Prazo
    type: date
  - name: Referência
    type: url
  - name: Relacionado
    type: page
---
| Tarefa | Status | Tags | Estimativa | Prazo | Referência | Relacionado |
| --- | --- | --- | --- | --- | --- | --- |
| API | done | infra | 8 | 2026-08-05 | https://example.com | pages/kanban-projeto.md |
| UI | doing | urgente, bug | 5 | 2026-08-10 |  |  |
| Testes | todo |  | 3 |  |  |  |
{{ /table }}

Acima do embed você pode ter texto normal. Abaixo também.

## Destaque (`callout`)

Caixa que separa um aviso do texto em volta. O corpo é markdown de
verdade — título, lista, código, tudo vale ali dentro.

{{ type: "callout" }}
variant: warning
title: Cuidado com a gravação
body: |
  Editar uma página aberta em duas janelas sobrescreve a mais antiga.

  - a checagem de versão avisa antes
  - o conflito abre um diff pra escolher
{{ /callout }}

`variant` aceita `info`, `warning`, `danger`, `success` e `note` — muda
a cor e o ícone, não o comportamento.

## Colunas (`columns`)

Painéis lado a lado, cada um com markdown próprio. `width` é peso
relativo: `1` e `2` dá um terço e dois terços.

{{ type: "columns" }}
columns:
- width: 1
  body: |
    ### O que entra
    - o que já foi decidido
    - o que está em revisão
- width: 2
  body: |
    ### O que não entra

    Ideia solta vive em [[Propostas]], não aqui. Esta coluna é mais
    larga porque o texto dela é mais longo.
{{ /columns }}

## Galeria (`gallery`)

Grade de imagens do vault, com legenda. `columns` é quantas cabem por
linha; `size` (`sm`, `md`, `lg`) é a altura da miniatura.

{{ type: "gallery" }}
columns: 3
size: md
items:
- path: assets/exemplo-a.png
  caption: Primeira captura
- path: assets/exemplo-b.png
  caption: Segunda captura
{{ /gallery }}

Imagem que não existe aparece como moldura vazia com o caminho — some
quando o arquivo entra em `assets/`.

## Consulta (`query`)

Recorte vivo do vault: o YAML É a consulta, e o resultado se atualiza
sozinho conforme as páginas mudam.

{{ type: "query" }}
from: pages
where:
- field: type
  op: exists
limit: 5
view: table
columns:
- type
{{ /query }}

`view` aceita `list`, `table` e `cards`. É o único embed cujo conteúdo
NÃO está no arquivo — ele vem do vault na hora de desenhar, e por isso
não aparece na árvore de blocos nem no `anotadinho-cli ver`.

## Etapa de fluxo (`fluxo`)

Onde um artefato está no caminho de spec → proposta → execução. Não
guarda coleção: guarda um estado, e os botões movem esse estado.

{{ type: "fluxo" }}
artefato: proposta
etapa: em-revisao
{{ /fluxo }}

`artefato` é `spec`, `proposta`, `execucao` ou `conversa`; `etapa` vai
de `rascunho` a `concluida`, passando por `bloqueada`. A etapa fechada
impede edição automática por agente.

## PDF Embed

Diferente dos blocos acima (`{{ type: "..." }}`, sintaxe de fence
explícita), embed de PDF é automático: qualquer link markdown normal
apontando pra um arquivo `.pdf` em `assets/` vira um frame com scroll
interno sozinho, sem precisar de sintaxe especial — só um link comum:

[Relatório de exemplo](assets/exemplo-relatorio.pdf)

Clicar/scrollar dentro do frame acima navega dentro do PDF (zoom, busca
e paginação do próprio visualizador do WebView), sem abrir outro
programa. Ao salvar a página, o link markdown original é preservado
tal qual — o frame é só uma forma de exibir, o arquivo `.md` continua
guardando `[texto](caminho.pdf)`.

## Cronograma (`timeline`)

Barras por intervalo de datas, em escala de semana, mês ou trimestre.
Arrastar move preservando a duração; a alça da borda estica só aquela
ponta. Pelo teclado: `Alt+←/→` movem, `Alt+Shift+←/→` esticam.

{{ type: "timeline" }}
scale: week
source: vault
items:
- title: Levantar requisitos
  start: 2026-08-03
  end: 2026-08-10
  tags:
  - infra
- title: Implementar
  start: 2026-08-11
  end: 2026-08-24
- title: Revisar e publicar
  start: 2026-08-25
  end: 2026-08-31
  tags:
  - urgente
{{ /timeline }}

Com `source: vault` no lugar dos `items`, ele lê as páginas do vault
que tenham `start`/`date` no frontmatter — aí vira somente leitura, e
clicar numa barra abre a página de origem.

## Ações (`actions`)

O único embed que ESCREVE. Os botões criam página a partir de template
na pasta certa, abrem página, gravam propriedade ou abrem a busca — e o
que cada um faz está declarado no YAML, legível por humano e por
agente.

{{ type: "actions" }}
layout: row
buttons:
- label: Ver exemplos de consulta
  icon: search
  action: open-page
  path: pages/exemplos/consultas.md
- label: Abrir o painel
  icon: home
  action: open-page
  path: pages/produto/painel.md
- label: Nova nota de reunião
  icon: file-text
  variant: primary
  action: new-from-template
  template: templates/nota-de-reuniao.md
{{ /actions }}

A lista de ações é FECHADA de propósito: nada de rodar comando de
shell. Um `.md` que chegasse de terceiro não pode executar nada só por
ser aberto.
