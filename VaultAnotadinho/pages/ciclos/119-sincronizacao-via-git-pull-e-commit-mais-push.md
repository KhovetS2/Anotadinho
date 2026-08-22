---
title: Ciclo 119 — Sincronizacao via git pull e commit mais push
type: ciclo
ciclo: "119"
status: concluida
date: 2026-08-08
prioridade: media
depende_de: ["103"]
tags:
- ciclo
---

# Ciclo 119 — Sincronizacao via git pull e commit mais push

{{ type: "fluxo" }}
artefato: execucao
etapa: concluida
{{ /fluxo }}

# Sincronização via git (pull / commit + push)

## Objetivo

Hoje sync entre máquinas é literalmente copiar a pasta pro pendrive
(`pages/sync.md` do próprio vault documenta isso). Pra vaults que já
são um repo git (cada vez mais comum dado o ciclo 103), um botão
"Pull" e um "Commit + Push" no popover de git status do header
resolve sync real com o menor esforço possível — sem inventar um
protocolo de sync novo.

## Critérios de aceite

- [x] `crates/vault` ganha `git_pull(vault_root) -> Result<String>` e
      `git_commit_and_push(vault_root, message) -> Result<String>`
      (novo módulo `git_sync.rs`), shell out pro `git` do sistema
- [x] Handlers IPC + comandos Tauri novos
- [x] Popover de git status (`header_bar.rs`, ciclo 103) ganha botões
      "Pull" e "Commit + Push" (este último abre um `PendingDialog::Prompt`
      pra mensagem de commit)
- [x] Erros (conflito de merge, sem remote configurado, etc) mostram
      a mensagem de erro do git tal qual, num `PendingDialog::Alert`
      — sem tentar resolver conflito automaticamente
- [x] Teste em `crates/vault` cobrindo pull/commit+push num fixture de
      2 repos git locais (`tempfile`, um "remoto" bare + um clone) —
      4 testes novos, incluindo roundtrip completo (push de verdade
      e confirmação no "remoto")
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

- Resolução de conflito de merge pela UI — se der conflito, mostra o
  erro e a pessoa resolve via `git` normal fora do app
- Configurar remote/autenticação (SSH key, token) pela UI — assume
  que o `git` do sistema já está configurado (mesma soluação de
  `git_status`, que também só funciona se já é um repo)
- Sync automático em background/no save — sempre uma ação explícita
  do usuário (botão), nunca implícita

## Notas

Reaproveita 100% a UI do popover de git status já existente (ciclo
103) — só adiciona botões nele, não cria um painel novo.

`push` usa `-u origin HEAD` (não só `push`) — funciona tanto pra
branch já com upstream configurado quanto pro primeiro push de um
branch novo, sem precisar saber o nome do branch de antemão (evita o
erro "current branch has no upstream branch" no primeiro push depois
de um clone fresco).

Validado ao vivo via MCP `tauri` com um vault de teste isolado
(`/tmp/git-sync-test/`, remoto bare + clone local — NÃO o vault real
do projeto, pra não arriscar mexer no repo de verdade): editei uma
página, "Commit + Push" pediu a mensagem e o commit apareceu tanto no
clone local quanto no "remoto" bare (conferido com `git log`
diretamente); testei "Pull" simulando mudança de outra máquina (um
segundo clone commitando+pushando) e o conteúdo apareceu no vault
aberto no app depois do pull, sem erro. Indicador de git status
(`⎇ N`) atualiza imediatamente após qualquer uma das duas ações (não
espera o próximo tick do polling de 3s).

## Resultado

# Ciclo 119 - done

## Resumo

Botões "Pull" e "Commit + Push" no popover de git status do header —
sync real entre máquinas pra vaults versionados, sem inventar
protocolo novo (shell out pro `git` do sistema, mesmo princípio de
`git_status`/`git_log`). Antes, sync era "copie a pasta pro pendrive".

## Arquivos criados/modificados

- `crates/vault/src/git_sync.rs` (novo) — `git_pull`,
  `git_commit_and_push`, 4 testes (roundtrip completo com remoto bare)
- `crates/vault/src/lib.rs` — re-exporta
- `crates/ipc/src/lib.rs` — `handle_git_pull`, `handle_git_commit_and_push`
- `src-tauri/src/main.rs` — comandos Tauri `git_pull`, `git_commit_and_push`
- `ui/src/api.rs` — wrappers
- `ui/src/components/header_bar.rs` — botões no popover, prompt de
  mensagem de commit, tratamento de erro
- `ui/src/app.rs` — `on_git_changed` (rebusca status + lista de
  páginas imediatamente após qualquer ação)
- `ui/src/styles/main.css` — `.git-status__actions`

## Testes

`cargo test --workspace`: 111. `cd ui && cargo test --lib`: 75. Total 186.
`trunk build` + `cargo build --manifest-path src-tauri/Cargo.toml`: OK.

Validação ao vivo via MCP `tauri` com vault de teste isolado (repo
bare + clone, fora do vault real do projeto): commit+push completo
(mensagem pedida via prompt, commit apareceu no "remoto" de verdade)
e pull (mudança de outra "máquina" simulada apareceu no vault aberto).

## Notas

Push usa `-u origin HEAD` — cobre tanto branch já rastreado quanto o
primeiro push de um branch novo.

Próximo: grafo visual de backlinks (120) e embed de PDF (121).
