//! Testes de integração do binário `anotadinho-cli` — chama o binário
//! de verdade (via `assert_cmd`) contra um vault temporário, pra
//! garantir que o parsing de argumentos + saída no stdout/stderr/exit
//! code funcionam de ponta a ponta (não só a lógica de `anotadinho-ipc`,
//! que já tem seus próprios testes).

use std::fs;

use assert_cmd::Command;
use predicates::prelude::PredicateBooleanExt;
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

// ── query (ciclo 158) ────────────────────────────────────────────────

/// Vault com specs em estados diferentes + uma página com um embed de
/// consulta declarado, pro `--from-embed`.
fn setup_query_vault() -> TempDir {
    let dir = TempDir::new().expect("cria temp dir");
    fs::create_dir_all(dir.path().join("pages/specs")).unwrap();
    fs::create_dir_all(dir.path().join("pages/produto")).unwrap();
    fs::write(
        dir.path().join("pages/specs/a.md"),
        "---\ntitle: Spec A\nstatus: backlog\npriority: alta\npeso: 3\ntags:\n- spec\n---\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("pages/specs/b.md"),
        "---\ntitle: Spec B\nstatus: done\npriority: baixa\npeso: 10\ntags:\n- spec\n- api\n---\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("pages/specs/c.md"),
        "---\ntitle: Spec C\ntags:\n- spec\n---\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("pages/produto/painel.md"),
        concat!(
            "---\ntitle: Painel\n---\n",
            "{{ type: \"query\" }}\n",
            "from: pages/specs\nwhere:\n- field: status\n  op: eq\n  value: backlog\n",
            "{{ /query }}\n",
        ),
    )
    .unwrap();
    dir
}

#[test]
fn query_filtra_por_pasta_e_condicao() {
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--from", "pages/specs", "--where", "status=backlog"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/a.md"))
        .stdout(predicates::str::contains("pages/specs/b.md").not());
}

#[test]
fn query_neq_inclui_pagina_sem_o_campo() {
    // O trabalho não classificado (spec sem `status`) é justamente o que
    // não pode sumir de "o que ainda não está pronto".
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--from", "pages/specs", "--where", "status!=done"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/a.md"))
        .stdout(predicates::str::contains("pages/specs/c.md"))
        .stdout(predicates::str::contains("pages/specs/b.md").not());
}

