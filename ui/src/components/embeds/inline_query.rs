//! Consulta viva do vault (`{{ type: "query" }}`).
//!
//! Uma lista que se mantém sozinha: "specs em backlog ordenadas por
//! prioridade" deixa de ser um kanban manual que alguém precisa lembrar
//! de mover e vira estado derivado, recalculado a cada abertura da
//! página. O YAML do embed É a consulta (`crate::query::Query`), a
//! mesma struct que o `anotadinho-cli` executa no terminal — o agente
//! headless lê exatamente o recorte que o humano vê.
//!
//! Somente leitura de propósito: editar a página continua sendo na
//! página. Escrita a partir de um painel é o embed `actions` (ciclo 156).

use yew::prelude::*;

use crate::api::{self, PageIndexEntry, PageMeta};
use crate::components::embeds::QuerySettingsModal;
use crate::components::icon::Icon;
use crate::query::{Query, QueryView};

/// Campos que sempre aparecem no seletor, mesmo que nenhuma página do
/// vault use ainda.
const BASE_FIELDS: [&str; 5] = ["title", "path", "type", "status", "priority"];

/// Props do `InlineQuery`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineQueryProps {
    /// A consulta declarada no wrapper.
    pub data: Query,
    /// Path do vault.
    pub vault_path: String,
    /// Disparado quando a consulta é reconfigurada.
    pub on_change: Callback<Query>,
    /// Abre a página clicada.
    pub on_page_selected: Callback<PageMeta>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Consulta viva.
