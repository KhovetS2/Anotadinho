//! Sidebar com lista de páginas do vault.
//!
//! Mostra duas seções: Pages (vault/pages/) e Journals (vault/journals/).
//! Pages é organizada em árvore de pastas (subdiretórios reais sob
//! `pages/`, ver `crates/vault/src/io.rs`) — Journals continua uma
//! lista flat por data. Click em um item emite callback com o path da
//! página selecionada. Botão "+" na seção Pages cria nova página; botão
//! "📁+" cria pasta; cada pasta tem seu próprio "+" pra criar página já
//! dentro dela. Campo de busca (com filtro ativo) volta pra lista flat
//! — mais simples de escanear resultados espalhados por várias pastas.

use std::collections::{BTreeMap, HashSet};

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::dialog::PendingDialog;

/// Nó da árvore de pastas construída a partir de `PageMeta::path` +
/// pastas vazias listadas separadamente (arquivos não revelam pastas
/// sem nada dentro).
#[derive(Default)]
struct TreeNode {
    folders: BTreeMap<String, TreeNode>,
    pages: Vec<PageMeta>,
}

fn build_tree(pages: &[PageMeta], folders: &[String]) -> TreeNode {
    let mut root = TreeNode::default();
    for f in folders {
        let rel = f.strip_prefix("pages/").unwrap_or(f.as_str());
        if rel.is_empty() {
            continue;
        }
        let mut node = &mut root;
        for seg in rel.split('/') {
            node = node.folders.entry(seg.to_string()).or_default();
        }
    }
    for p in pages {
        let rel = p.path.strip_prefix("pages/").unwrap_or(p.path.as_str());
        let mut segs: Vec<&str> = rel.split('/').collect();
        segs.pop(); // último segmento é o arquivo, não uma pasta
        let mut node = &mut root;
        for seg in segs {
            node = node.folders.entry(seg.to_string()).or_default();
        }
        node.pages.push(p.clone());
    }
    root
}

/// Um item navegável por teclado (ciclo 106) — pasta ou página, na
/// mesma ordem visual em que `render_tree` desenha (folders primeiro,
/// alfabético via `BTreeMap`, depois páginas), respeitando pastas
/// colapsadas (filhas de uma pasta colapsada não entram na lista).
struct NavItem {
    /// Path da pasta ou da página — mesma chave usada em
    /// `collapsed_folders`/`selected_path`.
    key: String,
    is_folder: bool,
    /// Path do pai (`None` pra itens no nível raiz, que não têm um
    /// `NavItem` de pasta correspondente pra subir com `ArrowLeft`).
    parent: Option<String>,
}

/// Achata a árvore de pastas numa lista navegável, na mesma ordem em
/// que `render_tree` renderiza — usada pra `ArrowDown`/`ArrowUp`/
/// `ArrowRight`/`ArrowLeft` (ciclo 106) sem duplicar a lógica de
/// travessia em dois lugares.
fn flatten_nav(node: &TreeNode, path_prefix: &str, collapsed: &HashSet<String>, depth: usize, out: &mut Vec<NavItem>) {
    let parent = if depth == 0 { None } else { Some(path_prefix.to_string()) };
    for (name, sub) in node.folders.iter() {
        let full_path = format!("{}/{}", path_prefix, name);
        out.push(NavItem { key: full_path.clone(), is_folder: true, parent: parent.clone() });
        if !collapsed.contains(&full_path) {
            flatten_nav(sub, &full_path, collapsed, depth + 1, out);
        }
    }
    for p in &node.pages {
        out.push(NavItem { key: p.path.clone(), is_folder: false, parent: parent.clone() });
    }
}

/// Remove `/`/`\`/`..` de um nome de pasta digitado pelo usuário — pastas
/// são só um nível de organização visual, não precisam de slug com
/// hífens como as páginas.
fn sanitize_folder_name(name: &str) -> String {
    let name = name.trim().replace(['/', '\\'], "-").replace("..", "-");
    if name.is_empty() {
        "nova-pasta".to_string()
    } else {
        name
    }
}

