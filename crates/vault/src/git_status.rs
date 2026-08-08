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
}