#[function_component(InlineQuery)]
pub fn inline_query(props: &InlineQueryProps) -> Html {
    let entries = use_state(Vec::<PageIndexEntry>::new);
    let loading = use_state(|| true);
    let settings_open = use_state(|| false);

    // UMA varredura (ciclo 150) alimenta a consulta inteira — o filtro
    // roda em memória, então reconfigurar não custa I/O nenhum.
    {
        let entries = entries.clone();
        let loading = loading.clone();
        let vault_path = props.vault_path.clone();
        use_effect_with(vault_path, move |vault_path| {
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                entries.set(api::scan_vault(&vault_path).await.unwrap_or_default());
                loading.set(false);
            });
            || {}
        });
    }

    let results = props.data.run(&entries);

    // Campos oferecidos no modal: os fixos + toda chave de frontmatter/
    // property vista no vault, pra configurar sem precisar decorar nome.
    let known_fields = {
        let mut fields: Vec<String> = BASE_FIELDS.iter().map(|f| f.to_string()).collect();
        for entry in entries.iter() {
            for key in entry.properties.keys() {
                if !fields.contains(key) {
                    fields.push(key.clone());
                }
            }
        }
        fields
    };

    let open_page = {
        let on_page_selected = props.on_page_selected.clone();
        move |entry: &PageIndexEntry| {
            let meta = PageMeta {
                path: entry.path.clone(),
                title: entry.title.clone(),
                section: entry.section.clone(),
            };
            let on_page_selected = on_page_selected.clone();
            Callback::from(move |_| on_page_selected.emit(meta.clone()))
        }
    };

    let nav_group = props.nav_group.clone();
    let columns = props.data.columns.clone();

    let body = if *loading {
        html! { <p class="query-embed__empty">{ "Consultando o vault..." }</p> }
    } else if results.is_empty() {
        html! { <p class="query-embed__empty">{ "Nenhuma página bate com esta consulta." }</p> }
    } else {
        match props.data.view {
            QueryView::List => html! {
                <ul class="query-embed__list">
                    { for results.iter().map(|entry| {
                        let activate = open_page(entry);
                        html! {
                            <li class="query-embed__row" tabindex="0" role="button"
                                data-nav-item="query-row" data-nav-parent={nav_group.clone()}
                                onclick={activate.reform(|_: MouseEvent| ())}
                                onkeydown={crate::keyboard_activate::activate_on_enter_or_space(activate.clone())}>
                                <span class="query-embed__title">{ entry.title.clone() }</span>
                                <span class="query-embed__meta">
                                    { for columns.iter().filter_map(|c| {
                                        entry.field(c).filter(|v| !v.is_empty()).map(|v| html! {
                                            <span class="query-embed__chip">{ format!("{c}: {v}") }</span>
                                        })
                                    }) }
                                </span>
                            </li>
                        }
                    }) }
                </ul>
            },
            QueryView::Table => html! {
                <table class="query-embed__table">
                    <thead>
                        <tr>
                            <th>{ "Página" }</th>
                            { for columns.iter().map(|c| html! { <th>{ c.clone() }</th> }) }
                        </tr>
                    </thead>
                    <tbody>
                        { for results.iter().map(|entry| {
                            let activate = open_page(entry);
                            html! {
                                <tr class="query-embed__row" tabindex="0" role="button"
                                    data-nav-item="query-row" data-nav-parent={nav_group.clone()}
                                    onclick={activate.reform(|_: MouseEvent| ())}
                                    onkeydown={crate::keyboard_activate::activate_on_enter_or_space(activate.clone())}>
                                    <td class="query-embed__title">{ entry.title.clone() }</td>
                                    { for columns.iter().map(|c| html! {
                                        <td>{ entry.field(c).unwrap_or_default() }</td>
                                    }) }
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
            },
            QueryView::Cards => html! {
                <div class="query-embed__cards">
                    { for results.iter().map(|entry| {
                        let activate = open_page(entry);
                        html! {
                            <div class="query-embed__card" tabindex="0" role="button"
                                data-nav-item="query-row" data-nav-parent={nav_group.clone()}
                                onclick={activate.reform(|_: MouseEvent| ())}
                                onkeydown={crate::keyboard_activate::activate_on_enter_or_space(activate.clone())}>
                                <span class="query-embed__title">{ entry.title.clone() }</span>
                                <span class="query-embed__path">{ entry.path.clone() }</span>
                                <div class="query-embed__meta">
                                    { for columns.iter().filter_map(|c| {
                                        entry.field(c).filter(|v| !v.is_empty()).map(|v| html! {
                                            <span class="query-embed__chip">{ v }</span>
                                        })
                                    }) }
                                </div>
                            </div>
                        }
                    }) }
                </div>
            },
        }
    };

    let describe = describe_query(&props.data);

    html! {
        <div class="query-embed" data-nav-group={nav_group.clone()}>
            <div class="query-embed__bar">
                <Icon name="search" />
                <span class="query-embed__desc">{ describe }</span>
                <span class="query-embed__count">
                    { format!("{} {}", results.len(), if results.len() == 1 { "página" } else { "páginas" }) }
                </span>
                <button class="query-embed__btn" type="button" title="Configurar consulta"
                    data-nav-item="query-settings" data-nav-parent={nav_group.clone()}
                    onclick={{
                        let settings_open = settings_open.clone();
                        Callback::from(move |_| settings_open.set(true))
                    }}>
                    <Icon name="settings" />
                </button>
            </div>
            { body }
            if *settings_open {
                <QuerySettingsModal
                    query={props.data.clone()}
                    known_fields={known_fields}
                    on_change={props.on_change.clone()}
                    on_close={{
                        let settings_open = settings_open.clone();
                        Callback::from(move |_| settings_open.set(false))
                    }}
                />
            }
        </div>
    }
}

/// Resumo legível da consulta, pra dar pra ver o recorte sem abrir a
/// configuração (e pra o `.md` fazer sentido lido de fora).
fn describe_query(q: &Query) -> String {
    let mut parts = Vec::new();
    parts.push(match &q.from {
        Some(from) => format!("em {from}"),
        None => "no vault inteiro".to_string(),
    });
    if !q.tags.is_empty() {
        parts.push(format!("com tag {}", q.tags.join(" + ")));
    }
    for c in &q.conditions {
        if c.field.trim().is_empty() {
            continue;
        }
        parts.push(format!("{} {} {}", c.field, c.op.label(), c.value).trim().to_string());
    }
    if let Some(sort) = &q.sort {
        parts.push(format!(
            "ordenado por {}{}",
            sort.field,
            if sort.desc { " (desc)" } else { "" }
        ));
    }
    parts.join(" · ")
}
