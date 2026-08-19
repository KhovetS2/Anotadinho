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
    fs::create_dir_all(dir.path().join("pages/specs")).unwrap();
    fs::create_dir_all(dir.path().join("templates")).unwrap();
    fs::write(
        dir.path().join("pages/alpha.md"),
        "---\ntitle: Alpha\n---\nConteúdo da página alpha.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("pages/specs/minha-spec.md"),
        "---\ntitle: Minha Spec\nstatus: backlog\npriority: alta\ntags:\n- spec\n---\n# Minha Spec\n",
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
fn list_pages_filters_by_folder() {
    let dir = setup_vault();
    let output = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "list-pages", "--folder", "pages/specs"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("minha-spec"));
    assert!(!stdout.contains("alpha"));
}

#[test]
fn list_pages_filters_by_tag() {
    let dir = setup_vault();
    let output = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "list-pages", "--tag", "spec"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("minha-spec"));
    assert!(!stdout.contains("alpha.md"));
}

#[test]
fn list_pages_filters_by_status_and_priority_combined() {
    let dir = setup_vault();
    let output = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args([
            "--vault", dir.path().to_str().unwrap(), "list-pages",
            "--status", "backlog", "--priority", "alta",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("minha-spec"));

    let output_no_match = Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args(["--vault", dir.path().to_str().unwrap(), "list-pages", "--status", "done"])
        .output()
        .unwrap();
    assert!(String::from_utf8(output_no_match.stdout).unwrap().trim().is_empty());
}

#[test]
fn set_property_updates_existing_field_preserves_body() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args([
            "--vault", dir.path().to_str().unwrap(), "set-property",
            "pages/specs/minha-spec.md", "status", "in-progress",
        ])
        .assert()
        .success();
    let content = fs::read_to_string(dir.path().join("pages/specs/minha-spec.md")).unwrap();
    assert!(content.contains("status: in-progress"));
    assert!(content.contains("# Minha Spec"));
    assert!(!content.contains("status: backlog"));
}

#[test]
fn set_property_adds_new_key_without_touching_others() {
    let dir = setup_vault();
    Command::cargo_bin("anotadinho-cli")
        .unwrap()
        .args([
            "--vault", dir.path().to_str().unwrap(), "set-property",
            "pages/specs/minha-spec.md", "owner", "elis",
        ])
        .assert()
        .success();
    let content = fs::read_to_string(dir.path().join("pages/specs/minha-spec.md")).unwrap();
    assert!(content.contains("owner: elis"));
    assert!(content.contains("status: backlog"));
    assert!(content.contains("priority: alta"));
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

// ── embed (ciclo 157) ────────────────────────────────────────────────

/// Vault com uma página que tem markdown ao redor de DOIS embeds — é o
/// entorno que precisa sobreviver a toda escrita.
fn setup_embed_vault() -> TempDir {
    let dir = TempDir::new().expect("cria temp dir");
    fs::create_dir_all(dir.path().join("pages")).unwrap();
    fs::write(
        dir.path().join("pages/painel.md"),
        concat!(
            "---\ntitle: Painel\n---\n",
            "# Antes\n\n",
            "Texto que não pode sumir.\n\n",
            "{{ type: \"kanban\" }}\n",
            "columns:\n- Backlog\n- Done\n",
            "items:\n- title: Card 1\n  column: Backlog\n",
            "{{ /kanban }}\n\n",
            "Texto do meio.\n\n",
            "{{ type: \"calendar\" }}\n",
            "entries:\n- date: '2026-08-01'\n  title: Evento\n",
            "{{ /calendar }}\n\n",
            "Texto do fim.\n",
        ),
    )
    .unwrap();
    dir
}

fn cli(dir: &TempDir) -> Command {
    let mut cmd = Command::cargo_bin("anotadinho-cli").unwrap();
    cmd.args(["--vault", dir.path().to_str().unwrap()]);
    cmd
}

#[test]
fn embed_list_mostra_indice_tipo_e_resumo() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args(["embed", "list", "pages/painel.md"])
        .assert()
        .success()
        .stdout(predicates::str::contains("0\tkanban"))
        .stdout(predicates::str::contains("1\tcalendar"))
        .stdout(predicates::str::contains("1 card(s)"));
}

