//! Campo de markdown editável DENTRO de um embed (`EmbedMarkdownField`).
//!
//! Um embed que carrega prosa (callout, e as colunas do ciclo 152)
//! precisa mostrar markdown renderizado e deixar editar no lugar. A
//! tentação é usar um `contenteditable` com o HTML renderizado dentro —
//! e é exatamente o que já deu errado no ciclo 076: um nó de texto
//! dentro de um `contenteditable` que o Yew re-renderiza a cada mudança
//! do embed acaba com o VDOM apontando pra um nó desatualizado, e a
//! quebra de linha que o WebKit insere sozinho vira texto duplicado que
//! nunca é reconciliado.
//!
//! Aqui o campo tem dois estados explícitos:
//!
//! - **lendo**: `<div>` com o HTML renderizado, injetado por
//!   `set_inner_html` (o Yew não é dono desse conteúdo, mesma técnica do
//!   editor com os segmentos de markdown);
//! - **editando**: `<textarea>` com o markdown CRU, que cresce em altura
//!   e commita no blur — o valor é propriedade do elemento, não filhos
//!   de DOM, então o problema do ciclo 076 não existe.
//!
//! Clicar (ou Enter/Espaço com o campo focado) entra em edição; Escape
//! cancela sem gravar; Ctrl+Enter grava sem precisar tirar o foco.

use wasm_bindgen::JsCast;
use web_sys::{HtmlTextAreaElement, KeyboardEvent};
use yew::prelude::*;

/// Props do `EmbedMarkdownField`.
#[derive(Properties, PartialEq, Clone)]
pub struct EmbedMarkdownFieldProps {
    /// Markdown atual.
    pub markdown: String,
    /// Disparado ao gravar (blur ou Ctrl+Enter), só quando o texto
    /// realmente mudou.
    pub on_change: Callback<String>,
    /// Texto mostrado quando o markdown está vazio.
    #[prop_or_else(|| "Clique pra escrever...".to_string())]
    pub placeholder: String,
    /// Classe extra no wrapper (o embed dono decide o espaçamento).
    #[prop_or_default]
    pub class: Classes,
    /// Grupo de navegação por teclado (ciclo 135) do embed dono.
    #[prop_or_default]
    pub nav_group: Option<String>,
}

/// Campo de markdown de um embed: lê renderizado, edita cru.
#[function_component(EmbedMarkdownField)]
pub fn embed_markdown_field(props: &EmbedMarkdownFieldProps) -> Html {
    let editing = use_state(|| false);
    let view_ref = use_node_ref();
    let textarea_ref = use_node_ref();

    // Injeta o HTML renderizado fora do VDOM do Yew. Roda de novo quando
    // o markdown muda por fora (undo, edição pelo CLI + watcher) ou ao
    // voltar do modo de edição.
    {
        let view_ref = view_ref.clone();
        let markdown = props.markdown.clone();
        let placeholder = props.placeholder.clone();
        let is_editing = *editing;
        use_effect_with((markdown, is_editing), move |(markdown, is_editing)| {
            if !*is_editing {
                if let Some(el) = view_ref.cast::<web_sys::Element>() {
                    if markdown.trim().is_empty() {
                        el.set_text_content(Some(&placeholder));
                    } else {
                        el.set_inner_html(&crate::markdown_render::render(markdown));
                    }
                }
            }
            || {}
        });
    }

    // Foco + cursor no fim assim que o textarea aparece: entrar em edição
    // com o cursor em lugar nenhum obrigaria um segundo clique.
    {
        let textarea_ref = textarea_ref.clone();
        use_effect_with(*editing, move |is_editing| {
            if *is_editing {
                if let Some(el) = textarea_ref.cast::<HtmlTextAreaElement>() {
                    let _ = el.focus();
                    let end = el.value().chars().count() as u32;
                    let _ = el.set_selection_range(end, end);
                    autogrow(&el);
                }
            }
            || {}
        });
    }

    let start_edit: Callback<()> = {
        let editing = editing.clone();
        Callback::from(move |_| editing.set(true))
    };

    let commit = {
        let editing = editing.clone();
        let on_change = props.on_change.clone();
        let current = props.markdown.clone();
        Callback::from(move |value: String| {
            editing.set(false);
            if value != current {
                on_change.emit(value);
            }
        })
    };

    let onblur = {
        let commit = commit.clone();
        Callback::from(move |e: FocusEvent| {
            let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()) else {
                return;
            };
            commit.emit(el.value());
        })
    };

    let onkeydown = {
        let commit = commit.clone();
        let editing = editing.clone();
        Callback::from(move |e: KeyboardEvent| {
            // Não deixa a tecla subir pro editor da página: Escape lá
            // fecha menu, e Ctrl+Enter/atalhos de bloco não deveriam
            // disparar enquanto se escreve dentro do embed.
            e.stop_propagation();
            if e.key() == "Escape" {
                e.prevent_default();
                editing.set(false);
            } else if e.key() == "Enter" && (e.ctrl_key() || e.meta_key()) {
                e.prevent_default();
                if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()) {
                    commit.emit(el.value());
                }
            }
        })
    };

    let oninput = Callback::from(|e: InputEvent| {
        if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()) {
            autogrow(&el);
        }
    });

    let class = classes!("embed-md", props.class.clone());
    let is_empty = props.markdown.trim().is_empty();

    if *editing {
        html! {
            <textarea
                ref={textarea_ref}
                class={classes!("embed-md__input", props.class.clone())}
                value={props.markdown.clone()}
                spellcheck="false"
                {onblur}
                {onkeydown}
                {oninput}
            />
        }
    } else {
        html! {
            <div
                ref={view_ref}
                class={classes!(class, is_empty.then_some("embed-md--empty"))}
                tabindex="0"
                role="button"
                title="Clique pra editar (Markdown)"
                data-nav-item="embed-md"
                data-nav-parent={props.nav_group.clone()}
                onclick={start_edit.reform(|_: MouseEvent| ())}
                onkeydown={crate::keyboard_activate::activate_on_enter_or_space(start_edit.clone())}
            />
        }
    }
}

/// Cresce em altura pra caber o conteúdo (mesma função da célula Text da
/// tabela, ciclo 077 — zera antes de medir pra também encolher).
fn autogrow(el: &HtmlTextAreaElement) {
    let style = el.style();
    let _ = style.set_property("height", "auto");
    let _ = style.set_property("height", &format!("{}px", el.scroll_height()));
}
