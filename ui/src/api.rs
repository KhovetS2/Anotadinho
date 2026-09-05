//! Ponte IPC entre o Yew (WASM) e o backend Tauri.
//!
//! Fornece funções async que chamam comandos Tauri via
//! `window.__TAURI_INTERNALS__.invoke()`.

pub use anotadinho_core::PageIndexEntry;
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;

/// Informações de um vault (retornadas pelo comando `get_vault_info`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VaultInfo {
    /// Path absoluto do vault.
    pub path: String,
    /// Nome do diretório.
    pub name: String,
}

/// Metadados de uma página listada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PageMeta {
    /// Path relativo ao vault.
    pub path: String,
    /// Nome do arquivo (sem extensão).
    pub title: String,
    /// Seção (`pages` ou `journals`).
    pub section: String,
}

/// Uma linha de `git status --porcelain` (ciclo 103, somente leitura).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitFileEntry {
    /// Path relativo ao vault.
    pub path: String,
    /// Status resumido: `M`/`A`/`D`/`R`/`??`.
    pub status: String,
}

/// Um commit do histórico de uma página (ciclo 117, somente leitura).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GitLogEntry {
    /// Hash curto do commit.
    pub hash: String,
    /// Data do commit (`YYYY-MM-DD`).
    pub date: String,
    /// Mensagem do commit.
    pub message: String,
}

fn get_invoke_fn() -> Result<js_sys::Function, JsValue> {
    let w = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
    let ipc = js_sys::Reflect::get(&w, &JsValue::from_str("__TAURI_INTERNALS__"))?;
    if ipc.is_undefined() {
        return Err(JsValue::from_str("__TAURI_INTERNALS__ not available"));
    }
    let invoke = js_sys::Reflect::get(&ipc, &JsValue::from_str("invoke"))?;
    invoke.dyn_into::<js_sys::Function>()
}

async fn tauri_invoke(cmd: &str, args: &JsValue) -> Result<JsValue, JsValue> {
    let invoke = get_invoke_fn()?;
    let promise_js = invoke.call2(&JsValue::null(), &JsValue::from_str(cmd), args)?;
    let promise: js_sys::Promise = promise_js.unchecked_into();
    JsFuture::from(promise).await
}

/// Argumentos de uma chamada IPC, montados em cadeia.
///
/// Existe porque as 48 funções deste módulo repetiam o mesmo bloco:
/// criar o objeto, `Reflect::set` de cada argumento com o mesmo
/// `map_err` e converter pra `JsValue`. Eram 46 `Object::new`, 72
/// `Reflect::set` e 94 `map_err` idênticos — e cada função nova copiava
/// o bloco de uma vizinha, o que fez as inconsistências abaixo se
/// espalharem por cópia.
#[derive(Default)]
struct Args(Option<js_sys::Object>);

impl Args {
    fn novo() -> Self {
        Self(Some(js_sys::Object::new()))
    }

    /// Argumento de texto — o caso de longe mais comum.
    fn texto(self, chave: &str, valor: &str) -> Self {
        self.bruto(chave, JsValue::from_str(valor))
    }

    /// Argumento que só entra quando existe.
    ///
    /// A diferença importa: mandar `folderPath: null` não é o mesmo que
    /// não mandar `folderPath`, e o backend distingue os dois.
    fn texto_opcional(self, chave: &str, valor: Option<&str>) -> Self {
        match valor {
            Some(v) => self.texto(chave, v),
            None => self,
        }
    }

    /// Argumento que vira JSON pelo `serde`.
    ///
    /// Falha de serialização derruba o argumento inteiro e a chamada
    /// segue sem ele, em vez de propagar: o backend recusa a chamada
    /// incompleta com a mensagem dele, que é mais útil pra quem lê do
    /// que um erro de serialização vazado da ponte.
    fn serde<T: Serialize + ?Sized>(self, chave: &str, valor: &T) -> Self {
        match serde_wasm_bindgen::to_value(valor) {
            Ok(v) => self.bruto(chave, v),
            Err(_) => self,
        }
    }

