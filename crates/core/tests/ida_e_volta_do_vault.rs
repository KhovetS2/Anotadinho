//! A ida e volta contra o vault DE VERDADE.
//!
//! Os testes unitários de `analise` usam corpos que eu escrevi, e um
//! fixture escrito por quem implementa tende a conter só o que ele
//! lembrou de tratar. O vault tem centenas de páginas escritas ao longo
//! de meses, com tudo que apareceu de verdade — é a única amostra que
//! não foi filtrada pelo meu conhecimento do parser.
//!
//! Roda com `--ignored` porque depende do vault estar ali:
//!
//! ```bash
//! cargo test -p anotadinho-core --test ida_e_volta_do_vault -- --ignored --nocapture
//! ```

use anotadinho_core::analise::{analisar, escrever};
use anotadinho_core::MarkdownCodec;

fn paginas() -> Vec<(String, String)> {
    let raiz = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../VaultAnotadinho");
    let mut fora = Vec::new();
    for pasta in ["pages", "journals"] {
        let dir = raiz.join(pasta);
        if !dir.is_dir() {
            continue;
        }
        for e in walkdir::WalkDir::new(&dir).into_iter().filter_map(|e| e.ok()) {
            if e.path().extension().is_some_and(|x| x == "md") {
                if let Ok(texto) = std::fs::read_to_string(e.path()) {
                    fora.push((e.path().display().to_string(), texto));
                }
            }
        }
    }
    fora
}

#[test]
#[ignore = "depende do vault; roda com --ignored"]
fn a_arvore_sobrevive_a_ida_e_volta_em_todo_o_vault() {
    let todas = paginas();
    assert!(!todas.is_empty(), "não achei o vault");

    let mut instaveis = Vec::new();
    for (caminho, texto) in &todas {
        let (_, corpo) = MarkdownCodec::split_frontmatter_text(texto);
        let uma = analisar(corpo);
        let duas = analisar(&escrever(&uma));
        if uma != duas {
            instaveis.push(caminho.clone());
        }
    }

    println!("{} páginas conferidas", todas.len());
    assert!(
        instaveis.is_empty(),
        "{} de {} páginas mudam de árvore na ida e volta:\n  {}",
        instaveis.len(),
        todas.len(),
        instaveis
            .iter()
            .take(15)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}

#[test]
#[ignore = "depende do vault; roda com --ignored"]
fn a_segunda_volta_e_identica_em_texto() {
    // Mais forte que a árvore: depois de UMA normalização, escrever de
    // novo não pode mudar mais nada. Sem isso, salvar a mesma página
    // duas vezes daria diff em git pra sempre.
    let todas = paginas();
    let mut instaveis = Vec::new();
    for (caminho, texto) in &todas {
        let (_, corpo) = MarkdownCodec::split_frontmatter_text(texto);
        let uma = escrever(&analisar(corpo));
        let duas = escrever(&analisar(&uma));
        if uma != duas {
            instaveis.push(caminho.clone());
        }
    }
    println!("{} páginas conferidas", todas.len());
    assert!(
        instaveis.is_empty(),
        "{} páginas não estabilizam na segunda volta:\n  {}",
        instaveis.len(),
        instaveis
            .iter()
            .take(15)
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join("\n  ")
    );
}
