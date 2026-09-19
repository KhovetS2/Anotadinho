//! Anotadinho Tauri shell.
//!
//! Entry point do Tauri. Conecta o frontend Yew (no WebView) com
//! os comandos IPC expostos pelos crates do workspace.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod grupo_de_processos;

use std::collections::HashMap;
use std::sync::Mutex;

use anotadinho_core::PageIndexEntry;
use anotadinho_ipc::{
    handle_copy_to_assets, handle_create_folder, handle_create_page,
    handle_create_page_from_template, handle_create_page_in_folder, handle_create_page_typed,
    handle_delete_asset, handle_delete_page, handle_export_folder, handle_git_commit_and_push,
    handle_git_log, handle_git_pull, handle_git_status, handle_list_assets,
    handle_list_assets_info, handle_list_folders, handle_list_pages, handle_list_templates,
    handle_move_page, handle_open_today_journal, handle_ping, handle_read_asset_data_url,
    handle_read_page, handle_read_page_versioned, handle_save_image_assets,
    handle_save_pasted_asset, handle_scan_vault, handle_search_content, handle_write_page,
    handle_write_page_checked, AssetInfo, ImageAssetPayload, PageMeta, PingArgs, PingResult,
    VaultInfo, VersionedPage,
};
use anotadinho_vault::{GitFileEntry, GitLogEntry, VaultIo, VaultWatcher};
use tauri_plugin_dialog::DialogExt;

struct AppWatchers(Mutex<HashMap<String, VaultWatcher>>);

/// Execuções do agente em andamento ou recém-terminadas, por conversa.
///
/// A chave é o path da página de conversa: uma execução por conversa,
/// várias conversas em paralelo.
///
/// Este registro é o que permite a pessoa sair da página enquanto o
/// modelo pensa. Antes, a requisição vivia dentro do componente Yew da
/// conversa — navegar pra outra nota desmontava o componente e a
/// resposta caía no vazio, sem erro e sem aviso. Agora o processo é do
/// backend e a tela só consulta o estado, então voltar pra conversa
/// recupera tudo, inclusive o que chegou enquanto ela estava fora.
///
/// É um global, não um `tauri::State`, porque a thread que espera o
/// processo precisa alcançá-lo depois que o comando já retornou — e
/// `State` só vive durante a chamada.
static JOBS: std::sync::LazyLock<std::sync::Arc<Mutex<HashMap<String, Job>>>> =
    std::sync::LazyLock::new(|| std::sync::Arc::new(Mutex::new(HashMap::new())));

/// A fila da janela (ciclo 429). A política é do núcleo, a mesma da TUI
/// (408); a carga é o que ESTA UI precisa pra disparar quando a vez
/// chegar — aqui o prompt já vem montado pela tela.
static FILA: std::sync::LazyLock<Mutex<anotadinho_core::fila::Fila<(anotadinho_core::agente::Adaptador, String)>>> =
    std::sync::LazyLock::new(|| Mutex::new(anotadinho_core::fila::Fila::nova(anotadinho_core::fila::LIMITE_PADRAO)));

/// O que aconteceu com o pedido de envio (ciclo 429): subiu na hora, ou
/// entrou na fila e vai subir quando abrir vaga.
#[derive(serde::Serialize)]
#[serde(tag = "estado", rename_all = "snake_case")]
pub enum StatusEnvio {
    Rodando,
    NaFila { posicao: usize },
}

struct Job {
    /// `None` enquanto roda; `Some` quando terminou (de qualquer jeito).
    /// O resultado FICA aqui até a tela consumir — é o que evita perder
    /// a resposta de quem estava noutra página quando ela chegou.
    fim: Option<anotadinho_core::agente::EstadoJob>,
    /// Saída parcial do agente, alimentada linha a linha enquanto ele
    /// escreve. É o sinal de vida durante uma execução longa.
    parcial: std::sync::Arc<Mutex<String>>,
    inicio: std::time::Instant,
    /// O processo, pra poder matá-lo quando alguém pedir. Vira `None`
    /// assim que ele termina sozinho.
    filho: std::sync::Arc<Mutex<Option<std::process::Child>>>,
    /// Marcado por `cancelar_agente` — o laço de espera olha pra ele.
    cancelado: std::sync::Arc<std::sync::atomic::AtomicBool>,
}

#[tauri::command]
fn check_changes(vault_path: String, state: tauri::State<'_, AppWatchers>) -> Result<bool, String> {
    let mut map = state.0.lock().map_err(|e| e.to_string())?;
    if let Some(watcher) = map.get_mut(&vault_path) {
        return Ok(watcher.has_changes());
    }
    let watcher = VaultWatcher::start(vault_path.clone().into()).map_err(|e| e.to_string())?;
    let changed = watcher.has_changes();
    let _ = map.insert(vault_path, watcher);
    Ok(changed)
}

/// Controles da janela (ciclo 180). Com `decorations: false` a barra do
/// sistema some, então minimizar/maximizar/fechar passam a ser botões do
/// próprio header do Anotadinho.
#[tauri::command]
fn window_minimize(window: tauri::Window) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

/// Alterna maximizado/restaurado e devolve o estado NOVO — o botão
/// precisa saber pra trocar o ícone.
#[tauri::command]
fn window_toggle_maximize(window: tauri::Window) -> Result<bool, String> {
    let maximizada = window.is_maximized().map_err(|e| e.to_string())?;
    if maximizada {
        window.unmaximize().map_err(|e| e.to_string())?;
    } else {
        window.maximize().map_err(|e| e.to_string())?;
    }
    Ok(!maximizada)
}

#[tauri::command]
fn window_close(window: tauri::Window) -> Result<(), String> {
    window.close().map_err(|e| e.to_string())
}

/// Começa a redimensionar a janela pela borda indicada (ciclo 180).
///
/// Sem a decoração do sistema não existe borda de arraste no WM, então
/// as faixas invisíveis do `.window-resize` no frontend chamam isso no
/// `mousedown` e o próprio compositor assume o arraste dali em diante.
#[tauri::command]
fn window_start_resize(window: tauri::Window, direcao: String) -> Result<(), String> {
    use tauri_runtime::ResizeDirection as D;
    let direcao = match direcao.as_str() {
        "n" => D::North,
        "s" => D::South,
        "w" => D::West,
        "e" => D::East,
        "nw" => D::NorthWest,
        "ne" => D::NorthEast,
        "sw" => D::SouthWest,
        "se" => D::SouthEast,
        outro => return Err(format!("direção desconhecida: {outro}")),
    };
    window
        .start_resize_dragging(direcao)
        .map_err(|e| e.to_string())
}

