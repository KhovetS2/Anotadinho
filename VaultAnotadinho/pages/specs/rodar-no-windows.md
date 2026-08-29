---
title: Rodar no Windows
type: spec
date: 2026-08-29
status: rascunho
prioridade: media
tags:
- spec
- portabilidade
---

{{ type: "fluxo" }}
artefato: spec
etapa: rascunho
{{ /fluxo }}

## Por que isto existe

A configuração de agentes foi construída e testada só no Linux. Esta spec
registra o levantamento do que impede o Anotadinho de rodar no Windows,
para a decisão de portar ser tomada com o custo à vista em vez de no
escuro. **Não é um pedido de implementação** — é o mapa.

macOS foi levantado junto: nada bloqueia lá.

{{ type: "callout" }}
variant: warning
title: Nada disto foi testado numa máquina Windows
body: |
  O levantamento é por leitura de código. Os bloqueios listados são
  consequências diretas do que está escrito, mas a ordem em que aparecem
  na prática, e o que mais surge depois de resolvê-los, só se descobre
  rodando de verdade.
{{ /callout }}

## O que bloqueia

**B1. O binário do agente não é executável por `Command::new`.**
`claude`, `codex` e `opencode` são instalados por npm como shims `.cmd`.
`CreateProcessW` resolve `.exe`, não `.cmd`/`.bat`. O spawn em
`src-tauri/src/main.rs` falha antes de qualquer coisa.

**B2. A validação recusa caminho com espaço.**
`ProblemaConfig::BinarioComEspaco`, em `crates/core/src/agente.rs`, rejeita
binário cujo caminho contenha espaço. No Windows o caminho canônico é
`C:\Program Files\...`. A configuração legítima é recusada.

**B3. Não há como contornar B1 e B2 pela interface.**
Não existe campo para editar `binario`, `args` ou `timeout_s`: a conversa
só permite trocar de preset e escolher pastas. A única forma de apontar
outro executável é editar o `localStorage` na mão. Isto é um problema
**mesmo no Linux** — só não aparece porque os presets funcionam lá.

**B4. Separador de caminho.**
`strip_prefix` em `crates/vault/src/io.rs` produz caminhos relativos com
`\`, e cerca de dez lugares comparam com `/` literal: a hierarquia da
sidebar, a descoberta de prompts padrão (`prompt_padrao::descobrir`), a
exportação de pasta, a detecção de `journals/`, a resolução de wikilink e
as chaves do cache de índice. Nenhum deles dá erro — todos simplesmente
param de casar, em silêncio.

**B5. Build local.**
`beforeDevCommand` e `beforeBuildCommand` em `src-tauri/tauri.conf.json`
usam sintaxe POSIX, executada por `cmd.exe`.

## O que degrada, sem bloquear

**D1.** `proc.kill()` mata só o filho direto. `claude` e `codex` são
wrappers Node que criam subprocessos; sem *job object* no Windows, os
netos sobrevivem ao cancelamento e ao timeout. O mesmo vale no Linux, onde
o processo órfão costuma morrer ao fechar o pipe — mas não é garantia.

**D2.** O watcher faz `strip_prefix` de um caminho UNC. Se o `notify`
devolver caminho não-UNC, o `strip_prefix` falha e um caminho absoluto
passa como `VaultEvent.path`.

## Cosmético

`contornar_travamento_nvidia` testa `/sys/module/nvidia/version` e vira
no-op fora do Linux. Merece um `#[cfg(target_os = "linux")]` por higiene.
Os `scripts/*.sh` são ferramenta de desenvolvimento, não do app.

## Ordem sugerida

B3 primeiro, porque é o único que também melhora o Linux de hoje e é o
que destrava testar os outros à mão. Depois B4, que é mecânico e grande.
B1 e B2 são pequenos e vêm juntos. B5 por último, e só para quem for
compilar no Windows.

## Fora de escopo

- Empacotamento e instalador para Windows
- CI em Windows
- Testar a suíte de harness em outro sistema