    fn bruto(self, chave: &str, valor: JsValue) -> Self {
        if let Some(obj) = &self.0 {
            // `Reflect::set` num objeto simples recém-criado não falha —
            // só falharia num objeto congelado ou num `Proxy`, e não há
            // nenhum dos dois aqui. Por isso o resultado é descartado em
            // vez de virar um `Result` que 48 funções teriam de carregar.
            let _ = js_sys::Reflect::set(obj, &JsValue::from_str(chave), &valor);
        }
        self
    }

    fn valor(self) -> JsValue {
        self.0.map(JsValue::from).unwrap_or_else(|| JsValue::from(js_sys::Object::new()))
    }
}

/// A mensagem de erro que o backend mandou, e não o `Debug` do `JsValue`.
///
/// Um comando Tauri que devolve `Err(String)` chega aqui como um
/// `JsValue` de string. `format!("{:?}", e)` nele produz
/// `JsValue("não consegui gravar: permissão negada")` — com as aspas, o
/// nome do tipo e os escapes — e isso ia PRA TELA. Oito das funções
/// tratavam isso direito e as outras não, então a mesma falha aparecia
/// legível ou como despejo de depuração dependendo de por onde tivesse
/// passado.
fn erro(cmd: &str, e: JsValue) -> String {
    e.as_string()
        .unwrap_or_else(|| format!("{cmd} falhou: {e:?}"))
}

/// Chama um comando e desserializa a resposta.
async fn chamar<T: serde::de::DeserializeOwned>(cmd: &str, args: Args) -> Result<T, String> {
    let r = tauri_invoke(cmd, &args.valor())
        .await
        .map_err(|e| erro(cmd, e))?;
    serde_wasm_bindgen::from_value(r).map_err(|e| format!("{cmd}: resposta ilegível ({e})"))
}

/// Chama um comando que responde com texto.
async fn chamar_texto(cmd: &str, args: Args) -> Result<String, String> {
    let r = tauri_invoke(cmd, &args.valor())
        .await
        .map_err(|e| erro(cmd, e))?;
    r.as_string()
        .ok_or_else(|| format!("{cmd}: a resposta não é texto"))
}

/// Chama um comando que responde com um booleano.
///
/// Resposta ausente ou de outro tipo vira `padrao` em vez de erro: os
/// usos disto perguntam sobre o ESTADO da janela, e uma janela que não
/// sabe dizer se está maximizada não é motivo pra derrubar a interface.
async fn chamar_bool(cmd: &str, args: Args, padrao: bool) -> Result<bool, String> {
    let r = tauri_invoke(cmd, &args.valor())
        .await
        .map_err(|e| erro(cmd, e))?;
    Ok(r.as_bool().unwrap_or(padrao))
}

/// Chama um comando que pode responder com nada.
///
/// `null`/`undefined` viram `None` — é como um diálogo cancelado e uma
/// execução inexistente chegam aqui, e nenhum dos dois é erro.
async fn chamar_opcional<T: serde::de::DeserializeOwned>(
    cmd: &str,
    args: Args,
) -> Result<Option<T>, String> {
    let r = tauri_invoke(cmd, &args.valor())
        .await
        .map_err(|e| erro(cmd, e))?;
    if r.is_null() || r.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(r).map_err(|e| format!("{cmd}: resposta ilegível ({e})"))
}

/// Chama um comando cujo retorno não interessa.
async fn chamar_sem_retorno(cmd: &str, args: Args) -> Result<(), String> {
    tauri_invoke(cmd, &args.valor())
        .await
        .map(|_| ())
        .map_err(|e| erro(cmd, e))
}

/// Abre o dialog nativo de seleção de pasta via comando Tauri.
pub async fn open_folder_dialog() -> Result<Option<String>, String> {
    chamar_opcional("open_vault_dialog", Args::novo()).await
}

/// Obtém informações do vault a partir do path.
pub async fn get_vault_info(path: &str) -> Result<VaultInfo, String> {

    chamar("get_vault_info", Args::novo().texto("path", path)).await
}

/// Lista todas as páginas `.md` do vault.
pub async fn list_pages(vault_path: &str) -> Result<Vec<PageMeta>, String> {

    chamar("list_pages", Args::novo().texto("vaultPath", vault_path)).await
}

