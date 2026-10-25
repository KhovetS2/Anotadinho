---
title: Tarefa rica, vault novo completo, painel inicial e medida de memória
tags:
- spec
- agent-os
- projeto
type: spec
date: 2026-10-22
prioridade: alta
status: aprovada
---
# Tarefa rica, vault novo completo, painel inicial e medida de memória

{{ type: "fluxo" }}
artefato: spec
etapa: aprovada
{{ /fluxo }}

## Contexto

O cartão do embed de kanban tem detalhes que a tarefa de projeto não tem:
tags, prazo, checklist, comentários e anexos. Quem planeja um projeto
quer os mesmos detalhes na tarefa, que é onde o trabalho de verdade mora.

Um vault novo nasce sem nada do módulo de projetos além da pasta vazia:
sem guia e sem um exemplo pra ver o quadro funcionando. E a página que
abre primeiro é um guia, não um painel do que está acontecendo.

Por fim, não existe estimativa de quanto o Anotadinho ocupa de memória
num dia de uso normal.

## Requisitos funcionais

### A. Tarefa com os detalhes do kanban

- **RF1.** Uma tarefa tem tags e prazo.
- **RF2.** Os itens de "Critérios de aceite" são a checklist da tarefa:
  dá pra marcar e desmarcar cada um sem abrir o editor.
- **RF3.** Uma tarefa guarda comentários, cada um com data e hora, na
  própria página.
- **RF4.** Uma tarefa tem anexos: arquivos guardados no vault e listados
  na página.
- **RF5.** O cartão do quadro mostra, quando houver: as tags, o prazo
  (destacado quando vencido e a tarefa não está feita), o progresso da
  checklist (2/5), e a contagem de comentários e de anexos.
- **RF6.** Um painel de detalhes, aberto do cartão, edita tags e prazo,
  marca a checklist, acrescenta comentário e anexa arquivo. Abrir a
  página da tarefa continua possível.
- **RF7.** O plano de projeto e a tarefa proposta pela LLM aceitam tags
  e prazo.

### B. Vault novo

- **RF8.** Um vault criado do zero traz o guia de projetos (com
  sprints) e o guia do modo agêntico.
- **RF9.** E traz um projeto de exemplo pequeno: uma sprint ativa, uma
  planejada e algumas tarefas em etapas diferentes, com os detalhes de
  A preenchidos.
- **RF10.** Criar a semente num vault que já tem páginas nunca
  sobrescreve nenhuma delas.

### C. Painel inicial

- **RF11.** A página que abre primeiro num vault novo é um painel com:
  os projetos, as sprints ativas (com o fim de cada uma), as tarefas
  esperando revisão, as specs por etapa e as propostas por etapa.
- **RF12.** O painel se atualiza sozinho quando as páginas mudam.
- **RF13.** O texto do guia que hoje é o Início continua existindo como
  página própria, com link no painel.

### D. Medida de memória

- **RF14.** Um script mede a memória da janela: o processo do app e os
  processos auxiliares da página web, somados e separados.
- **RF15.** Outro mede a TUI.
- **RF16.** Os dois simulam um dia de uso, repetido em rodadas: abrir e
  trocar páginas e abas, digitar, buscar, abrir projeto, conversa e
  grafo, e fechar abas.
- **RF17.** O relatório diz, por processo: a memória em repouso, o pico,
  a do fim, e quanto cresce por rodada. O crescimento por rodada é o
  sinal de vazamento.
- **RF18.** O relatório diz qual build mediu (desenvolvimento ou
  instalado), porque os dois diferem muito.

## Requisitos não funcionais

- **RNF1.** Tudo que a LLM escreve PROPÕE.
- **RNF2.** A tarefa continua legível como markdown fora do app: os
  detalhes moram no frontmatter e em seções da página, e nunca num
  formato à parte.
- **RNF3.** Um projeto sem nenhum desses detalhes aparece como hoje.
- **RNF4.** O script de memória não altera nenhuma página da pessoa:
  trabalha em páginas temporárias e as remove no fim, mesmo se falhar.
- **RNF5.** A janela e a TUI fazem as mesmas coisas, cada uma do seu jeito.

## Critérios de aceite

- Marcar um item da checklist no painel muda o `[ ]` daquele item na
  página, e só ele.
- Um comentário acrescentado aparece na página com a data, e a contagem
  no cartão sobe.
- Um anexo fica no vault e abre a partir da tarefa.
- O cartão de uma tarefa vencida e não feita mostra o prazo destacado.
- `init` num diretório vazio cria o painel, os dois guias e o projeto de
  exemplo, e o quadro do exemplo abre na sprint ativa.
- O painel mostra as sprints ativas e as propostas por etapa de um vault
  com conteúdo.
- O script de memória termina com um relatório por processo, e nenhuma
  página `uitest-*` fica para trás.

## Fora de escopo

- Responsável (assignee), estimativa de pontos e campos personalizados.
- Comentários feitos pela LLM.
- Medida de memória no Windows e no macOS. O script lê o `/proc` do
  Linux.
- Otimizar a memória. Este trabalho MEDE; o que fazer com o número é
  decisão depois.

## Ambiguidades apontadas

- **"Personalizações do kanban"** foi lida como os campos do cartão
  (tags, prazo, checklist, comentários e anexos), escolhidos pela
  pessoa. Cor de cartão e configuração de colunas ficam de fora: as
  colunas do projeto são as etapas, que têm regra.
- **O painel no vault deste repositório.** A spec fala do vault novo. O
  `VaultAnotadinho` tem o próprio Início: trocá-lo é uma proposta à
  parte, que a pessoa pode recusar sem recusar esta spec.