/// Props da Sidebar.
#[derive(Properties, PartialEq, Clone)]
pub struct SidebarProps {
    /// Path absoluto do vault aberto.
    pub vault_path: String,
    /// Callback ao selecionar uma página (path relativo).
    pub on_page_selected: Callback<PageMeta>,
    /// Incrementa para forçar reload da lista.
    #[prop_or_default]
    pub list_version: u32,
    /// Sidebar colapsada (compacta).
    #[prop_or_default]
    pub collapsed: bool,
    /// Abre o modal de diálogo do app (ver `crate::dialog`).
    pub open_dialog: Callback<PendingDialog>,
    /// Nonce disparado pela ação "Focar sidebar" do `GlobalKeymap`
    /// (ciclo 105/106) — qualquer mudança de valor (mesmo repetida)
    /// ativa a navegação por teclado, destacando o primeiro item.
    #[prop_or_default]
    pub activate_nav_signal: Option<u32>,
}

/// Componente Sidebar.
#[function_component(Sidebar)]
pub fn sidebar(props: &SidebarProps) -> Html {
    let pages = use_state(Vec::<PageMeta>::new);
    let folders = use_state(Vec::<String>::new);
    let selected_path = use_state(|| None::<String>);
    let loading = use_state(|| true);
    let refresh_tick = use_state(|| 0u32);
    let search = use_state(String::new);
    // Navegação por teclado (ciclo 106). `collapsed_folders` substitui
    // o `<details open=true>` sempre-aberto de antes por um estado de
    // verdade — precisa ser controlável pra `ArrowLeft`/`ArrowRight`
    // funcionarem; um listener `ontoggle` mantém o clique nativo do
    // mouse no `<summary>` sincronizado com esse mesmo estado (fonte
    // única, não duas coisas competindo).
    let collapsed_folders = use_state(HashSet::<String>::new);
    // `None` = navegação por teclado inativa (comportamento de sempre,
    // só clique de mouse); `Some(key)` = item destacado (pasta ou
    // página) — ativado pela ação "Focar sidebar" do `GlobalKeymap`.
    let nav_active = use_state(|| None::<String>);
    let nav_container_ref = use_node_ref();
    let nav_item_ref = use_node_ref();

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        let folders = folders.clone();
        let loading = loading.clone();
        let tick = (*refresh_tick, props.list_version);

        use_effect_with(tick, move |_| {
            let vault_path = vault_path.clone();
            let pages = pages.clone();
            let folders = folders.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                match api::list_pages(&vault_path).await {
                    Ok(list) => {
                        pages.set(list);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        pages.set(Vec::new());
                    }
                }
                match api::list_folders(&vault_path).await {
                    Ok(list) => folders.set(list),
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        folders.set(Vec::new());
                    }
                }
                loading.set(false);
            });
            || ()
        });
    }

    let filter = search.trim().to_lowercase();
    let all_pages: Vec<PageMeta> = if filter.is_empty() {
        (*pages).clone()
    } else {
        pages
            .iter()
            .filter(|p| p.title.to_lowercase().contains(&filter))
            .cloned()
            .collect()
    };

    let page_items: Vec<PageMeta> = all_pages.iter().filter(|p| p.section == "pages").cloned().collect();
    let journal_items: Vec<PageMeta> = all_pages.iter().filter(|p| p.section == "journals").cloned().collect();

    // Lista navegável (ciclo 106) — árvore achatada quando sem filtro
    // (mesma ordem de `render_tree`), ou a lista flat de resultados
    // quando filtrando (mesma ordem de `render_movable_list`). Só a
    // seção Pages entra aqui (Journals fica de fora, ver Não-objetivos
    // do ciclo 106).
    let flat_nav: Vec<NavItem> = if filter.is_empty() {
        let mut out = Vec::new();
        flatten_nav(&build_tree(&page_items, &folders), "pages", &collapsed_folders, 0, &mut out);
        out
    } else {
        page_items.iter().map(|p| NavItem { key: p.path.clone(), is_folder: false, parent: None }).collect()
    };

    // Ativa a navegação por teclado quando "Focar sidebar"
    // (`GlobalKeymap`) dispara — destaca o primeiro item e foca o
    // container (pra `onkeydown` capturar as setas a partir daqui).
    {
        let nav_active = nav_active.clone();
        let nav_container_ref = nav_container_ref.clone();
        let first_key = flat_nav.first().map(|i| i.key.clone());
        use_effect_with(props.activate_nav_signal, move |signal| {
            if signal.is_some() {
                nav_active.set(first_key);
                if let Some(el) = nav_container_ref.cast::<web_sys::HtmlElement>() {
                    let _ = el.focus();
                }
            }
            || ()
        });
    }

    // Rola o item destacado pra dentro da área visível ao navegar —
    // mesmo padrão do menu `/` do editor (ciclo 073/082).
    {
        let nav_item_ref = nav_item_ref.clone();
        use_effect_with((*nav_active).clone(), move |_| {
            if let Some(el) = nav_item_ref.cast::<web_sys::Element>() {
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&opts);
            }
            || ()
        });
    }

    let on_nav_keydown = {
        let nav_active = nav_active.clone();
        let collapsed_folders = collapsed_folders.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let flat_nav_keys: Vec<(String, bool, Option<String>)> = flat_nav.iter()
            .map(|i| (i.key.clone(), i.is_folder, i.parent.clone()))
            .collect();
        let page_items_kd = page_items.clone();
        Callback::from(move |e: KeyboardEvent| {
            let Some(ref active) = *nav_active else { return };
            let Some(idx) = flat_nav_keys.iter().position(|(k, ..)| k == active) else { return };
            let (key, is_folder, parent) = flat_nav_keys[idx].clone();
            match e.key().as_str() {
                "ArrowDown" => {
                    e.prevent_default();
                    if let Some((next_key, ..)) = flat_nav_keys.get(idx + 1) {
                        nav_active.set(Some(next_key.clone()));
                    }
                }
                "ArrowUp" => {
                    e.prevent_default();
                    if idx > 0 {
                        if let Some((prev_key, ..)) = flat_nav_keys.get(idx - 1) {
                            nav_active.set(Some(prev_key.clone()));
                        }
                    }
                }
                "ArrowRight" => {
                    e.prevent_default();
                    if is_folder {
                        if collapsed_folders.contains(&key) {
                            let mut c = (*collapsed_folders).clone();
                            c.remove(&key);
                            collapsed_folders.set(c);
                        } else if let Some((child_key, _, child_parent)) = flat_nav_keys.get(idx + 1) {
                            if child_parent.as_deref() == Some(key.as_str()) {
                                nav_active.set(Some(child_key.clone()));
                            }
                        }
                    }
                }
                "ArrowLeft" => {
                    e.prevent_default();
                    if is_folder && !collapsed_folders.contains(&key) {
                        let mut c = (*collapsed_folders).clone();
                        c.insert(key.clone());
                        collapsed_folders.set(c);
                    } else if let Some(parent_key) = parent {
                        nav_active.set(Some(parent_key));
                    }
                }
                "Enter" => {
                    e.prevent_default();
                    if !is_folder {
                        if let Some(meta) = page_items_kd.iter().find(|p| p.path == key) {
                            selected_path.set(Some(key.clone()));
                            on_page_selected.emit(meta.clone());
                        }
                    }
                }
                "Escape" => {
                    e.prevent_default();
                    // Sem isso o Escape também borbulha até `.app-root`,
                    // que tem seu PRÓPRIO caso especial de Escape (fora
                    // do GlobalKeymap) pra deselecionar a página aberta —
                    // sairia da região da sidebar E fechava a página ao
                    // mesmo tempo, dois efeitos por um Escape só.
                    e.stop_propagation();
                    nav_active.set(None);
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        let target = doc.query_selector(".editor__wysiwyg[contenteditable=\"true\"]").ok().flatten()
                            .or_else(|| doc.query_selector(".app-root").ok().flatten());
                        if let Some(el) = target.and_then(|e| e.dyn_into::<web_sys::HtmlElement>().ok()) {
                            let _ = el.focus();
                        }
                    }
                }
                _ => {}
            }
        })
    };

    // Content search results
    let content_results = use_state(Vec::<(String, String)>::new);
    let searching = use_state(|| false);
    {
        let vault_path = props.vault_path.clone();
        let filter = filter.clone();
        let content_results = content_results.clone();
        let searching = searching.clone();
        use_effect_with(filter.clone(), move |_| {
            let should_run = filter.len() >= 3;
            if should_run {
                let vault_path = vault_path.clone();
                let filter = filter.clone();
                let content_results = content_results.clone();
                let searching = searching.clone();
                searching.set(true);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::search_content(&vault_path, &filter).await {
                        Ok(r) => content_results.set(r),
                        Err(_) => content_results.set(Vec::new()),
                    }
                    searching.set(false);
                });
            } else {
                content_results.set(Vec::new());
                searching.set(false);
            }
            || {}
        });
    }

    let on_search_input = {
        let search = search.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                search.set(input.value());
            }
        })
    };

    let clear_search = {
        let search = search.clone();
        Callback::from(move |_| search.set(String::new()))
    };

    let on_search_keydown = {
        let search = search.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Escape" {
                search.set(String::new());
            }
        })
    };

    let on_new_page = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog_for_error = open_dialog.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título da nova página".to_string(),
                default: "Nova nota".to_string(),
                on_submit: Callback::from(move |title: String| {
                    let vault_path = vault_path.clone();
                    let selected_path = selected_path.clone();
                    let on_page_selected = on_page_selected.clone();
                    let refresh_tick = refresh_tick.clone();
                    let open_dialog = open_dialog_for_error.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::create_page(&vault_path, &title).await {
                            Ok(meta) => {
                                selected_path.set(Some(meta.path.clone()));
                                on_page_selected.emit(meta);
                                refresh_tick.set(*refresh_tick + 1);
                            }
                            Err(e) => {
                                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                                open_dialog.emit(PendingDialog::Alert {
                                    message: format!("Erro ao criar página: {}", e),
                                });
                            }
                        }
                    });
                }),
            });
        })
    };

    let on_today = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::open_today_journal(&vault_path).await {
                    Ok(meta) => {
                        selected_path.set(Some(meta.path.clone()));
                        on_page_selected.emit(meta);
                        refresh_tick.set(*refresh_tick + 1);
                    }
                    Err(e) => {
                        web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        open_dialog.emit(PendingDialog::Alert {
                            message: format!("Erro ao abrir journal: {}", e),
                        });
                    }
                }
            });
        })
    };

    let on_new_landing = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog_for_error = open_dialog.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Título da página inicial".to_string(),
                default: "Início".to_string(),
                on_submit: Callback::from(move |title: String| {
                    let vault_path = vault_path.clone();
                    let selected_path = selected_path.clone();
                    let on_page_selected = on_page_selected.clone();
                    let refresh_tick = refresh_tick.clone();
                    let open_dialog = open_dialog_for_error.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::create_page_with_type(&vault_path, &title, "landing").await {
                            Ok(meta) => {
                                selected_path.set(Some(meta.path.clone()));
                                on_page_selected.emit(meta);
                                refresh_tick.set(*refresh_tick + 1);
                            }
                            Err(e) => {
                                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                                open_dialog.emit(PendingDialog::Alert {
                                    message: format!("Erro ao criar página inicial: {}", e),
                                });
                            }
                        }
                    });
                }),
            });
        })
    };

    let on_new_folder = {
        let vault_path = props.vault_path.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog_for_error = open_dialog.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nome da nova pasta".to_string(),
                default: "Nova pasta".to_string(),
                on_submit: Callback::from(move |name: String| {
                    let vault_path = vault_path.clone();
                    let refresh_tick = refresh_tick.clone();
                    let open_dialog = open_dialog_for_error.clone();
                    let folder_path = format!("pages/{}", sanitize_folder_name(&name));
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(e) = api::create_folder(&vault_path, &folder_path).await {
                            web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                            open_dialog.emit(PendingDialog::Alert {
                                message: format!("Erro ao criar pasta: {}", e),
                            });
                        } else {
                            refresh_tick.set(*refresh_tick + 1);
                        }
                    });
                }),
            });
        })
    };

    let make_on_new_page_in = {
        let vault_path = props.vault_path.clone();
        let selected_path = selected_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        move |folder_path: String| {
            let vault_path = vault_path.clone();
            let selected_path = selected_path.clone();
            let on_page_selected = on_page_selected.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog = open_dialog.clone();
            Callback::from(move |e: MouseEvent| {
                // Botão vive dentro do <summary> da pasta — sem isso, o
                // clique também dispararia o toggle nativo do <details>.
                e.prevent_default();
                e.stop_propagation();
                let vault_path = vault_path.clone();
                let selected_path = selected_path.clone();
                let on_page_selected = on_page_selected.clone();
                let refresh_tick = refresh_tick.clone();
                let folder_path = folder_path.clone();
                let open_dialog_for_error = open_dialog.clone();
                open_dialog.emit(PendingDialog::Prompt {
                    title: "Título da nova página".to_string(),
                    default: "Nova nota".to_string(),
                    on_submit: Callback::from(move |title: String| {
                        let vault_path = vault_path.clone();
                        let selected_path = selected_path.clone();
                        let on_page_selected = on_page_selected.clone();
                        let refresh_tick = refresh_tick.clone();
                        let folder_path = folder_path.clone();
                        let open_dialog = open_dialog_for_error.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match api::create_page_in_folder(&vault_path, &folder_path, &title, "md").await {
                                Ok(meta) => {
                                    selected_path.set(Some(meta.path.clone()));
                                    on_page_selected.emit(meta);
                                    refresh_tick.set(*refresh_tick + 1);
                                }
                                Err(e) => {
                                    web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                                    open_dialog.emit(PendingDialog::Alert {
                                        message: format!("Erro ao criar página: {}", e),
                                    });
                                }
                            }
                        });
                    }),
                });
            })
        }
    };

    // "Exportar pasta" (ciclo 101): concatena o markdown fonte de
    // todas as páginas da pasta (recursivo) num dump único e dispara o
    // download — pensado pra colar o conteúdo inteiro no contexto de
    // um agente, diferente do `on_export` de 1 página do editor (que
    // exporta o HTML renderizado).
    let make_on_export = {
        let vault_path = props.vault_path.clone();
        let open_dialog = props.open_dialog.clone();
        move |folder_path: String| {
            let vault_path = vault_path.clone();
            let open_dialog = open_dialog.clone();
            Callback::from(move |e: MouseEvent| {
                e.prevent_default();
                e.stop_propagation();
                let vault_path = vault_path.clone();
                let folder_path = folder_path.clone();
                let open_dialog = open_dialog.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::export_folder(&vault_path, &folder_path).await {
                        Ok(dump) => {
                            let name = folder_path.rsplit('/').next().unwrap_or("pasta");
                            crate::download::download_text_file(&format!("{}.md", name), "text/markdown", &dump);
                        }
                        Err(e) => {
                            open_dialog.emit(PendingDialog::Alert {
                                message: format!("Erro ao exportar pasta: {}", e),
                            });
                        }
                    }
                });
            })
        }
    };

    let make_on_move_page = {
        let vault_path = props.vault_path.clone();
        let refresh_tick = refresh_tick.clone();
        let open_dialog = props.open_dialog.clone();
        move |page_path: String| {
            let vault_path = vault_path.clone();
            let refresh_tick = refresh_tick.clone();
            let open_dialog = open_dialog.clone();
            let file_name = std::path::Path::new(&page_path)
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_default();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let vault_path = vault_path.clone();
                let refresh_tick = refresh_tick.clone();
                let page_path = page_path.clone();
                let file_name = file_name.clone();
                let open_dialog_for_error = open_dialog.clone();
                open_dialog.emit(PendingDialog::Prompt {
                    title: "Mover pra qual pasta? (digite \"raiz\" pra tirar de pastas)".to_string(),
                    default: "raiz".to_string(),
                    on_submit: Callback::from(move |folder: String| {
                        let vault_path = vault_path.clone();
                        let refresh_tick = refresh_tick.clone();
                        let page_path = page_path.clone();
                        let file_name = file_name.clone();
                        let open_dialog = open_dialog_for_error.clone();
                        let folder = folder.trim().trim_matches('/').to_string();
                        let is_root = folder.eq_ignore_ascii_case("raiz") || folder == ".";
                        let to_path = if is_root {
                            format!("pages/{}", file_name)
                        } else {
                            format!("pages/{}/{}", sanitize_folder_name(&folder), file_name)
                        };
                        wasm_bindgen_futures::spawn_local(async move {
                            if let Err(e) = api::move_page(&vault_path, &page_path, &to_path).await {
                                web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                                open_dialog.emit(PendingDialog::Alert {
                                    message: format!("Erro ao mover página: {}", e),
                                });
                            } else {
                                refresh_tick.set(*refresh_tick + 1);
                            }
                        });
                    }),
                });
            })
        }
    };

    let has_results = !all_pages.is_empty() || filter.is_empty();

    html! {
        <aside class={ if props.collapsed { "app-sidebar app-sidebar--collapsed" } else { "app-sidebar" } }>
            if props.collapsed {
                <div class="sidebar-collapsed" title="Expandir sidebar">
                    <span class="sidebar-collapsed__icon" title="Pages">{ "📄" }</span>
                    <span class="sidebar-collapsed__icon" title="Journals">{ "📅" }</span>
                    <span class="sidebar-collapsed__icon" title="Buscar">{ "🔍" }</span>
                </div>
            } else {
            <div class="sidebar-search">
                <input class="input input--sm" type="text" placeholder="Buscar páginas..."
                    value={(*search).clone()} oninput={on_search_input} onkeydown={on_search_keydown} />
                if !search.is_empty() {
                    <button class="sidebar-search__clear" onclick={clear_search} title="Limpar busca">
                        { "✕" }
                    </button>
                }
            </div>
            if *loading && pages.is_empty() {
                <p class="app-sidebar__hint">
                    <span class="spinner"></span>
                    { " Carregando..." }
                </p>
            } else if !has_results {
                <p class="app-sidebar__hint">{ "Nenhum resultado" }</p>
            } else {
                <div class="sidebar-section" tabindex="0" ref={nav_container_ref} onkeydown={on_nav_keydown}>
                    <div class="sidebar-section__header">
                        <h3 class="sidebar-section__title">{ "Pages" }</h3>
                        <button class="btn btn--ghost btn--xs" title="Nova página inicial (landing)" onclick={on_new_landing}>{ "🏠+" }</button>
                        <button class="btn btn--ghost btn--xs" title="Nova pasta" onclick={on_new_folder}>{ "📁+" }</button>
                        <button class="btn btn--ghost btn--xs" title="Nova página" onclick={on_new_page}>{ "+" }</button>
                    </div>
                    if page_items.is_empty() {
                        <p class="sidebar-section__empty">{ "Nenhuma página ainda" }</p>
                    } else if filter.is_empty() {
                        { render_tree(&build_tree(&page_items, &folders), "pages", &selected_path, &props.on_page_selected, &make_on_move_page, &make_on_new_page_in, &make_on_export, &collapsed_folders, &nav_active, &nav_item_ref) }
                    } else {
                        { render_movable_list(&page_items, &selected_path, &props.on_page_selected, &make_on_move_page, &nav_active, &nav_item_ref) }
                    }
                </div>
                <div class="sidebar-section">
                    <div class="sidebar-section__header">
                        <h3 class="sidebar-section__title">{ "Journals" }</h3>
                        <button class="btn btn--ghost btn--xs" title="Journal de hoje" onclick={on_today}>{ "Hoje" }</button>
                    </div>
                    { render_list(&journal_items, &selected_path, &props.on_page_selected) }
                </div>
                if !content_results.is_empty() {
                    <div class="sidebar-section">
                        <h3 class="sidebar-section__title">{ format!("Resultados ({})", content_results.len()) }</h3>
                        <ul class="sidebar-list">
                            { for content_results.iter().map(|(path, excerpt)| {
                                let path = path.clone();
                                let excerpt = excerpt.clone();
                                let title = std::path::Path::new(&path).file_stem()
                                    .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                                let page_meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                                let on_page_selected = props.on_page_selected.clone();
                                let selected_path = selected_path.clone();
                                let onclick = Callback::from(move |_| {
                                    selected_path.set(Some(path.clone()));
                                    on_page_selected.emit(page_meta.clone());
                                });
                                html! {
                                    <li class="sidebar-item" {onclick}>
                                        <span class="sidebar-item__icon">{ page_icon("search") }</span>
                                        <div class="sidebar-item__result">
                                            <span class="sidebar-item__title">{ &title }</span>
                                            <span class="sidebar-item__excerpt">{ render_excerpt_highlight(&excerpt) }</span>
                                        </div>
                                    </li>
                                }
                            }) }
                        </ul>
                    </div>
                }
            }
            }
        </aside>
    }
}

