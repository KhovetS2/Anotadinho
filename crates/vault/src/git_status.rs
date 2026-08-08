//! Visibilidade de git, SOMENTE LEITURA: chama o binário `git` do
//! sistema via `std::process::Command` — de propósito SEM depender de
//! `git2`/libgit2 (dependência pesada nova evitada). Se `git` não
//! estiver instalado ou o vault não for um repositório, degrada pra
//! `None` silenciosamente — nunca erro pro usuário.

use std::path::Path;

use serde::{Deserialize, Serialize};

/// Uma linha de `git status --porcelain`: path + status resumido
/// (`M`/`A`/`D`/`R`/`??`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitFileEntry {
    /// Path relativo ao repositório (= relativo ao vault, já que o
    /// vault É a raiz do repo nesse uso).
    pub path: String,
    /// Status resumido: `M` (modificado), `A` (adicionado), `D`
    /// (removido), `R` (renomeado), `??` (não rastreado).
    pub status: String,
}

/// Roda `git -C <vault> status --porcelain` e retorna a lista de
/// arquivos com mudanças. `None` se `git` não estiver instalado, o
/// path não for um repositório git, ou qualquer outro erro — nunca
/// propaga erro pro chamador, só "não tem git pra mostrar aqui".
pub fn git_status(vault_path: &Path) -> Option<Vec<GitFileEntry>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(vault_path)
        .arg("status")
        .arg("--porcelain")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().filter_map(parse_porcelain_line).collect())
}

fn parse_porcelain_line(line: &str) -> Option<GitFileEntry> {
    if line.len() < 4 {
        return None;
    }
    let xy = &line[0..2];
    let path = line[3..].to_string();
    Some(GitFileEntry { path, status: classify(xy).to_string() })
}

/// Um commit do histórico de uma página específica: hash curto, data
/// (`YYYY-MM-DD`) e mensagem.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitLogEntry {
    /// Hash curto do commit.
    pub hash: String,
    /// Data do commit (`YYYY-MM-DD`).
    pub date: String,
    /// Mensagem do commit (primeira linha).
    pub message: String,
}

/// Roda `git -C <vault> log --follow -- <path>` pra listar os commits
/// que tocaram uma página específica (mais recente primeiro,
/// `--follow` acompanha renomeações). `None` nas mesmas condições de
/// `git_status` (sem `git`, vault não é um repo). Lista vazia (não
/// `None`) se é um repo git mas o arquivo nunca foi commitado.
pub fn git_log(vault_path: &Path, relative_path: &str) -> Option<Vec<GitLogEntry>> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(vault_path)
        .arg("log")
        .arg("--follow")
        .arg("--date=short")
        .arg("--pretty=format:%h%x09%ad%x09%s")
        .arg("--")
        .arg(relative_path)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Some(stdout.lines().filter_map(parse_log_line).collect())
}

fn parse_log_line(line: &str) -> Option<GitLogEntry> {
    let mut parts = line.splitn(3, '\t');
    let hash = parts.next()?.to_string();
    let date = parts.next()?.to_string();
    let message = parts.next().unwrap_or("").to_string();
    Some(GitLogEntry { hash, date, message })
}

