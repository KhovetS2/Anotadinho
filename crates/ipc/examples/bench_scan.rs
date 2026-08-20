//! Mede a varredura do vault, fria e quente (ciclo 171).
//!
//! ```bash
//! cargo run --release -p anotadinho-ipc --example bench_scan -- <vault>
//! ```
//!
//! A primeira rodada é o custo SEM cache (o cache é apagado antes); as
//! seguintes usam o cache em disco. Serve pra decidir com número, não
//! com chute, se vale mexer no índice de novo.

fn main() {
    let vault = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("uso: bench_scan <path do vault>");
        std::process::exit(2);
    });
    let _ = std::fs::remove_file(std::path::Path::new(&vault).join(".anotadinho/index.json"));

    for rodada in 1..=3 {
        let inicio = std::time::Instant::now();
        let total = match anotadinho_ipc::handle_scan_vault(vault.clone()) {
            Ok(v) => v.len(),
            Err(e) => {
                eprintln!("erro: {e}");
                std::process::exit(1);
            }
        };
        let rotulo = if rodada == 1 { "fria " } else { "quente" };
        println!("{rotulo} | {total} páginas em {:?}", inicio.elapsed());
    }
}
