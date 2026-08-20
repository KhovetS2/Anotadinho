//! Tabela inline (dentro do wrapper `{{ type: "table" }}`), dinâmica:
//! criar/excluir linha e coluna, configurar cada coluna com 1 de 8 tipos
//! (texto, número, data, checkbox, url, página, seleção, tags) via
//! `ColumnSettingsModal`. Reaproveita `.task-table__*` (já estilizadas
//! globalmente) e `.badge--*` (já existentes) pras células de
//! seleção/tags, em vez de criar CSS de badge novo.

use gloo_events::EventListener;
use wasm_bindgen::JsCast;
use web_sys::{FocusEvent, HtmlInputElement, HtmlTextAreaElement, InputEvent};
use yew::prelude::*;

use crate::api::PageMeta;
use crate::components::date_picker::DatePicker;
use crate::components::embeds::ColumnSettingsModal;
use crate::components::icon::Icon;
use crate::dialog::PendingDialog;
use crate::embed::{badge_class, ColumnKind, TableEmbedData};

/// Props do `InlineTable`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineTableProps {
    /// Colunas (com tipo) + linhas.
    pub data: TableEmbedData,
    /// Path do vault (pra listar páginas na célula de tipo Página).
    pub vault_path: String,
    /// Disparado quando a tabela muda.
    pub on_change: Callback<TableEmbedData>,
    /// Abre o modal de diálogo do app.
    pub open_dialog: Callback<PendingDialog>,
    /// Navega pra outra página do vault (célula de tipo Página).
    pub on_page_selected: Callback<PageMeta>,
}

fn split_tags(cell: &str) -> Vec<String> {
    cell.split(", ").map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

fn join_tags(tags: &[String]) -> String {
    tags.join(", ")
}

fn input_value(e: &Event) -> Option<String> {
    e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()).map(|el| el.value())
}

fn textarea_value(e: &Event) -> Option<String> {
    e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()).map(|el| el.value())
}

/// Redimensiona a altura do `<textarea>` pra caber o conteúdo — coluna
/// Text precisa "crescer em altura" em vez de deixar a coluna esticar em
/// largura (era o comportamento do `contenteditable` antigo). Zera a
/// altura antes de medir `scroll_height` pra funcionar tanto ao crescer
/// quanto ao encolher (apagar texto).
fn autogrow_textarea(el: &HtmlTextAreaElement) {
    let style = el.style();
    let _ = style.set_property("height", "auto");
    let _ = style.set_property("height", &format!("{}px", el.scroll_height()));
}

