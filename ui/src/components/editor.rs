//! Editor WYSIWYG contenteditable + slash commands + markdown live formatting.

use base64::Engine;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use web_sys::KeyboardEvent;

use crate::api::{self, PageMeta};
use crate::components::embeds::InlineEmbed;
use crate::components::modal::Modal;
use crate::components::properties_panel::PropertiesPanel;
use crate::dialog::PendingDialog;
use crate::embed::DocSegment;
use crate::state;

#[derive(Properties, PartialEq, Clone)]
pub struct EditorProps {
    pub vault_path: String,
    pub page: Option<PageMeta>,
    #[prop_or_default]
    pub on_page_deleted: Callback<()>,
    /// Abre o modal de diálogo do app (ver `crate::dialog`).
    pub open_dialog: Callback<PendingDialog>,
    /// Navega pra outra página do vault — usado pela célula de tipo
    /// Página do embed de tabela.
    #[prop_or_default]
    pub on_page_selected: Callback<PageMeta>,
    /// Se falso, edições só marcam "não salvo" — sem agendar o save
    /// automático após alguns segundos de inatividade. O flush ao trocar
    /// de página (evitar perder edições) continua acontecendo de qualquer
    /// jeito, independente disso — essa flag só controla a conveniência
    /// de salvar sozinho enquanto o usuário ainda está na página.
    #[prop_or(true)]
    pub autosave_enabled: bool,
    /// Se o vim mode (modal Normal/Insert) está ativado.
    #[prop_or(false)]
    pub vim_mode_enabled: bool,
    /// Mapa de teclas do vim mode.
    #[prop_or_default]
    pub vim_keymap: crate::state::VimKeymap,
    /// Ação disparada de fora via `GlobalKeymap` (ciclo 105) — `Some`
    /// com um nonce (pra disparar de novo mesmo se a mesma ação repetir
    /// em sequência) quando o app quer Salvar/Desfazer/Refazer sem o
    /// foco estar dentro do contenteditable (que já trata Ctrl+S/Ctrl+Z
    /// localmente, sem passar por aqui).
    #[prop_or_default]
    pub global_action: Option<(crate::state::GlobalEditorAction, u32)>,
    /// Path da página inicial do vault (ciclo 089; estado vive no
    /// `App`, ver ciclo 109 — a `TabBar`, irmã deste componente,
    /// também precisa saber pra mostrar o ícone na aba fixa).
    #[prop_or_default]
    pub home_page: Option<String>,
    /// Alterna a página atual como inicial (define/remove).
    #[prop_or_default]
    pub on_toggle_home: Callback<String>,
}

struct SlashItem {
    label: &'static str,
    desc: &'static str,
    html: &'static str,
}

static SLASH_ITEMS: &[SlashItem] = &[
    SlashItem { label: "Título 1", desc: "Título grande", html: "<h1>Título</h1>" },
    SlashItem { label: "Título 2", desc: "Título médio", html: "<h2>Título</h2>" },
    SlashItem { label: "Título 3", desc: "Título pequeno", html: "<h3>Título</h3>" },
    SlashItem { label: "Lista", desc: "Lista com marcadores", html: "<ul><li>Item</li></ul>" },
    SlashItem { label: "Checklist", desc: "Lista de tarefas", html: "<div><input type='checkbox'> Tarefa</div>" },
    SlashItem { label: "Citação", desc: "Bloco de citação", html: "<blockquote>Citação</blockquote>" },
    SlashItem { label: "Código", desc: "Bloco de código", html: "<pre><code>código</code></pre>" },
    SlashItem { label: "Tabela", desc: "Tabela 3×2", html: "<table><tr><td>A</td><td>B</td><td>C</td></tr><tr><td></td><td></td><td></td></tr></table>" },
    SlashItem { label: "Linha", desc: "Divisor horizontal", html: "<hr>" },
    SlashItem { label: "Imagem", desc: "URL ou arquivo de imagem", html: "__IMG__" },
    SlashItem { label: "Diagrama", desc: "Mermaid (fluxograma)", html: "__MERMAID__" },
    SlashItem { label: "Assets", desc: "Inserir arquivo do vault", html: "__ASSET__" },
    SlashItem { label: "Kanban", desc: "Board kanban interativo", html: "__EMBED_KANBAN__" },
    SlashItem { label: "Calendário", desc: "Lista de eventos por data", html: "__EMBED_CALENDAR__" },
    SlashItem { label: "Tabela de Tarefas", desc: "Tabela com colunas tipadas (embed)", html: "__EMBED_TABLE__" },
];