fn render_list(
    items: &[PageMeta],
    selected_path: &UseStateHandle<Option<String>>,
    on_page_selected: &Callback<PageMeta>,
) -> Html {
    if items.is_empty() {
        return html! {
            <p class="sidebar-section__empty">{ "Nenhuma página ainda" }</p>
        };
    }
    html! {
        <ul class="sidebar-list">
            { for items.iter().map(|page| {
                let path = page.path.clone();
                let title = page.title.clone();
                let page_meta = page.clone();
                let is_selected = selected_path.as_deref() == Some(path.as_str());
                let class = if is_selected {
                    "sidebar-item sidebar-item--selected"
                } else {
                    "sidebar-item"
                };
                let on_page_selected = on_page_selected.clone();
                let selected_path = selected_path.clone();
                let path_for_cb = path.clone();
                let onclick = Callback::from(move |_| {
                    selected_path.set(Some(path_for_cb.clone()));
                    on_page_selected.emit(page_meta.clone());
                });
                html! {
                    <li {class} {onclick}>
                        <span class="sidebar-item__icon">{ page_icon(&page.section) }</span>
                        <span class="sidebar-item__title">{ &title }</span>
                    </li>
                }
            }) }
        </ul>
    }
}

/// Como `render_list`, mas cada item ganha um botão "📁" pra mover a
/// página pra outra pasta — usado só na seção Pages (Journals e
/// resultados de busca continuam com `render_list`).
fn render_movable_list<F: Fn(String) -> Callback<MouseEvent>>(
    items: &[PageMeta],
    selected_path: &UseStateHandle<Option<String>>,
    on_page_selected: &Callback<PageMeta>,
    make_on_move: &F,
    nav_active: &Option<String>,
    nav_item_ref: &NodeRef,
) -> Html {
    if items.is_empty() {
        return html! {};
    }
    html! {
        <ul class="sidebar-list">
            { for items.iter().map(|page| {
                let path = page.path.clone();
                let title = page.title.clone();
                let page_meta = page.clone();
                let is_selected = selected_path.as_deref() == Some(path.as_str());
                let is_nav_active = nav_active.as_deref() == Some(path.as_str());
                let class = match (is_selected, is_nav_active) {
                    (true, true) => "sidebar-item sidebar-item--selected sidebar-item--nav-active",
                    (true, false) => "sidebar-item sidebar-item--selected",
                    (false, true) => "sidebar-item sidebar-item--nav-active",
                    (false, false) => "sidebar-item",
                };
                let node_ref = if is_nav_active { nav_item_ref.clone() } else { NodeRef::default() };
                let on_page_selected = on_page_selected.clone();
                let selected_path = selected_path.clone();
                let path_for_cb = path.clone();
                let onclick = Callback::from(move |_| {
                    selected_path.set(Some(path_for_cb.clone()));
                    on_page_selected.emit(page_meta.clone());
                });
                let on_move = make_on_move(path.clone());
                html! {
                    <li {class} ref={node_ref} {onclick}>
                        <span class="sidebar-item__icon">{ page_icon(&page.section) }</span>
                        <span class="sidebar-item__title">{ &title }</span>
                        <button class="sidebar-item__move btn btn--ghost btn--xs" title="Mover pra pasta" onclick={on_move}>{ "📁" }</button>
                    </li>
                }
            }) }
        </ul>
    }
}

