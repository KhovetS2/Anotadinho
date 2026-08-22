---
title: "Como usar o modo agêntico"
type: docs
tags:
- agent-os
- guia
---
# Como usar o modo agêntico

Passo a passo de operar o Anotadinho com um modelo. Pressupõe o agente
já instalado (`claude`, `codex` ou `opencode`).

{{ type: "callout" }}
variant: info
title: O que garante que isto é seguro
body: |
  O agente nunca grava no vault: ele propõe, e você aprova. Nenhuma
  etapa do fluxo avança sozinha. Os detalhes estão em
  [[Capacidades de agente]].
{{ /callout }}

## 1. Configurar o modelo, uma vez

Nas preferências do app. O contrato é `{prompt}` na linha de comando:

| Campo | Exemplo |
|---|---|
| Executável | `claude` |
| Argumentos | `-p` e `{prompt}` |
| Timeout | `180` segundos |

Executável com espaço é recusado de propósito — quase sempre é uma linha
de shell colada.

## 2. Conversar

Crie uma página com `type: conversa` no frontmatter. Ela abre como
painel de conversa em vez de editor.

A página que você estava lendo antes vai junto como contexto, com botão
pra desligar. Isso significa que dá pra abrir um relatório, ir pra
conversa e perguntar "quais os números?" sem copiar nada.

## 3. Virar artefato

Toda resposta do agente tem, no hover, os botões **virar spec** e
**virar proposta**. Eles criam a página no lugar certo, em rascunho, com
a origem apontando de volta pra conversa.

O título sai da primeira linha da resposta — então peça o título na
primeira linha quando quiser controlá-lo.

## 4. Revisar e aprovar

A página criada traz o embed de fluxo. Os botões mostram **só** as
transições que existem: de rascunho, o único caminho é "Em revisão".
Não há como pular pra execução.

Aprovar espelha o `status` no frontmatter, que é o campo que as
consultas filtram — é assim que a spec aparece no painel.

## 5. Deixar o agente escrever

Quando o agente for mexer no vault, ele **propõe**:

```
echo "conteúdo" | anotadinho-cli propor pages/nova.md --motivo "por quê"
anotadinho-cli propostas
anotadinho-cli aplicar <id>
```

Entre o primeiro e o terceiro comando, a página não existe. Uma página
`type: propostas` mostra o diff de tudo que está pendente, com Aplicar e
Recusar.

## 6. Ou deixar o agente se conectar

O caminho inverso: você trabalha no Claude Code e o Anotadinho é o
estado compartilhado.

```
anotadinho-cli --vault /caminho/do/vault mcp
```

Seis ferramentas, das quais **só `propor` escreve** — e ela também passa
pela revisão.

## Quando algo dá errado

| Sintoma | Causa provável |
|---|---|
| "não consegui executar" | executável fora do `PATH` |
| "passou de Ns e foi interrompido" | timeout curto pro tamanho da tarefa |
| "terminou sem escrever nada" | o agente escreveu no stderr, não no stdout |
| A resposta ignora a página aberta | o botão "com contexto" está desligado |
| A spec criada não aparece no painel | frontmatter quebrado — ver [[Ciclo 206 — Histórico de implementação dentro do vault]] |

## Relacionado

- [[Capacidades de agente]] — o que existe e os limites
- [[Uso agêntico do Anotadinho no dia a dia]] — o que ainda falta
- [[Guia do Agent OS]]
