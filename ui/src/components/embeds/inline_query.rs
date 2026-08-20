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

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
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
    let erro = use_state(|| None::<String>);

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

    // Edição de propriedade na própria linha (ciclo 168). Antes disso a
    // consulta era só leitura: ver a spec em backlog e mudar o status
    // exigia abrir a página, achar o painel de propriedades e voltar —
    // "ver" e "agir" em lugares diferentes.
    // `Some((path, campo))` = célula em edição.
    let editando = use_state(|| None::<(String, String)>);

    // Valores já usados em cada campo, no recorte atual — viram sugestão
    // (`<datalist>`) em vez de o usuário ter que decorar
    // `in-progress`/`in-review`.
    let sugestoes = {
        let mut mapa: std::collections::BTreeMap<String, Vec<String>> = Default::default();
        for campo in &props.data.columns {
            let mut vistos: Vec<String> = Vec::new();
            for e in entries.iter() {
                if let Some(v) = e.field(campo).filter(|v| !v.trim().is_empty()) {
                    if !vistos.contains(&v) {
                        vistos.push(v);
                    }
                }
            }
            vistos.sort();
            mapa.insert(campo.clone(), vistos);
        }
        mapa
    };

    // Grava o campo na página e recarrega a varredura: se a página
    // deixou de bater com o filtro, ela SOME da lista na hora — que é o
    // sinal de que a ação funcionou.
    let gravar = {
        let vault_path = props.vault_path.clone();
        let entries = entries.clone();
        let editando = editando.clone();
        let erro = erro.clone();
        Callback::from(move |(path, campo, valor): (String, String, String)| {
            let vault_path = vault_path.clone();
            let entries = entries.clone();
            let editando = editando.clone();
            let erro = erro.clone();
            wasm_bindgen_futures::spawn_local(async move {
                editando.set(None);
                let atual = match api::read_page_versioned(&vault_path, &path).await {
                    Ok(p) => p,
                    Err(e) => return erro.set(Some(format!("não consegui ler {path}: {e}"))),
                };
                let novo = match anotadinho_core::MarkdownCodec::set_frontmatter_field(
                    &atual.content,
                    &campo,
                    &valor,
                ) {
                    Ok(c) => c,
                    Err(e) => return erro.set(Some(format!("não consegui gravar {campo}: {e}"))),
                };
                match api::write_page_checked(&vault_path, &path, &novo, atual.version.as_deref()).await {
                    Ok(_) => {
                        erro.set(None);
                        // Reavalia o recorte com o vault atualizado.
                        entries.set(api::scan_vault(&vault_path).await.unwrap_or_default());
                    }
                    Err(e) if e.contains(api::CONFLICT_PREFIX) => erro.set(Some(format!(
                        "{path} mudou no disco enquanto você editava — abra a página pra resolver"
                    ))),
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

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

    // Célula de um campo de `columns`: mostra o valor e, no clique/Enter,
    // vira um `<input>` com sugestões dos valores já usados no recorte.
    let celula = {
        let editando = editando.clone();
        let gravar = gravar.clone();
        let sugestoes = sugestoes.clone();
        let nav_group = nav_group.clone();
        move |path: String, campo: String, valor: String, classe: &'static str| -> Html {
            let em_edicao = editando.as_ref().is_some_and(|(p, c)| *p == path && *c == campo);
            if em_edicao {
                let commit = {
                    let gravar = gravar.clone();
                    let path = path.clone();
                    let campo = campo.clone();
                    let anterior = valor.clone();
                    move |novo: String| {
                        if novo != anterior {
                            gravar.emit((path.clone(), campo.clone(), novo));
                        }
                    }
                };
                let onblur = {
                    let commit = commit.clone();
                    let editando = editando.clone();
                    Callback::from(move |e: FocusEvent| {
                        let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) else { return };
                        editando.set(None);
                        commit(el.value());
                    })
                };
                let onkeydown = {
                    let editando = editando.clone();
                    Callback::from(move |e: web_sys::KeyboardEvent| {
                        e.stop_propagation();
                        if e.key() == "Enter" {
                            if let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) {
                                let _ = el.blur();
                            }
                        } else if e.key() == "Escape" {
                            editando.set(None);
                        }
                    })
                };
                let lista_id = format!("sug-{}-{}", campo, path.replace(['/', '.'], "-"));
                let opcoes = sugestoes.get(&campo).cloned().unwrap_or_default();
                html! {
                    <span class={classe}>
                        <input class="query-embed__editar" type="text" value={valor}
                            list={lista_id.clone()} autofocus=true {onblur} {onkeydown} />
                        <datalist id={lista_id}>
                            { for opcoes.into_iter().map(|o| html! { <option value={o} /> }) }
                        </datalist>
                    </span>
                }
            } else {
                let abrir: Callback<()> = {
                    let editando = editando.clone();
                    let path = path.clone();
                    let campo = campo.clone();
                    Callback::from(move |_| editando.set(Some((path.clone(), campo.clone()))))
                };
                let rotulo = if valor.trim().is_empty() { "—".to_string() } else { valor };
                html! {
                    <span class={classes!(classe, "query-embed__editavel")} tabindex="0" role="button"
                        title={format!("Editar {campo}")}
                        data-nav-item="query-cell" data-nav-parent={nav_group.clone()}
                        onclick={abrir.reform(|e: MouseEvent| { e.stop_propagation(); })}
                        onkeydown={crate::keyboard_activate::activate_on_enter_or_space(abrir.clone())}>
                        { rotulo }
                    </span>
                }
            }
        }
    };

    // Cabeçalho de grupo (ciclo 169): rótulo, contagem, agregados e o
    // botão de recolher — o estado de recolhido mora no YAML, então o
    // painel reabre do jeito que ficou.
    let cabecalho_grupo = {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        let nav_group = nav_group.clone();
        move |grupo: &crate::query::Grupo<'_>| -> Html {
            if data.group_by.is_none() && data.aggregate.is_empty() {
                return html! {};
            }
            let valor = grupo.valor.clone();
            let recolhido = data.recolhido(&valor);
            let alternar: Callback<()> = {
                let data = data.clone();
                let on_change = on_change.clone();
                let valor = valor.clone();
                Callback::from(move |_| {
                    let mut novo = data.clone();
                    novo.alternar_recolhido(&valor);
                    on_change.emit(novo);
                })
            };
            html! {
                <div class="query-embed__grupo" tabindex="0" role="button"
                    data-nav-item="query-group" data-nav-parent={nav_group.clone()}
                    onclick={alternar.reform(|_: MouseEvent| ())}
                    onkeydown={crate::keyboard_activate::activate_on_enter_or_space(alternar.clone())}>
                    if data.group_by.is_some() {
                        <Icon name={if recolhido { "chevron-right" } else { "chevron-down" }} />
                        <span class="query-embed__grupo-nome">{ grupo.rotulo.clone() }</span>
                    }
                    <span class="query-embed__grupo-total">{ format!("{}", grupo.itens.len()) }</span>
                    { for grupo.agregados.iter().map(|(rotulo, valor)| html! {
                        <span class="query-embed__chip">{ format!("{rotulo}: {valor}") }</span>
                    }) }
                </div>
            }
        }
    };

    let grupos = props.data.run_grouped(&entries);

    let body = if *loading {
        html! { <p class="query-embed__empty">{ "Consultando o vault..." }</p> }
    } else if results.is_empty() {
        html! { <p class="query-embed__empty">{ "Nenhuma página bate com esta consulta." }</p> }
    } else if props.data.group_by.is_some() || !props.data.aggregate.is_empty() {
        html! {
            <div class="query-embed__grupos">
                { for grupos.iter().map(|grupo| {
                    let recolhido = props.data.recolhido(&grupo.valor);
                    html! {
                        <div class="query-embed__grupo-bloco">
                            { cabecalho_grupo(grupo) }
                            if !recolhido {
                                <ul class="query-embed__list">
                                    { for grupo.itens.iter().map(|entry| {
                                        let activate = open_page(entry);
                                        html! {
                                            <li class="query-embed__row" tabindex="0" role="button"
                                                data-nav-item="query-row" data-nav-parent={nav_group.clone()}
                                                onclick={activate.reform(|_: MouseEvent| ())}
                                                onkeydown={crate::keyboard_activate::activate_on_enter_or_space(activate.clone())}>
                                                <span class="query-embed__title">{ entry.title.clone() }</span>
                                                <span class="query-embed__meta">
                                                    { for columns.iter().map(|c| celula(
                                                        entry.path.clone(), c.clone(),
                                                        entry.field(c).unwrap_or_default(), "query-embed__chip")) }
                                                </span>
                                            </li>
                                        }
                                    }) }
                                </ul>
                            }
                        </div>
                    }
                }) }
            </div>
        }
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
                                    { for columns.iter().map(|c| celula(
                                        entry.path.clone(), c.clone(),
                                        entry.field(c).unwrap_or_default(), "query-embed__chip")) }
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
                                        <td>{ celula(entry.path.clone(), c.clone(),
                                            entry.field(c).unwrap_or_default(), "query-embed__chip") }</td>
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
        <div class="query-embed" data-nav-group={nav_group.clone()} data-nav-item={nav_group.clone()} data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
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
            if let Some(msg) = (*erro).clone() {
                <p class="query-embed__erro">{ msg }</p>
            }
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