/// Renderiza a árvore de pastas recursivamente — pastas primeiro,
/// depois as páginas do nível atual. `<details open>` é CONTROLADO por
/// `collapsed` (ciclo 106, pra `ArrowLeft`/`ArrowRight` poderem
/// expandir/colapsar) — um `ontoggle` sincroniza de volta o clique
/// nativo do mouse no `<summary>`, fonte única de verdade.
#[allow(clippy::too_many_arguments)]
fn render_tree<
    F: Fn(String) -> Callback<MouseEvent>,
    G: Fn(String) -> Callback<MouseEvent>,
    H: Fn(String) -> Callback<MouseEvent>,
>(
    node: &TreeNode,
    path_prefix: &str,
    selected_path: &UseStateHandle<Option<String>>,
    on_page_selected: &Callback<PageMeta>,
    make_on_move: &F,
    make_on_new_page_in: &G,
    make_on_export: &H,
    collapsed_folders: &UseStateHandle<HashSet<String>>,
    nav_active: &Option<String>,
    nav_item_ref: &NodeRef,
) -> Html {
    html! {
        <>
            { for node.folders.iter().map(|(name, sub)| {
                let full_path = format!("{}/{}", path_prefix, name);
                let on_new = make_on_new_page_in(full_path.clone());
                let on_export = make_on_export(full_path.clone());
                let is_open = !collapsed_folders.contains(&full_path);
                let is_nav_active = nav_active.as_deref() == Some(full_path.as_str());
                let summary_class = if is_nav_active {
                    "sidebar-folder__header sidebar-folder__header--nav-active"
                } else {
                    "sidebar-folder__header"
                };
                let summary_ref = if is_nav_active { nav_item_ref.clone() } else { NodeRef::default() };
                let ontoggle = {
                    let collapsed_folders = collapsed_folders.clone();
                    let full_path = full_path.clone();
                    Callback::from(move |e: Event| {
                        let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlDetailsElement>().ok()) else { return };
                        let mut c = (*collapsed_folders).clone();
                        if target.open() { c.remove(&full_path); } else { c.insert(full_path.clone()); }
                        collapsed_folders.set(c);
                    })
                };
                html! {
                    <details class="sidebar-folder" open={is_open} {ontoggle}>
                        <summary class={summary_class} ref={summary_ref}>
                            <span class="sidebar-folder__icon">{ "📁" }</span>
                            <span class="sidebar-folder__name">{ name }</span>
                            <button class="btn btn--ghost btn--xs" title="Nova página nesta pasta" onclick={on_new}>{ "+" }</button>
                            <button class="btn btn--ghost btn--xs" title="Exportar pasta" onclick={on_export}>{ "⬇" }</button>
                        </summary>
                        <div class="sidebar-folder__body">
                            { render_tree(sub, &full_path, selected_path, on_page_selected, make_on_move, make_on_new_page_in, make_on_export, collapsed_folders, nav_active, nav_item_ref) }
                        </div>
                    </details>
                }
            }) }
            { render_movable_list(&node.pages, selected_path, on_page_selected, make_on_move, nav_active, nav_item_ref) }
        </>
    }
}

