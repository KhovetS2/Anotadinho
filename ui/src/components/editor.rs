//! Editor WYSIWYG contenteditable + slash commands + markdown live formatting.

use base64::Engine;
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use yew::prelude::*;
use web_sys::KeyboardEvent;

use crate::api::{self, PageMeta};
use crate::components::embeds::InlineEmbed;
use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::components::properties_panel::PropertiesPanel;
use crate::dialog::PendingDialog;
use crate::embed::DocSegment;
use crate::state;

/// Conteúdo que chegou do disco enquanto havia edição local pendente
/// (ciclo 190).
///
/// Guardar o CONTEÚDO (e não só um aviso) é o que permite mostrar a
/// diferença e recarregar sem ir ao disco de novo — entre o aviso e a
/// decisão o arquivo pode ter mudado outra vez, e recarregar algo
/// diferente do que foi mostrado seria pior que não mostrar nada.
#[derive(Clone, PartialEq)]
struct ConflitoExterno {
    conteudo: String,
    versao: Option<String>,
}

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
    /// `true` enquanto a sessão de navegação por blocos está ativa
    /// (ciclo 194). Os atalhos de bloco (`d`, `y`, `n`, `K`, `J`, `c`)
    /// SÓ valem aqui — em digitação eles são letras comuns.
    pub nav_mode_active: bool,
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
    /// Abre a paleta de comandos já preenchida — repassado aos embeds
    /// (ação `run-search` do embed de ações, ciclo 156).
    #[prop_or_default]
    pub on_search: Callback<String>,
    /// Muda quando o watcher acusa alteração no vault (ciclo 173).
    #[prop_or_default]
    pub vault_version: u32,
    /// Abre uma sessão de navegação no nível dos BLOCOS da página
    /// (ciclo 174) — o editor pede isso quando o Escape sobe do texto.
    #[prop_or_default]
    pub on_enter_block_nav: Callback<()>,
    /// O inverso do `on_enter_block_nav`: encerra a sessão de nav-mode
    /// porque o editor acabou de entrar em digitação (ciclo 185).
    pub on_leave_block_nav: Callback<()>,
}

/// Um item do menu `/`. `action` é ou HTML pra inserir no cursor, ou
/// uma sentinela (`__IMG__`, `__MERMAID__`, `__ASSET__`) tratada à parte
/// em `select_slash`, ou `__EMBED__:<type>` pros tipos de embed.
#[derive(Clone, PartialEq)]
struct SlashItem {
    label: &'static str,
    desc: &'static str,
    icon: &'static str,
    action: String,
}

/// Prefixo da sentinela de embed. Um só braço em `select_slash` cobre
/// todos os tipos — antes era uma sentinela hardcoded por tipo
/// (`__EMBED_KANBAN__` etc) com o corpo YAML inicial cravado literal lá
/// dentro, o que fazia cada embed novo tocar em 3 pontos do editor.
const EMBED_PREFIX: &str = "__EMBED__:";

static SLASH_BLOCKS: &[(&str, &str, &str, &str)] = &[
    ("Título 1", "Título grande", "heading", "<h1>Título</h1>"),
    ("Título 2", "Título médio", "heading", "<h2>Título</h2>"),
    ("Título 3", "Título pequeno", "heading", "<h3>Título</h3>"),
    ("Lista", "Lista com marcadores", "list", "<ul><li>Item</li></ul>"),
    ("Checklist", "Lista de tarefas", "check-square", "<ul><li><input type='checkbox'> Tarefa</li></ul>"),
    ("Citação", "Bloco de citação", "quote", "<blockquote>Citação</blockquote>"),
    ("Código", "Bloco de código", "code", "<pre><code>código</code></pre>"),
    ("Tabela", "Tabela 3×2", "table", "<table><tr><td>A</td><td>B</td><td>C</td></tr><tr><td></td><td></td><td></td></tr></table>"),
    ("Linha", "Divisor horizontal", "minus", "<hr>"),
    ("Imagem", "URL ou arquivo de imagem", "image", "__IMG__"),
    ("Diagrama", "Mermaid (fluxograma)", "network", "__MERMAID__"),
    ("Assets", "Inserir arquivo do vault", "paperclip", "__ASSET__"),
];

/// Monta a lista do menu `/`: os blocos markdown fixos acima + um item
/// por tipo de embed, gerado de `EmbedKind::all()`. Um embed novo
/// aparece no menu sozinho, sem tocar neste arquivo.
fn slash_items() -> Vec<SlashItem> {
    let mut items: Vec<SlashItem> = SLASH_BLOCKS
        .iter()
        .map(|(label, desc, icon, action)| SlashItem {
            label,
            desc,
            icon,
            action: (*action).to_string(),
        })
        .collect();
    items.extend(crate::embed::EmbedKind::all().iter().map(|kind| SlashItem {
        label: kind.label(),
        desc: kind.desc(),
        icon: kind.icon(),
        action: format!("{EMBED_PREFIX}{}", kind.type_name()),
    }));
    items
}

