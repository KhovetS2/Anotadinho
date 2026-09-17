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
    /// Path do vault. Sem ele, reabre o último (ciclo 372), como a janela.
    #[arg(long)]
    vault: Option<String>,

    /// Prepara a pasta como vault novo — estrutura, templates, padrões,
    /// prompts e o guia — sem perguntar. Nunca sobrescreve nada.
    #[arg(long)]
    criar: bool,

    /// Tema: escuro, papel, contraste ou claro.
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
            _ => None,
        };
    }
    Some(match k.code {
        KeyCode::Char(c) => c.to_string(),
        KeyCode::Enter => "Enter".into(),
        KeyCode::Esc => "Escape".into(),
        KeyCode::Tab => "Tab".into(),
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
    agora_local().split(' ').next().unwrap_or_default().to_string()
}

/// Agora, `AAAA-MM-DD HH:MM`, no fuso da pessoa — o carimbo das conversas
/// (ciclo 339).
fn agora_local() -> String {
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
            let dia = anotadinho_core::date_util::format_date(tm.tm_year + 1900, (tm.tm_mon + 1) as u32, tm.tm_mday as u32);
            return format!("{dia} {:02}:{:02}", tm.tm_hour, tm.tm_min);
        }
    }
    let dia = anotadinho_core::date_util::add_days("1970-01-01", agora.div_euclid(86_400)).unwrap_or_default();
    let s = agora.rem_euclid(86_400);
    format!("{dia} {:02}:{:02}", s / 3600, (s % 3600) / 60)
}

