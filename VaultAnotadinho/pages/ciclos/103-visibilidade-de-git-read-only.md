---
title: Ciclo 103 — Visibilidade de git read-only
type: ciclo
ciclo: "103"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: []
tags:
- ciclo
---

# Ciclo 103 — Visibilidade de git read-only

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Visibilidade de git (read-only)

## Objetivo

Sexto e último ciclo do tema "agent-os readiness". Muitos vaults de
agent-os são versionados em git — hoje o Anotadinho não sabe nada sobre
isso (campo totalmente verde, confirmado na auditoria). Este ciclo dá
visibilidade BÁSICA e SOMENTE LEITURA: quantos arquivos modificados,
quais, sem nenhuma ação de commit/push pela UI.

## Critérios de aceite

- [x] Chama o binário `git` do sistema via `std::process::Command`
      (`git -C <vault> status --porcelain`) — SEM depender da crate
      `git2`/libgit2 (dependência pesada nova evitada de propósito)
- [x] Se `git` não estiver instalado OU o vault não for um repositório
      git, degrada silenciosamente (indicador de git simplesmente não
      aparece) — nunca trava nem mostra erro pro usuário por causa disso
- [x] Indicador na `HeaderBar` (contagem de arquivos modificados/não
      rastreados), atualizado no mesmo polling de 3s que já existe
      (`app.rs`, `check_changes`)
- [x] Clicar o indicador mostra a lista de arquivos modificados (nome +
      status M/A/D/??) num popover simples — sem diff de conteúdo nesta
      v1
- [x] `cargo test --workspace`, `cd ui && cargo test --lib`,
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

Implementado em `crates/vault/src/git_status.rs` (módulo pequeno, sem
crate nova). A guarda contra empilhar processos `git` ficou no
FRONTEND (`app.rs`, `Rc<RefCell<bool>>` no mesmo efeito de polling de
`check_changes`), não no backend — mais simples, e correto porque o
runtime WASM é single-threaded (não tem race de verdade entre
check-e-set).

Comando IPC `git_status` retorna `Option<Vec<GitFileEntry>>`
diretamente (sem `Result<_, String>`) — toda falha (git não instalado,
não é repo) já vira `None` dentro de `anotadinho_vault::git_status`,
então não tem "erro" pra propagar, só "não tem indicador pra mostrar".

Validado ao vivo via MCP `tauri`: indicador mostra "⎇ 14" (contagem
real do repo do projeto — o vault de demonstração é um subdiretório do
monorepo, então `git -C <vault> status` reflete o repo INTEIRO, não só
o vault; num deploy real de agent-os o vault normalmente É a raiz do
repo, cenário onde o indicador mostraria só as mudanças do vault).
Popover ao clicar lista os arquivos com status M/??  corretamente.

## Resultado

# Ciclo 103 - done

## Resumo

Sexto e último ciclo do tema "agent-os readiness". Visibilidade
SOMENTE LEITURA de git: indicador na `HeaderBar` com a contagem de
arquivos modificados/não rastreados (`git status --porcelain`),
atualizado no mesmo polling de 3s de `check_changes`. Clicar mostra um
popover com a lista de arquivos + status (M/A/D/R/??). Degrada
silenciosamente se `git` não estiver instalado ou o vault não for um
repositório — sem `git2`/libgit2, só o binário do sistema.

## Arquivos criados/modificados

- `crates/vault/src/git_status.rs` (novo) — `git_status()`,
  `GitFileEntry`, 8 testes
- `crates/vault/src/lib.rs` — exporta o módulo novo
- `crates/ipc/src/lib.rs` — `handle_git_status`
- `src-tauri/src/main.rs` — comando `git_status` registrado
- `ui/src/api.rs` — `GitFileEntry`, wrapper `git_status`
- `ui/src/app.rs` — estado `git_files`, busca inicial + polling com
  guarda contra empilhar processos `git`
- `ui/src/components/header_bar.rs` — indicador + popover
- `ui/src/styles/main.css` — `.git-status-wrapper`/`.git-status__*`

## Testes

`cargo test --workspace`: 82 (25 core + 1 ipc + 8 search + 48 vault).
`cd ui && cargo test --lib`: 66. Total 148.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri`: indicador "⎇ 14" refletindo o
estado real do repo; popover lista arquivos com status correto.

## Notas

Fecha o tema "agent-os readiness" (098-103): frontmatter customizável
+ painel de propriedades + templates + exportação em massa + busca
full-text na paleta + visibilidade de git. Detalhes/ressalvas no
arquivo de task.

Próximo: tema B (keyboard-only), começando pelo ciclo 104
(generalização do padrão de keymap e modal de captura de tecla).
