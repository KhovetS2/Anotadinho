//! Estado global da aplicação.
//!
//! Mantém o vault atual, nome do vault, e persiste no localStorage
//! via `gloo-storage` para reabrir automaticamente na próxima sessão.

use gloo_storage::Storage;
use serde::{Deserialize, Serialize};

const KEY_VAULT_PATH: &str = "anotadinho.vault_path";
const KEY_VAULT_NAME: &str = "anotadinho.vault_name";
const KEY_AUTOSAVE_ENABLED: &str = "anotadinho.autosave_enabled";
const KEY_HOME_PAGE_PREFIX: &str = "anotadinho.home_page::";
const KEY_VIM_MODE_ENABLED: &str = "anotadinho.vim_mode_enabled";
const KEY_VIM_KEYMAP: &str = "anotadinho.vim_keymap";
const KEY_GLOBAL_KEYMAP: &str = "anotadinho.global_keymap";
const KEY_NAV_MODE_ENABLED: &str = "anotadinho.nav_mode_enabled";

/// Mapa de teclas do modo Normal do vim mode — cada ação tem UMA tecla
/// configurável. `delete_line`/`yank_line` são especiais: pressionar a
/// tecla configurada DUAS vezes seguidas confirma a ação (mesmo padrão
/// mnemônico do vim `dd`/`yy`, mas sobre a tecla que o usuário escolher,
/// não fixo em "d"/"y").
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VimKeymap {
    pub left: String,
    pub down: String,
    pub up: String,
    pub right: String,
    pub word_forward: String,
    pub word_backward: String,
    pub line_start: String,
    pub line_end: String,
    pub doc_start: String,
    pub doc_end: String,
    pub insert_before: String,
    pub insert_after: String,
    pub open_below: String,
    pub open_above: String,
    pub delete_char: String,
    pub delete_line: String,
    pub yank_line: String,
    pub paste: String,
    pub undo: String,
}

impl Default for VimKeymap {
    fn default() -> Self {
        Self {
            left: "h".into(),
            down: "j".into(),
            up: "k".into(),
            right: "l".into(),
            word_forward: "w".into(),
            word_backward: "b".into(),
            line_start: "0".into(),
            line_end: "$".into(),
            doc_start: "g".into(),
            doc_end: "G".into(),
            insert_before: "i".into(),
            insert_after: "a".into(),
            open_below: "o".into(),
            open_above: "O".into(),
            delete_char: "x".into(),
            delete_line: "d".into(),
            yank_line: "y".into(),
            paste: "p".into(),
            undo: "u".into(),
        }
    }
}

impl VimKeymap {
    /// Lista `(rótulo, campo)` — usada pela tela de configuração de
    /// atalhos pra iterar todas as ações sem repetir os nomes na UI.
    pub fn labeled_fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("Esquerda", &self.left),
            ("Baixo", &self.down),
            ("Cima", &self.up),
            ("Direita", &self.right),
            ("Palavra seguinte", &self.word_forward),
            ("Palavra anterior", &self.word_backward),
            ("Início da linha", &self.line_start),
            ("Fim da linha", &self.line_end),
            ("Início do documento", &self.doc_start),
            ("Fim do documento", &self.doc_end),
            ("Inserir antes do cursor", &self.insert_before),
            ("Inserir depois do cursor", &self.insert_after),
            ("Nova linha abaixo", &self.open_below),
            ("Nova linha acima", &self.open_above),
            ("Apagar caractere", &self.delete_char),
            ("Apagar linha (2x)", &self.delete_line),
            ("Copiar linha (2x)", &self.yank_line),
            ("Colar", &self.paste),
            ("Desfazer", &self.undo),
        ]
    }

    /// Atualiza o campo correspondente ao rótulo (mesmos rótulos de
    /// `labeled_fields`). Não faz nada se o rótulo não existir.
    pub fn set_by_label(&mut self, label: &str, key: String) {
        match label {
            "Esquerda" => self.left = key,
            "Baixo" => self.down = key,
            "Cima" => self.up = key,
            "Direita" => self.right = key,
            "Palavra seguinte" => self.word_forward = key,
            "Palavra anterior" => self.word_backward = key,
            "Início da linha" => self.line_start = key,
            "Fim da linha" => self.line_end = key,
            "Início do documento" => self.doc_start = key,
            "Fim do documento" => self.doc_end = key,
            "Inserir antes do cursor" => self.insert_before = key,
            "Inserir depois do cursor" => self.insert_after = key,
            "Nova linha abaixo" => self.open_below = key,
            "Nova linha acima" => self.open_above = key,
            "Apagar caractere" => self.delete_char = key,
            "Apagar linha (2x)" => self.delete_line = key,
            "Copiar linha (2x)" => self.yank_line = key,
            "Colar" => self.paste = key,
            "Desfazer" => self.undo = key,
            _ => {}
        }
    }
}

