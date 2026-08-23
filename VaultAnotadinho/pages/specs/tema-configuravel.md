---
title: "Tema configurável"
type: spec
date: 2026-08-23
status: em-revisao
prioridade: baixa
tags:
- spec
- ui
---
# Tema configurável

{{ type: "fluxo" }}
artefato: spec
etapa: em-revisao
{{ /fluxo }}

## Contexto

O visual do app é um conjunto de tokens em `main.css`, com claro e
escuro. Trocar qualquer coisa além disso exige editar CSS e recompilar —
não há nenhum controle no app.

O sistema de tokens e a disciplina de BEM (nada de hex cru, `color-mix`
pra translúcidos) já são a metade difícil de um tema configurável. Falta
expor.

## Requisitos funcionais

- **RF1.** Existe uma tela de configuração de tema, alcançável pelas
  configurações.
- **RF2.** Dá pra escolher entre temas prontos, com pré-visualização
  antes de aplicar.
- **RF3.** Dá pra ajustar a cor de destaque separadamente do tema.
- **RF4.** Dá pra escolher o estilo dos botões entre variantes
  oferecidas.
- **RF5.** A escolha persiste entre sessões.
- **RF6.** Dá pra voltar ao padrão num clique.

## Requisitos não funcionais

- **RNF1.** Todo tema oferecido atende contraste de leitura — não é
  possível escolher uma combinação ilegível.
- **RNF2.** Tema é preferência do app, não conteúdo: nada disso entra
  no vault.
- **RNF3.** Aplicar um tema não recarrega a janela nem perde trabalho
  não salvo.
- **RNF4.** O `data-theme` continua sendo o mecanismo, pra o snapshot
  visual do harness seguir válido.

## Critérios de aceite

- [ ] A tela lista os temas com prévia e aplica no clique.
- [ ] A cor de destaque muda botões, foco e seleção de forma
      consistente.
- [ ] Fechar e reabrir o app mantém a escolha.
- [ ] "Voltar ao padrão" restaura tudo.
- [ ] O snapshot visual dos embeds passa em pelo menos dois temas.
- [ ] Cenários de harness pra aplicar tema e pra persistência.

## Fora de escopo

- Tema escrito pelo usuário (CSS próprio, arquivo de tema).
- Tema por página ou por vault.
- Trocar fonte e tipografia.

## Relacionado

- [[Sistema de design]]
