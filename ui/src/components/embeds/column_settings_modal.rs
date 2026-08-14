//! Modal de configuração de coluna da tabela — nome, tipo, e (se Seleção
//! ou Tags) a lista de opções. Substitui o fluxo antigo de 3
//! `PendingDialog::Prompt` encadeados digitando o tipo como texto livre —
//! com 8 tipos de coluna isso não escalava mais.

use wasm_bindgen::JsCast;
use web_sys::{FocusEvent, HtmlSelectElement};
use yew::prelude::*;

use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::embed::{ColumnKind, TableColumn};

/// Props do `ColumnSettingsModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct ColumnSettingsModalProps {
    /// Coluna sendo configurada (snapshot atual).
    pub column: TableColumn,
    /// Disparado quando o nome muda (commit no blur).
    pub on_rename: Callback<String>,
    /// Disparado quando o tipo muda (troca já commita na hora).
    pub on_retype: Callback<ColumnKind>,
    /// Disparado ao adicionar uma opção nova (Select/MultiSelect).
    pub on_add_option: Callback<String>,
    /// Disparado ao remover uma opção existente.
    pub on_remove_option: Callback<String>,
    /// Disparado ao fechar o modal.
    pub on_close: Callback<()>,
}

fn kind_select_value(kind: &ColumnKind) -> &'static str {
    match kind {
        ColumnKind::Text => "texto",
        ColumnKind::Checkbox => "checkbox",
        ColumnKind::Select { .. } => "selecao",
        ColumnKind::MultiSelect { .. } => "tags",
        ColumnKind::Number => "numero",
        ColumnKind::Date => "data",
        ColumnKind::Url => "url",
        ColumnKind::PageLink => "pagina",
    }
}

fn current_options(kind: &ColumnKind) -> Vec<String> {
    match kind {
        ColumnKind::Select { options } | ColumnKind::MultiSelect { options } => options.clone(),
        _ => Vec::new(),
    }
}

/// Modal de configuração de uma coluna da tabela.
#[function_component(ColumnSettingsModal)]
pub fn column_settings_modal(props: &ColumnSettingsModalProps) -> Html {
    let new_option = use_state(String::new);
    let column = &props.column;
    let has_options = matches!(column.kind, ColumnKind::Select { .. } | ColumnKind::MultiSelect { .. });

    let on_name_blur = {
        let on_rename = props.on_rename.clone();
        let current = column.name.clone();
        Callback::from(move |e: FocusEvent| {
            let Some(target) = e.target() else { return };
            let Ok(el) = target.dyn_into::<web_sys::Element>() else { return };
            let text = el.text_content().unwrap_or_default();
            if text != current {
                on_rename.emit(text);
            }
        })
    };

    let on_type_change = {
        let on_retype = props.on_retype.clone();
        let existing_options = current_options(&column.kind);
        Callback::from(move |e: Event| {
            let Some(target) = e.target() else { return };
            let Ok(select) = target.dyn_into::<HtmlSelectElement>() else { return };
            let kind = match select.value().as_str() {
                "checkbox" => ColumnKind::Checkbox,
                "selecao" => ColumnKind::Select { options: existing_options.clone() },
                "tags" => ColumnKind::MultiSelect { options: existing_options.clone() },
                "numero" => ColumnKind::Number,
                "data" => ColumnKind::Date,
                "url" => ColumnKind::Url,
                "pagina" => ColumnKind::PageLink,
                _ => ColumnKind::Text,
            };
            on_retype.emit(kind);
        })
    };

    let on_new_option_input = {
        let new_option = new_option.clone();
        Callback::from(move |e: InputEvent| {
            let Some(target) = e.target() else { return };
            let Ok(el) = target.dyn_into::<web_sys::HtmlInputElement>() else { return };
            new_option.set(el.value());
        })
    };

    let submit_new_option = {
        let new_option = new_option.clone();
        let on_add_option = props.on_add_option.clone();
        Callback::from(move |_: MouseEvent| {
            let value = new_option.trim().to_string();
            if !value.is_empty() {
                on_add_option.emit(value);
                new_option.set(String::new());
            }
        })
    };

    let on_new_option_keydown = {
        let new_option = new_option.clone();
        let on_add_option = props.on_add_option.clone();
        Callback::from(move |e: KeyboardEvent| {
            if e.key() == "Enter" {
                e.prevent_default();
                let value = new_option.trim().to_string();
                if !value.is_empty() {
                    on_add_option.emit(value);
                    new_option.set(String::new());
                }
            }
        })
    };

    html! {
        <Modal title="Configurar coluna" open={true} on_close={props.on_close.clone()}>
            <div class="card-modal__section">
                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Nome" }</label>
                    <div class="card-modal__textarea" contenteditable="true" onblur={on_name_blur}>
                        { &column.name }
                    </div>
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Tipo" }</label>
                    <select class="column-settings__type" onchange={on_type_change}>
                        { for [
                            ("texto", "Texto"), ("numero", "Número"), ("data", "Data"),
                            ("checkbox", "Checkbox"), ("url", "URL"), ("pagina", "Página"),
                            ("selecao", "Seleção"), ("tags", "Tags"),
                        ].iter().map(|(value, label)| {
                            let selected = kind_select_value(&column.kind) == *value;
                            html! { <option value={*value} {selected}>{ *label }</option> }
                        }) }
                    </select>
                </div>

                if has_options {
                    <div class="card-modal__field">
                        <label class="card-modal__label">{ "Opções" }</label>
                        <div class="card-modal__tags">
                            { for current_options(&column.kind).iter().map(|opt| {
                                let on_remove_option = props.on_remove_option.clone();
                                let opt_value = opt.clone();
                                let remove = Callback::from(move |_: MouseEvent| on_remove_option.emit(opt_value.clone()));
                                html! {
                                    <span class="badge badge--info card-modal__tag">
                                        { opt }
                                        <button class="card-modal__tag-remove" onclick={remove}><Icon name="x" /></button>
                                    </span>
                                }
                            }) }
                        </div>
                        <div class="column-settings__add-option">
                            <input
                                class="column-settings__add-option-input"
                                type="text"
                                placeholder="Nova opção"
                                value={(*new_option).clone()}
                                oninput={on_new_option_input}
                                onkeydown={on_new_option_keydown}
                            />
                            <button class="card-modal__add-chip" onclick={submit_new_option}>{ "+ opção" }</button>
                        </div>
                    </div>
                }
            </div>
        </Modal>
    }
}