fn classify(xy: &str) -> &'static str {
    if xy == "??" {
        "??"
    } else if xy.contains('A') {
        "A"
    } else if xy.contains('D') {
        "D"
    } else if xy.contains('R') {
        "R"
    } else if xy.contains('M') {
        "M"
    } else {
        "?"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_porcelain_line_modified() {
        let e = parse_porcelain_line(" M src/main.rs").unwrap();
        assert_eq!(e.path, "src/main.rs");
        assert_eq!(e.status, "M");
    }

    #[test]
    fn parse_porcelain_line_untracked() {
        let e = parse_porcelain_line("?? novo.txt").unwrap();
        assert_eq!(e.path, "novo.txt");
        assert_eq!(e.status, "??");
    }

    #[test]
    fn parse_porcelain_line_added() {
        let e = parse_porcelain_line("A  staged.txt").unwrap();
        assert_eq!(e.status, "A");
    }

    #[test]
    fn parse_porcelain_line_deleted() {
        let e = parse_porcelain_line(" D removido.txt").unwrap();
        assert_eq!(e.status, "D");
    }

    #[test]
    fn parse_porcelain_line_too_short_returns_none() {
        assert!(parse_porcelain_line("MM").is_none());
    }

    #[test]
    fn git_status_none_for_non_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        assert!(git_status(dir.path()).is_none());
    }

    #[test]
    fn git_status_some_empty_for_clean_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        let init = std::process::Command::new("git")
            .arg("-C").arg(dir.path()).arg("init").arg("--quiet")
            .output();
        if init.is_err() {
            return; // git não instalado no ambiente de teste — pula
        }
        let status = git_status(dir.path());
        assert_eq!(status, Some(Vec::new()));
    }

    #[test]
    fn git_status_lists_untracked_file() {
        let dir = tempfile::TempDir::new().unwrap();
        let init = std::process::Command::new("git")
            .arg("-C").arg(dir.path()).arg("init").arg("--quiet")
            .output();
        if init.is_err() {
            return;
        }
        std::fs::write(dir.path().join("novo.md"), "conteudo").unwrap();
        let status = git_status(dir.path()).unwrap();
        assert_eq!(status.len(), 1);
        assert_eq!(status[0].path, "novo.md");
        assert_eq!(status[0].status, "??");
    }

    /// Inicializa um repo git de teste com identidade configurada
    /// (necessário pra `git commit` funcionar em CI sem `~/.gitconfig`).
    /// Retorna `None` se `git` não estiver instalado no ambiente.
    fn init_repo_with_identity(dir: &std::path::Path) -> Option<()> {
        let run = |args: &[&str]| {
            std::process::Command::new("git").arg("-C").arg(dir).args(args).output()
        };
        run(&["init", "--quiet"]).ok()?;
        run(&["config", "user.email", "test@example.com"]).ok()?;
        run(&["config", "user.name", "Test"]).ok()?;
        Some(())
    }

    #[test]
    fn git_log_none_for_non_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        assert!(git_log(dir.path(), "arquivo.md").is_none());
    }

    #[test]
    fn git_log_empty_for_uncommitted_file_in_repo() {
        let dir = tempfile::TempDir::new().unwrap();
        if init_repo_with_identity(dir.path()).is_none() {
            return;
        }
        // `git log` falha inteiro (sem HEAD) num repo sem NENHUM commit
        // ainda — precisa de ao menos um commit pra testar o caso real
        // que importa aqui: arquivo específico nunca commitado, mas o
        // repo em si já tem histórico.
        std::fs::write(dir.path().join("outro.md"), "x").unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(dir.path()).args(["add", "outro.md"]).output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(dir.path()).args(["commit", "-m", "outro", "--quiet"]).output().unwrap();

        assert_eq!(git_log(dir.path(), "nunca-commitado.md"), Some(Vec::new()));
    }

    #[test]
    fn git_log_lists_commits_most_recent_first() {
        let dir = tempfile::TempDir::new().unwrap();
        if init_repo_with_identity(dir.path()).is_none() {
            return;
        }
        let file = dir.path().join("pagina.md");
        std::fs::write(&file, "v1").unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(dir.path()).args(["add", "pagina.md"]).output().unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(dir.path()).args(["commit", "-m", "primeira versao", "--quiet"]).output().unwrap();

        std::fs::write(&file, "v2").unwrap();
        std::process::Command::new("git")
            .arg("-C").arg(dir.path()).args(["commit", "-am", "segunda versao", "--quiet"]).output().unwrap();

        let log = git_log(dir.path(), "pagina.md").unwrap();
        assert_eq!(log.len(), 2);
        assert_eq!(log[0].message, "segunda versao");
        assert_eq!(log[1].message, "primeira versao");
        assert!(!log[0].hash.is_empty());
        assert_eq!(log[0].date.len(), 10); // YYYY-MM-DD
    }
}
