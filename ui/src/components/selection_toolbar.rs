//! Barra flutuante que aparece ao selecionar texto (ciclo 234).
//!
//! Até aqui o editor não tinha formatação nenhuma por interface: negrito
//! e itálico só saíam digitando `**` e `*`, e quem não sabia markdown não
//! tinha como descobrir que existiam. O menu `/` só oferece BLOCOS.
//!
//! Marca aplicada é HTML no DOM, e o markdown é recomposto do DOM ao
//! salvar — então só entra aqui marca que o `html_to_md` sabe devolver
//! pro arquivo (`strong`, `em`, `s`, `code`, `a`). Marca que não
//! sobrevive ao autosave é pior que marca nenhuma: some sozinha três
//! segundos depois, e ninguém entende por quê.
//!
//! Nada de `execCommand`: ele inventa `<font>` e `<span style>` conforme
//! o motor, e o `html_to_md` não os reconhece. Aqui a seleção é embrulhada
//! na mão, pela API de `Range`.

use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement, Range};
use yew::prelude::*;

use crate::dialog::PendingDialog;

/// Quanto a barra fica acima da seleção.
const FOLGA: f64 = 10.0;

#[derive(Properties, PartialEq, Clone)]
pub struct SelectionToolbarProps {
    /// Raiz do editor. Seleção fora dela não abre a barra — a página de
    /// propostas e a conversa também têm texto selecionável.
    pub editor_ref: NodeRef,
    /// Abre o diálogo do app (usado pra pedir a URL do link).
    pub open_dialog: Callback<PendingDialog>,
}

#[function_component(SelectionToolbar)]
pub fn selection_toolbar(props: &SelectionToolbarProps) -> Html {
    // Onde desenhar. `None` = sem seleção de texto, barra escondida.
    let posicao = use_state(|| None::<(f64, f64)>);
    // A seleção de antes de abrir um modal. Abrir diálogo tira o foco do
    // editor e a seleção se perde; sem guardar, o link seria aplicado
    // em lugar nenhum. Mesmo cuidado do modal de imagens (ciclo 226).
    let guardada = use_mut_ref(|| None::<Range>);

    {
        let posicao = posicao.clone();
        let editor_ref = props.editor_ref.clone();
        use_effect_with((), move |_| {
            let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let ouvinte = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                posicao.set(medir_selecao(&editor_ref));
            });
            let _ = doc.add_event_listener_with_callback(
                "selectionchange",
                ouvinte.as_ref().unchecked_ref(),
            );
            let doc2 = doc.clone();
            Box::new(move || {
                let _ = doc2.remove_event_listener_with_callback(
                    "selectionchange",
                    ouvinte.as_ref().unchecked_ref(),
                );
                drop(ouvinte);
            }) as Box<dyn FnOnce()>
        });
    }

    let Some((x, y)) = *posicao else {
        return html! {};
    };

    // Aplicar marca não avisa o editor por callback: dispara um `input`
    // no bloco, que borbulha até o `oninput` do contêiner. É o mesmo
    // caminho de quando a pessoa digita, com autosave e recomposição do
    // markdown já pendurados — um segundo caminho só divergiria do
    // primeiro com o tempo.
    let marcar = move |tag: &'static str| {
        Callback::from(move |e: MouseEvent| {
            // O clique não pode roubar a seleção antes de a gente usá-la.
            e.prevent_default();
            aplicar_marca(tag);
        })
    };

    let link = {
        let open_dialog = props.open_dialog.clone();
        let guardada = guardada.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            *guardada.borrow_mut() = selecao_atual();
            let guardada = guardada.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Link para onde?".to_string(),
                default: "https://".to_string(),
                on_submit: Callback::from(move |url: String| {
                    let url = url.trim().to_string();
                    if url.is_empty() {
                        return;
                    }
                    if let Some(r) = guardada.borrow().clone() {
                        restaurar(&r);
                    }
                    aplicar_link(&url);
                }),
            });
        })
    };

    let estilo = format!("left: {x}px; top: {y}px;");
    html! {
        <div class="selecao-barra" style={estilo} data-nav-group="selecao-barra"
            // `mousedown` no próprio botão desfaria a seleção antes do
            // clique chegar — por isso o default morre já aqui.
            onmousedown={Callback::from(|e: MouseEvent| e.prevent_default())}>
            { for [
                ("strong", "N", "Negrito", "selecao-barra__negrito"),
                ("em", "I", "Itálico", "selecao-barra__italico"),
                ("s", "S", "Tachado", "selecao-barra__tachado"),
                ("code", "‹›", "Código", "selecao-barra__codigo"),
            ].into_iter().map(|(tag, rotulo, titulo, classe)| html! {
                <button class={classes!("selecao-barra__botao", classe)}
                    title={titulo} data-nav-item="true" onclick={marcar(tag)}>
                    { rotulo }
                </button>
            }) }
            <span class="selecao-barra__separador"></span>
            <button class="selecao-barra__botao selecao-barra__link" title="Link"
                data-nav-item="true" onclick={link}>{ "🔗" }</button>
        </div>
    }
}