/// Mapa de atalhos do app inteiro (ciclo 105) — cada ação tem UMA
/// tecla, sempre combinada com Ctrl/Cmd (mesma convenção de TODOS os
/// atalhos globais já existentes hoje: Ctrl+N, Ctrl+K, Ctrl+B, Ctrl+W,
/// Ctrl+S, Ctrl+Z). String vazia = sem atalho atribuído. Reusa o mesmo
/// `KeymapCaptureModal` genérico (ciclo 104) do `VimKeymap` — por isso
/// o valor capturado é só a tecla crua (`e.key()`), sem compor
/// modificador; o Ctrl é sempre implícito no DISPATCHER
/// (`app.rs::onkeydown`), nunca guardado no valor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalKeymap {
    pub new_page: String,
    pub new_folder: String,
    pub toggle_theme: String,
    pub toggle_sidebar: String,
    pub today: String,
    pub view_tags: String,
    pub view_assets: String,
    pub open_palette: String,
    pub save: String,
    pub close_tab: String,
    pub next_tab: String,
    pub prev_tab: String,
    pub undo: String,
    pub redo: String,
    pub toggle_vim_mode: String,
    /// Foca a sidebar pra navegar por teclado — ação já cadastrada
    /// aqui, mas o COMPORTAMENTO (destacar item + setas) só chega no
    /// ciclo 106. Sem tecla configurada nesta v1 (vazio).
    pub focus_sidebar: String,
    /// Mesma ideia de `focus_sidebar`, mas pro editor — comportamento
    /// completo é responsabilidade de ciclos futuros.
    pub focus_editor: String,
    /// Liga/desliga a CAPACIDADE do modo de navegação hierárquico
    /// (ciclo 133) — mesmo padrão de `toggle_vim_mode`. Enquanto
    /// ligado, a primeira seta pressionada fora de um campo de texto
    /// já inicia uma sessão de navegação (Enter/Backspace/Escape são
    /// fixos dentro da sessão, não remapeáveis aqui — ver
    /// `ui/src/nav_mode.rs`).
    pub toggle_nav_mode: String,
    /// Salta o foco pro PRÓXIMO embed inline da página e abre uma
    /// sessão de navegação dentro dele (ciclo 165) — é o caminho de
    /// teclado pra alcançar kanban/consulta/cronograma sem Tab às
    /// cegas por todos os botões do embed anterior.
    pub next_embed: String,
    /// Mesma coisa, pro embed anterior.
    pub prev_embed: String,
}