/// Estado inicial do botão de maximizar (a janela pode abrir já
/// maximizada pelo gerenciador de janelas).
#[tauri::command]
fn window_is_maximized(window: tauri::Window) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

#[tauri::command]
fn ping(args: PingArgs) -> PingResult {
    handle_ping(args)
}

#[tauri::command]
fn get_vault_info(path: String) -> Result<VaultInfo, String> {
    let vault = VaultIo::open(&path);
    let name = vault
        .root()
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string());
    Ok(VaultInfo {
        path: path.to_string(),
        name,
    })
}

#[tauri::command]
fn list_pages(vault_path: String) -> Result<Vec<PageMeta>, String> {
    handle_list_pages(vault_path)
}

/// Varredura única do vault: metadados de todas as páginas numa
/// chamada só (ciclo 150), no lugar de `list_pages` + N `read_page`.
#[tauri::command]
fn scan_vault(vault_path: String) -> Result<Vec<PageIndexEntry>, String> {
    handle_scan_vault(vault_path)
}

/// A visão que o MODELO tem dos blocos de uma página (ciclo 272).
#[tauri::command]
fn arvore_da_pagina(
    vault_path: String,
    page_path: String,
) -> Result<Vec<anotadinho_ipc::UnidadeDaPagina>, String> {
    anotadinho_ipc::handle_arvore_da_pagina(vault_path, page_path)
}

#[tauri::command]
fn read_page(vault_path: String, page_path: String) -> Result<String, String> {
    handle_read_page(vault_path, page_path)
}

/// Leitura pra virar contexto de prompt (ciclo 414): com as
/// transclusões trocadas pelo conteúdo, e a lista do que veio junto.
#[tauri::command]
fn ler_para_contexto(
    vault_path: String,
    page_path: String,
) -> Result<anotadinho_ipc::PaginaExpandida, String> {
    anotadinho_ipc::handle_ler_para_contexto(vault_path, page_path)
}

/// Leitura com marca de versão (ciclo 173) — o editor guarda a marca e
/// devolve ela ao salvar, pra escrita concorrente não passar batida.
#[tauri::command]
fn read_page_versioned(vault_path: String, page_path: String) -> Result<VersionedPage, String> {
    handle_read_page_versioned(vault_path, page_path)
}

/// Gravação condicionada à versão lida. Devolve a versão nova.
#[tauri::command]
fn write_page_checked(
    vault_path: String,
    page_path: String,
    content: String,
    expected_version: Option<String>,
) -> Result<String, String> {
    handle_write_page_checked(vault_path, page_path, content, expected_version)
}

#[tauri::command]
fn write_page(vault_path: String, page_path: String, content: String) -> Result<(), String> {
    handle_write_page(vault_path, page_path, content)
}

#[tauri::command]
fn create_page(vault_path: String, title: String) -> Result<PageMeta, String> {
    handle_create_page(vault_path, title)
}

#[tauri::command]
fn create_page_with_type(
    vault_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    handle_create_page_typed(vault_path, title, page_type)
}

#[tauri::command]
fn open_today_journal(vault_path: String) -> Result<PageMeta, String> {
    handle_open_today_journal(vault_path)
}

#[tauri::command]
fn delete_page(vault_path: String, page_path: String) -> Result<(), String> {
    handle_delete_page(vault_path, page_path)
}

#[tauri::command]
fn create_folder(vault_path: String, folder_path: String) -> Result<(), String> {
    handle_create_folder(vault_path, folder_path)
}

#[tauri::command]
fn list_folders(vault_path: String) -> Result<Vec<String>, String> {
    handle_list_folders(vault_path)
}

#[tauri::command]
fn move_page(vault_path: String, from_path: String, to_path: String) -> Result<PageMeta, String> {
    handle_move_page(vault_path, from_path, to_path)
}

#[tauri::command]
fn create_page_in_folder(
    vault_path: String,
    folder_path: String,
    title: String,
    page_type: String,
) -> Result<PageMeta, String> {
    handle_create_page_in_folder(vault_path, folder_path, title, page_type)
}

#[tauri::command]
fn git_status(vault_path: String) -> Option<Vec<GitFileEntry>> {
    handle_git_status(vault_path)
}

#[tauri::command]
fn git_log(vault_path: String, page_path: String) -> Option<Vec<GitLogEntry>> {
    handle_git_log(vault_path, page_path)
}

#[tauri::command]
fn git_pull(vault_path: String) -> Result<String, String> {
    handle_git_pull(vault_path)
}

#[tauri::command]
fn git_commit_and_push(vault_path: String, message: String) -> Result<String, String> {
    handle_git_commit_and_push(vault_path, message)
}

#[tauri::command]
fn export_folder(vault_path: String, folder_path: String) -> Result<String, String> {
    handle_export_folder(vault_path, folder_path)
}

#[tauri::command]
fn list_templates(vault_path: String) -> Result<Vec<PageMeta>, String> {
    handle_list_templates(vault_path)
}

#[tauri::command]
fn create_page_from_template(
    vault_path: String,
    template_path: String,
    title: String,
    folder_path: Option<String>,
) -> Result<PageMeta, String> {
    handle_create_page_from_template(vault_path, template_path, title, folder_path)
}

#[tauri::command]
fn list_assets(vault_path: String) -> Result<Vec<String>, String> {
    handle_list_assets(vault_path)
}

#[tauri::command]
fn copy_to_assets(vault_path: String, source_path: String) -> Result<String, String> {
    handle_copy_to_assets(vault_path, source_path)
}

#[tauri::command]
fn read_asset_data_url(vault_path: String, asset_path: String) -> Result<String, String> {
    handle_read_asset_data_url(vault_path, asset_path)
}

#[tauri::command]
fn save_pasted_asset(
    vault_path: String,
    extension: String,
    base64_data: String,
) -> Result<String, String> {
    handle_save_pasted_asset(vault_path, extension, base64_data)
}

