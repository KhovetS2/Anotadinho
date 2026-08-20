//! Caixa de destaque inline (`{{ type: "callout" }}`): cor e ícone por
//! variante, título editável, corpo markdown e recolher/expandir.
//!
//! É o primeiro embed de COMPOSIÇÃO (não de banco de dados): não guarda
//! registros, guarda prosa formatada. O corpo usa o
//! `EmbedMarkdownField`, que resolve o problema de editar markdown
//! dentro de um componente que o Yew re-renderiza (ver o módulo dele).

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, KeyboardEvent};
use yew::prelude::*;

use crate::components::embeds::EmbedMarkdownField;
use crate::components::icon::Icon;
use crate::embed::{CalloutEmbedData, CalloutVariant};

/// Props do `InlineCallout`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineCalloutProps {
    /// Dados do callout.
    pub data: CalloutEmbedData,
    /// Disparado quando variante, título, corpo ou estado de recolhido
    /// mudam.
    pub on_change: Callback<CalloutEmbedData>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Caixa de destaque inline.
#[function_component(InlineCallout)]
pub fn inline_callout(props: &InlineCalloutProps) -> Html {
    let variant_menu = use_state(|| false);

    let on_toggle_collapsed = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_| {
            let mut new_data = data.clone();
            new_data.toggle_collapsed();
            on_change.emit(new_data);
        })
    };

    let on_title_commit = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |value: String| {
            if value == data.title {
                return;
            }
            let mut new_data = data.clone();
            new_data.set_title(value);
            on_change.emit(new_data);
        })
    };

    let on_body_change = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |body: String| {
            let mut new_data = data.clone();
            new_data.set_body(body);
            on_change.emit(new_data);
        })
    };

    // Título usa `<input>` (não `contenteditable`) pelo mesmo motivo da
    // célula Text da tabela, ciclo 076: valor é propriedade do elemento.
    let title_onblur = {
        let commit = on_title_commit.clone();
        Callback::from(move |e: FocusEvent| {
            if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                commit.emit(el.value());
            }
        })
    };
    let title_onkeydown = {
        let commit = on_title_commit.clone();
        Callback::from(move |e: KeyboardEvent| {
            e.stop_propagation();
            if e.key() == "Enter" {
                e.prevent_default();
                if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                    let _ = el.blur();
                    commit.emit(el.value());
                }
            }
        })
    };

    let variant = props.data.variant;
    let nav_group = props.nav_group.clone();

    html! {
        <div class={classes!("callout", format!("callout--{}", variant.slug()))}
            data-nav-group={nav_group.clone()} data-nav-item={nav_group.clone()} data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
            <div class="callout__header">
                <button class="callout__variant" type="button"
                    title={format!("Variante: {}", variant.label())}
                    data-nav-item="callout-variant" data-nav-parent={nav_group.clone()}
                    onclick={{
                        let variant_menu = variant_menu.clone();
                        Callback::from(move |_| variant_menu.set(!*variant_menu))
                    }}>
                    <Icon name={variant.icon()} />
                </button>
                <input
                    class="callout__title"
                    type="text"
                    value={props.data.title.clone()}
                    placeholder="Título do destaque"
                    data-nav-item="callout-title" data-nav-parent={nav_group.clone()}
                    onblur={title_onblur}
                    onkeydown={title_onkeydown}
                />
                <button class="callout__collapse" type="button"
                    title={if props.data.collapsed { "Expandir" } else { "Recolher" }}
                    data-nav-item="callout-collapse" data-nav-parent={nav_group.clone()}
                    onclick={on_toggle_collapsed}>
                    <Icon name={if props.data.collapsed { "chevron-right" } else { "chevron-down" }} />
                </button>
            </div>
            if *variant_menu {
                <div class="callout__variants">
                    { for CalloutVariant::all().iter().map(|v| {
                        let v = *v;
                        let is_active = v == variant;
                        let onclick = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            let variant_menu = variant_menu.clone();
                            Callback::from(move |_| {
                                variant_menu.set(false);
                                if data.variant == v {
                                    return;
                                }
                                let mut new_data = data.clone();
                                new_data.set_variant(v);
                                on_change.emit(new_data);
                            })
                        };
                        html! {
                            <button
                                class={classes!("callout__variant-option",
                                    format!("callout__variant-option--{}", v.slug()),
                                    is_active.then_some("callout__variant-option--active"))}
                                type="button"
                                data-nav-item="callout-variant-option" data-nav-parent={nav_group.clone()}
                                {onclick}>
                                <Icon name={v.icon()} />{ v.label() }
                            </button>
                        }
                    }) }
                </div>
            }
            if !props.data.collapsed {
                <EmbedMarkdownField
                    markdown={props.data.body.clone()}
                    on_change={on_body_change}
                    class={classes!("callout__body")}
                    nav_group={nav_group.clone()}
                    placeholder="Clique pra escrever o destaque (Markdown)"
                />
            }
        </div>
    }
}
