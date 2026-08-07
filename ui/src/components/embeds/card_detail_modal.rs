//! Modal de detalhes de um card do kanban — abas Detalhes / Comentários /
//! Anexos. Diferente do `PendingDialog` genérico (pensado pra fluxos
//! simples de 1 campo): aqui é um formulário real com várias seções, por
//! isso é um componente próprio reaproveitando só a casca do `Modal`.

use wasm_bindgen::JsCast;
use web_sys::FocusEvent;
use yew::prelude::*;

use crate::components::modal::Modal;
use crate::dialog::PendingDialog;
use crate::embed::{Attachment, ChecklistItem, KanbanCard};

#[derive(Clone, Copy, PartialEq)]
enum Tab {
    Details,
    Comments,
    Attachments,
}

/// Props do `CardDetailModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct CardDetailModalProps {
    /// Card sendo editado (snapshot atual).
    pub card: KanbanCard,
    /// Path do vault (pra copiar anexos pra `assets/`).
    pub vault_path: String,
    /// Disparado a cada mudança no card (título, descrição, tags, data,
    /// checklist, comentário, anexo — cada ação salva na hora).
    pub on_change: Callback<KanbanCard>,
    /// Disparado ao fechar o modal.
    pub on_close: Callback<()>,
    /// Abre o modal de diálogo simples do app (pra pedir texto/confirmar).
    pub open_dialog: Callback<PendingDialog>,
}

fn today_string() -> String {
    let d = js_sys::Date::new_0();
    format!("{:04}-{:02}-{:02}", d.get_full_year(), d.get_month() + 1, d.get_date())
}

fn text_of(e: FocusEvent) -> Option<String> {
    let target = e.target()?;
    let el = target.dyn_into::<web_sys::Element>().ok()?;
    el.text_content()
}

