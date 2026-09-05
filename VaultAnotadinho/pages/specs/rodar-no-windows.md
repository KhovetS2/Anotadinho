---
title: Rodar no Windows
type: spec
date: 2026-08-29
status: concluida
prioridade: media
tags:
- spec
- portabilidade
---
{{ type: "fluxo" }}
artefato: spec
etapa: concluida
{{ /fluxo }}
## Por que isto existe

A configuração de agentes foi construída e testada só no Linux. Esta spec
nasceu como o levantamento do que impede o Anotadinho de rodar no
Windows, para a decisão de portar ser tomada com o custo à vista em vez
de no escuro.

macOS foi levantado junto: nada bloqueia lá.

## O que foi feito

Todos os itens levantados estão resolvidos. Dois deles saíram pelo
caminho, em ciclos que miravam outra coisa:

| Item | Onde foi resolvido |
|---|---|
| B1 — shim `.cmd` do npm | ciclo 255, `agente::resolver_executavel` |
| B2 — caminho com espaço recusado | ciclo 241 |
| B3 — sem campo pra editar o binário | ciclo 239 |
| B4 — separador de caminho | ciclo 255, `vault::caminho` |
| B5 — `beforeDevCommand` POSIX | ciclo 255, `tauri.windows.conf.json` |
| D1 — `kill` não alcança os netos | ciclo 255, `grupo_de_processos` |
| D2 — `strip_prefix` de caminho UNC | ciclo 255, `vault::caminho` |
| Cosmético — `contornar_travamento_nvidia` | ciclo 255 |
{{ type: "callout" }}
variant: warning
title: Nada disto foi testado numa máquina Windows
body: |
  Nem o levantamento nem as correções passaram por uma máquina Windows —
  o projeto não tem uma. O que existe é `cargo check` cruzado para
  `x86_64-pc-windows-gnu`, que compila os ramos `#[cfg(windows)]` de
  verdade e por isso pega erro de API, e teste unitário da lógica pura,
  com o sistema entrando por PARÂMETRO em vez de `cfg!` — é o que faz o
  comportamento do Windows ser exercido numa máquina Linux.

  Isso cobre "compila" e "a regra está certa". Não cobre "roda". O que
  aparece só rodando — ordem dos problemas, permissões, antivírus,
  console — continua por descobrir.
{{ /callout }}
## O que bloqueia

**B1. O binário do agente não é executável por `Command::new`.**`claude`, `codex` e `opencode` são instalados por npm como shims `.cmd`.
`CreateProcessW` resolve `.exe`, não `.cmd`/`.bat`. O spawn em
`src-tauri/src/main.rs` falha antes de qualquer coisa.

**B2. A validação recusa caminho com espaço.**`ProblemaConfig::BinarioComEspaco`, em `crates/core/src/agente.rs`, rejeita
binário cujo caminho contenha espaço. No Windows o caminho canônico é
`C:\Program Files\...`. A configuração legítima é recusada.

**B3. Não há como contornar B1 e B2 pela interface.**
Não existe campo para editar `binario`, `args` ou `timeout_s`: a conversa
só permite trocar de preset e escolher pastas. A única forma de apontar
outro executável é editar o `localStorage` na mão. Isto é um problema
**mesmo no Linux** — só não aparece porque os presets funcionam lá.

**B4. Separador de caminho.**`strip_prefix` em `crates/vault/src/io.rs` produz caminhos relativos com
`\`, e cerca de dez lugares comparam com `/` literal: a hierarquia da
sidebar, a descoberta de prompts padrão (`prompt_padrao::descobrir`), a
exportação de pasta, a detecção de `journals/`, a resolução de wikilink e
as chaves do cache de índice. Nenhum deles dá erro — todos simplesmente
param de casar, em silêncio.

**B5. Build local.**`beforeDevCommand` e `beforeBuildCommand` em `src-tauri/tauri.conf.json`
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