/// Data de hoje em `YYYY-MM-DD` — usada pelo corpo inicial do embed de
/// calendário (`EmbedKind::default_body`).
fn today_iso() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

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
    // Marca de versão do arquivo lido (ciclo 173). Fica num `use_mut_ref`
    // e não num `use_state` porque quem lê é o `persist`, chamado de
    // dentro de efeitos/timers que capturariam um handle congelado —
    // mesmo motivo do `edited_ref` aqui em cima.
    let file_version_ref = use_mut_ref(|| None::<String>);

    // Undo/redo genérico: pilha de snapshots de markdown inteiro
    // (`anotadinho_core::history::History`, ciclo 186), não um mecanismo
    // por tipo de embed — cobre texto solto E qualquer mutação de embed
    // (mover card, editar evento, etc) com uma implementação só, já que
    // TODA mutação passa por `mark_edited` (ponto único desde o ciclo
    // 074). O histórico guarda o último markdown que `mark_edited` viu —
    // não é o mesmo que `content_md` (que nem sempre é atualizado em
    // sync, ver `on_edit`). `render_gen` força o Effect 2
    // (abaixo) a reinjetar o HTML mesmo quando path/has_embeds/
    // segment_count não mudaram — sem isso, desfazer/refazer atualizava
    // `content_md` (embeds declarativos refletiam certo) mas os trechos
    // de markdown solto injetados via `set_inner_html` ficavam com o
    // texto antigo na tela.
    // Conflito com o disco (ciclo 190): guarda o conteúdo que chegou de
    // fora enquanto havia edição local pendente. `Some` = a barra de
    // decisão está na tela.
    let conflito = use_state(|| None::<ConflitoExterno>);
    // Texto local no momento em que a diferença foi aberta. Recalculado
    // do DOM e não lido de `content_md`: a fonte de verdade do texto
    // digitado é o DOM (`content_md` só é atualizado em algumas
    // transições), e um comparativo que não mostra o que a PESSOA
    // escreveu é pior que nenhum.
    let conflito_meu_texto = use_state(String::new);

    // Histórico de desfazer/refazer (ciclo 186): o tipo mora no core,
    // testado fora do WASM. O que fica aqui é só a decisão de AGRUPAR,
    // que depende de relógio.
    let historico = use_mut_ref(|| anotadinho_core::history::History::new(String::new()));
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

    let all_slash_items = slash_items();
    let filtered: Vec<usize> = all_slash_items.iter().enumerate()
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
        let historico = historico.clone();
        let file_version_ref = file_version_ref.clone();

        use_effect_with(page.clone(), move |page| {
            // Histórico de undo/redo é por página — trocar de página não
            // deveria deixar "desfazer" aplicar uma edição de outra
            // página bem diferente.
            historico.borrow_mut().reiniciar(String::new());
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
                let historico_load = historico.clone();
                let file_version_ref = file_version_ref.clone();
                loading.set(true);
                error.set(None);
                edited.set(false);
                *edited_ref.borrow_mut() = false;
                pending_flush_ref.borrow_mut().clear();
                wasm_bindgen_futures::spawn_local(async move {
                    match api::read_page_versioned(&vault_path, &path).await {
                        Ok(page) => {
                            *file_version_ref.borrow_mut() = page.version;
                            historico_load.borrow_mut().reiniciar(page.content.clone());
                            content_md.set(page.content.clone());
                            saved_content.set(page.content);
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
                let caminho_para_transclusao = current_path.clone();
                *last_rendered.borrow_mut() = (current_path, has_embeds_eff, segment_count, render_gen_val);

                if has_embeds_eff {
                    for (i, seg) in segments_eff.iter().enumerate() {
                        if let DocSegment::Markdown(text) = seg {
                            if let Some(div) = segment_refs_eff.get(i).and_then(|r| r.cast::<web_sys::Element>()) {
                                div.set_inner_html(&crate::markdown_render::render(text));
                                upgrade_embedded_assets_at(&div, vault_path_eff.clone());
                                upgrade_transclusions_at(&div, vault_path_eff.clone(), caminho_para_transclusao.clone());
                                marcar_blocos(&div);
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
                    upgrade_transclusions_at(&div, vault_path_eff.clone(), caminho_para_transclusao.clone());
                    marcar_blocos(&div);
                    let _div = div.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(200)).await;
                        init_mermaid_at(&_div);
                    });
                }
                init_highlight();

                // Veio da busca com um alvo dentro de um embed (ciclo
                // 188): rola até ele e destaca. Depois do
                // `set_inner_html` mas com folga, porque os embeds são
                // componentes Yew — eles não estão no DOM no instante
                // em que este efeito roda.
                if let Some(ancora) = crate::nav_mode::tomar_alvo_de_busca() {
                    wasm_bindgen_futures::spawn_local(async move {
                        gloo_timers::future::sleep(std::time::Duration::from_millis(260)).await;
                        crate::nav_mode::revelar_alvo_de_busca(&ancora);
                    });
                }
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
                                .filter(|hit| hit.path != current_path)
                                .map(|hit| (hit.path, hit.snippet))
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
    // Ref à parte pro CONTEÚDO do menu (não o wrapper, que também tem
    // o botão "⋯" que abre/fecha) — mesma razão dos menus da
    // `HeaderBar` (ciclo 125).
    let editor_menu_content_ref = use_node_ref();
    // Devolve o foco pro botão "⋯" ao fechar via Escape (ciclo 136,
    // mesmo tratamento da `HeaderBar` — evita o foco cair fora da
    // árvore de qualquer coisa que dependa dele, ex: nav-mode).
    let editor_menu_toggle_ref = use_node_ref();
    let toggle_editor_menu = { let m = editor_menu_open.clone(); Callback::from(move |_| m.set(!*m)) };
    {
        let editor_menu_open = editor_menu_open.clone();
        let editor_menu_ref = editor_menu_ref.clone();
        let editor_menu_content_ref = editor_menu_content_ref.clone();
        let editor_menu_toggle_ref = editor_menu_toggle_ref.clone();
        use_effect_with(*editor_menu_open, move |open| {
            let mut listeners = Vec::new();
            if *open {
                crate::menu_keyboard::focus_first_item(&editor_menu_content_ref);
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
                    let editor_menu_content_ref = editor_menu_content_ref.clone();
                    let editor_menu_toggle_ref = editor_menu_toggle_ref.clone();
                    EventListener::new(&window, "keydown", move |e| {
                        if let Some(e) = e.dyn_ref::<web_sys::KeyboardEvent>() {
                            match e.key().as_str() {
                                // Escape é tratado pelo `escape_consumer`
                                // abaixo (ciclo 161) — aqui ele seguiria
                                // até o `app.rs` e fecharia a página.
                                "Escape" => {}
                                "ArrowDown" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&editor_menu_content_ref, 1);
                                }
                                "ArrowUp" => {
                                    e.prevent_default();
                                    crate::menu_keyboard::move_item_focus(&editor_menu_content_ref, -1);
                                }
                                _ => {}
                            }
                        }
                    })
                };
                let escape = {
                    let editor_menu_open = editor_menu_open.clone();
                    let editor_menu_toggle_ref = editor_menu_toggle_ref.clone();
                    crate::menu_keyboard::escape_consumer(move || {
                        editor_menu_open.set(false);
                        if let Some(el) = editor_menu_toggle_ref.cast::<web_sys::HtmlElement>() {
                            let _ = el.focus();
                        }
                    })
                };
                listeners.push(close_on_outside);
                listeners.push(close_on_escape);
                listeners.push(escape);
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
    // Arquivo mudou no disco (ciclo 173). Sem edição pendente,
    // recarrega sozinho — é o que faz o loop agente↔UI ser ao vivo de
    // verdade: o `anotadinho-cli` escreve e a página aberta acompanha.
    // COM edição pendente, não toca em nada e só avisa: sobrescrever o
    // que a pessoa está digitando seria trocar um problema por outro.
    {
        let vault_path = props.vault_path.clone();
        let page_path = page.path.clone();
        let content_md = content_md.clone();
        let saved_content = saved_content.clone();
        let status = status.clone();
        let edited_ref = edited_ref.clone();
        let historico = historico.clone();
        let file_version_ref = file_version_ref.clone();
        let render_gen = render_gen.clone();
        let conflito = conflito.clone();
        use_effect_with(props.vault_version, move |version| {
            if *version == 0 || page_path.is_empty() {
                return;
            }
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(page) = api::read_page_versioned(&vault_path, &page_path).await else {
                    return;
                };
                let known = file_version_ref.borrow().clone();
                if page.version == known || page.version.is_none() {
                    return;
                }
                if *edited_ref.borrow() {
                    // Não toca em nada: sobrescrever o que a pessoa está
                    // digitando seria trocar um problema por outro. Só
                    // levanta a barra de decisão (ciclo 190) — antes
                    // aqui havia só um aviso de texto, sem saída.
                    conflito.set(Some(ConflitoExterno {
                        conteudo: page.content.clone(),
                        versao: page.version.clone(),
                    }));
                    return;
                }
                *file_version_ref.borrow_mut() = page.version;
                // Recarga vinda do DISCO zera o histórico: desfazer
                // depois dela regravaria o arquivo com o conteúdo velho.
                historico.borrow_mut().reiniciar(page.content.clone());
                content_md.set(page.content.clone());
                saved_content.set(page.content);
                // Força a reinjeção do HTML dos segmentos (o guard de
                // render compara path/contagem, que podem não mudar).
                render_gen.set(*render_gen + 1);
                status.set(Some("Recarregado do disco".to_string()));
            });
        });
    }

    let persist = {
        let content_md = content_md.clone(); let saved_content = saved_content.clone();
        let saving = saving.clone(); let error = error.clone(); let status = status.clone();
        let vault_path = props.vault_path.clone(); let page_path = page.path.clone();
        let edited = edited.clone();
        let edited_ref = edited_ref.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        let file_version_ref = file_version_ref.clone();
        let open_dialog = props.open_dialog.clone();
        move |md: String| {
            let saved_content = saved_content.clone(); let saving = saving.clone();
            let error = error.clone(); let status = status.clone();
            let vault_path = vault_path.clone(); let page_path = page_path.clone();
            let content_md = content_md.clone(); let edited = edited.clone();
            let edited_ref = edited_ref.clone();
            let pending_flush_ref = pending_flush_ref.clone();
            let file_version_ref = file_version_ref.clone();
            let open_dialog = open_dialog.clone();
            saving.set(true); error.set(None);
            wasm_bindgen_futures::spawn_local(async move {
                let expected = file_version_ref.borrow().clone();
                match api::write_page_checked(&vault_path, &page_path, &md, expected.as_deref()).await {
                    Ok(new_version) => {
                        *file_version_ref.borrow_mut() = Some(new_version);
                        content_md.set(md.clone()); saved_content.set(md); edited.set(false);
                        *edited_ref.borrow_mut() = false; pending_flush_ref.borrow_mut().clear();
                        status.set(Some("Salvo".to_string()));
                    }
                    // Conflito (ciclo 173): alguém escreveu no arquivo
                    // depois que abrimos — o CLI, um agente, um git pull.
                    // Antes disso a gravação passava por cima em silêncio.
                    // Quem decide é o usuário; o padrão (fechar o diálogo)
                    // é NÃO gravar, que preserva o trabalho do outro lado.
                    Err(e) if e.contains(api::CONFLICT_PREFIX) => {
                        saving.set(false);
                        let vault_path2 = vault_path.clone();
                        let page_path2 = page_path.clone();
                        let content_md2 = content_md.clone();
                        let saved_content2 = saved_content.clone();
                        let edited2 = edited.clone();
                        let edited_ref2 = edited_ref.clone();
                        let file_version_ref2 = file_version_ref.clone();
                        let status2 = status.clone();
                        let error2 = error.clone();
                        open_dialog.emit(PendingDialog::Confirm {
                            message: format!(
                                "\"{page_path}\" mudou no disco depois que você abriu (outro app, o anotadinho-cli ou um git pull).\n\nSalvar por cima descarta a versão do disco. Cancelar mantém o que está no disco e recarrega a página."
                            ),
                            confirm_label: "Salvar por cima".to_string(),
                            on_confirm: Callback::from(move |_| {
                                let (vault_path, page_path, md) = (vault_path2.clone(), page_path2.clone(), md.clone());
                                let content_md = content_md2.clone();
                                let saved_content = saved_content2.clone();
                                let edited = edited2.clone();
                                let edited_ref = edited_ref2.clone();
                                let file_version_ref = file_version_ref2.clone();
                                let status = status2.clone();
                                let error = error2.clone();
                                wasm_bindgen_futures::spawn_local(async move {
                                    match api::write_page_checked(&vault_path, &page_path, &md, None).await {
                                        Ok(v) => {
                                            *file_version_ref.borrow_mut() = Some(v);
                                            content_md.set(md.clone());
                                            saved_content.set(md);
                                            edited.set(false);
                                            *edited_ref.borrow_mut() = false;
                                            status.set(Some("Salvo por cima".to_string()));
                                        }
                                        Err(e) => error.set(Some(e)),
                                    }
                                });
                            }),
                        });
                        return;
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
    let mark_edited_com = {
        let e = edited.clone();
        let save_counter = save_counter.clone();
        let edited_ref = edited_ref.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        let autosave_enabled = props.autosave_enabled;
        let persist = persist.clone();
        let historico = historico.clone();
        let last_snapshot_at = last_snapshot_at.clone();
        move |md: String, estrutural: bool| {
            e.set(true);
            *edited_ref.borrow_mut() = true;
            // Mantém o flush de segurança sempre atualizado, independente
            // do salvamento automático estar ligado — isso é o que evita
            // perder texto ao trocar de página rápido, não o timer de 3s.
            *pending_flush_ref.borrow_mut() = md.clone();

            // Agrupa uma rajada de digitação num passo só de "desfazer",
            // em vez de um passo por tecla. Mutação ESTRUTURAL (inserir,
            // remover, mover, duplicar segmento, mudar dados de embed)
            // NUNCA agrupa: era esse o bug do ciclo 186 — inserir um
            // embed logo depois de digitar caía dentro da janela de
            // agrupamento, o estado pré-inserção sumia do histórico e
            // Ctrl+Z pulava direto pra um estado bem mais antigo.
            let now = js_sys::Date::now();
            let agrupar = !estrutural && (now - *last_snapshot_at.borrow()) <= 800.0;
            if historico.borrow_mut().registrar(md.clone(), agrupar) {
                *last_snapshot_at.borrow_mut() = now;
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

    // Digitação: pode agrupar. Este é o `mark_edited` de sempre, e é o
    // que a maior parte do arquivo continua chamando.
    let mark_edited = {
        let f = mark_edited_com.clone();
        move |md: String| f(md, false)
    };
    // Mutação estrutural: sempre vira um ponto de desfazer próprio.
    let mark_edited_estrutural = {
        let f = mark_edited_com.clone();
        move |md: String| f(md, true)
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
        let historico = historico.clone();
        let render_gen = render_gen.clone();
        let persist = persist.clone();
        Callback::from(move |_: ()| {
            let Some(prev) = historico.borrow_mut().desfazer() else { return };
            content_md.set(prev.clone());
            render_gen.set(*render_gen + 1);
            persist(prev);
        })
    };
    let do_redo = {
        let content_md = content_md.clone();
        let historico = historico.clone();
        let render_gen = render_gen.clone();
        let persist = persist.clone();
        Callback::from(move |_: ()| {
            let Some(next) = historico.borrow_mut().refazer() else { return };
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
        let all_items = all_slash_items.clone();
        let vault_path = props.vault_path.clone();
        let open_dialog = props.open_dialog.clone();
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited_estrutural = mark_edited_estrutural.clone();
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
                let Some(item) = all_items.get(item_idx) else { return };
                match item.action.as_str() {
                    "__IMG__" => {
                        let vault_path = vault_path.clone();
                        let content_md = content_md.clone();
                        let editor_ref = editor_ref.clone();
                        let segment_refs = segment_refs.clone();
                        let mark_edited_estrutural = mark_edited_estrutural.clone();
                        open_dialog.emit(PendingDialog::Prompt {
                            title: "Caminho da imagem ou URL".to_string(),
                            default: String::new(),
                            on_submit: Callback::from(move |path: String| {
                                let content_md = content_md.clone();
                                let editor_ref = editor_ref.clone();
                                let segment_refs = segment_refs.clone();
                                let mark_edited_estrutural = mark_edited_estrutural.clone();
                                if path.starts_with("http") {
                                    let html = format!("<img src=\"{}\" alt=\"imagem\" style=\"max-width:100%;border-radius:8px;\">", path.replace('"', "&quot;"));
                                    if let Some(el) = parse_single_element(&html) {
                                        if insert_element_at_cursor(&el, false) {
                                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                            content_md.set(new_md.clone());
                                            mark_edited_estrutural(new_md);
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
                                                    mark_edited_estrutural(new_md);
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
                        let mark_edited_estrutural = mark_edited_estrutural.clone();
                        open_dialog.emit(PendingDialog::Prompt {
                            title: "Código Mermaid (ex: graph TD; A-->B)".to_string(),
                            default: String::new(),
                            on_submit: Callback::from(move |code: String| {
                                let html = format!("<div class=\"mermaid\">{}</div>", code.replace('<', "&lt;").replace('>', "&gt;"));
                                if let Some(el) = parse_single_element(&html) {
                                    if insert_element_at_cursor(&el, true) {
                                        let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                        content_md.set(new_md.clone());
                                        mark_edited_estrutural(new_md);
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
                        let mark_edited_estrutural = mark_edited_estrutural.clone();
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
                                                        mark_edited_estrutural(new_md);
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
                    // Um braço só pra TODO tipo de embed: o corpo
                    // inicial vem de `EmbedKind::default_body`, então um
                    // tipo novo não toca neste arquivo.
                    action if action.starts_with(EMBED_PREFIX) => {
                        let type_name = &action[EMBED_PREFIX.len()..];
                        if let Some(kind) = crate::embed::EmbedKind::from_type_name(type_name) {
                            let body = kind.default_body(&today_iso());
                            if insert_embed_marker_at_cursor(kind.type_name(), &body) {
                                let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                                content_md.set(new_md.clone());
                                mark_edited_estrutural(new_md);
                            }
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
                                mark_edited_estrutural(new_md);
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
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        Callback::from(move |vi: usize| {
            if let Some((text_node, pos, query)) = find_wikilink_context() {
                delete_range_and_collapse(&text_node, pos, 2 + query.chars().count() as u32);
            }
            if let Some(&page_idx) = items.get(vi) {
                if let Some(page) = pages.get(page_idx) {
                    // `escapar_barra` (ciclo 192): título com `|` viraria
                    // alias na próxima leitura, apontando pro lugar
                    // errado. Quem gera o wikilink escapa; a pessoa nunca
                    // digita isso na mão.
                    let bruto = anotadinho_core::links::escapar_barra(&page.title);
                    let href = format!("{}{}", crate::wikilink::SCHEME_PREFIX, crate::wikilink::encode_title(&bruto));
                    let html = format!("<a href=\"{}\">{}</a>", href, page.title.replace('<', "&lt;").replace('>', "&gt;"));
                    if let Some(el) = parse_single_element(&html) {
                        if insert_element_at_cursor(&el, false) {
                            let new_md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                            content_md.set(new_md.clone());
                            mark_edited_estrutural(new_md);
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
        let on_enter_blocos = props.on_enter_block_nav.clone();
        let on_sair_blocos = props.on_leave_block_nav.clone();
        let em_navegacao = props.nav_mode_active;
        // O modo, calculado uma vez por render — é o que a tabela de
        // atalhos consulta (ciclo 199).
        let modo_atual = Modo::atual(props.nav_mode_active, props.vim_mode_enabled, false);
        let content_md_esc = content_md.clone();
        let editor_ref_esc = editor_ref.clone();
        let segment_refs_esc = segment_refs.clone();
        let mark_edited_esc = mark_edited.clone();
        let mark_edited_bloco = mark_edited_estrutural.clone();
        let frontmatter_novo = frontmatter_text.clone();
        let content_md_ref_copia = content_md.clone();
        let frontmatter_copia = frontmatter_text.clone();
        let titulo_copia = page.title.clone();
        let mark_edited_copia = mark_edited_estrutural.clone();
        let render_gen_copia = render_gen.clone();
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
                    // Escape CANCELA: além de fechar o menu, apaga o
                    // "/consulta" que foi digitado (ciclo 184). Antes
                    // ficava um "/" solto no texto — sem graça quando
                    // você digitou, e pior quando o menu veio do atalho
                    // `n`, que também tinha criado um bloco pra ele.
                    "Escape" => {
                        e.stop_propagation();
                        e.prevent_default();
                        if let Some((no, pos, consulta)) = find_slash_context() {
                            delete_slash_context_and_collapse(&no, pos, consulta.chars().count());
                            let novo = recompute_markdown_from_dom(&content_md_esc, &editor_ref_esc, &segment_refs_esc);
                            content_md_esc.set(novo.clone());
                            mark_edited_esc(novo);
                        }
                        slash_open.set(false);
                        slash_text.set(String::new());
                        slash_idx.set(0);
                    }
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

            // "c" com um BLOCO focado (nav-mode, ciclo 174) copia a
            // referência dele: grava um `^id` naquela linha — e só nela
            // — e põe `![[Página^id]]` na área de transferência
            // (ciclo 176). Só dispara quando o foco está NO BLOCO, não
            // no texto: digitando, "c" é só a letra c.
            if comando_vale(&e, "c", false, modo_atual) {
                if let Some(bloco) = bloco_focado() {
                    e.prevent_default();
                    e.stop_propagation();
                    copiar_referencia(
                        &bloco,
                        &content_md_ref_copia,
                        &frontmatter_copia,
                        &titulo_copia,
                        &mark_edited_copia,
                        &render_gen_copia,
                    );
                    return;
                }
            }

            // "n" com um BLOCO focado abre um bloco NOVO logo abaixo e
            // já traz o menu `/` (ciclo 181): pelo teclado, criar
            // conteúdo dependia de sair do nav-mode, achar o fim do
            // texto e digitar — o mouse tinha o botão "+" de hover e o
            // teclado não tinha equivalente.
            //
            // Não mexe no markdown: põe o cursor no fim do bloco, deixa
            // o `contenteditable` criar o parágrafo (mesma coisa que
            // apertar Enter) e digita "/" — daí o menu de sempre assume,
            // com todos os 9 embeds e os blocos de markdown.
            // Manipular o bloco focado no modo de navegação (ciclo 175):
            // mover, duplicar e apagar SEM sair pro mouse.
            //
            // Age no DOM e recompõe o markdown a partir dele — o mesmo
            // caminho que toda edição de texto já usa. Não precisa de um
            // `contenteditable` por bloco (ver a nota da task): o bloco
            // já é um filho de primeiro nível marcado por
            // `marcar_blocos`, e mover um nó é operação de DOM.
            // Atalhos de BLOCO só no modo de navegação (ciclo 194).
            //
            // Sem esta guarda, digitar a letra `d` no meio de uma frase
            // apagava o bloco inteiro — e digitar depressa apagava um
            // bloco por `d` digitado. Antes do ciclo 175 isso não
            // acontecia por acidente: o elemento focado durante a
            // digitação era o CONTÊINER, então `bloco_focado()` devolvia
            // `None`. Quando o `contenteditable` desceu pro bloco, essa
            // distinção sumiu — e ela nunca deveria ter sido implícita.
            if !e.ctrl_key() && !e.meta_key() {
                // A tabela `ATALHOS` decide SE a tecla é comando neste
                // modo; o `match` decide o que ela faz (ciclo 199).
                let acao = match (e.key().as_str(), e.alt_key()) {
                    ("ArrowUp", true) | ("K", false) => Some(AcaoBloco::Subir),
                    ("ArrowDown", true) | ("J", false) => Some(AcaoBloco::Descer),
                    ("d", false) => Some(AcaoBloco::Apagar),
                    ("y", false) => Some(AcaoBloco::Duplicar),
                    _ => None,
                };
                let acao = acao.filter(|_| comando_vale(&e, &e.key(), e.alt_key(), modo_atual));
                if let (Some(acao), Some(bloco)) = (acao, bloco_focado()) {
                    e.prevent_default();
                    e.stop_propagation();
                    if let Some(indice) = aplicar_acao_de_bloco(&bloco, acao) {
                        let novo = recompute_markdown_from_dom(
                            &content_md_esc,
                            &editor_ref_esc,
                            &segment_refs_esc,
                        );
                        content_md_esc.set(novo.clone());
                        mark_edited_bloco(novo);
                        // O re-render troca os nós do DOM, então o
                        // `focus_item` feito lá dentro morre junto e o
                        // foco cai no `<body>` — daí "apaguei um bloco e
                        // não consigo mais navegar nem sair com Esc"
                        // (ciclo 195). Reancora depois que o DOM novo
                        // existe.
                        refocar_bloco_apos_render(indice);
                    }
                    return;
                }
            }

            if comando_vale(&e, "n", false, modo_atual) {
                if let Some(bloco) = bloco_focado() {
                    e.prevent_default();
                    e.stop_propagation();
                    if entrar_no_bloco(&bloco) {
                        // A sessão de nav-mode acaba aqui — daqui pra
                        // frente é digitação (ciclo 185). Sem isso o
                        // bloco de ORIGEM ficava com o destaque azul
                        // aceso e as setas continuavam pulando de bloco
                        // em vez de andar no texto.
                        sair_do_nav_mode(&on_sair_blocos);
                        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
                            exec_cmd(&doc, "insertParagraph", "");
                            exec_cmd(&doc, "insertText", "/");
                        }
                    }
                    return;
                }
            }

            // Em NAVEGAÇÃO, tecla imprimível que não é comando não pode
            // virar texto (ciclo 197). Fica DEPOIS de todos os comandos de bloco, senão
            // engoliria `d`, `n`, `y`, `K` e `J` antes deles agirem. O bloco continua `contenteditable`
            // e focado, então sem esta guarda qualquer letra solta era
            // inserida no meio do texto — a mesma classe do bug do 194,
            // só que na direção contrária.
            //
            // Só teclas de UM caractere: setas, Enter, Escape, Backspace
            // e afins têm nome longo e seguem pro tratamento normal.
            if modo_atual == Modo::Navegacao
                && !e.ctrl_key()
                && !e.meta_key()
                && !e.alt_key()
                && e.key().chars().count() == 1
                // Comando conhecido já foi tratado acima; o que chega
                // aqui é letra solta, e em navegação ela não é texto.
                && atalho_de(&e.key(), false).is_none()
            {
                e.prevent_default();
                return;
            }


            // Escape com o cursor no texto SOBE pro nível de blocos
            // (ciclo 174) em vez de desselecionar a página, que era o
            // que o handler global fazia. Sem `stop_propagation` os dois
            // aconteceriam na mesma tecla.
            if e.key() == "Escape" {
                if let Some(bloco) = bloco_do_cursor() {
                    e.prevent_default();
                    e.stop_propagation();
                    crate::nav_mode::focus_item(&bloco);
                    on_enter_blocos.emit(());
                    return;
                }
            }

            // Enter e Backspace (ciclos 175 e 194).
            //
            // A divisão de responsabilidade, pedida pelo usuário:
            //   Enter        — quebra de LINHA dentro do bloco
            //   Shift+Enter  — bloco NOVO
            //   Backspace no início — funde com o anterior
            //
            // Antes o Enter criava bloco, o que tirava da pessoa a
            // quebra de linha simples: não havia como escrever duas
            // linhas dentro do mesmo parágrafo.
            //
            // Em lista e tabela o Enter sem shift continua nativo (item
            // novo, célula nova) — ali "linha" JÁ é a unidade. Em bloco
            // de código o Enter nativo insere a quebra dentro do código,
            // e o Shift+Enter FECHA o bloco e abre um parágrafo depois,
            // que é a única saída de um `<pre>` que termina a página.
            // `!em_navegacao` (ciclo 195): em navegação o Enter significa
            // ENTRAR no bloco, e quem trata isso é o `app.rs`. Sem esta
            // guarda o handler daqui fazia quebra de linha e dava
            // `stop_propagation`, então o Enter nunca chegava lá — a
            // pessoa passava a editar SEM sair do modo de navegação, com
            // a barra ainda dizendo NAVEGAÇÃO e as setas ainda pulando
            // de bloco. Era o "dois editores ao mesmo tempo".
            if !em_navegacao && !e.ctrl_key() && !e.meta_key() {
                if let Some(bloco) = bloco_do_cursor() {
                    let tag = bloco.tag_name().to_lowercase();
                    let lista_ou_tabela = matches!(tag.as_str(), "ul" | "ol" | "table");
                    let codigo = tag == "pre";

                    let tratou = if e.key() == "Enter" && e.shift_key() {
                        if codigo {
                            bloco_novo_depois(&bloco)
                        } else {
                            dividir_bloco(&bloco)
                        }
                    } else if e.key() == "Enter" && !lista_ou_tabela && !codigo {
                        quebra_de_linha()
                    } else if e.key() == "Backspace"
                        && !e.shift_key()
                        && !lista_ou_tabela
                        && !codigo
                    {
                        fundir_com_anterior(&bloco)
                    } else {
                        false
                    };

                    if tratou {
                        e.prevent_default();
                        e.stop_propagation();
                        if let Some(pai) = bloco.parent_element() {
                            marcar_blocos(&pai);
                        }
                        let novo = recompute_markdown_from_dom(
                            &content_md_esc,
                            &editor_ref_esc,
                            &segment_refs_esc,
                        );
                        content_md_esc.set(novo.clone());
                        mark_edited_bloco(novo);
                        return;
                    }
                }
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
    // `n` com um EMBED focado (ciclo 184). Fica no CONTÊINER dos
    // segmentos, não no `contenteditable`: os controles de um embed
    // ficam fora de qualquer contenteditable, então a tecla nunca
    // chegava no handler do editor — só o bloco de texto funcionava.
    //
    // Não dá pra pôr cursor "dentro" de um embed, então o bloco novo
    // nasce como um segmento de markdown logo DEPOIS dele — o mesmo que
    // o botão "+" de hover já faz com o mouse.
    let on_segments_keydown = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        let on_sair_blocos = props.on_leave_block_nav.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() != "n" || e.ctrl_key() || e.meta_key() || e.alt_key() {
                return;
            }
            let Some(pos) = segmento_do_embed_focado() else { return };
            e.prevent_default();
            e.stop_propagation();
            sair_do_nav_mode(&on_sair_blocos);
            inserir_segmento_e_abrir_menu(
                pos + 1,
                &content_md,
                &frontmatter_text,
                &mark_edited_estrutural,
            );
        })
    };

    // Grava um campo do frontmatter da página aberta (ciclo 201).
    //
    // Passa pelo MESMO `content_md` que o resto do editor usa — um
    // embed gravando direto no disco competiria com o salvamento normal,
    // e o último a escrever apagaria o outro.
    let on_set_property = {
        let content_md = content_md.clone();
        let pending_flush_ref = pending_flush_ref.clone();
        let mark_edited = mark_edited_estrutural.clone();
        Callback::from(move |(campo, valor): (String, String)| {
            // Lê do `pending_flush_ref`, não do `content_md` capturado:
            // o embed emite `on_change` e `on_set_property` no MESMO
            // tick, e o handle de `use_state` capturado ainda tem o
            // valor de antes — o segundo `set` apagava o primeiro, e a
            // etapa não avançava (ciclo 201).
            let atual = {
                let pendente = pending_flush_ref.borrow().clone();
                if pendente.is_empty() { (*content_md).clone() } else { pendente }
            };
            let Ok(novo) = anotadinho_core::MarkdownCodec::set_frontmatter_field(&atual, &campo, &valor)
            else {
                return;
            };
            content_md.set(novo.clone());
            mark_edited(novo);
        })
    };

    let insert_blank_line = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        move |pos: usize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited_estrutural = mark_edited_estrutural.clone();
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
                mark_edited_estrutural(new_full);

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
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        let open_dialog = props.open_dialog.clone();
        move |pos: usize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited_estrutural = mark_edited_estrutural.clone();
            let open_dialog = open_dialog.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let content_md = content_md.clone();
                let frontmatter_text = frontmatter_text.clone();
                let mark_edited_estrutural = mark_edited_estrutural.clone();
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
                        mark_edited_estrutural(new_full);
                    }),
                });
            })
        }
    };

    // Move o embed uma posição pra cima/baixo trocando de lugar com o
    // segmento vizinho, e duplica o embed logo abaixo. Agem no nível do
    // `DocSegment`, então valem pros 9 tipos de embed com o mesmo
    // código (ciclo 159). A aritmética em si mora no core, testada:
    // `embed::move_segment` / `embed::duplicate_segment`.
    let reorder_embed = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        move |pos: usize, delta: isize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited_estrutural = mark_edited_estrutural.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let full = (*content_md).clone();
                let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full);
                let mut segs = crate::embed::segment(body);
                if !crate::embed::move_segment(&mut segs, pos, delta) {
                    return;
                }
                let new_body = crate::embed::join(&segs);
                let new_full = if frontmatter_text.is_empty() { new_body } else { format!("{}\n{}", frontmatter_text, new_body) };
                content_md.set(new_full.clone());
                mark_edited_estrutural(new_full);
            })
        }
    };

    let duplicate_embed = {
        let content_md = content_md.clone();
        let frontmatter_text = frontmatter_text.clone();
        let mark_edited_estrutural = mark_edited_estrutural.clone();
        move |pos: usize| {
            let content_md = content_md.clone();
            let frontmatter_text = frontmatter_text.clone();
            let mark_edited_estrutural = mark_edited_estrutural.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                let full = (*content_md).clone();
                let (_, body) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&full);
                let mut segs = crate::embed::segment(body);
                if !crate::embed::duplicate_segment(&mut segs, pos) {
                    return;
                }
                let new_body = crate::embed::join(&segs);
                let new_full = if frontmatter_text.is_empty() { new_body } else { format!("{}\n{}", frontmatter_text, new_body) };
                content_md.set(new_full.clone());
                mark_edited_estrutural(new_full);
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
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        let mark_edited = mark_edited.clone();
        Callback::from(move |e: MouseEvent| {
            let Some(target) = e.target() else { return };
            let Ok(el) = target.dyn_into::<web_sys::Element>() else { return };

            // Toggle de checkbox de tarefa — clicar no <input> só dispara
            // "click" no container, nunca "input" (que é quem chama
            // `mark_edited` normalmente), então marcar/desmarcar nunca
            // era persistido sem isso. No momento do "click", o navegador
            // já aplicou o toggle nativo do checkbox (checked mudou antes
            // do evento disparar), então já dá pra reler o DOM direto.
            if el.tag_name().eq_ignore_ascii_case("input")
                && el.get_attribute("type").as_deref() == Some("checkbox")
            {
                let md = recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs);
                content_md.set(md.clone());
                mark_edited(md);
                return;
            }

            let Ok(Some(anchor)) = el.closest("a") else { return };
            let Some(href) = anchor.get_attribute("href") else { return };
            let Some(encoded) = href.strip_prefix(crate::wikilink::SCHEME_PREFIX) else { return };
            e.prevent_default();
            // O href leva o miolo CRU do `[[...]]`; o alvo sai dele.
            let bruto_do_link = crate::wikilink::decode_title(encoded);
            let (title, _alias) = anotadinho_core::links::split_wikilink(&bruto_do_link);
            let vault_path = vault_path.clone();
            let on_page_selected = on_page_selected.clone();
            let open_dialog = open_dialog.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // `scan_vault` e não `list_pages` (ciclo 191): o título
                // que interessa é o do FRONTMATTER, e `list_pages`
                // devolve o nome do ARQUIVO. Por isso `[[Grafo do
                // Vault]]` não abria nada — grafo.md tem esse título no
                // frontmatter, mas o nome do arquivo é "grafo". Só
                // wikilink cujo alvo tem título igual ao nome do arquivo
                // funcionava, o que fazia o bug parecer intermitente.
                //
                // É o mesmo motivo já documentado em
                // `upgrade_transclusions_at`, que foi corrigido no 170;
                // este caminho ficou pra trás.
                let paginas = api::scan_vault(&vault_path).await.unwrap_or_default();
                let achado = resolver_alvo(&paginas, &title).or_else(|| {
                    // Rede de segurança do ciclo 192: `|` é nome de
                    // arquivo válido no POSIX. Se alguém escreveu
                    // `[[estranho|nome]]` sem escapar a barra, o alvo
                    // acima virou "estranho" e não resolveu — antes de
                    // desistir, tenta a string INTEIRA como alvo.
                    //
                    // Custa uma busca a mais só no caminho do erro, e é
                    // o que evita "página não encontrada" num arquivo
                    // que existe.
                    Some(bruto_do_link.as_str())
                        .filter(|b| *b != title)
                        .and_then(|b| resolver_alvo(&paginas, b))
                });
                match achado {
                    Some(entry) => on_page_selected.emit(PageMeta {
                        path: entry.path.clone(),
                        title: entry.title.clone(),
                        section: entry.section.clone(),
                    }),
                    None => open_dialog.emit(PendingDialog::Alert {
                        message: format!("Página \"{}\" não encontrada.", title),
                    }),
                }
            });
        })
    };

    // ── ações da barra de conflito (ciclo 190) ────────────────────

    // Recarrega com o conteúdo QUE FOI MOSTRADO no aviso, não relendo o
    // disco: entre o aviso e o clique o arquivo pode ter mudado de novo,
    // e trazer algo diferente do que a pessoa viu seria pior que nada.
    let conflito_recarregar = {
        let conflito = conflito.clone();
        let conflito_meu_texto = conflito_meu_texto.clone();
        let content_md = content_md.clone();
        let saved_content = saved_content.clone();
        let historico = historico.clone();
        let file_version_ref = file_version_ref.clone();
        let render_gen = render_gen.clone();
        let edited = edited.clone();
        let edited_ref = edited_ref.clone();
        let status = status.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(c) = (*conflito).clone() else { return };
            *file_version_ref.borrow_mut() = c.versao.clone();
            // Recarga do disco zera o histórico: desfazer depois dela
            // regravaria o arquivo com o conteúdo velho.
            historico.borrow_mut().reiniciar(c.conteudo.clone());
            content_md.set(c.conteudo.clone());
            saved_content.set(c.conteudo);
            edited.set(false);
            *edited_ref.borrow_mut() = false;
            render_gen.set(*render_gen + 1);
            status.set(Some("Recarregado do disco".to_string()));
            conflito_meu_texto.set(String::new());
            conflito.set(None);
        })
    };

    // Fica com o texto local. O `file_version_ref` passa a ser o da
    // versão de fora, então o `write_page_checked` do próximo salvamento
    // aceita gravar por cima — que é exatamente o que "manter o meu"
    // quer dizer.
    let conflito_manter = {
        let conflito = conflito.clone();
        let conflito_meu_texto = conflito_meu_texto.clone();
        let file_version_ref = file_version_ref.clone();
        let status = status.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(c) = (*conflito).clone() else { return };
            *file_version_ref.borrow_mut() = c.versao;
            status.set(Some("Mantido o seu — salve pra gravar por cima".to_string()));
            conflito_meu_texto.set(String::new());
            conflito.set(None);
        })
    };

    let conflito_ver_diff = {
        let conflito_meu_texto = conflito_meu_texto.clone();
        let content_md = content_md.clone();
        let editor_ref = editor_ref.clone();
        let segment_refs = segment_refs.clone();
        Callback::from(move |_: MouseEvent| {
            if conflito_meu_texto.is_empty() {
                conflito_meu_texto
                    .set(recompute_markdown_from_dom(&content_md, &editor_ref, &segment_refs));
            } else {
                conflito_meu_texto.set(String::new());
            }
        })
    };

    let modo = Modo::atual(props.nav_mode_active, props.vim_mode_enabled, *vim_insert);

    html! {
        <main class="editor">
            <header class="editor__header">
                <h2 class="editor__title">{ &page.title }</h2>
                <div class="editor__actions">
                    if let Some(ref s) = *status { <span class="editor__status-badge">{ s }</span> }
                    if *edited { <span class="editor__dirty">{ "não salvo" }</span> }
                    <button class="btn btn--primary btn--sm" onclick={do_save.reform(|_| ())} disabled={*saving || !*edited}>{ save_label }</button>
                    <div class="header-menu-wrapper" ref={editor_menu_ref}>
                        <button class="btn btn--ghost btn--sm" ref={editor_menu_toggle_ref} onclick={toggle_editor_menu} title="Mais ações"><Icon name="more-horizontal" /></button>
                        if *editor_menu_open {
                            <div class="header-menu" ref={editor_menu_content_ref}>
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let toggle_home = toggle_home.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); toggle_home.emit(e); })
                                }}>
                                    <Icon name="home" />{ if is_home { " Remover como início" } else { " Definir como início" } }
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
                                    <Icon name="download" />{ " Exportar HTML" }
                                </button>
                                <button class="header-menu__item btn btn--ghost btn--sm" onclick={{
                                    let editor_menu_open = editor_menu_open.clone();
                                    let open_history = open_history.clone();
                                    Callback::from(move |e: MouseEvent| { editor_menu_open.set(false); open_history.emit(e); })
                                }}>
                                    <Icon name="clock" />{ " Histórico" }
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
            if let Some(c) = &*conflito {
                <div class="conflito" role="alert">
                    <div class="conflito__linha">
                        <Icon name="alert-triangle" />
                        <span class="conflito__texto">
                            { "Esta página mudou no disco enquanto você editava." }
                        </span>
                        <button class="btn btn--ghost btn--sm" onclick={conflito_ver_diff.clone()}>
                            { if conflito_meu_texto.is_empty() { "Ver a diferença" } else { "Esconder a diferença" } }
                        </button>
                        <button class="btn btn--ghost btn--sm" onclick={conflito_manter.clone()}>
                            { "Manter o meu" }
                        </button>
                        <button class="btn btn--sm" onclick={conflito_recarregar.clone()}
                            title="Descarta o que você escreveu e traz o conteúdo do disco">
                            { "Recarregar (perde o que você escreveu)" }
                        </button>
                    </div>
                    if !conflito_meu_texto.is_empty() {
                        { render_diff(&conflito_meu_texto, &c.conteudo) }
                    }
                </div>
            }
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
                    <div class="editor__wysiwyg-segments" key="segments" onclick={on_wysiwyg_click.clone()}
                        onkeydown={on_segments_keydown.clone()}>
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
                                        // Ciclo 175: o contêiner NÃO é mais
                                        // editável — cada bloco dentro dele é
                                        // (ver `marcar_blocos`). Os handlers
                                        // ficam aqui mesmo: eventos borbulham
                                        // do bloco, então um handler só continua
                                        // servindo pra todos.
                                        <div class="editor__wysiwyg" {key} data-segment-index={i.to_string()} ref={node_ref} contenteditable="false"
                                            spellcheck="false" onkeydown={on_keydown.clone()} oninput={on_edit.clone()}
                                            ondrop={on_drop.clone()} ondragover={on_dragover.clone()} onpaste={on_paste.clone()} />
                                    }
                                }
                                DocSegment::Embed(data) => {
                                    let content_md = content_md.clone();
                                    let mark_edited_estrutural = mark_edited_estrutural.clone();
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
                                        mark_edited_estrutural(new_full);
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
                                            <div class="embed-hover-wrapper__toolbar">
                                                <button class="embed-hover-wrapper__btn"
                                                    disabled={i == 0}
                                                    onclick={reorder_embed(i, -1)} title="Mover pra cima"><Icon name="arrow-up" /></button>
                                                <button class="embed-hover-wrapper__btn"
                                                    disabled={i + 1 >= segments.len()}
                                                    onclick={reorder_embed(i, 1)} title="Mover pra baixo"><Icon name="arrow-down" /></button>
                                                <button class="embed-hover-wrapper__btn"
                                                    onclick={duplicate_embed(i)} title="Duplicar embed"><Icon name="copy" /></button>
                                                <button class="embed-hover-wrapper__btn embed-hover-wrapper__btn--danger"
                                                    onclick={remove_embed(i)} title="Remover embed"><Icon name="x" /></button>
                                            </div>
                                            <InlineEmbed
                                                nav_group={format!("embed-{i}")}
                                                data={data.clone()}
                                                vault_path={props.vault_path.clone()}
                                                on_change={on_change}
                                                open_dialog={props.open_dialog.clone()}
                                                on_page_selected={props.on_page_selected.clone()}
                                                on_search={props.on_search.clone()}
                                                on_set_property={on_set_property.clone()}
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
                    // Caminho da página SEM embed. Também
                    // `contenteditable="false"` (ciclo 194): a reescrita do
                    // 175 só trocou o caminho com embeds, e aqui ficaram
                    // dois editáveis aninhados — contêiner E bloco. Era o
                    // que fazia o Enter num bloco vazio criar parágrafo no
                    // lugar errado e o bloco de origem crescer junto.
                    <div class="editor__wysiwyg" key="plain" ref={editor_ref} contenteditable="false"
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
                            let item = &all_slash_items[item_idx];
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
                                    <Icon name={item.icon} class="slash-menu__item-icon" />
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
                                    <span class="wikilink-menu__item-icon"><Icon name="file-text" /></span>
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
                        <Icon name="link" />{ format!(" Backlinks ({})", backlinks.len()) }
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
                // Indicador de MODO, no espírito do vim (ciclo 194): a
                // pessoa precisa saber quais teclas estão sendo
                // capturadas. O bug que motivou isto foi digitar `d` no
                // meio de uma frase e ver um bloco sumir — sem nada na
                // tela dizendo que `d` era um comando naquele momento.
                <span class={classes!("editor__modo", modo.classe())} title={modo.dica()}>
                    { modo.rotulo() }
                </span>
                <span>{ format!("{} palavras · {} caracteres", word_count, char_count) }</span>
                if props.vim_mode_enabled {
                    <span class={ if *vim_insert { "editor__vim-mode editor__vim-mode--insert" } else { "editor__vim-mode editor__vim-mode--normal" } }>
                        { if *vim_insert { "-- INSERT --" } else { "-- NORMAL --" } }
                    </span>
                }
                <span class="editor__statusbar-hint">{ modo.atalhos() }</span>
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
    // `Selection.modify` move o caret mas, diferente do comportamento
    // nativo de seta do navegador numa página comum, NÃO rola o
    // container sozinho — reportado pelo usuário: navegar pra baixo no
    // vim mode saía da área visível do editor sem o scroll acompanhar.
    vim_scroll_caret_into_view();
}

/// Rola o ancestral do caret pra dentro da área visível — chamado
/// depois de todo `vim_move`. Usa o elemento mais próximo do container
/// da seleção (o nó do range pode ser um nó de texto, sem
/// `scrollIntoView` próprio) com `block: "nearest"`, mesmo critério já
/// usado pra manter o item destacado visível na sidebar/paleta (rola o
/// mínimo pra reaparecer, sem centralizar à toa a cada tecla).
fn vim_scroll_caret_into_view() {
    let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok()).flatten() else { return };
    if sel.range_count() == 0 {
        return;
    }
    let Ok(range) = sel.get_range_at(0) else { return };
    let Ok(node) = range.start_container() else { return };
    let Some(el) = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element()) else { return };
    let opts = web_sys::ScrollIntoViewOptions::new();
    opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
    el.scroll_into_view_with_scroll_into_view_options(&opts);
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
    let is_space = e.key() == " ";

    if prefix.chars().all(|c| c == '#') && !prefix.is_empty() && prefix.len() <= 6 {
        let level = prefix.len();
        select_prefix(doc, &sel, &container, prefix.len());
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", &format!("h{}", level));
        e.prevent_default();
        return;
    }

    // Os atalhos abaixo disparam no espaço, não no Enter: o gatilho real
    // é digitar o marcador ("-", ">", "1.") e seguir digitando o texto
    // do item, igual ao heading acima — não "digitar o marcador e
    // apertar Enter sem nada no meio" (o `prefix` no momento do Enter é
    // a linha inteira já digitada, quase nunca bate com o literal
    // esperado). No momento do `keydown` do espaço, o espaço em si AINDA
    // não foi inserido no DOM, então o prefixo aqui não tem o espaço à
    // direita (mesmo motivo do heading comparar só com "#", não "# ").
    if !is_space { return; }

    // Checkbox: "[]" ou "[ ]" — checado ANTES do "-"/"*" solto pra
    // cobrir tanto o caso solto num parágrafo quanto o combo "digitar
    // '- ' (vira lista) e depois '[] ' dentro do item" (lista +
    // checkbox juntos, pedido explícito do usuário).
    if prefix == "[]" || prefix == "[ ]" {
        select_prefix(doc, &sel, &container, prefix.len());
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertHTML", "<input type=\"checkbox\">");
        e.prevent_default();
        return;
    }

    if prefix == "-" || prefix == "*" {
        select_prefix(doc, &sel, &container, prefix.len());
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertUnorderedList", "");
        e.prevent_default();
        return;
    }

    if prefix == ">" {
        select_prefix(doc, &sel, &container, prefix.len());
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "formatBlock", "blockquote");
        e.prevent_default();
        return;
    }

    if prefix.len() > 1 && prefix[..prefix.len()-1].chars().all(|c| c.is_ascii_digit()) && prefix.ends_with('.') {
        select_prefix(doc, &sel, &container, prefix.len());
        exec_cmd(doc, "delete", "");
        exec_cmd(doc, "insertOrderedList", "");
        e.prevent_default();
    }
}

/// Seleciona `[0, len)` do `container` de texto — usado pelos atalhos de
/// bloco acima pra marcar o marcador digitado (`"#"`, `"-"`, `">"`,
/// `"1."`) antes de apagá-lo com `exec_cmd(doc, "delete", "")`.
fn select_prefix(doc: &web_sys::Document, sel: &web_sys::Selection, container: &web_sys::Node, len: usize) {
    if let Ok(r) = doc.create_range() {
        r.set_start(container, 0u32).ok();
        r.set_end(container, len as u32).ok();
        sel.remove_all_ranges().ok();
        sel.add_range(&r).ok();
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

/// Marca cada filho de primeiro nível do contenteditable como um item de
/// navegação (ciclo 174): é o que dá ao nav-mode algo pra destacar e
/// percorrer DENTRO do editor.
///
/// Os blocos são derivados da renderização — nada é escrito no `.md`.
/// Roda logo depois do `set_inner_html`, então pega exatamente os
/// elementos que o markdown gerou (`<p>`, `<h1..h6>`, `<ul>`, `<ol>`,
/// `<blockquote>`, `<pre>`, `<table>`, `<hr>`, imagem solta).
fn marcar_blocos(container: &web_sys::Element) {
    // Segmento sem bloco nenhum não teria onde receber cursor desde que
    // o `contenteditable` desceu pro bloco (ciclo 175) — uma página nova
    // ficava literalmente impossível de digitar. Um parágrafo vazio é o
    // mínimo pra existir um alvo.
    if container.children().length() == 0 {
        if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
            if let Ok(p) = doc.create_element("p") {
                if let Ok(br) = doc.create_element("br") {
                    let _ = p.append_child(&br);
                }
                let _ = container.append_child(&p);
            }
        }
    }

    let filhos = container.children();
    for i in 0..filhos.length() {
        let Some(bloco) = filhos.item(i) else { continue };
        let _ = bloco.set_attribute("data-nav-item", &format!("bloco-{i}"));
        let _ = bloco.set_attribute("data-nav-parent", crate::nav_mode::GRUPO_BLOCOS);
        let _ = bloco.set_attribute(crate::nav_mode::ATTR_BLOCO_TEXTO, "texto");
        // Cada bloco é seu próprio `contenteditable` (ciclo 175). A TAG
        // continua sendo a original (`<p>`, `<h1>`, `<ul>`, `<pre>`...),
        // então `html_to_md` não muda uma linha e o markdown gerado
        // continua idêntico — foi o que permitiu fazer esta troca sem
        // reescrever a serialização junto.
        let _ = bloco.set_attribute("contenteditable", "true");
        let _ = bloco.class_list().add_1("editor__bloco");
        let _ = bloco.class_list().remove_1(CLASSE_CONVITE);
        // `tabindex` pra o foco poder pousar no bloco sem que ele vire
        // um alvo de Tab (que continua andando pelos controles, não
        // pelo texto).
        let _ = bloco.set_attribute("tabindex", "-1");
    }

    marcar_convite();
}

/// Põe o cursor DENTRO do bloco indicado e devolve o foco ao
/// contenteditable — o "entrar em inserção" do ciclo 174.
pub fn entrar_no_bloco(bloco: &web_sys::Element) -> bool {
    // O próprio bloco é o editável desde o ciclo 175 — antes o foco ia
    // pro contêiner do segmento e o cursor era posicionado por range.
    let Some(editavel) = bloco.dyn_ref::<web_sys::HtmlElement>() else {
        return false;
    };
    let _ = editavel.focus();
    let Some(window) = web_sys::window() else { return false };
    let Some(doc) = window.document() else { return false };
    let Ok(range) = doc.create_range() else { return false };
    let _ = range.select_node_contents(bloco);
    range.collapse_with_to_start(false);
    if let Some(sel) = window.get_selection().ok().flatten() {
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(&range);
    }
    true
}

/// Elemento de bloco (filho direto do contenteditable) que contém o
/// cursor agora — o alvo pra onde o Escape devolve o foco no nível de
/// blocos (ciclo 174).
fn bloco_do_cursor() -> Option<web_sys::Element> {
    let sel = web_sys::window()?.get_selection().ok()??;
    let node = sel.anchor_node()?;
    let el = node
        .dyn_ref::<web_sys::Element>()
        .cloned()
        .or_else(|| node.parent_element())?;
    let seletor = format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO);

    if let Ok(Some(bloco)) = el.closest(&seletor) {
        return Some(bloco);
    }
    // Cursor colapsado no PRÓPRIO contenteditable (acontece quando ele
    // acabou de receber foco, antes de entrar num filho): o bloco é o
    // filho na posição do offset — sem isso o Escape não achava bloco
    // nenhum e caía no handler global, que fecha a página.
    let container = el.closest(".editor__wysiwyg").ok().flatten()?;
    let filhos = container.children();
    let idx = sel.anchor_offset().min(filhos.length().saturating_sub(1));
    filhos.item(idx).or_else(|| filhos.item(0))
}

/// Preenche os marcadores de transclusão (`![[Página]]`, ciclo 170) com
/// o conteúdo real da página alvo.
///
/// Roda depois do `set_inner_html`, uma busca por marcador. O conteúdo
/// entra RENDERIZADO mas somente leitura — editar continua sendo na
/// página de origem, que é onde o texto de verdade mora.
///
/// Ciclo (A inclui B que inclui A) para no primeiro nível: o conteúdo
/// transcluído NÃO é varrido de novo por marcadores, então nada se
/// aninha infinitamente. Auto-referência é barrada explicitamente,
/// porque é o erro mais fácil de cometer.
fn upgrade_transclusions_at(el: &web_sys::Element, vault_path: String, pagina_atual: String) {
    let Ok(marcadores) = el.query_selector_all("[data-transclusao]") else { return };
    for i in 0..marcadores.length() {
        let Some(node) = marcadores.item(i) else { continue };
        let Ok(alvo_el) = node.dyn_into::<web_sys::Element>() else { continue };
        let Some(alvo) = alvo_el.get_attribute("data-transclusao") else { continue };
        if alvo_el.has_attribute("data-transcluido") {
            continue;
        }
        let _ = alvo_el.set_attribute("data-transcluido", "1");

        // `Página#Seção` recorta um heading; `Página^bloco` recorta UMA
        // linha, pelo id do ciclo 176.
        let (titulo, secao, bloco) = if let Some((t, b)) = alvo.split_once('^') {
            (t.trim().to_string(), None, Some(b.trim().to_string()))
        } else if let Some((t, s)) = alvo.split_once('#') {
            (t.trim().to_string(), Some(s.trim().to_string()), None)
        } else {
            (alvo.trim().to_string(), None, None)
        };
        let vault_path = vault_path.clone();
        let pagina_atual = pagina_atual.clone();
        let alvo_el = alvo_el.clone();
        wasm_bindgen_futures::spawn_local(async move {
            // `scan_vault` (e não `list_pages`) porque o título que
            // interessa é o do FRONTMATTER — `list_pages` devolve o nome
            // do arquivo, então `![[Guia do Agent OS]]` nunca casaria
            // com `guia-agent-os.md`.
            let paginas = crate::api::scan_vault(&vault_path).await.unwrap_or_default();
            let encontrada = paginas
                .iter()
                .find(|p| p.title.eq_ignore_ascii_case(&titulo))
                .or_else(|| paginas.iter().find(|p| p.path == titulo))
                .or_else(|| {
                    paginas.iter().find(|p| {
                        std::path::Path::new(&p.path)
                            .file_stem()
                            .map(|s| s.to_string_lossy().eq_ignore_ascii_case(&titulo))
                            .unwrap_or(false)
                    })
                });
            let Some(pagina) = encontrada else {
                alvo_el.set_inner_html(&format!(
                    "<p class=\"transclusao__vazia\">Página <strong>{}</strong> não existe ainda.</p>",
                    escape_html(&titulo)
                ));
                return;
            };
            if pagina.path == pagina_atual {
                alvo_el.set_inner_html(
                    "<p class=\"transclusao__vazia\">Uma página não pode transcluir ela mesma.</p>",
                );
                return;
            }
            let Ok(conteudo) = crate::api::read_page(&vault_path, &pagina.path).await else {
                alvo_el.set_inner_html(&format!(
                    "<p class=\"transclusao__vazia\">Não consegui ler {}.</p>",
                    escape_html(&pagina.path)
                ));
                return;
            };
            let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
            if let Some(id) = &bloco {
                match anotadinho_core::links::find_block(corpo, id) {
                    Some(texto) => {
                        alvo_el.set_inner_html(&format!(
                            "<a class=\"transclusao__origem\" href=\"{}{}\">{} › ^{}</a><div class=\"transclusao__corpo\">{}</div>",
                            crate::wikilink::SCHEME_PREFIX,
                            crate::wikilink::encode_title(&pagina.title),
                            escape_html(&pagina.title),
                            escape_html(id),
                            crate::markdown_render::render(texto)
                        ));
                    }
                    None => alvo_el.set_inner_html(&format!(
                        "<p class=\"transclusao__vazia\">A página <strong>{}</strong> não tem o bloco <strong>^{}</strong>.</p>",
                        escape_html(&pagina.title),
                        escape_html(id)
                    )),
                }
                return;
            }
            let corpo = match &secao {
                Some(s) => match anotadinho_core::links::extract_section(corpo, s) {
                    Some(trecho) => trecho.to_string(),
                    None => {
                        alvo_el.set_inner_html(&format!(
                            "<p class=\"transclusao__vazia\">A página <strong>{}</strong> não tem a seção <strong>{}</strong>.</p>",
                            escape_html(&pagina.title),
                            escape_html(s)
                        ));
                        return;
                    }
                },
                None => corpo.to_string(),
            };

            // Embed dentro de página transcluída: o conteúdo entra como
            // HTML, então um kanban viraria YAML solto no meio do texto.
            // Vira um aviso com link pra origem — ver Notas da task 170.
            let segmentos = crate::embed::segment(&corpo);
            let mut html = String::new();
            for seg in &segmentos {
                match seg {
                    crate::embed::DocSegment::Markdown(texto) => {
                        html.push_str(&crate::markdown_render::render(texto));
                    }
                    crate::embed::DocSegment::Embed(dados) => {
                        html.push_str(&format!(
                            "<p class=\"transclusao__embed\">Bloco <strong>{}</strong> — abra a página pra usar.</p>",
                            dados.kind().type_name()
                        ));
                    }
                }
            }
            let cabecalho = format!(
                "<a class=\"transclusao__origem\" href=\"{}{}\">{}{}</a>",
                crate::wikilink::SCHEME_PREFIX,
                crate::wikilink::encode_title(&pagina.title),
                escape_html(&pagina.title),
                secao.as_deref().map(|s| format!(" › {}", escape_html(s))).unwrap_or_default()
            );
            alvo_el.set_inner_html(&format!("{cabecalho}<div class=\"transclusao__corpo\">{html}</div>"));
        });
    }
}

/// Escapa texto que vai pro HTML montado à mão aqui.
fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// Bloco de texto atualmente FOCADO pelo nav-mode (ciclo 174) — `None`
/// quando o foco está no texto (aí o cursor manda, não o bloco).
fn bloco_focado() -> Option<web_sys::Element> {
    let ativo = web_sys::window()?.document()?.active_element()?;
    if ativo.has_attribute(crate::nav_mode::ATTR_BLOCO_TEXTO) {
        Some(ativo)
    } else {
        None
    }
}

/// Grava o `^id` na linha do bloco (se ainda não tiver) e copia
/// `![[Página^id]]`.
///
/// A linha é achada pelo TEXTO do bloco, não por posição: o índice do
/// filho no DOM não corresponde a linha do markdown (um parágrafo pode
/// ocupar várias linhas, uma lista ocupa uma por item). Se o texto não
/// for encontrado, nada é gravado — melhor não fazer do que marcar a
/// linha errada.
fn copiar_referencia(
    bloco: &web_sys::Element,
    content_md: &UseStateHandle<String>,
    frontmatter: &str,
    titulo_pagina: &str,
    mark_edited: &impl Fn(String),
    render_gen: &UseStateHandle<u32>,
) {
    let texto = bloco.text_content().unwrap_or_default();
    let primeira = texto.lines().next().unwrap_or("").trim().to_string();
    if primeira.is_empty() {
        return;
    }
    let completo = (**content_md).clone();
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&completo);
    let alvo = corpo
        .lines()
        .position(|l| anotadinho_core::links::strip_block_id(l).trim().contains(&primeira));
    let Some(alvo) = alvo else { return };
    let Some((novo_corpo, id)) = anotadinho_core::links::garantir_block_id(corpo, alvo) else {
        return;
    };

    if novo_corpo != corpo {
        let novo_completo = if frontmatter.is_empty() {
            novo_corpo
        } else {
            format!("{}\n{}", frontmatter, novo_corpo)
        };
        content_md.set(novo_completo.clone());
        mark_edited(novo_completo);
        // Força reinjetar o HTML: o guard de render compara path e
        // contagem de segmentos, que não mudaram — e sem reinjetar, o
        // `^id` fica só no estado e some no próximo salvamento, que
        // recompõe o markdown a partir do DOM.
        render_gen.set(**render_gen + 1);
    }
    copiar_para_area_de_transferencia(&format!("![[{titulo_pagina}^{id}]]"));
}

/// Copia texto usando um `<textarea>` temporário + `execCommand`.
///
/// `navigator.clipboard` exigiria permissão e uma feature a mais do
/// `web-sys`; este caminho usa o mesmo `execCommand` que o editor já
/// usa e funciona no WebView do Tauri.
fn copiar_para_area_de_transferencia(texto: &str) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(el) = doc.create_element("textarea") else { return };
    let Ok(area) = el.dyn_into::<web_sys::HtmlTextAreaElement>() else { return };
    area.set_value(texto);
    let _ = area.style().set_property("position", "fixed");
    let _ = area.style().set_property("opacity", "0");
    if let Some(body) = doc.body() {
        let _ = body.append_child(&area);
        area.select();
        exec_cmd(&doc, "copy", "");
        let _ = body.remove_child(&area);
    }
}

/// Índice do SEGMENTO do embed que está focado agora — `None` quando o
/// foco não está num embed.
///
/// O id do grupo é `embed-<índice do segmento>` (ciclo 165), então o
/// número sai dali em vez de precisar de outro atributo.
fn segmento_do_embed_focado() -> Option<usize> {
    let ativo = web_sys::window()?.document()?.active_element()?;
    let raiz = ativo
        .closest("[data-nav-group^=\"embed-\"]")
        .ok()
        .flatten()
        .or(Some(ativo))?;
    raiz.get_attribute("data-nav-group")?
        .strip_prefix("embed-")?
        .parse()
        .ok()
}

/// Encerra a sessão de nav-mode porque o editor entrou em digitação
/// (ciclo 185).
///
/// São duas coisas separadas e as DUAS precisam acontecer: apagar o
/// destaque do item (mora no DOM, gerenciado por `nav_mode::focus_item`)
/// e derrubar o estado em `app.rs` (que é quem decide se as setas andam
/// entre blocos ou dentro do texto). Fazer só a primeira deixava as
/// setas presas na navegação; só a segunda deixava o retângulo azul
/// aceso no bloco de origem.
fn sair_do_nav_mode(on_sair: &Callback<()>) {
    crate::nav_mode::clear_item_highlight();
    on_sair.emit(());
}

/// Insere um segmento de markdown vazio na posição `pos` e abre o menu
/// `/` nele (ciclo 184).
///
/// Reconsulta o DOM pelo `data-segment-index` depois de um sleep curto
/// em vez de guardar um `NodeRef`: os refs são recriados a cada
/// renderização, e o elemento novo só existe depois dela — mesma razão
/// documentada no `insert_blank_line`.
fn inserir_segmento_e_abrir_menu(
    pos: usize,
    content_md: &UseStateHandle<String>,
    frontmatter: &str,
    mark_edited: &impl Fn(String),
) {
    let completo = (**content_md).clone();
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&completo);
    let mut segs = crate::embed::segment(corpo);
    let pos = pos.min(segs.len());
    segs.insert(pos, DocSegment::Markdown(crate::embed::BLANK_SEGMENT.to_string()));
    let novo_corpo = crate::embed::join(&segs);
    let novo = if frontmatter.is_empty() {
        novo_corpo
    } else {
        format!("{}\n{}", frontmatter, novo_corpo)
    };
    content_md.set(novo.clone());
    mark_edited(novo);

    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::sleep(std::time::Duration::from_millis(80)).await;
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        let seletor = format!("[data-segment-index=\"{pos}\"]");
        let Some(segmento) = doc.query_selector(&seletor).ok().flatten() else { return };
        // O foco vai no BLOCO, não no segmento: desde o ciclo 175 o
        // contêiner do segmento é `contenteditable="false"`, então focar
        // nele não põe cursor em lugar nenhum e o "/" digitado logo
        // abaixo se perdia.
        let el = segmento
            .query_selector(".editor__bloco")
            .ok()
            .flatten()
            .unwrap_or(segmento);
        if let Ok(html_el) = el.clone().dyn_into::<web_sys::HtmlElement>() {
            let _ = html_el.focus();
        }
        // Cursor no fim do segmento novo e menu aberto pelo caminho de
        // sempre.
        if let Ok(range) = doc.create_range() {
            let _ = range.select_node_contents(&el);
            range.collapse_with_to_start(false);
            if let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) {
                let _ = sel.remove_all_ranges();
                let _ = sel.add_range(&range);
            }
        }
        exec_cmd(&doc, "insertText", "/");
    });
}

/// Desenha o comparativo entre o texto local e o que está no disco
/// (ciclo 190).
///
/// Só as linhas que MUDARAM, mais uma de contexto em volta: uma página
/// grande com uma linha alterada não deve virar uma parede de texto
/// idêntico onde a mudança se perde.
fn render_diff(local: &str, disco: &str) -> Html {
    let linhas = anotadinho_core::diff::diff_linhas(local, disco);
    let (removidas, adicionadas) = anotadinho_core::diff::contar(&linhas);

    let relevante: Vec<bool> = (0..linhas.len())
        .map(|i| {
            let ini = i.saturating_sub(1);
            let fim = (i + 2).min(linhas.len());
            linhas[ini..fim].iter().any(|l| l.mudou())
        })
        .collect();

    html! {
        <div class="conflito__diff">
            <p class="conflito__resumo">
                { format!("{removidas} linha(s) sua(s) · {adicionadas} do disco") }
            </p>
            <pre class="conflito__pre">
                { for linhas.iter().enumerate().filter(|(i, _)| relevante[*i]).map(|(_, l)| {
                    let (classe, marca) = match l {
                        anotadinho_core::diff::LinhaDiff::Igual { .. } => ("conflito__l", " "),
                        anotadinho_core::diff::LinhaDiff::Removida { .. } => ("conflito__l conflito__l--meu", "-"),
                        anotadinho_core::diff::LinhaDiff::Adicionada { .. } => ("conflito__l conflito__l--disco", "+"),
                    };
                    html! { <div class={classe}>{ format!("{marca}{}", l.texto()) }</div> }
                }) }
            </pre>
        </div>
    }
}

/// Acha a página que um alvo de wikilink aponta (ciclo 192).
///
/// A ordem importa e vai do mais específico pro mais tolerante:
/// caminho exato resolve sem ambiguidade nenhuma, título do frontmatter
/// é o jeito natural de escrever, e o nome do arquivo é o que sobra pra
/// página sem `title:`.
fn resolver_alvo<'a>(
    paginas: &'a [anotadinho_core::PageIndexEntry],
    alvo: &str,
) -> Option<&'a anotadinho_core::PageIndexEntry> {
    let alvo = alvo.trim();
    paginas
        .iter()
        .find(|p| p.path.eq_ignore_ascii_case(alvo))
        .or_else(|| {
            // `pages/produto/grafo` (sem `.md`) também vale.
            paginas.iter().find(|p| {
                p.path
                    .strip_suffix(".md")
                    .is_some_and(|sem| sem.eq_ignore_ascii_case(alvo))
            })
        })
        .or_else(|| paginas.iter().find(|p| p.title.eq_ignore_ascii_case(alvo)))
        .or_else(|| {
            paginas.iter().find(|p| {
                std::path::Path::new(&p.path)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .is_some_and(|s| s.eq_ignore_ascii_case(alvo))
            })
        })
}

