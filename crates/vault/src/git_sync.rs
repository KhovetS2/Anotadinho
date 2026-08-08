//! Sincronização via git (ciclo 119): pull e commit+push, sempre uma
//! ação EXPLÍCITA do usuário (nunca automática/em background, nunca
//! disparada ao salvar). Mesmo princípio de `git_status`/`git_log` —
//! shell out pro `git` do sistema via `std::process::Command`, sem
//! `git2`/`gix`. Diferente do resto do módulo de git (que degrada
//! silenciosamente pra `None` quando não há repo), aqui um erro É
//! informação que o usuário pediu (clicou o botão) e precisa ver —
//! por isso retorna `Result`, não `Option`.

use std::path::Path;

use anyhow::{bail, Result};

/// Roda `git -C <vault> pull`. Erro (conflito de merge, sem remote
/// configurado, vault não é um repo git, etc) retorna a mensagem de
/// stderr do git tal qual, pra UI mostrar direto.
pub fn git_pull(vault_path: &Path) -> Result<String> {
    run_git(vault_path, &["pull"])
}

/// Roda `git add -A && commit -m <message> && push -u origin HEAD`
/// em sequência — para no primeiro passo que falhar (ex: nada pra
/// commitar) com a mensagem correspondente. `-u origin HEAD` no push
/// funciona tanto pra branch já com upstream configurado (não muda
/// nada) quanto pro primeiro push de um branch novo (configura o
/// tracking), sem precisar saber o nome do branch de antemão.
pub fn git_commit_and_push(vault_path: &Path, message: &str) -> Result<String> {
    run_git(vault_path, &["add", "-A"])?;
    let commit_out = run_git(vault_path, &["commit", "-m", message])?;
    let push_out = run_git(vault_path, &["push", "-u", "origin", "HEAD"])?;
    Ok(format!("{}\n{}", commit_out, push_out))
}

fn run_git(vault_path: &Path, args: &[&str]) -> Result<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(vault_path)
        .args(args)
        .output()
        .map_err(|e| anyhow::anyhow!("erro ao rodar git: {}", e))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        let msg = if !stderr.trim().is_empty() { stderr.trim().to_string() } else { stdout.trim().to_string() };
        bail!("{}", msg);
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::Command;

    fn run(dir: &Path, args: &[&str]) {
        Command::new("git").arg("-C").arg(dir).args(args).output().unwrap();
    }

    fn git_available() -> bool {
        Command::new("git").arg("--version").output().is_ok()
    }

    #[test]
    fn git_pull_fails_on_non_repo() {
        if !git_available() {
            return;
        }
        let dir = tempfile::TempDir::new().unwrap();
        assert!(git_pull(dir.path()).is_err());
    }

    /// Fixture: um "remoto" bare + um clone local com identidade
    /// configurada. `None` se `git clone` falhar (git muito antigo,
    /// ou ambiente sem suporte a clone local — pula o teste).
    fn setup_remote_and_clone() -> Option<(tempfile::TempDir, tempfile::TempDir)> {
        let remote_dir = tempfile::TempDir::new().unwrap();
        run(remote_dir.path(), &["init", "--bare", "--quiet"]);

        let local_dir = tempfile::TempDir::new().unwrap();
        let clone_ok = Command::new("git")
            .args(["clone", "--quiet"])
            .arg(remote_dir.path())
            .arg(local_dir.path())
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);
        if !clone_ok {
            return None;
        }
        run(local_dir.path(), &["config", "user.email", "test@example.com"]);
        run(local_dir.path(), &["config", "user.name", "Test"]);
        Some((remote_dir, local_dir))
    }

    #[test]
    fn git_commit_and_push_full_roundtrip() {
        if !git_available() {
            return;
        }
        let Some((remote_dir, local_dir)) = setup_remote_and_clone() else { return };

        std::fs::write(local_dir.path().join("nota.md"), "conteudo").unwrap();
        let result = git_commit_and_push(local_dir.path(), "primeira nota");
        assert!(result.is_ok(), "commit+push falhou: {:?}", result.err());

        let check_dir = tempfile::TempDir::new().unwrap();
        Command::new("git")
            .args(["clone", "--quiet"])
            .arg(remote_dir.path())
            .arg(check_dir.path())
            .output()
            .unwrap();
        assert!(check_dir.path().join("nota.md").exists());
    }

    #[test]
    fn git_commit_and_push_fails_when_nothing_to_commit() {
        if !git_available() {
            return;
        }
        let Some((_remote_dir, local_dir)) = setup_remote_and_clone() else { return };
        assert!(git_commit_and_push(local_dir.path(), "vazio").is_err());
    }

    #[test]
    fn git_pull_succeeds_on_clean_clone() {
        if !git_available() {
            return;
        }
        let Some((_remote_dir, local_dir)) = setup_remote_and_clone() else { return };
        std::fs::write(local_dir.path().join("nota.md"), "x").unwrap();
        git_commit_and_push(local_dir.path(), "inicial").unwrap();
        // Pull num repo já atualizado (nada novo pra trazer) deve dar certo.
        assert!(git_pull(local_dir.path()).is_ok());
    }
}
