---
title: Proposta de implementação — CLI junto do app instalado
tags:
- proposta
- portabilidade
type: proposta
date: 2026-10-06
status: em-revisao
---
# Proposta de implementação — CLI junto do app instalado

{{ type: "fluxo" }}
artefato: proposta
etapa: em-revisao
origem: pages/specs/cli-junto-do-app.md
{{ /fluxo }}

## Decisão pendente

Há dois caminhos com custos diferentes. Esta proposta recomenda o
primeiro, mas **a escolha é de quem aprova**: ela muda o que o projeto
entrega.

### A. O CLI como binário externo do pacote (recomendado)

O Tauri tem `bundle.externalBin`, que copia um binário pra junto do app
no instalador. É exatamente o lugar onde `cli_ao_lado` já procura.

O problema é que o `tauri-build` exige que o arquivo
(`binaries/anotadinho-cli-<alvo>[.exe]`) exista **em todo build**, e não
só no empacotamento. Pôr isso no `tauri.conf.json` quebraria o
`cargo build` de um checkout limpo, que é o RNF da spec.

Por isso o `externalBin` vai num arquivo à parte,
`src-tauri/tauri.bundle.json`, aplicado só no empacotamento:

```
cargo tauri build --config src-tauri/tauri.bundle.json
```

Um script `scripts/empacotar.(sh|ps1)` compila o CLI pro alvo, copia com
o sufixo do alvo e chama o comando acima. O dev e o `cargo build` não
mudam.

### B. O próprio app atende como CLI

`anotadinho mcp --vault X` entraria no `main` do app antes do Tauri.
Pra isso o `crates/cli` vira biblioteca, e `cli_ao_lado` passa a
devolver o próprio executável com o subcomando. Não há nada a empacotar.

O custo: no Windows o app é `windows_subsystem = "windows"`. Servir MCP
por stdio a partir de um processo sem console funciona com pipes
herdados, mas isso **não foi medido**, e não há onde medir aqui. A TUI
precisaria do mesmo, o que a faz depender do CLI inteiro.

## Comum aos dois

- RF3: se nem o vizinho nem o `PATH` têm o CLI, `cli_ao_lado` devolve
  "não achei", e a conversa registra um aviso em vez de passar um nome
  que não existe pro `--mcp-config`.

## Riscos

- A: o nome com sufixo de alvo é convenção do Tauri; errar o sufixo
  falha no empacotamento, não em silêncio.
- B: o ponto não medido do processo sem console, descrito acima.

## Etapas (A)

1. `tauri.bundle.json` e o script de empacotamento.
2. `cli_ao_lado` distingue "achei" de "não achei"; aviso na conversa.
3. Gerar o `.deb` e conferir que o CLI está dentro, e que o MCP sobe no
   app instalado.
