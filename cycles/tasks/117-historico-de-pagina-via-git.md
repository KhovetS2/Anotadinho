---
id: "117"
titulo: "Historico de pagina via git"
status: pending
criado: 2026-08-08
autor: humano
prioridade: media
depende_de: ["103"]
estima_min: 60
agente_alvo: claude-sonnet
---

# Histórico de página via git

## Objetivo

`crates/vault/src/git_status.rs` (ciclo 103) já lê `git status`. Como
o vault normalmente é uma pasta versionada, dá pra mostrar o
histórico real de commits que tocaram uma página — bem mais robusto
que o undo em memória (ciclo 095, perdido ao recarregar). Novo painel
no editor ("⋯" → "Histórico") lista os commits que tocaram o arquivo
atual.

## Critérios de aceite

- [ ] `crates/vault/src/git_status.rs` ganha `git_log(vault_root,
      relative_path) -> Option<Vec<GitLogEntry>>` (hash curto, data,
      mensagem), via `git -C <root> log --follow --oneline -- <path>`;
      `None` se não for um repo git (mesmo padrão de `git_status`)
- [ ] Handler IPC + comando Tauri novos, expostos em `ui/src/api.rs`
- [ ] Painel "Histórico" no menu "⋯" do editor, lista os commits (mais
      recente primeiro)
- [ ] Se o vault não é um repo git, o item do menu não aparece (ou
      mostra estado vazio claro) — mesmo tratamento de ausência que o
      indicador de git status já tem
- [ ] Teste em `crates/vault` cobrindo `git_log` num fixture de repo
      git real (`tempfile` + `git init` + commits, mesmo padrão dos
      testes de `git_status.rs`)
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

- Diff visual entre versões (mostrar o conteúdo de um commit
  específico) — v1 é só a lista de commits com hash/data/mensagem;
  ver o conteúdo antigo fica pra outro ciclo (o usuário pode sempre
  usar `git show <hash>:<path>` fora do app enquanto isso)
- Reverter/restaurar uma versão antiga pela UI — só visualização

## Notas

Mesmo padrão de `git_status()` — shell out pro `git` do sistema via
`std::process::Command`, sem dependência de lib git nova
(`git2`/`gix`). Painel reaproveita o padrão visual do popover de git
status do `header_bar.rs` (ciclo 103).