#[function_component(Editor)]
pub fn editor(props: &EditorProps) -> Html {
    let content_md = use_state(String::new);
    let saved_content = use_state(String::new);
    let loading = use_state(|| false);
    let saving = use_state(|| false);
    let error = use_state(|| None::<String>);
    let status = use_state(|| None::<String>);
    let edited = use_state(|| false);
    let editor_ref = use_node_ref();
    // `edited_ref`/`pending_flush_ref`: espelham `edited`/o markdown mais
    // recente fora do ciclo de render do Yew (`use_mut_ref` devolve o
    // MESMO `Rc<RefCell<_>>` em toda renderização — diferente de
    // `use_state`, cujo handle capturado por um efeito antigo fica
    // congelado no valor de quando foi criado). Precisamos disso porque o
    // flush-ao-trocar-de-página roda dentro do cleanup do efeito que
    // observa `props.page`, criado só uma vez por página — se ele lesse
    // `*edited`/`*content_md` diretamente, sempre veria os valores de
    // quando a página foi carregada (sempre `false`/vazio), nunca as
    // edições feitas depois.
    let edited_ref = use_mut_ref(|| false);
    let pending_flush_ref = use_mut_ref(String::new);

    // Undo/redo genérico: pilha de snapshots de markdown inteiro
    // (cap. ~20), não um mecanismo por tipo de embed — cobre texto solto
    // E qualquer mutação de embed (mover card, editar evento, etc) com
    // uma implementação só, já que TODA mutação passa por `mark_edited`
    // (ponto único desde o ciclo 074). `last_content_ref` guarda o
    // último markdown que `mark_edited` viu — não é o mesmo que
    // `content_md` (que nem sempre é atualizado em sync, ver
    // `on_edit`), é a base de comparação certa pra decidir quando
    // empilhar um novo snapshot (agrupa digitação rápida numa pausa só,
    // em vez de um snapshot por tecla). `render_gen` força o Effect 2
    // (abaixo) a reinjetar o HTML mesmo quando path/has_embeds/
    // segment_count não mudaram — sem isso, desfazer/refazer atualizava
    // `content_md` (embeds declarativos refletiam certo) mas os trechos
    // de markdown solto injetados via `set_inner_html` ficavam com o
    // texto antigo na tela.
    let undo_stack = use_mut_ref(Vec::<String>::new);
    let redo_stack = use_mut_ref(Vec::<String>::new);
    let last_content_ref = use_mut_ref(String::new);
    let last_snapshot_at = use_mut_ref(|| 0.0f64);
    let render_gen = use_state(|| 0u32);

    // Vim mode: modo Normal (motions/comandos) vs Insert (digitação
    // normal). Começa em Normal quando ativado — mesmo comportamento do
    // vim de verdade (abrir um arquivo não te deixa digitando na hora).
    // `vim_register` (yy/p) e `vim_pending` (confirmação de dd/yy — a
    // tecla configurada precisa ser pressionada 2x seguidas) usam
    // `use_mut_ref` por serem lidos/escritos de dentro do handler de
    // teclado sem precisar disparar re-render a cada tecla motion.
    let vim_insert = use_state(|| false);
    let vim_register = use_mut_ref(String::new);
    let vim_pending = use_mut_ref(|| None::<String>);

    let slash_open = use_state(|| false);
    let slash_text = use_state(String::new);
    let slash_idx = use_state(|| 0usize);
    let slash_active_ref = use_node_ref();

    // Popup de autocomplete de wikilink (`[[Título`) — mesmo mecanismo do
    // menu `/`: abre/atualiza olhando o texto de verdade a cada `oninput`
    // (`find_wikilink_context`), navega com teclado, aplica via `Range`.
    let wikilink_open = use_state(|| false);
    let wikilink_text = use_state(String::new);
    let wikilink_idx = use_state(|| 0usize);
    let wikilink_active_ref = use_node_ref();
    let wikilink_pages = use_state(Vec::<PageMeta>::new);

    // Rola a lista pra manter o item ativo visível ao navegar com o
    // teclado — sem isso, se o item selecionado saísse da área visível do
    // menu (scrollável), a navegação continuava funcionando mas o usuário
    // não via qual item estava ativo.
    {
        let slash_active_ref = slash_active_ref.clone();
        use_effect_with((*slash_idx, *slash_open), move |_| {
            if let Some(el) = slash_active_ref.cast::<web_sys::Element>() {
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&opts);
            }
            || {}
        });
    }

    // Fecha o menu de slash ao clicar fora dele — sem isso ele ficava aberto
    // pra sempre se o usuário clicasse na sidebar ou em outro lugar da
    // página em vez de Escape/selecionar um item.
    {
        let slash_open = slash_open.clone();
        let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        use_effect_with(*slash_open, move |open| {
            let listener = if *open {
                let window = web_sys::window().expect("no global window");
                Some(EventListener::new(&window, "mousedown", move |e| {
                    let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                    let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                    let Some(target) = target else { return };
                    if target.closest(".editor__wysiwyg, .slash-menu").ok().flatten().is_none() {
                        slash_open.set(false);
                        slash_text.set(String::new());
                        slash_idx.set(0);
                    }
                }))
            } else {
                None
            };
            move || drop(listener)
        });
    }

    // Mesmo scroll-into-view do menu `/`, pro popup de wikilink.
    {
        let wikilink_active_ref = wikilink_active_ref.clone();
        use_effect_with((*wikilink_idx, *wikilink_open), move |_| {
            if let Some(el) = wikilink_active_ref.cast::<web_sys::Element>() {
                let opts = web_sys::ScrollIntoViewOptions::new();
                opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
                el.scroll_into_view_with_scroll_into_view_options(&opts);
            }
            || {}
        });
    }

    // Mesmo fechar-ao-clicar-fora do menu `/`, pro popup de wikilink.
    {
        let wikilink_open = wikilink_open.clone();
        let wikilink_text = wikilink_text.clone();
        let wikilink_idx = wikilink_idx.clone();
        use_effect_with(*wikilink_open, move |open| {
            let listener = if *open {
                let window = web_sys::window().expect("no global window");
                Some(EventListener::new(&window, "mousedown", move |e| {
                    let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                    let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                    let Some(target) = target else { return };
                    if target.closest(".editor__wysiwyg, .wikilink-menu").ok().flatten().is_none() {
                        wikilink_open.set(false);
                        wikilink_text.set(String::new());
                        wikilink_idx.set(0);
                    }
                }))
            } else {
                None
            };
            move || drop(listener)
        });
    }

    // Busca a lista de páginas do vault sempre que o popup abre — dado
    // simples e barato de buscar de novo (não vale a pena manter em
    // cache num campo à parte só pra isso).
    {
        let vault_path = props.vault_path.clone();
        let wikilink_pages = wikilink_pages.clone();
        use_effect_with(*wikilink_open, move |open| {
            if *open {
                let vault_path = vault_path.clone();
                let wikilink_pages = wikilink_pages.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    if let Ok(pages) = api::list_pages(&vault_path).await {
                        wikilink_pages.set(pages);
                    }
                });
            }
            || {}
        });
    }

    let filtered: Vec<usize> = SLASH_ITEMS.iter().enumerate()
        .filter(|(_, item)| {
            let q = slash_text.to_lowercase();
            q.is_empty() || item.label.to_lowercase().contains(&q) || item.desc.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();

    let filtered_wikilink: Vec<usize> = wikilink_pages.iter().enumerate()
        .filter(|(_, p)| {
            let q = wikilink_text.to_lowercase();
            q.is_empty() || p.title.to_lowercase().contains(&q)
        })
        .map(|(i, _)| i)
        .collect();

    let char_count = content_md.chars().count();
    let word_count = content_md.split_whitespace().count();

    // Segmenta o corpo (sem frontmatter) em markdown comum + embeds
    // (kanban/calendar/table) já parseados. Recalculado a cada render —
    // é uma varredura barata (pulldown-cmark) sobre o texto da página.
    // Trechos de markdown continuam no fluxo contenteditable de sempre;
    // embeds viram componentes Yew reais fora dele (ver InlineEmbed).
    let full_snapshot = (*content_md).clone();
    let (frontmatter_text, body_text) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full_snapshot);
    let frontmatter_text = frontmatter_text.to_string();
    let segments: Vec<DocSegment> = crate::embed::segment(body_text);
    let has_embeds = segments.iter().any(|s| matches!(s, DocSegment::Embed(_)));
    let segment_refs: Vec<NodeRef> = (0..segments.len()).map(|_| NodeRef::default()).collect();

    // Painel de propriedades (ciclo 099): parseia o frontmatter cru pro
    // tipo estruturado (`Frontmatter`, com `extra` do ciclo 098) só pra
    // ALIMENTAR o painel — `on_frontmatter_change` (definido mais abaixo,
    // depois de `mark_edited` existir) é quem escreve de volta.
    let parsed_frontmatter: anotadinho_core::Frontmatter =
        anotadinho_core::MarkdownCodec::split_frontmatter(&frontmatter_text)
            .map(|(fm, _)| fm)
            .unwrap_or_default();

    // Effect 1: fetch page content when page changes
    {
        let content_md = content_md.clone();
        let page = props.page.clone();
        let vault_path = props.vault_path.clone();
        let saved_content = saved_content.clone();
        let loading = loading.clone();
        let error = error.clone();
        let edited = edited.clone();
        let edited_ref = edited_ref.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        let undo_stack = undo_stack.clone();
        let redo_stack = redo_stack.clone();
        let last_content_ref = last_content_ref.clone();

        use_effect_with(page.clone(), move |page| {
            // Histórico de undo/redo é por página — trocar de página não
            // deveria deixar "desfazer" aplicar uma edição de outra
            // página bem diferente.
            undo_stack.borrow_mut().clear();
            redo_stack.borrow_mut().clear();
            // Página que ESTE efeito está carregando — usada no cleanup
            // como identidade da página que está sendo deixada pra trás
            // (o cleanup roda antes do próximo efeito, ou seja, exatamente
            // no momento da troca).
            let leaving_page = page.clone();
            let flush_vault_path = vault_path.clone();

            if let Some(p) = page {
                let vault_path = vault_path.clone();
                let path = p.path.clone();
                let content_md = content_md.clone();
                let saved_content = saved_content.clone();
                let loading = loading.clone();
                let error = error.clone();
                let edited = edited.clone();
                let edited_ref = edited_ref.clone();
                let pending_flush_ref = pending_flush_ref.clone();
                let last_content_ref = last_content_ref.clone();
                loading.set(true);
                error.set(None);
                edited.set(false);
                *edited_ref.borrow_mut() = false;
                pending_flush_ref.borrow_mut().clear();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page(&vault_path, &path).await {
                        Ok(text) => {
                            *last_content_ref.borrow_mut() = text.clone();
                            content_md.set(text.clone());
                            saved_content.set(text);
                        }
                        Err(e) => { error.set(Some(e)); }
                    }
                    loading.set(false);
                });
            } else {
                content_md.set(String::new());
                saved_content.set(String::new());
                error.set(None);
                edited.set(false);
                *edited_ref.borrow_mut() = false;
                pending_flush_ref.borrow_mut().clear();
                loading.set(false);
            }

            // Flush de segurança: se a página que está sendo deixada tinha
            // edições pendentes (marcadas via `edited_ref`/
            // `pending_flush_ref`, que — diferente de `edited`/
            // `content_md` — refletem o valor mais recente mesmo lido de
            // dentro de um efeito criado só uma vez), salva antes de
            // trocar. Sem isso, editar rápido e clicar noutra página na
            // sidebar descartava o texto digitado silenciosamente.
            move || {
                if *edited_ref.borrow() {
                    if let Some(p) = leaving_page {
                        let md = pending_flush_ref.borrow().clone();
                        if !md.is_empty() {
                            let vp = flush_vault_path.clone();
                            wasm_bindgen_futures::spawn_local(async move {
                                let _ = api::write_page(&vp, &p.path, &md).await;
                            });
                        }
                    }
                }
            }
        });
    }

    // Effect 2: set innerHTML only once when content loads.
    // Sem embeds: injeta a página inteira num único contenteditable, como
    // sempre foi. Com embeds: injeta cada trecho de markdown no seu próprio
    // contenteditable (via segment_refs) — os embeds em si já são
    // componentes Yew declarativos, não precisam de injeção imperativa.
    //
    // O guard rastreia (path, has_embeds, contagem de segmentos), não só o
    // path: sem isso, inserir o primeiro embed numa página via slash
    // command (transição has_embeds false→true na mesma sessão, sem trocar
    // de página) nunca repovoaria os trechos de markdown recém-criados —
    // eles ficariam com innerHTML vazio. Digitar texto normal não muda
    // has_embeds nem a contagem de segmentos, então isso não reintroduz o
    // loop infinito do ciclo 043 (que reagia a toda mudança de content_md).
    {
        let loading_val = *loading;
        let content_md_empty = content_md.is_empty();
        let editor_ref = editor_ref.clone();
        let segment_refs_eff = segment_refs.clone();
        let segments_eff = segments.clone();
        let full_snapshot_eff = full_snapshot.clone();
        let has_embeds_eff = has_embeds;
        let segment_count = segments.len();
        let last_rendered = use_mut_ref(|| (String::new(), false, 0usize, 0u32));
        let current_path = props.page.as_ref().map(|p| p.path.clone()).unwrap_or_default();
        let render_gen_val = *render_gen;
        let vault_path_eff = props.vault_path.clone();

        use_effect_with((loading_val, current_path.clone(), has_embeds_eff, segment_count, render_gen_val), move |_| {
            let should_render = {
                let last = last_rendered.borrow();
                !loading_val && !content_md_empty
                    && (last.0 != current_path || last.1 != has_embeds_eff || last.2 != segment_count || last.3 != render_gen_val)
            };
            if should_render {
                *last_rendered.borrow_mut() = (current_path, has_embeds_eff, segment_count, render_gen_val);

                if has_embeds_eff {
                    for (i, seg) in segments_eff.iter().enumerate() {
                        if let DocSegment::Markdown(text) = seg {
                            if let Some(div) = segment_refs_eff.get(i).and_then(|r| r.cast::<web_sys::Element>()) {
                                div.set_inner_html(&crate::markdown_render::render(text));
                                upgrade_embedded_assets_at(&div, vault_path_eff.clone());
                            }
                        }
                    }
                    for r in segment_refs_eff.iter() {
                        if let Some(el) = r.cast::<web_sys::Element>() {
                            wasm_bindgen_futures::spawn_local(async move {
                                gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                                init_mermaid_at(&el);
                            });
                        }
                    }
                } else if let Some(div) = editor_ref.cast::<web_sys::Element>() {
                    let html = crate::markdown_render::render(&full_snapshot_eff);
                    div.set_inner_html(&html);
                    upgrade_embedded_assets_at(&div, vault_path_eff.clone());
                    let _div = div.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                        init_mermaid_at(&_div);
                    });
                }
                init_highlight();
            }
            || {}
        });
    }

    // Backlinks: quais páginas têm `[[Título desta página]]`. Calculado
    // sob demanda ao abrir a página (não é um índice mantido — mesmo
    // perfil de custo da busca ingênua já existente), reaproveitando
    // `search_content` como "grep" no servidor em vez de expor uma rota
    // nova só pra isso: procurar pela sintaxe `[[Título]]` literal já
    // acha exatamente os wikilinks que apontam pra cá.
    let backlinks = use_state(Vec::<(String, String)>::new);
    {
        let vault_path = props.vault_path.clone();
        let page = props.page.clone();
        let backlinks = backlinks.clone();
        use_effect_with(page.clone(), move |page| {
            if let Some(p) = page.clone() {
                let vault_path = vault_path.clone();
                let backlinks = backlinks.clone();
                let current_path = p.path.clone();
                let query = format!("[[{}]]", p.title);
                wasm_bindgen_futures::spawn_local(async move {
                    match api::search_content(&vault_path, &query).await {
                        Ok(results) => {
                            let filtered: Vec<(String, String)> = results
                                .into_iter()
                                .filter(|(path, _)| path != &current_path)
                                .collect();
                            backlinks.set(filtered);
                        }
                        Err(_) => backlinks.set(Vec::new()),
                    }
                });
            } else {
                backlinks.set(Vec::new());
            }
            || {}
        });
    }

    let save_counter = use_state(|| 0u32);

    if props.page.is_none() {
        return html! { <main class="app-main"><p class="app-main__placeholder">{ "Selecione uma página na sidebar" }</p></main> };
    }

    let page = props.page.as_ref().unwrap().clone();

    // "Definir como início" — página aberta automaticamente ao abrir este
    // vault (ver `App`). Estado vive no `App` desde o ciclo 109 (a
    // `TabBar`, irmã deste componente, também precisa saber qual página
    // é a inicial) — aqui é só derivado do prop, sem estado próprio.
    let is_home = props.home_page.as_deref() == Some(page.path.as_str());
    let toggle_home = {
        let on_toggle_home = props.on_toggle_home.clone();
        let path = page.path.clone();
        Callback::from(move |_: MouseEvent| on_toggle_home.emit(path.clone()))
    };

    // Menu "⋯" do header (ciclo 109) — agrupa Definir início/Propriedades/
    // Exportar/Excluir, que antes eram botões soltos. Mesmo padrão visual
    // e de fechar-ao-clicar-fora/Escape do menu ⚙ da `HeaderBar`.
    let editor_menu_open = use_state(|| false);
    let editor_menu_ref = use_node_ref();
    let toggle_editor_menu = { let m = editor_menu_open.clone(); Callback::from(move |_| m.set(!*m)) };
    {
        let editor_menu_open = editor_menu_open.clone();
        let editor_menu_ref = editor_menu_ref.clone();
        use_effect_with(*editor_menu_open, move |open| {
            let mut listeners = Vec::new();
            if *open {
                let window = web_sys::window().expect("no global window");
                let close_on_outside = {
                    let editor_menu_open = editor_menu_open.clone();
                    let editor_menu_ref = editor_menu_ref.clone();
                    EventListener::new(&window, "mousedown", move |e| {
                        let Some(target) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                        if let Some(el) = editor_menu_ref.cast::<web_sys::Element>() {
                            if !el.contains(Some(&target)) {
                                editor_menu_open.set(false);
                            }
                        }
                    })
                };
                let close_on_escape = {
                    let editor_menu_open = editor_menu_open.clone();
                    EventListener::new(&window, "keydown", move |e| {
                        if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                            if e.key() == "Escape" {
                                editor_menu_open.set(false);
                            }
                        }
                    })
                };
                listeners.push(close_on_outside);
                listeners.push(close_on_escape);
            }
            move || drop(listeners)
        });
    }
    let properties_modal_open = use_state(|| false);

    let doc_exec = {
        let edited = edited.clone();
        let doc = web_sys::window().and_then(|w| w.document());
        move |cmd: &str, val: &str| {
            if let Some(ref doc) = doc {
                exec_cmd(doc, cmd, val);
            }
            edited.set(true);
        }
    };

    // `persist`: grava `md` (já calculado, recebido por valor) no disco e
    // atualiza o estado de "salvo" — compartilhado pelo clique manual em
    // "Salvar" e pelo autosave debounced. Recebe o markdown JÁ PRONTO em
    // vez de reler `content_md` de dentro do `spawn_local`: um
    // `UseStateHandle` capturado numa closure não vê `.set()` chamado por
    // OUTRO clone do mesmo handle (cada clone fica congelado no valor de
    // quando foi criado) — se o autosave debounced relesse `content_md`
    // só na hora de persistir, qualquer edição de embed feita
    // logo antes (que já tinha chamado `content_md.set()` mas cujo efeito
    // só aparece na PRÓXIMA renderização) seria perdida silenciosamente.
    // Passando `md` como valor isso não acontece: o autosave sempre grava
    // exatamente o que foi calculado no momento da edição.
    let persist = {
        let content_md = content_md.clone(); let saved_content = saved_content.clone();
        let saving = saving.clone(); let error = error.clone(); let status = status.clone();
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let edited = edited.clone();
        let edited_ref = edited_ref.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        move |md: String| {
            let saved_content = saved_content.clone(); let saving = saving.clone();
            let error = error.clone(); let status = status.clone();
            let vault_path = vault_path.clone(); let page_path = page_path.clone();
            let content_md = content_md.clone(); let edited = edited.clone();
            let edited_ref = edited_ref.clone();
            let pending_flush_ref = pending_flush_ref.clone();
            saving.set(true); error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                match api::write_page(&vault_path, &page_path, &md).await {
                    Ok(()) => {
                        content_md.set(md.clone()); saved_content.set(md); edited.set(false);
                        *edited_ref.borrow_mut() = false; pending_flush_ref.borrow_mut().clear();
                        status.set(Some("Salvo".to_string()));
                    }
                    Err(e) => { error.set(Some(e)); }
                }
                saving.set(false);
            });
        }
    };

    let do_save = {
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let saving = saving.clone();
        let persist = persist.clone();
        Callback::from(move |_| {
            if *saving { return; }
            let md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
            persist(md);
        })
    };

    // mark_edited: marca como editado e agenda um save daqui a 3s
    // (cancelado se outra edição chegar antes) — recebe o markdown JÁ
    // CALCULADO em vez de recalcular de dentro do `spawn_local` (mesmo
    // motivo do `persist`: evita persistir um valor desatualizado se
    // `content_md` tiver sido atualizado por uma edição de embed bem
    // antes do timer disparar). `on_edit` (ligado ao `oninput` do
    // contenteditable) recalcula a partir do DOM ao vivo antes de chamar
    // — sempre correto pra texto puro, já que a fonte de verdade ali é o
    // DOM, não `content_md`. Embeds chamam com o markdown que ELES já
    // calcularam (`new_full`), sem depender de `content_md` nenhuma.
    let mark_edited = {
        let e = edited.clone();
        let save_counter = save_counter.clone();
        let edited_ref = edited_ref.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        let autosave_enabled = props.autosave_enabled;
        let persist = persist.clone();
        let undo_stack = undo_stack.clone();
        let redo_stack = redo_stack.clone();
        let last_content_ref = last_content_ref.clone();
        let last_snapshot_at = last_snapshot_at.clone();
        move |md: String| {
            e.set(true);
            *edited_ref.borrow_mut() = true;
            // Mantém o flush de segurança sempre atualizado, independente
            // do salvamento automático estar ligado — isso é o que evita
            // perder texto ao trocar de página rápido, não o timer de 3s.
            *pending_flush_ref.borrow_mut() = md.clone();

            // Empilha o markdown ANTERIOR pro undo — só se já passou um
            // tempinho desde o último snapshot (agrupa uma rajada de
            // digitação numa pausa só num único passo de "desfazer", em
            // vez de um passo por tecla). `last_content_ref` (não
            // `content_md`) é a base porque `content_md` nem sempre é
            // atualizado em sync com toda edição (ver `on_edit`).
            let now = js_sys::Date::now();
            let previous = last_content_ref.borrow().clone();
            if previous != md {
                let elapsed = now - *last_snapshot_at.borrow();
                if elapsed > 800.0 {
                    let mut stack = undo_stack.borrow_mut();
                    stack.push(previous);
                    if stack.len() > 20 {
                        stack.remove(0);
                    }
                    redo_stack.borrow_mut().clear();
                    *last_snapshot_at.borrow_mut() = now;
                }
                *last_content_ref.borrow_mut() = md.clone();
            }

            if !autosave_enabled {
                return;
            }
            let save_counter = save_counter.clone();
            let id = *save_counter + 1;
            save_counter.set(id);
            let persist = persist.clone();
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::sleep(std::time::Duration::from_secs(3)).await;
                if *save_counter == id {
                    persist(md);
                }
            });
        }
    };

    // Painel de propriedades (ciclo 099): único lugar do editor que
    // edita frontmatter de verdade — o resto do editor sempre preservou
    // o bloco de frontmatter cru, intocado, ao salvar (ver `persist`/
    // `recompute_markdown_from_dom`). Serializa `Frontmatter` de volta
    // pra YAML e reconstrói `content_md` com o MESMO corpo (`body_text`)
    // intocado — mesmo formato de bloco (`---\n...\n---`, sem newline
    // final) que `split_frontmatter_text` espera.
    let on_frontmatter_change = {
        let body_text = body_text.to_string();
        let content_md = content_md.clone();
        let mark_edited = mark_edited.clone();
        Callback::from(move |new_fm: anotadinho_core::Frontmatter| {
            let yaml = serde_yaml::to_string(&new_fm).unwrap_or_default();
            let mut block = String::from("---\n");
            block.push_str(yaml.trim_start_matches("---\n"));
            if !block.ends_with('\n') {
                block.push('\n');
            }
            block.push_str("---");
            let new_full = format!("{}\n{}", block, body_text);
            content_md.set(new_full.clone());
            mark_edited(new_full);
        })
    };

    // `Ctrl+Z`/`Ctrl+Shift+Z` — desfazer/refazer genérico (texto solto E
    // qualquer mutação de embed, mesma pilha pras duas coisas). Atualiza
    // `content_md` na hora (feedback imediato) + `render_gen` (força o
    // Effect 2 a reinjetar os trechos de markdown solto, que não
    // reagem sozinhos a `content_md` mudar) + `persist` (grava a versão
    // restaurada, mesmo caminho do salvamento normal).
    let do_undo = {
        let content_md = content_md.clone();
        let undo_stack = undo_stack.clone();
        let redo_stack = redo_stack.clone();
        let last_content_ref = last_content_ref.clone();
        let render_gen = render_gen.clone();
        let persist = persist.clone();
        Callback::from(move |_: ()| {
            let popped = undo_stack.borrow_mut().pop();
            let Some(prev) = popped else { return };
            let current = last_content_ref.borrow().clone();
            redo_stack.borrow_mut().push(current);
            *last_content_ref.borrow_mut() = prev.clone();
            content_md.set(prev.clone());
            render_gen.set(*render_gen + 1);
            persist(prev);
        })
    };
    let do_redo = {
        let content_md = content_md.clone();
        let undo_stack = undo_stack.clone();
        let redo_stack = redo_stack.clone();
        let last_content_ref = last_content_ref.clone();
        let render_gen = render_gen.clone();
        let persist = persist.clone();
        Callback::from(move |_: ()| {
            let popped = redo_stack.borrow_mut().pop();
            let Some(next) = popped else { return };
            let current = last_content_ref.borrow().clone();
            undo_stack.borrow_mut().push(current);
            *last_content_ref.borrow_mut() = next.clone();
            content_md.set(next.clone());
            render_gen.set(*render_gen + 1);
            persist(next);
        })
    };

    // Ponte do GlobalKeymap (ciclo 105): quando `App` dispara
    // Salvar/Desfazer/Refazer sem o foco estar no contenteditable
    // (onde Ctrl+S/Ctrl+Z já funcionam direto), reage aqui. O nonce no
    // prop garante que o efeito dispara de novo mesmo pra ações
    // repetidas em sequência (`Option` sozinho não mudaria de valor).
    {
        let do_save = do_save.clone();
        let do_undo = do_undo.clone();
        let do_redo = do_redo.clone();
        use_effect_with(props.global_action, move |action| {
            if let Some((action, _nonce)) = action {
                match action {
                    crate::state::GlobalEditorAction::Save => do_save.emit(()),
                    crate::state::GlobalEditorAction::Undo => do_undo.emit(()),
                    crate::state::GlobalEditorAction::Redo => do_redo.emit(()),
                }
            }
            || ()
        });
    }

    let select_slash = {
        let slash_open = slash_open.clone();
        let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let exec_fn = doc_exec.clone();
        let items = filtered.clone();
        let vault_path = props.vault_path.clone();
        let open_dialog = props.open_dialog.clone();
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited = mark_edited.clone();
        // Recebe a posição na lista filtrada explicitamente (`vi`) em vez
        // de ler `*slash_idx` — o clique do mouse num item precisa
        // aplicar AQUELE item, não o que estava destacado por último via
        // teclado (podiam divergir: navegar com seta e depois clicar em
        // outro item aplicava o item errado).
        Callback::from(move |vi: usize| {
            // Apaga o "/consulta" que está de verdade no texto (digitado
            // normalmente agora, não mais só em estado interno) antes de
            // inserir o item escolhido no lugar. Reconsulta a posição do
            // "/" fresca (não reaproveita nada calculado antes) — o
            // cursor deveria continuar exatamente onde a pessoa parou de
            // digitar o filtro.
            if let Some((text_node, slash_pos, query)) = find_slash_context() {
                delete_slash_context_and_collapse(&text_node, slash_pos, query.chars().count());
            }
            if let Some(&item_idx) = items.get(vi) {
                let item = &SLASH_ITEMS[item_idx];
                match item.html {
                    "__IMG__" => {
                        let vault_path = vault_path.clone();
                        let content_md = content_md.clone();
                        let editor_ref = editor_ref.clone();
                        let segment_refs = segment_refs.clone();
                        let mark_edited = mark_edited.clone();
                        open_dialog.emit(PendingDialog::Prompt {
                            title: "Caminho da imagem ou URL".to_string(),
                            default: String::new(),
                            on_submit: Callback::from(move |path: String| {
                                let content_md = content_md.clone();
                                let editor_ref = editor_ref.clone();
                                let segment_refs = segment_refs.clone();
                                let mark_edited = mark_edited.clone();
                                if path.starts_with("http") {
                                    let html = format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", path.replace('"', "&quot;"));
                                    if let Some(el) = parse_single_element(&html) {
                                        if insert_element_at_cursor(&el, false) {
                                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                            content_md.set(new_md.clone());
                                            mark_edited(new_md);
                                        }
                                    }
                                } else {
                                    let vp = vault_path.clone();
                                    wasm_bindgen_futures::spawn_local(async move {
                                        if let Ok(relative) = crate::api::copy_to_assets(&vp, &path).await {
                                            let html = format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", relative.replace('"', "&quot;"));
                                            if let Some(el) = parse_single_element(&html) {
                                                if insert_element_at_cursor(&el, false) {
                                                    let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                                    content_md.set(new_md.clone());
                                                    mark_edited(new_md);
                                                }
                                            }
                                        }
                                    });
                                }
                            }),
                        });
                    }
                    "__MERMAID__" => {
                        let content_md = content_md.clone();
                        let editor_ref = editor_ref.clone();
                        let segment_refs = segment_refs.clone();
                        let mark_edited = mark_edited.clone();
                        open_dialog.emit(PendingDialog::Prompt {
                            title: "Código Mermaid (ex: graph TD; A-->B)".to_string(),
                            default: String::new(),
                            on_submit: Callback::from(move |code: String| {
                                let html = format!("<div class=\"mermaid\">{}</div>", code.replace('<', "&lt;").replace('>', "&gt;"));
                                if let Some(el) = parse_single_element(&html) {
                                    if insert_element_at_cursor(&el, true) {
                                        let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                        content_md.set(new_md.clone());
                                        mark_edited(new_md);
                                    }
                                }
                                wasm_bindgen_futures::spawn_local(async {
                                    gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                                    if let Some(window) = web_sys::window() {
                                        if let Some(doc) = window.document() {
                                            if let Some(el) = doc.query_selector(".mermaid").ok().flatten() {
                                                if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                                                    init_mermaid_at(&el);
                                                }
                                            }
                                        }
                                    }
                                });
                            }),
                        });
                    }
                    "__ASSET__" => {
                        let vp = vault_path.clone();
                        let open_dialog = open_dialog.clone();
                        let content_md = content_md.clone();
                        let editor_ref = editor_ref.clone();
                        let segment_refs = segment_refs.clone();
                        let mark_edited = mark_edited.clone();
                        wasm_bindgen_futures::spawn_local(async move {
                            match crate::api::list_assets(&vp).await {
                                Ok(assets) => {
                                    if assets.is_empty() {
                                        open_dialog.emit(PendingDialog::Alert {
                                            message: "Nenhum arquivo em assets/. Use /img para adicionar imagens.".to_string(),
                                        });
                                    } else {
                                        let list = assets.join("\n");
                                        open_dialog.emit(PendingDialog::Prompt {
                                            title: format!("Assets disponíveis:\n{}\n\nDigite o nome do arquivo", list),
                                            default: String::new(),
                                            on_submit: Callback::from(move |choice: String| {
                                                let relative = if choice.starts_with("assets/") { choice } else { format!("assets/{}", choice) };
                                                let ext = std::path::Path::new(&relative).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                                                let html = match ext.as_str() {
                                                    "png" | "jpg" | "jpeg" | "gif" | "svg" | "webp" => {
                                                        format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", relative.replace('"', "&quot;"))
                                                    }
                                                    _ => format!("<a href=\"{}\">{}</a>", relative.replace('"', "&quot;"), relative),
                                                };
                                                if let Some(el) = parse_single_element(&html) {
                                                    if insert_element_at_cursor(&el, false) {
                                                        let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                                        content_md.set(new_md.clone());
                                                        mark_edited(new_md);
                                                    }
                                                }
                                            }),
                                        });
                                    }
                                }
                                Err(e) => {
                                    open_dialog.emit(PendingDialog::Alert {
                                        message: format!("Erro ao listar assets: {}", e),
                                    });
                                }
                            }
                        });
                    }
                    // Insere o marcador via `Range` (não `execCommand`,
                    // que fragmentava o HTML de forma imprevisível — ver
                    // `insert_embed_marker_at_cursor`) e já recalcula +
                    // aplica o markdown na hora, em vez de esperar o
                    // usuário clicar em "Salvar" ou digitar de novo pra
                    // disparar o `oninput`: o board/calendário/tabela de
                    // verdade (componente Yew interativo) já aparece no
                    // lugar do marcador imediatamente.
                    "__EMBED_KANBAN__" => {
                        let body = "columns:\n- Backlog\n- Todo\n- Done\nitems:\n- title: Novo card\n  column: Backlog";
                        if insert_embed_marker_at_cursor("kanban", body) {
                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                            content_md.set(new_md.clone());
                            mark_edited(new_md);
                        }
                    }
                    "__EMBED_CALENDAR__" => {
                        let today = {
                            let d = js_sys::Date::new_0();
                            format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
                        };
                        let body = format!("entries:\n- date: '{today}'\n  title: Novo evento");
                        if insert_embed_marker_at_cursor("calendar", &body) {
                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                            content_md.set(new_md.clone());
                            mark_edited(new_md);
                        }
                    }
                    "__EMBED_TABLE__" => {
                        let body = "| Tarefa | Status | Prioridade |\n| ------ | ------ | ---------- |\n| Nova tarefa | todo | media |";
                        if insert_embed_marker_at_cursor("table", body) {
                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                            content_md.set(new_md.clone());
                            mark_edited(new_md);
                        }
                    }
                    other => {
                        // Mesma troca de `execCommand` por `Range`
                        // (ver `insert_embed_marker_at_cursor`/
                        // `insert_element_at_cursor`) — título, lista,
                        // checklist, citação, código, linha e tabela
                        // markdown têm o MESMO risco de fragmentação que
                        // os embeds tinham (menos catastrófico — não
                        // ficam irreconhecíveis pra sempre — mas ainda
                        // saem errados: ex. um heading inserido dentro de
                        // um item de lista virava `- # Título`, texto
                        // literal, não um heading de verdade).
                        if let Some(el) = parse_single_element(other) {
                            if insert_element_at_cursor(&el, true) {
                                let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                content_md.set(new_md.clone());
                                mark_edited(new_md);
                            }
                        } else {
                            exec_fn("insertHTML", other);
                        }
                    }
                }
            }
            slash_open.set(false);
            slash_text.set(String::new());
            slash_idx.set(0);
            wasm_bindgen_futures::spawn_local(async {
                gloo_timers::future::sleep(std::time::Duration::from_millis(100)).await;
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        if let Some(el) = doc.query_selector(".editor__wysiwyg .mermaid").ok().flatten() {
                            if let Ok(el) = el.dyn_into::<web_sys::Element>() {
                                init_mermaid_at(&el);
                            }
                        }
                    }
                }
            });
        })
    };

    let select_wikilink = {
        let wikilink_open = wikilink_open.clone();
        let wikilink_text = wikilink_text.clone();
        let wikilink_idx = wikilink_idx.clone();
        let items = filtered_wikilink.clone();
        let pages = wikilink_pages.clone();
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited = mark_edited.clone();
        Callback::from(move |vi: usize| {
            if let Some((text_node, pos, query)) = find_wikilink_context() {
                delete_range_and_collapse(&text_node, pos, 2 + query.chars().count() as u32);
            }
            if let Some(&page_idx) = items.get(vi) {
                if let Some(page) = pages.get(page_idx) {
                    let href = format!("{}{}", crate::wikilink::SCHEME_PREFIX, crate::wikilink::encode_title(&page.title));
                    let html = format!("<a href=\"{}\">{}</a>", href, page.title.replace('<', "&lt;").replace('>', "&gt;"));
                    if let Some(el) = parse_single_element(&html) {
                        if insert_element_at_cursor(&el, false) {
                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                            content_md.set(new_md.clone());
                            mark_edited(new_md);
                        }
                    }
                }
            }
            wikilink_open.set(false);
            wikilink_text.set(String::new());
            wikilink_idx.set(0);
        })
    };

    let on_keydown = {
        let do_save = do_save.clone();
        let slash_open = slash_open.clone(); let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let filtered_len = filtered.len();
        let select_slash = select_slash.clone();
        let wikilink_open = wikilink_open.clone(); let wikilink_text = wikilink_text.clone();
        let wikilink_idx = wikilink_idx.clone();
        let filtered_wikilink_len = filtered_wikilink.len();
        let select_wikilink = select_wikilink.clone();
        let vim_mode_enabled = props.vim_mode_enabled;
        let vim_keymap = props.vim_keymap.clone();
        let vim_insert = vim_insert.clone();
        let vim_register = vim_register.clone();
        let vim_pending = vim_pending.clone();
        let doc_exec_vim = doc_exec.clone();
        let content_md_vim = content_md.clone();
        let editor_ref_vim = editor_ref.clone();
        let segment_refs_vim = segment_refs.clone();
        let mark_edited_vim = mark_edited.clone();
        let do_undo = do_undo.clone();
        let do_redo = do_redo.clone();
        Callback::from(move |e: KeyboardEvent| {
            // `stop_propagation` nos dois blocos abaixo (ciclo 105): sem
            // isso, o evento borbulha até `.app-root` e o `GlobalKeymap`
            // (que também reconhece Ctrl+S/Ctrl+Z como Salvar/Desfazer
            // por padrão) dispara a MESMA ação de novo — aqui já é
            // tratado por completo, então não deve continuar subindo.
            if (e.ctrl_key()||e.meta_key()) && e.key()=="s" { e.prevent_default(); e.stop_propagation(); do_save.emit(()); return; }

            // Ctrl+Z/Ctrl+Shift+Z funcionam independente do vim mode
            // estar ligado (checado ANTES da interceptação de modo
            // Normal, mesma prioridade do Ctrl+S acima) — desfazer é
            // uma ação de documento, não uma motion de texto.
            if (e.ctrl_key()||e.meta_key()) && e.key().eq_ignore_ascii_case("z") {
                e.prevent_default();
                e.stop_propagation();
                if e.shift_key() { do_redo.emit(()); } else { do_undo.emit(()); }
                return;
            }

            // Popups (menu `/` e autocomplete de wikilink) SEMPRE têm
            // prioridade sobre o vim mode — checados antes de qualquer
            // interceptação de modo Normal/Escape-pra-Normal. Sem isso,
            // Escape com o popup aberto em modo Insert caía no handler
            // do vim (Insert→Normal) em vez de fechar o popup: o popup
            // ficava preso visualmente aberto, o texto cru "/consulta"
            // nunca era apagado, e teclas seguintes (inclusive Enter)
            // eram tratadas como motion do vim em vez de navegar/
            // aplicar o item — o tipo de bug que produzia texto de
            // tabela cru duplicado ao lado do embed de verdade.
            if *slash_open {
                match e.key().as_str() {
                    "Escape" => { e.stop_propagation(); slash_open.set(false); slash_text.set(String::new()); slash_idx.set(0); e.prevent_default(); }
                    "ArrowDown" => { e.stop_propagation(); e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + 1) % filtered_len); } }
                    "ArrowUp" => { e.stop_propagation(); e.prevent_default(); if filtered_len > 0 { slash_idx.set((*slash_idx + filtered_len - 1) % filtered_len); } }
                    "Enter" => { e.stop_propagation(); e.prevent_default(); select_slash.emit(*slash_idx); return; }
                    _ => {}
                }
                return;
            }

            if *wikilink_open {
                match e.key().as_str() {
                    "Escape" => { e.stop_propagation(); wikilink_open.set(false); wikilink_text.set(String::new()); wikilink_idx.set(0); e.prevent_default(); }
                    "ArrowDown" => { e.stop_propagation(); e.prevent_default(); if filtered_wikilink_len > 0 { wikilink_idx.set((*wikilink_idx + 1) % filtered_wikilink_len); } }
                    "ArrowUp" => { e.stop_propagation(); e.prevent_default(); if filtered_wikilink_len > 0 { wikilink_idx.set((*wikilink_idx + filtered_wikilink_len - 1) % filtered_wikilink_len); } }
                    "Enter" => { e.stop_propagation(); e.prevent_default(); select_wikilink.emit(*wikilink_idx); return; }
                    _ => {}
                }
                return;
            }

            // Vim mode — modo Normal: toda tecla é comando, nada digita
            // no texto. `stop_propagation` nas duas saídas (Normal e o
            // Escape de Insert→Normal) pela mesma razão do menu `/`:
            // sem isso, Escape borbulharia pro atalho global de
            // `app.rs` que desseleciona a página inteira.
            if vim_mode_enabled && !*vim_insert {
                e.prevent_default();
                e.stop_propagation();
                let key = e.key();

                if let Some(pending) = vim_pending.borrow_mut().take() {
                    let matched = (pending == "delete_line" && key == vim_keymap.delete_line)
                        || (pending == "yank_line" && key == vim_keymap.yank_line);
                    if matched {
                        if pending == "delete_line" {
                            if vim_delete_line(&vim_register) {
                                let new_md = recompute_markdown_from_dom(&content_md_vim, &editor_ref_vim, &segment_refs_vim);
                                content_md_vim.set(new_md.clone());
                                mark_edited_vim(new_md);
                            }
                        } else {
                            vim_yank_line(&vim_register);
                        }
                        return;
                    }
                    // não confirmou o par — cai pro tratamento normal da
                    // tecla atual abaixo (não perde o input do usuário)
                }

                if key == vim_keymap.delete_line {
                    *vim_pending.borrow_mut() = Some("delete_line".to_string());
                    return;
                }
                if key == vim_keymap.yank_line {
                    *vim_pending.borrow_mut() = Some("yank_line".to_string());
                    return;
                }

                if key == vim_keymap.left { vim_move("backward", "character"); }
                else if key == vim_keymap.right { vim_move("forward", "character"); }
                else if key == vim_keymap.down { vim_move("forward", "line"); }
                else if key == vim_keymap.up { vim_move("backward", "line"); }
                else if key == vim_keymap.word_forward { vim_move("forward", "word"); }
                else if key == vim_keymap.word_backward { vim_move("backward", "word"); }
                else if key == vim_keymap.line_start { vim_move("backward", "lineboundary"); }
                else if key == vim_keymap.line_end { vim_move("forward", "lineboundary"); }
                else if key == vim_keymap.doc_start { vim_move("backward", "documentboundary"); }
                else if key == vim_keymap.doc_end { vim_move("forward", "documentboundary"); }
                else if key == vim_keymap.insert_before { vim_insert.set(true); }
                else if key == vim_keymap.insert_after { vim_move("forward", "character"); vim_insert.set(true); }
                else if key == vim_keymap.open_below {
                    if vim_open_line(false) {
                        vim_insert.set(true);
                        let new_md = recompute_markdown_from_dom(&content_md_vim, &editor_ref_vim, &segment_refs_vim);
                        content_md_vim.set(new_md.clone());
                        mark_edited_vim(new_md);
                    }
                }
                else if key == vim_keymap.open_above {
                    if vim_open_line(true) {
                        vim_insert.set(true);
                        let new_md = recompute_markdown_from_dom(&content_md_vim, &editor_ref_vim, &segment_refs_vim);
                        content_md_vim.set(new_md.clone());
                        mark_edited_vim(new_md);
                    }
                }
                else if key == vim_keymap.delete_char { doc_exec_vim("forwardDelete", ""); }
                else if key == vim_keymap.paste {
                    if vim_paste_after(&vim_register) {
                        let new_md = recompute_markdown_from_dom(&content_md_vim, &editor_ref_vim, &segment_refs_vim);
                        content_md_vim.set(new_md.clone());
                        mark_edited_vim(new_md);
                    }
                }
                else if key == vim_keymap.undo {
                    // Reusa o MESMO undo de documento do Ctrl+Z (não
                    // `execCommand("undo")` nativo) — o undo nativo do
                    // contenteditable opera fora do controle de
                    // `content_md`/`undo_stack`, podia desincronizar os
                    // dois (o DOM voltava um passo que `content_md`
                    // nunca soube que aconteceu) e produzir conteúdo
                    // duplicado/preso na próxima vez que algo
                    // reinjetasse HTML a partir de `content_md`.
                    do_undo.emit(());
                }
                return;
            }
            if vim_mode_enabled && *vim_insert && e.key() == "Escape" {
                e.prevent_default();
                e.stop_propagation();
                vim_insert.set(false);
                return;
            }

            // Markdown block + inline shortcuts on Space/Enter
            if e.key() == " " || e.key() == "Enter" {
                if let Some(window) = web_sys::window() {
                    if let Some(doc) = window.document() {
                        apply_block_shortcut(&window, &doc, &e);
                        apply_inline_formatting(&window, &doc);
                    }
                }
            }
        })
    };

    let on_delete = {
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let page_title = page.title.clone(); let cb = props.on_page_deleted.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_| {
            let vault_path = vault_path.clone(); let page_path = page_path.clone(); let cb = cb.clone();
            open_dialog.emit(PendingDialog::Confirm {
                message: format!("Excluir \"{}\"?", page_title),
                confirm_label: "Excluir".to_string(),
                on_confirm: Callback::from(move |_| {
                    let vault_path = vault_path.clone(); let page_path = page_path.clone(); let cb = cb.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Err(e) = api::delete_page(&vault_path, &page_path).await {
                            web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e));
                        } else { cb.emit(()); }
                    });
                }),
            });
        })
    };

    let on_export = {
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let segments_export = segments.clone();
        let has_embeds_export = has_embeds;
        let page_title = page.title.clone();
        Callback::from(move |_| {
            let html = if has_embeds_export {
                let mut out = String::new();
                for (i, seg) in segments_export.iter().enumerate() {
                    match seg {
                        DocSegment::Markdown(_) => {
                            if let Some(div) = segment_refs.get(i).and_then(|r| r.cast::<web_sys::Element>()) {
                                out.push_str(&div.inner_html());
                            }
                        }
                        DocSegment::Embed(data) => {
                            let escaped = data.to_fence_text()
                                .replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;");
                            out.push_str(&format!("<pre>{}</pre>", escaped));
                        }
                    }
                }
                Some(out)
            } else {
                editor_ref.cast::<web_sys::Element>().map(|div| div.inner_html())
            };
            if let Some(html) = html {
                let full = format!(
                    "<!DOCTYPE html>\n<html lang=\"pt-BR\"><head><meta charset=\"utf-8\"><title>{title}</title>\
                    <style>\
                    body{{max-width:800px;margin:3rem auto;font-family:system-ui,Inter,sans-serif;line-height:1.8;color:#1a1a1a;padding:0 1.5rem}}\
                    h1,h2,h3{{margin-top:2rem;font-weight:600}}\
                    h1{{font-size:2rem}} h2{{font-size:1.5rem}} h3{{font-size:1.2rem}}\
                    pre{{background:#f4f4f4;padding:1.2rem;border-radius:8px;overflow-x:auto}}\
                    code{{background:#f0f0f0;padding:0.2em 0.4em;border-radius:4px;font-size:0.9em;font-family:JetBrains Mono,Fira Code,monospace}}\
                    pre code{{background:none;padding:0}}\
                    table{{border-collapse:collapse;width:100%;margin:1rem 0}}\
                    td,th{{border:1px solid #ddd;padding:8px 12px}}\
                    th{{background:#f8f8f8;text-align:left}}\
                    img{{max-width:100%;height:auto;border-radius:8px}}\
                    blockquote{{border-left:4px solid #8B5CF6;padding-left:1rem;color:#666;margin:1rem 0}}\
                    @media print{{body{{max-width:100%;margin:0;font-size:12pt}}}}\
                    </style></head><body>{html}</body></html>",
                    title = page_title, html = html
                );
                let arr = js_sys::Array::new();
                arr.push(&wasm_bindgen::JsValue::from_str(&full));
                if let Some(blob) = web_sys::Blob::new_with_str_sequence(&arr).ok() {
                    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                    let _ = web_sys::window().and_then(|w| w.open_with_url_and_target(&url, "_blank").ok());
                }
            }
        })
    };

    // Histórico de página via git (ciclo 117) — só busca ao abrir o
    // modal (não a cada render), `None` cobre tanto "ainda não
    // buscou" quanto "vault não é repo git" (mesmo degrade silencioso
    // de `git_status`, ciclo 103).
    let history_modal_open = use_state(|| false);
    let history_entries: UseStateHandle<Option<Vec<api::GitLogEntry>>> = use_state(|| None);
    let history_loading = use_state(|| false);
    let open_history = {
        let vault_path = props.vault_path.clone();
        let page_path = page.path.clone();
        let history_modal_open = history_modal_open.clone();
        let history_entries = history_entries.clone();
        let history_loading = history_loading.clone();
        Callback::from(move |_: MouseEvent| {
            history_modal_open.set(true);
            history_loading.set(true);
            let vault_path = vault_path.clone();
            let page_path = page_path.clone();
            let history_entries = history_entries.clone();
            let history_loading = history_loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let result = api::git_log(&vault_path, &page_path).await.unwrap_or(None);
                history_entries.set(result);
                history_loading.set(false);
            });
        })
    };
    let history_body: Html = if *history_loading {
        html! { <p class="editor__status">{ "Carregando..." }</p> }
    } else {
        match &*history_entries {
            None => html! {
                <p class="editor__status">{ "Este vault não é um repositório git (ou git não está instalado)." }</p>
            },
            Some(entries) if entries.is_empty() => html! {
                <p class="editor__status">{ "Nenhum commit encontrado pra esta página." }</p>
            },
            Some(entries) => html! {
                <ul class="git-history-list">
                    { for entries.iter().map(|e| html! {
                        <li class="git-history-item">
                            <span class="git-history-item__hash">{ &e.hash }</span>
                            <span class="git-history-item__date">{ &e.date }</span>
                            <span class="git-history-item__message">{ &e.message }</span>
                        </li>
                    }) }
                </ul>
            },
        }
    };

    let save_label = if *saving { "Salvando..." } else if *edited { "Salvar *" } else { "Salvar" };
    let on_edit: Callback<InputEvent> = {
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited = mark_edited.clone();
        let slash_open = slash_open.clone();
        let slash_text = slash_text.clone();
        let slash_idx = slash_idx.clone();
        let wikilink_open = wikilink_open.clone();
        let wikilink_text = wikilink_text.clone();
        let wikilink_idx = wikilink_idx.clone();
        Callback::from(move |_: InputEvent| {
            let md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
            mark_edited(md);

            // "/" digitado normalmente no texto (não é mais interceptado
            // no keydown) — a cada tecla, reconsulta o que está
            // imediatamente antes do cursor. Se casar com "/consulta",
            // abre/atualiza o menu; senão, fecha (cobre digitar espaço
            // sem precisar de tratamento especial pra essa tecla).
            match find_slash_context() {
                Some((_, _, query)) => {
                    if !*slash_open {
                        slash_open.set(true);
                    }
                    if *slash_text != query {
                        slash_text.set(query);
                        slash_idx.set(0);
                    }
                }
                None => {
                    if *slash_open {
                        slash_open.set(false);
                        slash_text.set(String::new());
                        slash_idx.set(0);
                    }
                }
            }

            // Mesma lógica pro popup de wikilink, gatilho "[[" em vez de
            // "/" — os dois nunca casam ao mesmo tempo (guardas de prefixo
            // diferentes), então rodar os dois checks todo `oninput` é
            // seguro.
            match find_wikilink_context() {
                Some((_, _, query)) => {
                    if !*wikilink_open {
                        wikilink_open.set(true);
                    }
                    if *wikilink_text != query {
                        wikilink_text.set(query);
                        wikilink_idx.set(0);
                    }
                }
                None => {
                    if *wikilink_open {
                        wikilink_open.set(false);
                        wikilink_text.set(String::new());
                        wikilink_idx.set(0);
                    }
                }
            }
        })
    };

    // Insere uma linha de markdown vazia na posição `pos` e foca nela —
    // usado pelos botões de hover que aparecem na borda de cima/baixo de
    // um embed. Sem isso, um embed que nasce como primeiro/último
    // segmento (ou colado a outro embed, sem nenhuma linha de markdown
    // entre eles) não tinha nenhum lugar clicável pra digitar texto ali.
    // Reconsulta o DOM pelo atributo `data-segment-index` depois de um
    // sleep curto em vez de guardar um `NodeRef` — os `segment_refs` são
    // recriados a cada renderização (o array muda de tamanho ao
    // inserir), então um `NodeRef` capturado antes da inserção não
    // apontaria pro elemento novo depois.
    let insert_blank_line = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited = mark_edited.clone();
        move |pos: usize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited = mark_edited.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let full = (*content_md).clone();
                let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full);
                let mut segs = crate::embed::segment(body);
                let pos = pos.min(segs.len());
                // Não pode ser string vazia: `embed::join` não escreve
                // nada (nem quebra de linha) pra um `Markdown("")`, então
                // ao serializar e reparsear o segmento em branco some de
                // novo (os dois embeds ficam colados, sem nada entre eles
                // pro parser reconhecer como um segmento distinto).
                segs.insert(pos, DocSegment::Markdown("\n".to_string()));
                let new_body = crate::embed::join(&segs);
                let new_full = if frontmatter_text.is_empty() { new_body } else { format!("{}\n{}", frontmatter_text, new_body) };
                content_md.set(new_full.clone());
                mark_edited(new_full);

                wasm_bindgen_futures::spawn_local(async move {
                    gloo_timers::future::sleep(std::time::Duration::from_millis(60)).await;
                    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                        let selector = format!("[data-segment-index=\"{pos}\"]");
                        if let Some(el) = doc.query_selector(&selector).ok().flatten() {
                            if let Ok(el) = el.dyn_into::<web_sys::HtmlElement>() {
                                let _ = el.focus();
                            }
                        }
                    }
                });
            })
        }
    };

    // Remove o embed inteiro (segmento de índice `pos`) da página — antes
    // deste botão não tinha nenhum jeito de tirar um kanban/calendário/
    // tabela da página depois de inserido (só dava pra apagar coisas
    // DENTRO dele, tipo uma coluna ou um card).
    let remove_embed = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited = mark_edited.clone();
        let open_dialog = props.open_dialog.clone();
        move |pos: usize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited = mark_edited.clone();
            let open_dialog = open_dialog.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let content_md = content_md.clone();
                let frontmatter_text = frontmatter_text.clone();
                let mark_edited = mark_edited.clone();
                open_dialog.emit(PendingDialog::Confirm {
                    message: "Remover este embed da página? O conteúdo dele (cards, eventos ou linhas da tabela) será perdido.".to_string(),
                    confirm_label: "Remover".to_string(),
                    on_confirm: Callback::from(move |_| {
                        let full = (*content_md).clone();
                        let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full);
                        let mut segs = crate::embed::segment(body);
                        if pos < segs.len() {
                            segs.remove(pos);
                        }
                        let new_body = crate::embed::join(&segs);
                        let new_full = if frontmatter_text.is_empty() { new_body } else { format!("{}\n{}", frontmatter_text, new_body) };
                        content_md.set(new_full.clone());
                        mark_edited(new_full);
                    }),
                });
            })
        }
    };

    let on_drop = {
        let vault_path = props.vault_path.clone();
        Callback::from(move |e: DragEvent| {
            e.prevent_default();
            let dt = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("dataTransfer")).ok();
            let files = dt.and_then(|v| js_sys::Reflect::get(&v, &wasm_bindgen::JsValue::from_str("files")).ok())
                .and_then(|v| v.dyn_into::<web_sys::FileList>().ok());
            if let Some(files) = files {
                let doc = web_sys::window().and_then(|w| w.document());
                for i in 0..files.length() {
                    if let Some(file) = files.item(i) {
                        let name = file.name();
                        let ext = std::path::Path::new(&name).extension().map(|e| e.to_string_lossy().to_lowercase()).unwrap_or_default();
                        if !matches!(ext.as_str(), "png"|"jpg"|"jpeg"|"gif"|"svg"|"webp") { continue; }
                        if let Ok(blob) = file.slice() {
                            let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap_or_default();
                            let html = format!("<img src=\"{}\" alt=\"{}\" style=\"max-width:100%;border-radius:8px;\">", url, name.replace('"', "&quot;"));
                            if let Some(ref doc) = doc {
                                let args = js_sys::Array::new();
                                args.push(&wasm_bindgen::JsValue::from_str("insertHTML"));
                                args.push(&wasm_bindgen::JsValue::from_bool(false));
                                args.push(&wasm_bindgen::JsValue::from_str(&html));
                                if let Some(f) = js_sys::Reflect::get(doc, &wasm_bindgen::JsValue::from_str("execCommand"))
                                    .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
                                { let _ = f.apply(doc, &args); }
                            }
                        }
                    }
                }
            }
        })
    };
    let on_dragover = Callback::from(|e: DragEvent| { e.prevent_default(); });

    // Colar (Ctrl+V) uma imagem da área de transferência (ciclo 118)
    // — diferente do `on_drop` acima (que usa uma URL `blob:` só de
    // sessão, perdida ao recarregar), grava de verdade em `assets/`
    // via `save_pasted_asset` e insere um `<img>` apontando pro path
    // relativo real, mesmo padrão já usado pelo item "__ASSET__" do
    // menu `/`. Só intercepta (chama `prevent_default`) se achar uma
    // imagem — paste de texto normal continua funcionando.
    let on_paste = {
        let vault_path = props.vault_path.clone();
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited = mark_edited.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |e: web_sys::Event| {
            let cd = js_sys::Reflect::get(&e, &wasm_bindgen::JsValue::from_str("clipboardData")).ok();
            let files = cd
                .and_then(|v| js_sys::Reflect::get(&v, &wasm_bindgen::JsValue::from_str("files")).ok())
                .and_then(|v| v.dyn_into::<web_sys::FileList>().ok());
            let Some(files) = files else { return };
            let mut image_file = None;
            for i in 0..files.length() {
                if let Some(file) = files.item(i) {
                    if file.type_().starts_with("image/") {
                        image_file = Some(file);
                        break;
                    }
                }
            }
            let Some(file) = image_file else { return };
            e.prevent_default();

            let mime = file.type_();
            let ext = mime.strip_prefix("image/").unwrap_or("png").to_string();
            let vault_path = vault_path.clone();
            let content_md = content_md.clone();
            let editor_ref = editor_ref.clone();
            let segment_refs = segment_refs.clone();
            let mark_edited = mark_edited.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let blob = gloo_file::Blob::from(file);
                let Ok(bytes) = gloo_file::futures::read_as_bytes(&blob).await else {
                    open_dialog.emit(PendingDialog::Alert {
                        message: "Erro ao ler a imagem colada.".to_string(),
                    });
                    return;
                };
                let b64 = base64::engine::general_purpose::STANDARD.encode(&bytes);
                match api::save_pasted_asset(&vault_path, &ext, &b64).await {
                    Ok(relative) => {
                        let html = format!(
                            "<img src=\"{}\" alt=\"imagem colada\" style=\"max-width:100%;border-radius:8px;\">",
                            relative.replace('"', "&quot;")
                        );
                        if let Some(el) = parse_single_element(&html) {
                            if insert_element_at_cursor(&el, false) {
                                let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                content_md.set(new_md.clone());
                                mark_edited(new_md);
                            }
                        }
                    }
                    Err(e) => {
                        open_dialog.emit(PendingDialog::Alert {
                            message: format!("Erro ao salvar imagem colada: {}", e),
                        });
                    }
                }
            });
        })
    };

    // Clicar num wikilink já renderizado (`<a href="anotadinho://page/...">`,
    // ver `crate::wikilink`) navega pra página em vez de tentar abrir como
    // link externo. Resolve por título (case-insensitive); primeiro match
    // se houver mais de uma página com o mesmo título (v1 — desambiguação
    // de verdade fica pra depois se virar problema real).
    let on_wysiwyg_click = {
        let vault_path = props.vault_path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |e: MouseEvent| {
            let Some(target) = e.target() else { return };
            let Ok(el) = target.dyn_into::<web_sys::Element>() else { return };
            let Ok(Some(anchor)) = el.closest("a") else { return };
            let Some(href) = anchor.get_attribute("href") else { return };
            let Some(encoded) = href.strip_prefix(crate::wikilink::SCHEME_PREFIX) else { return };
            e.prevent_default();
            let title = crate::wikilink::decode_title(encoded);
            let vault_path = vault_path.clone();
            let on_page_selected = on_page_selected.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::list_pages(&vault_path).await {
                    Ok(pages) => {
                        match pages.into_iter().find(|p| p.title.eq_ignore_ascii_case(&title)) {
                            Some(meta) => on_page_selected.emit(meta),
                            None => open_dialog.emit(PendingDialog::Alert {
                                message: format!("Página \"{}\" não encontrada.", title),
                            }),
                        }
                    }
                    Err(e) => web_sys::console::warn_1(&wasm_bindgen::JsValue::from_str(&e)),
                }
            });
        })
    };

    html! {
        <main class="editor">
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <div class="editor__actions">
                    if let Some(ref s) = *status { <span class="editor__status-badge">{ s }</span> }
                    if *edited { <span class="editor__dirty">{ "não salvo" }</span> }
                    <button class="btn btn--primary btn--sm" onclick={do_save.reform(|_| ())} disabled={*saving || !*edited}>{ save_label }</button>
                    <div class="header-menu-wrapper" ref={editor_menu_ref}>
                        <button class="btn btn--ghost btn--sm" onclick={toggle_editor_menu} title="Mais ações">{ "⋯" }</button>
                        if *editor_menu_open {
                            <div class="header-menu">
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let toggle_home = toggle_home.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); toggle_home.emit(e); })
                                }}>
                                    { if is_home { "🏠 Remover como início" } else { "🏠 Definir como início" } }
                                </button>
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let properties_modal_open = properties_modal_open.clone();
                                    Callback::from(move |_| { editor_menu_open.set(false); properties_modal_open.set(true); })
                                }}>
                                    { "Propriedades..." }
                                </button>
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let on_export = on_export.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); on_export.emit(e); })
                                }}>
                                    { "⬇ Exportar HTML" }
                                </button>
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let open_history = open_history.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); open_history.emit(e); })
                                }}>
                                    { "🕐 Histórico" }
                                </button>
                                <div class="divider"></div>
                                <button class="header-menu__item header-menu__item--danger btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let on_delete = on_delete.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); on_delete.emit(e); })
                                }}>
                                    { "Excluir" }
                                </button>
                            </div>
                        }
                    </div>
                </div>
            </header>
            if *properties_modal_open {
                <Modal title={"Propriedades".to_string()} open={true} on_close={{
                    let properties_modal_open = properties_modal_open.clone();
                    Callback::from(move |_: ()| properties_modal_open.set(false))
                }}>
                    <PropertiesPanel
                        frontmatter={parsed_frontmatter}
                        on_change={on_frontmatter_change}
                        open_dialog={props.open_dialog.clone()}
                    />
                </Modal>
            }
            if *history_modal_open {
                <Modal title={"Histórico".to_string()} open={true} on_close={{
                    let history_modal_open = history_modal_open.clone();
                    Callback::from(move |_: ()| history_modal_open.set(false))
                }}>
                    { history_body }
                </Modal>
            }
            <div class="editor__body">
                if *loading {
                    <div class="editor__overlay">
                        <div class="spinner"></div>
                        { "Carregando..." }
                    </div>
                }
                if let Some(ref err) = *error {
                    <div class="editor__overlay editor__overlay--error">{ err }</div>
                }
                if has_embeds {
                    // `key` força o Yew a desmontar/remontar esse `<div>`
                    // de verdade ao trocar de modo (em vez de reaproveitar
                    // o mesmo nó e só ajustar a classe) — os dois branches
                    // (com/sem embeds) renderizam `<div>` como raiz, então
                    // sem uma identidade explícita o Yew via essa raiz
                    // como "a mesma", e o conteúdo do modo anterior
                    // (injetado via `set_inner_html`/inserção de embed via
                    // `Range`, imperativos, fora do rastreamento do VDOM)
                    // ficava "grudado" ali, aparecendo duplicado ao lado
                    // do conteúdo novo de verdade.
                    <div class="editor__wysiwyg-segments" key="segments" onclick={on_wysiwyg_click.clone()}>
                        { for segments.iter().enumerate().map(|(i, seg)| {
                            // `key` inclui o TIPO do segmento (`md`/`embed`), não
                            // só a posição: o caso que realmente quebrava era um
                            // segmento Markdown virar Embed NO MESMO ÍNDICE, com
                            // a CONTAGEM total de segmentos igual antes e depois
                            // (ex: linha vazia + `/tabela` colada nela vira só o
                            // embed, sem sobrar markdown nenhum ali) — só
                            // `segments.len()` no key não pegava essa troca, o
                            // Yew reaproveitava o mesmo `<div>` na mesma posição
                            // (mesma chave) e só ANEXAVA os filhos novos
                            // (botões + `InlineEmbed`) depois do marcador
                            // imperativo que já estava lá (inserido via `Range`,
                            // ver `insert_element_at_cursor`), em vez de
                            // desmontar/remontar de verdade — o mesmo bug de
                            // duplicação já corrigido uma vez (ciclo 079) pra
                            // transição plain↔segments, agora reaparecendo entre
                            // segmentos individuais ao inserir um embed onde
                            // antes havia só markdown.
                            let seg_kind = match seg { DocSegment::Markdown(_) => "md", DocSegment::Embed(_) => "embed" };
                            let key = format!("{}-{}-{}", i, seg_kind, segments.len());
                            match seg {
                                DocSegment::Markdown(_) => {
                                    let node_ref = segment_refs[i].clone();
                                    html! {
                                        <div class="editor__wysiwyg" {key} data-segment-index={i.to_string()} ref={node_ref} contenteditable="true"
                                            spellcheck="false" onkeydown={on_keydown.clone()} oninput={on_edit.clone()}
                                            ondrop={on_drop.clone()} ondragover={on_dragover.clone()} onpaste={on_paste.clone()} />
                                    }
                                }
                                DocSegment::Embed(data) => {
                                    let content_md = content_md.clone();
                                    let mark_edited = mark_edited.clone();
                                    let frontmatter_text = frontmatter_text.clone();
                                    let idx = i;
                                    let on_change = Callback::from(move |new_data: crate::embed::EmbedData| {
                                        let full = (*content_md).clone();
                                        let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full);
                                        let mut segs = crate::embed::segment(body);
                                        if let Some(s) = segs.get_mut(idx) {
                                            *s = DocSegment::Embed(new_data);
                                        }
                                        let new_body = crate::embed::join(&segs);
                                        let new_full = if frontmatter_text.is_empty() {
                                            new_body
                                        } else {
                                            format!("{}\n{}", frontmatter_text, new_body)
                                        };
                                        content_md.set(new_full.clone());
                                        mark_edited(new_full);
                                    });
                                    // Botões que só aparecem no hover da borda de
                                    // cima/baixo do embed — sem isso, um embed que
                                    // nasce sem uma linha de markdown vizinha (é o
                                    // primeiro/último segmento, ou está colado a
                                    // outro embed) não tinha nenhum lugar clicável
                                    // pra digitar texto ali.
                                    html! {
                                        <div class="embed-hover-wrapper" {key}>
                                            <button class="embed-hover-wrapper__add-line embed-hover-wrapper__add-line--top"
                                                onclick={insert_blank_line(i)} title="Adicionar linha acima">{ "+" }</button>
                                            <button class="embed-hover-wrapper__remove"
                                                onclick={remove_embed(i)} title="Remover embed">{ "✕" }</button>
                                            <InlineEmbed
                                                data={data.clone()}
                                                vault_path={props.vault_path.clone()}
                                                on_change={on_change}
                                                open_dialog={props.open_dialog.clone()}
                                                on_page_selected={props.on_page_selected.clone()}
                                            />
                                            <button class="embed-hover-wrapper__add-line embed-hover-wrapper__add-line--bottom"
                                                onclick={insert_blank_line(i + 1)} title="Adicionar linha abaixo">{ "+" }</button>
                                        </div>
                                    }
                                }
                            }
                        }) }
                    </div>
                } else {
                    <div class="editor__wysiwyg" key="plain" ref={editor_ref} contenteditable="true"
                        spellcheck="false" onkeydown={on_keydown} oninput={on_edit}
                        ondrop={on_drop} ondragover={on_dragover} onclick={on_wysiwyg_click} onpaste={on_paste} />
                }
            </div>
            if *slash_open {
                <div class="slash-menu">
                    <div class="slash-menu__header">
                        <span>{ "/" }{ &*slash_text }</span>
                        <span class="slash-menu__hint">{ format!("{} resultados", filtered.len()) }</span>
                    </div>
                    <div class="slash-menu__list">
                        { for filtered.iter().enumerate().map(|(vi, &item_idx)| {
                            let item = &SLASH_ITEMS[item_idx];
                            let is_active = vi == *slash_idx;
                            let class = if is_active { "slash-menu__item slash-menu__item--active" } else { "slash-menu__item" };
                            let sel = select_slash.clone();
                            // Sem isso o clique do mouse tira o foco/seleção
                            // de dentro do contenteditable (o navegador
                            // colapsa a seleção no mousedown antes do
                            // onclick disparar), e o insertHTML do
                            // execCommand acaba não tendo onde inserir —
                            // aplicava em lugar nenhum ou no lugar errado.
                            let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
                            let onclick = Callback::from(move |_| sel.emit(vi));
                            let node_ref = if is_active { slash_active_ref.clone() } else { NodeRef::default() };
                            html! {
                                <div {class} ref={node_ref} {onmousedown} {onclick}>
                                    <span class="slash-menu__item-label">{ item.label }</span>
                                    <span class="slash-menu__item-desc">{ item.desc }</span>
                                </div>
                            }
                        }) }
                    </div>
                </div>
            }
            if *wikilink_open {
                <div class="wikilink-menu">
                    <div class="wikilink-menu__header">
                        <span>{ "[[" }{ &*wikilink_text }</span>
                        <span class="wikilink-menu__hint">{ format!("{} páginas", filtered_wikilink.len()) }</span>
                    </div>
                    <div class="wikilink-menu__list">
                        if filtered_wikilink.is_empty() {
                            <p class="wikilink-menu__empty">{ "Nenhuma página com esse título" }</p>
                        }
                        { for filtered_wikilink.iter().enumerate().map(|(vi, &page_idx)| {
                            let Some(page) = wikilink_pages.get(page_idx) else { return html! {} };
                            let is_active = vi == *wikilink_idx;
                            let class = if is_active { "wikilink-menu__item wikilink-menu__item--active" } else { "wikilink-menu__item" };
                            let sel = select_wikilink.clone();
                            let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
                            let onclick = Callback::from(move |_| sel.emit(vi));
                            let node_ref = if is_active { wikilink_active_ref.clone() } else { NodeRef::default() };
                            html! {
                                <div {class} ref={node_ref} {onmousedown} {onclick}>
                                    <span class="wikilink-menu__item-icon">{ "📄" }</span>
                                    <span class="wikilink-menu__item-title">{ &page.title }</span>
                                </div>
                            }
                        }) }
                    </div>
                </div>
            }
            if !backlinks.is_empty() {
                <details class="editor__backlinks">
                    <summary class="editor__backlinks-summary">
                        { format!("🔗 Backlinks ({})", backlinks.len()) }
                    </summary>
                    <ul class="editor__backlinks-list">
                        { for backlinks.iter().map(|(path, excerpt)| {
                            let path = path.clone();
                            let excerpt = excerpt.clone();
                            let title = std::path::Path::new(&path).file_stem()
                                .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                            let meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                            let on_page_selected = props.on_page_selected.clone();
                            let onclick = Callback::from(move |_| on_page_selected.emit(meta.clone()));
                            html! {
                                <li class="editor__backlinks-item" {onclick}>
                                    <span class="editor__backlinks-item-title">{ &title }</span>
                                    <span class="editor__backlinks-item-excerpt">{ &excerpt }</span>
                                </li>
                            }
                        }) }
                    </ul>
                </details>
            }
            <div class="editor__statusbar">
                <span>{ format!("{} palavras · {} caracteres", word_count, char_count) }</span>
                if props.vim_mode_enabled {
                    <span class={ if *vim_insert { "editor__vim-mode editor__vim-mode--insert" } else { "editor__vim-mode editor__vim-mode--normal" } }>
                        { if *vim_insert { "-- INSERT --" } else { "-- NORMAL --" } }
                    </span>
                }
                <span class="editor__statusbar-hint">{ "Digite / ou use # - > * para formatar" }</span>
            </div>
        </main>
    }
}

