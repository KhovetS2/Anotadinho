//! O laço: liga o terminal, lê tecla, desenha, desliga.
//!
//! Tudo que dá pra testar mora em `app` e `tela`. Aqui fica só o que
//! precisa de um terminal de verdade — e é curto de propósito.

use anotadinho_ipc::{
    handle_create_page_typed, handle_delete_page, handle_list_pages, handle_open_today_journal, handle_read_page_versioned,
    handle_scan_vault, handle_write_page, handle_write_page_checked,
};
use anotadinho_tui::app::especiais::{self, TipoEspecial};
use anotadinho_tui::app::{Pedido, Preferencias};
use anotadinho_tui::app::{self, Estado};
use anotadinho_tui::sistema;
use clap::Parser;
use crossterm::event::{self, DisableBracketedPaste, DisableMouseCapture, EnableBracketedPaste, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers, MouseEventKind};
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
    /// Path do vault. Sem ele, reabre o último (ciclo 372), como a janela.
    #[arg(long)]
    vault: Option<String>,

    /// Prepara a pasta como vault novo — estrutura, templates, padrões,
    /// prompts e o guia — sem perguntar. Nunca sobrescreve nada.
    #[arg(long)]
    criar: bool,

    /// Tema: mocha (o padrão), escuro, papel, contraste ou claro.
    ///
    /// São os mesmos quatro da janela, e as cores saem do mesmo
    /// `main.css` (ciclo 288).
    /// Sem ele, vale o tema das preferências (a barra de comandos troca
    /// e grava, ciclo 339).
    #[arg(long)]
    tema: Option<String>,
}