#[tauri::command]
fn save_image_assets(
    vault_path: String,
    images: Vec<ImageAssetPayload>,
) -> Result<Vec<String>, String> {
    handle_save_image_assets(vault_path, images)
}

#[tauri::command]
async fn pick_images(app: tauri::AppHandle) -> Result<Vec<ImageAssetPayload>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Imagens", &["png", "jpg", "jpeg", "gif", "webp", "svg"])
        .pick_files(move |paths| {
            let values = paths
                .unwrap_or_default()
                .into_iter()
                .filter_map(|p| {
                    let path = p.as_path()?;
                    let bytes = std::fs::read(path).ok()?;
                    use base64::Engine;
                    Some(ImageAssetPayload {
                        name: path.file_name()?.to_string_lossy().to_string(),
                        extension: path.extension()?.to_string_lossy().to_ascii_lowercase(),
                        base64_data: base64::engine::general_purpose::STANDARD.encode(bytes),
                    })
                })
                .collect();
            let _ = tx.send(values);
        });
    rx.await.map_err(|e| e.to_string())
}

/// Lê imagens do disco por CAMINHO, pro arrasto vindo do sistema
/// (ciclo 245).
///
/// O WebKitGTK não entrega o arquivo arrastado como `File`: o
/// `dataTransfer` chega com `text/uri-list` e `files` vazio. Não há bytes
/// no navegador pra ler — só o caminho. E o webview não pode abrir
/// `file://` por conta própria, então quem lê é aqui.
///
/// Só imagens, e só o que existe: um caminho que não seja arquivo, ou
/// tenha extensão de outra coisa, é ignorado em silêncio — arrastar uma
/// pasta junto com duas fotos deve inserir as duas fotos.
#[tauri::command]
fn ler_imagens_locais(caminhos: Vec<String>) -> Result<Vec<ImageAssetPayload>, String> {
    use base64::Engine;
    const ACEITAS: &[&str] = &["png", "jpg", "jpeg", "gif", "webp", "svg"];
    let mut saida = Vec::new();
    for caminho in caminhos {
        let path = std::path::Path::new(&caminho);
        let Some(ext) = path.extension().map(|e| e.to_string_lossy().to_ascii_lowercase()) else {
            continue;
        };
        if !ACEITAS.contains(&ext.as_str()) || !path.is_file() {
            continue;
        }
        let Ok(bytes) = std::fs::read(path) else { continue };
        let Some(nome) = path.file_name() else { continue };
        saida.push(ImageAssetPayload {
            name: nome.to_string_lossy().to_string(),
            extension: ext,
            base64_data: base64::engine::general_purpose::STANDARD.encode(bytes),
        });
    }
    Ok(saida)
}

#[tauri::command]
fn list_assets_info(vault_path: String) -> Result<Vec<AssetInfo>, String> {
    handle_list_assets_info(vault_path)
}

#[tauri::command]
fn delete_asset(vault_path: String, asset_path: String) -> Result<(), String> {
    handle_delete_asset(vault_path, asset_path)
}

/// Grava uma proposta de escrita pra revisão (ciclo 204).
#[tauri::command]
fn propor(
    vault_path: String,
    proposta: anotadinho_core::proposta::Proposta,
) -> Result<String, String> {
    anotadinho_ipc::handle_propor(vault_path, proposta)
}

/// Propostas pendentes de revisão.
#[tauri::command]
fn listar_propostas(
    vault_path: String,
) -> Result<Vec<anotadinho_core::proposta::Proposta>, String> {
    anotadinho_ipc::handle_listar_propostas(vault_path)
}

/// Aplica uma proposta aprovada.
#[tauri::command]
fn aplicar_proposta(vault_path: String, id: String) -> Result<String, String> {
    anotadinho_ipc::handle_aplicar_proposta(vault_path, id)
}

/// Descarta uma proposta.
/// Aplica a proposta com o conteúdo que a pessoa escolheu — trecho a
/// trecho (ciclo 404) ou editado à mão (411). A janela ganhou os dois no
/// ciclo 424.
#[tauri::command]
fn aplicar_proposta_parcial(
    vault_path: String,
    id: String,
    conteudo: String,
) -> Result<String, String> {
    anotadinho_ipc::handle_aplicar_proposta_parcial(vault_path, id, conteudo)
}

/// Aplica um LOTE de propostas: todas ou nenhuma (ciclo 420). A janela
/// ganhou no 428.
#[tauri::command]
fn aplicar_lote(vault_path: String, lote: String) -> Result<Vec<String>, String> {
    anotadinho_ipc::handle_aplicar_lote(vault_path, lote)
}

/// Descarta um lote inteiro sem aplicar (ciclo 420).
#[tauri::command]
fn recusar_lote(vault_path: String, lote: String) -> Result<usize, String> {
    anotadinho_ipc::handle_recusar_lote(vault_path, lote)
}

/// Os gatilhos do vault (ciclo 412); a janela ganhou no 431.
#[tauri::command]
fn ler_gatilhos(vault_path: String) -> Result<Vec<anotadinho_core::gatilho::Gatilho>, String> {
    anotadinho_ipc::handle_ler_gatilhos(vault_path)
}

#[tauri::command]
fn gravar_gatilhos(
    vault_path: String,
    gatilhos: Vec<anotadinho_core::gatilho::Gatilho>,
) -> Result<(), String> {
    anotadinho_ipc::handle_gravar_gatilhos(vault_path, gatilhos)
}

/// A foto das datas de modificação por vault, pra saber o que mudou
/// entre duas avaliações de gatilho (ciclo 431).
static VISTO: std::sync::LazyLock<Mutex<HashMap<String, HashMap<String, std::time::SystemTime>>>> =
    std::sync::LazyLock::new(|| Mutex::new(HashMap::new()));

