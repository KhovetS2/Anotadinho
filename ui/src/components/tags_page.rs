//! Página `type: tags` — lista todas as tags usadas no vault (cards de
//! kanban + eventos de calendário) e as páginas onde cada uma aparece.
//! Somente leitura/navegação, mesmo espírito da página `type: calendar`
//! (ciclo 085): componente Rust dedicado, dispatchado por
//! `page_view.rs`, sem edição — editar tags continua sendo feito no
//! embed de origem.

use std::collections::BTreeMap;

use yew::prelude::*;

use crate::api::PageMeta;
use crate::embed::badge_class;

/// Props da `TagsPage`.
#[derive(Properties, PartialEq, Clone)]
pub struct TagsPageProps {
    /// Path do vault.
    pub vault_path: String,
    /// Navega pra uma página ao clicar.
    pub on_page_selected: Callback<PageMeta>,
}

#[function_component(TagsPage)]
pub fn tags_page(props: &TagsPageProps) -> Html {
    let tags = use_state(BTreeMap::<String, Vec<(String, String)>>::new);
    let loading = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let tags = tags.clone();
        let loading = loading.clone();
        use_effect_with(vault_path.clone(), move |vault_path| {
            let vault_path = vault_path.clone();
            let tags = tags.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let scanned = crate::embed::scan_vault_tags(&vault_path).await;
                tags.set(scanned);
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="tags-page"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let all_tag_names: Vec<String> = tags.keys().cloned().collect();

    html! {
        <div class="tags-page">
            <h2 class="tags-page__title">{ "Tags" }</h2>
            if tags.is_empty() {
                <p class="tags-page__empty">
                    { "Nenhuma tag encontrada. Adicione tags em cards de kanban ou eventos de calendário." }
                </p>
            } else {
                <div class="tags-page__list">
                    { for tags.iter().map(|(tag, pages)| {
                        let class = format!("tags-page__badge badge {}", badge_class(&all_tag_names, tag));
                        html! {
                            <div class="tags-page__group">
                                <div class="tags-page__group-header">
                                    <span {class}>{ tag }</span>
                                    <span class="tags-page__count">{ pages.len() }</span>
                                </div>
                                <div class="tags-page__pages">
                                    { for pages.iter().map(|(path, title)| {
                                        let meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                                        let on_page_selected = props.on_page_selected.clone();
                                        let onclick = Callback::from(move |_| on_page_selected.emit(meta.clone()));
                                        html! { <span class="tags-page__page-chip" {onclick}>{ title }</span> }
                                    }) }
                                </div>
                            </div>
                        }
                    }) }
                </div>
            }
        </div>
    }
}