/// Recalcula o markdown a partir do `content_md` mais recente + o texto
/// ao vivo dos trechos contenteditable (`editor_ref` pra página sem
/// embeds, `segment_refs` pra página com embeds — embeds em si já vêm
/// atualizados dentro de `content_md` via `on_change`). Extraído de
/// `do_save` pra ser reaproveitado no flush de segurança ao trocar de
/// página sem salvar.
/// Insere um marcador `<div data-embed-insert="kind">corpo</div>` na
/// posição do cursor via `Range::insert_node` em vez de
/// `execCommand("insertHTML", ...)`. O `execCommand` demonstrou ser
/// pouco confiável no WebKitGTK pra HTML multi-linha: dependendo de
/// onde o cursor estava (dentro de um item de lista, no fim de um
/// parágrafo com texto), ele fragmentava o HTML inserido de formas
/// imprevisíveis — texto do corpo do embed vazava pro parágrafo vizinho,
/// ou a própria abertura `{{ type: "..." }}` saía com um `- ` de bullet
/// grudado na frente, quebrando o parser de embeds pra sempre (o embed
/// nunca virava um componente de verdade, nem depois de salvar).
/// `Range::insert_node` é uma API de DOM mais baixo nível e previsível:
/// insere o nó EXATAMENTE onde o cursor está, sem o `execCommand`
/// reinterpretar/reformatar o HTML ao redor.
/// Acha o "/consulta" imediatamente antes do cursor — usado tanto pra
/// decidir se o menu de comando deve abrir/atualizar (chamado a cada
/// `oninput`) quanto pra saber o que apagar na hora de aplicar um item
/// selecionado (chamado de novo, fresco, em `select_slash`). Só
/// reconhece o caso mais comum: cursor colapsado dentro de um nó de
/// texto puro, com o "/" no início da linha ou logo depois de um espaço
/// (evita disparar em "3/4"), sem espaço nenhum entre o "/" e o cursor
/// (digitar espaço encerra o comando naturalmente, sem precisar de
/// tratamento especial pra tecla Espaço).
fn find_slash_context() -> Option<(web_sys::Text, u32, String)> {
    let window = web_sys::window()?;
    let sel = window.get_selection().ok().flatten()?;
    if sel.range_count() == 0 {
        return None;
    }
    let range = sel.get_range_at(0).ok()?;
    if !range.collapsed() {
        return None;
    }
    let node = range.start_container().ok()?;
    let text_node = node.dyn_ref::<web_sys::Text>()?.clone();
    let offset = range.start_offset().ok()? as usize;
    let data = text_node.data();
    let prefix: String = data.chars().take(offset).collect();
    let slash_byte_pos = prefix.rfind('/')?;
    let query = &prefix[slash_byte_pos + 1..];
    if query.chars().any(char::is_whitespace) {
        return None;
    }
    let before_slash = &prefix[..slash_byte_pos];
    if !before_slash.is_empty() && !before_slash.ends_with(char::is_whitespace) {
        return None;
    }
    let slash_char_pos = prefix[..slash_byte_pos].chars().count() as u32;
    Some((text_node, slash_char_pos, query.to_string()))
}

