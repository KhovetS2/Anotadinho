//! Testes de integração do binário `anotadinho-cli` — chama o binário
//! de verdade (via `assert_cmd`) contra um vault temporário, pra
//! garantir que o parsing de argumentos + saída no stdout/stderr/exit
//! code funcionam de ponta a ponta (não só a lógica de `anotadinho-ipc`,
//! que já tem seus próprios testes).

use std::fs;

use assert_cmd::Command;
use tempfile::TempDir;

fn setup_vault() -> TempDir {
    let dir = TempDir::new().expect("cria temp dir");
    fs::create_dir_all(dir.path().join("pages")).unwrap();
    fs::create_dir_all(dir.path().join("templates")).unwrap();
    fs::write(
        dir.path().join("pages/alpha.md"),
        "---\ntitle: Alpha\n---\nConteúdo da página alpha.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("templates/spec.md"),
        "---\ntitle: {{title}}\nstatus: draft\n---\n# {{title}}\n",
    )
    .unwrap();
    dir
}

#[test]
fn list_pages_prints_one_line_per_page() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "list-pages"])
        .assert()
        .success()
        .stdout(predicates::str::contains("alpha"));
}

#[test]
fn list_pages_json_emits_valid_json() {
    let dir = setup_vault();
    let output = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "--json", "list-pages"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed[0]["title"], "alpha");
}

#[test]
fn read_prints_raw_content() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "read", "pages/alpha.md"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Conteúdo da página alpha."));
}

#[test]
fn read_missing_page_fails_with_nonzero_exit() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "read", "pages/nao-existe.md"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("erro:"));
}

#[test]
fn search_finds_page_by_content() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "search", "alpha"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/alpha.md"));
}

#[test]
fn export_concatenates_pages() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "export"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Conteúdo da página alpha."));
}

#[test]
fn list_templates_finds_template() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "list-templates"])
        .assert()
        .success()
        .stdout(predicates::str::contains("templates/spec.md"));
}

#[test]
fn new_from_template_creates_page_and_prints_path() {
    let dir = setup_vault();
    let output = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args([
            "--vault",
            dir.path().to_str().unwrap(),
            "new-from-template",
            "templates/spec.md",
            "Minha Spec",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let printed_path = String::from_utf8(output.stdout).unwrap().trim().to_string();
    assert_eq!(printed_path, "pages/minha-spec.md");
    let content = fs::read_to_string(dir.path().join(&printed_path)).unwrap();
    assert!(content.contains("title: Minha Spec"));
    assert!(content.contains("# Minha Spec"));
}
