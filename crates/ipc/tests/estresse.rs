//! Bateria de estresse do backend: mede, não afirma.
//!
//! O que estes testes cobrem não é comportamento — é ESCALA. O vault
//! de quem escreveu o Anotadinho tem ~250 páginas, e nesse tamanho
//! quase tudo parece rápido. Um `O(n²)` num caminho quente custa 60ms
//! aqui e 25 segundos num vault de 4 mil páginas, e não há como
//! perceber a diferença sem construir o vault grande.
//!
//! Por isso todos são `#[ignore]`: eles montam vaults sintéticos de
//! milhares de páginas e levam segundos. Não têm lugar na suíte que
//! roda a cada commit. Rodam sob demanda:
//!
//! ```bash
//! cargo test -p anotadinho-ipc --test estresse -- --ignored --nocapture
//! ```
//!
//! Cada um imprime o tempo medido E afirma um teto generoso. O teto não
//! é uma meta de desempenho: é um alarme de mudança de ORDEM. Está posto
//! com folga larga o bastante pra não disparar por máquina lenta ou
//! máquina ocupada, e apertado o bastante pra `O(n)` virar `O(n²)`
//! acender a luz.

use std::path::Path;
use std::time::{Duration, Instant};

/// Monta um vault sintético com `n` páginas em `pages/`.
///
/// O conteúdo imita o vault real de propósito: frontmatter com tags e
/// propriedades, wikilinks entre páginas e um embed. Um vault de
/// páginas vazias mediria o `WalkDir` e mais nada — e o custo está no
/// que se faz com o conteúdo depois de lê-lo.
fn vault_sintetico(raiz: &Path, n: usize) {
    let pages = raiz.join("pages");
    std::fs::create_dir_all(&pages).unwrap();
    // Subpastas, pra a hierarquia da sidebar ter o que montar.
    for pasta in ["specs", "ciclos", "notas"] {
        std::fs::create_dir_all(pages.join(pasta)).unwrap();
    }
    for i in 0..n {
        let pasta = ["specs", "ciclos", "notas"][i % 3];
        let tipo = ["spec", "ciclo", "md"][i % 3];
        let prioridade = ["alta", "media", "baixa"][i % 3];
        let corpo = format!(
            "---\n\
             title: Página {i}\n\
             type: {tipo}\n\
             prioridade: {prioridade}\n\
             status: {}\n\
             tags:\n- {tipo}\n- lote\n\
             ---\n\
             # Página {i}\n\n\
             Texto de corpo com [[Página {}]] e [[Página {}]].\n\n\
             campo:: valor{}\n\n\
             {{{{ type: \"callout\" }}}}\n\
             variant: info\n\
             title: Nota {i}\n\
             body: |\n  corpo do callout\n\
             {{{{ /callout }}}}\n",
            ["backlog", "in-progress", "done"][i % 3],
            (i + 1) % n.max(1),
            (i + 7) % n.max(1),
            i % 10,
        );
        std::fs::write(pages.join(pasta).join(format!("pagina-{i}.md")), corpo).unwrap();
    }
}

/// Roda `f` e devolve quanto levou.
fn cronometrar<T>(f: impl FnOnce() -> T) -> (T, Duration) {
    let t0 = Instant::now();
    let r = f();
    (r, t0.elapsed())
}

