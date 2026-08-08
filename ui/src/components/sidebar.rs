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

use std::collections::BTreeMap;

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
                <div class="sidebar-section">
                    <div class="sidebar-section__header">
                        <h3 class="sidebar-section__title">{ "Pages" }</h3>
                        <button class="btn btn--ghost btn--xs" title="Nova página inicial (landing)" onclick={on_new_landing}>{ "🏠+" }</button>
                        <button class="btn btn--ghost btn--xs" title="Nova pasta" onclick={on_new_folder}>{ "📁+" }</button>
                        <button class="btn btn--ghost btn--xs" title="Nova página" onclick={on_new_page}>{ "+" }</button>
                    </div>
                    if page_items.is_empty() {
                        <p class="sidebar-section__empty">{ "Nenhuma página ainda" }</p>
                    } else if filter.is_empty() {
                        { render_tree(&build_tree(&page_items, &folders), "pages", &selected_path, &props.on_page_selected, &make_on_move_page, &make_on_new_page_in, &make_on_export) }
                    } else {
                        { render_movable_list(&page_items, &selected_path, &props.on_page_selected, &make_on_move_page) }
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
                let on_move = make_on_move(path.clone());
                html! {
                    <li {class} {onclick}>
                        <span class="sidebar-item__icon">{ page_icon(&page.section) }</span>
                        <span class="sidebar-item__title">{ &title }</span>
                        <button class="sidebar-item__move btn btn--ghost btn--xs" title="Mover pra pasta" onclick={on_move}>{ "📁" }</button>
                    </li>
                }
            }) }
        </ul>
    }
}

/// Renderiza a árvore de pastas recursivamente — pastas primeiro
/// (`<details>` nativo, aberto por padrão, dá o expandir/colapsar de
/// graça sem precisar de estado extra), depois as páginas do nível
/// atual.
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
) -> Html {
    html! {
        <>
            { for node.folders.iter().map(|(name, sub)| {
                let full_path = format!("{}/{}", path_prefix, name);
                let on_new = make_on_new_page_in(full_path.clone());
                let on_export = make_on_export(full_path.clone());
                html! {
                    <details class="sidebar-folder" open=true>
                        <summary class="sidebar-folder__header">
                            <span class="sidebar-folder__icon">{ "📁" }</span>
                            <span class="sidebar-folder__name">{ name }</span>
                            <button class="btn btn--ghost btn--xs" title="Nova página nesta pasta" onclick={on_new}>{ "+" }</button>
                            <button class="btn btn--ghost btn--xs" title="Exportar pasta" onclick={on_export}>{ "⬇" }</button>
                        </summary>
                        <div class="sidebar-folder__body">
                            { render_tree(sub, &full_path, selected_path, on_page_selected, make_on_move, make_on_new_page_in, make_on_export) }
                        </div>
                    </details>
                }
            }) }
            { render_movable_list(&node.pages, selected_path, on_page_selected, make_on_move) }
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