impl Default for GlobalKeymap {
    fn default() -> Self {
        Self {
            // Atalhos que já existiam hoje — MESMA tecla, pra não mudar
            // nada sem o usuário mexer (Ctrl+P como alias de Ctrl+K foi
            // descontinuado: v1 é uma tecla por ação, ver Notas do
            // ciclo 105).
            new_page: "n".into(),
            open_palette: "k".into(),
            toggle_sidebar: "b".into(),
            next_tab: "w".into(),
            save: "s".into(),
            undo: "z".into(),
            // Ctrl+Shift+Z (refazer) não é representável neste esquema
            // de "uma tecla + Ctrl implícito" — Ctrl+Y é a convenção
            // alternativa de "refazer" mais comum (Word/VSCode/etc).
            redo: "y".into(),
            // Ciclo 130: as 10 ações abaixo nasceram sem tecla (ciclo
            // 105 preferiu não arriscar uma colisão escolhida às
            // pressas) — preenchidas agora com um mapeamento pensado
            // pra quem já usa neovim no dia a dia (motivado pelo pedido
            // do usuário; ver a config real dele — `harpoon.lua`,
            // `neo-tree.lua`, `bufferline.lua`, `keymaps.lua` — usada
            // como referência pras escolhas abaixo). Evita
            // deliberadamente 'a'/'c'/'v'/'x' (select-all/copy/paste/
            // cut nativos do editor de texto — colidiriam de verdade
            // dentro do `contenteditable`, diferente de atalhos tipo
            // Ctrl+N/Ctrl+F que só existem em CHROME de navegador, que
            // o WebView do Tauri não tem).
            //
            // "f" = Folder — mesma inicial do rótulo em português
            // ("nova pasta") e do conceito em si.
            new_folder: "f".into(),
            // "t" = Theme.
            toggle_theme: "t".into(),
            // "d" = Day — jornal/página do dia.
            today: "d".into(),
            // "g" = taG — mnemônico mais fraco (letras óbvias já
            // ocupadas), mas livre e fácil de lembrar por associação.
            view_tags: "g".into(),
            // "u" = sem mnemônico forte disponível pra "Assets" nesse
            // alfabeto já quase todo ocupado — só a letra livre que
            // sobrou, documentado aqui pra não parecer acidental.
            view_assets: "u".into(),
            // "q" = mesmo "q" do `:q` do vim (fecha o buffer/janela
            // atual) — a aba aqui é o equivalente mais próximo de um
            // buffer.
            close_tab: "q".into(),
            // "h" = mesma semântica de "esquerda/voltar" do `h` de
            // movimento do vim (par conceitual do `next_tab` = "w",
            // não literalmente h/l porque "w" já era o padrão
            // existente antes deste ciclo).
            prev_tab: "h".into(),
            // "m" = Modal (vim mode é edição modal).
            toggle_vim_mode: "m".into(),
            // "e" = Explorer — o `<C-n>` do neovim do usuário abre o
            // Neo-tree (árvore de arquivos); "n" já está ocupado aqui
            // por "Nova página", então usa o mnemônico em inglês do
            // que a árvore DE FATO é (um "explorer" de arquivos) em
            // vez da tecla literal dele.
            focus_sidebar: "e".into(),
            // "l" = tecla EXATA que o usuário já usa no próprio
            // `keymaps.lua` pra "mover o foco pra janela da direita"
            // (`<C-l>` → `<C-w><C-l>`) — no Anotadinho, sidebar fica à
            // esquerda e o editor à direita, mesma geometria mental.
            focus_editor: "l".into(),
            // "." e "," = o par de "avançar/voltar" mais neutro que
            // sobrou (o alfabeto já estava quase todo ocupado) e que
            // não colide com edição de texto dentro do
            // `contenteditable`, diferente de qualquer letra.
            next_embed: ".".into(),
            prev_embed: ",".into(),
            // "j" — "r" (ciclo 133) foi trocado por causa de um bug
            // real relatado pelo usuário: Ctrl+R não disparava o
            // atalho, quase certamente porque o WebKitGTK (motor do
            // Tauri no Linux) já reserva Ctrl+R pra "recarregar" a
            // nível de engine, antes até de chegar no JS da página —
            // diferente do Ctrl+N (testado nesta mesma sessão e
            // confirmado inofensivo) porque "recarregar" é uma
            // convenção de fato universal de qualquer coisa baseada
            // em WebView/browser, não só chrome de navegador. "j" não
            // tem convenção conhecida nesse nível nem colide com
            // atalhos nativos de edição de texto (cut/copy/paste/
            // select-all/bold/italic/underline — ver comentário do
            // `new_folder` acima); conexão fraca com "j" = baixo no
            // vim, ao menos soa como "mover/navegar".
            toggle_nav_mode: "j".into(),
        }
    }
}