/// Varredura única do vault (ciclo 150): metadados de TODAS as páginas
/// numa chamada só — frontmatter, properties `chave:: valor` do corpo,
/// tags e alvos de wikilink.
///
/// Use isto no lugar de `list_pages()` + `read_page()` em laço sempre
/// que a informação necessária for metadado: o laço faz uma travessia
/// WASM↔Tauri por página, carregando o arquivo inteiro em cada uma.
pub async fn scan_vault(vault_path: &str) -> Result<Vec<PageIndexEntry>, String> {

    chamar("scan_vault", Args::novo().texto("vaultPath", vault_path)).await
}

/// Lê o conteúdo bruto de uma página.
pub async fn read_page(vault_path: &str, page_path: &str) -> Result<String, String> {

    chamar_texto("read_page", Args::novo().texto("vaultPath", vault_path).texto("pagePath", page_path)).await
}

/// Controles da janela (ciclo 180) — a barra de título é do próprio
/// app, então minimizar/maximizar/fechar passam por aqui.
pub async fn window_minimize() -> Result<(), String> {
    chamar_sem_retorno("window_minimize", Args::novo()).await
}

/// Alterna maximizado e devolve o estado NOVO.
pub async fn window_toggle_maximize() -> Result<bool, String> {
    chamar_bool("window_toggle_maximize", Args::novo(), false).await
}

/// Fecha a janela.
pub async fn window_close() -> Result<(), String> {
    chamar_sem_retorno("window_close", Args::novo()).await
}

/// Começa a redimensionar pela borda indicada (`n`, `s`, `w`, `e`,
/// `nw`, `ne`, `sw`, `se`).
pub async fn window_start_resize(direcao: &str) -> Result<(), String> {
    chamar_sem_retorno("window_start_resize", Args::novo().texto("direcao", direcao)).await
}

/// Se a janela está maximizada agora.
pub async fn window_is_maximized() -> Result<bool, String> {
    chamar_bool("window_is_maximized", Args::novo(), false).await
}

/// Conteúdo + marca de versão do arquivo (ciclo 173).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VersionedPage {
    /// Markdown cru.
    pub content: String,
    /// Marca de versão (`None` = arquivo não existe).
    pub version: Option<String>,
}

/// Lê a página junto da marca de versão — usar sempre que o conteúdo
/// lido vá ser EDITADO e regravado depois.
pub async fn read_page_versioned(
    vault_path: &str,
    page_path: &str,
) -> Result<VersionedPage, String> {
    chamar(
        "read_page_versioned",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("pagePath", page_path),
    )
    .await
}

/// Grava só se o arquivo ainda estiver na versão `expected_version`.
/// Devolve a versão nova; erro começando com `CONFLITO: ` quando alguém
/// escreveu por fora desde a leitura.
pub async fn write_page_checked(
    vault_path: &str,
    page_path: &str,
    content: &str,
    expected_version: Option<&str>,
) -> Result<String, String> {
    // A mensagem de erro que sobe daqui é lida por quem chama (a trava
    // de esvaziamento do ciclo 248, o prefixo `CONFLITO: ` do 173), e
    // por isso ela precisa chegar limpa. Era esta a única função que
    // tratava isso; hoje é `erro()` que trata, pra todas.
    let result: String = chamar_texto(
        "write_page_checked",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("pagePath", page_path)
            .texto("content", content)
            .texto_opcional("expectedVersion", expected_version),
    )
    .await?;
    Ok(result)
}

/// Prefixo do erro de conflito de escrita — o editor reconhece por ele.
pub const CONFLICT_PREFIX: &str = "CONFLITO: ";

/// Exclui uma página do vault.
pub async fn delete_page(vault_path: &str, page_path: &str) -> Result<(), String> {
    chamar_sem_retorno(
        "delete_page",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("pagePath", page_path),
    )
    .await
}

/// Cria uma pasta (subdiretório) sob `pages/`.
pub async fn create_folder(vault_path: &str, folder_path: &str) -> Result<(), String> {
    chamar_sem_retorno(
        "create_folder",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("folderPath", folder_path),
    )
    .await
}