/// Modal de detalhes do card, com abas.
#[function_component(CardDetailModal)]
pub fn card_detail_modal(props: &CardDetailModalProps) -> Html {
    let tab = use_state(|| Tab::Details);
    let card = &props.card;

    let tab_button = |label: &'static str, value: Tab| {
        let tab = tab.clone();
        let active = *tab == value;
        let class = if active { "card-modal__tab card-modal__tab--active" } else { "card-modal__tab" };
        html! {
            <button {class} onclick={Callback::from(move |_| tab.set(value))}>{ label }</button>
        }
    };

    let on_title_blur = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: FocusEvent| {
            if let Some(text) = text_of(e) {
                if text != card.title {
                    let mut new_card = card.clone();
                    new_card.title = text;
                    on_change.emit(new_card);
                }
            }
        })
    };

    let on_description_blur = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |e: FocusEvent| {
            if let Some(text) = text_of(e) {
                let text = text.trim().to_string();
                let mut new_card = card.clone();
                new_card.description = if text.is_empty() { None } else { Some(text) };
                on_change.emit(new_card);
            }
        })
    };

    let add_tag = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let card = card.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Nova tag".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |tag: String| {
                    let mut new_card = card.clone();
                    if !new_card.tags.iter().any(|t| t == &tag) {
                        new_card.tags.push(tag);
                        on_change.emit(new_card);
                    }
                }),
            });
        })
    };

    let set_due = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let card = card.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Data de vencimento (AAAA-MM-DD, vazio pra remover)".to_string(),
                default: card.due.clone().unwrap_or_default(),
                on_submit: Callback::from(move |date: String| {
                    let mut new_card = card.clone();
                    new_card.due = if date.trim().is_empty() { None } else { Some(date.trim().to_string()) };
                    on_change.emit(new_card);
                }),
            });
        })
    };

    let add_checklist_item = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let card = card.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Novo item da checklist".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |text: String| {
                    let mut new_card = card.clone();
                    new_card.checklist.push(ChecklistItem { text, done: false });
                    on_change.emit(new_card);
                }),
            });
        })
    };

    let add_comment = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        Callback::from(move |_: MouseEvent| {
            let card = card.clone();
            let on_change = on_change.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Novo comentário".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |text: String| {
                    let mut new_card = card.clone();
                    new_card.comments.push(crate::embed::Comment { text, created: today_string() });
                    on_change.emit(new_card);
                }),
            });
        })
    };

    let add_attachment = {
        let card = card.clone();
        let on_change = props.on_change.clone();
        let open_dialog = props.open_dialog.clone();
        let vault_path = props.vault_path.clone();
        Callback::from(move |_: MouseEvent| {
            let card = card.clone();
            let on_change = on_change.clone();
            let vault_path = vault_path.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Caminho do arquivo pra anexar".to_string(),
                default: String::new(),
                on_submit: Callback::from(move |path: String| {
                    let card = card.clone();
                    let on_change = on_change.clone();
                    let vault_path = vault_path.clone();
                    wasm_bindgen_futures::spawn_local(async move {
                        if let Ok(relative) = crate::api::copy_to_assets(&vault_path, &path).await {
                            let name = std::path::Path::new(&relative)
                                .file_name()
                                .map(|n| n.to_string_lossy().to_string())
                                .unwrap_or_else(|| relative.clone());
                            let mut new_card = card.clone();
                            new_card.attachments.push(Attachment { name, path: relative });
                            on_change.emit(new_card);
                        }
                    });
                }),
            });
        })
    };

    let is_overdue = card.due.as_deref().map(|d| d < today_string().as_str()).unwrap_or(false);

    let body = match *tab {
        Tab::Details => html! {
            <div class="card-modal__section">
                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Descrição" }</label>
                    <div class="card-modal__textarea" contenteditable="true" onblur={on_description_blur}>
                        { card.description.clone().unwrap_or_default() }
                    </div>
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Tags" }</label>
                    <div class="card-modal__tags">
                        { for card.tags.iter().enumerate().map(|(i, tag)| {
                            let card = card.clone();
                            let on_change = props.on_change.clone();
                            let remove = Callback::from(move |_: MouseEvent| {
                                let mut new_card = card.clone();
                                new_card.tags.remove(i);
                                on_change.emit(new_card);
                            });
                            html! {
                                <span class="badge badge--info card-modal__tag">
                                    { tag }
                                    <button class="card-modal__tag-remove" onclick={remove}>{ "✕" }</button>
                                </span>
                            }
                        }) }
                        <button class="card-modal__add-chip" onclick={add_tag}>{ "+ tag" }</button>
                    </div>
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Vencimento" }</label>
                    <div class="card-modal__due">
                        <span class={if is_overdue { "card-modal__due-value card-modal__due-value--overdue" } else { "card-modal__due-value" }}>
                            { card.due.clone().unwrap_or_else(|| "Sem data".to_string()) }
                        </span>
                        <button class="card-modal__add-chip" onclick={set_due}>{ "Definir data" }</button>
                    </div>
                </div>

                <div class="card-modal__field">
                    <label class="card-modal__label">{ "Checklist" }</label>
                    <div class="card-modal__checklist">
                        { for card.checklist.iter().enumerate().map(|(i, item)| {
                            let card_toggle = card.clone();
                            let on_change_toggle = props.on_change.clone();
                            let toggle = Callback::from(move |_: MouseEvent| {
                                let mut new_card = card_toggle.clone();
                                if let Some(it) = new_card.checklist.get_mut(i) { it.done = !it.done; }
                                on_change_toggle.emit(new_card);
                            });
                            let card_remove = card.clone();
                            let on_change_remove = props.on_change.clone();
                            let remove = Callback::from(move |_: MouseEvent| {
                                let mut new_card = card_remove.clone();
                                new_card.checklist.remove(i);
                                on_change_remove.emit(new_card);
                            });
                            let item_class = if item.done { "card-modal__checklist-text card-modal__checklist-text--done" } else { "card-modal__checklist-text" };
                            html! {
                                <div class="card-modal__checklist-item">
                                    <input class="checkbox" type="checkbox" checked={item.done} onclick={toggle} />
                                    <span class={item_class}>{ &item.text }</span>
                                    <button class="card-modal__tag-remove" onclick={remove}>{ "✕" }</button>
                                </div>
                            }
                        }) }
                        <button class="card-modal__add-chip" onclick={add_checklist_item}>{ "+ item" }</button>
                    </div>
                </div>
            </div>
        },
        Tab::Comments => html! {
            <div class="card-modal__section">
                <div class="card-modal__comments">
                    { for card.comments.iter().map(|c| html! {
                        <div class="card-modal__comment">
                            <span class="card-modal__comment-date">{ &c.created }</span>
                            <p class="card-modal__comment-text">{ &c.text }</p>
                        </div>
                    }) }
                    if card.comments.is_empty() {
                        <p class="card-modal__empty">{ "Nenhum comentário ainda." }</p>
                    }
                </div>
                <button class="card-modal__add-chip" onclick={add_comment}>{ "+ comentário" }</button>
            </div>
        },
        Tab::Attachments => html! {
            <div class="card-modal__section">
                <div class="card-modal__attachments">
                    { for card.attachments.iter().enumerate().map(|(i, a)| {
                        let card = card.clone();
                        let on_change = props.on_change.clone();
                        let remove = Callback::from(move |_: MouseEvent| {
                            let mut new_card = card.clone();
                            new_card.attachments.remove(i);
                            on_change.emit(new_card);
                        });
                        html! {
                            <div class="card-modal__attachment">
                                <span class="card-modal__attachment-name">{ &a.name }</span>
                                <button class="card-modal__tag-remove" onclick={remove}>{ "✕" }</button>
                            </div>
                        }
                    }) }
                    if card.attachments.is_empty() {
                        <p class="card-modal__empty">{ "Nenhum anexo ainda." }</p>
                    }
                </div>
                <button class="card-modal__add-chip" onclick={add_attachment}>{ "+ anexo" }</button>
            </div>
        },
    };

    html! {
        <Modal title={card.title.clone()} open={true} wide={true} on_close={props.on_close.clone()}>
            <div
                class="card-modal__title"
                contenteditable="true"
                onblur={on_title_blur}
            >
                { &card.title }
            </div>
            <div class="card-modal__tabs">
                { tab_button("Detalhes", Tab::Details) }
                { tab_button("Comentários", Tab::Comments) }
                { tab_button("Anexos", Tab::Attachments) }
            </div>
            { body }
        </Modal>
    }
}
