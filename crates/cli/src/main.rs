//! CLI headless do Anotadinho.
//!
//! Reaproveita os handlers de `anotadinho-ipc` (a mesma lógica que o
//! app Tauri expõe pro frontend Yew) sem nenhuma dependência de Tauri —
//! dá pra listar/ler/buscar/exportar o vault de um terminal comum, sem
//! a janela do app aberta. Pensado pra um agente (Claude Code ou
//! outro processo) conseguir consumir o vault programaticamente.

use anotadinho_ipc::{
    handle_create_page_from_template, handle_export_folder, handle_list_pages,
    handle_list_templates, handle_read_page, handle_search_content, handle_write_page,
};
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "anotadinho-cli",
    version,
    about = "Acesso headless ao vault do Anotadinho — sem precisar da janela Tauri"
)]
struct Cli {
    /// Path do vault (pasta raiz com pages/, journals/, templates/ etc).
    #[arg(long)]
    vault: String,

    /// Emite JSON em vez de texto legível (nos comandos de listagem).
    #[arg(long)]
    json: bool,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Lista páginas em pages/ e journals/, com filtros opcionais.
    ListPages {
        /// Filtra por prefixo de path (ex: pages/specs).
        #[arg(long)]
        folder: Option<String>,
        /// Filtra por tag — repetível, todas devem estar presentes
        /// (AND). Lê o frontmatter de cada página candidata.
        #[arg(long = "tag")]
        tags: Vec<String>,
        /// Filtra por `status` do frontmatter (campo livre, ex: usado
        /// pelas specs do esquema de agent-os).
        #[arg(long)]
        status: Option<String>,
        /// Filtra por `priority` do frontmatter.
        #[arg(long)]
        priority: Option<String>,
    },
    /// Imprime o conteúdo bruto (.md) de uma página.
    Read {
        /// Path relativo ao vault (ex: pages/minha-nota.md).
        page_path: String,
    },
    /// Busca full-text (FTS5) no conteúdo das páginas.
    Search {
        /// Termo de busca.
        query: String,
    },
    /// Exporta o dump concatenado do vault, ou de uma pasta específica.
    Export {
        /// Pasta a exportar (ex: pages/specs). Omitido = vault inteiro.
        #[arg(long)]
        folder: Option<String>,
    },
    /// Lista templates em templates/.
    ListTemplates,
    /// Cria uma página nova a partir de um template, substituindo
    /// `{{title}}` (e `{{date}}`, ciclo 112) pelo título escolhido.
    NewFromTemplate {
        /// Path do template relativo ao vault (ex: templates/spec.md).
        template_path: String,
        /// Título da página nova.
        title: String,
    },
    /// Grava um campo do frontmatter, preservando o corpo intocado.
    /// `title`/`type`/`tags` (lista separada por vírgula) setam o
    /// campo tipado; qualquer outra chave vai pra `extra` (ciclo 098).
    SetProperty {
        /// Path relativo ao vault (ex: pages/specs/minha-spec.md).
        page_path: String,
        /// Nome do campo de frontmatter.
        key: String,
        /// Novo valor.
        value: String,
    },
}

fn main() {
    let cli = Cli::parse();
    if let Err(msg) = run(cli) {
        eprintln!("erro: {}", msg);
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), String> {
    match cli.command {
        Command::ListPages { folder, tags, status, priority } => {
            let mut pages = handle_list_pages(cli.vault.clone())?;
            if let Some(prefix) = &folder {
                pages.retain(|p| p.path.starts_with(prefix.as_str()));
            }
            if !tags.is_empty() || status.is_some() || priority.is_some() {
                let mut filtered = Vec::new();
                for p in pages {
                    let content = handle_read_page(cli.vault.clone(), p.path.clone())?;
                    let fm = anotadinho_core::MarkdownCodec::split_frontmatter(&content)
                        .map(|(fm, _)| fm)
                        .unwrap_or_default();
                    if !tags.iter().all(|t| fm.tags.contains(t)) {
                        continue;
                    }
                    if let Some(want) = &status {
                        if frontmatter_extra_str(&fm, "status") != Some(want.as_str()) {
                            continue;
                        }
                    }
                    if let Some(want) = &priority {
                        if frontmatter_extra_str(&fm, "priority") != Some(want.as_str()) {
                            continue;
                        }
                    }
                    filtered.push(p);
                }
                pages = filtered;
            }
            if cli.json {
                print_json(&pages)?;
            } else {
                for p in pages {
                    println!("{}\t{}\t{}", p.section, p.title, p.path);
                }
            }
        }
        Command::Read { page_path } => {
            let content = handle_read_page(cli.vault, page_path)?;
            print!("{}", content);
        }
        Command::Search { query } => {
            let results = handle_search_content(cli.vault, query)?;
            if cli.json {
                print_json(&results)?;
            } else {
                for (path, snippet) in results {
                    println!("{}\t{}", path, snippet);
                }
            }
        }
        Command::Export { folder } => {
            let dump = handle_export_folder(cli.vault, folder.unwrap_or_default())?;
            print!("{}", dump);
        }
        Command::ListTemplates => {
            let templates = handle_list_templates(cli.vault)?;
            if cli.json {
                print_json(&templates)?;
            } else {
                for t in templates {
                    println!("{}\t{}", t.title, t.path);
                }
            }
        }
        Command::NewFromTemplate { template_path, title } => {
            let meta = handle_create_page_from_template(cli.vault, template_path, title)?;
            println!("{}", meta.path);
        }
        Command::SetProperty { page_path, key, value } => {
            let content = handle_read_page(cli.vault.clone(), page_path.clone())?;
            let updated = anotadinho_core::MarkdownCodec::set_frontmatter_field(&content, &key, &value)
                .map_err(|e| e.to_string())?;
            handle_write_page(cli.vault, page_path, updated)?;
        }
    }
    Ok(())
}

fn print_json<T: serde::Serialize>(value: &T) -> Result<(), String> {
    println!("{}", serde_json::to_string_pretty(value).map_err(|e| e.to_string())?);
    Ok(())
}

/// Lê um campo string de `Frontmatter.extra` (ex: `status`/`priority`,
/// campos livres que não têm um lugar tipado na struct — ver
/// `crates/core/src/page.rs`).
fn frontmatter_extra_str<'a>(fm: &'a anotadinho_core::Frontmatter, key: &str) -> Option<&'a str> {
    fm.extra.get(key).and_then(|v| v.as_str())
}