impl GlobalKeymap {
    /// Lista `(rótulo, tecla)` — mesmo papel de `VimKeymap::labeled_fields`.
    pub fn labeled_fields(&self) -> Vec<(&'static str, &str)> {
        vec![
            ("Nova página", &self.new_page),
            ("Nova pasta", &self.new_folder),
            ("Alternar tema", &self.toggle_theme),
            ("Alternar sidebar", &self.toggle_sidebar),
            ("Ir pra Hoje", &self.today),
            ("Ver Tags", &self.view_tags),
            ("Ver Assets", &self.view_assets),
            ("Abrir paleta de comandos", &self.open_palette),
            ("Salvar", &self.save),
            ("Fechar aba atual", &self.close_tab),
            ("Próxima aba", &self.next_tab),
            ("Aba anterior", &self.prev_tab),
            ("Desfazer", &self.undo),
            ("Refazer", &self.redo),
            ("Alternar vim mode", &self.toggle_vim_mode),
            ("Focar sidebar", &self.focus_sidebar),
            ("Focar editor", &self.focus_editor),
            ("Alternar modo de navegação", &self.toggle_nav_mode),
            ("Próximo embed", &self.next_embed),
            ("Embed anterior", &self.prev_embed),
        ]
    }

    /// Atualiza o campo correspondente ao rótulo. Não faz nada se o
    /// rótulo não existir.
    pub fn set_by_label(&mut self, label: &str, key: String) {
        match label {
            "Nova página" => self.new_page = key,
            "Nova pasta" => self.new_folder = key,
            "Alternar tema" => self.toggle_theme = key,
            "Alternar sidebar" => self.toggle_sidebar = key,
            "Ir pra Hoje" => self.today = key,
            "Ver Tags" => self.view_tags = key,
            "Ver Assets" => self.view_assets = key,
            "Abrir paleta de comandos" => self.open_palette = key,
            "Salvar" => self.save = key,
            "Fechar aba atual" => self.close_tab = key,
            "Próxima aba" => self.next_tab = key,
            "Aba anterior" => self.prev_tab = key,
            "Desfazer" => self.undo = key,
            "Refazer" => self.redo = key,
            "Alternar vim mode" => self.toggle_vim_mode = key,
            "Focar sidebar" => self.focus_sidebar = key,
            "Focar editor" => self.focus_editor = key,
            "Alternar modo de navegação" => self.toggle_nav_mode = key,
            "Próximo embed" => self.next_embed = key,
            "Embed anterior" => self.prev_embed = key,
            _ => {}
        }
    }
}

/// Ação de editor disparada de fora dele (GlobalKeymap, ciclo 105) —
/// usada como "ponte" pra `App` conseguir mandar Salvar/Desfazer/Refazer
/// pro `Editor` mesmo quando o foco de teclado não está dentro do
/// contenteditable (onde o atalho local — Ctrl+S/Ctrl+Z — já funciona
/// direto, sem precisar dessa ponte).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GlobalEditorAction {
    /// Salva a página atual.
    Save,
    /// Desfaz a última edição.
    Undo,
    /// Refaz a última edição desfeita.
    Redo,
}

/// Estado da aplicação.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AppState {
    /// Vault aberto (None = nenhum).
    pub vault_path: Option<String>,
    /// Nome do vault (nome do diretório).
    pub vault_name: Option<String>,
}

impl AppState {
    /// Cria um estado novo vazio.
    pub fn new() -> Self {
        Self::default()
    }
}