/// O que fazer com o bloco focado no modo de navegação (ciclo 175).
#[derive(Clone, Copy, PartialEq)]
enum AcaoBloco {
    Subir,
    Descer,
    Duplicar,
    Apagar,
}

/// Aplica a ação no DOM e devolve o ÍNDICE pra onde o foco deve ir
/// depois do re-render — `None` quando nada mudou.
///
/// Devolver o índice (e não `true`) é o que corrige o foco depois de
/// mover: quem chama capturava a posição ANTES da ação, e ao subir um
/// bloco a posição nova é outra — o foco pousava no vizinho que tomou o
/// lugar antigo, e a digitação seguinte ia pro bloco errado (ciclo 195).
///
/// Mexe no DOM, não no `Vec<DocSegment>`: um bloco de texto não é um
/// segmento, é um filho de primeiro nível DENTRO de um segmento de
/// markdown, e o markdown é recomposto a partir do DOM na gravação
/// (`recompute_markdown_from_dom`). Fazer no nível do markdown exigiria
/// mapear bloco → intervalo de linhas, que é justamente o que o ciclo
/// 176 evitou não escrevendo id nenhum no arquivo.
///
/// O foco é mantido no bloco (ou no vizinho, quando ele é apagado) pra
/// dar pra encadear as ações — subir um bloco três posições é `K K K`,
/// não `K`, achar de novo, `K`.
fn aplicar_acao_de_bloco(bloco: &web_sys::Element, acao: AcaoBloco) -> Option<usize> {
    let doc = web_sys::window().and_then(|w| w.document())?;
    let pai = bloco.parent_element()?;

    match acao {
        AcaoBloco::Subir => {
            let anterior = bloco.previous_element_sibling()?;
            let _ = pai.insert_before(bloco, Some(&anterior));
        }
        AcaoBloco::Descer => {
            let proximo = bloco.next_element_sibling()?;
            // Insere DEPOIS do próximo: `insert_before(proximo.next)`.
            let depois = proximo.next_element_sibling();
            let _ = pai.insert_before(bloco, depois.as_ref().map(|e| e.unchecked_ref()));
        }
        AcaoBloco::Duplicar => {
            let copia = bloco.clone_node_with_deep(true).ok()?;
            let _ = pai.insert_before(&copia, bloco.next_element_sibling().as_ref().map(|e| e.unchecked_ref()));
        }
        AcaoBloco::Apagar => {
            // Vizinho pra onde o foco vai: o de baixo, ou o de cima se
            // era o último. Sem isso o foco cai no `<body>` e as setas
            // param de andar.
            let vizinho = bloco
                .next_element_sibling()
                .or_else(|| bloco.previous_element_sibling());
            let _ = pai.remove_child(bloco);
            let indice = match &vizinho {
                Some(v) => {
                    crate::nav_mode::focus_item(v);
                    indice_do_bloco(v)
                }
                // Apagou o único bloco: `marcar_blocos` cria um vazio no
                // lugar, e o foco vai pra ele.
                None => 0,
            };
            let _ = doc;
            return Some(indice);
        }
    }

    // Os blocos foram reordenados: os `data-nav-item` precisam voltar a
    // bater com a posição, senão a navegação pula na ordem antiga.
    marcar_blocos(&pai);
    crate::nav_mode::focus_item(bloco);
    // Índice DEPOIS da mutação — o DOM já está na ordem nova aqui.
    Some(indice_do_bloco(bloco))
}

