---
title: Prompts padrão reutilizáveis
tags:
- spec
- agent-os
type: spec
date: 2026-08-22
prioridade: media
status: aprovada
---
# Prompts padrão reutilizáveis

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada

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

- **RF1.** A pessoa cria um prompt padrão, com título e corpo, como uma
  página de `type: prompt` dentro de `pages/prompts-default/` ou de uma
  de suas subpastas.
- **RF2.** Na página de conversa, um seletor acima da ação de enviar
  lista somente os prompts padrão encontrados nessa pasta e em suas
  subpastas.
- **RF3.** O seletor oferece uma opção vazia, que mantém o comportamento
  de escrever a mensagem inteira do zero, sem aplicar prompt padrão.
- **RF4.** Ao escolher um prompt padrão, o que foi digitado na conversa
  entra no lugar indicado pelo marcador correspondente.
- **RF5.** Marcadores são variáveis nomeadas no formato `{{title}}`. Um
  prompt pode declarar mais de uma variável, e a pessoa preenche os
  valores na ordem da primeira ocorrência de cada variável no prompt.
- **RF6.** Quando uma mesma variável aparece mais de uma vez, ela é
  preenchida uma única vez e o mesmo valor é repetido em todas as suas
  ocorrências.
- **RF7.** Um prompt padrão pode declarar páginas de contexto que devem
  ser anexadas junto quando ele for usado.
- **RF8.** Prompts padrão são editáveis como qualquer página, sem
  precisar de tela de configuração própria.
- **RF9.** Antes do envio, uma ação de visualização abre um modal com o
  prompt final e os valores já substituídos.

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

- [ ] Criar uma página `type: prompt` em `pages/prompts-default/`,
      escolhê-la no seletor da conversa e ver o texto digitado aparecer
      no lugar de `{{title}}` no prompt final.
- [ ] Páginas fora de `pages/prompts-default/` e de suas subpastas não
      aparecem no seletor, mesmo que tenham `type: prompt`.
- [ ] A opção vazia do seletor permite escrever e enviar uma mensagem
      sem aplicar um prompt padrão.
- [ ] Um prompt com três variáveis diferentes pede as três na ordem em
      que aparecem no prompt.
- [ ] Duas ocorrências de `{{title}}` pedem um único valor e repetem esse
      valor nos dois lugares.
- [ ] Um marcador esquecido é apontado antes do envio, não depois.
- [ ] Um prompt que declara contexto anexa as páginas ao ser escolhido.
- [ ] O modal de visualização mostra o prompt final com todos os valores
      substituídos e não envia a mensagem automaticamente.
- [ ] Cenários de harness cobrem marcador único, múltiplas variáveis,
      variável repetida, marcador faltando, opção vazia e preview.

## Fora de escopo

- Compartilhar prompts entre vaults ou publicar uma biblioteca.
- Descobrir prompts fora de `pages/prompts-default/` e de suas
  subpastas.
- Condicionais ou laços dentro do prompt — se precisar disso, é código,
  não molde de texto.
- Gerar o prompt automaticamente a partir do histórico.

## Notas de escopo

O marcador usa a mesma convenção legível dos templates de página: nome
entre chaves duplas, como `{{title}}`. Nomes diferentes representam
campos diferentes. A ordem de preenchimento é a ordem da primeira
ocorrência de cada nome no prompt; ocorrências posteriores do mesmo nome
reutilizam o valor já informado.

A localização e o tipo são necessários ao mesmo tempo: somente páginas
com `type: prompt` dentro de `pages/prompts-default/` ou de suas
subpastas são descobertas pelo seletor.

A spec ainda não define como as páginas de contexto são declaradas no
Markdown nem se os anexos valem apenas para o uso atual ou permanecem na
conversa. Essas decisões precisam ser registradas antes da implementação.

## Relacionado

- [[Como usar o modo agêntico]]
- [[Capacidades de agente]]
- [[Uso agêntico do Anotadinho no dia a dia]]
