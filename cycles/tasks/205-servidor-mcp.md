---
id: "205"
titulo: "Servidor MCP expondo o vault"
status: done
criado: 2026-08-22
autor: humano
prioridade: media
depende_de: [204]
estima_min: 150
agente_alvo: claude-opus
---

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