/// Enter num bloco: divide no cursor, ou cria um bloco vazio depois
/// quando o cursor está no fim (ciclo 175).
///
/// Devolve `false` quando não há o que fazer, pra o handler deixar o
/// comportamento nativo seguir.
fn dividir_bloco(bloco: &web_sys::Element) -> bool {
    let Some(win) = web_sys::window() else { return false };
    let Some(doc) = win.document() else { return false };
    let Some(pai) = bloco.parent_element() else { return false };
    let Some(sel) = win.get_selection().ok().flatten() else { return false };
    let Ok(range) = sel.get_range_at(0) else { return false };

    // O que fica DEPOIS do cursor vira o bloco novo. `extract_contents`
    // já remove essa parte do bloco atual, então não dá pra duplicar
    // conteúdo por engano.
    let Ok(resto) = doc.create_range() else { return false };
    let _ = resto.select_node_contents(bloco);
    let (Ok(fim_c), Ok(fim_o)) = (range.end_container(), range.end_offset()) else {
        return false;
    };
    if resto.set_start(&fim_c, fim_o).is_err() {
        return false;
    }
    let Ok(fragmento) = resto.extract_contents() else { return false };

    // Bloco novo com a MESMA tag: dividir um `<h2>` no meio dá dois
    // headings, que é o que se espera. Menos o caso do fim de um
    // heading, onde o natural é começar a escrever texto comum.
    let tag = bloco.tag_name().to_lowercase();
    let vazio = fragmento
        .text_content()
        .map(|t| t.trim().is_empty())
        .unwrap_or(true);
    let tag_nova = if vazio && tag.starts_with('h') { "p" } else { &tag };
    let Ok(novo) = doc.create_element(tag_nova) else { return false };
    let _ = novo.append_child(&fragmento);
    if novo.text_content().unwrap_or_default().is_empty() {
        // Bloco sem nada não recebe cursor no WebKit; o `<br>` é o
        // truque de sempre pra ele ter altura e ser clicável.
        if let Ok(br) = doc.create_element("br") {
            let _ = novo.append_child(&br);
        }
    }
    let _ = pai.insert_before(&novo, bloco.next_sibling().as_ref());

    if let Some(html) = novo.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html.set_attribute("contenteditable", "true");
        let _ = html.focus();
    }
    if let Ok(r) = doc.create_range() {
        let _ = r.select_node_contents(&novo);
        r.collapse_with_to_start(true);
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(&r);
    }
    true
}

