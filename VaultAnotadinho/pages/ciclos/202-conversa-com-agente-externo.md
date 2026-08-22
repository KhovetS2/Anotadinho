---
title: Ciclo 202 — Conversa com agente externo, como página do vault
type: ciclo
ciclo: "202"
status: concluida
date: 2026-08-21
prioridade: alta
depende_de: [201]
tags:
- ciclo
---

# Ciclo 202 — Conversa com agente externo, como página do vault

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

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

## Resultado

# 202 — Conversa com agente externo

## O que mudou

- `crates/core/src/conversa.rs` (novo): mensagens como headings
  markdown, `parse`/`serializar`/`append` e `montar_prompt` (contexto →
  histórico → pergunta; corta as mensagens mais ANTIGAS). 11 testes.
- `crates/core/src/agente.rs` (novo): `Adaptador` com `{prompt}`,
  validação e presets. 9 testes.
- `src-tauri/src/main.rs`: comando `rodar_agente` — sem shell, com
  timeout que mata o processo.
- `ui/src/components/conversa_view.rs` (novo): painel de conversa.
- `ui/src/state.rs`: `agora_legivel`, `load_adaptador`, `save_adaptador`.
- `scripts/uitest/agente-falso.sh` (novo) + 9 cenários.

## Por que um agente de mentira nos testes

O que se testa é o CONTRATO — prompt chega inteiro, saída volta, timeout
mata, falha é reportada, texto perigoso não vira comando. Usar
claude/codex de verdade tornaria a suíte lenta, cara e não
determinística.

## Dois bugs meus, com o mesmo padrão

1. `abrirPagina` do harness esperava o cabeçalho do EDITOR, e a conversa
   tem cabeçalho próprio.
2. **Handle de estado congelado** — de novo. A resposta do agente era
   montada a partir de `(*mensagens)`, o handle capturado no closure, que
   ainda tinha a lista de ANTES da pergunta: a pergunta sumia da tela.
   É o mesmo bug dos ciclos 155, 157 e 201, e o comentário agora aponta
   pros quatro.

## Validação

- `cargo test --workspace`: 0 falhas; `ui`: 46 testes.
- `trunk build` ok; Tauri: 0 erros.
- `node scripts/uitest/run.mjs`: **111/111 em 270.5s**.