/// Salva o path do vault no localStorage.
pub fn save_vault_path(path: &str) {
    let _ = gloo_storage::LocalStorage::set(KEY_VAULT_PATH, path);
}

/// Carrega o path do vault do localStorage.
pub fn load_vault_path() -> Option<String> {
    gloo_storage::LocalStorage::get(KEY_VAULT_PATH).ok()
}

/// Salva o nome do vault no localStorage.
pub fn save_vault_name(name: &str) {
    let _ = gloo_storage::LocalStorage::set(KEY_VAULT_NAME, name);
}

/// Carrega o nome do vault do localStorage.
pub fn load_vault_name() -> Option<String> {
    gloo_storage::LocalStorage::get(KEY_VAULT_NAME).ok()
}

/// Extrai o nome do diretório de um path.
pub fn extract_name_from_path(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "vault".to_string())
}

/// Remove vault path/name do localStorage.
pub fn clear_vault() {
    let _ = gloo_storage::LocalStorage::delete(KEY_VAULT_PATH);
    let _ = gloo_storage::LocalStorage::delete(KEY_VAULT_NAME);
}

/// Salva a preferência de salvamento automático no localStorage.
pub fn save_autosave_enabled(enabled: bool) {
    let _ = gloo_storage::LocalStorage::set(KEY_AUTOSAVE_ENABLED, enabled);
}

/// Carrega a preferência de salvamento automático (padrão: ativado — sem
/// isso o usuário perdia edições ao trocar de página sem salvar antes).
pub fn load_autosave_enabled() -> bool {
    gloo_storage::LocalStorage::get(KEY_AUTOSAVE_ENABLED).unwrap_or(true)
}

/// Chave de storage da página inicial — por vault (cada vault tem a sua
/// própria página de início, guardadas separadamente pelo path do vault).
fn key_home_page(vault_path: &str) -> String {
    format!("{}{}", KEY_HOME_PAGE_PREFIX, vault_path)
}

/// Marca `page_path` como a página inicial deste vault — aberta
/// automaticamente ao abrir o vault (ver `App`).
pub fn save_home_page(vault_path: &str, page_path: &str) {
    let _ = gloo_storage::LocalStorage::set(key_home_page(vault_path), page_path);
}

/// Path da página inicial deste vault, se alguma tiver sido definida.
pub fn load_home_page(vault_path: &str) -> Option<String> {
    gloo_storage::LocalStorage::get(key_home_page(vault_path)).ok()
}

/// Remove a página inicial deste vault.
pub fn clear_home_page(vault_path: &str) {
    let _ = gloo_storage::LocalStorage::delete(key_home_page(vault_path));
}

/// Salva se o vim mode está ativado.
pub fn save_vim_mode_enabled(enabled: bool) {
    let _ = gloo_storage::LocalStorage::set(KEY_VIM_MODE_ENABLED, enabled);
}

/// Carrega se o vim mode está ativado (padrão: desativado).
pub fn load_vim_mode_enabled() -> bool {
    gloo_storage::LocalStorage::get(KEY_VIM_MODE_ENABLED).unwrap_or(false)
}

/// Salva o mapa de teclas do vim mode.
pub fn save_vim_keymap(keymap: &VimKeymap) {
    let _ = gloo_storage::LocalStorage::set(KEY_VIM_KEYMAP, keymap);
}

/// Carrega o mapa de teclas do vim mode (padrão: teclas clássicas do vim).
pub fn load_vim_keymap() -> VimKeymap {
    gloo_storage::LocalStorage::get(KEY_VIM_KEYMAP).unwrap_or_default()
}

/// Salva se o modo de navegação hierárquico (ciclo 133) está ativado.
pub fn save_nav_mode_enabled(enabled: bool) {
    let _ = gloo_storage::LocalStorage::set(KEY_NAV_MODE_ENABLED, enabled);
}

/// Carrega se o modo de navegação está ativado (padrão: desativado).
pub fn load_nav_mode_enabled() -> bool {
    gloo_storage::LocalStorage::get(KEY_NAV_MODE_ENABLED).unwrap_or(false)
}