/// Backspace no INÍCIO de um bloco: funde com o anterior (ciclo 175).
///
/// Fora do início devolve `false` — apagar caractere é trabalho do
/// navegador, e reimplementar isso seria trocar código testado por
/// código novo sem ganho.
fn fundir_com_anterior(bloco: &web_sys::Element) -> bool {
    let Some(win) = web_sys::window() else { return false };
    let Some(sel) = win.get_selection().ok().flatten() else { return false };
    let Ok(range) = sel.get_range_at(0) else { return false };
    if !range.collapsed() {
        return false;
    }
    // Só age se o cursor estiver colado no começo do bloco.
    let Some(doc) = win.document() else { return false };
    let Ok(ate_aqui) = doc.create_range() else { return false };
    let _ = ate_aqui.select_node_contents(bloco);
    let Ok(fim_c) = range.start_container() else { return false };
    let Ok(fim_o) = range.start_offset() else { return false };
    if ate_aqui.set_end(&fim_c, fim_o).is_err() {
        return false;
    }
    if !ate_aqui.to_string().as_string().unwrap_or_default().is_empty() {
        return false;
    }

    let Some(anterior) = bloco.previous_element_sibling() else {
        // Primeiro bloco do segmento: não faz nada, como manda a task.
        return false;
    };

    // Cursor na junta ANTES de mover o conteúdo, senão ele acaba no fim
    // do texto que veio junto.
    let comprimento = anterior.text_content().unwrap_or_default().len() as u32;
    let _ = anterior.insert_adjacent_html("beforeend", &bloco.inner_html());
    if let Some(pai) = bloco.parent_element() {
        let _ = pai.remove_child(bloco);
    }
    if let Some(html) = anterior.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html.focus();
    }
    if let (Ok(r), Some(no)) = (doc.create_range(), primeiro_texto(&anterior)) {
        let offset = comprimento.min(no.text_content().unwrap_or_default().len() as u32);
        if r.set_start(&no, offset).is_ok() {
            r.collapse_with_to_start(true);
            let _ = sel.remove_all_ranges();
            let _ = sel.add_range(&r);
        }
    }
    true
}

