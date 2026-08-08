---
id: "119"
titulo: "Sincronizacao via git pull e commit mais push"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["103"]
estima_min: 75
agente_alvo: claude-sonnet
---

# Sincronização via git (pull / commit + push)

## Objetivo

Hoje sync entre máquinas é literalmente copiar a pasta pro pendrive
(`pages/sync.md` do próprio vault documenta isso). Pra vaults que já
são um repo git (cada vez mais comum dado o ciclo 103), um botão
"Pull" e um "Commit + Push" no popover de git status do header
resolve sync real com o menor esforço possível — sem inventar um
protocolo de sync novo.

## Critérios de aceite

- [ ] `crates/vault` ganha `git_pull(vault_root) -> Result<String>` e
      `git_commit_and_push(vault_root, message) -> Result<String>`,
      shell out pro `git` do sistema (`git -C <root> pull` /
      `git -C <root> add -A && git commit -m <msg> && git push`)
- [ ] Handlers IPC + comandos Tauri novos
- [ ] Popover de git status (`header_bar.rs`, ciclo 103) ganha botões
      "Pull" e "Commit + Push" (este último abre um `PendingDialog::Prompt`
      pra mensagem de commit)
- [ ] Erros (conflito de merge, sem remote configurado, etc) mostram
      a mensagem de erro do git tal qual, num `PendingDialog::Alert`
      — sem tentar resolver conflito automaticamente
- [ ] Teste em `crates/vault` cobrindo pull/commit+push num fixture de
      2 repos git locais (`tempfile`, um "remoto" bare + um clone)
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
