---
id: "529"
titulo: "O Claude usa as ferramentas do Anotadinho (propor, ler_pagina) no -p, e o bloco de código da resposta quebra a linha dentro do balão"
status: done
criado: 2026-10-26
autor: agente
prioridade: alta
depende_de: ["426", "528"]
estima_min: 60
---

# 529 — "o acesso às ferramentas do Anotadinho não foi autorizado"

Relato (com print), na conversa `tarefa-001` do projeto
`plataforma-artesaos`:

1. O Claude respondeu "não consegui propor a alteração no vault: o acesso
   às ferramentas do Anotadinho (onde_posso_escrever, ler_pagina) não foi
   autorizado" e devolveu o texto pra colar.
2. O bloco de código com esse texto vazou pra fora do balão da mensagem e
   da coluna (linhas longas, sem quebra).

Causa de 1, medida em 2026-10-26: no `-p`, o Claude Code NEGA toda
ferramenta MCP que não foi liberada; `--permission-mode acceptEdits` só
libera edição de arquivo. Com `--mcp-config` e sem liberar, a mesma
pergunta ("use ler_pagina…") deu NEGADA; com
`--allowedTools=mcp__anotadinho`, leu a página — e `propor` criou a
proposta na fila.

## Critérios de aceite

- [x] Quando o app liga o servidor MCP no dialeto do Claude Code
      (`--mcp-config`), libera junto as ferramentas dele
      (`--allowedTools=mcp__anotadinho`, um argumento só: o `--allowedTools`
      aceita vários valores e engoliria o seguinte). Vale pra qualquer
      agente com esse `arg_mcp` — os presets e os que a pessoa copiou
      deles —, e só quando há config MCP.
- [x] O agente de teste pula o argumento novo, como já pulava o
      `--mcp-config`.
- [x] O `pre` da mensagem quebra a linha e fica dentro do balão; o balão
      não passa da coluna. Cenário de harness com o texto do relato.

## Comandos de validação

```bash
cargo test --workspace
node scripts/uitest/run.mjs "(529)"
node scripts/uitest/run.mjs
```

## Não-objetivos

- O agente revisor também ganha as ferramentas MCP: `propor` dele vai pra
  fila como a de qualquer agente, e ler o vault é o que ele precisa.