/// Primeiro nó de texto de um elemento — onde pousar o cursor.
fn primeiro_texto(el: &web_sys::Element) -> Option<web_sys::Node> {
    let filhos = el.child_nodes();
    for i in 0..filhos.length() {
        let no = filhos.item(i)?;
        if no.node_type() == 3 {
            return Some(no);
        }
        if let Some(el_filho) = no.dyn_ref::<web_sys::Element>() {
            if let Some(achado) = primeiro_texto(el_filho) {
                return Some(achado);
            }
        }
    }
    None
}

/// Quebra de linha DENTRO do bloco (ciclo 194) — o Enter sem shift.
fn quebra_de_linha() -> bool {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return false };
    exec_cmd(&doc, "insertLineBreak", "");
    true
}

/// Cria um parágrafo vazio DEPOIS do bloco e põe o cursor nele.
///
/// É a saída de um bloco de código (Shift+Enter num `<pre>`): dividir um
/// `<pre>` daria dois blocos de código, e o que se quer é sair dele.
fn bloco_novo_depois(bloco: &web_sys::Element) -> bool {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return false };
    let Some(pai) = bloco.parent_element() else { return false };
    let Ok(novo) = doc.create_element("p") else { return false };
    if let Ok(br) = doc.create_element("br") {
        let _ = novo.append_child(&br);
    }
    let _ = novo.set_attribute("contenteditable", "true");
    let _ = pai.insert_before(&novo, bloco.next_sibling().as_ref());
    if let Some(html) = novo.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html.focus();
    }
    if let (Ok(r), Some(sel)) = (
        doc.create_range(),
        web_sys::window().and_then(|w| w.get_selection().ok().flatten()),
    ) {
        let _ = r.select_node_contents(&novo);
        r.collapse_with_to_start(true);
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(&r);
    }
    true
}