/// Converte os marcadores `**termo**` que `search_content` usa pra
/// indicar o trecho que casou com a busca (via `snippet()` do FTS5, ver
/// `crates/search`) em `<strong>` de verdade — sem isso o usuário via
/// os asteriscos literais em vez de destaque visual.
/// Extraído aqui (não é específico da sidebar) pra ser reusado pela
/// paleta de comandos (ciclo 102), que também mostra trechos de
/// `search_content` com o termo destacado.
pub(crate) fn render_excerpt_highlight(excerpt: &str) -> Html {
    let parts: Vec<&str> = excerpt.split("**").collect();
    html! {
        <>
            { for parts.iter().enumerate().map(|(i, part)| {
                if i % 2 == 1 {
                    html! { <strong>{ *part }</strong> }
                } else {
                    html! { { *part } }
                }
            }) }
        </>
    }
}

fn page_icon(section: &str) -> &'static str {
    match section {
        "journals" => "📅",
        "search" => "🔍",
        _ => "📄",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(path: &str) -> PageMeta {
        let title = std::path::Path::new(path).file_stem().unwrap().to_string_lossy().to_string();
        PageMeta { path: path.to_string(), title, section: "pages".to_string() }
    }

    #[test]
    fn flatten_nav_orders_folders_before_pages_alphabetically() {
        let pages = vec![page("pages/alpha.md"), page("pages/trabalho/nota.md")];
        let folders = vec!["pages/trabalho".to_string()];
        let tree = build_tree(&pages, &folders);
        let mut out = Vec::new();
        flatten_nav(&tree, "pages", &HashSet::new(), 0, &mut out);
        let keys: Vec<&str> = out.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(keys, vec!["pages/trabalho", "pages/trabalho/nota.md", "pages/alpha.md"]);
    }

    #[test]
    fn flatten_nav_skips_children_of_collapsed_folder() {
        let pages = vec![page("pages/trabalho/nota.md")];
        let folders = vec!["pages/trabalho".to_string()];
        let tree = build_tree(&pages, &folders);
        let mut collapsed = HashSet::new();
        collapsed.insert("pages/trabalho".to_string());
        let mut out = Vec::new();
        flatten_nav(&tree, "pages", &collapsed, 0, &mut out);
        let keys: Vec<&str> = out.iter().map(|i| i.key.as_str()).collect();
        assert_eq!(keys, vec!["pages/trabalho"]);
    }

    #[test]
    fn flatten_nav_tracks_parent_for_nested_items() {
        let pages = vec![page("pages/trabalho/nota.md")];
        let folders = vec!["pages/trabalho".to_string()];
        let tree = build_tree(&pages, &folders);
        let mut out = Vec::new();
        flatten_nav(&tree, "pages", &HashSet::new(), 0, &mut out);
        let folder_item = out.iter().find(|i| i.key == "pages/trabalho").unwrap();
        assert_eq!(folder_item.parent, None);
        let page_item = out.iter().find(|i| i.key == "pages/trabalho/nota.md").unwrap();
        assert_eq!(page_item.parent.as_deref(), Some("pages/trabalho"));
    }

    #[test]
    fn flatten_nav_root_level_page_has_no_parent() {
        let pages = vec![page("pages/alpha.md")];
        let tree = build_tree(&pages, &[]);
        let mut out = Vec::new();
        flatten_nav(&tree, "pages", &HashSet::new(), 0, &mut out);
        assert_eq!(out[0].parent, None);
    }
}