/// Apaga o "/consulta" (achado por `find_slash_context`) do nó de texto e
/// deixa o cursor colapsado exatamente onde o "/" estava — pronto pro
/// item selecionado ser inserido ali no lugar.
fn delete_slash_context_and_collapse(text_node: &web_sys::Text, slash_pos: u32, query_len: usize) -> bool {
    delete_range_and_collapse(text_node, slash_pos, (1 + query_len) as u32)
}

/// Acha o "[[consulta" imediatamente antes do cursor — mesmo mecanismo de
/// `find_slash_context`, mas pro gatilho de wikilink. Diferente do "/", o
/// "[[" não precisa estar em início de linha/depois de espaço (um
/// wikilink pode aparecer no meio de uma frase) — só exige que não haja
/// espaço nem "]" entre o "[[" mais recente e o cursor.
fn find_wikilink_context() -> Option<(web_sys::Text, u32, String)> {
    let window = web_sys::window()?;
    let sel = window.get_selection().ok().flatten()?;
    if sel.range_count() == 0 {
        return None;
    }
    let range = sel.get_range_at(0).ok()?;
    if !range.collapsed() {
        return None;
    }
    let node = range.start_container().ok()?;
    let text_node = node.dyn_ref::<web_sys::Text>()?.clone();
    let offset = range.start_offset().ok()? as usize;
    let data = text_node.data();
    let prefix: String = data.chars().take(offset).collect();
    let open_byte_pos = prefix.rfind("[[")?;
    let query = &prefix[open_byte_pos + 2..];
    if query.chars().any(|c| c.is_whitespace() || c == ']' || c == '[') {
        return None;
    }
    let open_char_pos = prefix[..open_byte_pos].chars().count() as u32;
    Some((text_node, open_char_pos, query.to_string()))
}