/// Uma tecla que o editor trata, e em que MODO ela vale (ciclo 199).
///
/// Existe porque os três bugs seguidos dos ciclos 194, 195 e 197 foram o
/// MESMO defeito estrutural: uma tecla tratada no modo errado. Cada
/// atalho tinha sua própria condição solta dentro de um `on_keydown` de
/// centenas de linhas, e nada obrigava a responder "isto vale em qual
/// modo?" — dava pra esquecer, e esqueceu-se três vezes.
///
/// Com a tabela, essa pergunta vira DADO. E o harness pode gerar os
/// cenários de "esta tecla não pode disparar aqui" a partir dela, em vez
/// de alguém lembrar de escrever um por um.
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Atalho {
    /// `e.key()` exato.
    pub tecla: &'static str,
    /// Precisa de Alt?
    pub alt: bool,
    /// Modo em que a tecla é COMANDO. Fora dele, é texto.
    pub modo: Modo,
    /// O que faz — só pra documentação e pro harness.
    pub descricao: &'static str,
}

/// Tudo que o editor captura como comando, e onde.
///
/// Ordem de leitura: se você for acrescentar um atalho, acrescente aqui
/// primeiro. Um atalho que não está nesta tabela não deveria existir no
/// `on_keydown`.
pub const ATALHOS: &[Atalho] = &[
    Atalho { tecla: "n", alt: false, modo: Modo::Navegacao, descricao: "bloco novo com o menu /" },
    Atalho { tecla: "c", alt: false, modo: Modo::Navegacao, descricao: "copiar referência do bloco" },
    Atalho { tecla: "d", alt: false, modo: Modo::Navegacao, descricao: "apagar bloco" },
    Atalho { tecla: "y", alt: false, modo: Modo::Navegacao, descricao: "duplicar bloco" },
    Atalho { tecla: "K", alt: false, modo: Modo::Navegacao, descricao: "mover bloco pra cima" },
    Atalho { tecla: "J", alt: false, modo: Modo::Navegacao, descricao: "mover bloco pra baixo" },
    Atalho { tecla: "ArrowUp", alt: true, modo: Modo::Navegacao, descricao: "mover bloco pra cima" },
    Atalho { tecla: "ArrowDown", alt: true, modo: Modo::Navegacao, descricao: "mover bloco pra baixo" },
];