/// Onde as preferências da TUI moram: fora do vault, na pasta de
/// configuração da pessoa.
fn caminho_das_preferencias() -> Option<std::path::PathBuf> {
    let base = std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config")))?;
    Some(base.join("anotadinho").join("tui.json"))
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

/// As execuções do agente, por conversa (ciclo 340). Vivem aqui, fora do
/// estado da tela: sair da conversa não para o agente, e a resposta cai no
/// arquivo quando ele acaba — como o registro de jobs do backend da janela.
type Trabalhos = std::collections::HashMap<String, anotadinho_tui::agente::Trabalho>;

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

/// Grava a pergunta e dispara o agente com o histórico e os anexos.
fn enviar_na_conversa(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos, path: &str, pergunta: &str, anexos: &[String]) {
    use anotadinho_core::conversa::{self, Autor, Mensagem};
    if trabalhos.contains_key(path) {
        estado.aviso = Some("já tem uma execução em andamento nesta conversa".into());
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
    let contextos: Vec<conversa::Contexto> = anexos
        .iter()
        .filter(|a| a.as_str() != path)
        .filter_map(|a| {
            anotadinho_ipc::handle_read_page(vault.to_string(), a.clone())
                .ok()
                .map(|c| conversa::Contexto { nome: a.clone(), conteudo: c })
        })
        .collect();
    let prompt = conversa::montar_prompt(&historico, pergunta, &contextos, app::conversa::HISTORICO_NO_PROMPT);
    let adaptador = estado.preferencias.agente.clone().unwrap_or_default().migrado();
    let cwd = if adaptador.cwd.trim().is_empty() {
        anotadinho_core::agente::raiz_do_projeto(vault, |d| d.join(".git").exists())
    } else {
        adaptador.cwd.clone()
    };
    match anotadinho_tui::agente::Trabalho::iniciar(&adaptador, &prompt, &cwd) {
        Ok(t) => {
            trabalhos.insert(path.to_string(), t);
        }
        Err(e) => {
            if let Some(c) = estado.conversa.as_mut() {
                c.erro = Some(e);
            }
        }
    }
    if estado.paginas.get(estado.pagina).is_some_and(|p| p.path == path) {
        abrir(estado, vault, path);
    }
}

/// A cada volta: mostra o agente rodando na conversa aberta e, quando ele
/// acaba, grava a resposta (ou o erro) e relê a conversa.
fn acompanhar(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos) {
    use anotadinho_core::conversa::{Autor, Mensagem};
    let mut prontos = Vec::new();
    for (path, t) in trabalhos.iter_mut() {
        if let Some(fim) = t.terminou() {
            prontos.push((path.clone(), fim));
        }
    }
    for (path, fim) in prontos {
        trabalhos.remove(&path);
        let erro = match fim {
            Ok(texto) => acrescentar_mensagem(vault, &path, &Mensagem { autor: Autor::Agente, quando: agora_local(), texto })
                .err()
                .map(|e| format!("não gravou a resposta: {e}")),
            Err(e) => Some(e),
        };
        let aberta = estado.paginas.get(estado.pagina).is_some_and(|p| p.path == path);
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
    if let Some(c) = estado.conversa.as_mut() {
        c.trabalho = trabalhos.get(&c.path).map(|t| (t.segundos(), t.parcial()));
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

/// Executa o que a TUI pediu e só quem tem o vault pode fazer (ciclo 339).
fn atender(estado: &mut Estado, vault: &str, trabalhos: &mut Trabalhos) {
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
            match pedido {
                Pedido::AbrirPagina(caminho) => abrir(estado, vault, &caminho),
                Pedido::CarregarEspecial(tipo) => carregar_especial(estado, vault, tipo),
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
                    let pasta = match (pasta.strip_prefix('~'), std::env::var("HOME")) {
                        (Some(resto), Ok(casa)) => format!("{casa}{resto}"),
                        _ => pasta,
                    };
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
                        std::path::Path::new(vault).join(limpo).to_string_lossy().to_string()
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
                    let pasta = match (pasta.strip_prefix('~'), std::env::var("HOME")) {
                        (Some(resto), Ok(casa)) => format!("{casa}{resto}"),
                        _ => pasta,
                    };
                    let tem_paginas = handle_list_pages(pasta.clone()).is_ok_and(|p| !p.is_empty());
                    if criar {
                        match anotadinho_ipc::handle_criar_vault(pasta.clone()) {
                            Ok(_) => estado.trocar_de_vault = Some((pasta, !tem_paginas)),
                            Err(e) => estado.aviso = Some(format!("não preparou o vault: {e}")),
                        }
                    } else if tem_paginas {
                        estado.trocar_de_vault = Some((pasta, false));
                    } else {
                        estado.aviso = Some(format!("{pasta} não é um vault com páginas — \"Criar vault novo…\" prepara um"));
                    }
                }
                Pedido::BuscarNaSidebar(termo) => {
                    // Só vale se a pessoa ainda está nesse termo.
                    if estado.busca == termo {
                        let hits = anotadinho_ipc::handle_search_content(vault.to_string(), termo.clone()).unwrap_or_default();
                        estado.resultados_da_busca = Some((termo, hits));
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
                Pedido::DecidirProposta { id, aplicar } => {
                    let r = if aplicar {
                        anotadinho_ipc::handle_aplicar_proposta(vault.to_string(), id)
                    } else {
                        anotadinho_ipc::handle_recusar_proposta(vault.to_string(), id).map(|_| String::new())
                    };
                    if let Some(t) = estado.especial.as_mut() {
                        t.erro = r.as_ref().err().cloned();
                    }
                    match r {
                        // Aplicada, abre a página, como a janela.
                        Ok(alvo) if !alvo.is_empty() => {
                            recarregar(estado);
                            abrir(estado, vault, &alvo);
                        }
                        _ => carregar_especial(estado, vault, TipoEspecial::Propostas),
                    }
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
                    enviar_na_conversa(estado, vault, trabalhos, &path, &pergunta, &anexos);
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
                    if let Some(t) = trabalhos.get(&path) {
                        t.interromper();
                    }
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
                    enviar_na_conversa(estado, vault, trabalhos, &conversa, &pergunta, &anexos);
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
        let criados = anotadinho_ipc::handle_criar_vault(vault.clone())?;
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
        preferencias.tema = "escuro".into();
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
    execute!(saida, EnterAlternateScreen).map_err(|e| e.to_string())?;
    let mut term = Terminal::new(CrosstermBackend::new(saida)).map_err(|e| e.to_string())?;

    // Trocar de vault (ciclo 373) monta tudo de novo sem sair do terminal.
    let resultado = loop {
        match laco(&mut term, &mut estado, &vault_da_sessao) {
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
    execute!(term.backend_mut(), LeaveAlternateScreen).map_err(|e| e.to_string())?;
    term.show_cursor().map_err(|e| e.to_string())?;
    resultado
}

fn laco<B: ratatui::backend::Backend>(
    term: &mut Terminal<B>,
    estado: &mut Estado,
    vault: &str,
) -> Result<Option<(String, bool)>, String> {
    let mut trabalhos = Trabalhos::new();
    let mut voltas_sem_tecla: u64 = 0;
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
            if voltas_sem_tecla % 4 == 0 {
                vigiar_disco(estado, vault, voltas_sem_tecla % 8 == 0);
            }
            acompanhar(estado, vault, &mut trabalhos);
            atender(estado, vault, &mut trabalhos);
            continue;
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
            return Ok(None);
        }
        let Some(nome) = nome_da_tecla(&k) else { continue };
        let destino = app::tecla(estado, &nome);
        if destino.is_some() || estado.sair || estado.trocar_de_vault.is_some() {
            salvar_pendente(estado, vault);
        }
        if let Some(caminho) = destino {
            // Página que não abre não derruba a sessão: a pessoa
            // continua no que estava.
            if let Ok((texto, versao)) = ler(vault, &caminho) {
                estado.abrir_texto(&texto, versao);
            }
        }
        atender(estado, vault, &mut trabalhos);
        acompanhar(estado, vault, &mut trabalhos);
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
