---
title: Ciclo 205 — Servidor MCP expondo o vault
type: ciclo
ciclo: "205"
status: concluida
date: 2026-08-22
prioridade: media
depende_de: [204]
tags:
- ciclo
---

# Ciclo 205 — Servidor MCP expondo o vault

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Servidor MCP

## Objetivo

O sentido de integração complementar ao ciclo 202: lá o Anotadinho CHAMA
o agente; aqui o agente se conecta e opera o vault, com o Anotadinho como
estado compartilhado. É o que permite usar o Claude Code (ou outro
cliente MCP) trabalhando dentro do protocolo de fluxo.

## Critérios de aceite

- [x] `anotadinho-cli mcp` — JSON-RPC 2.0 por stdio.
- [x] `initialize`, `tools/list`, `tools/call`, `ping`.
- [x] Ferramentas: `listar_paginas`, `ler_pagina`, `buscar`,
      `consultar`, `propor`, `propostas_pendentes`.
- [x] **A única escrita é `propor`** — teste reprova se alguém expuser
      escrita direta.
- [x] JSON quebrado não derruba o servidor.
- [x] Notificação (sem `id`) não gera resposta.
- [x] 8 testes.

## Comandos de validação

```bash
cargo test -p anotadinho-cli
cargo test --workspace
```

## Como conectar

No cliente MCP, um servidor stdio:

```json
{
  "command": "anotadinho-cli",
  "args": ["--vault", "/caminho/do/VaultAnotadinho", "mcp"]
}
```

## Não-objetivos

- Recursos e prompts do MCP (só ferramentas por ora).
- Transporte por HTTP/SSE.

## A escolha que define a segurança

As ferramentas de leitura são diretas; a de escrita é `propor`, não
`escrever`. Um agente conectado não grava página nenhuma sozinho: o que
ele propuser aparece na tela de revisão (ciclo 204) e só vira arquivo
depois de um clique humano.

O teste `mcp_lista_as_ferramentas_e_a_unica_escrita_e_propor` existe pra
essa garantia não se perder num ciclo futuro por descuido.

## Resultado

# 205 — Servidor MCP

## O que mudou

- `crates/cli/src/mcp.rs` (novo): servidor JSON-RPC 2.0 por stdio, com
  6 ferramentas.
- `crates/cli/src/main.rs`: subcomando `mcp`.
- 8 testes em `crates/cli/tests/cli.rs` (51 no total).

## Conferido à mão

```
$ anotadinho-cli --vault <v> mcp
1 initialize -> anotadinho 2024-11-05
2 tools -> listar_paginas, ler_pagina, buscar, consultar, propor, propostas_pendentes
4 propor -> proposta pages-nova-md-... criada. A página NÃO foi escrita.
```

Depois disso, `pages/nova.md` não existe e há uma proposta pendente — que
é exatamente o comportamento pretendido.

## Validação

- `cargo test --workspace`: 0 falhas; `cargo test -p anotadinho-cli`: 51.
- `node scripts/uitest/run.mjs`: **117/117 em 280.5s**, vault limpo.