/// Avalia os gatilhos e dispara os devidos (ciclo 431).
///
/// Quem chama é a tela, de minuto em minuto — a janela não tem laço
/// próprio como a TUI. Devolve os nomes que dispararam, pra mostrar.
///
/// A primeira avaliação de um vault só FOTOGRAFA: o que existia antes de
/// abrir o app não é mudança.
#[tauri::command]
fn avaliar_gatilhos(
    vault_path: String,
    adaptador: anotadinho_core::agente::Adaptador,
    teto: usize,
    limite: usize,
) -> Result<Vec<String>, String> {
    use anotadinho_core::gatilho::{self, Quando};
    let mut lista = anotadinho_ipc::handle_ler_gatilhos(vault_path.clone())?;
    if lista.is_empty() {
        return Ok(Vec::new());
    }
    let paginas = anotadinho_ipc::handle_scan_vault(vault_path.clone())?;
    let raiz = std::path::Path::new(&vault_path);
    let agora: HashMap<String, std::time::SystemTime> = paginas
        .iter()
        .filter_map(|p| {
            let q = std::fs::metadata(raiz.join(&p.path)).and_then(|m| m.modified()).ok()?;
            Some((p.path.clone(), q))
        })
        .collect();
    let mudaram: Vec<String> = {
        let mut visto = VISTO.lock().map_err(|_| "registro de mudanças travado".to_string())?;
        let anterior = visto.insert(vault_path.clone(), agora.clone());
        match anterior {
            None => Vec::new(),
            Some(antes) => {
                let mut m: Vec<String> = agora
                    .iter()
                    .filter(|(path, quando)| antes.get(*path).is_none_or(|a| a != *quando))
                    .map(|(path, _)| path.clone())
                    .collect();
                m.sort();
                m
            }
        }
    };
    let com_resultado: Vec<String> = lista
        .iter()
        .filter_map(|g| match &g.quando {
            Quando::Consulta { de, onde } => {
                let q = anotadinho_core::query::Query {
                    from: Some(de.clone()),
                    conditions: onde
                        .iter()
                        .filter_map(|c| anotadinho_core::query::Condition::parse(c).ok())
                        .collect(),
                    ..Default::default()
                };
                (!q.run(&paginas).is_empty()).then(|| g.nome.clone())
            }
            _ => None,
        })
        .collect();

    let quando = chrono::Local::now().format("%Y-%m-%d %H:%M").to_string();
    let devidos = gatilho::devidos(&lista, &quando, &mudaram, &com_resultado);
    let mut dispararam = Vec::new();
    for i in devidos {
        let g = lista[i].clone();
        let path = format!(
            "pages/conversas/{}.md",
            anotadinho_core::conversa::nome_de_arquivo(&format!("gatilho-{}-{quando}", g.nome))
        );
        let md = anotadinho_core::conversa::montar_pagina(&format!("Gatilho: {}", g.nome), None, &[]);
        if anotadinho_ipc::handle_write_page(vault_path.clone(), path.clone(), md).is_err() {
            continue;
        }
        let pergunta = format!(
            "{}\n\n(disparado pelo gatilho \"{}\": {})",
            g.prompt,
            g.nome,
            g.quando.rotulo()
        );
        // A pergunta vai pro arquivo antes do disparo, como no envio
        // humano: se o agente falhar, o pedido não se perde.
        if let Ok(atual) = anotadinho_ipc::handle_read_page(vault_path.clone(), path.clone()) {
            let (fm, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
            let minha = anotadinho_core::conversa::Mensagem {
                autor: anotadinho_core::conversa::Autor::Voce,
                quando: quando.clone(),
                texto: pergunta.clone(),
            };
            let novo_corpo = anotadinho_core::conversa::append(corpo, &minha);
            let novo = if fm.is_empty() { novo_corpo } else { format!("{fm}\n{novo_corpo}") };
            let _ = anotadinho_ipc::handle_write_page(vault_path.clone(), path.clone(), novo);
        }
        let envio = anotadinho_core::envio::montar(&[], &[], &pergunta, 12, teto, &vault_path);
        match iniciar_agente(adaptador.clone(), envio.prompt, vault_path.clone(), path, Some(limite)) {
            Ok(_) => {
                gatilho::marcar(&mut lista[i], &quando);
                dispararam.push(g.nome.clone());
            }
            Err(e) => eprintln!("gatilho {}: {e}", g.nome),
        }
    }
    if !dispararam.is_empty() {
        anotadinho_ipc::handle_gravar_gatilhos(vault_path, lista)?;
    }
    Ok(dispararam)
}

/// As permissões de escrita do agente (ciclo 405), pro painel da janela
/// (ciclo 427).
#[tauri::command]
fn ler_permissoes(vault_path: String) -> Result<anotadinho_core::permissoes::Permissoes, String> {
    anotadinho_ipc::handle_ler_permissoes(vault_path)
}

#[tauri::command]
fn gravar_permissoes(
    vault_path: String,
    permissoes: anotadinho_core::permissoes::Permissoes,
) -> Result<(), String> {
    anotadinho_ipc::handle_gravar_permissoes(vault_path, permissoes)
}

/// O registro de execuções, do mais novo pro mais velho (ciclo 406).
#[tauri::command]
fn listar_execucoes(vault_path: String) -> Result<Vec<anotadinho_core::execucao::Execucao>, String> {
    anotadinho_ipc::handle_listar_execucoes(vault_path)
}

/// O registro de decisões sobre propostas (ciclo 404).
#[tauri::command]
fn listar_decisoes(vault_path: String) -> Result<Vec<anotadinho_core::decisao::Decisao>, String> {
    anotadinho_ipc::handle_listar_decisoes(vault_path)
}

/// Acrescenta uma decisão ao registro (ciclo 404).
#[tauri::command]
fn registrar_decisao(
    vault_path: String,
    decisao: anotadinho_core::decisao::Decisao,
) -> Result<(), String> {
    anotadinho_ipc::handle_registrar_decisao(vault_path, decisao)
}

#[tauri::command]
fn recusar_proposta(vault_path: String, id: String) -> Result<(), String> {
    anotadinho_ipc::handle_recusar_proposta(vault_path, id)
}

/// Dispara o agente configurado e volta na hora (ciclo 213).
///
/// Deliberadamente SEM shell: `Command::new(binario).args(...)`, com o
/// prompt entrando como um argumento. Aspas, quebras de linha e
/// `$(...)` dentro do prompt são texto — não há interpretador no
/// caminho pra transformá-los em comando.
///
/// A configuração vem das preferências do app, nunca do conteúdo de uma
/// página. É a mesma invariante que mantém a lista de ações do embed
/// `actions` fechada.
///
/// Esta chamada NÃO espera o agente terminar: ela registra o trabalho
/// no registro `JOBS` e devolve. Quem quer saber como foi pergunta depois
/// com `estado_agente`. É o que permite navegar pra outra nota no meio
/// de uma execução longa sem perder nem o processo nem a resposta.
#[tauri::command]
fn iniciar_agente(
    adaptador: anotadinho_core::agente::Adaptador,
    prompt: String,
    vault_path: String,
    conversa_path: String,
    limite: Option<usize>,
) -> Result<StatusEnvio, String> {
    if let Some(problema) = adaptador.validar() {
        return Err(format!(
            "configuração do agente inválida: {}",
            problema.mensagem()
        ));
    }

    let rodando = {
        let mapa = JOBS
            .lock()
            .map_err(|_| "registro de execuções travado".to_string())?;
        if let Some(j) = mapa.get(&conversa_path) {
            if j.fim.is_none() {
                return Err("já existe uma execução em andamento nesta conversa".to_string());
            }
        }
        mapa.values().filter(|j| j.fim.is_none()).count()
    };

    // Sem vaga, espera (ciclo 429): cinco conversas mandando ao mesmo
    // tempo subiam cinco processos de modelo, e quem paga por token
    // levava cinco cobranças sem ter pedido isso.
    {
        let mut fila = FILA.lock().map_err(|_| "fila travada".to_string())?;
        if let Some(l) = limite {
            fila.limite = l;
        }
        if fila.ja_espera(&vault_path, &conversa_path) {
            return Err("esta conversa já tem um envio esperando vaga".to_string());
        }
        if !fila.tem_vaga(rodando) {
            let posicao = fila.enfileirar(anotadinho_core::fila::Espera {
                vault: vault_path.clone(),
                conversa: conversa_path.clone(),
                carga: (adaptador, prompt),
            });
            return Ok(StatusEnvio::NaFila { posicao });
        }
    }
    disparar(adaptador, prompt, vault_path, conversa_path)?;
    Ok(StatusEnvio::Rodando)
}

/// Sobe o processo do agente de verdade. Separado de `iniciar_agente`
/// porque a fila também dispara daqui, quando uma vaga abre.
fn disparar(
    adaptador: anotadinho_core::agente::Adaptador,
    prompt: String,
    vault_path: String,
    conversa_path: String,
) -> Result<(), String> {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Liga o MCP do Anotadinho nesta execução (ciclo 426): o agente
    // ganha as ferramentas do vault sem ninguém configurar por fora.
    let config_mcp = {
        let cli = anotadinho_core::agente::cli_ao_lado(std::env::current_exe().ok());
        let json = anotadinho_core::agente::config_mcp(&cli, &vault_path);
        let arquivo = std::path::Path::new(&vault_path).join(".anotadinho/mcp.json");
        if let Some(pai) = arquivo.parent() {
            let _ = std::fs::create_dir_all(pai);
        }
        match std::fs::write(&arquivo, json) {
            Ok(()) => arquivo.to_string_lossy().to_string(),
            // Sem o arquivo o agente continua rodando: o prompt explica
            // o CLI (ciclo 425), que é o plano B.
            Err(_) => String::new(),
        }
    };
    // Sessão contínua (ciclo 433): a conversa guarda o id, e o agente
    // continua de onde parou em vez de reler tudo.
    let sessao = if adaptador.arg_sessao.trim().is_empty() {
        String::new()
    } else {
        anotadinho_ipc::handle_read_page(vault_path.clone(), conversa_path.clone())
            .ok()
            .and_then(|t| {
                let (fm, _) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&t);
                anotadinho_core::conversa::sessao_do_frontmatter(fm)
            })
            .unwrap_or_default()
    };
    let args = adaptador.montar_args_completo(&prompt, &config_mcp, &sessao);
    let vault_conversa = vault_path.clone();
    // Sem `cwd` configurado, o agente trabalha na raiz do PROJETO, não
    // no vault: rodar dentro das notas o deixava sem enxergar o código
    // que a proposta manda mudar — e com escrita justo nas notas, que é
    // o que o fluxo de propostas existe pra proteger.
    let cwd = if adaptador.cwd.trim().is_empty() {
        anotadinho_core::agente::raiz_do_projeto(&vault_path, |d| d.join(".git").exists())
    } else {
        adaptador.cwd.clone()
    };
    let binario = adaptador.binario.clone();
    let limite = adaptador.timeout_s;
    let formato = adaptador.formato;

    let parcial = Arc::new(Mutex::new(String::new()));
    let sessao_aberta: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
    let filho_slot: Arc<Mutex<Option<std::process::Child>>> = Arc::new(Mutex::new(None));
    let cancelado = Arc::new(AtomicBool::new(false));

    {
        let mut mapa = JOBS
            .lock()
            .map_err(|_| "registro de execuções travado".to_string())?;
        mapa.insert(
            conversa_path.clone(),
            Job {
                fim: None,
                parcial: parcial.clone(),
                inicio: std::time::Instant::now(),
                filho: filho_slot.clone(),
                cancelado: cancelado.clone(),
            },
        );
    }

    // O processo vive numa thread própria, não no runtime async: o
    // `std::process` é bloqueante e travaria o executor.
    let registro = JOBS.clone();
    std::thread::spawn(move || {
        use std::io::{BufRead, BufReader};
        use std::process::{Command, Stdio};

        let resultado = (|| -> Result<String, String> {
            // No Windows o nome cru não basta: `claude` e `codex` são
            // shims `.cmd` do npm, e o `CreateProcessW` só resolve
            // `.exe`. Fora do Windows isto devolve o nome intacto e
            // quem procura no `PATH` continua sendo o sistema.
            let executavel = anotadinho_core::agente::executavel_para_spawn(&binario);
            let mut comando = Command::new(&executavel);
            grupo_de_processos::preparar(&mut comando);
            comando
                .args(&args)
                .current_dir(&cwd)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            // Sem isso, o Windows abre uma janela de console pro
            // processo filho a cada disparo — o app é `windows_subsystem
            // = "windows"` (sem console próprio), então herdar um novo é
            // o comportamento padrão do SO pra filho de console (git,
            // claude/codex/opencode).
            #[cfg(windows)]
            {
                use std::os::windows::process::CommandExt;
                comando.creation_flags(0x0800_0000);
            }
            let mut filho = comando
                .spawn()
                .map_err(|e| format!("não consegui executar \"{binario}\": {e}"))?;

            // Lê a saída LINHA A LINHA numa thread separada. Sem isso
            // não haveria nada pra mostrar durante a execução — e numa
            // tarefa de meia hora, uma tela parada é indistinguível de
            // uma tela travada.
            //
            // Também evita o deadlock clássico: um agente que escreve
            // muito enche o buffer do pipe e fica bloqueado esperando
            // alguém ler, enquanto nós esperamos ele terminar.
            let saida = filho.stdout.take();
            let acumulado = parcial.clone();
            let sessao_lida = sessao_aberta.clone();
            let leitor = saida.map(|s| {
                std::thread::spawn(move || {
                    let mut stream = anotadinho_core::agente::LeitorStream::novo();
                    let mut bruto = String::new();
                    for linha in BufReader::new(s).lines().map_while(Result::ok) {
                        match formato {
                            anotadinho_core::agente::FormatoSaida::StreamJson => {
                                stream.linha(&linha);
                                // O painel mostra o PROGRESSO (que
                                // ferramenta está em uso, o que ele
                                // acabou de dizer), não o JSON cru —
                                // que não diz nada pra quem olha.
                                if let Ok(mut p) = acumulado.lock() {
                                    *p = stream.progresso();
                                }
                            }
                            anotadinho_core::agente::FormatoSaida::Texto => {
                                bruto.push_str(&linha);
                                bruto.push('\n');
                                if let Ok(mut p) = acumulado.lock() {
                                    p.push_str(&linha);
                                    p.push('\n');
                                }
                            }
                        }
                    }
                    // A sessão sai junto com a resposta (ciclo 433): é
                    // ela que a próxima pergunta continua.
                    if let (Ok(mut s), Some(id)) = (sessao_lida.lock(), stream.sessao()) {
                        *s = Some(id);
                    }
                    match formato {
                        anotadinho_core::agente::FormatoSaida::StreamJson => stream.resposta(),
                        anotadinho_core::agente::FormatoSaida::Texto => Ok(bruto),
                    }
                })
            });

            // O stderr é lido SEMPRE, em thread própria, por dois
            // motivos: um agente que escreve muito nele encheria o
            // buffer do pipe e ficaria bloqueado esperando alguém ler;
            // e quando a saída vem vazia, o que ele disse no stderr é a
            // única pista do motivo — "terminou sem escrever nada" por
            // si só não ajuda ninguém.
            let erro_thread = filho.stderr.take().map(|mut e| {
                std::thread::spawn(move || {
                    use std::io::Read;
                    let mut s = String::new();
                    let _ = e.read_to_string(&mut s);
                    s
                })
            });

            // O processo passa a viver no slot COMPARTILHADO: é por ele
            // que `cancelar_agente` alcança o `kill`. Guardá-lo só nesta
            // thread deixaria o botão de interromper sem nada pra matar.
            *filho_slot
                .lock()
                .map_err(|_| "registro travado".to_string())? = Some(filho);

            let inicio = std::time::Instant::now();
            let status = loop {
                let mut guarda = filho_slot
                    .lock()
                    .map_err(|_| "registro travado".to_string())?;
                let Some(proc) = guarda.as_mut() else {
                    return Err("__CANCELADO__".to_string());
                };
                if cancelado.load(Ordering::Relaxed) {
                    grupo_de_processos::encerrar_arvore(proc);
                    let _ = proc.wait();
                    return Err("__CANCELADO__".to_string());
                }
                match proc.try_wait() {
                    Ok(Some(s)) => break s,
                    Ok(None) => {
                        if limite > 0 && inicio.elapsed().as_secs() >= limite {
                            grupo_de_processos::encerrar_arvore(proc);
                            let _ = proc.wait();
                            let minutos = limite / 60;
                            return Err(format!(
                                "o agente passou de {minutos} min e foi interrompido"
                            ));
                        }
                        // Solta o cadeado ANTES de dormir: com ele na
                        // mão, `cancelar_agente` ficaria bloqueado e o
                        // botão de interromper não responderia.
                        drop(guarda);
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                    Err(e) => return Err(format!("erro esperando o agente: {e}")),
                }
            };

            let lido = leitor
                .and_then(|h| h.join().ok())
                .unwrap_or_else(|| Err("não consegui ler a saída do agente".to_string()));

            let stderr = erro_thread
                .and_then(|h| h.join().ok())
                .unwrap_or_default()
                .trim()
                .to_string();

            if status.success() {
                // No `stream-json` o próprio agente reporta erro dentro
                // do stream, com código de saída 0 — por isso o erro do
                // leitor vale mesmo quando o processo "deu certo".
                return lido.map(|s| s.trim().to_string()).and_then(|s| {
                    if !s.is_empty() {
                        return Ok(s);
                    }
                    if stderr.is_empty() {
                        Err("o agente terminou sem escrever nada na saída".to_string())
                    } else {
                        Err(format!(
                            "o agente terminou sem resposta. Ele disse: {}",
                            ultimas_linhas(&stderr, 6)
                        ))
                    }
                });
            } else {
                // A ordem aqui é o que decide se a mensagem ajuda ou
                // não. O agente diz o MOTIVO no stream (stdout); o
                // stderr costuma ter só ruído de inicialização.
                //
                // Preferir stderr escondia o motivo real: o Codex
                // avisou "You've hit your usage limit" no stream, e a
                // tela mostrou "Reading additional input from stdin...",
                // que não diz nada a ninguém.
                let detalhe = match lido {
                    // O leitor entendeu o erro que o agente reportou.
                    Err(e) => e,
                    // Saiu com erro mas o stream trouxe texto: é a
                    // melhor pista que existe.
                    Ok(s) if !s.trim().is_empty() => s,
                    // Nada no stream: aí sim o stderr é o que sobrou.
                    Ok(_) if !stderr.is_empty() => stderr,
                    Ok(_) => format!("terminou com código {status}, sem dizer por quê"),
                };
                Err(format!("o agente falhou: {}", ultimas_linhas(&detalhe, 6)))
            }
        })();

        let estado = match resultado {
            Ok(texto) => {
                // Quem GRAVA a resposta é o backend, não a tela.
                //
                // A tela pode não estar lá: a pessoa manda a pergunta e
                // vai trabalhar noutra nota, que é justamente o que
                // este ciclo passou a permitir. Se a gravação
                // dependesse dela, a resposta viveria só na memória até
                // alguém voltar — e sumiria de vez ao fechar o app.
                //
                // Não há dois escritores: a tela grava a PERGUNTA, o
                // backend grava a RESPOSTA, e `iniciar_agente` recusa
                // uma segunda execução na mesma conversa, então as duas
                // escritas nunca se cruzam.
                if let Err(e) = gravar_resposta(&vault_conversa, &conversa_path, &texto) {
                    eprintln!("não consegui gravar a resposta do agente: {e}");
                }
                anotadinho_core::agente::EstadoJob::Concluido { texto }
            }
            Err(e) if e == "__CANCELADO__" => anotadinho_core::agente::EstadoJob::Cancelado,
            Err(erro) => anotadinho_core::agente::EstadoJob::Falhou { erro },
        };
        // A sessão fica na página (ciclo 433); falha limpa o id, e o
        // envio seguinte começa do zero — conserto automático de um id
        // que o agente não reconhece mais.
        {
            let nova = sessao_aberta.lock().ok().and_then(|s| s.clone());
            let manter = matches!(estado, anotadinho_core::agente::EstadoJob::Concluido { .. });
            if let Ok(atual) =
                anotadinho_ipc::handle_read_page(vault_conversa.clone(), conversa_path.clone())
            {
                let atualizado = anotadinho_core::conversa::reescrever_sessao(
                    &atual,
                    if manter { nova.as_deref() } else { None },
                );
                if atualizado != atual {
                    let _ = anotadinho_ipc::handle_write_page(
                        vault_conversa.clone(),
                        conversa_path.clone(),
                        atualizado,
                    );
                }
            }
        }
        if let Ok(mut mapa) = registro.lock() {
            if let Some(j) = mapa.get_mut(&conversa_path) {
                j.fim = Some(estado);
                if let Ok(mut f) = j.filho.lock() {
                    *f = None;
                }
            }
        }
        // A vaga abriu: chama a próxima da fila (ciclo 429).
        chamar_proxima_da_fila();
    });

    Ok(())
}

/// Sobe quem estava esperando, se agora cabe (ciclo 429).
///
/// Chamado por quem termina, não por um relógio: a fila só anda quando
/// uma vaga abre, e assim não há laço rodando à toa.
fn chamar_proxima_da_fila() {
    let rodando = match JOBS.lock() {
        Ok(m) => m.values().filter(|j| j.fim.is_none()).count(),
        Err(_) => return,
    };
    let proxima = match FILA.lock() {
        Ok(mut f) => f.proxima(rodando),
        Err(_) => return,
    };
    let Some(e) = proxima else { return };
    let (adaptador, prompt) = e.carga;
    if let Err(erro) = disparar(adaptador, prompt, e.vault, e.conversa.clone()) {
        // Falhou ao subir: registra como falha naquela conversa, senão a
        // tela fica esperando uma resposta que nunca vem.
        if let Ok(mut mapa) = JOBS.lock() {
            if let Some(j) = mapa.get_mut(&e.conversa) {
                j.fim = Some(anotadinho_core::agente::EstadoJob::Falhou { erro });
            }
        }
    }
}

/// As últimas `n` linhas não vazias — o rabo do erro é o que interessa.
fn ultimas_linhas(texto: &str, n: usize) -> String {
    let linhas: Vec<&str> = texto.lines().filter(|l| !l.trim().is_empty()).collect();
    let inicio = linhas.len().saturating_sub(n);
    linhas[inicio..].join("\n")
}

/// Acrescenta a resposta do agente ao arquivo da conversa.
///
/// Read-modify-write do arquivo inteiro, preservando o frontmatter —
/// é lá que moram `type: conversa` e a lista de páginas anexadas.
fn gravar_resposta(vault_path: &str, conversa_path: &str, texto: &str) -> Result<(), String> {
    // Só ACRESCENTA a uma conversa que já existe; nunca cria arquivo.
    //
    // Em uso real a página sempre existe: ela é criada antes, e a
    // pergunta é gravada nela antes do disparo. Criar aqui só
    // aconteceria com um path que não é conversa nenhuma — foi o que
    // aconteceu com o harness, que apontava pra raiz do repositório e
    // encheu `pages/` de arquivo solto.
    let atual = anotadinho_ipc::handle_read_page(vault_path.to_string(), conversa_path.to_string())
        .map_err(|_| format!("a conversa \"{conversa_path}\" não existe mais"))?;
    let (frontmatter, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
    let mensagem = anotadinho_core::conversa::Mensagem {
        autor: anotadinho_core::conversa::Autor::Agente,
        quando: chrono::Local::now().format("%Y-%m-%d %H:%M").to_string(),
        texto: texto.to_string(),
    };
    let novo_corpo = anotadinho_core::conversa::append(corpo, &mensagem);
    let novo = if frontmatter.is_empty() {
        novo_corpo
    } else {
        format!("{frontmatter}\n{novo_corpo}")
    };
    anotadinho_ipc::handle_write_page(vault_path.to_string(), conversa_path.to_string(), novo)
}

/// Como está a execução desta conversa.
///
/// `None` significa "nunca houve" ou "a tela já consumiu o resultado".
/// Um estado terminal é entregue UMA vez e some do registro: quem
/// perguntou é quem grava a resposta na conversa, então deixá-lo ali
/// faria a mesma resposta ser escrita de novo a cada consulta.
#[tauri::command]
fn estado_agente(conversa_path: String) -> Option<anotadinho_core::agente::EstadoJob> {
    let mut mapa = JOBS.lock().ok()?;
    let job = mapa.get(&conversa_path)?;
    match &job.fim {
        None => {
            let parcial = job.parcial.lock().map(|p| p.clone()).unwrap_or_default();
            Some(anotadinho_core::agente::EstadoJob::Rodando {
                segundos: job.inicio.elapsed().as_secs(),
                parcial,
            })
        }
        Some(_) => mapa.remove(&conversa_path).and_then(|j| j.fim),
    }
}

/// Interrompe a execução desta conversa.
///
/// Existe porque o timeout deixou de ser curto: com meia hora de
/// margem, quem percebe que pediu a coisa errada precisa de um jeito
/// de parar sem esperar o limite.
#[tauri::command]
fn cancelar_agente(conversa_path: String) -> Result<(), String> {
    use std::sync::atomic::Ordering;
    let mapa = JOBS.lock().map_err(|_| "registro travado".to_string())?;
    let job = mapa
        .get(&conversa_path)
        .ok_or_else(|| "não há execução nesta conversa".to_string())?;
    job.cancelado.store(true, Ordering::Relaxed);
    if let Ok(mut f) = job.filho.lock() {
        if let Some(ref mut c) = *f {
            grupo_de_processos::encerrar_arvore(c);
        }
    }
    Ok(())
}

#[tauri::command]
fn search_content(
    vault_path: String,
    query: String,
) -> Result<Vec<anotadinho_core::embed::SearchHit>, String> {
    handle_search_content(vault_path, query)
}

/// Abre o seletor de pasta (ciclo 216).
///
/// A pasta de trabalho do agente é ESCOLHA da pessoa, não dedução do
/// app: adivinhar pela raiz do git só acerta quando as notas moram
/// dentro do repositório. Quem tem o vault num lugar e os repositórios
/// noutro precisa dizer onde é — e é essa escolha que autoriza o agente
/// a escrever lá.
#[tauri::command]
async fn escolher_pasta(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |file_path| {
        let _ = tx.send(file_path.map(|p| p.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn open_vault_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |file_path| {
        let _ = tx.send(file_path.map(|p| p.to_string()));
    });
    rx.await.map_err(|e| e.to_string())
}

/// Prepara uma pasta pra ser um vault (ciclo 233).
#[tauri::command]
fn criar_vault(vault_path: String) -> Result<Vec<String>, String> {
    anotadinho_ipc::handle_criar_vault(vault_path)
}

/// A pasta aberta ainda não tem página nenhuma? (ciclo 233)
#[tauri::command]
fn vault_esta_vazio(vault_path: String) -> Result<bool, String> {
    anotadinho_ipc::handle_vault_esta_vazio(vault_path)
}

/// Contorna um travamento do WebKitGTK com o driver NVIDIA proprietário.
///
/// Sintoma: a janela congela e o processo de renderização fica a 100% de
/// CPU, sem nenhum IPC. Amostrando a pilha do processo travado, ele está
/// sempre no mesmo lugar:
///
/// ```text
/// WebCore::BitmapTexturePool::releaseUnusedTexturesTimerFired()
///   → WebCore::BitmapTexture::~BitmapTexture()
///     → libnvidia-eglcore.so
/// ```
///
/// É o compositor liberando textura de GPU e ficando preso dentro do
/// driver. Não é laço do nosso código: o backend fica em 0% o tempo
/// todo, e não há chamada de IPC nenhuma.
///
/// `WEBKIT_DISABLE_DMABUF_RENDERER=1` é a saída conhecida: tira o
/// caminho de DMABUF entre o WebKit e o driver, que é justamente onde a
/// interação azeda. Custa um pouco de desempenho de composição.
///
/// Só age quando há NVIDIA proprietária carregada, e nunca por cima de
/// uma escolha explícita — quem definiu a variável decidiu, e a decisão
/// é dela.
#[cfg(target_os = "linux")]
fn contornar_travamento_nvidia() {
    if std::env::var_os("WEBKIT_DISABLE_DMABUF_RENDERER").is_some() {
        return;
    }
    let tem_nvidia = std::path::Path::new("/sys/module/nvidia/version").exists()
        || std::env::var("__GLX_VENDOR_LIBRARY_NAME")
            .map(|v| v.eq_ignore_ascii_case("nvidia"))
            .unwrap_or(false);
    if !tem_nvidia {
        return;
    }
    // Antes de qualquer coisa do WebKit subir: ele lê isto ao criar o
    // processo de renderização.
    std::env::set_var("WEBKIT_DISABLE_DMABUF_RENDERER", "1");
}

/// Fora do Linux não existe o travamento nem o `/sys` onde ele se
/// detecta — a variável do WebKit não teria a quem servir.
#[cfg(not(target_os = "linux"))]
fn contornar_travamento_nvidia() {}

fn main() {
    contornar_travamento_nvidia();

    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        // Porta separada por perfil (ciclo 208).
        //
        // Dev e release abrindo a MESMA porta faz a ponte responder pelo
        // app errado quando os dois estão de pé — e os dois ficam de pé
        // com frequência, porque o app está instalado no sistema e o
        // `dev.sh` sobe outro. O sintoma é traiçoeiro: você edita, o dev
        // reconstrói, e a janela na sua frente não muda, porque é a
        // outra. Custou tempo duas vezes antes de ser diagnosticado.
        .plugin(
            tauri_plugin_mcp_bridge::Builder::new()
                .base_port(if cfg!(debug_assertions) { 9223 } else { 9323 })
                .build(),
        )
        .manage(AppWatchers(Mutex::new(HashMap::new())))
        .invoke_handler(tauri::generate_handler![
            ping,
            window_minimize,
            window_toggle_maximize,
            window_close,
            window_is_maximized,
            window_start_resize,
            get_vault_info,
            list_pages,
            scan_vault,
            read_page,
            ler_para_contexto,
            arvore_da_pagina,
            read_page_versioned,
            write_page,
            write_page_checked,
            create_page,
            create_page_with_type,
            open_today_journal,
            delete_page,
            create_folder,
            list_folders,
            move_page,
            create_page_in_folder,
            export_folder,
            git_status,
            git_log,
            git_pull,
            git_commit_and_push,
            list_templates,
            create_page_from_template,
            list_assets,
            copy_to_assets,
            read_asset_data_url,
            save_pasted_asset,
            save_image_assets,
            pick_images,
            ler_imagens_locais,
            list_assets_info,
            delete_asset,
            search_content,
            iniciar_agente,
            estado_agente,
            cancelar_agente,
            propor,
            listar_propostas,
            aplicar_proposta,
            aplicar_proposta_parcial,
            aplicar_lote,
            recusar_lote,
            ler_gatilhos,
            gravar_gatilhos,
            avaliar_gatilhos,
            ler_permissoes,
            gravar_permissoes,
            listar_execucoes,
            listar_decisoes,
            registrar_decisao,
            recusar_proposta,
            check_changes,
            open_vault_dialog,
            criar_vault,
            vault_esta_vazio,
            escolher_pasta
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar Anotadinho");
}
