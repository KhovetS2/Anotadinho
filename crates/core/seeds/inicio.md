---
title: Início
type: landing
tags:
- guia
---

# Bem-vindo

Este vault é uma pasta de arquivos `.md` comuns. Tudo que você vê aqui
continua legível fora do app, e entra no git como texto.

{{ type: "callout" }}
variant: tip
title: Por onde começar
body: |
  Aperte `Ctrl+K` para abrir a paleta de comandos. É de lá que se cria
  página, pasta, conversa com o agente e tudo mais. `Ctrl+/` mostra os
  atalhos.
{{ /callout }}

## O que uma página pode ser

O campo `type:` no início do arquivo decide como a página é aberta. Sem
ele, é uma nota comum — que já é o bastante para a maior parte das coisas.

| `type:` | O que abre |
| --- | --- |
| *(vazio)* | Nota comum: markdown, com blocos e embeds |
| `landing` | Página de entrada, como esta |
| `conversa` | Conversa com um agente, gravada no próprio arquivo |
| `spec` | O que precisa ser feito, e por quê |
| `proposta` | Como fazer — nasce de uma spec, e é revisada antes |
| `execucao` | O trabalho em si, e o relato do que aconteceu |
| `kanban` | Quadro de cards em colunas |
| `calendar` | Calendário do mês |
| `table` | Tabela de tarefas |
| `graph` | Grafo das conexões entre páginas |
| `prompt` | Molde de pergunta reutilizável na conversa |

## O que dá para colocar dentro de uma página

Digite `/` no editor para inserir qualquer um destes. Eles são blocos de
verdade — arrastáveis, editáveis — e continuam texto no arquivo.

{{ type: "columns" }}
columns:
- width: 1
  body: |
    **Estrutura**

    - Destaque (`callout`)
    - Colunas
    - Galeria de imagens
    - Botões de ação
- width: 1
  body: |
    **Dados**

    - Kanban
    - Calendário
    - Tabela
    - Consulta ao vault
    - Cronograma
{{ /columns }}

A **consulta** é a que mais rende: ela lê o vault inteiro e monta uma
lista viva. A de baixo mostra tudo que existe aqui, agrupado por tipo.

{{ type: "query" }}
view: list
group_by: type
aggregate:
- op: count
{{ /query }}

## Trabalhar com um agente

Uma conversa é uma página. O que você pergunta e o que o modelo responde
ficam gravados no `.md`, então sobrevivem a fechar o app e entram no git.

O ciclo que este vault usa é **spec → proposta → execução**. A spec diz o
que precisa acontecer; a proposta diz como; a execução é o trabalho. Cada
uma é uma página, e a barra no topo delas mostra em que etapa está.

{{ type: "callout" }}
variant: warning
title: O agente nunca escreve sozinho
body: |
  Quando o modelo quer mudar uma página, ele **propõe**. A mudança
  aparece na fila de revisão, com diff e com uma visualização de como a
  página fica, e só vira arquivo depois que você aprova.
{{ /callout }}

## As pastas

- `pages/` — suas páginas
- `pages/padroes/` — o que já se aprendeu, para anexar às conversas e o
  modelo não repetir erro conhecido
- `pages/prompts-default/` — moldes de pergunta que aparecem na conversa
- `journals/` — uma página por dia
- `templates/` — modelos para páginas novas
- `assets/` — imagens e arquivos colados ou arrastados
