---
title: "Proposta de implementação — Atalhos do dia a dia: aba fixa e"
type: proposta
date: 2026-08-23
status: rascunho
tags:
- proposta
---
# Proposta de implementação — Atalhos do dia a dia: aba fixa e

{{ type: "fluxo" }}
artefato: proposta
etapa: rascunho
origem: pages/conversas/conversa-2026-08-23-02-48.md
{{ /fluxo }}

Proposta de implementação — Atalhos do dia a dia: aba fixa e criação de conversa

## Abordagem

Concentrar a regra de fixação na gestão de `open_tabs`: quando houver página inicial configurada, sua aba é mantida no índice zero; sem configuração, a lista permanece inalterada. A ação de fechar recusará a aba cujo caminho corresponde à home.

A barra continuará renderizando a home como uma aba comum, com modificador visual de “fixa” e sem controle de fechamento. Ela preservará a mesma semântica e participação na navegação por teclado das demais abas; inclusive, os atalhos que dependem de índice passam a enxergar a home na primeira posição.

Para criação, incluir `conversa` nas ações existentes de “Nova página: tipo”, reutilizando o fluxo já empregado pelos demais tipos: pedir título, criar com `type: conversa` e abrir o resultado. A página será exibida pelo painel de conversa já existente, portanto ficará pronta para uso sem criar um segundo fluxo de inicialização.

## Etapas

1. Mapear os pontos que alteram `open_tabs`, definem/removem a home e fecham abas.
2. Criar uma operação única de ordenação que promova a home configurada para a primeira posição, preservando a ordem relativa das demais abas.
3. Aplicar essa operação ao abrir páginas, restaurar a home ao iniciar o vault e trocar a página inicial.
4. Bloquear o fechamento da aba home tanto na interface quanto no handler que remove a aba.
5. Ajustar `TabBar` para distinguir visualmente a aba fixa e omitir seu botão de fechar, mantendo seu elemento acionável e navegável por teclado.
6. Adicionar a ação “Nova página: Conversa” à família de tipos na paleta e encaminhá-la ao fluxo de criação tipada.
7. Acrescentar cenários ao harness: home fixa/reatribuída e criação de conversa pela paleta.
8. Executar as validações do ciclo, registrar status, atualizar a task e concluir o ciclo conforme o processo do repositório.

## Padrões seguidos

- [[Nomenclatura]]: componentes Yew novos ou alterados seguirão PascalCase; arquivos, funções e variáveis usarão `snake_case`; o valor de frontmatter continuará sendo `type: conversa`, com mapeamento explícito caso seja representado por campo Rust.
- Ciclo 089 — Landing page novo page_type e definir como início: a home permanece preferência local por vault, sem gravar esse estado no conteúdo versionado.
- Ciclo 107 — Navegação de abas via teclado: a home continua na lista regular de abas, portanto a ordem visual, os índices e os atalhos permanecem coerentes.
- Ciclo 128 — Criar página de tipo específico via paleta de comandos: `conversa` entra como mais uma ação `NewPageOfType`, sem criar mecanismo paralelo.
- Ciclo 202 — Conversa com agente externo: a conversa é criada como página do vault com `type: conversa`, fazendo o `ConversaView` existente assumir a interface utilizável.

## Alternativas consideradas

- Criar uma “área de home” separada antes da barra de abas: descartado porque violaria o requisito de ela continuar sendo uma aba e dificultaria a paridade de teclado.
- Adicionar uma flag genérica de abas fixadas: descartado por ampliar o escopo para fixação arbitrária de abas.
- Criar uma ação exclusiva de “Nova conversa” dentro da família de tipos: descartado porque manteria a inconsistência que a spec quer eliminar e duplicaria a criação tipada.
- Persistir a posição fixa junto ao vault: descartado porque a home já é preferência local por vault e a spec não pede mudança nesse modelo.

## Riscos

- Reordenar abas em pontos diferentes pode produzir ordens inconsistentes; por isso a promoção da home deve ter uma única operação reutilizada.
- O bloqueio apenas visual permitiria fechamento por atalho ou handler indireto; a proteção precisa existir também na lógica de remoção.
- A alteração de ordem afeta atalhos por índice, exigindo cobertura no harness.
- Há uma ambiguidade em RNF1: a spec pede distinção visual, mas não define ícone, cor ou texto. A proposta reserva um modificador visual discreto, sem escolher esse detalhe de design.