/// Lista pastas (incluindo vazias) sob `pages/`.
pub async fn list_folders(vault_path: &str) -> Result<Vec<String>, String> {
    chamar("list_folders", Args::novo().texto("vaultPath", vault_path)).await
}

/// Move (renomeia) uma página — usado pra organizar em pastas.
pub async fn move_page(
    vault_path: &str,
    from_path: &str,
    to_path: &str,
) -> Result<PageMeta, String> {
    chamar("move_page", Args::novo().texto("vaultPath", vault_path).texto("fromPath", from_path).texto("toPath", to_path)).await
}

/// Cria página dentro de uma pasta.
pub async fn create_page_in_folder(
    vault_path: &str,
    folder_path: &str,
    title: &str,
    page_type: &str,
) -> Result<PageMeta, String> {
    chamar("create_page_in_folder", Args::novo().texto("vaultPath", vault_path).texto("folderPath", folder_path).texto("title", title).texto("pageType", page_type)).await
}

/// Lista arquivos modificados/não rastreados via `git status
/// --porcelain` (somente leitura). `None` se o vault não for um
/// repositório git ou `git` não estiver instalado — a UI deve tratar
/// isso como "não mostrar indicador", não como erro.
pub async fn git_status(vault_path: &str) -> Result<Option<Vec<GitFileEntry>>, String> {
    chamar("git_status", Args::novo().texto("vaultPath", vault_path)).await
}

/// Histórico de commits que tocaram uma página específica (ciclo 117,
/// somente leitura). `None` nas mesmas condições de `git_status`.
pub async fn git_log(
    vault_path: &str,
    page_path: &str,
) -> Result<Option<Vec<GitLogEntry>>, String> {
    chamar("git_log", Args::novo().texto("vaultPath", vault_path).texto("pagePath", page_path)).await
}

/// `git pull` — ação explícita do usuário (ciclo 119). Retorna a
/// saída do git em sucesso, ou `Err` com a mensagem de erro (conflito,
/// sem remote, etc) tal qual.
pub async fn git_pull(vault_path: &str) -> Result<String, String> {
    chamar_texto("git_pull", Args::novo().texto("vaultPath", vault_path)).await
}

/// `git add -A && commit -m <message> && push` — ação explícita do
/// usuário (ciclo 119).
pub async fn git_commit_and_push(vault_path: &str, message: &str) -> Result<String, String> {
    chamar_texto("git_commit_and_push", Args::novo().texto("vaultPath", vault_path).texto("message", message)).await
}

/// Concatena o markdown fonte de todas as páginas dentro de uma pasta
/// (recursivo) num dump único. `folder_path` vazio exporta o vault
/// inteiro (`pages/` + `journals/`).
pub async fn export_folder(vault_path: &str, folder_path: &str) -> Result<String, String> {
    chamar("export_folder", Args::novo().texto("vaultPath", vault_path).texto("folderPath", folder_path)).await
}

/// Lista templates em `templates/`.
pub async fn list_templates(vault_path: &str) -> Result<Vec<PageMeta>, String> {
    chamar("list_templates", Args::novo().texto("vaultPath", vault_path)).await
}

/// Cria página a partir de um template, substituindo `{{title}}`.
/// `folder_path` escolhe a pasta de destino (`None` = `pages/`) — usado
/// pelo embed de ações (ciclo 156) pra criar spec/decisão já na pasta
/// certa do esquema de agent-os.
pub async fn create_page_from_template(
    vault_path: &str,
    template_path: &str,
    title: &str,
    folder_path: Option<&str>,
) -> Result<PageMeta, String> {
    chamar(
        "create_page_from_template",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("templatePath", template_path)
            .texto("title", title)
            .texto_opcional("folderPath", folder_path),
    )
    .await
}

/// Lista arquivos no diretório assets/.
pub async fn list_assets(vault_path: &str) -> Result<Vec<String>, String> {
    chamar("list_assets", Args::novo().texto("vaultPath", vault_path)).await
}

