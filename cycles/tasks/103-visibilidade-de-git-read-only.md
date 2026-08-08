---
id: "103"
titulo: "Visibilidade de git read-only"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: []
estima_min: 90
agente_alvo: claude-sonnet
---

# Visibilidade de git (read-only)

## Objetivo

Sexto e último ciclo do tema "agent-os readiness". Muitos vaults de
agent-os são versionados em git — hoje o Anotadinho não sabe nada sobre
isso (campo totalmente verde, confirmado na auditoria). Este ciclo dá
visibilidade BÁSICA e SOMENTE LEITURA: quantos arquivos modificados,
quais, sem nenhuma ação de commit/push pela UI.

## Critérios de aceite

- [ ] Chama o binário `git` do sistema via `std::process::Command`
      (`git -C <vault> status --porcelain`) — SEM depender da crate
      `git2`/libgit2 (dependência pesada nova evitada de propósito)
- [ ] Se `git` não estiver instalado OU o vault não for um repositório
      git, degrada silenciosamente (indicador de git simplesmente não
      aparece) — nunca trava nem mostra erro pro usuário por causa disso
- [ ] Indicador na `HeaderBar` (contagem de arquivos modificados/não
      rastreados), atualizado no mesmo polling de 3s que já existe
      (`app.rs`, `check_changes`)
- [ ] Clicar o indicador mostra a lista de arquivos modificados (nome +
      status M/A/D/??) num popover simples — sem diff de conteúdo nesta
      v1
- [ ] `cargo test --workspace`, `cd ui && cargo test --lib`,
      `trunk build`, `cargo build --manifest-path src-tauri/Cargo.toml`
      passam

## Comandos de validação

```bash
cargo test --workspace
cd ui && cargo test --lib
cd ui && trunk build
cargo build --manifest-path src-tauri/Cargo.toml
```

## Não-objetivos

- Commit/push/stage pela UI — só leitura nesta fase; virar uma ação de
  escrita é um passo bem maior (mensagem de commit, autenticação de
  push, etc.) que merece seu próprio ciclo se for pedido depois
- Diff de conteúdo linha a linha — só a lista de arquivos + status
- Inicializar um repositório git novo pela UI — se o vault não é um
  repo, o indicador só não aparece; criar um é responsabilidade do
  usuário fora do app

## Notas

Rodar um processo externo (`git`) a partir do backend Tauri já é um
padrão novo pro projeto — colocar isso num módulo pequeno e isolado
(`crates/vault/src/git_status.rs` ou crate própria `crates/git_status`)
em vez de espalhar `Command::new("git")` em vários lugares. Tratar
timeout/processo travado com cuidado (não deixar o polling de 3s
empilhar processos `git` se um demorar) — um jeito simples: só disparar
a checagem de git se a anterior já tiver terminado (flag de "em
andamento", mesmo padrão de `AppWatchers` já usado por `check_changes`).