#[test]
fn query_operadores_de_existencia_contem_e_numerico() {
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--where", "priority?"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/c.md").not());

    cli(&dir)
        .args(["query", "--where", "title~spec b"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/b.md"));

    // Alfabeticamente "10" < "5"; numericamente 10 > 5.
    cli(&dir)
        .args(["query", "--where", "peso>5"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/b.md"))
        .stdout(predicates::str::contains("pages/specs/a.md").not());
}

#[test]
fn query_ordena_e_limita() {
    let dir = setup_query_vault();
    let out = cli(&dir)
        .args(["query", "--from", "pages/specs", "--sort", "peso", "--desc", "--limit", "1"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("pages/specs/b.md"), "esperava o maior peso primeiro: {stdout}");
}

#[test]
fn query_tag_em_and() {
    let dir = setup_query_vault();
    let out = cli(&dir)
        .args(["query", "--tag", "spec", "--tag", "api"])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert_eq!(stdout.lines().count(), 1);
    assert!(stdout.contains("pages/specs/b.md"));
}

#[test]
fn query_condicao_malformada_falha_com_mensagem_util() {
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--where", "status"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("condição inválida"));
}

#[test]
fn query_from_embed_roda_a_consulta_declarada_na_pagina() {
    // O agente executa exatamente a view que o humano configurou.
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--from-embed", "pages/produto/painel.md:0"])
        .assert()
        .success()
        .stdout(predicates::str::contains("pages/specs/a.md"))
        .stdout(predicates::str::contains("pages/specs/b.md").not());
}

#[test]
fn query_from_embed_em_tipo_errado_falha() {
    let dir = setup_embed_vault();
    cli(&dir)
        .args(["query", "--from-embed", "pages/painel.md:0"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("kanban"));
}

#[test]
fn query_json_usa_o_schema_do_indice() {
    let dir = setup_query_vault();
    let out = cli(&dir)
        .args(["--json", "query", "--from", "pages/specs", "--where", "status=backlog"])
        .output()
        .unwrap();
    let parsed: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert!(parsed.is_array());
    assert_eq!(parsed[0]["path"], "pages/specs/a.md");
    assert_eq!(parsed[0]["title"], "Spec A");
    assert_eq!(parsed[0]["properties"]["priority"], "alta");
}

#[test]
fn list_pages_e_query_concordam_no_mesmo_filtro() {
    // Paridade: `list-pages --status` passou a delegar pro mesmo motor
    // (ciclo 158) — se divergir, uma das duas está mentindo pro agente.
    let dir = setup_query_vault();
    let list = cli(&dir)
        .args(["list-pages", "--folder", "pages/specs", "--status", "backlog"])
        .output()
        .unwrap();
    let query = cli(&dir)
        .args(["query", "--from", "pages/specs", "--where", "status=backlog"])
        .output()
        .unwrap();
    let list_paths: Vec<&str> = String::from_utf8_lossy(&list.stdout)
        .lines()
        .filter_map(|l| l.split('\t').nth(2))
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &str)
        .collect();
    let query_paths: Vec<&str> = String::from_utf8_lossy(&query.stdout)
        .lines()
        .filter_map(|l| l.split('\t').next())
        .map(|s| Box::leak(s.to_string().into_boxed_str()) as &str)
        .collect();
    assert_eq!(list_paths, query_paths);
    assert_eq!(list_paths, vec!["pages/specs/a.md"]);
}

// ── ciclo 162: frontmatter sem campo nulo ────────────────────────────

#[test]
fn set_property_nao_introduz_chave_nova_no_frontmatter() {
    let dir = setup_vault();
    cli(&dir)
        .args(["set-property", "pages/specs/minha-spec.md", "status", "done"])
        .assert()
        .success();
    let conteudo = fs::read_to_string(dir.path().join("pages/specs/minha-spec.md")).unwrap();
    assert!(conteudo.contains("status: done"), "{conteudo}");
    assert!(!conteudo.contains("null"), "gravou campo nulo:\n{conteudo}");
    assert!(!conteudo.contains("created:"), "inventou campo:\n{conteudo}");
    assert!(conteudo.contains("# Minha Spec"), "o corpo se perdeu:\n{conteudo}");
}

// ── ciclo 169: agrupamento e agregados ───────────────────────────────

#[test]
fn query_agrupa_e_agrega() {
    let dir = setup_query_vault();
    let out = cli(&dir)
        .args([
            "query", "--from", "pages/specs", "--group-by", "status",
            "--aggregate", "count", "--aggregate", "sum:peso",
        ])
        .output()
        .unwrap();
    let stdout = String::from_utf8(out.stdout).unwrap();
    assert!(stdout.contains("# backlog (1)"), "{stdout}");
    assert!(stdout.contains("# done (1)"), "{stdout}");
    // A spec sem status vira o próprio grupo, no fim.
    assert!(stdout.contains("# sem status (1)"), "{stdout}");
    let pos_sem = stdout.find("# sem status").unwrap();
    let pos_done = stdout.find("# done").unwrap();
    assert!(pos_done < pos_sem, "o grupo sem campo devia vir por último:\n{stdout}");
    assert!(stdout.contains("soma peso: 3"), "agregado de soma errado:\n{stdout}");
}

#[test]
fn query_agregado_invalido_falha_com_mensagem_util() {
    let dir = setup_query_vault();
    cli(&dir)
        .args(["query", "--aggregate", "mediana:peso"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("agregado inválido"));

    cli(&dir)
        .args(["query", "--aggregate", "sum"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("precisa de um campo"));
}

// ── ciclo 176: id de bloco ───────────────────────────────────────────

#[test]
fn read_com_id_de_bloco_devolve_so_aquela_linha() {
    let dir = setup_vault();
    fs::write(
        dir.path().join("pages/blocos.md"),
        "---\ntitle: Blocos\n---\nprimeira linha\nsegunda linha ^alvo1\nterceira linha\n",
    )
    .unwrap();

    cli(&dir)
        .args(["read", "pages/blocos.md^alvo1"])
        .assert()
        .success()
        .stdout(predicates::str::contains("segunda linha"))
        .stdout(predicates::str::contains("primeira linha").not())
        .stdout(predicates::str::contains("^alvo1").not());
}

#[test]
fn read_com_id_inexistente_falha() {
    let dir = setup_vault();
    cli(&dir)
        .args(["read", "pages/alpha.md^naoexiste"])
        .assert()
        .failure()
        .stderr(predicates::str::contains("não tem o bloco"));
}

// ── ciclo 189: validação semântica de embed ──────────────────────────

/// Página com um cronograma válido, pra os testes partirem de um estado
/// bom e checarem o que acontece ao TENTAR piorá-lo.
fn vault_com_cronograma() -> tempfile::TempDir {
    let dir = setup_vault();
    fs::write(
        dir.path().join("pages/crono.md"),
        "---\ntitle: Crono\n---\n\n{{ type: \"timeline\" }}\nscale: month\nitems:\n- title: Etapa\n  start: '2026-08-03'\n  end: '2026-08-10'\n{{ /timeline }}\n",
    )
    .unwrap();
    dir
}

#[test]
fn check_nao_reclama_de_pagina_sa() {
    let dir = vault_com_cronograma();
    cli(&dir)
        .args(["embed", "check", "pages/crono.md"])
        .assert()
        .success()
        .stdout(predicates::str::contains("nenhum problema"));
}

#[test]
fn set_recusa_intervalo_invertido_e_nao_grava() {
    let dir = vault_com_cronograma();
    cli(&dir)
        .args(["embed", "set", "pages/crono.md", "0"])
        .write_stdin("scale: month\nitems:\n- title: Etapa\n  start: '2026-08-20'\n  end: '2026-08-03'\n")
        .assert()
        .failure()
        .stderr(predicates::str::contains("vem antes de start"));

    // O arquivo tem que continuar exatamente como estava.
    let disco = fs::read_to_string(dir.path().join("pages/crono.md")).unwrap();
    assert!(disco.contains("start: '2026-08-03'"), "{disco}");
    assert!(!disco.contains("2026-08-20"), "gravou mesmo recusando:\n{disco}");
}

#[test]
fn forcar_grava_mesmo_com_erro() {
    let dir = vault_com_cronograma();
    cli(&dir)
        .args(["embed", "--forcar", "set", "pages/crono.md", "0"])
        .write_stdin("scale: month\nitems:\n- title: Etapa\n  start: '2026-08-20'\n  end: '2026-08-03'\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("vem antes de start"));

    let disco = fs::read_to_string(dir.path().join("pages/crono.md")).unwrap();
    assert!(disco.contains("2026-08-20"), "--forcar devia ter gravado:\n{disco}");
}

#[test]
fn check_falha_no_que_ja_esta_no_disco() {
    let dir = setup_vault();
    fs::write(
        dir.path().join("pages/ruim.md"),
        "---\ntitle: Ruim\n---\n\n{{ type: \"actions\" }}\nbuttons:\n- label: X\n  action: rodar-shell\n{{ /actions }}\n",
    )
    .unwrap();
    cli(&dir)
        .args(["embed", "check", "pages/ruim.md"])
        .assert()
        .failure()
        .stdout(predicates::str::contains("rodar-shell"));
}

/// Guarda de regressão da checagem que o próprio `add-row` já fazia
/// antes do ciclo 189 — não é a validação nova que barra aqui. A regra
/// nova de "número de células diferente do de colunas" só é alcançável
/// por construção direta (o pulldown-cmark completa linha curta ao
/// parsear), e está coberta em `core::embed`.
#[test]
fn add_row_com_numero_errado_de_valores_nao_grava() {
    let dir = setup_vault();
    fs::write(
        dir.path().join("pages/tab.md"),
        "---\ntitle: Tab\n---\n\n{{ type: \"table\" }}\ncolumns:\n- name: A\n- name: B\n---\n| A | B |\n| - | - |\n| 1 | 2 |\n{{ /table }}\n",
    )
    .unwrap();
    let antes = fs::read_to_string(dir.path().join("pages/tab.md")).unwrap();

    cli(&dir)
        .args(["embed", "add-row", "pages/tab.md", "0", "--values", "so-uma"])
        .assert()
        .failure();

    assert_eq!(
        fs::read_to_string(dir.path().join("pages/tab.md")).unwrap(),
        antes,
        "arquivo mudou apesar da recusa"
    );
}

#[test]
fn aviso_nao_bloqueia_a_gravacao() {
    let dir = setup_vault();
    fs::write(
        dir.path().join("pages/destaque.md"),
        "---\ntitle: D\n---\n\n{{ type: \"callout\" }}\nvariant: info\ntitle: Tem titulo\nbody: |\n  corpo\n{{ /callout }}\n",
    )
    .unwrap();
    // Callout sem título e sem corpo é AVISO, não erro.
    cli(&dir)
        .args(["embed", "set", "pages/destaque.md", "0"])
        .write_stdin("variant: info\ntitle: ''\nbody: ''\n")
        .assert()
        .success()
        .stderr(predicates::str::contains("aviso:"));
}

// ── ciclo 205: servidor MCP ──────────────────────────────────────────

/// Manda linhas JSON-RPC no stdin do servidor e devolve as respostas.
fn mcp(dir: &tempfile::TempDir, pedidos: &[serde_json::Value]) -> Vec<serde_json::Value> {
    let entrada = pedidos
        .iter()
        .map(|p| p.to_string())
        .collect::<Vec<_>>()
        .join("\n");
    let saida = cli(dir).arg("mcp").write_stdin(entrada).output().unwrap();
    String::from_utf8_lossy(&saida.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| serde_json::from_str(l).expect("resposta não é JSON"))
        .collect()
}

#[test]
fn mcp_responde_o_handshake() {
    let dir = setup_vault();
    let r = mcp(&dir, &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"initialize"})]);
    assert_eq!(r.len(), 1);
    assert_eq!(r[0]["result"]["serverInfo"]["name"], "anotadinho");
    assert!(r[0]["result"]["capabilities"]["tools"].is_object());
}

#[test]
fn mcp_lista_as_ferramentas_e_a_unica_escrita_e_propor() {
    // A garantia do ciclo: um agente conectado aqui NÃO consegue gravar
    // página. Se alguém acrescentar uma ferramenta de escrita direta,
    // este teste reprova.
    let dir = setup_vault();
    let r = mcp(&dir, &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/list"})]);
    let nomes: Vec<String> = r[0]["result"]["tools"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["name"].as_str().unwrap().to_string())
        .collect();
    assert!(nomes.contains(&"propor".to_string()));
    for proibida in ["escrever", "escrever_pagina", "write_page", "apagar", "deletar"] {
        assert!(
            !nomes.contains(&proibida.to_string()),
            "ferramenta de escrita direta exposta: {proibida}"
        );
    }
}

#[test]
fn mcp_le_uma_pagina() {
    let dir = setup_vault();
    let r = mcp(
        &dir,
        &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"ler_pagina","arguments":{"path":"pages/alpha.md"}}})],
    );
    let texto = r[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("---"), "{texto}");
}

#[test]
fn mcp_propor_nao_escreve_a_pagina() {
    let dir = setup_vault();
    let r = mcp(
        &dir,
        &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"propor","arguments":{
                "path":"pages/vinda-do-mcp.md",
                "conteudo":"---\ntitle: X\n---\ncorpo\n",
                "motivo":"teste"}}})],
    );
    let texto = r[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("NÃO foi escrita"), "{texto}");
    assert!(
        !dir.path().join("pages/vinda-do-mcp.md").exists(),
        "o MCP escreveu a página sem revisão"
    );
    assert!(dir.path().join(".anotadinho/propostas").exists());
}