/// Apaga `delete_len` caracteres a partir de `start_pos` no nó de texto e
/// deixa o cursor colapsado ali — usado tanto pro menu `/` quanto pro
/// popup de wikilink pra remover o gatilho+consulta digitados antes de
/// inserir o item escolhido no lugar.
fn delete_range_and_collapse(text_node: &web_sys::Text, start_pos: u32, delete_len: u32) -> bool {
    if text_node.delete_data(start_pos, delete_len).is_err() {
        return false;
    }
    let Some(window) = web_sys::window() else { return false };
    let Some(doc) = window.document() else { return false };
    let Some(sel) = window.get_selection().ok().flatten() else { return false };
    let Ok(range) = doc.create_range() else { return false };
    if range.set_start(text_node, start_pos).is_err() {
        return false;
    }
    range.collapse_with_to_start(true);
    sel.remove_all_ranges().ok();
    sel.add_range(&range).is_ok()
}

fn insert_embed_marker_at_cursor(kind: &str, body: &str) -> bool {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return false };
    let Ok(div) = doc.create_element("div") else { return false };
    if div.set_attribute("data-embed-insert", kind).is_err() {
        return false;
    }
    div.set_inner_html(&body.replace('\n', "<br>"));
    insert_element_at_cursor(&div, true)
}

