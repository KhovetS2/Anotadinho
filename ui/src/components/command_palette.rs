//! Paleta de comandos (Ctrl+K) — navegar pra qualquer página do vault ou
//! disparar um comando nomeado, tudo pelo teclado, sem precisar do
//! mouse. Mesmo padrão visual/mecânico do menu `/` do editor (lista
//! filtrada, ArrowUp/Down/Enter, fechar ao Escape/clicar fora).

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Comando nomeado disparado pela paleta — tratado centralmente em
/// `App` (é quem tem os callbacks de tema/sidebar/etc já em mãos).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PaletteAction {
    NewPage,
    NewFolder,
    ToggleTheme,
    ToggleSidebar,
    Today,
    ViewTags,
}

const COMMANDS: &[(&str, PaletteAction)] = &[
    ("Nova página", PaletteAction::NewPage),
    ("Nova pasta", PaletteAction::NewFolder),
    ("Alternar tema", PaletteAction::ToggleTheme),
    ("Alternar sidebar", PaletteAction::ToggleSidebar),
    ("Ir pra Hoje (journal)", PaletteAction::Today),
    ("Ver Tags", PaletteAction::ViewTags),
];

#[derive(Debug, Clone, PartialEq)]
enum Item {
    Command(&'static str, PaletteAction),
    Page(PageMeta),
}

/// Props da `CommandPalette`.
#[derive(Properties, PartialEq, Clone)]
pub struct CommandPaletteProps {
    /// Path do vault — usado só pra buscar a lista de páginas.
    pub vault_path: String,
    /// Disparado quando a paleta deve fechar (Escape, clicar fora, item
    /// selecionado).
    pub on_close: Callback<()>,
    /// Disparado ao selecionar uma página.
    pub on_page_selected: Callback<PageMeta>,
    /// Disparado ao selecionar um comando nomeado.
    pub on_action: Callback<PaletteAction>,
}

/// Paleta de comandos — montada/desmontada pelo pai a cada abrir/fechar
/// (não fica sempre viva escondida), então o estado interno já nasce
/// limpo toda vez.
#[function_component(CommandPalette)]
pub fn command_palette(props: &CommandPaletteProps) -> Html {
    let query = use_state(String::new);
    let idx = use_state(|| 0usize);
    let pages = use_state(Vec::<PageMeta>::new);
    let input_ref = use_node_ref();

    {
        let vault_path = props.vault_path.clone();
        let pages = pages.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(list) = api::list_pages(&vault_path).await {
                    pages.set(list);
                }
            });
            || {}
        });
    }

    {
        let input_ref = input_ref.clone();
        use_effect_with((), move |_| {
            if let Some(el) = input_ref.cast::<HtmlInputElement>() {
                let _ = el.focus();
            }
            || {}
        });
    }

    let q = query.to_lowercase();
    let items: Vec<Item> = {
        let mut v: Vec<Item> = COMMANDS.iter()
            .filter(|(label, _)| q.is_empty() || label.to_lowercase().contains(&q))
            .map(|(label, action)| Item::Command(label, *action))
            .collect();
        v.extend(
            pages.iter()
                .filter(|p| q.is_empty() || p.title.to_lowercase().contains(&q))
                .cloned()
                .map(Item::Page),
        );
        v
    };
    let items_len = items.len();

    let select_idx = {
        let on_close = props.on_close.clone();
        let on_page_selected = props.on_page_selected.clone();
        let on_action = props.on_action.clone();
        let items = items.clone();
        Callback::from(move |i: usize| {
            if let Some(item) = items.get(i) {
                match item.clone() {
                    Item::Command(_, action) => on_action.emit(action),
                    Item::Page(meta) => on_page_selected.emit(meta),
                }
            }
            on_close.emit(());
        })
    };

    let oninput = {
        let query = query.clone();
        let idx = idx.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                query.set(input.value());
                idx.set(0);
            }
        })
    };

    let onkeydown = {
        let idx = idx.clone();
        let select_idx = select_idx.clone();
        let on_close = props.on_close.clone();
        Callback::from(move |e: KeyboardEvent| {
            match e.key().as_str() {
                "ArrowDown" => { e.prevent_default(); if items_len > 0 { idx.set((*idx + 1) % items_len); } }
                "ArrowUp" => { e.prevent_default(); if items_len > 0 { idx.set((*idx + items_len - 1) % items_len); } }
                "Enter" => { e.prevent_default(); select_idx.emit(*idx); }
                "Escape" => { e.prevent_default(); on_close.emit(()); }
                _ => {}
            }
        })
    };

    let onclick_overlay = {
        let on_close = props.on_close.clone();
        Callback::from(move |_: MouseEvent| on_close.emit(()))
    };
    let stop_propagation = Callback::from(|e: MouseEvent| e.stop_propagation());

    html! {
        <div class="command-palette-overlay" onclick={onclick_overlay}>
            <div class="command-palette" onclick={stop_propagation}>
                <input ref={input_ref} class="command-palette__input" type="text"
                    placeholder="Buscar página ou comando..." value={(*query).clone()}
                    {oninput} {onkeydown} />
                <div class="command-palette__list">
                    if items.is_empty() {
                        <p class="command-palette__empty">{ "Nada encontrado" }</p>
                    } else {
                        { for items.iter().enumerate().map(|(i, item)| {
                            let is_active = i == *idx;
                            let class = if is_active { "command-palette__item command-palette__item--active" } else { "command-palette__item" };
                            let sel = select_idx.clone();
                            let onmousedown = Callback::from(|e: MouseEvent| e.prevent_default());
                            let onclick = Callback::from(move |_| sel.emit(i));
                            match item {
                                Item::Command(label, _) => html! {
                                    <div {class} {onmousedown} {onclick}>
                                        <span class="command-palette__item-icon">{ "⚡" }</span>
                                        <span class="command-palette__item-title">{ *label }</span>
                                    </div>
                                },
                                Item::Page(meta) => html! {
                                    <div {class} {onmousedown} {onclick}>
                                        <span class="command-palette__item-icon">{ "📄" }</span>
                                        <span class="command-palette__item-title">{ &meta.title }</span>
                                    </div>
                                },
                            }
                        }) }
                    }
                </div>
            </div>
        </div>
    }
}