/// Falha com uma mensagem que diz o número medido, não só que passou do
/// teto — o número é o dado, o teto é só o gatilho.
#[track_caller]
fn dentro_do_teto(rotulo: &str, medido: Duration, teto: Duration) {
    println!("  {rotulo}: {medido:?} (teto {teto:?})");
    assert!(
        medido <= teto,
        "{rotulo} levou {medido:?}, acima do teto de {teto:?} — \
         provavelmente a complexidade mudou de ordem"
    );
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn varredura_de_vault_grande_e_linear() {
    // Custo POR PÁGINA da varredura quente, por tamanho de vault. É a
    // asserção que importa neste arquivo: um teto absoluto passa a
    // valer pela máquina, mas o custo por página crescendo com o
    // tamanho do vault só acontece por uma razão — alguma coisa no
    // caminho é quadrática.
    //
    // Foi assim que dois `O(n²)` apareceram (ciclo 259):
    // `IndexCache::manter_apenas` confrontava cada entrada do cache com
    // a lista de páginas por busca linear, e `Query::run_grouped`
    // mantinha um `Vec` de grupos consultado com `contains` a cada
    // item. Antes deles, o custo por página TRIPLICAVA de 500 pra 4000.
    let mut por_pagina: Vec<(usize, f64)> = Vec::new();

    for n in [500usize, 2000, 4000] {
        let dir = tempfile::TempDir::new().unwrap();
        vault_sintetico(dir.path(), n);
        let caminho = dir.path().to_string_lossy().to_string();

        // Primeira varredura: cache frio, tudo é lido e parseado.
        let (fria, t_fria) =
            cronometrar(|| anotadinho_ipc::handle_scan_vault(caminho.clone()).unwrap());
        assert_eq!(fria.len(), n);

        // Segunda: cache quente, nada mudou. É esta que roda o tempo
        // todo — cada embed de consulta da página chama por conta
        // própria.
        let (quente, t_quente) =
            cronometrar(|| anotadinho_ipc::handle_scan_vault(caminho.clone()).unwrap());
        assert_eq!(quente.len(), n);

        println!("n={n}: fria {t_fria:?}, quente {t_quente:?}");
        // Tetos por página, não absolutos: é o que detecta mudança de
        // ordem sem depender da velocidade da máquina.
        dentro_do_teto(
            &format!("varredura fria de {n}"),
            t_fria,
            Duration::from_micros(2000) * n as u32,
        );
        dentro_do_teto(
            &format!("varredura quente de {n}"),
            t_quente,
            Duration::from_micros(1000) * n as u32,
        );
        por_pagina.push((n, t_quente.as_secs_f64() * 1e6 / n as f64));
    }

    for (n, us) in &por_pagina {
        println!("  custo por página com n={n}: {us:.1}µs");
    }
    let (menor, maior) = (por_pagina[0].1, por_pagina[por_pagina.len() - 1].1);
    // 1,6× de folga, e o número foi CALIBRADO, não escolhido: com as
    // correções o custo por página vai de 19,6µs (n=500) a 21,6µs
    // (n=4000), 1,10×; revertendo só o `manter_apenas` ele vai de 22,5 a
    // 48,8µs, 2,17×. O primeiro limite que escrevi aqui foi 2,5× e não
    // pegou a regressão que eu já sabia estar lá — vale desconfiar de
    // qualquer teto que nunca foi visto reprovando.
    //
    // Algum crescimento é honesto (cache maior, mais pressão de
    // alocação); o que não é honesto é ele acompanhar o tamanho do
    // vault.
    assert!(
        maior <= menor * 1.6,
        "o custo por página subiu de {menor:.1}µs (n={}) pra {maior:.1}µs (n={}) —          alguma coisa na varredura voltou a ser quadrática",
        por_pagina[0].0,
        por_pagina[por_pagina.len() - 1].0,
    );
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn consulta_sobre_indice_grande() {
    use anotadinho_core::query::{Condition, Query, QueryOp, Sort};

    let dir = tempfile::TempDir::new().unwrap();
    vault_sintetico(dir.path(), 4000);
    let entradas =
        anotadinho_ipc::handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap();

    let filtrada = Query {
        conditions: vec![Condition {
            field: "type".into(),
            op: QueryOp::Eq,
            value: "spec".into(),
        }],
        sort: Some(Sort { field: "title".into(), desc: false }),
        ..Default::default()
    };
    let (r, t) = cronometrar(|| filtrada.run(&entradas));
    println!("consulta filtrada+ordenada sobre 4000: {} resultados", r.len());
    dentro_do_teto("consulta filtrada", t, Duration::from_millis(400));

    // Agrupar é o caminho mais pesado: filtra, ordena e ainda distribui
    // em grupos.
    let agrupada = Query {
        group_by: Some("prioridade".into()),
        ..Default::default()
    };
    let (grupos, t) = cronometrar(|| agrupada.run_grouped(&entradas));
    println!("consulta agrupada sobre 4000: {} grupos", grupos.len());
    dentro_do_teto("consulta agrupada", t, Duration::from_millis(400));

    // Muitos grupos DISTINTOS é o caso que separa um agrupamento linear
    // de um quadrático: agrupar por `title` faz um grupo por página.
    let espalhada = Query {
        group_by: Some("title".into()),
        ..Default::default()
    };
    let (grupos, t) = cronometrar(|| espalhada.run_grouped(&entradas));
    assert_eq!(grupos.len(), 4000, "esperava um grupo por página");
    dentro_do_teto("agrupar em 4000 grupos distintos", t, Duration::from_millis(600));
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn pagina_gigante_nao_trava_a_indexacao() {
    // Uma página só, enorme. O editor por bloco divide por bloco, mas
    // quem indexa lê o arquivo inteiro de uma vez.
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("pages")).unwrap();
    let mut corpo = String::from("---\ntitle: Gigante\ntags:\n- lote\n---\n");
    for i in 0..20_000 {
        corpo.push_str(&format!("Parágrafo {i} com um [[link {}]].\n\n", i % 50));
    }
    println!("página de {} KB", corpo.len() / 1024);
    std::fs::write(dir.path().join("pages/gigante.md"), &corpo).unwrap();

    let (entradas, t) = cronometrar(|| {
        anotadinho_ipc::handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap()
    });
    assert_eq!(entradas.len(), 1);
    assert_eq!(entradas[0].wikilinks.len(), 50, "wikilinks únicos");
    dentro_do_teto("indexar página de 20 mil parágrafos", t, Duration::from_secs(3));
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn muitos_embeds_numa_pagina_so() {
    // O `segment` do embed roda sobre o corpo inteiro a cada leitura, e
    // a página de tags o chama pra CADA página do vault.
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("pages")).unwrap();
    let mut corpo = String::from("---\ntitle: Muitos embeds\n---\n");
    for i in 0..500 {
        corpo.push_str(&format!(
            "{{{{ type: \"kanban\" }}}}\ncolumns:\n- A\nitems:\n- title: Card {i}\n  column: A\n  tags:\n  - t{}\n{{{{ /kanban }}}}\n\n",
            i % 20
        ));
    }
    std::fs::write(dir.path().join("pages/embeds.md"), &corpo).unwrap();

    let (entradas, t) = cronometrar(|| {
        anotadinho_ipc::handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap()
    });
    assert_eq!(entradas[0].embed_tags.len(), 20);
    dentro_do_teto("indexar 500 embeds numa página", t, Duration::from_secs(3));
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn apagar_muitas_paginas_nao_explode_o_cache() {
    // O caso que expõe o `manter_apenas`: cache cheio, vault menor.
    // Cada página cacheada é confrontada com a lista de páginas atuais,
    // e se essa comparação for linear o custo é n×m.
    let dir = tempfile::TempDir::new().unwrap();
    vault_sintetico(dir.path(), 4000);
    let caminho = dir.path().to_string_lossy().to_string();
    anotadinho_ipc::handle_scan_vault(caminho.clone()).unwrap();

    // Apaga metade: o cache fica com 4000 entradas e o vault com 2000.
    for i in 0..4000 {
        if i % 2 == 0 {
            let pasta = ["specs", "ciclos", "notas"][i % 3];
            let _ = std::fs::remove_file(
                dir.path().join("pages").join(pasta).join(format!("pagina-{i}.md")),
            );
        }
    }

    let (entradas, t) = cronometrar(|| anotadinho_ipc::handle_scan_vault(caminho.clone()).unwrap());
    assert_eq!(entradas.len(), 2000);
    dentro_do_teto("varredura com 2000 entradas obsoletas no cache", t, Duration::from_secs(4));
}

#[test]
#[ignore = "monta um vault de milhares de páginas; roda com --ignored"]
fn pagina_indice_com_milhares_de_wikilinks() {
    // Uma página que aponta pra TODAS as outras — um índice feito à mão,
    // ou a saída de uma consulta colada. Cada alvo é distinto, que é o
    // pior caso pra qualquer deduplicação por varredura linear.
    let dir = tempfile::TempDir::new().unwrap();
    std::fs::create_dir_all(dir.path().join("pages")).unwrap();
    let mut corpo = String::from("---\ntitle: Índice\n---\n");
    for i in 0..5000 {
        corpo.push_str(&format!("- [[Página {i}]]\n"));
    }
    std::fs::write(dir.path().join("pages/indice.md"), &corpo).unwrap();

    let (entradas, t) = cronometrar(|| {
        anotadinho_ipc::handle_scan_vault(dir.path().to_string_lossy().to_string()).unwrap()
    });
    assert_eq!(entradas[0].wikilinks.len(), 5000);
    dentro_do_teto("indexar página com 5 mil wikilinks distintos", t, Duration::from_secs(2));
}

/// Um projeto sintético com `n` tarefas (ciclo 462): 9 em 10 `feita`, e
/// 1 em 10 em `executar`, dependendo da anterior. Uma tarefa em
/// `executando` ocupa a única vaga, então a fila é calculada inteira e
/// nada é reservado — a medida pode se repetir sem mudar o vault.
fn projeto_sintetico(raiz: &Path, n: usize) -> String {
    let vault = raiz.to_string_lossy().to_string();
    anotadinho_ipc::handle_criar_projeto(vault.clone(), "Escala".into(), Some("escala".into()), "2026-10-21".into())
        .unwrap();
    let repo = raiz.join("repo");
    std::fs::create_dir_all(&repo).unwrap();
    let mut cfg = anotadinho_core::projeto::ConfigProjetos::new();
    cfg.insert(
        "escala".into(),
        anotadinho_core::projeto::ConfigProjeto { repositorio: Some(repo.to_string_lossy().to_string()), ..Default::default() },
    );
    anotadinho_ipc::handle_gravar_projetos_cfg(vault.clone(), cfg).unwrap();
    let pasta = raiz.join(anotadinho_core::projeto::pasta_das_tarefas("escala"));
    for i in 1..=n {
        let (etapa, depende) = match i {
            1 => ("executando", String::new()),
            _ if i % 10 == 0 => ("executar", format!("depende:\n- {}\n", i - 1)),
            _ => ("feita", String::new()),
        };
        // Numa linha só por `concat!`/`format!` com `\n` explícito: a
        // continuação com `\` em literal de YAML já vazou indentação pro
        // frontmatter três vezes (ciclos 283, 284, 296).
        let md = format!(
            "---\ntitle: Tarefa {i}\ntype: tarefa\nprojeto: escala\nstatus: {etapa}\n{depende}---\n# Tarefa {i}\n\n## Objetivo\n\nalgo\n"
        );
        std::fs::write(pasta.join(format!("{i:04}-t.md")), md).unwrap();
    }
    vault
}

#[test]
#[ignore = "monta um projeto de milhares de tarefas; roda com --ignored"]
fn fila_de_projeto_grande_e_linear() {
    // A fila é conferida a cada 5 s pela janela e pela TUI, e a tela do
    // projeto relê o estado a cada 3 s: um custo por tarefa que cresce
    // com o projeto vira travamento justamente no projeto que deu certo.
    let mut fila_por_tarefa: Vec<(usize, f64)> = Vec::new();
    let mut tela_por_tarefa: Vec<(usize, f64)> = Vec::new();
    for n in [250usize, 1000, 2000] {
        let dir = tempfile::TempDir::new().unwrap();
        let vault = projeto_sintetico(dir.path(), n);
        let params = anotadinho_ipc::ParametrosDaFila {
            agora: "2026-10-21 10:00".into(),
            teto: 0,
            limite_historico: 12,
            vagas: None,
        };
        // Aquece o cache de disco do sistema: a primeira leitura mede o
        // disco, não o código.
        anotadinho_ipc::handle_tarefas_a_executar(vault.clone(), params.clone()).unwrap();
        let (r, t_fila) = cronometrar(|| anotadinho_ipc::handle_tarefas_a_executar(vault.clone(), params.clone()).unwrap());
        assert!(r.iniciar.is_empty(), "a vaga está ocupada: nada podia ser reservado");
        let (e, t_tela) = cronometrar(|| anotadinho_ipc::handle_estado_do_projeto(vault.clone(), "escala".into()).unwrap());
        assert_eq!(e.tarefas.len(), n);
        println!("n={n}: fila {t_fila:?}, tela {t_tela:?}");
        dentro_do_teto(&format!("fila com {n} tarefas"), t_fila, Duration::from_micros(1000) * n as u32);
        dentro_do_teto(&format!("estado do projeto com {n} tarefas"), t_tela, Duration::from_micros(1000) * n as u32);
        fila_por_tarefa.push((n, t_fila.as_secs_f64() * 1e6 / n as f64));
        tela_por_tarefa.push((n, t_tela.as_secs_f64() * 1e6 / n as f64));
    }
    for (rotulo, serie) in [("fila", &fila_por_tarefa), ("tela", &tela_por_tarefa)] {
        for (n, us) in serie.iter() {
            println!("  {rotulo}: custo por tarefa com n={n}: {us:.1}µs");
        }
        let (menor, maior) = (serie[0].1, serie[serie.len() - 1].1);
        assert!(
            maior <= menor * 1.6,
            "{rotulo}: o custo por tarefa subiu de {menor:.1}µs (n={}) pra {maior:.1}µs (n={}) — alguma coisa ficou quadrática",
            serie[0].0,
            serie[serie.len() - 1].0,
        );
    }
}
