---
title: Capacidades de agente
type: docs
tags: [produto, agent-os]
---
# Capacidades de agente

O que o Anotadinho oferece pra trabalhar COM um modelo, e — mais
importante — quais são os limites.

{{ type: "callout" }}
variant: warning
title: A regra que sustenta tudo
body: |
  Um agente **nunca grava** no vault. Ele propõe, e a mudança só vira
  arquivo depois de alguém ver o diff e aprovar. As outras defesas
  reduzem a chance de o modelo ser enganado; esta continua valendo mesmo
  se ele for.
{{ /callout }}

## As quatro camadas

| Camada | O que faz | Ciclo |
|---|---|---|
| Sem shell | O prompt entra como UM argumento; `$(...)` dentro dele é texto | 202 |
| Contexto blindado | Conteúdo do vault vai em bloco delimitado que o texto não forja | 202 |
| Escrita é proposta | Nem CLI, nem MCP, nem UI gravam direto | 204 |
| Revisão humana | Nenhuma etapa do fluxo avança sozinha | 201 |

Não há varredura de "comandos perigosos" no prompt, de propósito: seria
contornável numa linha, daria falso positivo em nota que FALA de comando,
e criaria falsa confiança.

## Conversar dentro do app

Uma página `type: conversa` vira um painel de conversa. A conversa é um
`.md` comum — uma mensagem por heading — então dá pra ligá-la à spec com
`[[wikilink]]`, versioná-la no git e deixar o agente lê-la como contexto.

A página que estava aberta antes vai junto no prompt, com botão pra
desligar. Uma resposta boa vira spec ou proposta com um clique, já no
lugar certo e com a origem apontando de volta pra conversa.

## Configurar o modelo

Nas preferências do app — **nunca no vault**, pra uma página de terceiro
não escolher o que será executado. O contrato é `{prompt}` na linha de
comando:

| Agente | Executável | Argumentos |
|---|---|---|
| Claude Code | `claude` | `-p` `{prompt}` |
| Codex | `codex` | `exec` `{prompt}` |
| opencode | `opencode` | `run` `{prompt}` |

Executável com espaço é recusado: quase sempre é uma linha de shell
colada, e aceitar seria execução de shell pela porta dos fundos.

## Conectar um cliente MCP

O caminho inverso — o agente se conecta e opera o vault, com o
Anotadinho como estado compartilhado:

```
anotadinho-cli --vault /caminho/do/vault mcp
```

Seis ferramentas: `listar_paginas`, `ler_pagina`, `buscar`, `consultar`,
`propor` e `propostas_pendentes`. **A única escrita é `propor`.**

## Propor pelo terminal

```
echo "conteúdo" | anotadinho-cli propor pages/nova.md --motivo "por quê"
anotadinho-cli propostas
anotadinho-cli aplicar <id>
```

Entre o primeiro e o terceiro comando, a página não existe. Uma página
`type: propostas` mostra o diff de tudo que está pendente.

Ver também: [[Guia do Agent OS]], [[Ciclos]].
