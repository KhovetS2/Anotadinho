//! Conteúdo do painel de propriedades de frontmatter — renderizado
//! dentro de um `Modal` (ciclo 109, aberto pelo menu "⋯" do header do
//! editor). Mostra os campos fixos (título/tags/tipo) E qualquer
//! propriedade customizada (`Frontmatter.extra`, ciclo 098), com
//! ver/adicionar/editar/remover. Único lugar do app que edita
//! frontmatter de verdade (o resto do editor sempre preservou o
//! frontmatter cru, intocado).

use anotadinho_core::Frontmatter;
use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icon::Icon;
use crate::dialog::PendingDialog;

/// Tipos de página reconhecidos por `page_view.rs` — o campo `type`
/// só pode ser um destes (ou vazio = "md"), pra não quebrar o
/// roteamento ao digitar um valor livre por engano.
const KNOWN_TYPES: &[(&str, &str)] = &[
    ("", "Página normal"),
    ("landing", "Landing"),
    ("kanban", "Kanban"),
    ("calendar", "Calendário"),
    ("table", "Tabela de tarefas"),
    ("tags", "Tags"),
    ("assets", "Assets"),
    ("graph", "Grafo de conexões"),
];

/// Props do `PropertiesPanel`.
#[derive(Properties, PartialEq, Clone)]
pub struct PropertiesPanelProps {
    /// Frontmatter atual da página.
    pub frontmatter: Frontmatter,
    /// Disparado com o frontmatter atualizado a cada edição.
    pub on_change: Callback<Frontmatter>,
    /// Abre o modal de diálogo do app (usado pra pedir o nome da nova
    /// propriedade).
    pub open_dialog: Callback<PendingDialog>,
}

/// Converte um `serde_yaml::Value` em texto pra exibir/editar num
/// campo simples — v1 não tem editor rico por tipo (número/bool/data),
/// só texto livre; ver Não-objetivos do ciclo 099.
fn value_to_display(value: &serde_yaml::Value) -> String {
    match value {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Null => String::new(),
        other => serde_yaml::to_string(other).unwrap_or_default().trim().to_string(),
    }
}

#[function_component(PropertiesPanel)]
pub fn properties_panel(props: &PropertiesPanelProps) -> Html {
    let fm = &props.frontmatter;

    let emit = {
        let on_change = props.on_change.clone();
        move |new_fm: Frontmatter| on_change.emit(new_fm)
    };

    let on_title_input = {
        let fm = fm.clone();
        let emit = emit.clone();
        Callback::from(move |e: InputEvent| {
            let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) else { return };
            let mut new_fm = fm.clone();
            let v = input.value();
            new_fm.title = if v.is_empty() { None } else { Some(v) };
            emit(new_fm);
        })
    };

    let on_type_change = {
        let fm = fm.clone();
        let emit = emit.clone();
        Callback::from(move |e: Event| {
            let Some(target) = e.target() else { return };
            let Ok(select) = target.dyn_into::<HtmlSelectElement>() else { return };
            let mut new_fm = fm.clone();
            let v = select.value();
            new_fm.page_type = if v.is_empty() { None } else { Some(v) };
            emit(new_fm);
        })
    };

    let remove_tag = {
        let fm = fm.clone();
        let emit = emit.clone();
        move |tag: String| {
            let fm = fm.clone();
            let emit = emit.clone();
            Callback::from(move |_: MouseEvent| {
                let mut new_fm = fm.clone();
                new_fm.tags.retain(|t| t != &tag);
                emit(new_fm);
            })
        }
    };

    let add_tag = {
        let fm = fm.clone();
        let emit = emit.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let fm = fm.clone();
            let emit = emit.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nova tag".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |tag: String| {
                    let mut new_fm = fm.clone();
                    if !new_fm.tags.contains(&tag) {
                        new_fm.tags.push(tag);
                    }
                    emit(new_fm.clone());
                }),
            });
        })
    };

    let update_extra_value = {
        let fm = fm.clone();
        let emit = emit.clone();
        move |key: String| {
            let fm = fm.clone();
            let emit = emit.clone();
            Callback::from(move |e: InputEvent| {
                let Some(input) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) else { return };
                let mut new_fm = fm.clone();
                new_fm.extra.insert(key.clone(), serde_yaml::Value::String(input.value()));
                emit(new_fm);
            })
        }
    };

    let remove_extra = {
        let fm = fm.clone();
        let emit = emit.clone();
        move |key: String| {
            let fm = fm.clone();
            let emit = emit.clone();
            Callback::from(move |_: MouseEvent| {
                let mut new_fm = fm.clone();
                new_fm.extra.remove(&key);
                emit(new_fm);
            })
        }
    };

    let add_property = {
        let fm = fm.clone();
        let emit = emit.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let fm = fm.clone();
            let emit = emit.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nome da nova propriedade".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |key: String| {
                    let mut new_fm = fm.clone();
                    new_fm.extra.entry(key).or_insert_with(|| serde_yaml::Value::String(String::new()));
                    emit(new_fm.clone());
                }),
            });
        })
    };

    let extra_entries: Vec<(String, String)> = fm.extra.iter()
        .map(|(k, v)| (k.clone(), value_to_display(v)))
        .collect();

    html! {
        <div class="properties-panel__body">
            <div class="properties-panel__row">
                <span class="properties-panel__key">{ "título" }</span>
                <input class="properties-panel__input" type="text"
                    value={fm.title.clone().unwrap_or_default()} oninput={on_title_input} />
            </div>
            <div class="properties-panel__row">
                <span class="properties-panel__key">{ "tipo" }</span>
                <select class="properties-panel__input" onchange={on_type_change}>
                    { for KNOWN_TYPES.iter().map(|(value, label)| {
                        let selected = fm.page_type.as_deref().unwrap_or("") == *value;
                        html! { <option value={*value} {selected}>{ *label }</option> }
                    }) }
                </select>
            </div>
            <div class="properties-panel__row">
                <span class="properties-panel__key">{ "tags" }</span>
                <div class="properties-panel__tags">
                    { for fm.tags.iter().map(|tag| {
                        let onclick = remove_tag(tag.clone());
                        html! {
                            <span class="properties-panel__tag-chip">
                                { tag }
                                <button class="properties-panel__tag-remove" {onclick}><Icon name="x" /></button>
                            </span>
                        }
                    }) }
                    <button class="btn btn--ghost btn--xs" onclick={add_tag}>{ "+ tag" }</button>
                </div>
            </div>
            { for extra_entries.iter().map(|(key, value)| {
                let oninput = update_extra_value(key.clone());
                let onclick = remove_extra(key.clone());
                html! {
                    <div class="properties-panel__row">
                        <span class="properties-panel__key">{ key }</span>
                        <input class="properties-panel__input" type="text" value={value.clone()} {oninput} />
                        <button class="properties-panel__remove" title="Remover propriedade" {onclick}><Icon name="x" /></button>
                    </div>
                }
            }) }
            <button class="btn btn--ghost btn--sm properties-panel__add" onclick={add_property}>
                { "+ propriedade" }
            </button>
        </div>
    }
}