/// Pergunta sim/não no terminal, antes do modo cru. Sem terminal, não.
fn perguntar_no_terminal(pergunta: &str) -> bool {
    use std::io::Write;
    if !io::stdin().is_terminal() {
        return false;
    }
    eprint!("{pergunta}");
    let _ = io::stderr().flush();
    let mut resposta = String::new();
    io::stdin().read_line(&mut resposta).is_ok() && matches!(resposta.trim().to_lowercase().as_str(), "s" | "sim" | "y" | "yes")
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
    // `Alt+1`…`Alt+9`, `Alt+H/L/Q`: as abas (ciclo 354).
    if k.modifiers.contains(KeyModifiers::ALT) {
        return match k.code {
            KeyCode::Char(c) => Some(format!("Alt+{}", c.to_ascii_lowercase())),
            _ => None,
        };
    }
    if k.modifiers.contains(KeyModifiers::CONTROL) {
        return match k.code {
            KeyCode::Char(c) => Some(format!("Ctrl+{}", c.to_ascii_lowercase())),
            // O começo e o fim do texto no editor de código (ciclo 506).
            KeyCode::Home => Some("Ctrl+Home".into()),
            KeyCode::End => Some("Ctrl+End".into()),
            _ => None,
        };
    }
    // Shift+setas/Home/End selecionam na inserção (ciclo 395).
    if k.modifiers.contains(KeyModifiers::SHIFT) {
        let nome = match k.code {
            KeyCode::Left => Some("Shift+ArrowLeft"),
            KeyCode::Right => Some("Shift+ArrowRight"),
            KeyCode::Home => Some("Shift+Home"),
            KeyCode::End => Some("Shift+End"),
            // Selecionar linhas no editor de código (ciclo 512).
            KeyCode::Up => Some("Shift+ArrowUp"),
            KeyCode::Down => Some("Shift+ArrowDown"),
            _ => None,
        };
        if let Some(n) = nome {
            return Some(n.into());
        }
    }
    Some(match k.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Escape".into(),
        KeyCode::Tab => "Tab".into(),
        // Desindentar no editor de código (ciclo 506). O terminal manda
        // Shift+Tab como uma tecla própria, não como Tab com Shift.
        KeyCode::BackTab => "Shift+Tab".into(),
        KeyCode::Backspace => "Backspace".into(),
        KeyCode::Delete => "Delete".into(),
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

/// Manda `texto` pra área de transferência do sistema pelo OSC 52 (ciclo
/// 512): o terminal é quem copia, então funciona por SSH e dentro do tmux
/// (com `set-clipboard on`), sem depender de X11, Wayland ou do Windows.
/// Terminal que não entende a sequência só a ignora.
fn copiar_pro_sistema(texto: &str) {
    use std::io::Write;
    let mut saida = io::stdout();
    let _ = write!(saida, "\x1b]52;c;{}\x07", base64(texto.as_bytes()));
    let _ = saida.flush();
}

/// Base64 padrão, sem dependência: é só pro OSC 52.
fn base64(dados: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut fora = String::with_capacity(dados.len().div_ceil(3) * 4);
    for bloco in dados.chunks(3) {
        let b = [bloco[0], *bloco.get(1).unwrap_or(&0), *bloco.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        for i in 0..4 {
            if i <= bloco.len() {
                fora.push(T[((n >> (18 - 6 * i)) & 63) as usize] as char);
            } else {
                fora.push('=');
            }
        }
    }
    fora
}

#[cfg(test)]
mod testes_do_base64 {
    #[test]
    fn base64_como_o_padrao() {
        assert_eq!(super::base64(b""), "");
        assert_eq!(super::base64(b"f"), "Zg==");
        assert_eq!(super::base64(b"fo"), "Zm8=");
        assert_eq!(super::base64(b"foo"), "Zm9v");
        assert_eq!(super::base64("ação".as_bytes()), "YcOnw6Nv");
    }
}

/// O arquivo de uma página e a versão dele (ciclo 318): a edição grava
/// por cima só se o arquivo ainda estiver nessa versão.
fn ler(vault: &str, caminho: &str) -> Result<(String, Option<String>), String> {
    let p = handle_read_page_versioned(vault.to_string(), caminho.to_string())?;
    Ok((p.content, p.version))
}

/// O dia de hoje no fuso da pessoa, `AAAA-MM-DD` (ciclo 315).
///
/// O núcleo não lê relógio de propósito; quem lê é quem roda.
fn hoje_local() -> String {
    agora_local().split(' ').next().unwrap_or_default().to_string()
}

/// Agora, `AAAA-MM-DD HH:MM`, no fuso da pessoa — o carimbo das conversas
/// (ciclo 339).
///
/// Pelo `chrono` (ciclo 448), e não pelo `localtime_r` da libc: aquele
/// só existia no Unix, e no Windows a TUI caía em UTC — depois das 21h
/// em Brasília, o calendário já marcava amanhã como hoje. Um caminho só
/// nos três sistemas é o que o Linux consegue exercer; e é o mesmo
/// relógio que a janela usa pro registro de execuções (ciclo 441).
fn agora_local() -> String {
    chrono::Local::now().format("%Y-%m-%d %H:%M").to_string()
}

/// Onde as preferências da TUI moram: fora do vault, na pasta de
/// configuração da pessoa (`sistema::arquivo_de_preferencias`).
fn caminho_das_preferencias() -> Option<std::path::PathBuf> {
    sistema::arquivo_de_preferencias(sistema::ambiente, cfg!(windows)).map(std::path::PathBuf::from)
}

fn ler_preferencias() -> Preferencias {
    caminho_das_preferencias()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .and_then(|t| serde_json::from_str(&t).ok())
        .unwrap_or_default()
}

fn gravar_preferencias(p: &Preferencias) -> Result<(), String> {
    let caminho = caminho_das_preferencias().ok_or("sem pasta de configuração")?;
    if let Some(pai) = caminho.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(p).map_err(|e| e.to_string())?;
    std::fs::write(caminho, json).map_err(|e| e.to_string())
}

/// Abre uma página pelo caminho: a lista aponta pra ela e o texto vem do
/// disco.
fn abrir(estado: &mut Estado, vault: &str, caminho: &str) {
    salvar_pendente(estado, vault);
    if let Some(i) = estado.paginas.iter().position(|p| p.path == caminho) {
        estado.pagina = i;
    }
    match ler(vault, caminho) {
        Ok((texto, versao)) => estado.abrir_texto(&texto, versao),
        Err(e) => estado.aviso = Some(format!("não abriu: {e}")),
    }
}

/// A fila desta UI (ciclo 429): a política é do núcleo, a carga é o que
/// a TUI precisa pra disparar quando a vez chegar — a pergunta e os
/// anexos, porque aqui o prompt é montado no disparo.
type FilaDaTui = anotadinho_core::fila::Fila<(String, Vec<String>)>;

/// As execuções do agente, por conversa (ciclo 340). Vivem aqui, fora do
/// estado da tela: sair da conversa não para o agente, e a resposta cai no
/// arquivo quando ele acaba — como o registro de jobs do backend da janela.
type Trabalhos = std::collections::HashMap<String, EmAndamento>;

/// Uma execução rodando e o que o registro vai precisar dela no fim
/// (ciclo 406).
struct EmAndamento {
    trabalho: anotadinho_tui::agente::Trabalho,
    /// O começo, pro registro dizer quando foi.
    quando: String,
    /// Nome e executável do agente que está rodando.
    agente: String,
    binario: String,
    /// Quantas páginas foram de contexto e o tamanho do prompt.
    anexos: usize,
    prompt: usize,
}

/// O adaptador em uso — quem decide se dá pra continuar sessão.
fn adaptador_da_conversa(estado: &Estado) -> anotadinho_core::agente::Adaptador {
    estado.preferencias.agente.clone().unwrap_or_default().migrado()
}

/// Esta conversa continua uma sessão aberta? (ciclo 433)
///
/// Uma função só porque o ENVIO e a PRÉVIA precisam da mesma resposta:
/// com sessão, o histórico não vai no prompt, e uma prévia que mostrasse
/// o histórico estaria mentindo sobre o que o agente recebe.
fn sessao_da_conversa(estado: &Estado, vault: &str, path: &str) -> Option<String> {
    if adaptador_da_conversa(estado).arg_sessao.trim().is_empty() {
        return None;
    }
    let texto = anotadinho_ipc::handle_read_page(vault.to_string(), path.to_string()).ok()?;
    let (fm, _) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
    anotadinho_core::conversa::sessao_do_frontmatter(fm)
}

/// Grava a configuração MCP desta execução e devolve o caminho dela
/// (ciclo 426). Vazio quando não deu — o prompt ainda explica o CLI —
/// ou quando não há CLI nenhum pra atender (ciclo 450).
fn escrever_config_mcp(vault: &str) -> String {
    let Some(cli) = anotadinho_core::agente::cli_ao_lado(std::env::current_exe().ok()) else {
        return String::new();
    };
    let json = anotadinho_core::agente::config_mcp(&cli, vault);
    // Junto das propostas: é estado daquele vault, e some com ele.
    let arquivo = std::path::Path::new(vault).join(".anotadinho/mcp.json");
    if let Some(pai) = arquivo.parent() {
        let _ = std::fs::create_dir_all(pai);
    }
    match std::fs::write(&arquivo, json) {
        Ok(()) => arquivo.to_string_lossy().to_string(),
        Err(_) => String::new(),
    }
}

/// A chave de uma execução: o vault e a conversa (ciclo 396).
fn chave_do_trabalho(vault: &str, conversa: &str) -> String {
    format!("{vault}\u{0}{conversa}")
}

/// Lê o arquivo, acrescenta a mensagem e grava de volta (com o
/// frontmatter).
fn acrescentar_mensagem(vault: &str, conversa: &str, mensagem: &anotadinho_core::conversa::Mensagem) -> Result<String, String> {
    let atual = anotadinho_ipc::handle_read_page(vault.to_string(), conversa.to_string())?;
    let (frontmatter, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
    let novo_corpo = anotadinho_core::conversa::append(corpo, mensagem);
    let novo = if frontmatter.is_empty() { novo_corpo } else { format!("{frontmatter}\n{novo_corpo}") };
    handle_write_page(vault.to_string(), conversa.to_string(), novo)?;
    Ok(corpo.to_string())
}

/// O que vai pro agente (ciclo 417), com o recado da transclusão.
///
/// A montagem em si é do núcleo desde o ciclo 423 (`core::envio`), pra a
/// janela usar a MESMA — aqui fica só o que precisa de disco: ler os
/// anexos com as transclusões resolvidas.
struct Envio {
    prompt: String,
    orcamento: anotadinho_core::orcamento::Orcamento,
    /// O recado sobre a transclusão, quando há o que dizer.
    aviso: Option<String>,
    /// O que a poda cortou pra caber no teto (ciclo 419).
    cortes: Vec<anotadinho_core::orcamento::Corte>,
}

/// Lê os anexos (transclusões resolvidas) e monta o envio.
///
/// O envio e a PRÉVIA passam os dois por aqui. Se fossem dois caminhos,
/// a prévia mentiria no primeiro dia em que um deles mudasse — e uma
/// prévia que mente é pior que nenhuma.
fn montar_envio(
    vault: &str,
    historico: &[anotadinho_core::conversa::Mensagem],
    pergunta: &str,
    anexos: &[String],
    conversa_path: &str,
    teto: usize,
) -> Envio {
    use anotadinho_core::conversa;
    let mut trazidas: Vec<String> = Vec::new();
    let mut avisos: Vec<String> = Vec::new();
    let mut contextos: Vec<conversa::Contexto> = anexos
        .iter()
        .filter(|a| a.as_str() != conversa_path)
        .filter_map(|a| {
            // Anexo que não abre precisa APARECER: some do prompt, e
            // sem recado a resposta sai pior sem ninguém entender.
            let expandida = match anotadinho_ipc::handle_ler_para_contexto(vault.to_string(), a.clone()) {
                Ok(x) => x,
                Err(_) => {
                    avisos.push(format!("não consegui ler {a}"));
                    return None;
                }
            };
            for t in expandida.trazidas {
                if !trazidas.contains(&t) {
                    trazidas.push(t);
                }
            }
            avisos.extend(expandida.avisos);
            Some(conversa::Contexto { nome: a.clone(), conteudo: expandida.texto })
        })
        .collect();
    // Conversa de projeto (ciclo 461): o índice das tarefas vai junto,
    // como na janela (460) — sem ele, "quebre em tarefas" devolvia de
    // novo o que já estava no quadro.
    if let Some(slug) = anotadinho_core::projeto::projeto_do_caminho(conversa_path) {
        match anotadinho_ipc::handle_indice_do_projeto(vault.to_string(), slug.to_string()) {
            Ok(indice) => contextos.push(conversa::Contexto {
                nome: format!("{} (índice)", anotadinho_core::projeto::pasta_das_tarefas(slug)),
                conteudo: indice,
            }),
            Err(_) => avisos.push(format!("não consegui ler as tarefas de {slug}")),
        }
    }
    let montado = anotadinho_core::envio::montar(
        &contextos,
        historico,
        pergunta,
        app::conversa::HISTORICO_NO_PROMPT,
        teto,
        vault,
    );
    let mut nota = if trazidas.is_empty() {
        String::new()
    } else {
        format!("transclusão trouxe {} página(s)", trazidas.len())
    };
    if !avisos.is_empty() {
        if !nota.is_empty() {
            nota.push_str(" · ");
        }
        nota.push_str(&avisos.join(" · "));
    }
    Envio {
        prompt: montado.prompt,
        orcamento: montado.orcamento,
        aviso: (!nota.is_empty()).then_some(nota),
        cortes: montado.cortes,
    }
}

/// Grava a pergunta e dispara o agente com o histórico e os anexos —
/// ou põe na fila, se o limite de paralelismo já está cheio (ciclo 408).
///
/// `vault` é o da conversa; `vault_atual` é o que a TUI mostra. Eles
/// diferem quando a vez de um envio enfileirado chega depois de trocar
/// de vault (ciclo 396) — aí a resposta vai pro arquivo certo, mas a
/// tela não se mexe.
fn enviar_na_conversa(
    estado: &mut Estado,
    vault: &str,
    vault_atual: &str,
    trabalhos: &mut Trabalhos,
    fila: &mut FilaDaTui,
    path: &str,
    pergunta: &str,
    anexos: &[String],
) {
    use anotadinho_core::conversa::{self, Autor, Mensagem};
    if trabalhos.contains_key(&chave_do_trabalho(vault, path)) {
        estado.aviso = Some("já tem uma execução em andamento nesta conversa".into());
        return;
    }
    if fila.ja_espera(vault, path) {
        estado.aviso = Some("esta conversa já tem um envio esperando vaga".into());
        return;
    }
    // Sem vaga, o envio espera. A pergunta NÃO vai pro arquivo agora:
    // ela entra junto com a execução, pra o histórico do prompt não
    // ficar com duas perguntas e uma resposta.
    if !fila.tem_vaga(trabalhos.len()) {
        let posicao = fila.enfileirar(anotadinho_core::fila::Espera {
            vault: vault.to_string(),
            conversa: path.to_string(),
            carga: (pergunta.to_string(), anexos.to_vec()),
        });
        estado.aviso = Some(format!("{posicao}º na fila — {} rodando", trabalhos.len()));
        if let Some(c) = estado.conversa.as_mut() {
            if c.path == path {
                c.na_fila = Some(posicao);
                c.rascunho = Default::default();
            }
        }
        return;
    }
    let minha = Mensagem { autor: Autor::Voce, quando: agora_local(), texto: pergunta.to_string() };
    let corpo_antes = match acrescentar_mensagem(vault, path, &minha) {
        Ok(c) => c,
        Err(e) => {
            estado.aviso = Some(format!("não gravou a pergunta: {e}"));
            return;
        }
    };
    let historico = conversa::parse(&corpo_antes);
    // Sessão contínua (ciclo 433): se o agente já tem a conversa, o
    // prompt não remonta o histórico — o que ele já entendeu fica com
    // ele, e o envio é muito menor.
    let sessao = sessao_da_conversa(estado, vault, path);
    let historico_no_prompt: Vec<conversa::Mensagem> =
        if sessao.is_some() { Vec::new() } else { historico.clone() };
    let Envio { prompt, aviso, cortes, .. } = montar_envio(
        vault,
        &historico_no_prompt,
        pergunta,
        anexos,
        path,
        estado.preferencias.teto_de_contexto,
    );
    if let Some(nota) = aviso {
        estado.aviso = Some(nota);
    }
    if !cortes.is_empty() {
        // A poda não pode ser surpresa: ela mudou o que o agente vai ler.
        estado.aviso = Some(format!(
            "contexto podado pra caber — {}",
            cortes.iter().map(|c| c.rotulo()).collect::<Vec<_>>().join(" · ")
        ));
    }
    let adaptador = estado.preferencias.agente.clone().unwrap_or_default().migrado();
    subir_agente(estado, vault, vault_atual, trabalhos, path, &prompt, &adaptador, anexos.len(), sessao.as_deref());
}

/// Sobe o processo do agente pra uma conversa cuja pergunta JÁ está no
/// arquivo e cujo prompt JÁ está montado.
///
/// Separado de `enviar_na_conversa` no ciclo 459: a fila dos projetos
/// chega com as duas coisas prontas pelo `ipc` (que é compartilhado com
/// a janela), e só precisa da parte que inicia e acompanha.
#[allow(clippy::too_many_arguments)]
fn subir_agente(
    estado: &mut Estado,
    vault: &str,
    vault_atual: &str,
    trabalhos: &mut Trabalhos,
    path: &str,
    prompt: &str,
    adaptador: &anotadinho_core::agente::Adaptador,
    anexos: usize,
    sessao: Option<&str>,
) -> bool {
    let cwd = if adaptador.cwd.trim().is_empty() {
        anotadinho_core::agente::raiz_do_projeto(vault, |d| d.join(".git").exists())
    } else {
        adaptador.cwd.clone()
    };
    // Liga o MCP do Anotadinho nesta execução (ciclo 426): o agente
    // ganha ler_pagina, buscar e propor apontados PRA ESTE vault, sem
    // ninguém configurar nada por fora.
    let config_mcp = escrever_config_mcp(vault);
    // O mesmo aviso da janela (ciclo 450): sem ele, um agente sem as
    // ferramentas do vault só se denuncia pela resposta pior.
    let aviso_mcp = (config_mcp.is_empty()
        && !adaptador.arg_mcp.trim().is_empty()
        && anotadinho_core::agente::cli_ao_lado(std::env::current_exe().ok()).is_none())
    .then(|| anotadinho_core::agente::AVISO_SEM_CLI.to_string());
    let subiu = match anotadinho_tui::agente::Trabalho::iniciar(
        adaptador,
        prompt,
        &cwd,
        &config_mcp,
        sessao.unwrap_or(""),
    ) {
        Ok(t) => {
            if let Some(a) = &aviso_mcp {
                estado.aviso = Some(a.clone());
            }
            trabalhos.insert(
                chave_do_trabalho(vault, path),
                EmAndamento {
                    trabalho: t.com_aviso(aviso_mcp),
                    quando: agora_local(),
                    agente: adaptador.nome.clone(),
                    binario: adaptador.binario.clone(),
                    anexos,
                    prompt: prompt.chars().count(),
                },
            );
            true
        }
        Err(e) => {
            // Se era tarefa de projeto, ela não pode ficar em
            // `executando` sem processo (ciclo 459).
            let _ = anotadinho_ipc::handle_fechar_tarefa_da_conversa(
                vault.to_string(),
                path.to_string(),
                anotadinho_ipc::FimDaTarefa::Falhou(e.clone()),
            );
            if let Some(c) = estado.conversa.as_mut() {
                c.erro = Some(e);
            }
            false
        }
    };
    if vault == vault_atual && estado.paginas.get(estado.pagina).is_some_and(|p| p.path == path) {
        abrir(estado, vault, path);
    }
    subiu
}

/// As tarefas que a thread de fechamento já fechou (ciclo 520): vault,
/// conversa e o resultado. Sem isto o aviso "fechando a tarefa…" ficava
/// na tela pra sempre, o erro do fechamento (um commit, a escrita da
/// etapa) sumia calado, e o quadro só mudava na releitura de 3 s — que
/// não roda enquanto a pessoa aperta tecla.
type Fechada = (String, String, Result<Option<anotadinho_core::projeto::EtapaTarefa>, String>);
static FECHADAS: std::sync::Mutex<Vec<Fechada>> = std::sync::Mutex::new(Vec::new());

/// Colhe o que a thread de fechamento terminou: diz no aviso pra onde a
/// tarefa foi (ou o erro) e relê a tela do projeto, se for ela a aberta.
fn colher_fechadas(estado: &mut Estado, vault: &str) {
    let prontas: Vec<Fechada> = match FECHADAS.lock() {
        Ok(mut f) => std::mem::take(&mut *f),
        Err(_) => return,
    };
    let mut deste = false;
    for (v, conversa, r) in prontas {
        deste |= v == vault;
        // O nome do arquivo da tarefa (`001-login`) e não o caminho da
        // conversa: o aviso divide a linha do rodapé com as dicas.
        let tarefa = anotadinho_ipc::handle_tarefa_da_conversa(v.clone(), conversa.clone()).unwrap_or(conversa);
        let nome = std::path::Path::new(&tarefa).file_stem().and_then(|n| n.to_str()).unwrap_or(&tarefa).to_string();
        // No laço de revisão (ciclo 522) a tarefa continua em `executando`:
        // "→ Executando" não diria nada. O ponto do laço diz.
        let no_laco = anotadinho_ipc::handle_read_page(v.clone(), tarefa.clone())
            .ok()
            .and_then(|t| anotadinho_core::projeto::Tarefa::ler(&tarefa, &t))
            .and_then(|t| t.revisao_ia);
        match r {
            Ok(Some(anotadinho_core::projeto::EtapaTarefa::Executando)) => {
                use anotadinho_core::revisor::estado as laco;
                estado.aviso = Some(match no_laco.as_deref() {
                    Some(laco::PENDENTE) => format!("tarefa {nome}: passou · vai pro revisor"),
                    Some(laco::AJUSTES) => format!("tarefa {nome}: o revisor pediu ajustes · volta pro executor"),
                    _ => format!("tarefa {nome} → Executando"),
                })
            }
            Ok(Some(etapa)) => estado.aviso = Some(format!("tarefa {nome} → {}", etapa.label())),
            Ok(None) => {}
            Err(e) => estado.aviso = Some(format!("não fechou a tarefa {nome}: {}", e.lines().next().unwrap_or_default())),
        }
    }
    if deste && estado.especial.as_ref().is_some_and(|x| x.tipo == TipoEspecial::Projeto) {
        carregar_especial(estado, vault, TipoEspecial::Projeto);
    }
}

/// A fila dos projetos nesta TUI (ciclo 459): o que foi avisado por
/// último, pra só repetir quando mudar.
#[derive(Default)]
struct Projetos {
    avisos: Vec<String>,
}

/// Confere a fila dos projetos e sobe o agente de cada tarefa reservada.
///
/// A decisão do que roda é do `ipc`, a mesma da janela. Só reserva o
/// que cabe agora nesta TUI: tarefa reservada que fosse pra espera
/// ficaria em `executando` sem processo nenhum.
fn conferir_projetos(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos, fila: &FilaDaTui, projetos: &mut Projetos) {
    let limite = estado.preferencias.limite_de_agentes.max(1);
    let vagas = limite.saturating_sub(trabalhos.len() + fila.len());
    let params = anotadinho_ipc::ParametrosDaFila {
        agora: agora_local(),
        teto: estado.preferencias.teto_de_contexto,
        limite_historico: app::conversa::HISTORICO_NO_PROMPT,
        vagas: Some(vagas),
    };
    let resultado = match anotadinho_ipc::handle_tarefas_a_executar(vault.to_string(), params) {
        Ok(r) => r,
        Err(e) => {
            let aviso = vec![format!("projetos: {e}")];
            if projetos.avisos != aviso {
                estado.aviso = Some(aviso[0].clone());
                projetos.avisos = aviso;
            }
            return;
        }
    };
    if resultado.avisos != projetos.avisos {
        if !resultado.avisos.is_empty() {
            estado.aviso = Some(format!("projetos: {}", resultado.avisos.join(" · ")));
        }
        projetos.avisos = resultado.avisos;
    }
    let base = adaptador_da_conversa(estado);
    for t in resultado.iniciar {
        // O agente do papel (ciclo 521), pela regra única do núcleo. Nome
        // que não existe fecha a tarefa com o motivo: reservada e sem
        // processo, ela prenderia a vaga do projeto.
        let mut adaptador = match anotadinho_core::agente::escolher(t.agente.as_deref(), &estado.preferencias.agentes, &base) {
            Ok(a) => a,
            Err(e) => {
                let _ = anotadinho_ipc::handle_fechar_tarefa_da_conversa(
                    vault.to_string(),
                    t.conversa.clone(),
                    anotadinho_ipc::FimDaTarefa::Falhou(e.clone()),
                );
                estado.aviso = Some(format!("{}: {e}", t.tarefa));
                continue;
            }
        };
        // O repositório entra por PARÂMETRO, no campo `cwd` que o
        // adaptador já tinha — o mesmo caminho da janela.
        adaptador.cwd = t.cwd.clone();
        let sessao = sessao_da_conversa(estado, vault, &t.conversa);
        if subir_agente(estado, vault, vault, trabalhos, &t.conversa, &t.prompt, &adaptador, 0, sessao.as_deref()) {
            estado.aviso = Some(match t.papel {
                anotadinho_core::projeto::Papel::Revisor => format!("revisando: {} ({})", t.tarefa, adaptador.nome),
                anotadinho_core::projeto::Papel::Executor => format!("tarefa em execução: {}", t.tarefa),
            });
        }
    }
}

/// A cada volta: mostra o agente rodando na conversa aberta e, quando ele
/// acaba, grava a resposta (ou o erro) e relê a conversa.
fn acompanhar(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos, fila: &mut FilaDaTui) {
    use anotadinho_core::conversa::{Autor, Mensagem};
    let mut prontos = Vec::new();
    for (chave, a) in trabalhos.iter_mut() {
        if let Some(fim) = a.trabalho.terminou() {
            // O uso é lido JUNTO do fim: depois disto o trabalho sai do
            // mapa e não há a quem perguntar.
            prontos.push((chave.clone(), fim, a.trabalho.segundos(), a.trabalho.uso(), a.trabalho.sessao()));
        }
    }
    for (chave, fim, segundos, uso, sessao) in prontos {
        let em_andamento = trabalhos.remove(&chave);
        // A resposta vai pro vault da conversa, mesmo que a TUI esteja em
        // outro agora (ciclo 396).
        let (vault_da_conversa, path) = chave.split_once('\u{0}').map(|(v, p)| (v.to_string(), p.to_string())).unwrap_or_default();
        let deste_vault = vault_da_conversa == vault;
        let vault = vault_da_conversa.as_str();
        // O registro da execução (ciclo 406): o que rodou, quanto durou e
        // como terminou. Vai pro vault da conversa.
        if let Some(a) = &em_andamento {
            use anotadinho_core::execucao::{Execucao, Fim};
            let como = match &fim {
                Ok(_) => Fim::Respondeu,
                Err(e) if e.contains("interrompida") => Fim::Interrompida,
                Err(e) => Fim::Falhou(e.clone()),
            };
            let registro = Execucao {
                quando: a.quando.clone(),
                conversa: path.clone(),
                agente: a.agente.clone(),
                binario: a.binario.clone(),
                anexos: a.anexos,
                prompt: a.prompt,
                segundos,
                fim: como,
                uso,
            };
            if let Err(e) = anotadinho_ipc::handle_registrar_execucao(vault.to_string(), registro) {
                estado.aviso = Some(format!("não registrou a execução: {e}"));
            }
        }
        // A sessão aberta pelo agente fica na página (ciclo 433): a
        // próxima pergunta continua de onde parou. Falhou? A sessão sai,
        // e o envio seguinte começa do zero — é o conserto automático de
        // um id que o agente não reconhece mais.
        if let Ok(atual) = anotadinho_ipc::handle_read_page(vault.to_string(), path.clone()) {
            let nova = if fim.is_ok() { sessao.as_deref() } else { None };
            let atualizado = anotadinho_core::conversa::reescrever_sessao(&atual, nova);
            if atualizado != atual {
                let _ = handle_write_page(vault.to_string(), path.clone(), atualizado);
            }
        }
        // A tarefa dona desta conversa sai de `executando` (ciclo 459),
        // DEPOIS da resposta ir pro arquivo: quem abrir a tarefa em
        // `revisao` tem que achar o relato lá.
        let fim_da_tarefa = match &fim {
            Ok(texto) => anotadinho_ipc::FimDaTarefa::Respondeu(texto.clone()),
            Err(e) if e.contains("interrompida") => anotadinho_ipc::FimDaTarefa::Interrompida,
            Err(e) => anotadinho_ipc::FimDaTarefa::Falhou(e.clone()),
        };
        let erro = match fim {
            Ok(texto) => acrescentar_mensagem(vault, &path, &Mensagem { autor: Autor::Agente, quando: agora_local(), texto })
                .err()
                .map(|e| format!("não gravou a resposta: {e}")),
            Err(e) => Some(e),
        };
        // Numa thread (ciclo 517): com `verificar` no projeto, fechar a
        // tarefa roda a suíte do SaaS, que leva minutos — no laço, a TUI
        // congelava junto. A tarefa fica em `executando` até a thread
        // acabar, então a vaga do projeto continua ocupada; quando ela
        // acaba, `colher_fechadas` avisa e relê o quadro.
        if anotadinho_ipc::handle_tarefa_da_conversa(vault.to_string(), path.clone()).is_some() {
            estado.aviso = Some(format!("fechando a tarefa de {path} (verificação no repositório, se houver)…"));
        }
        {
            let (v, p) = (vault.to_string(), path.clone());
            std::thread::spawn(move || {
                let r = anotadinho_ipc::handle_fechar_tarefa_da_conversa(v.clone(), p.clone(), fim_da_tarefa);
                if let Ok(mut f) = FECHADAS.lock() {
                    f.push((v, p, r));
                }
            });
        }
        let aberta = deste_vault && estado.paginas.get(estado.pagina).is_some_and(|p| p.path == path);
        if aberta {
            if let Some(c) = estado.conversa.as_mut() {
                c.trabalho = None;
            }
            abrir(estado, vault, &path);
            if let Some(c) = estado.conversa.as_mut() {
                c.erro = erro;
                c.selecionada = c.mensagens.len();
            }
        } else if let Some(e) = erro {
            estado.aviso = Some(format!("{path}: {e}"));
        }
    }
    // Vaga aberta chama a próxima da fila (ciclo 408).
    fila.limite = estado.preferencias.limite_de_agentes;
    while let Some(e) = fila.proxima(trabalhos.len()) {
        let antes = trabalhos.len();
        let (pergunta, anexos) = e.carga.clone();
        enviar_na_conversa(estado, &e.vault, vault, trabalhos, fila, &e.conversa, &pergunta, &anexos);
        // Se não subiu (binário quebrado, arquivo sumiu), para de puxar
        // pra não girar a fila inteira em erro na mesma volta.
        if trabalhos.len() == antes {
            break;
        }
    }
    if let Some(c) = estado.conversa.as_mut() {
        c.trabalho = trabalhos.get(&chave_do_trabalho(vault, &c.path)).map(|a| (a.trabalho.segundos(), a.trabalho.parcial()));
        c.na_fila = fila.posicao(vault, &c.path);
    }
}

/// A proposta pelo id, pra o registro saber alvo e autor (ciclo 404).
fn proposta_por_id(vault: &str, id: &str) -> Option<anotadinho_core::proposta::Proposta> {
    anotadinho_ipc::handle_listar_propostas(vault.to_string()).ok()?.into_iter().find(|p| p.id == id)
}

/// Acrescenta a decisão ao registro do vault (ciclo 404).
fn registrar_decisao(
    estado: &mut Estado,
    vault: &str,
    proposta: Option<anotadinho_core::proposta::Proposta>,
    acao: anotadinho_core::decisao::Acao,
    motivo: String,
) {
    let Some(p) = proposta else { return };
    let decisao = anotadinho_core::decisao::Decisao {
        quando: agora_local(),
        proposta: p.id,
        alvo: p.alvo,
        autor: p.autor,
        acao,
        motivo,
    };
    if let Err(e) = anotadinho_ipc::handle_registrar_decisao(vault.to_string(), decisao) {
        estado.aviso = Some(format!("não registrou a decisão: {e}"));
    }
}

/// O depois de decidir: aplicada abre a página, o resto recarrega a fila.
fn decidido(estado: &mut Estado, vault: &str, r: Result<String, String>) {
    if let Some(t) = estado.especial.as_mut() {
        t.erro = r.as_ref().err().cloned();
    }
    match r {
        Ok(alvo) if !alvo.is_empty() => {
            if let Ok(p) = handle_list_pages(vault.to_string()) {
                estado.atualizar_paginas(p);
            }
            abrir(estado, vault, &alvo);
        }
        _ => carregar_especial(estado, vault, TipoEspecial::Propostas),
    }
}

/// A conversa que o botão do fluxo abre (ciclo 348), como o
/// `planejar_implementacao` da janela: uma spec aprovada abre "Planejar",
/// uma proposta aprovada "Executar" — na conversa que a gerou, se ela
/// ainda existe —, e em revisão "Alterar". A pergunta vai pro campo, não é
/// enviada: a pessoa anexa o que falta e manda.
fn conversa_do_fluxo(estado: &mut Estado, vault: &str, pagina: &str, alterar: bool) {
    use anotadinho_core::fluxo::{self, Artefato};
    let paginas = handle_scan_vault(vault.to_string()).unwrap_or_default();
    let entrada = paginas.iter().find(|p| p.path == pagina);
    let titulo_da_pagina = entrada
        .map(|p| p.title.clone())
        .filter(|t| !t.trim().is_empty())
        .unwrap_or_else(|| pagina.to_string());
    let e_proposta = pagina.contains("/propostas/") || entrada.is_some_and(|p| p.page_type == "proposta");
    let artefato = if e_proposta { Artefato::Proposta } else { Artefato::Spec };
    let (titulo, pergunta) = if alterar {
        (format!("Alterar: {titulo_da_pagina}"), fluxo::pergunta_de_alteracao(&titulo_da_pagina, artefato))
    } else if e_proposta {
        (format!("Executar: {titulo_da_pagina}"), fluxo::pergunta_de_execucao(&titulo_da_pagina, pagina))
    } else {
        (format!("Planejar: {titulo_da_pagina}"), fluxo::pergunta_de_planejamento(&titulo_da_pagina))
    };
    let continuacao = (e_proposta && !alterar)
        .then(|| anotadinho_ipc::handle_read_page(vault.to_string(), pagina.to_string()).ok())
        .flatten()
        .and_then(|conteudo| {
            let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
            fluxo::origem_da_pagina(corpo)
        })
        .filter(|o| paginas.iter().any(|p| p.path == *o && p.page_type == "conversa"));
    let conversa = match continuacao {
        Some(c) => c,
        None => {
            let carimbo = agora_local();
            let md = anotadinho_core::conversa::montar_pagina(&titulo, Some(pagina), &[pagina.to_string()]);
            let path = format!("pages/conversas/{}.md", anotadinho_core::conversa::nome_de_arquivo(&carimbo));
            if let Err(e) = handle_write_page(vault.to_string(), path.clone(), md) {
                estado.aviso = Some(format!("não criou a conversa: {e}"));
                return;
            }
            if let Ok(p) = handle_list_pages(vault.to_string()) {
                estado.atualizar_paginas(p);
            }
            path
        }
    };
    abrir(estado, vault, &conversa);
    app::conversa::escrever_no_campo(estado, &pergunta);
}

/// O que mudou no disco por fora (ciclo 351): a página aberta, a cada
/// segundo parado, e a lista de páginas e pastas a cada dois — o watcher
/// da janela, por consulta.
fn vigiar_disco(estado: &mut Estado, vault: &str, tambem_a_lista: bool) {
    if tambem_a_lista {
        estado.propostas_pendentes = anotadinho_ipc::handle_listar_propostas(vault.to_string()).map(|l| l.len()).unwrap_or(0);
        estado.mudancas_no_git = anotadinho_ipc::handle_git_status(vault.to_string()).map(|l| l.len());
        if let Ok(p) = handle_list_pages(vault.to_string()) {
            let chave = |l: &[anotadinho_ipc::PageMeta]| l.iter().map(|x| (x.path.clone(), x.title.clone())).collect::<Vec<_>>();
            if chave(&p) != chave(&estado.paginas) {
                estado.pastas_do_vault = anotadinho_ipc::handle_list_folders(vault.to_string()).unwrap_or_default();
                estado.atualizar_paginas(p);
            }
        }
    }
    let Some(caminho) = estado.paginas.get(estado.pagina).map(|p| p.path.clone()) else { return };
    // Com gravação pendente, quem manda é ela (e a trava de versão).
    if estado.gravacao.is_some() || estado.nao_salvo.is_some() || estado.versao.is_none() {
        return;
    }
    if let Ok((texto, versao)) = ler(vault, &caminho) {
        if versao.is_some() && versao != estado.versao && estado.texto_da_pagina.as_deref() != Some(texto.as_str()) {
            estado.recarregar_do_disco(&texto, versao);
        } else if versao != estado.versao {
            estado.versao = versao;
        }
    }
}

/// Os gatilhos em memória (ciclo 412), com o que já se sabe do disco:
/// reler o arquivo a cada volta do laço seria varredura sem motivo.
struct Gatilhos {
    lista: Vec<anotadinho_core::gatilho::Gatilho>,
    /// O minuto da última checagem, pro diário não ser avaliado a cada
    /// 250 ms.
    minuto: String,
    /// Quando as páginas foram fotografadas, pra saber o que mudou.
    visto: std::collections::HashMap<String, std::time::SystemTime>,
}

impl Gatilhos {
    fn novo(vault: &str) -> Self {
        let mut g = Self {
            lista: anotadinho_ipc::handle_ler_gatilhos(vault.to_string()).unwrap_or_default(),
            minuto: String::new(),
            visto: std::collections::HashMap::new(),
        };
        // A primeira foto não dispara nada: o que existia antes de abrir
        // o programa não é mudança.
        g.visto = fotografar(vault);
        g
    }
}

/// A data de modificação de cada `.md` do vault.
fn fotografar(vault: &str) -> std::collections::HashMap<String, std::time::SystemTime> {
    let raiz = std::path::Path::new(vault);
    handle_list_pages(vault.to_string())
        .unwrap_or_default()
        .into_iter()
        .filter_map(|p| {
            let quando = std::fs::metadata(raiz.join(&p.path)).and_then(|m| m.modified()).ok()?;
            Some((p.path, quando))
        })
        .collect()
}

/// As páginas que mudaram desde a última foto, atualizando a foto.
fn mudaram_desde(g: &mut Gatilhos, vault: &str) -> Vec<String> {
    let agora = fotografar(vault);
    let mut mudaram: Vec<String> = agora
        .iter()
        .filter(|(path, quando)| g.visto.get(*path).is_none_or(|antes| antes != *quando))
        .map(|(path, _)| path.clone())
        .collect();
    mudaram.sort();
    g.visto = agora;
    mudaram
}

/// Dispara os gatilhos devidos (ciclo 412): abre uma conversa pro
/// gatilho e manda o prompt pelo mesmo caminho de um envio humano —
/// fila, limite, registro de execução e propostas pra aprovar.
fn disparar_gatilhos(
    estado: &mut Estado,
    vault: &str,
    trabalhos: &mut Trabalhos,
    fila: &mut FilaDaTui,
    g: &mut Gatilhos,
    mudou_algo: bool,
) {
    use anotadinho_core::gatilho::{self, Quando};
    let agora = agora_local();
    let minuto = agora.clone();
    // Sem mudança no disco, só vale reavaliar uma vez por minuto (o
    // diário e as consultas).
    if !mudou_algo && g.minuto == minuto {
        return;
    }
    let mudaram = if mudou_algo { mudaram_desde(g, vault) } else { Vec::new() };
    if g.minuto != minuto {
        g.minuto = minuto;
        // O arquivo pode ter sido editado à mão; relê de minuto em
        // minuto, preservando o `ultimo` que esta sessão já marcou.
        if let Ok(do_disco) = anotadinho_ipc::handle_ler_gatilhos(vault.to_string()) {
            let meus: std::collections::HashMap<String, String> =
                g.lista.iter().map(|x| (x.nome.clone(), x.ultimo.clone())).collect();
            g.lista = do_disco
                .into_iter()
                .map(|mut x| {
                    if let Some(meu) = meus.get(&x.nome) {
                        if meu > &x.ultimo {
                            x.ultimo = meu.clone();
                        }
                    }
                    x
                })
                .collect();
        }
    }
    if g.lista.is_empty() {
        return;
    }
    // O freio do dia (ciclo 434): silêncio, teto de disparos ou de
    // custo. Vale só pro automático — o que a pessoa manda é decisão
    // dela, com ela na frente da tela.
    match anotadinho_ipc::handle_freio_do_dia(vault.to_string(), agora.clone()) {
        Ok(freio) if freio.parou() => {
            // Um aviso por minuto, não por volta do laço — quem acabou
            // de trocar de minuto é quem conta.
            if g.minuto == agora {
                estado.aviso = Some(format!("gatilhos pausados: {}", freio.motivo()));
            }
            return;
        }
        Ok(_) => {}
        Err(e) => estado.aviso = Some(format!("não li as guardas: {e}")),
    }
    // As consultas: quem tem o índice responde se há resultado.
    let com_resultado: Vec<String> = g
        .lista
        .iter()
        .filter_map(|x| match &x.quando {
            Quando::Consulta { de, onde } => {
                let q = anotadinho_core::query::Query {
                    from: Some(de.clone()),
                    conditions: onde
                        .iter()
                        .filter_map(|c| anotadinho_core::query::Condition::parse(c).ok())
                        .collect(),
                    ..Default::default()
                };
                let tem = !q.run(&estado.indice_do_vault).is_empty();
                tem.then(|| x.nome.clone())
            }
            _ => None,
        })
        .collect();

    for i in gatilho::devidos(&g.lista, &agora, &mudaram, &com_resultado) {
        let nome = g.lista[i].nome.clone();
        let prompt = g.lista[i].prompt.clone();
        let regra = g.lista[i].quando.rotulo();
        let path = format!("pages/conversas/{}.md", anotadinho_core::conversa::nome_de_arquivo(&format!("gatilho-{nome}-{agora}")));
        let titulo = format!("Gatilho: {nome}");
        let md = anotadinho_core::conversa::montar_pagina(&titulo, None, &[]);
        if let Err(e) = handle_write_page(vault.to_string(), path.clone(), md) {
            estado.aviso = Some(format!("gatilho {nome}: não criou a conversa: {e}"));
            continue;
        }
        gatilho::marcar(&mut g.lista[i], &agora);
        if let Err(e) = anotadinho_ipc::handle_gravar_gatilhos(vault.to_string(), g.lista.clone()) {
            estado.aviso = Some(format!("gatilho {nome}: não gravou o disparo: {e}"));
        }
        // A conversa nova precisa estar na lista antes do envio, senão
        // `enviar_na_conversa` não acha a página pra reabrir.
        if let Ok(p) = handle_list_pages(vault.to_string()) {
            estado.atualizar_paginas(p);
        }
        let pergunta = format!("{prompt}\n\n(disparado pelo gatilho \"{nome}\": {regra})");
        enviar_na_conversa(estado, vault, vault, trabalhos, fila, &path, &pergunta, &[]);
        estado.aviso = Some(format!("gatilho {nome} disparou"));
    }
}

/// Lê do vault o que a tela de tags, assets ou propostas mostra (ciclo 346).
fn carregar_especial(estado: &mut Estado, vault: &str, tipo: TipoEspecial) {
    let dados = match tipo {
        TipoEspecial::Tags => handle_scan_vault(vault.to_string()).map(|i| especiais::tags_do_indice(&i)),
        TipoEspecial::Grafo => handle_scan_vault(vault.to_string()).map(|i| especiais::grafo_do_indice(&i)),
        TipoEspecial::Assets => anotadinho_ipc::handle_list_assets_info(vault.to_string()).map(|lista| {
            // Um texto só com todas as páginas decide o "usado" de todos.
            let mut paginas = String::new();
            for p in handle_list_pages(vault.to_string()).unwrap_or_default() {
                if let Ok(c) = anotadinho_ipc::handle_read_page(vault.to_string(), p.path) {
                    paginas.push_str(&c);
                    paginas.push('\n');
                }
            }
            especiais::assets_com_uso(lista.into_iter().map(|a| (a.path, a.size)).collect(), &paginas)
        }),
        TipoEspecial::Kanban => Ok(especiais::kanban_da_pagina(estado.texto_da_pagina.as_deref().unwrap_or(""))),
        TipoEspecial::Tarefas => handle_list_pages(vault.to_string()).map(|lista| {
            especiais::tarefas_das_paginas(
                lista
                    .into_iter()
                    .filter_map(|p| {
                        let c = anotadinho_ipc::handle_read_page(vault.to_string(), p.path.clone()).ok()?;
                        Some((p.path, p.title, c))
                    })
                    .collect(),
            )
        }),
        TipoEspecial::Projeto => {
            let caminho = estado.paginas.get(estado.pagina).map(|p| p.path.clone()).unwrap_or_default();
            match anotadinho_core::projeto::projeto_do_caminho(&caminho) {
                Some(slug) => anotadinho_ipc::handle_estado_do_projeto(vault.to_string(), slug.to_string()).map(|e| {
                    let mut p = especiais::ProjetoNaTela::novo(e.projeto, e.config, e.tarefas, e.avisos, e.sprints);
                    p.hoje = agora_local().split(' ').next().unwrap_or("").to_string();
                    especiais::Dados::Projeto(p)
                }),
                None => Err(format!("{caminho} não está dentro de {}", anotadinho_core::projeto::PASTA)),
            }
        }
        TipoEspecial::Propostas => anotadinho_ipc::handle_listar_propostas(vault.to_string()).map(|lista| {
            especiais::Dados::Propostas(
                lista
                    .into_iter()
                    .map(|proposta| especiais::PropostaNaTela {
                        atual: anotadinho_ipc::handle_read_page(vault.to_string(), proposta.alvo.clone()).unwrap_or_default(),
                        proposta,
                    })
                    .collect(),
            )
        }),
    };
    match dados {
        Ok(d) => especiais::carregar(estado, d),
        Err(e) => estado.aviso = Some(format!("não leu: {e}")),
    }
}

/// A busca no conteúdo que a barra de comandos aberta pediu (ciclo 379).
fn buscar_na_paleta(estado: &mut Estado, vault: &str) {
    if let Some(termo) = estado.busca_na_paleta.take() {
        let hits = anotadinho_ipc::handle_search_content(vault.to_string(), termo.clone()).unwrap_or_default();
        app::modais::resultados_na_paleta(estado, &termo, &hits);
    }
    // E a da sidebar (ciclo 370), se a pessoa ainda está nesse termo.
    if let Some(termo) = estado.busca_na_sidebar.take() {
        if estado.busca == termo {
            let hits = anotadinho_ipc::handle_search_content(vault.to_string(), termo.clone()).unwrap_or_default();
            estado.resultados_da_busca = Some((termo, hits));
        }
    }
}

/// Executa o que a TUI pediu e só quem tem o vault pode fazer (ciclo 339).
fn atender(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos, fila: &mut FilaDaTui) {
    let recarregar = |estado: &mut Estado| {
        if let Ok(p) = handle_list_pages(vault.to_string()) {
            estado.atualizar_paginas(p);
        }
    };
    // Um pedido pode deixar outro (abrir a página de tags pede os dados
    // dela): atende até esvaziar, com um teto contra laço.
    for _ in 0..4 {
        if estado.pedidos.is_empty() {
            break;
        }
        for pedido in std::mem::take(&mut estado.pedidos) {
            // Quem PEDE pra abrir algo quer olhar pra ele (ciclo 454). A
            // barra de comandos, as escolhas e os links só faziam o
            // pedido, e o foco ficava onde estava: abrir as propostas
            // pela barra deixava a lista lateral com o foco, e `v` e
            // espaço não faziam nada até um Tab. O Enter da lista lateral
            // já fazia isto por conta própria.
            if matches!(pedido, Pedido::AbrirPagina(_) | Pedido::AbrirEspecial(_)) {
                estado.foco = app::Foco::Conteudo;
            }
            match pedido {
                Pedido::AbrirPagina(caminho) => abrir(estado, vault, &caminho),
                Pedido::CarregarEspecial(tipo) => carregar_especial(estado, vault, tipo),
                // ── Projeto (ciclo 461): a regra é do `ipc`, a mesma da janela.
                Pedido::CriarProjeto(titulo) => {
                    let hoje = agora_local().split(' ').next().unwrap_or("").to_string();
                    match anotadinho_ipc::handle_criar_projeto(vault.to_string(), titulo, None, hoje) {
                        Ok(c) => {
                            recarregar(estado);
                            abrir(estado, vault, &anotadinho_core::projeto::caminho_do_projeto(&c.slug));
                            estado.foco = app::Foco::Conteudo;
                        }
                        Err(e) => estado.aviso = Some(format!("não criou o projeto: {e}")),
                    }
                }
                Pedido::CriarTarefa { slug, titulo } => {
                    let hoje = agora_local().split(' ').next().unwrap_or("").to_string();
                    match anotadinho_ipc::handle_criar_tarefa(vault.to_string(), slug, titulo, hoje) {
                        Ok(c) => {
                            recarregar(estado);
                            estado.aviso = Some(format!("criada: {c}"));
                            carregar_especial(estado, vault, TipoEspecial::Projeto);
                        }
                        Err(e) => estado.aviso = Some(format!("não criou a tarefa: {e}")),
                    }
                }
                Pedido::MoverTarefa { caminho, destino } => {
                    use anotadinho_core::projeto::{self, EtapaTarefa, Tarefa};
                    // Da barra da tarefa (ciclo 502), a página está aberta
                    // e pode ter edição pendente (sem salvamento
                    // automático). Grava ANTES: a etapa muda sobre o texto
                    // do disco, e a página reaberta logo abaixo jogaria a
                    // edição fora.
                    if estado.nao_salvo.as_ref().is_some_and(|(c, _)| *c == caminho) {
                        salvar_pendente(estado, vault);
                    }
                    let antes = anotadinho_ipc::handle_read_page(vault.to_string(), caminho.clone())
                        .ok()
                        .and_then(|t| Tarefa::ler(&caminho, &t));
                    match anotadinho_ipc::handle_mudar_etapa_tarefa(vault.to_string(), caminho.clone(), destino) {
                        Ok(_) => {
                            // Bloquear o que está rodando PARA o agente
                            // aqui, como a janela (460). Se ele roda
                            // noutro app, quem para é aquele.
                            if let Some(t) = antes.filter(|t| t.etapa == EtapaTarefa::Executando && destino == EtapaTarefa::Bloqueada) {
                                let conversa = projeto::caminho_da_conversa_da_tarefa(&t.projeto, t.numero);
                                if let Some(a) = trabalhos.get(&chave_do_trabalho(vault, &conversa)) {
                                    a.trabalho.interromper();
                                }
                            }
                            estado.aviso = Some(format!("tarefa → {}", destino.label()));
                            // Na hora, sem esperar a volta de 5 s: quem
                            // moveu pra `executar` quer ver o agente subir.
                            conferir_projetos(estado, vault, trabalhos, fila, &mut Projetos::default());
                            if estado.especial.as_ref().is_some_and(|x| x.tipo == TipoEspecial::Projeto) {
                                carregar_especial(estado, vault, TipoEspecial::Projeto);
                            } else if estado.paginas.get(estado.pagina).is_some_and(|p| p.path == caminho) {
                                abrir(estado, vault, &caminho);
                            }
                        }
                        // Ir pra feita com `commitar_ao_aceitar` falha quando
                        // o commit falha (ciclo 519): a tarefa fica em revisão,
                        // e a pessoa precisa saber o porquê — o erro do git
                        // tem várias linhas, e o rodapé só uma.
                        Err(e) => estado.aviso = Some(app::especiais::aviso_de_mover(destino, &e)),
                    }
                }
                // ── Sprints (ciclo 473): pela pessoa, como na janela.
                Pedido::CriarSprint { slug, nome, objetivo, inicio, fim } => {
                    let hoje = agora_local().split(' ').next().unwrap_or("").to_string();
                    match anotadinho_ipc::handle_criar_sprint(vault.to_string(), slug, nome, objetivo, inicio, fim, hoje) {
                        Ok(c) => {
                            recarregar(estado);
                            estado.aviso = Some(format!("sprint criada: {c}"));
                            carregar_especial(estado, vault, TipoEspecial::Projeto);
                        }
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::IniciarSprint(caminho) => match anotadinho_ipc::handle_iniciar_sprint(vault.to_string(), caminho) {
                    Ok(()) => {
                        estado.aviso = Some("sprint iniciada".into());
                        carregar_especial(estado, vault, TipoEspecial::Projeto);
                    }
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::EncerrarSprint { caminho, destino } => match anotadinho_ipc::handle_encerrar_sprint(vault.to_string(), caminho, destino) {
                    Ok(r) => {
                        estado.aviso = Some(format!("sprint encerrada: {} feita(s) no histórico, {} pendente(s) movida(s)", r.feitas, r.mudancas.len()));
                        carregar_especial(estado, vault, TipoEspecial::Projeto);
                    }
                    Err(e) => estado.aviso = Some(e),
                },
                // ── Detalhes da tarefa (ciclo 478): os mesmos `handle_*` da janela.
                Pedido::AlternarItemDaTarefa { caminho, indice } => {
                    if let Err(e) = anotadinho_ipc::handle_alternar_item_da_tarefa(vault.to_string(), caminho, indice) {
                        estado.aviso = Some(e);
                    }
                    carregar_especial(estado, vault, TipoEspecial::Projeto);
                }
                Pedido::TagsDaTarefa { caminho, tags } => match anotadinho_ipc::handle_tags_da_tarefa(vault.to_string(), caminho, tags) {
                    Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::PrazoDaTarefa { caminho, prazo } => match anotadinho_ipc::handle_prazo_da_tarefa(vault.to_string(), caminho, prazo) {
                    Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::PrioridadeDaTarefa { caminho, prioridade } => match anotadinho_ipc::handle_prioridade_da_tarefa(vault.to_string(), caminho, prioridade) {
                    Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::DependenciasDaTarefa { caminho, depende } => match anotadinho_ipc::handle_dependencias_da_tarefa(vault.to_string(), caminho, depende) {
                    Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::ComentarTarefa { caminho, texto } => {
                    match anotadinho_ipc::handle_comentar_tarefa(vault.to_string(), caminho, agora_local(), texto) {
                        Ok(()) => {
                            estado.aviso = Some("comentário gravado".into());
                            carregar_especial(estado, vault, TipoEspecial::Projeto);
                        }
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::AnexarNaTarefa { caminho, origem } => {
                    // `~` e relativo ao diretório de onde a TUI foi aberta,
                    // como a pessoa digita num terminal.
                    let origem = sistema::expandir_til(&origem, sistema::ambiente, cfg!(windows));
                    match anotadinho_ipc::handle_anexar_na_tarefa(vault.to_string(), caminho, origem) {
                        Ok(c) => {
                            estado.aviso = Some(format!("anexado: {c}"));
                            carregar_especial(estado, vault, TipoEspecial::Projeto);
                        }
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::AbrirAnexo(caminho) => match anotadinho_ipc::handle_abrir_anexo(vault.to_string(), caminho.clone()) {
                    Ok(()) => estado.aviso = Some(format!("abrindo {caminho}")),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::MudarSprintDaTarefa { caminho, sprint } => {
                    match anotadinho_ipc::handle_mudar_sprint_da_tarefa(vault.to_string(), caminho, sprint) {
                        Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::DetalharTarefa(caminho) => {
                    let texto = anotadinho_ipc::handle_read_page(vault.to_string(), caminho.clone()).unwrap_or_default();
                    match (
                        anotadinho_core::projeto::Tarefa::ler(&caminho, &texto),
                        anotadinho_ipc::handle_conversa_da_tarefa(vault.to_string(), caminho.clone()),
                    ) {
                        (Some(t), Ok(conversa)) => {
                            recarregar(estado);
                            abrir(estado, vault, &conversa);
                            estado.foco = app::Foco::Conteudo;
                            app::conversa::escrever_no_campo(estado, &anotadinho_core::projeto::pergunta_de_detalhar(&t));
                        }
                        (_, Err(e)) => estado.aviso = Some(e),
                        (None, _) => estado.aviso = Some(format!("{caminho} não é uma tarefa")),
                    }
                }
                // A conversa da tarefa sem pergunta (ciclo 502): a mesma
                // página do detalhar, criada pelo `ipc` se ainda não existe.
                Pedido::ConversaDaTarefa(caminho) => match anotadinho_ipc::handle_conversa_da_tarefa(vault.to_string(), caminho) {
                    Ok(conversa) => {
                        recarregar(estado);
                        abrir(estado, vault, &conversa);
                        estado.foco = app::Foco::Conteudo;
                    }
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::GravarConfigDoProjeto { slug, config } => {
                    let repositorio = config.repositorio.clone().map(|r| sistema::expandir_til(&r, sistema::ambiente, cfg!(windows)));
                    if repositorio.as_ref().is_some_and(|r| !std::path::Path::new(r).is_dir()) {
                        // O formulário volta com o que foi escrito (ciclo 519):
                        // a pasta errada não pode custar o comando de
                        // verificação digitado junto.
                        app::modais::abrir_config_do_projeto(estado, &slug, &config);
                        estado.aviso = Some(format!("\"{}\" não é uma pasta", repositorio.unwrap_or_default()));
                    } else {
                        let config = anotadinho_core::projeto::ConfigProjeto { repositorio, ..config };
                        match anotadinho_ipc::handle_gravar_config_do_projeto(vault.to_string(), slug, config) {
                            Ok(()) => carregar_especial(estado, vault, TipoEspecial::Projeto),
                            Err(e) => estado.aviso = Some(e),
                        }
                    }
                }
                // O código da tarefa (ciclo 519): o diff desde a base. O erro
                // do `ipc` (sem repositório, não é tarefa) vai pro aviso.
                Pedido::MudancasDaTarefa(caminho) => {
                    let titulo = anotadinho_ipc::handle_read_page(vault.to_string(), caminho.clone())
                        .ok()
                        .and_then(|t| anotadinho_core::projeto::Tarefa::ler(&caminho, &t))
                        .map(|t| format!("Mudanças — {:03} {}", t.numero, t.titulo))
                        .unwrap_or_else(|| "Mudanças".into());
                    match anotadinho_ipc::handle_mudancas_da_tarefa(vault.to_string(), caminho) {
                        Ok(m) => app::mudancas::abrir(estado, &titulo, &m),
                        Err(e) => estado.aviso = Some(format!("sem as mudanças: {e}")),
                    }
                }
                Pedido::VerificacaoDaTarefa(caminho) => match anotadinho_ipc::handle_read_page(vault.to_string(), caminho.clone()) {
                    Ok(texto) => {
                        let tarefa = anotadinho_core::projeto::Tarefa::ler(&caminho, &texto);
                        match app::especiais::relato_da_verificacao(&texto) {
                            Some(relato) => {
                                let titulo = match &tarefa {
                                    Some(t) => format!("Verificação — {:03} {} ({})", t.numero, t.titulo, t.verificacao.as_deref().unwrap_or("?")),
                                    None => "Verificação".into(),
                                };
                                // Esc volta pros detalhes, no mesmo item, como
                                // o comentário inteiro do 503.
                                let volta = app::especiais::menu_de_detalhes(estado, &caminho).map(|m| match m {
                                    app::Modal::Escolha { titulo, mut lista, acao } => {
                                        lista.selecionado = lista.itens.iter().position(|i| i.chave.starts_with("verificacao\u{0}")).unwrap_or(0);
                                        Box::new(app::Modal::Escolha { titulo, lista, acao })
                                    }
                                    outro => Box::new(outro),
                                });
                                estado.modal = Some(app::Modal::Leitura { titulo, texto: relato, rolagem: 0, volta });
                            }
                            None => estado.aviso = Some("a tarefa ainda não passou por uma verificação".into()),
                        }
                    }
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::ProporTarefas { slug, texto } => {
                    match anotadinho_ipc::handle_propor_tarefas(vault.to_string(), slug, texto, agora_local()) {
                        Ok(ids) => estado.aviso = Some(format!("{} tarefa(s) propostas num lote — revise em Propostas", ids.len())),
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::ProporArquitetura { slug, texto } => {
                    match anotadinho_ipc::handle_propor_arquitetura(vault.to_string(), slug, texto, agora_local()) {
                        Ok(_) => estado.aviso = Some("mudança na arquitetura proposta — revise em Propostas".into()),
                        Err(e) => estado.aviso = Some(e),
                    }
                }
                Pedido::AbrirEspecial(tipo) => {
                    let (path, titulo) = tipo.pagina();
                    let existe = estado.paginas.iter().any(|p| p.path == path);
                    let criada = existe || {
                        let tipo_no_frontmatter = match tipo {
                            TipoEspecial::Tags => "tags",
                            TipoEspecial::Assets => "assets",
                            TipoEspecial::Propostas => "propostas",
                        TipoEspecial::Kanban => "kanban",
                        TipoEspecial::Tarefas => "table",
                        TipoEspecial::Grafo => "graph",
                        TipoEspecial::Projeto => "projeto",
                        };
                        let md = format!("---\ntitle: {titulo}\ntype: {tipo_no_frontmatter}\n---\n");
                        match handle_write_page(vault.to_string(), path.to_string(), md) {
                            Ok(_) => {
                                recarregar(estado);
                                true
                            }
                            Err(e) => {
                                estado.aviso = Some(format!("não criou {path}: {e}"));
                                false
                            }
                        }
                    };
                    if criada {
                        abrir(estado, vault, path);
                    }
                }
                Pedido::ConversaDoFluxo { pagina, alterar } => conversa_do_fluxo(estado, vault, &pagina, alterar),
                Pedido::ListarTemplates => app::modais::escolher_template(
                    estado,
                    anotadinho_ipc::handle_list_templates(vault.to_string())
                        .unwrap_or_default()
                        .into_iter()
                        .map(|t| (t.path, t.title))
                        .collect(),
                ),
                Pedido::PastaDoAgente { pasta, extra } => {
                    // `~` vale como a pasta da pessoa.
                    let pasta = sistema::expandir_til(&pasta, sistema::ambiente, cfg!(windows));
                    if !(pasta.is_empty() && !extra) && !std::path::Path::new(&pasta).is_dir() {
                        estado.aviso = Some(format!("\"{pasta}\" não é uma pasta"));
                    } else {
                        let mut a = estado.preferencias.agente.clone().unwrap_or_default();
                        if extra {
                            if !a.pastas_extras.contains(&pasta) {
                                a.pastas_extras.push(pasta);
                            }
                        } else {
                            a.cwd = pasta;
                        }
                        estado.preferencias.agente = Some(a);
                        if let Err(e) = gravar_preferencias(&estado.preferencias) {
                            estado.aviso = Some(format!("não gravou as preferências: {e}"));
                        }
                    }
                }
                Pedido::AbrirExterno(alvo) => {
                    // URL vai como está; caminho é do vault (`assets/x.png`,
                    // `../assets/x.png` de uma página).
                    let e_url = ["http://", "https://", "mailto:", "file://"].iter().any(|p| alvo.starts_with(p));
                    let destino = if e_url {
                        alvo.clone()
                    } else {
                        let limpo = alvo.trim_start_matches("./").trim_start_matches("../");
                        sistema::caminho_nativo(vault, limpo, cfg!(windows))
                    };
                    if !e_url && !std::path::Path::new(&destino).exists() {
                        estado.aviso = Some(format!("não achei {alvo}"));
                    } else {
                        let programa = if cfg!(target_os = "macos") { "open" } else if cfg!(windows) { "explorer" } else { "xdg-open" };
                        match std::process::Command::new(programa)
                            .arg(&destino)
                            .stdin(std::process::Stdio::null())
                            .stdout(std::process::Stdio::null())
                            .stderr(std::process::Stdio::null())
                            .spawn()
                        {
                            Ok(_) => estado.aviso = Some(format!("abrindo {alvo}")),
                            Err(e) => estado.aviso = Some(format!("não abriu {alvo}: {e}")),
                        }
                    }
                }
                Pedido::TrocarVault { pasta, criar } => {
                    let pasta = sistema::expandir_til(&pasta, sistema::ambiente, cfg!(windows));
                    let tem_paginas = handle_list_pages(pasta.clone()).is_ok_and(|p| !p.is_empty());
                    if criar {
                        match anotadinho_ipc::handle_criar_vault(pasta.clone(), agora_local().split(' ').next().unwrap_or("").to_string()) {
                            Ok(_) => estado.trocar_de_vault = Some((pasta, !tem_paginas)),
                            Err(e) => estado.aviso = Some(format!("não preparou o vault: {e}")),
                        }
                    } else if tem_paginas {
                        estado.trocar_de_vault = Some((pasta, false));
                    } else {
                        estado.aviso = Some(format!("{pasta} não é um vault com páginas — \"Criar vault novo…\" prepara um"));
                    }
                }
                Pedido::GravarPorCima { path, conteudo } => match handle_write_page(vault.to_string(), path.clone(), conteudo) {
                    Ok(_) => {
                        if let Ok((_, v)) = ler(vault, &path) {
                            estado.versao = v;
                        }
                        estado.aviso = Some("gravado por cima do disco".into());
                    }
                    Err(e) => estado.aviso = Some(format!("não gravou: {e}")),
                },
                Pedido::AlternarInicio(path) => {
                    let atual = estado.preferencias.inicio.get(vault).cloned();
                    if atual.as_deref() == Some(path.as_str()) {
                        estado.preferencias.inicio.remove(vault);
                        estado.inicio = None;
                        estado.aviso = Some("não é mais a página de início".into());
                    } else {
                        estado.preferencias.inicio.insert(vault.to_string(), path.clone());
                        estado.inicio = Some(path);
                        estado.organizar_abas();
                        estado.aviso = Some("definida como página de início".into());
                    }
                    if let Err(e) = gravar_preferencias(&estado.preferencias) {
                        estado.aviso = Some(format!("não gravou as preferências: {e}"));
                    }
                }
                Pedido::ExportarHtml(path) => {
                    let feito = anotadinho_ipc::handle_read_page(vault.to_string(), path.clone()).and_then(|conteudo| {
                        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
                        let titulo = estado.paginas.iter().find(|p| p.path == path).map(|p| p.title.clone()).unwrap_or_default();
                        let nome = std::path::Path::new(&path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                        let destino = std::env::current_dir().unwrap_or_default().join(format!("{nome}.html"));
                        std::fs::write(&destino, anotadinho_core::exportar::pagina_em_html(&titulo, corpo))
                            .map(|_| destino)
                            .map_err(|e| e.to_string())
                    });
                    estado.aviso = Some(match feito {
                        Ok(d) => format!("exportado em {}", d.display()),
                        Err(e) => format!("não exportou: {e}"),
                    });
                }
                Pedido::LerPermissoes => match anotadinho_ipc::handle_ler_permissoes(vault.to_string()) {
                    Ok(p) => app::modais::abrir_permissoes(estado, &p),
                    Err(e) => estado.aviso = Some(format!("não leu as permissões: {e}")),
                },
                Pedido::GravarPermissoes(p) => {
                    estado.aviso = Some(match anotadinho_ipc::handle_gravar_permissoes(vault.to_string(), p) {
                        Ok(()) => "permissões do agente gravadas no vault".to_string(),
                        Err(e) => format!("não gravou as permissões: {e}"),
                    });
                }
                // Mais tempo pro agente (ciclo 437): meia vez a mais,
                // gravado — o próximo envio já nasce com ele.
                Pedido::MaisTempoParaOAgente => {
                    let mut a = estado.preferencias.agente.clone().unwrap_or_default().migrado();
                    let antes = a.timeout_s;
                    a.timeout_s = (antes + antes / 2).max(antes + 60);
                    let minutos = a.timeout_s / 60;
                    estado.preferencias.agente = Some(a);
                    estado.pedidos.push(Pedido::GravarPreferencias);
                    estado.aviso = Some(format!("o agente agora tem {minutos} min; mande de novo"));
                }
                // Reexecutar (ciclo 437): a mesma pergunta, no campo.
                Pedido::Reexecutar(conversa) => {
                    match anotadinho_ipc::handle_read_page(vault.to_string(), conversa.clone()) {
                        Ok(texto) => {
                            let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
                            let ultima = anotadinho_core::conversa::parse(corpo)
                                .into_iter()
                                .rev()
                                .find(|m| m.autor == anotadinho_core::conversa::Autor::Voce)
                                .map(|m| m.texto);
                            abrir(estado, vault, &conversa);
                            match ultima {
                                Some(t) => app::conversa::escrever_no_campo(estado, &t),
                                None => estado.aviso = Some("essa conversa não tem pergunta sua".into()),
                            }
                        }
                        Err(e) => estado.aviso = Some(format!("não abriu: {e}")),
                    }
                }
                Pedido::ListarExecucoesDoProjeto { slug, titulo } => match anotadinho_ipc::handle_listar_execucoes(vault.to_string()) {
                    Ok(x) => app::modais::mostrar_execucoes_do_projeto(estado, &x, &slug, &titulo),
                    Err(e) => estado.aviso = Some(e),
                },
                Pedido::ListarExecucoes => match anotadinho_ipc::handle_listar_execucoes(vault.to_string()) {
                    Ok(x) => app::modais::mostrar_execucoes(estado, &x),
                    Err(e) => estado.aviso = Some(format!("não leu as execuções: {e}")),
                },
                Pedido::ListarDecisoes => match anotadinho_ipc::handle_listar_decisoes(vault.to_string()) {
                    Ok(d) => app::modais::mostrar_decisoes(estado, &d),
                    Err(e) => estado.aviso = Some(format!("não leu as decisões: {e}")),
                },
                Pedido::StatusDoGit => app::modais::mostrar_git(
                    estado,
                    anotadinho_ipc::handle_git_status(vault.to_string())
                        .map(|l| l.into_iter().map(|f| (f.status, f.path)).collect()),
                ),
                Pedido::GitPull => match anotadinho_ipc::handle_git_pull(vault.to_string()) {
                    Ok(_) => {
                        recarregar(estado);
                        if let Some(p) = estado.paginas.get(estado.pagina).map(|p| p.path.clone()) {
                            abrir(estado, vault, &p);
                        }
                        estado.aviso = Some("pull feito".into());
                    }
                    Err(e) => estado.aviso = Some(format!("Erro ao dar pull: {}", e.lines().last().unwrap_or(&e))),
                },
                Pedido::GitCommit(mensagem) => match anotadinho_ipc::handle_git_commit_and_push(vault.to_string(), mensagem) {
                    Ok(_) => estado.aviso = Some("commit + push feitos".into()),
                    Err(e) => estado.aviso = Some(format!("Erro ao commitar/dar push: {}", e.lines().last().unwrap_or(&e))),
                },
                Pedido::HistoricoDoGit(path) => app::modais::mostrar_historico(
                    estado,
                    anotadinho_ipc::handle_git_log(vault.to_string(), path)
                        .map(|l| l.into_iter().map(|c| (c.hash, c.date, c.message)).collect()),
                ),
                Pedido::AssetsParaInserir => match anotadinho_ipc::handle_list_assets_info(vault.to_string()) {
                    Ok(lista) => app::markdown::escolher_asset(estado, lista.into_iter().map(|a| a.path).collect()),
                    Err(e) => estado.aviso = Some(format!("não listou os assets: {e}")),
                },
                Pedido::ExcluirAsset(path) => {
                    if let Err(e) = anotadinho_ipc::handle_delete_asset(vault.to_string(), path.clone()) {
                        estado.aviso = Some(format!("Erro ao excluir: {e}"));
                    }
                    carregar_especial(estado, vault, TipoEspecial::Assets);
                }
                // Editada (ciclo 411): grava o texto que a pessoa
                // revisou. O caminho de escrita é o mesmo da aplicação
                // parcial — conteúdo dado de fora, permissões checadas,
                // proposta consumida —, só o registro muda.
                Pedido::AplicarPropostaEditada { id, conteudo } => {
                    let dados = proposta_por_id(vault, &id);
                    let r = anotadinho_ipc::handle_aplicar_proposta_parcial(vault.to_string(), id, conteudo);
                    if r.is_ok() {
                        registrar_decisao(estado, vault, dados, anotadinho_core::decisao::Acao::Editada, String::new());
                    }
                    decidido(estado, vault, r);
                }
                Pedido::AplicarPropostaParcial { id, conteudo, aceitos, de } => {
                    let dados = proposta_por_id(vault, &id);
                    let r = anotadinho_ipc::handle_aplicar_proposta_parcial(vault.to_string(), id, conteudo);
                    if r.is_ok() {
                        registrar_decisao(estado, vault, dados, anotadinho_core::decisao::Acao::Parcial { aceitos, de }, String::new());
                    }
                    decidido(estado, vault, r);
                }
                // Lote atômico (ciclo 420): o IPC aplica todas ou
                // nenhuma; aqui só se registra a decisão de cada uma,
                // pra auditoria continuar por página.
                Pedido::DecidirLote { lote, aplicar, motivo } => {
                    let dados: Vec<anotadinho_core::proposta::Proposta> =
                        anotadinho_ipc::handle_listar_propostas(vault.to_string())
                            .unwrap_or_default()
                            .into_iter()
                            .filter(|p| p.lote.as_deref() == Some(lote.as_str()))
                            .collect();
                    // O lote mexe em VÁRIAS páginas: abrir uma delas
                    // seria escolher arbitrariamente, então a tela fica
                    // onde está e o aviso conta o que entrou.
                    let feito = if aplicar {
                        anotadinho_ipc::handle_aplicar_lote(vault.to_string(), lote.clone()).map(|alvos| {
                            estado.aviso = Some(format!("lote {lote} aplicado: {}", alvos.join(", ")));
                            String::new()
                        })
                    } else {
                        anotadinho_ipc::handle_recusar_lote(vault.to_string(), lote.clone()).map(|n| {
                            estado.aviso = Some(format!("lote {lote} recusado: {n} proposta(s)"));
                            String::new()
                        })
                    };
                    if feito.is_ok() {
                        let acao = if aplicar {
                            anotadinho_core::decisao::Acao::Aplicada
                        } else {
                            anotadinho_core::decisao::Acao::Recusada
                        };
                        for p in dados {
                            registrar_decisao(estado, vault, Some(p), acao.clone(), motivo.clone());
                        }
                    }
                    decidido(estado, vault, feito);
                }
                // Lote (ciclo 409): cada uma segue o mesmo caminho de
                // uma só — inclusive o registro da decisão —, e o que
                // falhar não impede as outras.
                Pedido::DecidirVarias { ids, aplicar, motivo } => {
                    let (mut feitas, mut falhas) = (0usize, Vec::new());
                    for id in ids {
                        let dados = proposta_por_id(vault, &id);
                        let alvo = dados.as_ref().map(|p| p.alvo.clone()).unwrap_or_else(|| id.clone());
                        let r = if aplicar {
                            anotadinho_ipc::handle_aplicar_proposta(vault.to_string(), id.clone())
                        } else {
                            anotadinho_ipc::handle_recusar_proposta(vault.to_string(), id.clone()).map(|_| String::new())
                        };
                        match r {
                            Ok(_) => {
                                let acao = if aplicar {
                                    anotadinho_core::decisao::Acao::Aplicada
                                } else {
                                    anotadinho_core::decisao::Acao::Recusada
                                };
                                registrar_decisao(estado, vault, dados, acao, motivo.clone());
                                feitas += 1;
                            }
                            Err(e) => falhas.push(format!("{alvo}: {e}")),
                        }
                    }
                    if let Some(t) = estado.especial.as_mut() {
                        t.marcadas.clear();
                    }
                    let verbo = if aplicar { "apliquei" } else { "recusei" };
                    estado.aviso = Some(if falhas.is_empty() {
                        format!("{verbo} {feitas}")
                    } else {
                        format!("{verbo} {feitas}; falhou — {}", falhas.join(" · "))
                    });
                    recarregar(estado);
                    carregar_especial(estado, vault, TipoEspecial::Propostas);
                }
                Pedido::DecidirProposta { id, aplicar, motivo } => {
                    let dados = proposta_por_id(vault, &id);
                    let r = if aplicar {
                        anotadinho_ipc::handle_aplicar_proposta(vault.to_string(), id)
                    } else {
                        anotadinho_ipc::handle_recusar_proposta(vault.to_string(), id).map(|_| String::new())
                    };
                    if r.is_ok() {
                        let acao = if aplicar { anotadinho_core::decisao::Acao::Aplicada } else { anotadinho_core::decisao::Acao::Recusada };
                        registrar_decisao(estado, vault, dados, acao, motivo);
                    }
                    decidido(estado, vault, r);
                }
                Pedido::CriarPagina { path, conteudo } => match handle_write_page(vault.to_string(), path.clone(), conteudo) {
                    Ok(()) => {
                        recarregar(estado);
                        abrir(estado, vault, &path);
                    }
                    Err(e) => estado.aviso = Some(format!("não criou: {e}")),
                },
                Pedido::CriarPaginaComTitulo { titulo, tipo } => {
                    match handle_create_page_typed(vault.to_string(), titulo, tipo.unwrap_or_else(|| "md".into())) {
                        Ok(meta) => {
                            recarregar(estado);
                            abrir(estado, vault, &meta.path);
                        }
                        Err(e) => estado.aviso = Some(format!("não criou: {e}")),
                    }
                }
                Pedido::AbrirHoje => match handle_open_today_journal(vault.to_string()) {
                    Ok(meta) => {
                        recarregar(estado);
                        abrir(estado, vault, &meta.path);
                    }
                    Err(e) => estado.aviso = Some(format!("não abriu o diário: {e}")),
                },
                Pedido::ExcluirPagina(caminho) => match handle_delete_page(vault.to_string(), caminho.clone()) {
                    Ok(()) => {
                        recarregar(estado);
                        if let Some(p) = estado.paginas.first().map(|p| p.path.clone()) {
                            abrir(estado, vault, &p);
                        }
                        estado.aviso = Some(format!("{caminho} excluída"));
                    }
                    Err(e) => estado.aviso = Some(format!("não excluiu: {e}")),
                },
                Pedido::EnviarNaConversa { path, pergunta, anexos } => {
                    enviar_na_conversa(estado, vault, vault, trabalhos, fila, &path, &pergunta, &anexos);
                }
                Pedido::CriarDeTemplate { template, titulo, pasta } => {
                    match anotadinho_ipc::handle_create_page_from_template(vault.to_string(), template, titulo, pasta) {
                        Ok(meta) => {
                            recarregar(estado);
                            abrir(estado, vault, &meta.path);
                        }
                        Err(e) => estado.aviso = Some(format!("não criou: {e}")),
                    }
                }
                Pedido::DefinirPropriedade { path, campo, valor } => {
                    let feito = anotadinho_ipc::handle_read_page(vault.to_string(), path.clone())
                        .and_then(|c| anotadinho_core::MarkdownCodec::set_frontmatter_field(&c, &campo, &valor).map_err(|e| e.to_string()))
                        .and_then(|novo| handle_write_page(vault.to_string(), path.clone(), novo));
                    match feito {
                        Ok(()) => {
                            estado.aviso = Some(format!("{campo} de {path} agora é \"{valor}\""));
                            if estado.paginas.get(estado.pagina).is_some_and(|p| p.path == path) {
                                abrir(estado, vault, &path);
                            }
                        }
                        Err(e) => estado.aviso = Some(format!("não gravou {campo}: {e}")),
                    }
                }
                Pedido::BuscarConteudo(termo) => match anotadinho_ipc::handle_search_content(vault.to_string(), termo.clone()) {
                    Ok(hits) => app::modais::mostrar_resultados_da_busca(estado, &termo, &hits),
                    Err(e) => estado.aviso = Some(format!("a busca falhou: {e}")),
                },
                Pedido::CriarPasta(pasta) => match anotadinho_ipc::handle_create_folder(vault.to_string(), pasta.clone()) {
                    Ok(()) => {
                        estado.pastas_do_vault = anotadinho_ipc::handle_list_folders(vault.to_string()).unwrap_or_default();
                        recarregar(estado);
                        estado.aviso = Some(format!("pasta {pasta} criada"));
                    }
                    Err(e) => estado.aviso = Some(format!("não criou a pasta: {e}")),
                },
                Pedido::CriarPaginaNaPasta { pasta, titulo } => {
                    match anotadinho_ipc::handle_create_page_in_folder(vault.to_string(), pasta, titulo, "md".into()) {
                        Ok(meta) => {
                            recarregar(estado);
                            abrir(estado, vault, &meta.path);
                        }
                        Err(e) => estado.aviso = Some(format!("não criou: {e}")),
                    }
                }
                Pedido::MoverPagina { de, para } => match anotadinho_ipc::handle_move_page(vault.to_string(), de, para.clone()) {
                    Ok(meta) => {
                        recarregar(estado);
                        abrir(estado, vault, &meta.path);
                        estado.aviso = Some(format!("movida pra {para}"));
                    }
                    Err(e) => estado.aviso = Some(format!("não moveu: {e}")),
                },
                Pedido::ExportarPasta(pasta) => match anotadinho_ipc::handle_export_folder(vault.to_string(), pasta.clone()) {
                    Ok(texto) => {
                        let nome = if pasta.is_empty() { "vault".to_string() } else { pasta.replace('/', "-") };
                        let destino = std::env::current_dir().unwrap_or_default().join(format!("anotadinho-{nome}.md"));
                        match std::fs::write(&destino, texto) {
                            Ok(()) => estado.aviso = Some(format!("exportado em {}", destino.display())),
                            Err(e) => estado.aviso = Some(format!("não exportou: {e}")),
                        }
                    }
                    Err(e) => estado.aviso = Some(format!("não exportou: {e}")),
                },
                Pedido::CarregarPrompt(path) => match anotadinho_ipc::handle_read_page(vault.to_string(), path.clone()) {
                    Ok(conteudo) => app::conversa::aplicar_prompt(estado, &path, &conteudo),
                    Err(e) => estado.aviso = Some(format!("não consegui ler o prompt: {e}")),
                },
                Pedido::InterromperAgente(path) => {
                    if let Some(a) = trabalhos.get(&chave_do_trabalho(vault, &path)) {
                        a.trabalho.interromper();
                    }
                    // Esperando vaga, interromper é desistir (ciclo 408).
                    if fila.desistir(vault, &path) {
                        estado.aviso = Some("saiu da fila".into());
                        if let Some(c) = estado.conversa.as_mut() {
                            c.na_fila = None;
                        }
                    }
                }
                // Os gatilhos (ciclo 412). Toda escrita relê a lista do
                // disco antes: o arquivo é editável à mão.
                // Cada alvo vira o pedaço que ele pede (ciclo 415). A
                // busca do título é a mesma do wikilink, e a leitura
                // passa pelo `handle_ler_para_contexto`, então uma
                // página transcluída que transclui outra já vem inteira.
                // A página de contexto (ciclo 416): os anexos viram
                // transclusões numa página só, e ela vira O anexo.
                Pedido::ContarDecisoes { desde } => {
                    match anotadinho_ipc::handle_listar_decisoes(vault.to_string()) {
                        Ok(decisoes) => {
                            let texto = anotadinho_core::decisao::resumo_para_agente(&decisoes, &desde);
                            if texto.is_empty() {
                                estado.aviso = Some("nenhuma decisão desde a última mensagem".into());
                            } else {
                                app::conversa::escrever_no_campo(estado, &texto);
                            }
                        }
                        Err(e) => estado.aviso = Some(format!("não leu as decisões: {e}")),
                    }
                }
                // Só o peso, pro cabeçalho (ciclo 417).
                Pedido::PesarContexto(conversa) => {
                    let anexos = estado.conversa.as_ref().map(|c| c.anexos.clone()).unwrap_or_default();
                    // Com sessão aberta, o histórico não vai (ciclo 433).
                    let historico = if sessao_da_conversa(estado, vault, &conversa).is_some() {
                        Vec::new()
                    } else {
                        estado.conversa.as_ref().map(|c| c.mensagens.clone()).unwrap_or_default()
                    };
                    let rascunho = estado.conversa.as_ref().map(|c| c.rascunho.texto.clone()).unwrap_or_default();
                    let envio = montar_envio(
                        vault,
                        &historico,
                        &rascunho,
                        &anexos,
                        &conversa,
                        estado.preferencias.teto_de_contexto,
                    );
                    if let Some(c) = estado.conversa.as_mut() {
                        c.peso = Some((envio.orcamento.resumo(), envio.orcamento.estourou()));
                    }
                }
                // A prévia (ciclo 417): monta pelo MESMO caminho do
                // envio, sem gravar nada nem disparar o agente.
                Pedido::PreviaDoPrompt { conversa, pergunta } => {
                    let anexos = estado.conversa.as_ref().map(|c| c.anexos.clone()).unwrap_or_default();
                    let continua = sessao_da_conversa(estado, vault, &conversa).is_some();
                    let historico = if continua {
                        Vec::new()
                    } else {
                        estado.conversa.as_ref().map(|c| c.mensagens.clone()).unwrap_or_default()
                    };
                    let envio = montar_envio(
                        vault,
                        &historico,
                        &pergunta,
                        &anexos,
                        &conversa,
                        estado.preferencias.teto_de_contexto,
                    );
                    if let Some(c) = estado.conversa.as_mut() {
                        c.peso = Some((envio.orcamento.resumo(), envio.orcamento.estourou()));
                    }
                    app::conversa::mostrar_previa(estado, &envio.prompt, &envio.orcamento, &envio.cortes);
                }
                Pedido::GuardarContexto { titulo, anexos } => {
                    // O título do frontmatter primeiro: é ele que a
                    // pessoa lê e edita no marcador. O nome do arquivo
                    // resolve igual, mas `![[padroes]]` diz menos que
                    // `![[Padrões de nomenclatura]]`.
                    // O marcador usa o título quando ele é único no
                    // conjunto; título repetido cai pro caminho (439).
                    let pares: Vec<(String, String)> = anexos
                        .iter()
                        .map(|a| {
                            let titulo = estado
                                .indice_do_vault
                                .iter()
                                .find(|p| &p.path == a)
                                .map(|p| p.title.clone())
                                .filter(|t| !t.trim().is_empty())
                                .or_else(|| estado.paginas.iter().find(|p| &p.path == a).map(|p| p.title.clone()))
                                .unwrap_or_default();
                            (a.clone(), titulo)
                        })
                        .collect();
                    let corpo: String = anotadinho_core::transclusao::marcadores_de_contexto(&pares)
                        .iter()
                        .map(|m| format!("{m}\n\n"))
                        .collect();
                    let md = format!(
                        "---\ntitle: {titulo}\ntype: contexto\n---\n\nO que o agente precisa saber:\n\n{corpo}"
                    );
                    let path = format!("pages/contextos/{}.md", anotadinho_core::fluxo::slug_de_titulo(&titulo));
                    match handle_write_page(vault.to_string(), path.clone(), md) {
                        Ok(()) => {
                            recarregar(estado);
                            // A conversa passa a ter UM anexo: o recorte.
                            if let Some(c) = estado.conversa.as_mut() {
                                c.anexos = vec![path.clone()];
                                let conversa = c.path.clone();
                                estado.pedidos.push(Pedido::AnexosDaConversa { conversa, lista: vec![path.clone()] });
                            }
                            estado.aviso = Some(format!("contexto guardado em {path}"));
                        }
                        Err(e) => estado.aviso = Some(format!("não guardou o contexto: {e}")),
                    }
                }
                Pedido::ResolverTransclusoes(alvos) => {
                    use anotadinho_core::transclusao::{analisar_alvo, recortar};
                    // A mesma busca do contexto (ciclo 439): caminho,
                    // título ou nome de arquivo.
                    let mut indice: Vec<(String, String)> = estado
                        .indice_do_vault
                        .iter()
                        .map(|p| (p.path.clone(), p.title.clone()))
                        .collect();
                    for p in &estado.paginas {
                        if !indice.iter().any(|(path, _)| path == &p.path) {
                            indice.push((p.path.clone(), p.title.clone()));
                        }
                    }
                    let achar = |titulo: &str| anotadinho_core::transclusao::achar_pagina(titulo, &indice);
                    let resolvidas = alvos
                        .into_iter()
                        .map(|bruto| {
                            let alvo = analisar_alvo(&bruto);
                            let r = match achar(&alvo.titulo) {
                                None => Err(format!("a página {} não existe", alvo.titulo)),
                                Some(path) => anotadinho_ipc::handle_ler_para_contexto(vault.to_string(), path)
                                    .map_err(|e| format!("não consegui ler: {e}"))
                                    .and_then(|x| {
                                        let (_, corpo) =
                                            anotadinho_core::MarkdownCodec::split_frontmatter_text(&x.texto);
                                        recortar(corpo, &alvo)
                                    }),
                            };
                            (bruto, r)
                        })
                        .collect();
                    estado.definir_transclusoes(resolvidas);
                }
                Pedido::LerGuardas => {
                    let guardas = anotadinho_ipc::handle_ler_guardas(vault.to_string()).unwrap_or_default();
                    let freio = anotadinho_ipc::handle_freio_do_dia(vault.to_string(), agora_local())
                        .unwrap_or(anotadinho_core::guardas::Freio::Nenhum);
                    app::modais::abrir_guardas(estado, &guardas, &freio);
                }
                Pedido::GravarGuardas(g) => {
                    match anotadinho_ipc::handle_gravar_guardas(vault.to_string(), g) {
                        Ok(()) => estado.aviso = Some("guardas salvas no vault".into()),
                        Err(e) => estado.aviso = Some(format!("não gravou: {e}")),
                    }
                }
                Pedido::ListarGatilhos => match anotadinho_ipc::handle_ler_gatilhos(vault.to_string()) {
                    Ok(g) => app::modais::mostrar_gatilhos(estado, &g),
                    Err(e) => estado.aviso = Some(format!("não leu os gatilhos: {e}")),
                },
                Pedido::EditarGatilho(nome) => match anotadinho_ipc::handle_ler_gatilhos(vault.to_string()) {
                    Ok(g) => match g.iter().find(|x| x.nome == nome) {
                        Some(x) => app::modais::editar_gatilho(estado, x),
                        None => estado.aviso = Some(format!("gatilho {nome} não existe mais")),
                    },
                    Err(e) => estado.aviso = Some(format!("não leu os gatilhos: {e}")),
                },
                Pedido::AlternarGatilho(nome) => {
                    let mut lista = anotadinho_ipc::handle_ler_gatilhos(vault.to_string()).unwrap_or_default();
                    let mut ligado = None;
                    for g in lista.iter_mut().filter(|g| g.nome == nome) {
                        g.ativo = !g.ativo;
                        ligado = Some(g.ativo);
                    }
                    match anotadinho_ipc::handle_gravar_gatilhos(vault.to_string(), lista.clone()) {
                        Ok(()) => {
                            estado.aviso = Some(match ligado {
                                Some(true) => format!("gatilho {nome} ligado"),
                                Some(false) => format!("gatilho {nome} desligado"),
                                None => format!("gatilho {nome} não existe mais"),
                            });
                            app::modais::mostrar_gatilhos(estado, &lista);
                        }
                        Err(e) => estado.aviso = Some(format!("não gravou: {e}")),
                    }
                }
                Pedido::GravarGatilho(novo) => {
                    let mut lista = anotadinho_ipc::handle_ler_gatilhos(vault.to_string()).unwrap_or_default();
                    // Mesmo nome é o MESMO gatilho: salvar substitui, e o
                    // último disparo sobrevive pra não repetir.
                    match lista.iter_mut().find(|g| g.nome == novo.nome) {
                        Some(antigo) => {
                            let ultimo = antigo.ultimo.clone();
                            *antigo = novo.clone();
                            antigo.ultimo = ultimo;
                        }
                        None => lista.push(novo.clone()),
                    }
                    match anotadinho_ipc::handle_gravar_gatilhos(vault.to_string(), lista) {
                        Ok(()) => estado.aviso = Some(format!("gatilho {} salvo", novo.nome)),
                        Err(e) => estado.aviso = Some(format!("não gravou: {e}")),
                    }
                }
                Pedido::ApagarGatilho(nome) => {
                    let mut lista = anotadinho_ipc::handle_ler_gatilhos(vault.to_string()).unwrap_or_default();
                    let antes = lista.len();
                    lista.retain(|g| g.nome != nome);
                    if lista.len() == antes {
                        estado.aviso = Some(format!("gatilho {nome} não existe mais"));
                    } else {
                        match anotadinho_ipc::handle_gravar_gatilhos(vault.to_string(), lista) {
                            Ok(()) => estado.aviso = Some(format!("gatilho {nome} apagado")),
                            Err(e) => estado.aviso = Some(format!("não apagou: {e}")),
                        }
                    }
                }
                Pedido::VerAgentes => {
                    let rodando: Vec<(String, String, u64)> = trabalhos
                        .iter()
                        .map(|(chave, a)| {
                            let conversa = chave.split_once('\u{0}').map(|(_, p)| p.to_string()).unwrap_or_else(|| chave.clone());
                            (conversa, a.agente.clone(), a.trabalho.segundos())
                        })
                        .collect();
                    let esperando: Vec<String> = fila.esperando().iter().map(|e| e.conversa.clone()).collect();
                    app::modais::mostrar_agentes(estado, &rodando, &esperando);
                }
                Pedido::InterromperTodos => {
                    for a in trabalhos.values() {
                        a.trabalho.interromper();
                    }
                    let quantos = trabalhos.len();
                    let desistiram = fila.limpar();
                    if let Some(c) = estado.conversa.as_mut() {
                        c.na_fila = None;
                    }
                    estado.aviso = Some(format!("interrompi {quantos} e tirei {desistiram} da fila"));
                }
                Pedido::AnexosDaConversa { conversa, lista } => {
                    let feito = anotadinho_ipc::handle_read_page(vault.to_string(), conversa.clone()).and_then(|atual| {
                        handle_write_page(vault.to_string(), conversa.clone(), anotadinho_core::conversa::reescrever_contexto(&atual, &lista))
                    });
                    match feito {
                        Ok(()) => abrir(estado, vault, &conversa),
                        Err(e) => estado.aviso = Some(format!("não gravou os anexos: {e}")),
                    }
                }
                Pedido::ExecutarDaConversa { conversa, texto } => {
                    use anotadinho_core::fluxo::{self, Artefato};
                    let titulo = fluxo::titulo_sugerido(&texto, 60);
                    let hoje = hoje_local();
                    let md = fluxo::montar_pagina(Artefato::Execucao, &titulo, &texto, Some(&conversa), &hoje);
                    let path = format!("{}/{}.md", Artefato::Execucao.pasta(), fluxo::slug_de_titulo(&titulo));
                    if let Err(e) = handle_write_page(vault.to_string(), path.clone(), md) {
                        estado.aviso = Some(format!("não criou a execução: {e}"));
                        continue;
                    }
                    recarregar(estado);
                    let mut anexos = estado.conversa.as_ref().map(|c| c.anexos.clone()).unwrap_or_default();
                    if !anexos.contains(&path) {
                        anexos.push(path.clone());
                    }
                    if let Ok(atual) = anotadinho_ipc::handle_read_page(vault.to_string(), conversa.clone()) {
                        let _ = handle_write_page(
                            vault.to_string(),
                            conversa.clone(),
                            anotadinho_core::conversa::reescrever_contexto(&atual, &anexos),
                        );
                    }
                    let pergunta = fluxo::pergunta_de_execucao_da_conversa(&titulo, &path);
                    enviar_na_conversa(estado, vault, vault, trabalhos, fila, &conversa, &pergunta, &anexos);
                }
                Pedido::GravarPreferencias => {
                    if let Err(e) = gravar_preferencias(&estado.preferencias) {
                        estado.aviso = Some(format!("não gravou as preferências: {e}"));
                    }
                }
            }
        }
    }
}

/// Monta a TUI pra um vault (ciclo 373: também ao trocar de vault).
fn montar_estado(vault: &str, recem_criado: bool, mut preferencias: Preferencias) -> Result<Estado, String> {
    let paginas = handle_list_pages(vault.to_string())?;
    if paginas.is_empty() {
        return Err(format!("o vault {vault} não tem páginas"));
    }
    let vault_absoluto = std::fs::canonicalize(vault).map(|p| p.to_string_lossy().to_string()).unwrap_or_else(|_| vault.to_string());
    if preferencias.ultimo_vault.as_deref() != Some(vault_absoluto.as_str()) {
        preferencias.ultimo_vault = Some(vault_absoluto);
        let _ = gravar_preferencias(&preferencias);
    }
    // Com página de início, ela abre primeiro (ciclo 362); vault
    // recém-preparado abre no guia.
    let guia = recem_criado.then(|| anotadinho_core::semente::PAGINA_INICIAL.to_string());
    let existe = |c: &String| paginas.iter().any(|p| p.path == *c);
    let inicio = preferencias.inicio.get(vault).cloned().filter(existe);
    let primeira_pagina = inicio.clone().or(guia.filter(existe)).unwrap_or_else(|| paginas[0].path.clone());
    let (texto, versao) = ler(vault, &primeira_pagina)?;
    // O índice do vault, varrido uma vez: calendários em modo vault e
    // consultas. Varrer falhando não impede a TUI — eles só ficam vazios.
    let indice = handle_scan_vault(vault.to_string()).unwrap_or_default();
    let primeira = {
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
        anotadinho_core::analise::analisar(corpo)
    };
    let mut estado = Estado::novo(paginas, primeira)
        .com_inicio(inicio)
        .com_pagina(&primeira_pagina)
        .com_texto(&texto, versao.clone())
        .com_preferencias(preferencias)
        .com_pastas(anotadinho_ipc::handle_list_folders(vault.to_string()).unwrap_or_default())
        .com_hoje(&hoje_local())
        .com_eventos_do_vault(anotadinho_core::calendario::entradas_do_vault(&indice))
        .com_indice_do_vault(indice);
    // Abre de novo pelo caminho de sempre: a primeira página pode ser uma
    // conversa, tags ou propostas, que têm tela própria.
    estado.abrir_texto(&texto, versao);
    estado.nome_do_vault = std::fs::canonicalize(vault)
        .ok()
        .and_then(|p| p.file_name().map(|n| n.to_string_lossy().to_string()))
        .or_else(|| Some(vault.to_string()));
    Ok(estado)
}

fn main() -> Result<(), String> {
    let cli = Cli::parse();
    let mut preferencias = ler_preferencias();
    let Some(vault) = cli.vault.clone().or_else(|| preferencias.ultimo_vault.clone()) else {
        return Err("diga qual vault abrir: anotadinho-tui --vault <pasta> (com --criar pra começar um do zero)".into());
    };
    // Pasta vazia ou nova: o "Criar vault novo" da janela (ciclo 372).
    let vazio = handle_list_pages(vault.clone()).map(|p| p.is_empty()).unwrap_or(true);
    if vazio {
        let criar = cli.criar || perguntar_no_terminal(&format!(
            "{vault} ainda não é um vault com páginas. Preparar aqui um vault novo (estrutura, templates, padrões e o guia)? [s/N] "
        ));
        if !criar {
            return Err(format!("o vault {vault} não tem páginas"));
        }
        let criados = anotadinho_ipc::handle_criar_vault(vault.clone(), agora_local().split(' ').next().unwrap_or("").to_string())?;
        eprintln!("vault preparado: {} arquivo(s) criado(s)", criados.len());
    }
    if let Some(tema) = &cli.tema {
        if !anotadinho_tui::tema::TEMAS.contains(&tema.as_str()) {
            return Err(format!(
                "tema \"{}\" não existe — os que existem são: {}",
                tema,
                anotadinho_tui::tema::TEMAS.join(", ")
            ));
        }
        preferencias.tema = tema.clone();
    }
    if !anotadinho_tui::tema::TEMAS.contains(&preferencias.tema.as_str()) {
        preferencias.tema = "mocha".into();
    }
    let mut vault_da_sessao = vault;
    let mut estado = montar_estado(&vault_da_sessao, vazio, preferencias)?;

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
    // O mouse capturado (ciclo 454): sem isto a roda ia pro terminal, que
    // na tela alternativa não rola nada. O preço é conhecido de todo
    // programa de terminal que usa mouse: selecionar texto pede Shift.
    // A colagem em bloco (ciclo 510): o terminal entrega o texto colado
    // inteiro, como `Event::Paste`, e não como teclas — senão o Enter
    // colado age (recuo somado no editor, mensagem enviada na conversa).
    execute!(saida, EnterAlternateScreen, EnableMouseCapture, EnableBracketedPaste).map_err(|e| e.to_string())?;
    let mut term = Terminal::new(CrosstermBackend::new(saida)).map_err(|e| e.to_string())?;

    // Trocar de vault (ciclo 373) monta tudo de novo sem sair do terminal.
    // As execuções do agente vivem fora do laço (ciclo 396): trocar de
    // vault não mata o que está rodando, e a resposta cai na conversa
    // quando ele acaba.
    let mut trabalhos = Trabalhos::new();
    let mut fila = FilaDaTui::nova(estado.preferencias.limite_de_agentes);
    let resultado = loop {
        match laco(&mut term, &mut estado, &vault_da_sessao, &mut trabalhos, &mut fila) {
            Ok(Some((outro, criado))) => match montar_estado(&outro, criado, estado.preferencias.clone()) {
                Ok(novo) => {
                    estado = novo;
                    estado.aviso = Some(format!("vault: {outro}"));
                    vault_da_sessao = outro;
                }
                Err(e) => estado.aviso = Some(format!("não abriu {outro}: {e}")),
            },
            outro => break outro.map(|_| ()),
        }
    };

    disable_raw_mode().map_err(|e| e.to_string())?;
    execute!(term.backend_mut(), DisableBracketedPaste, DisableMouseCapture, LeaveAlternateScreen).map_err(|e| e.to_string())?;
    term.show_cursor().map_err(|e| e.to_string())?;
    resultado
}

fn laco<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    estado: &mut Estado,
    vault: &str,
    trabalhos: &mut Trabalhos,
    fila: &mut FilaDaTui,
) -> Result<Option<(String, bool)>, String> {
    let mut voltas_sem_tecla: u64 = 0;
    // Mudança de .md por fora chega pelo watcher (ciclo 398), como na
    // janela; a consulta periódica fica de rede de segurança (e pro que
    // não é .md: propostas e git).
    let watcher = anotadinho_vault::VaultWatcher::start(std::path::PathBuf::from(vault)).ok();
    let mut gatilhos = Gatilhos::novo(vault);
    let mut projetos = Projetos::default();
    loop {
        estado.agora = Some(agora_local());
        term.draw(|f| app::desenhar(f, estado)).map_err(|e| e.to_string())?;
        if estado.sair {
            return Ok(None);
        }
        if let Some(outro) = estado.trocar_de_vault.take() {
            return Ok(Some(outro));
        }
        // Espera tecla por um instante e redesenha mesmo sem ela: o que
        // roda por fora (o agente de uma conversa) precisa aparecer
        // enquanto a pessoa só olha.
        if !event::poll(std::time::Duration::from_millis(250)).map_err(|e| e.to_string())? {
            app::tique(estado);
            voltas_sem_tecla += 1;
            let avisado = watcher.as_ref().is_some_and(|w| w.has_changes());
            if avisado {
                vigiar_disco(estado, vault, true);
            } else if voltas_sem_tecla % 8 == 0 {
                vigiar_disco(estado, vault, voltas_sem_tecla % 16 == 0);
            }
            // Os gatilhos (ciclo 412): na mudança, na hora, ou quando a
            // consulta passa a ter resposta.
            disparar_gatilhos(estado, vault, trabalhos, fila, &mut gatilhos, avisado);
            // A fila dos projetos (ciclo 459): a cada ~5 s (20 voltas de
            // 250 ms), o mesmo ritmo da janela.
            if voltas_sem_tecla % 20 == 0 {
                conferir_projetos(estado, vault, trabalhos, fila, &mut projetos);
            }
            // A tela do projeto se relê (~3 s): quem move as tarefas pra
            // `executando` e `revisao` é o agente, sem tecla nenhuma.
            if voltas_sem_tecla % 12 == 0 && estado.especial.as_ref().is_some_and(|x| x.tipo == TipoEspecial::Projeto) {
                carregar_especial(estado, vault, TipoEspecial::Projeto);
            }
            acompanhar(estado, vault, trabalhos, fila);
            colher_fechadas(estado, vault);
            atender(estado, vault, trabalhos, fila);
            buscar_na_paleta(estado, vault);
            continue;
        }
        let k = match event::read().map_err(|e| e.to_string())? {
            Event::Key(k) => k,
            Event::Paste(texto) => {
                app::colar(estado, &texto);
                atender(estado, vault, trabalhos, fila);
                continue;
            }
            Event::Mouse(m) => {
                let desce = match m.kind {
                    MouseEventKind::ScrollDown => true,
                    MouseEventKind::ScrollUp => false,
                    // O clique na sidebar (ciclo 467): pasta abre e fecha,
                    // página abre. O resto do clique segue sem efeito.
                    MouseEventKind::Down(crossterm::event::MouseButton::Left) => {
                        if let Some(caminho) = app::clicar(estado, m.column, m.row) {
                            salvar_pendente(estado, vault);
                            if let Ok((texto, versao)) = ler(vault, &caminho) {
                                if let Some(i) = estado.paginas.iter().position(|p| p.path == caminho) {
                                    estado.pagina = i;
                                }
                                estado.abrir_texto(&texto, versao);
                            }
                        }
                        atender(estado, vault, trabalhos, fila);
                        continue;
                    }
                    // Arrasto e os outros botões ficam sem efeito.
                    _ => continue,
                };
                app::rolar(estado, desce, m.column);
                atender(estado, vault, trabalhos, fila);
                continue;
            }
            _ => continue,
        };
        // O Windows manda Press E Release; sem esta guarda cada tecla
        // anda dois blocos.
        if k.kind != KeyEventKind::Press {
            continue;
        }
        let Some(nome) = nome_da_tecla(&k) else { continue };
        let destino = app::tecla(estado, &nome);
        if let Some(t) = estado.copiar_pro_sistema.take() {
            copiar_pro_sistema(&t);
        }
        if destino.is_some() || estado.sair || estado.trocar_de_vault.is_some() {
            salvar_pendente(estado, vault);
        }
        if let Some(caminho) = destino {
            // Página que não abre não derruba a sessão: a pessoa
            // continua no que estava.
            if let Ok((texto, versao)) = ler(vault, &caminho) {
                // Quem abriu foi o destino, então `pagina` passa a ser ele
                // (ciclo 456). Antes o Enter da lista já tinha trocado
                // `pagina` ao passar por cima; agora só a abertura troca.
                if let Some(i) = estado.paginas.iter().position(|p| p.path == caminho) {
                    estado.pagina = i;
                }
                estado.abrir_texto(&texto, versao);
            }
        }
        atender(estado, vault, trabalhos, fila);
        acompanhar(estado, vault, trabalhos, fila);
        // Uma edição deixou texto novo: grava com a trava de versão. Se
        // o arquivo mudou por fora, a gravação é recusada, a página volta
        // a ser a do disco e o rodapé diz por quê (ciclo 318).
        // Sem salvamento automático (ciclo 375), a edição fica pendente até
        // o Ctrl+S — ou até sair da página, do vault ou da TUI.
        if let Some(conteudo) = estado.gravacao.take() {
            let caminho = estado.paginas.get(estado.pagina).map(|p| p.path.clone());
            if !estado.preferencias.salvar_automatico && !estado.salvar_agora {
                if let Some(c) = caminho {
                    estado.nao_salvo = Some((c, conteudo));
                }
                continue;
            }
            if std::mem::take(&mut estado.salvar_agora) {
                estado.aviso = Some("salvo".into());
            }
            if let Some(caminho) = caminho {
                estado.nao_salvo = None;
                let conteudo_tentado = conteudo.clone();
                match handle_write_page_checked(vault.to_string(), caminho.clone(), conteudo, estado.versao.clone()) {
                    Ok(v) => estado.versao = Some(v),
                    Err(motivo) => match ler(vault, &caminho) {
                        // Mudou por fora: a pessoa decide (ciclo 363).
                        Ok((disco, _)) if disco != conteudo_tentado => {
                            estado.modal = Some(app::Modal::Conflito(app::modais::Conflito {
                                path: caminho.clone(),
                                meu: conteudo_tentado,
                                disco,
                                opcao: 0,
                                diff: false,
                                rolagem: 0,
                            }));
                        }
                        Ok((texto, versao)) => {
                            estado.abrir_texto(&texto, versao);
                            estado.aviso = Some(format!("não gravou: {motivo}"));
                        }
                        Err(_) => estado.aviso = Some(format!("não gravou: {motivo}")),
                    },
                }
            }
        }
        if std::mem::take(&mut estado.salvar_agora) {
            match estado.nao_salvo.is_some() {
                true => {
                    salvar_pendente(estado, vault);
                    estado.aviso.get_or_insert_with(|| "salvo".into());
                }
                false => estado.aviso = Some("nada pra salvar".into()),
            }
        }
    }
}

/// Grava a edição pendente (salvamento automático desligado, ciclo 375).
fn salvar_pendente(estado: &mut Estado, vault: &str) {
    if let Some((caminho, conteudo)) = estado.nao_salvo.take() {
        let aberta = estado.paginas.get(estado.pagina).is_some_and(|p| p.path == caminho);
        match handle_write_page_checked(vault.to_string(), caminho.clone(), conteudo.clone(), if aberta { estado.versao.clone() } else { None }) {
            Ok(v) => {
                if aberta {
                    estado.versao = Some(v);
                }
            }
            Err(motivo) => {
                // Mudou por fora: a pessoa decide, como na gravação de sempre.
                let disco = ler(vault, &caminho).map(|(t, _)| t).unwrap_or_default();
                estado.modal = Some(app::Modal::Conflito(app::modais::Conflito {
                    path: caminho,
                    meu: conteudo,
                    disco,
                    opcao: 0,
                    diff: false,
                    rolagem: 0,
                }));
                estado.aviso = Some(format!("não gravou: {motivo}"));
            }
        }
    }
}