/// Copia um arquivo para assets/ e retorna o path relativo.
pub async fn copy_to_assets(vault_path: &str, source_path: &str) -> Result<String, String> {
    chamar_texto("copy_to_assets", Args::novo().texto("vaultPath", vault_path).texto("sourcePath", source_path)).await
}

/// Grava bytes (já em base64) em `assets/` com nome único — usado pelo
/// paste de imagem no editor (ciclo 118), sem arquivo de origem no
/// disco. `extension` sem o ponto (ex: `"png"`). Retorna o path
/// relativo do asset criado.
pub async fn save_pasted_asset(
    vault_path: &str,
    extension: &str,
    base64_data: &str,
) -> Result<String, String> {
    chamar_texto("save_pasted_asset", Args::novo().texto("vaultPath", vault_path).texto("extension", extension).texto("base64Data", base64_data)).await
}

/// Arquivo de imagem mantido em memória até a confirmação do modal.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ImageAssetPayload {
    /// Nome original.
    #[serde(default)]
    pub name: String,
    /// Extensão validável.
    pub extension: String,
    /// Bytes em base64.
    pub base64_data: String,
}

/// Abre o seletor nativo e lê as imagens escolhidas sem gravá-las no vault.
pub async fn pick_images() -> Result<Vec<ImageAssetPayload>, String> {
    chamar("pick_images", Args::novo()).await
}

/// Lê imagens do disco por caminho (ciclo 245).
///
/// O arrasto vindo do sistema entrega `text/uri-list`, não `File`: não há
/// bytes no navegador, só o caminho, e o webview não abre `file://`.
pub async fn ler_imagens_locais(caminhos: &[String]) -> Result<Vec<ImageAssetPayload>, String> {
    chamar("ler_imagens_locais", Args::novo().serde("caminhos", caminhos)).await
}

/// Publica um lote de assets; o backend desfaz arquivos parciais em erro.
pub async fn save_image_assets(
    vault_path: &str,
    images: &[ImageAssetPayload],
) -> Result<Vec<String>, String> {
    chamar(
        "save_image_assets",
        Args::novo()
            .texto("vaultPath", vault_path)
            .serde("images", images),
    )
    .await
}

/// Lê um arquivo do vault (ex: `assets/x.png`, `assets/x.pdf`) como
/// uma `data:` URL (ciclo 121) — necessário porque um `src`/`href`
/// relativo cru resolve contra a origem do webview, não contra a
/// pasta real do vault no disco.
pub async fn read_asset_data_url(vault_path: &str, asset_path: &str) -> Result<String, String> {
    chamar_texto("read_asset_data_url", Args::novo().texto("vaultPath", vault_path).texto("assetPath", asset_path)).await
}

/// Metadados de um arquivo em `assets/`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AssetInfo {
    /// Path relativo ao vault.
    pub path: String,
    /// Tamanho em bytes.
    pub size: u64,
}

/// Lista arquivos em assets/ com tamanho.
pub async fn list_assets_info(vault_path: &str) -> Result<Vec<AssetInfo>, String> {
    chamar("list_assets_info", Args::novo().texto("vaultPath", vault_path)).await
}

/// Remove um arquivo de assets/.
pub async fn delete_asset(vault_path: &str, asset_path: &str) -> Result<(), String> {
    chamar_sem_retorno(
        "delete_asset",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("assetPath", asset_path),
    )
    .await
}

/// Busca texto no conteúdo de todas as páginas.
pub async fn search_content(
    vault_path: &str,
    query: &str,
) -> Result<Vec<anotadinho_core::embed::SearchHit>, String> {
    chamar("search_content", Args::novo().texto("vaultPath", vault_path).texto("query", query)).await
}

/// Verifica se houve mudanças no vault desde a última verificação.
pub async fn check_changes(vault_path: &str) -> Result<bool, String> {
    chamar_bool("check_changes", Args::novo().texto("vaultPath", vault_path), false).await
}

/// Abre ou cria o journal do dia.
pub async fn open_today_journal(vault_path: &str) -> Result<PageMeta, String> {

    chamar("open_today_journal", Args::novo().texto("vaultPath", vault_path)).await
}