/// Salva o mapa de atalhos globais do app.
pub fn save_global_keymap(keymap: &GlobalKeymap) {
    let _ = gloo_storage::LocalStorage::set(KEY_GLOBAL_KEYMAP, keymap);
}

/// Carrega o mapa de atalhos globais do app (padrão: atalhos atuais).
pub fn load_global_keymap() -> GlobalKeymap {
    gloo_storage::LocalStorage::get(KEY_GLOBAL_KEYMAP).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn global_keymap_default_preserves_existing_shortcuts() {
        let km = GlobalKeymap::default();
        assert_eq!(km.new_page, "n");
        assert_eq!(km.open_palette, "k");
        assert_eq!(km.toggle_sidebar, "b");
        assert_eq!(km.next_tab, "w");
        assert_eq!(km.save, "s");
        assert_eq!(km.undo, "z");
    }

    #[test]
    fn global_keymap_default_binds_neovim_inspired_shortcuts() {
        // Ciclo 130: as ações que nasceram sem tecla (ciclo 105) agora
        // têm um default pensado pra quem já usa neovim — ver
        // comentários no `impl Default for GlobalKeymap` pra cada
        // mnemônico.
        let km = GlobalKeymap::default();
        assert_eq!(km.new_folder, "f");
        assert_eq!(km.toggle_theme, "t");
        assert_eq!(km.today, "d");
        assert_eq!(km.view_tags, "g");
        assert_eq!(km.view_assets, "u");
        assert_eq!(km.close_tab, "q");
        assert_eq!(km.prev_tab, "h");
        assert_eq!(km.toggle_vim_mode, "m");
        assert_eq!(km.focus_sidebar, "e");
        assert_eq!(km.focus_editor, "l");
        assert_eq!(km.toggle_nav_mode, "j");
    }

    #[test]
    fn global_keymap_default_has_no_duplicate_keys() {
        // Cada tecla do default tem que ser única — uma colisão faria
        // duas ações disparar juntas no mesmo Ctrl+tecla (o dispatcher
        // em `app.rs::onkeydown` para no primeiro `match` que bater).
        let km = GlobalKeymap::default();
        let keys: Vec<&str> = km.labeled_fields().into_iter().map(|(_, k)| k).collect();
        let mut sorted = keys.clone();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), keys.len(), "teclas default duplicadas: {:?}", keys);
    }

    #[test]
    fn global_keymap_set_by_label_roundtrips_through_labeled_fields() {
        let mut km = GlobalKeymap::default();
        km.set_by_label("Nova pasta", "f".to_string());
        assert_eq!(km.new_folder, "f");
        let fields = km.labeled_fields();
        assert!(fields.iter().any(|(label, key)| *label == "Nova pasta" && *key == "f"));
    }

    #[test]
    fn global_keymap_set_by_label_unknown_label_is_noop() {
        let mut km = GlobalKeymap::default();
        let before = km.clone();
        km.set_by_label("Ação inexistente", "x".to_string());
        assert_eq!(km, before);
    }

    #[test]
    fn global_keymap_labeled_fields_cobre_todo_campo_configuravel() {
        // Conta explícita (em vez de um número solto): cada ação que dá
        // pra reconfigurar precisa aparecer no cheatsheet e no modal de
        // atalhos, e os dois leem daqui. Se um campo novo entrar sem
        // rótulo, ele fica invisível pro usuário — e este teste quebra.
        let km = GlobalKeymap::default();
        assert_eq!(km.labeled_fields().len(), 20);
        for (label, _) in km.labeled_fields() {
            let mut probe = GlobalKeymap::default();
            probe.set_by_label(label, "§".into());
            assert!(
                probe.labeled_fields().iter().any(|(l, v)| *l == label && *v == "§"),
                "rótulo {label} não tem par em set_by_label"
            );
        }
    }
}
