//! Página `type: graph` — grafo visual das conexões entre páginas via
//! wikilinks (ciclo 120). Nós = páginas, arestas = `[[wikilinks]]`
//! entre elas. Layout: círculo simples (`2πi/n` por índice), SEM
//! física force-directed — evita dependência nova; suficiente pro
//! tamanho de vault atual, reavaliar layout melhor se um vault muito
//! grande deixar o círculo ilegível (ver Não-objetivos do ciclo 120).

use std::collections::{HashMap, HashSet};
use std::f64::consts::PI;

use yew::prelude::*;

use crate::api::{self, PageMeta};

#[derive(Debug, Clone, PartialEq)]
struct Node {
    path: String,
    title: String,
    section: String,
    x: f64,
    y: f64,
}

/// Props da `GraphView`.
#[derive(Properties, PartialEq, Clone)]
pub struct GraphViewProps {
    /// Path do vault.
    pub vault_path: String,
    /// Navega pra uma página ao clicar num nó.
    pub on_page_selected: Callback<PageMeta>,
}

#[function_component(GraphView)]
pub fn graph_view(props: &GraphViewProps) -> Html {
    let nodes = use_state(Vec::<Node>::new);
    let edges = use_state(Vec::<(usize, usize)>::new);
    let loading = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let nodes = nodes.clone();
        let edges = edges.clone();
        let loading = loading.clone();
        use_effect_with(vault_path.clone(), move |vault_path| {
            let vault_path = vault_path.clone();
            let nodes = nodes.clone();
            let edges = edges.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let pages = api::list_pages(&vault_path).await.unwrap_or_default();
                let n = pages.len();
                let cx = 400.0;
                let cy = 400.0;
                let radius = if n > 1 { 320.0 } else { 0.0 };

                let mut node_list = Vec::with_capacity(n);
                let mut title_to_index: HashMap<String, usize> = HashMap::new();
                for (i, p) in pages.iter().enumerate() {
                    let angle = 2.0 * PI * (i as f64) / (n.max(1) as f64);
                    node_list.push(Node {
                        path: p.path.clone(),
                        title: p.title.clone(),
                        section: p.section.clone(),
                        x: cx + radius * angle.cos(),
                        y: cy + radius * angle.sin(),
                    });
                    title_to_index.insert(p.title.to_lowercase(), i);
                }

                // Arestas sem direção (A linka B ou B linka A — mostra a
                // mesma linha) e sem duplicata quando as duas páginas se
                // linkam mutuamente.
                let mut edge_list: Vec<(usize, usize)> = Vec::new();
                let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
                for (i, p) in pages.iter().enumerate() {
                    if let Ok(content) = api::read_page(&vault_path, &p.path).await {
                        for title in crate::wikilink::extract_titles(&content) {
                            if let Some(&j) = title_to_index.get(&title.to_lowercase()) {
                                if j != i {
                                    let pair = if i < j { (i, j) } else { (j, i) };
                                    if seen_pairs.insert(pair) {
                                        edge_list.push(pair);
                                    }
                                }
                            }
                        }
                    }
                }

                nodes.set(node_list);
                edges.set(edge_list);
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="graph-view"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    if nodes.is_empty() {
        return html! {
            <div class="graph-view">
                <div class="empty-state-card">
                    <div class="empty-state-card__icon">{ "🕸" }</div>
                    <div class="empty-state-card__title">{ "Nenhuma página no vault ainda" }</div>
                </div>
            </div>
        };
    }

    let on_page_selected = props.on_page_selected.clone();

    html! {
        <div class="graph-view">
            <p class="graph-view__hint">
                { format!("{} páginas, {} conexões — clique num nó pra abrir a página", nodes.len(), edges.len()) }
            </p>
            <svg class="graph-view__svg" viewBox="0 0 800 800">
                { for edges.iter().map(|&(i, j)| {
                    let a = &nodes[i];
                    let b = &nodes[j];
                    html! {
                        <line class="graph-view__edge"
                            x1={a.x.to_string()} y1={a.y.to_string()}
                            x2={b.x.to_string()} y2={b.y.to_string()} />
                    }
                }) }
                { for nodes.iter().map(|node| {
                    let meta = PageMeta { path: node.path.clone(), title: node.title.clone(), section: node.section.clone() };
                    let onclick = {
                        let on_page_selected = on_page_selected.clone();
                        Callback::from(move |_: MouseEvent| on_page_selected.emit(meta.clone()))
                    };
                    html! {
                        <g class="graph-view__node" {onclick}>
                            <circle cx={node.x.to_string()} cy={node.y.to_string()} r="8" />
                            <text x={(node.x + 12.0).to_string()} y={(node.y + 4.0).to_string()}>{ &node.title }</text>
                        </g>
                    }
                }) }
            </svg>
        </div>
    }
}