/// Insere `el` na posição do cursor via `Range::insert_node` em vez de
/// `execCommand("insertHTML", ...)`, que demonstrou ser pouco confiável
/// no WebKitGTK pra HTML multi-linha (fragmentava de formas
/// imprevisíveis dependendo de onde o cursor estava). Usado pelos itens
/// do menu `/` — tanto os embeds (kanban/calendário/tabela) quanto os
/// blocos "normais" (título, lista, citação, código, linha, tabela
/// markdown, diagrama).
///
/// `break_out_of_block`: quando `true`, se o cursor estiver dentro de um
/// `<li>`/`<p>`/blockquote/heading, insere `el` como IRMÃO desse bloco
/// (ou da lista inteira, no caso de `<li>`) em vez de aninhado dentro
/// dele — necessário pra blocos que precisam ficar em linha própria
/// (embeds SEMPRE, os itens "normais" de bloco também, pra não virar
/// `- # Título` em vez de um heading de verdade). `false` pra conteúdo
/// inline-safe (ex: imagem), que pode ficar aninhado normalmente dentro
/// de um parágrafo.
fn insert_element_at_cursor(el: &web_sys::Element, break_out_of_block: bool) -> bool {
    let Some(window) = web_sys::window() else { return false };
    let Some(sel) = window.get_selection().ok().flatten() else { return false };
    if sel.range_count() == 0 {
        return false;
    }
    let Ok(range) = sel.get_range_at(0) else { return false };

    let block_ancestor = break_out_of_block.then(|| {
        let start_container = range.start_container().ok();
        let container_el: Option<web_sys::Element> = start_container.and_then(|n| {
            n.dyn_ref::<web_sys::Element>().cloned().or_else(|| n.parent_element())
        });
        container_el.and_then(|e| e.closest("li, p, blockquote, h1, h2, h3, h4, h5, h6").ok().flatten())
    }).flatten();

    let inserted = if let Some(block) = block_ancestor {
        let anchor = if block.tag_name().to_lowercase() == "li" {
            block.parent_element() // quebra pra fora da lista inteira, não só do item
        } else {
            Some(block)
        };
        anchor.and_then(|a| a.parent_node().map(|p| (p, a.next_sibling())))
            .map(|(parent, next)| parent.insert_before(el, next.as_ref()).is_ok())
            .unwrap_or(false)
    } else {
        let _ = range.delete_contents();
        range.insert_node(el).is_ok()
    };
    if !inserted {
        return false;
    }

    // Move o cursor pra depois do nó inserido, senão continuaria "dentro"
    // dele — próxima tecla digitada iria pro lugar errado.
    range.set_start_after(el).ok();
    range.collapse();
    sel.remove_all_ranges().ok();
    let _ = sel.add_range(&range);
    true
}

