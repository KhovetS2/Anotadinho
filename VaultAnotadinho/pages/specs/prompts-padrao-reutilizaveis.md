---
title: "Prompts padrão reutilizáveis"
type: spec
date: 2026-08-22
status: em-revisao
prioridade: media
tags:
- spec
- agent-os
---
# Prompts padrão reutilizáveis

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao
{{ /fluxo }}

## Contexto

Pedidos que se repetem são redigitados toda vez. "Crie uma spec para
isto", "revise este texto seguindo o padrão X", "resuma esta página em
tópicos" — o formato é sempre o mesmo, só o assunto muda.

Hoje a pessoa reescreve o pedido inteiro a cada conversa, ou copia de
algum lugar. Duas consequências: cansa, e o pedido sai levemente
diferente a cada vez, o que faz a resposta variar sem motivo.

O botão "Planejar implementação" (ciclo 209) resolve **um** caso desses
com código dedicado. Não dá pra abrir um botão novo pra cada pedido
recorrente — quem sabe quais são é quem usa.

## Requisitos funcionais

- **RF1.** A pessoa cria um prompt padrão, com título e corpo, e ele
  fica guardado no vault como página.
- **RF2.** Ao escrever uma mensagem na conversa, é possível escolher um
  prompt padrão; o que foi digitado entra dentro dele, no lugar marcado.
- **RF3.** Um prompt padrão pode ter **mais de um ponto de inserção**,
  identificados por marcadores no texto; a pessoa preenche cada um.
- **RF4.** Um prompt padrão pode declarar páginas de contexto que devem
  ser anexadas junto quando ele for usado.
- **RF5.** Prompts padrão são editáveis como qualquer página, sem
  precisar de tela de configuração própria.

## Requisitos não funcionais

- **RNF1.** O prompt final continua visível antes de enviar — a pessoa
  precisa ver o que vai sair, não confiar num molde invisível.
- **RNF2.** Um marcador não preenchido não pode virar texto literal no
  prompt sem aviso.
- **RNF3.** O formato é markdown legível fora do app, como o resto do
  vault.
- **RNF4.** O conteúdo inserido continua entrando como DADO, não como
  instrução — a blindagem do ciclo 202 não pode ser contornada por um
  molde.

## Critérios de aceite

- [ ] Criar um prompt com um marcador, usá-lo numa conversa e ver o
      texto digitado aparecer no lugar certo do prompt final.
- [ ] Um prompt com três marcadores pede os três antes de enviar.
- [ ] Um marcador esquecido é apontado antes do envio, não depois.
- [ ] Um prompt que declara contexto anexa as páginas ao ser escolhido.
- [ ] Cenários de harness cobrindo marcador único, múltiplo e faltando.

## Fora de escopo

- Compartilhar prompts entre vaults ou publicar uma biblioteca.
- Condicionais ou laços dentro do prompt — se precisar disso, é código,
  não molde de texto.
- Gerar o prompt automaticamente a partir do histórico.

## Notas de escopo

O marcador precisa ser algo que não apareça por acidente em texto comum
e que sobreviva a ser editado à mão no `.md`. Vale olhar como o
`{{title}}` dos templates de página já resolve isso — usar a mesma
convenção evita inventar uma segunda linguagem no mesmo vault.

## Relacionado

- [[Como usar o modo agêntico]]
- [[Capacidades de agente]]
- [[Uso agêntico do Anotadinho no dia a dia]]
