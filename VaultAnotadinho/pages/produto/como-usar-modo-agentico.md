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

## 3.1 Spec e proposta são coisas diferentes

Vale insistir, porque é o que mantém o ciclo coerente:

| | Spec | Proposta |
|---|---|---|
| Responde | o **quê** e o **porquê** | o **como** |
| Contém | requisitos e critérios de aceite | abordagem, etapas, alternativas |
| Quando muda | quando o problema muda | quando a abordagem não serve |

Recusar uma proposta **não** recusa o trabalho: a spec continua valendo,
e você pede outra abordagem. Uma spec que já diz como fazer engessa a
implementação antes de alguém pensar nela.

## 4. Revisar e aprovar

A página criada traz o embed de fluxo. Os botões mostram **só** as
transições que existem: de rascunho, o único caminho é "Em revisão".
Não há como pular pra execução.

Aprovar espelha o `status` no frontmatter, que é o campo que as
consultas filtram — é assim que a spec aparece no painel.

## 4.1 Da spec aprovada pra proposta

Numa spec **aprovada** aparece o botão **Planejar implementação**. Ele
abre uma conversa com a spec já anexada e a pergunta pronta — e é ali
que você anexa as **páginas de padrão** que a proposta terá que
respeitar. A resposta vira proposta com "virar proposta".

A pergunta traz uma trava explícita: o modelo não deve propor requisito
novo nem mudar o escopo. Se achar algo ambíguo, ele aponta em vez de
decidir sozinho.

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

## 5.1 O aviso de proposta pendente

Quando há proposta esperando, um indicador aparece no **cabeçalho**, de
qualquer página. Clicar leva pra tela de revisão. Ele conta propostas
vindas de qualquer canal — UI, CLI ou MCP — e some quando a fila zera.

## 5.2 Executar a proposta aprovada

Numa proposta **aprovada** o botão vira **Executar**. Ele abre a conversa
com a proposta anexada e uma pergunta diferente da de planejar: aqui a
abordagem já foi aceita, então o que se pede é o trabalho e o relato.

A trava também é outra. No planejamento, o modelo não pode mudar o
escopo; na execução, não pode mudar a abordagem — se ela se mostrar
inviável, ele deve **parar e explicar**, porque mudar de rumo exige uma
proposta nova.

A resposta vira o registro com "virar execução".

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