impl Atalho {
    /// A tecla é comando NESTE modo?
    pub fn vale_em(&self, modo: Modo) -> bool {
        self.modo == modo
    }
}

/// Procura o atalho de uma tecla, se ela for comando em algum modo.
pub fn atalho_de(tecla: &str, alt: bool) -> Option<&'static Atalho> {
    ATALHOS.iter().find(|a| a.tecla == tecla && a.alt == alt)
}

/// Modo da aplicação (ciclo 194).
///
/// Existe pra a pessoa SABER quais teclas estão sendo capturadas, e pra
/// o código ter um lugar único onde essa pergunta é respondida. A ordem
/// de precedência importa: navegação vence os outros porque é ela que
/// sequestra letras comuns.
///
/// Modo novo entra aqui e ganha rótulo, cor e lista de atalhos de graça.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Modo {
    /// Teclas são COMANDOS.
    Navegacao,
    /// Modo normal do vim.
    VimNormal,
    /// Teclas são TEXTO.
    Edicao,
}

impl Modo {
    pub fn atual(nav: bool, vim_ligado: bool, vim_insert: bool) -> Self {
        if nav {
            Self::Navegacao
        } else if vim_ligado && !vim_insert {
            Self::VimNormal
        } else {
            Self::Edicao
        }
    }

    fn rotulo(&self) -> &'static str {
        match self {
            Self::Navegacao => "NAVEGAÇÃO",
            Self::VimNormal => "NORMAL",
            Self::Edicao => "EDIÇÃO",
        }
    }

    fn classe(&self) -> &'static str {
        match self {
            Self::Navegacao => "editor__modo--navegacao",
            Self::VimNormal => "editor__modo--normal",
            Self::Edicao => "editor__modo--edicao",
        }
    }

    fn atalhos(&self) -> &'static str {
        match self {
            Self::Navegacao => "setas movem · Enter entra · n novo · d apaga · y duplica · K/J move",
            Self::VimNormal => "h j k l movem · i insere",
            Self::Edicao => "Enter quebra linha · Shift+Enter novo bloco · / insere · Esc navega",
        }
    }

    fn dica(&self) -> &'static str {
        match self {
            Self::Navegacao => "Teclas são COMANDOS neste modo",
            Self::VimNormal => "Modo normal do vim",
            Self::Edicao => "Teclas são TEXTO neste modo",
        }
    }
}

/// Posição do bloco entre os irmãos — o que sobrevive a um re-render,
/// já que os nós em si não sobrevivem.
fn indice_do_bloco(bloco: &web_sys::Element) -> usize {
    let mut i = 0;
    let mut atual = bloco.previous_element_sibling();
    while let Some(el) = atual {
        i += 1;
        atual = el.previous_element_sibling();
    }
    i
}

/// Devolve o foco (e o destaque do nav-mode) a um bloco depois que o
/// re-render substituiu o DOM (ciclo 195).
///
/// Se o índice não existir mais — apagou o último bloco — pousa no
/// último que existe. O importante é NUNCA deixar o foco no `<body>`:
/// dali as setas e o Escape do nav-mode não têm em quê se ancorar, e o
/// modo fica preso sem saída.
fn refocar_bloco_apos_render(indice: usize) {
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::sleep(std::time::Duration::from_millis(80)).await;
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
        let Ok(blocos) = doc.query_selector_all(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
        else {
            return;
        };
        if blocos.length() == 0 {
            return;
        }
        let alvo = indice.min(blocos.length() as usize - 1);
        if let Some(el) = blocos
            .item(alvo as u32)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        {
            crate::nav_mode::focus_item(&el);
        }
    });
}

/// Classe do bloco que mostra o convite "Digite ou use / para inserir".
const CLASSE_CONVITE: &str = "editor__bloco--convite";

/// Marca o convite quando a PÁGINA tem um bloco só, e ele está vazio.
///
/// Precisa ser decidido aqui e não no CSS: `:only-child` conta filhos do
/// SEGMENTO, e uma página com embeds tem vários segmentos — cada um com
/// seu bloco de texto. Pela regra do CSS, um parágrafo vazio no meio da
/// página satisfazia `:only-child` do seu próprio segmento e a mensagem
/// aparecia no meio da escrita (ciclo 195).
fn marcar_convite() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(blocos) = doc.query_selector_all(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
    else {
        return;
    };
    for i in 0..blocos.length() {
        if let Some(el) = blocos.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
            let _ = el.class_list().remove_1(CLASSE_CONVITE);
        }
    }
    if blocos.length() != 1 {
        return;
    }
    let Some(unico) = blocos.item(0).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) else {
        return;
    };
    if unico.text_content().unwrap_or_default().trim().is_empty() {
        let _ = unico.class_list().add_1(CLASSE_CONVITE);
    }
}

/// A tecla do evento é um comando VÁLIDO neste modo? (ciclo 199)
///
/// Um lugar só respondendo isso, em vez de cada atalho repetir a mesma
/// condição — foi a repetição que deixou passar os bugs dos ciclos 194,
/// 195 e 197.
fn comando_vale(e: &KeyboardEvent, tecla: &str, alt: bool, modo: Modo) -> bool {
    if e.ctrl_key() || e.meta_key() || e.key() != tecla || e.alt_key() != alt {
        return false;
    }
    atalho_de(tecla, alt).is_some_and(|a| a.vale_em(modo))
}

#[cfg(test)]
mod testes_atalhos {
    use super::*;

    #[test]
    fn nenhum_atalho_repetido() {
        // Duas entradas pra mesma tecla+alt tornariam `atalho_de`
        // dependente da ORDEM da tabela, que é exatamente o tipo de
        // regra implícita que este ciclo veio remover.
        for (i, a) in ATALHOS.iter().enumerate() {
            for b in &ATALHOS[i + 1..] {
                assert!(
                    !(a.tecla == b.tecla && a.alt == b.alt),
                    "atalho duplicado: {} (alt={})",
                    a.tecla,
                    a.alt
                );
            }
        }
    }

    #[test]
    fn todo_atalho_de_bloco_e_de_navegacao() {
        // Se um atalho de bloco aparecer marcado como `Edicao`, ele volta
        // a disparar durante a digitação — o bug do ciclo 194.
        for a in ATALHOS {
            assert_eq!(
                a.modo,
                Modo::Navegacao,
                "o atalho {} não é de navegação; se isso for intencional, o teste precisa mudar junto",
                a.tecla
            );
        }
    }

    #[test]
    fn letra_comum_nao_e_atalho() {
        for tecla in ["a", "e", "x", "z", "q", "t"] {
            assert!(atalho_de(tecla, false).is_none(), "{tecla} virou comando sem querer");
        }
    }

    #[test]
    fn atalho_so_vale_no_proprio_modo() {
        let n = atalho_de("n", false).expect("n é atalho");
        assert!(n.vale_em(Modo::Navegacao));
        assert!(!n.vale_em(Modo::Edicao));
        assert!(!n.vale_em(Modo::VimNormal));
    }

    #[test]
    fn setas_com_alt_sao_comandos_e_sem_alt_nao() {
        assert!(atalho_de("ArrowUp", true).is_some());
        assert!(atalho_de("ArrowUp", false).is_none(), "seta sem Alt é navegação, não move bloco");
    }

    #[test]
    fn modo_atual_respeita_a_precedencia() {
        assert_eq!(Modo::atual(true, true, false), Modo::Navegacao, "navegação vence o vim");
        assert_eq!(Modo::atual(false, true, false), Modo::VimNormal);
        assert_eq!(Modo::atual(false, true, true), Modo::Edicao, "vim em insert é edição");
        assert_eq!(Modo::atual(false, false, false), Modo::Edicao);
    }

    #[test]
    fn todo_atalho_tem_descricao() {
        // A descrição vira a lista de atalhos da barra de modo e o
        // relatório do harness — atalho sem ela é atalho invisível.
        for a in ATALHOS {
            assert!(!a.descricao.trim().is_empty(), "{} sem descrição", a.tecla);
        }
    }
}