/// A seleção atual, se houver texto selecionado.
fn selecao_atual() -> Option<Range> {
    let sel = web_sys::window()?.get_selection().ok()??;
    if sel.is_collapsed() || sel.range_count() == 0 {
        return None;
    }
    sel.get_range_at(0).ok()
}

fn restaurar(r: &Range) {
    if let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) {
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(r);
    }
}

/// Onde desenhar a barra, ou `None` se não há seleção editável.
fn medir_selecao(editor_ref: &NodeRef) -> Option<(f64, f64)> {
    let range = selecao_atual()?;
    let editor = editor_ref.cast::<HtmlElement>()?;

    // Só dentro de um bloco editável do editor. Selecionar texto na
    // barra lateral, numa proposta ou numa conversa não pode abrir uma
    // barra que não vai conseguir escrever em lugar nenhum.
    let no = range.common_ancestor_container().ok()?;
    let alvo: Element = match no.dyn_ref::<Element>() {
        Some(e) => e.clone(),
        None => no.parent_element()?,
    };
    let editavel = alvo.closest("[contenteditable=\"true\"]").ok()??;
    if !editor.contains(Some(&editavel)) {
        return None;
    }

    let r = range.get_bounding_client_rect();
    if r.width() == 0.0 && r.height() == 0.0 {
        return None;
    }
    Some((r.x() + r.width() / 2.0, r.y() - FOLGA))
}

/// Envolve a seleção na marca, ou tira a marca se já estiver dentro dela.
fn aplicar_marca(tag: &str) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;

    if let Some(existente) = ancestral_da_marca(&range, tag) {
        desembrulhar(&existente)?;
    } else {
        let el = doc.create_element(tag).ok()?;
        // `extract_contents` + `insert_node` em vez de
        // `surround_contents`: este último falha quando a seleção
        // atravessa a borda de um elemento, que é o caso comum de
        // selecionar arrastando.
        let conteudo = range.extract_contents().ok()?;
        el.append_child(&conteudo).ok()?;
        range.insert_node(&el).ok()?;
        selecionar_conteudo(&doc, &el);
    }
    avisar_edicao(&range)
}

fn aplicar_link(url: &str) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;
    let el = doc.create_element("a").ok()?;
    el.set_attribute("href", url).ok()?;
    let conteudo = range.extract_contents().ok()?;
    el.append_child(&conteudo).ok()?;
    range.insert_node(&el).ok()?;
    selecionar_conteudo(&doc, &el);
    avisar_edicao(&range)
}

/// O elemento da marca que já envolve a seleção inteira, se houver.
fn ancestral_da_marca(range: &Range, tag: &str) -> Option<Element> {
    let no = range.common_ancestor_container().ok()?;
    let de = match no.dyn_ref::<Element>() {
        Some(e) => e.clone(),
        None => no.parent_element()?,
    };
    let achado = de.closest(tag).ok()??;
    // Não sai do bloco editável procurando marca.
    achado.closest("[contenteditable=\"true\"]").ok()??;
    Some(achado)
}

/// Troca o elemento pelos filhos dele, preservando o texto.
fn desembrulhar(el: &Element) -> Option<()> {
    let pai = el.parent_node()?;
    while let Some(filho) = el.first_child() {
        pai.insert_before(&filho, Some(el)).ok()?;
    }
    pai.remove_child(el).ok()?;
    Some(())
}

fn selecionar_conteudo(doc: &web_sys::Document, el: &Element) {
    let (Ok(novo), Some(Ok(Some(sel)))) = (
        doc.create_range(),
        web_sys::window().map(|w| w.get_selection()),
    ) else {
        return;
    };
    if novo.select_node_contents(el).is_ok() {
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(&novo);
    }
}

/// Dispara um `input` no bloco, pra a edição seguir o MESMO caminho de
/// quando a pessoa digita — autosave, undo e recomposição do markdown já
/// estão pendurados ali, e duplicá-los seria pedir pra divergirem.
fn avisar_edicao(range: &Range) -> Option<()> {
    let no = range.common_ancestor_container().ok()?;
    let de = match no.dyn_ref::<Element>() {
        Some(e) => e.clone(),
        None => no.parent_element()?,
    };
    let bloco = de.closest("[contenteditable=\"true\"]").ok()??;
    let ev = web_sys::Event::new_with_event_init_dict(
        "input",
        web_sys::EventInit::new().bubbles(true),
    )
    .ok()?;
    bloco.dispatch_event(&ev).ok()?;
    Some(())
}