/// Cria uma nova página em pages/.
pub async fn create_page(vault_path: &str, title: &str) -> Result<PageMeta, String> {
    create_page_with_type(vault_path, title, "md").await
}

/// Cria pagina com tipo especifico (md, kanban, calendar, table).
pub async fn create_page_with_type(
    vault_path: &str,
    title: &str,
    page_type: &str,
) -> Result<PageMeta, String> {

    chamar("create_page_with_type", Args::novo().texto("vaultPath", vault_path).texto("title", title).texto("pageType", page_type)).await
}

/// Grava o conteúdo de uma página no disco.
pub async fn write_page(vault_path: &str, page_path: &str, content: &str) -> Result<(), String> {
    chamar_sem_retorno(
        "write_page",
        Args::novo()
            .texto("vaultPath", vault_path)
            .texto("pagePath", page_path)
            .texto("content", content),
    )
    .await
}

/// Dispara o agente e volta na hora (ciclo 213).
///
/// Não espera a resposta: quem acompanha é `estado_agente`. É o que
/// permite sair da conversa no meio de uma execução longa sem matar o
/// processo nem perder o que ele responder.
pub async fn iniciar_agente(
    adaptador: &anotadinho_core::agente::Adaptador,
    prompt: &str,
    vault_path: &str,
    conversa_path: &str,
) -> Result<(), String> {
    chamar_sem_retorno(
        "iniciar_agente",
        Args::novo()
            .serde("adaptador", adaptador)
            .texto("prompt", prompt)
            .texto("vaultPath", vault_path)
            .texto("conversaPath", conversa_path),
    )
    .await
}

/// Como está a execução desta conversa.
///
/// `None` = não há nenhuma, ou o resultado já foi entregue. O backend
/// devolve um estado terminal UMA vez só: quem pergunta é quem grava a
/// resposta, então repetir faria a mesma resposta entrar duas vezes.
pub async fn estado_agente(
    conversa_path: &str,
) -> Result<Option<anotadinho_core::agente::EstadoJob>, String> {
    chamar_opcional("estado_agente", Args::novo().texto("conversaPath", conversa_path)).await
}

/// Interrompe a execução desta conversa.
pub async fn cancelar_agente(conversa_path: &str) -> Result<(), String> {
    chamar_sem_retorno("cancelar_agente", Args::novo().texto("conversaPath", conversa_path)).await
}

/// Abre o seletor de pasta e devolve o caminho escolhido (ciclo 216).
pub async fn escolher_pasta() -> Result<Option<String>, String> {
    chamar_opcional("escolher_pasta", Args::novo()).await
}

// ── propostas (ciclo 204) ────────────────────────────────────────────

/// Propostas pendentes de revisão.
pub async fn listar_propostas(
    vault_path: &str,
) -> Result<Vec<anotadinho_core::proposta::Proposta>, String> {
    chamar("listar_propostas", Args::novo().texto("vaultPath", vault_path)).await
}

/// Aplica uma proposta e devolve o path escrito.
pub async fn aplicar_proposta(vault_path: &str, id: &str) -> Result<String, String> {
    chamar_proposta("aplicar_proposta", vault_path, id).await
}

/// Descarta uma proposta.
pub async fn recusar_proposta(vault_path: &str, id: &str) -> Result<String, String> {
    chamar_proposta("recusar_proposta", vault_path, id).await
}

async fn chamar_proposta(cmd: &str, vault_path: &str, id: &str) -> Result<String, String> {
    chamar_texto(cmd, Args::novo().texto("vaultPath", vault_path).texto("id", id)).await
}

/// Prepara uma pasta pra ser um vault (ciclo 233). Devolve os caminhos
/// criados; o que já existia é deixado em paz.
pub async fn criar_vault(vault_path: &str) -> Result<Vec<String>, String> {
    chamar("criar_vault", Args::novo().texto("vaultPath", vault_path)).await
}

/// A pasta aberta ainda não tem página nenhuma? (ciclo 233)
pub async fn vault_esta_vazio(vault_path: &str) -> Result<bool, String> {
    chamar("vault_esta_vazio", Args::novo().texto("vaultPath", vault_path)).await
}