/// Constrói UM elemento a partir de uma string HTML (assume que `html`
/// tem exatamente um elemento raiz — todos os itens do menu `/`
/// respeitam isso).
fn parse_single_element(html: &str) -> Option<web_sys::Element> {
    let doc = web_sys::window()?.document()?;
    let wrapper = doc.create_element("div").ok()?;
    wrapper.set_inner_html(html);
    wrapper.first_element_child()
}

/// Move/estende a seleção via `Selection.modify` — a mesma API nativa
/// que o navegador usa pra Ctrl+seta/Shift+seta, generosa o bastante
/// (granularidade `character`/`word`/`line`/`lineboundary`/
/// `documentboundary`) pra implementar as motions do vim mode sem
/// reescrever navegação de texto/palavra/linha na mão.
fn vim_move(direction: &str, granularity: &str) {
    if let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok()).flatten() {
        let _ = sel.modify("move", direction, granularity);
    }
}

/// Bloco (linha, no sentido do vim) onde o cursor está — item de lista,
/// parágrafo, heading, citação. `dd`/`yy`/`p`/`o`/`O` operam nesse
/// elemento inteiro. `None` se o cursor não estiver dentro de um desses
/// (evita `dd` apagar o container inteiro do editor por engano).
fn vim_current_block() -> Option<web_sys::Element> {
    let window = web_sys::window()?;
    let sel = window.get_selection().ok()??;
    if sel.range_count() == 0 {
        return None;
    }
    let range = sel.get_range_at(0).ok()?;
    let node = range.start_container().ok()?;
    let el = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element())?;
    el.closest("li, p, h1, h2, h3, h4, h5, h6, blockquote").ok().flatten()
}

/// `yy`: copia o texto da linha atual pro registrador, sem mutar nada.
fn vim_yank_line(register: &std::rc::Rc<std::cell::RefCell<String>>) -> bool {
    let Some(block) = vim_current_block() else { return false };
    *register.borrow_mut() = block.text_content().unwrap_or_default();
    true
}

/// `dd`: copia a linha pro registrador (igual `yy`) e remove ela do DOM.
/// Mutação direta (não passa por `execCommand`), então quem chama
/// precisa recalcular o markdown e chamar `mark_edited` depois.
fn vim_delete_line(register: &std::rc::Rc<std::cell::RefCell<String>>) -> bool {
    let Some(block) = vim_current_block() else { return false };
    *register.borrow_mut() = block.text_content().unwrap_or_default();
    block.remove();
    true
}

/// Tag do elemento a criar quando abrindo/colando uma linha nova ao
/// lado de `block` — `li` continua `li` (senão colar dentro de uma
/// lista quebrava a lista, virando um `<p>` solto no meio dos itens);
/// qualquer outro tipo (heading, citação, parágrafo) vira `p` normal,
/// mesmo comportamento padrão de outros editores (abrir linha depois de
/// um título não repete o título).
fn sibling_line_tag(block: &web_sys::Element) -> &'static str {
    if block.tag_name().to_lowercase() == "li" { "li" } else { "p" }
}

/// `p`: insere o conteúdo do registrador como uma linha nova logo depois
/// da linha atual.
fn vim_paste_after(register: &std::rc::Rc<std::cell::RefCell<String>>) -> bool {
    let text = register.borrow().clone();
    if text.is_empty() {
        return false;
    }
    let Some(block) = vim_current_block() else { return false };
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return false };
    let Ok(new_el) = doc.create_element(sibling_line_tag(&block)) else { return false };
    new_el.set_text_content(Some(&text));
    let Some(parent) = block.parent_node() else { return false };
    let next = block.next_sibling();
    parent.insert_before(&new_el, next.as_ref()).is_ok()
}

