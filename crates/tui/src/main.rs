//! O laço: liga o terminal, lê tecla, desenha, desliga.
//!
//! Tudo que dá pra testar mora em `app` e `tela`. Aqui fica só o que
//! precisa de um terminal de verdade — e é curto de propósito.

use anotadinho_ipc::{handle_list_pages, handle_read_page_versioned, handle_scan_vault, handle_write_page_checked};
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
    // `Ctrl+R` refaz, `Ctrl+A`/`Ctrl+X` somam (ciclo 322).
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        return match k.code {
            KeyCode::Char(c) => Some(format!("Ctrl+{}", c.to_ascii_lowercase())),
            _ => None,
        };
    }
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

/// O arquivo de uma página e a versão dele (ciclo 318): a edição grava
/// por cima só se o arquivo ainda estiver nessa versão.
fn ler(vault: &str, caminho: &str) -> Result<(String, Option<String>), String> {
    let p = handle_read_page_versioned(vault.to_string(), caminho.to_string())?;
    Ok((p.content, p.version))
}

/// O dia de hoje no fuso da pessoa, `AAAA-MM-DD` (ciclo 315).
///
/// O núcleo não lê relógio de propósito; quem lê é quem roda. No Unix o
/// fuso vem do `localtime_r`; fora dele, UTC — um dia a menos ou a mais
/// perto da meia-noite, e só isso.
fn hoje_local() -> String {
    let agora = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0);
    #[cfg(unix)]
    {
        let t: libc::time_t = agora as libc::time_t;
        let mut tm: libc::tm = unsafe { std::mem::zeroed() };
        // SAFETY: `t` e `tm` são locais válidos; `localtime_r` só escreve
        // em `tm` e devolve nulo se falhar.
        let ok = unsafe { !libc::localtime_r(&t, &mut tm).is_null() };
        if ok {
            return anotadinho_core::date_util::format_date(tm.tm_year + 1900, (tm.tm_mon + 1) as u32, tm.tm_mday as u32);
        }
    }
    anotadinho_core::date_util::add_days("1970-01-01", agora.div_euclid(86_400)).unwrap_or_default()
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let paginas = handle_list_pages(cli.vault.clone())?;
    if paginas.is_empty() {
        return Err(format!("o vault {} não tem páginas", cli.vault));
    }
    let (texto, versao) = ler(&cli.vault, &paginas[0].path)?;
    let primeira = {
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
        anotadinho_core::analise::analisar(corpo)
    };
    if !anotadinho_tui::tema::TEMAS.contains(&cli.tema.as_str()) {
        return Err(format!(
            "tema \"{}\" não existe — os que existem são: {}",
            cli.tema,
            anotadinho_tui::tema::TEMAS.join(", ")
        ));
    }
    let mut estado = Estado::novo(paginas, primeira)
        .com_texto(&texto, versao)
        .com_tema(&cli.tema)
        .com_hoje(&hoje_local())
        // Os calendários em modo vault leem as páginas com data. Varrer
        // falhando não impede a TUI: o calendário só fica vazio.
        .com_eventos_do_vault(
            handle_scan_vault(cli.vault.clone())
                .map(|p| anotadinho_core::calendario::entradas_do_vault(&p))
                .unwrap_or_default(),
        );

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
            if let Ok((texto, versao)) = ler(vault, &caminho) {
                estado.abrir_texto(&texto, versao);
            }
        }
        // Uma edição deixou texto novo: grava com a trava de versão. Se
        // o arquivo mudou por fora, a gravação é recusada, a página volta
        // a ser a do disco e o rodapé diz por quê (ciclo 318).
        if let Some(conteudo) = estado.gravacao.take() {
            let caminho = estado.paginas.get(estado.pagina).map(|p| p.path.clone());
            if let Some(caminho) = caminho {
                match handle_write_page_checked(vault.to_string(), caminho.clone(), conteudo, estado.versao.clone()) {
                    Ok(v) => estado.versao = Some(v),
                    Err(motivo) => {
                        if let Ok((texto, versao)) = ler(vault, &caminho) {
                            estado.abrir_texto(&texto, versao);
                        }
                        estado.aviso = Some(format!("não gravou: {motivo}"));
                    }
                }
            }
        }
    }
}
