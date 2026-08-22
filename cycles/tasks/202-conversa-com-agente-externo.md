---
id: "202"
titulo: "Conversa com agente externo, como página do vault"
status: done
criado: 2026-08-21
autor: humano
prioridade: alta
depende_de: [201]
estima_min: 240
agente_alvo: claude-opus
---

# Conversa com agente externo

## Objetivo

Poder conversar com o modelo configurado DENTRO do Anotadinho — pedir
spec, pedir proposta, ou só tirar dúvida sobre a página aberta.

## A decisão de desenho

**A conversa é uma página do vault**, não um banco interno. Isso permite
ligá-la ao trabalho (`[[Conversa sobre X]]` com backlink dos dois
lados), versioná-la no git, e deixar o agente lê-la como contexto —
porque é uma página como qualquer outra.

## Critérios de aceite

- [x] `crates/core/src/conversa.rs`: formato de heading por mensagem,
      parse, serialização e montagem de prompt. 11 testes.
- [x] `crates/core/src/agente.rs`: `Adaptador` configurável com
      `{prompt}`, validação e presets pra claude/codex/opencode.
      9 testes.
- [x] Comando `rodar_agente` no Tauri, com timeout que MATA o processo.
- [x] `type: conversa` mostra o painel, como `type: kanban` mostra o board.
- [x] Contexto automático: a página anterior vai junto, com botão pra
      desligar.
- [x] 9 cenários de harness usando um agente de MENTIRA.

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
node scripts/uitest/run.mjs
```

## Não-objetivos

- Sessão persistente com o agente: o piso é UM DISPARO, que qualquer CLI
  atende. Sessão entra depois como capacidade opcional.
- Streaming da resposta token a token.

## Segurança

- **Sem shell.** `Command::new(binario).args(...)` com o prompt como UM
  argumento. Aspas, quebras e `$(...)` no prompt são texto — há cenário
  provando isso.
- **Binário com espaço é recusado**: quase sempre significa uma linha de
  shell colada, e aceitar isso seria execução de shell pela porta dos
  fundos.
- **A configuração mora nas preferências**, nunca no vault. Uma página de
  terceiro não escolhe o que será executado.
- **Timeout mata o processo** em vez de deixá-lo pendurado.