/// Tabela inline com colunas tipadas e células editáveis.
#[function_component(InlineTable)]
pub fn inline_table(props: &InlineTableProps) -> Html {
    let open_cell_menu = use_state(|| None::<(usize, usize)>);
    let menu_filter = use_state(String::new);
    let editing_column = use_state(|| None::<usize>);
    let pages = use_state(Vec::<PageMeta>::new);

    // Carrega as páginas do vault uma vez (pra célula de tipo Página) —
    // barato o bastante pra não precisar condicionar a colunas do tipo
    // certo existirem.
    {
        let pages = pages.clone();
        let vault_path = props.vault_path.clone();
        use_effect_with(vault_path.clone(), move |vault_path| {
            let pages = pages.clone();
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(list) = crate::api::list_pages(&vault_path).await {
                    pages.set(list);
                }
            });
            || {}
        });
    }

    // Fecha o dropdown de célula (seleção/tags/página) ao clicar fora ou
    // apertar Escape.
    {
        let open_cell_menu = open_cell_menu.clone();
        use_effect_with(*open_cell_menu, move |open| {
            let mut listeners = Vec::new();
            if open.is_some() {
                let window = web_sys::window().expect("no global window");

                let close_cell = open_cell_menu.clone();
                listeners.push(EventListener::new(&window, "mousedown", move |e| {
                    let Some(node) = e.target().and_then(|t| t.dyn_into::<web_sys::Node>().ok()) else { return };
                    let target = node.dyn_ref::<web_sys::Element>().cloned().or_else(|| node.parent_element());
                    let Some(target) = target else { return };
                    if target.closest(".task-table__td--menu").ok().flatten().is_none() {
                        close_cell.set(None);
                    }
                }));

                let close_cell = open_cell_menu.clone();
                listeners.push(crate::menu_keyboard::escape_consumer(move || close_cell.set(None)));
            }
            move || drop(listeners)
        });
    }

    let add_row = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_data = data.clone();
            new_data.add_row();
            on_change.emit(new_data);
        })
    };

    let add_column = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let data = data.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nome da nova coluna".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |name: String| {
                    let mut new_data = data.clone();
                    new_data.add_column(name);
                    on_change.emit(new_data);
                }),
            });
        })
    };

    let n_cols = props.data.columns.len();

    // Modal de configuração de coluna — um único componente pra tabela
    // inteira, gated no índice em `editing_column` (mesmo padrão do
    // `editing_card` do InlineKanban).
    let column_modal = (*editing_column).and_then(|idx| {
        props.data.columns.get(idx).cloned().map(|column| {
            let close = { let editing_column = editing_column.clone(); Callback::from(move |_: ()| editing_column.set(None)) };

            let on_rename = {
                let data = props.data.clone();
                let on_change = props.on_change.clone();
                Callback::from(move |name: String| {
                    let mut new_data = data.clone();
                    new_data.set_column_name(idx, name);
                    on_change.emit(new_data);
                })
            };
            let on_retype = {
                let data = props.data.clone();
                let on_change = props.on_change.clone();
                Callback::from(move |kind: ColumnKind| {
                    let mut new_data = data.clone();
                    new_data.set_column_kind(idx, kind);
                    on_change.emit(new_data);
                })
            };
            let on_add_option = {
                let data = props.data.clone();
                let on_change = props.on_change.clone();
                Callback::from(move |opt: String| {
                    let mut new_data = data.clone();
                    new_data.add_column_option(idx, opt);
                    on_change.emit(new_data);
                })
            };
            let on_remove_option = {
                let data = props.data.clone();
                let on_change = props.on_change.clone();
                Callback::from(move |opt: String| {
                    let mut new_data = data.clone();
                    new_data.remove_column_option(idx, &opt);
                    on_change.emit(new_data);
                })
            };

            html! {
                <ColumnSettingsModal
                    {column}
                    {on_rename}
                    {on_retype}
                    {on_add_option}
                    {on_remove_option}
                    on_close={close}
                />
            }
        })
    });

    html! {
        <div class="embed-table">
            <table class="task-table__table">
                <thead>
                    <tr>
                        { for props.data.columns.iter().enumerate().map(|(ci, col)| {
                            let open_settings = {
                                let editing_column = editing_column.clone();
                                Callback::from(move |_: MouseEvent| editing_column.set(Some(ci)))
                            };
                            let delete_col = {
                                let data = props.data.clone();
                                let on_change = props.on_change.clone();
                                let open_dialog = props.open_dialog.clone();
                                let name = col.name.clone();
                                Callback::from(move |_: MouseEvent| {
                                    let data = data.clone();
                                    let on_change = on_change.clone();
                                    open_dialog.emit(PendingDialog::Confirm {
                                        message: format!("Excluir coluna \"{}\"?", name),
                                        confirm_label: "Excluir".to_string(),
                                        on_confirm: Callback::from(move |_| {
                                            let mut new_data = data.clone();
                                            new_data.remove_column(ci);
                                            on_change.emit(new_data);
                                        }),
                                    });
                                })
                            };
                            html! {
                                <th class="task-table__th">
                                    <span class="task-table__th-name">{ &col.name }</span>
                                    <button class="task-table__th-action" onclick={open_settings} title="Configurar coluna"><Icon name="settings" /></button>
                                    <button class="task-table__th-action" onclick={delete_col} title="Excluir coluna"><Icon name="x" /></button>
                                </th>
                            }
                        }) }
                        <th class="task-table__th task-table__th--add">
                            <button class="task-table__add" onclick={add_column} title="Nova coluna">{ "+" }</button>
                        </th>
                    </tr>
                </thead>
                <tbody>
                    { for props.data.rows.iter().enumerate().map(|(ri, row)| {
                        let delete_row = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            let open_dialog = props.open_dialog.clone();
                            Callback::from(move |_: MouseEvent| {
                                let data = data.clone();
                                let on_change = on_change.clone();
                                open_dialog.emit(PendingDialog::Confirm {
                                    message: "Excluir esta linha?".to_string(),
                                    confirm_label: "Excluir".to_string(),
                                    on_confirm: Callback::from(move |_| {
                                        let mut new_data = data.clone();
                                        new_data.remove_row(ri);
                                        on_change.emit(new_data);
                                    }),
                                });
                            })
                        };
                        html! {
                            <tr class="task-table__row">
                                { for row.iter().enumerate().map(|(ci, cell)| {
                                    let kind = props.data.columns.get(ci).map(|c| c.kind.clone()).unwrap_or(ColumnKind::Text);
                                    match kind {
                                        ColumnKind::Checkbox => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            let checked = cell == "true";
                                            let toggle = Callback::from(move |_: MouseEvent| {
                                                let mut new_data = data.clone();
                                                new_data.set_cell(ri, ci, if checked { "false".to_string() } else { "true".to_string() });
                                                on_change.emit(new_data);
                                            });
                                            html! {
                                                <td class="task-table__td">
                                                    <input class="checkbox" type="checkbox" {checked} onclick={toggle} />
                                                </td>
                                            }
                                        }
                                        ColumnKind::Number => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            let input_ref = NodeRef::default();
                                            let onblur = {
                                                let data = data.clone();
                                                let on_change = on_change.clone();
                                                Callback::from(move |e: FocusEvent| {
                                                    let Some(value) = input_value(&e) else { return };
                                                    let mut new_data = data.clone();
                                                    new_data.set_cell(ri, ci, value);
                                                    on_change.emit(new_data);
                                                })
                                            };
                                            // Spinner nativo escondido via CSS — estas ▲▼ próprias
                                            // usam HTMLInputElement::step_up/step_down (mesmo
                                            // comportamento nativo) e commitam na hora, sem
                                            // depender de blur.
                                            let step = |delta: f64| {
                                                let input_ref = input_ref.clone();
                                                let data = data.clone();
                                                let on_change = on_change.clone();
                                                Callback::from(move |_: MouseEvent| {
                                                    let Some(input) = input_ref.cast::<web_sys::HtmlInputElement>() else { return };
                                                    let current: f64 = input.value().parse().unwrap_or(0.0);
                                                    let new_value = (current + delta).to_string();
                                                    input.set_value(&new_value);
                                                    let mut new_data = data.clone();
                                                    new_data.set_cell(ri, ci, new_value);
                                                    on_change.emit(new_data);
                                                })
                                            };
                                            html! {
                                                <td class="task-table__td task-table__td--number">
                                                    <div class="task-table__number-cell">
                                                        <input ref={input_ref.clone()} class="task-table__number-input" type="number" value={cell.clone()} {onblur} />
                                                        <div class="task-table__number-spin">
                                                            <button class="task-table__number-spin-btn" type="button" tabindex="-1" onclick={step(1.0)}>{ "▲" }</button>
                                                            <button class="task-table__number-spin-btn" type="button" tabindex="-1" onclick={step(-1.0)}>{ "▼" }</button>
                                                        </div>
                                                    </div>
                                                </td>
                                            }
                                        }
                                        ColumnKind::Date => {
                                            let is_open = *open_cell_menu == Some((ri, ci));
                                            let cell_value = cell.clone();
                                            // Ciclo 135: célula "chip" nunca teve
                                            // suporte de teclado (só `onclick`
                                            // num `<span>` não focável) —
                                            // ação de ativar vira `Callback<()>`
                                            // pra reaproveitar tanto no clique
                                            // quanto no Enter/Espaço.
                                            let toggle_activate = {
                                                let open_cell_menu = open_cell_menu.clone();
                                                Callback::from(move |_: ()| {
                                                    open_cell_menu.set(if is_open { None } else { Some((ri, ci)) });
                                                })
                                            };
                                            let toggle_open = {
                                                let toggle_activate = toggle_activate.clone();
                                                Callback::from(move |_: MouseEvent| toggle_activate.emit(()))
                                            };
                                            let toggle_onkeydown = crate::keyboard_activate::activate_on_enter_or_space(toggle_activate);
                                            let on_pick = {
                                                let data = props.data.clone();
                                                let on_change = props.on_change.clone();
                                                let open_cell_menu = open_cell_menu.clone();
                                                Callback::from(move |date: String| {
                                                    let mut new_data = data.clone();
                                                    new_data.set_cell(ri, ci, date);
                                                    on_change.emit(new_data);
                                                    open_cell_menu.set(None);
                                                })
                                            };
                                            let on_close = {
                                                let open_cell_menu = open_cell_menu.clone();
                                                Callback::from(move |_: ()| open_cell_menu.set(None))
                                            };
                                            html! {
                                                <td class="task-table__td task-table__td--menu">
                                                    <span class="task-table__date-chip" tabindex="0" onclick={toggle_open} onkeydown={toggle_onkeydown}>
                                                        if cell_value.is_empty() { { "+ data" } } else { { cell_value.clone() } }
                                                    </span>
                                                    if is_open {
                                                        <DatePicker value={if cell_value.is_empty() { None } else { Some(cell_value.clone()) }} {on_pick} {on_close} />
                                                    }
                                                </td>
                                            }
                                        }
                                        ColumnKind::Url => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            let open_dialog = props.open_dialog.clone();
                                            let current = cell.clone();
                                            let edit = Callback::from(move |_: MouseEvent| {
                                                let data = data.clone();
                                                let on_change = on_change.clone();
                                                open_dialog.emit(PendingDialog::Prompt {
                                                    title: "URL".to_string(),
                                                    default: current.clone(),
                                                    on_submit: Callback::from(move |value: String| {
                                                        let mut new_data = data.clone();
                                                        new_data.set_cell(ri, ci, value);
                                                        on_change.emit(new_data);
                                                    }),
                                                });
                                            });
                                            html! {
                                                <td class="task-table__td task-table__td--url">
                                                    <div class="task-table__url-cell">
                                                        if cell.is_empty() {
                                                            <button class="task-table__link-add" onclick={edit}>{ "+ url" }</button>
                                                        } else {
                                                            <a class="task-table__link" href={cell.clone()} target="_blank" rel="noopener noreferrer">{ cell.clone() }</a>
                                                            <button class="task-table__link-edit" onclick={edit} title="Editar URL"><Icon name="edit" /></button>
                                                        }
                                                    </div>
                                                </td>
                                            }
                                        }
                                        ColumnKind::PageLink => {
                                            let is_open = *open_cell_menu == Some((ri, ci));
                                            let cell_value = cell.clone();
                                            let linked = pages.iter().find(|p| p.path == cell_value).cloned();
                                            let toggle_activate = {
                                                let open_cell_menu = open_cell_menu.clone();
                                                let menu_filter = menu_filter.clone();
                                                Callback::from(move |_: ()| {
                                                    menu_filter.set(String::new());
                                                    open_cell_menu.set(if is_open { None } else { Some((ri, ci)) });
                                                })
                                            };
                                            let toggle_open = {
                                                let toggle_activate = toggle_activate.clone();
                                                Callback::from(move |_: MouseEvent| toggle_activate.emit(()))
                                            };
                                            let toggle_onkeydown = crate::keyboard_activate::activate_on_enter_or_space(toggle_activate);
                                            let open_page = {
                                                let on_page_selected = props.on_page_selected.clone();
                                                let cell_value = cell_value.clone();
                                                let linked = linked.clone();
                                                Callback::from(move |e: MouseEvent| {
                                                    e.stop_propagation();
                                                    let meta = linked.clone().unwrap_or_else(|| PageMeta {
                                                        path: cell_value.clone(),
                                                        title: cell_value.clone(),
                                                        section: "pages".to_string(),
                                                    });
                                                    on_page_selected.emit(meta);
                                                })
                                            };
                                            html! {
                                                <td class="task-table__td task-table__td--menu">
                                                    if cell_value.is_empty() {
                                                        <button class="task-table__page-link-add" onclick={toggle_open} onkeydown={toggle_onkeydown}>{ "+ página" }</button>
                                                    } else {
                                                        <span class="task-table__page-link" tabindex="0" onclick={toggle_open} onkeydown={toggle_onkeydown}>
                                                            <Icon name="file-text" />{ format!(" {}", linked.as_ref().map(|p| p.title.as_str()).unwrap_or(cell_value.as_str())) }
                                                        </span>
                                                        <button class="task-table__page-link-open" onclick={open_page} title="Abrir página"><Icon name="external-link" /></button>
                                                    }
                                                    if is_open {
                                                        <div class="table-select-menu">
                                                            <div class="table-select-menu__input-row">
                                                                <input
                                                                    class="table-select-menu__input"
                                                                    type="text"
                                                                    placeholder="Filtrar páginas..."
                                                                    value={(*menu_filter).clone()}
                                                                    oninput={{
                                                                        let menu_filter = menu_filter.clone();
                                                                        Callback::from(move |e: InputEvent| {
                                                                            if let Some(v) = input_value(&e) { menu_filter.set(v); }
                                                                        })
                                                                    }}
                                                                />
                                                            </div>
                                                            { for pages.iter()
                                                                .filter(|p| menu_filter.is_empty() || p.title.to_lowercase().contains(&menu_filter.to_lowercase()))
                                                                .map(|p| {
                                                                    let data = props.data.clone();
                                                                    let on_change = props.on_change.clone();
                                                                    let open_cell_menu = open_cell_menu.clone();
                                                                    let path = p.path.clone();
                                                                    let pick = Callback::from(move |_: MouseEvent| {
                                                                        let mut new_data = data.clone();
                                                                        new_data.set_cell(ri, ci, path.clone());
                                                                        on_change.emit(new_data);
                                                                        open_cell_menu.set(None);
                                                                    });
                                                                    html! {
                                                                        <div class="table-select-menu__item" onclick={pick}>{ &p.title }</div>
                                                                    }
                                                                }) }
                                                        </div>
                                                    }
                                                </td>
                                            }
                                        }
                                        ColumnKind::MultiSelect { options } => {
                                            let is_open = *open_cell_menu == Some((ri, ci));
                                            let selected = split_tags(cell);
                                            let toggle_activate = {
                                                let open_cell_menu = open_cell_menu.clone();
                                                let menu_filter = menu_filter.clone();
                                                Callback::from(move |_: ()| {
                                                    menu_filter.set(String::new());
                                                    open_cell_menu.set(if is_open { None } else { Some((ri, ci)) });
                                                })
                                            };
                                            let toggle_open = {
                                                let toggle_activate = toggle_activate.clone();
                                                Callback::from(move |_: MouseEvent| toggle_activate.emit(()))
                                            };
                                            let toggle_onkeydown = crate::keyboard_activate::activate_on_enter_or_space(toggle_activate);
                                            let submit_new_tag = {
                                                let data = props.data.clone();
                                                let on_change = props.on_change.clone();
                                                let menu_filter = menu_filter.clone();
                                                let selected = selected.clone();
                                                Callback::from(move |_: ()| {
                                                    let new_tag = menu_filter.trim().to_string();
                                                    if new_tag.is_empty() { return; }
                                                    let mut new_data = data.clone();
                                                    new_data.add_column_option(ci, new_tag.clone());
                                                    if !selected.iter().any(|t| t == &new_tag) {
                                                        let mut tags = selected.clone();
                                                        tags.push(new_tag);
                                                        new_data.set_cell(ri, ci, join_tags(&tags));
                                                    }
                                                    on_change.emit(new_data);
                                                    menu_filter.set(String::new());
                                                })
                                            };
                                            html! {
                                                <td class="task-table__td task-table__td--menu task-table__td--tags">
                                                    <span class="task-table__tags" tabindex="0" onclick={toggle_open} onkeydown={toggle_onkeydown}>
                                                        { for selected.iter().map(|t| html! {
                                                            <span class={classes!("badge", badge_class(&options, t))}>{ t }</span>
                                                        }) }
                                                        if selected.is_empty() {
                                                            <span class="task-table__tags-empty">{ "+ tags" }</span>
                                                        }
                                                    </span>
                                                    if is_open {
                                                        <div class="table-select-menu">
                                                            { for options.iter().map(|opt| {
                                                                let data = props.data.clone();
                                                                let on_change = props.on_change.clone();
                                                                let opt_value = opt.clone();
                                                                let is_checked = selected.iter().any(|t| t == opt);
                                                                let selected = selected.clone();
                                                                let toggle_tag = Callback::from(move |_: MouseEvent| {
                                                                    let mut tags = selected.clone();
                                                                    if is_checked {
                                                                        tags.retain(|t| t != &opt_value);
                                                                    } else {
                                                                        tags.push(opt_value.clone());
                                                                    }
                                                                    let mut new_data = data.clone();
                                                                    new_data.set_cell(ri, ci, join_tags(&tags));
                                                                    on_change.emit(new_data);
                                                                });
                                                                let class = if is_checked {
                                                                    "table-select-menu__item table-select-menu__item--checked"
                                                                } else {
                                                                    "table-select-menu__item"
                                                                };
                                                                html! {
                                                                    <div {class} onclick={toggle_tag}>
                                                                        <span class="table-select-menu__check"><Icon name={if is_checked { "check-square" } else { "square" }} /></span>
                                                                        { opt }
                                                                    </div>
                                                                }
                                                            }) }
                                                            <div class="table-select-menu__input-row">
                                                                <input
                                                                    class="table-select-menu__input"
                                                                    type="text"
                                                                    placeholder="Nova tag..."
                                                                    value={(*menu_filter).clone()}
                                                                    oninput={{
                                                                        let menu_filter = menu_filter.clone();
                                                                        Callback::from(move |e: InputEvent| {
                                                                            if let Some(v) = input_value(&e) { menu_filter.set(v); }
                                                                        })
                                                                    }}
                                                                    onkeydown={{
                                                                        let submit_new_tag = submit_new_tag.clone();
                                                                        Callback::from(move |e: KeyboardEvent| {
                                                                            if e.key() == "Enter" {
                                                                                e.prevent_default();
                                                                                submit_new_tag.emit(());
                                                                            }
                                                                        })
                                                                    }}
                                                                />
                                                                <button class="card-modal__add-chip" onclick={Callback::from(move |_: MouseEvent| submit_new_tag.emit(()))}>{ "+ tag" }</button>
                                                            </div>
                                                        </div>
                                                    }
                                                </td>
                                            }
                                        }
                                        ColumnKind::Select { options } => {
                                            let is_open = *open_cell_menu == Some((ri, ci));
                                            let cell_value = cell.clone();
                                            let toggle_activate = {
                                                let open_cell_menu = open_cell_menu.clone();
                                                Callback::from(move |_: ()| {
                                                    open_cell_menu.set(if is_open { None } else { Some((ri, ci)) });
                                                })
                                            };
                                            let toggle_open = {
                                                let toggle_activate = toggle_activate.clone();
                                                Callback::from(move |_: MouseEvent| toggle_activate.emit(()))
                                            };
                                            let toggle_onkeydown = crate::keyboard_activate::activate_on_enter_or_space(toggle_activate);
                                            html! {
                                                <td class="task-table__td task-table__td--menu">
                                                    <span class={classes!("badge", badge_class(&options, &cell_value))} tabindex="0" onclick={toggle_open} onkeydown={toggle_onkeydown}>
                                                        { if cell_value.is_empty() { "—".to_string() } else { cell_value.clone() } }
                                                    </span>
                                                    if is_open {
                                                        <div class="table-select-menu">
                                                            { for options.iter().map(|opt| {
                                                                let data = props.data.clone();
                                                                let on_change = props.on_change.clone();
                                                                let open_cell_menu = open_cell_menu.clone();
                                                                let opt_value = opt.clone();
                                                                let pick = Callback::from(move |_: MouseEvent| {
                                                                    let mut new_data = data.clone();
                                                                    new_data.set_cell(ri, ci, opt_value.clone());
                                                                    on_change.emit(new_data);
                                                                    open_cell_menu.set(None);
                                                                });
                                                                html! {
                                                                    <div class="table-select-menu__item" onclick={pick}>{ opt }</div>
                                                                }
                                                            }) }
                                                        </div>
                                                    }
                                                </td>
                                            }
                                        }
                                        ColumnKind::Text => {
                                            let data = props.data.clone();
                                            let on_change = props.on_change.clone();
                                            // `<textarea>` em vez de `contenteditable` (mesmo
                                            // princípio da coluna Number usando `<input>`) — um `<td
                                            // contenteditable>` cujo filho de texto é re-renderizado
                                            // pelo Yew a cada mudança em QUALQUER célula da tabela
                                            // duplicava o texto: digitar sem `oninput` deixa o VDOM
                                            // do Yew com uma referência desatualizada do nó de texto;
                                            // se o Enter criava um `<div>`/quebra de linha novo
                                            // (comportamento padrão de contenteditable no WebKit),
                                            // esse nó extra nunca era rastreado pelo Yew, então nunca
                                            // era removido ao reconciliar — sobrava como um
                                            // "duplicado" na célula. `<textarea>` não tem esse
                                            // problema (valor é propriedade do elemento, não filhos
                                            // de DOM) e, diferente de `<input>`, permite crescer em
                                            // altura pra caber texto longo em vez de esticar a
                                            // coluna em largura.
                                            let onblur = Callback::from(move |e: FocusEvent| {
                                                let Some(value) = textarea_value(&e) else { return };
                                                let mut new_data = data.clone();
                                                new_data.set_cell(ri, ci, value);
                                                on_change.emit(new_data);
                                            });
                                            let oninput = Callback::from(|e: InputEvent| {
                                                if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()) {
                                                    autogrow_textarea(&el);
                                                }
                                            });
                                            let onfocus = Callback::from(|e: FocusEvent| {
                                                if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlTextAreaElement>().ok()) {
                                                    autogrow_textarea(&el);
                                                }
                                            });
                                            html! {
                                                <td class="task-table__td">
                                                    <textarea class="task-table__text-input" rows="1" value={cell.clone()} {onblur} {oninput} {onfocus} />
                                                </td>
                                            }
                                        }
                                    }
                                }) }
                                <td class="task-table__td task-table__td--actions">
                                    <button class="task-table__row-delete" onclick={delete_row} title="Excluir linha"><Icon name="x" /></button>
                                </td>
                            </tr>
                        }
                    }) }
                    <tr class="task-table__row task-table__row--add">
                        <td class="task-table__td" colspan={(n_cols + 1).to_string()}>
                            <button class="task-table__add" onclick={add_row}>{ "+ linha" }</button>
                        </td>
                    </tr>
                </tbody>
            </table>
            { for column_modal }
        </div>
    }
}