#[test]
fn embed_get_devolve_o_conteudo_do_embed() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args(["embed", "get", "pages/painel.md", "1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("title: Evento"));
}

#[test]
fn embed_get_seguido_de_set_e_idempotente() {
    // Regressão de formatação: o mesmo tipo de bug dos ciclos 076/078/111
    // — se `set` normalizasse diferente do que `get` devolve, toda
    // escrita de agente sujaria o `git diff` de graça (ou pior, cresceria
    // o arquivo com uma linha em branco a cada rodada).
    //
    // A primeira escrita normaliza o arquivo (aspas de YAML, espaçamento
    // do wrapper) — isso é esperado, qualquer escrita pelo app faz o
    // mesmo. O que este teste garante é que da segunda em diante NADA
    // muda.
    let dir = setup_embed_vault();
    let page = dir.path().join("pages/painel.md");

    let roundtrip = |dir: &TempDir| {
        let out = cli(dir)
            .args(["embed", "get", "pages/painel.md", "0"])
            .output()
            .unwrap();
        let tmp = dir.path().join("body.yaml");
        fs::write(&tmp, out.stdout).unwrap();
        cli(dir)
            .args([
                "embed", "set", "pages/painel.md", "0", "--file",
                tmp.to_str().unwrap(),
            ])
            .assert()
            .success();
    };

    roundtrip(&dir);
    let normalized = fs::read_to_string(&page).unwrap();
    roundtrip(&dir);
    assert_eq!(fs::read_to_string(&page).unwrap(), normalized);
    // E o entorno continua lá depois das duas rodadas.
    assert!(normalized.contains("Texto que não pode sumir."));
    assert!(normalized.contains("Texto do fim."));
}

#[test]
fn embed_add_card_preserva_o_markdown_ao_redor() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args([
            "embed", "add-card", "pages/painel.md", "0", "--column", "Done",
            "--title", "Card do agente",
        ])
        .assert()
        .success();

    let content = fs::read_to_string(dir.path().join("pages/painel.md")).unwrap();
    assert!(content.contains("Card do agente"));
    assert!(content.contains("title: Painel"), "frontmatter sumiu");
    assert!(content.contains("Texto que não pode sumir."));
    assert!(content.contains("Texto do meio."));
    assert!(content.contains("Texto do fim."));
    assert!(content.contains("title: Evento"), "o outro embed foi afetado");
}

#[test]
fn embed_add_event_no_calendario() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args([
            "embed", "add-event", "pages/painel.md", "1", "--date", "2026-09-09",
            "--title", "Reunião",
        ])
        .assert()
        .success();
    let content = fs::read_to_string(dir.path().join("pages/painel.md")).unwrap();
    assert!(content.contains("2026-09-09"));
    assert!(content.contains("Reunião"));
}

#[test]
fn embed_add_card_em_tipo_errado_falha_sem_tocar_no_arquivo() {
    let dir = setup_embed_vault();
    let page = dir.path().join("pages/painel.md");
    let before = fs::read_to_string(&page).unwrap();

    cli(&dir)
        .args([
            "embed", "add-card", "pages/painel.md", "1", "--column", "Done",
            "--title", "X",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("kanban"));

    assert_eq!(fs::read_to_string(&page).unwrap(), before);
}

#[test]
fn embed_add_card_em_coluna_inexistente_falha() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args([
            "embed", "add-card", "pages/painel.md", "0", "--column", "Nope",
            "--title", "X",
        ])
        .assert()
        .failure()
        .stderr(predicates::str::contains("não existe"));
}

#[test]
fn embed_indice_fora_do_intervalo_falha_com_mensagem_clara() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args(["embed", "get", "pages/painel.md", "9"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("2 embed(s)"));
}

#[test]
fn embed_add_row_valida_a_quantidade_de_colunas() {
    let dir = TempDir::new().unwrap();
    fs::create_dir_all(dir.path().join("pages")).unwrap();
    fs::write(
        dir.path().join("pages/tab.md"),
        concat!(
            "---\ntitle: Tab\n---\n",
            "{{ type: \"table\" }}\n",
            "| Tarefa | Status |\n| --- | --- |\n| API | done |\n",
            "{{ /table }}\n",
        ),
    )
    .unwrap();

    cli(&dir)
        .args(["embed", "add-row", "pages/tab.md", "0", "--values", "só um"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("2 coluna(s)"));

    cli(&dir)
        .args(["embed", "add-row", "pages/tab.md", "0", "--values", "UI, doing"])
        .assert()
        .success();
    let content = fs::read_to_string(dir.path().join("pages/tab.md")).unwrap();
    assert!(content.contains("| UI | doing |"));
}
