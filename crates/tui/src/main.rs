//! O laço: liga o terminal, lê tecla, desenha, desliga.
//!
//! Tudo que dá pra testar mora em `app` e `tela`. Aqui fica só o que
//! precisa de um terminal de verdade — e é curto de propósito.

use anotadinho_ipc::{handle_list_pages, handle_read_page};
use anotadinho_tui::app::{self, Estado};
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::execute;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::io::{self, IsTerminal};

#[derive(Parser)]
#[command(
    name = "anotadinho-tui",
    version,
    about = "O Anotadinho num terminal — só leitura por enquanto"
)]
struct Cli {
    /// Path do vault.
    #[arg(long)]
    vault: String,

    /// Tema: escuro, papel, contraste ou claro.
    ///
    /// São os mesmos quatro da janela, e as cores saem do mesmo
    /// `main.css` (ciclo 288).
    #[arg(long, default_value = "escuro")]
    tema: String,
}

/// Traduz a tecla do crossterm pro nome que o NÚCLEO entende.
///
/// O núcleo fala o vocabulário do `KeyboardEvent.key` do navegador —
/// `"ArrowDown"`, `"Enter"`, `"Escape"` — porque nasceu servindo a
/// janela. É o único lugar onde o navegador vazou pra dentro, e a
/// tradução aqui é o preço disso.
///
/// Vale a pena: `roteamento::Interesse::da_tecla` e `vim::tecla_normal`
/// recebem `&str`, então os dois funcionam num terminal sem uma linha de
/// mudança.
fn nome_da_tecla(k: &KeyEvent) -> Option<String> {
    Some(match k.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Escape".into(),
        KeyCode::Tab => "Tab".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Up => "ArrowUp".into(),
        KeyCode::Down => "ArrowDown".into(),
        KeyCode::Left => "ArrowLeft".into(),
        KeyCode::Right => "ArrowRight".into(),
        KeyCode::Home => "Home".into(),
        KeyCode::End => "End".into(),
        KeyCode::PageUp => "PageUp".into(),
        KeyCode::PageDown => "PageDown".into(),
        _ => return None,
    })
}

fn arvore_de(vault: &str, caminho: &str) -> Result<anotadinho_core::unidade::Unidade, String> {
    let conteudo = handle_read_page(vault.to_string(), caminho.to_string())?;
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
    Ok(anotadinho_core::analise::analisar(corpo))
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let paginas = handle_list_pages(cli.vault.clone())?;
    if paginas.is_empty() {
        return Err(format!("o vault {} não tem páginas", cli.vault));
    }
    let primeira = arvore_de(&cli.vault, &paginas[0].path)?;
    if !anotadinho_tui::tema::TEMAS.contains(&cli.tema.as_str()) {
        return Err(format!(
            "tema \"{}\" não existe — os que existem são: {}",
            cli.tema,
            anotadinho_tui::tema::TEMAS.join(", ")
        ));
    }
    let mut estado = Estado::novo(paginas, primeira).com_tema(&cli.tema);

    // Sem terminal de verdade, `enable_raw_mode` falha com
    // "No such device or address (os error 6)" — que não diz nada a
    // quem rodou isto de um pipe ou de um script. Perguntar antes custa
    // uma linha e devolve uma frase que se entende.
    if !io::stdout().is_terminal() {
        return Err(
            "isto precisa de um terminal de verdade — a saída está redirecionada. \
             Pra ler o vault de um script, use `anotadinho-cli ver`."
                .to_string(),
        );
    }

    // A partir daqui o terminal está em modo cru: qualquer saída sem
    // desligar deixa o shell da pessoa quebrado, então o desligamento
    // acontece ANTES de qualquer erro subir.
    enable_raw_mode().map_err(|e| e.to_string())?;
    let mut saida = io::stdout();
    execute!(saida, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let mut term = Terminal::new(CrosstermBackend::new(saida)).map_err(|e| e.to_string())?;

    let resultado = laco(&mut term, &mut estado, &cli.vault);

    disable_raw_mode().map_err(|e| e.to_string())?;
    execute!(term.backend_mut(), LeaveAlternateScreen).map_err(|e| e.to_string())?;
    term.show_cursor().map_err(|e| e.to_string())?;
    resultado
}

fn laco<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    estado: &mut Estado,
    vault: &str,
) -> Result<(), String> {
    loop {
        term.draw(|f| app::desenhar(f, estado)).map_err(|e| e.to_string())?;
        if estado.sair {
            return Ok(());
        }
        let Event::Key(k) = event::read().map_err(|e| e.to_string())? else {
            continue;
        };
        // O Windows manda Press E Release; sem esta guarda cada tecla
        // anda dois blocos.
        if k.kind != KeyEventKind::Press {
            continue;
        }
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
            return Ok(());
        }
        let Some(nome) = nome_da_tecla(&k) else { continue };
        if let Some(caminho) = app::tecla(estado, &nome) {
            // Página que não abre não derruba a sessão: a pessoa
            // continua no que estava.
            if let Ok(arvore) = arvore_de(vault, &caminho) {
                estado.abrir(arvore);
            }
        }
    }
}