#[test]
fn mcp_propor_fora_do_vault_e_recusado() {
    let dir = setup_vault();
    let r = mcp(
        &dir,
        &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"propor","arguments":{"path":"../fora.md","conteudo":"x"}}})],
    );
    assert_eq!(r[0]["result"]["isError"], true);
    let texto = r[0]["result"]["content"][0]["text"].as_str().unwrap();
    assert!(texto.contains("fora do vault"), "{texto}");
}

#[test]
fn mcp_json_quebrado_nao_derruba_o_servidor() {
    // Um agente com bug de serialização não pode matar a sessão: o
    // servidor responde o erro e segue atendendo.
    let dir = setup_vault();
    let entrada = "{ isto nao e json\n{\"jsonrpc\":\"2.0\",\"id\":2,\"method\":\"ping\"}";
    let saida = cli(&dir).arg("mcp").write_stdin(entrada).output().unwrap();
    let linhas: Vec<&str> = String::from_utf8_lossy(&saida.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| Box::leak(l.to_string().into_boxed_str()) as &str)
        .collect();
    assert_eq!(linhas.len(), 2, "esperava erro + resposta do ping");
    assert!(linhas[0].contains("JSON inválido"), "{}", linhas[0]);
    assert!(linhas[1].contains("\"id\":2"), "{}", linhas[1]);
}

#[test]
fn mcp_notificacao_sem_id_nao_gera_resposta() {
    // Manda notificação, uma requisição normal.
    let dir = setup_vault();
    let entrada = "{\"jsonrpc\":\"2.0\",\"method\":\"notifications/initialized\"}\n\
                   {\"jsonrpc\":\"2.0\",\"id\":7,\"method\":\"ping\"}";
    let saida = cli(&dir).arg("mcp").write_stdin(entrada).output().unwrap();
    let n = String::from_utf8_lossy(&saida.stdout)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .count();
    assert_eq!(n, 1, "notificação não pode receber resposta");
}

#[test]
fn mcp_ferramenta_desconhecida_devolve_erro_legivel() {
    let dir = setup_vault();
    let r = mcp(
        &dir,
        &[serde_json::json!({"jsonrpc":"2.0","id":1,"method":"tools/call",
            "params":{"name":"inventada","arguments":{}}})],
    );
    assert_eq!(r[0]["result"]["isError"], true);
}