/// `o`/`O`: insere uma linha vazia abaixo (`before=false`)/acima
/// (`before=true`) da linha atual e coloca o cursor nela — quem chama
/// ainda precisa setar `vim_insert` pra `true`.
fn vim_open_line(before: bool) -> bool {
    let Some(block) = vim_current_block() else { return false };
    let Some(window) = web_sys::window() else { return false };
    let Some(doc) = window.document() else { return false };
    let Ok(new_el) = doc.create_element(sibling_line_tag(&block)) else { return false };
    new_el.set_inner_html("<br>");
    let Some(parent) = block.parent_node() else { return false };
    let inserted = if before {
        parent.insert_before(&new_el, Some(block.unchecked_ref())).is_ok()
    } else {
        let next = block.next_sibling();
        parent.insert_before(&new_el, next.as_ref()).is_ok()
    };
    if !inserted {
        return false;
    }
    let Ok(range) = doc.create_range() else { return false };
    if range.set_start(&new_el, 0).is_err() {
        return false;
    }
    range.collapse_with_to_start(true);
    let Some(sel) = window.get_selection().ok().flatten() else { return false };
    sel.remove_all_ranges().ok();
    sel.add_range(&range).is_ok()
}

fn recompute_markdown_from_dom(content_md: &str, editor_ref: &NodeRef, segment_refs: &[NodeRef]) -> String {
    let (fm, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(content_md);
    let segs = crate::embed::segment(body);
    let has_embeds_now = segs.iter().any(|s| matches!(s, DocSegment::Embed(_)));

    if has_embeds_now {
        let new_segs: Vec<DocSegment> = segs.iter().enumerate().map(|(i, seg)| match seg {
            DocSegment::Markdown(orig) => {
                let text = segment_refs.get(i)
                    .and_then(|r| r.cast::<web_sys::Element>())
                    .map(|el| crate::html_to_md::html_to_markdown(&el))
                    .unwrap_or_else(|| orig.clone());
                DocSegment::Markdown(text)
            }
            other => other.clone(),
        }).collect();
        let new_body = crate::embed::join(&new_segs);
        if fm.is_empty() { new_body } else { format!("{}\n{}", fm, new_body) }
    } else if let Some(div) = editor_ref.cast::<web_sys::Element>() {
        // Bug real encontrado durante o ciclo do autosave: esse branch
        // (páginas sem embed) reconstruía só o corpo a partir do DOM sem
        // recolocar `fm` na frente — qualquer salvamento de uma página
        // com frontmatter (`title::`, `type::` etc) e sem embeds perdia o
        // frontmatter inteiro. O branch com embeds (acima) já fazia isso
        // certo, só esse aqui estava faltando.
        let body_md = crate::html_to_md::html_to_markdown(&div);
        if fm.is_empty() { body_md } else { format!("{}\n{}", fm, body_md) }
    } else {
        content_md.to_string()
    }
}

fn exec_cmd_global(cmd: &str) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        let args = js_sys::Array::new();
        args.push(&wasm_bindgen::JsValue::from_str(cmd));
        args.push(&wasm_bindgen::JsValue::from_bool(false));
        if let Some(f) = js_sys::Reflect::get(&doc, &wasm_bindgen::JsValue::from_str("execCommand"))
            .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
        { let _ = f.apply(&doc, &args); }
    }
}

fn init_highlight() {
    if let Some(window) = web_sys::window() {
        if let Some(hljs) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("hljs")).ok() {
            if let Some(obj) = hljs.dyn_into::<js_sys::Object>().ok() {
                let _ = js_sys::Reflect::apply(
                    &js_sys::Reflect::get(&obj, &wasm_bindgen::JsValue::from_str("highlightAll")).ok()
                        .and_then(|v| v.dyn_into::<js_sys::Function>().ok()).unwrap_or_else(|| js_sys::Function::new_no_args("")),
                    &wasm_bindgen::JsValue::null(),
                    &js_sys::Array::new()
                );
            }
        }
    }
}

fn exec_cmd(doc: &web_sys::Document, cmd: &str, val: &str) {
    let args = js_sys::Array::new();
    args.push(&wasm_bindgen::JsValue::from_str(cmd));
    args.push(&wasm_bindgen::JsValue::from_bool(false));
    if !val.is_empty() { args.push(&wasm_bindgen::JsValue::from_str(val)); }
    if let Some(f) = js_sys::Reflect::get(doc, &wasm_bindgen::JsValue::from_str("execCommand"))
        .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok())
    { let _ = f.apply(doc, &args); }
}

fn apply_block_shortcut(win: &web_sys::Window, doc: &web_sys::Document, e: &KeyboardEvent) {
    let sel = match win.get_selection().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    if sel.range_count() == 0 { return; }
    let range = match sel.get_range_at(0) {
        Ok(r) => r,
        Err(_) => return,
    };
    let container = match range.start_container() {
        Ok(c) => c,
        Err(_) => return,
    };
    let offset = range.start_offset().unwrap_or(0) as usize;
    let text = container.text_content().unwrap_or_default();
    let prefix = &text[..offset.min(text.len())];
    let is_newline = e.key() == "Enter";

    if prefix.chars().all(|c| c == '#') && prefix.len() >= 1 && prefix.len() <= 6 {
        let level = prefix.len();
        if let Ok(mut r) = doc.create_range() {
            r.set_start(&container, 0u32).ok();
            r.set_end(&container, prefix.len() as u32).ok();
            sel.remove_all_ranges().ok();
            sel.add_range(&r).ok();
        }
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", &format!("h{}", level));
        e.prevent_default();
        return;
    }

    if !is_newline { return; }

    if prefix == "- " || prefix == "* " {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertUnorderedList", "");
        e.prevent_default();
        return;
    }

    if prefix == "> " {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", "blockquote");
        e.prevent_default();
        return;
    }

    if prefix.len() > 2 && prefix[..prefix.len()-1].chars().all(|c| c.is_ascii_digit() || c == '.') && prefix.ends_with(". ") {
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertOrderedList", "");
        e.prevent_default();
    }
}

fn apply_inline_formatting(win: &web_sys::Window, doc: &web_sys::Document) {
    let sel = match win.get_selection().ok().flatten() {
        Some(s) => s,
        None => return,
    };
    if sel.range_count() == 0 { return; }
    let range = match sel.get_range_at(0) {
        Ok(r) => r,
        Err(_) => return,
    };
    let container = match range.start_container() {
        Ok(c) => c,
        Err(_) => return,
    };
    let offset = range.start_offset().unwrap_or(0) as usize;
    let full_text = container.text_content().unwrap_or_default();
    let before = &full_text[..offset.min(full_text.len())];

    // **bold**
    if let Some(start) = before.rfind("**") {
        let between = &before[start + 2..];
        if let Some(inner_end) = between.find("**") {
            let inner = &between[..inner_end];
            let inner_start = start + 2;
            if !inner.is_empty() {
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, inner_start as u32).ok();
                    r.set_end(&container, (inner_start + inner.len()) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "bold", "");
                }
                let close_pos = inner_start + inner.len();
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, close_pos as u32).ok();
                    r.set_end(&container, (close_pos + 2) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "delete", "");
                }
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, start as u32).ok();
                    r.set_end(&container, (start + 2) as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "delete", "");
                }
                return;
            }
        }
    }

    // *italic* (single, not part of **)
    if let Some(start) = before.rfind('*') {
        if start == 0 || before.as_bytes()[start - 1] != b'*' {
            let between = &before[start + 1..];
            if let Some(inner_end) = between.find('*') {
                let inner = &between[..inner_end];
                let inner_start = start + 1;
                if !inner.is_empty() {
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, inner_start as u32).ok();
                        r.set_end(&container, (inner_start + inner.len()) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "italic", "");
                    }
                    let close_pos = inner_start + inner.len();
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, close_pos as u32).ok();
                        r.set_end(&container, (close_pos + 1) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "delete", "");
                    }
                    if let Ok(mut r) = doc.create_range() {
                        r.set_start(&container, start as u32).ok();
                        r.set_end(&container, (start + 1) as u32).ok();
                        sel.remove_all_ranges().ok();
                        sel.add_range(&r).ok();
                        exec_cmd(doc, "delete", "");
                    }
                }
            }
        }
    }

    // `code`
    if let Some(start) = before.rfind('`') {
        let between = &before[start + 1..];
        if let Some(inner_end) = between.find('`') {
            let inner = &between[..inner_end];
            let inner_start = start + 1;
            if !inner.is_empty() {
                let code_html = format!("<code>{}</code>", inner.replace('<', "&lt;").replace('>', "&gt;"));
                let full_end = inner_start + inner.len() + 1;
                if let Ok(mut r) = doc.create_range() {
                    r.set_start(&container, start as u32).ok();
                    r.set_end(&container, full_end as u32).ok();
                    sel.remove_all_ranges().ok();
                    sel.add_range(&r).ok();
                    exec_cmd(doc, "insertHTML", &code_html);
                }
            }
        }
    }
}

/// Roda depois de `set_inner_html`, ciclo 121:
///
/// 1. Troca `<a href="*.pdf">` (link markdown normal, `[texto](x.pdf)`)
///    por um wrapper `.pdf-embed` com um `<iframe>` dentro — pdf num
///    frame próprio com scroll interno, em vez de um link que abriria
///    em outro lugar. `html_to_md.rs` reconhece o wrapper (via
///    `data-pdf-href`/`data-pdf-text`) e serializa de volta pro MESMO
///    `[texto](x.pdf)` original ao salvar — ver ciclo 111 (cuidado
///    extra de round-trip depois do bug de tabela).
/// 2. Resolve `<img src="assets/...">` e o `data-asset-src` do iframe
///    recém-criado pra uma `data:` URL de verdade — um `src` relativo
///    cru (`assets/x.png`) resolve contra a origem do webview
///    (`http://localhost:1420/...` em dev), não contra a pasta real do
///    vault no disco, então SEM ISSO nenhuma imagem embutida jamais
///    aparecia — bug pré-existente (provavelmente desde a introdução
///    do slash command `/img`), corrigido de quebra aqui.
fn upgrade_embedded_assets_at(el: &web_sys::Element, vault_path: String) {
    let doc = el.owner_document();

    if let Ok(links) = el.query_selector_all("a[href]") {
        for i in 0..links.length() {
            let Some(node) = links.item(i) else { continue };
            let Ok(a) = node.dyn_into::<web_sys::Element>() else { continue };
            let Some(href) = a.get_attribute("href") else { continue };
            let is_local_pdf = href.to_lowercase().ends_with(".pdf")
                && !href.starts_with("http://")
                && !href.starts_with("https://")
                && !href.starts_with("data:")
                && !href.starts_with(crate::wikilink::SCHEME_PREFIX);
            if !is_local_pdf {
                continue;
            }
            let Some(ref doc) = doc else { continue };
            let Ok(wrapper) = doc.create_element("div") else { continue };
            let _ = wrapper.set_attribute("class", "pdf-embed");
            let _ = wrapper.set_attribute("data-pdf-href", &href);
            let _ = wrapper.set_attribute("data-pdf-text", &a.text_content().unwrap_or_default());
            let Ok(iframe) = doc.create_element("iframe") else { continue };
            let _ = iframe.set_attribute("class", "pdf-embed__frame");
            let _ = iframe.set_attribute("data-asset-src", &href);
            let _ = wrapper.append_child(&iframe);
            if let Some(parent) = a.parent_node() {
                let _ = parent.replace_child(&wrapper, &a);
            }
        }
    }

    if let Ok(assets) = el.query_selector_all("img[src^='assets/'], iframe[data-asset-src]") {
        for i in 0..assets.length() {
            let Some(node) = assets.item(i) else { continue };
            let Ok(target) = node.dyn_into::<web_sys::Element>() else { continue };
            let is_iframe = target.tag_name().eq_ignore_ascii_case("iframe");
            let asset_path = if is_iframe {
                target.get_attribute("data-asset-src")
            } else {
                target.get_attribute("src")
            };
            let Some(asset_path) = asset_path else { continue };
            let vault_path = vault_path.clone();
            let target = target.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(data_url) = api::read_asset_data_url(&vault_path, &asset_path).await {
                    let _ = target.set_attribute("src", &data_url);
                }
            });
        }
    }
}

fn init_mermaid_at(el: &web_sys::Element) {
    if let Some(window) = web_sys::window() {
        if let Some(mermaid) = js_sys::Reflect::get(&window, &wasm_bindgen::JsValue::from_str("mermaid"))
            .ok().and_then(|v| v.dyn_into::<js_sys::Object>().ok())
        {
            let config = js_sys::Object::new();
            js_sys::Reflect::set(&config, &wasm_bindgen::JsValue::from_str("theme"), &wasm_bindgen::JsValue::from_str("dark")).ok();
            js_sys::Reflect::set(&config, &wasm_bindgen::JsValue::from_str("startOnLoad"), &wasm_bindgen::JsValue::from_bool(false)).ok();
            js_sys::Reflect::set(&mermaid, &wasm_bindgen::JsValue::from_str("initialize"), &config).ok();
            let run = js_sys::Reflect::get(&mermaid, &wasm_bindgen::JsValue::from_str("run"))
                .ok().and_then(|v| v.dyn_into::<js_sys::Function>().ok());
            if let Some(run_fn) = run {
                let opts = js_sys::Object::new();
                js_sys::Reflect::set(&opts, &wasm_bindgen::JsValue::from_str("nodes"), el).ok();
                let _ = run_fn.call1(&wasm_bindgen::JsValue::null(), &opts);
            }
        }
    }
}
