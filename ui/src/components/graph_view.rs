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
use crate::components::icon::Icon;

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

/// Limites de zoom — abaixo de 0.25x os rótulos ficam ilegíveis, acima
/// de 4x o grafo sai do viewport sem ganhar nada em troca.
const MIN_SCALE: f64 = 0.25;
const MAX_SCALE: f64 = 4.0;

#[function_component(GraphView)]
pub fn graph_view(props: &GraphViewProps) -> Html {
    let nodes = use_state(Vec::<Node>::new);
    let edges = use_state(Vec::<(usize, usize)>::new);
    let loading = use_state(|| true);

    // Zoom (roda do mouse ou botões) + pan (arrastar). `scale`/`pan`
    // viram um `transform` CSS no `<g>` que envolve nós e arestas —
    // `translate` fica FORA do `scale` na composição CSS, então o
    // delta do mouse em px de tela mapeia direto pro pan sem precisar
    // dividir pela escala atual.
    let scale = use_state(|| 1.0f64);
    let pan = use_state(|| (0.0f64, 0.0f64));
    let dragging = use_mut_ref(|| None::<(f64, f64)>);

    let on_wheel = {
        let scale = scale.clone();
        Callback::from(move |e: WheelEvent| {
            e.prevent_default();
            let factor = if e.delta_y() > 0.0 { 0.9 } else { 1.1 };
            scale.set((*scale * factor).clamp(MIN_SCALE, MAX_SCALE));
        })
    };
    let on_mouse_down = {
        let dragging = dragging.clone();
        Callback::from(move |e: MouseEvent| {
            // Sem isso, arrastar pra fazer pan também inicia seleção de
            // texto nativa do navegador (rótulos dos nós são texto) —
            // o cursor vira "I-beam" e solta uma seleção azul no meio
            // do drag. `user-select: none` no CSS já ajuda, mas
            // `preventDefault` no mousedown é o que realmente bloqueia
            // o navegador de começar a seleção em primeiro lugar.
            e.prevent_default();
            *dragging.borrow_mut() = Some((e.client_x() as f64, e.client_y() as f64));
        })
    };
    let on_mouse_move = {
        let dragging = dragging.clone();
        let pan = pan.clone();
        Callback::from(move |e: MouseEvent| {
            let mut d = dragging.borrow_mut();
            if let Some((last_x, last_y)) = *d {
                let (cx, cy) = (e.client_x() as f64, e.client_y() as f64);
                let (px, py) = *pan;
                pan.set((px + (cx - last_x), py + (cy - last_y)));
                *d = Some((cx, cy));
            }
        })
    };
    let stop_dragging = {
        let dragging = dragging.clone();
        Callback::from(move |_: MouseEvent| {
            *dragging.borrow_mut() = None;
        })
    };
    let zoom_in = {
        let scale = scale.clone();
        Callback::from(move |_: MouseEvent| scale.set((*scale * 1.25).clamp(MIN_SCALE, MAX_SCALE)))
    };
    let zoom_out = {
        let scale = scale.clone();
        Callback::from(move |_: MouseEvent| scale.set((*scale / 1.25).clamp(MIN_SCALE, MAX_SCALE)))
    };
    let reset_view = {
        let scale = scale.clone();
        let pan = pan.clone();
        Callback::from(move |_: MouseEvent| {
            scale.set(1.0);
            pan.set((0.0, 0.0));
        })
    };

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
                    <div class="empty-state-card__icon"><Icon name="network" /></div>
                    <div class="empty-state-card__title">{ "Nenhuma página no vault ainda" }</div>
                </div>
            </div>
        };
    }

    let on_page_selected = props.on_page_selected.clone();
    let (pan_x, pan_y) = *pan;
    // Escala ancorada em (400,400) via composição EXPLÍCITA de
    // translate/scale/translate, em vez de `transform-origin` — o
    // comportamento de `transform-origin` em elementos SVG depende de
    // `transform-box` (`fill-box` vs `view-box`), cujo valor padrão
    // difere entre motores de navegador; no WebKitGTK usado pelo Tauri
    // isso fazia o ponto de ancoragem derivar a cada zoom/pan repetido,
    // "espalhando" os nós numa espiral em vez de manter o círculo
    // (bug reportado pelo usuário). Compor a translação/escala direto
    // na lista de funções do `transform` é bem-especificado e igual em
    // qualquer motor, sem depender de `transform-box`.
    let content_transform = format!(
        "transform: translate({px}px, {py}px) translate(400px, 400px) scale({s}) translate(-400px, -400px);",
        px = pan_x, py = pan_y, s = *scale
    );

    html! {
        <div class="graph-view" data-nav-content-root="true">
            <div class="graph-view__toolbar">
                <p class="graph-view__hint">
                    { format!("{} páginas, {} conexões", nodes.len(), edges.len()) }
                </p>
                <div class="graph-view__zoom-controls">
                    <button class="btn btn--ghost btn--xs" onclick={zoom_out} title="Diminuir zoom">{ "−" }</button>
                    <span class="graph-view__zoom-level">{ format!("{}%", (*scale * 100.0).round() as i64) }</span>
                    <button class="btn btn--ghost btn--xs" onclick={zoom_in} title="Aumentar zoom">{ "+" }</button>
                    <button class="btn btn--ghost btn--xs" onclick={reset_view} title="Resetar visualização">{ "Reset" }</button>
                </div>
            </div>
            <p class="graph-view__hint graph-view__hint--muted">
                { "Scroll pra zoom, arraste pra mover, clique num nó pra abrir a página" }
            </p>
            <svg class="graph-view__svg" viewBox="0 0 800 800"
                onwheel={on_wheel}
                onmousedown={on_mouse_down}
                onmousemove={on_mouse_move}
                onmouseup={stop_dragging.clone()}
                onmouseleave={stop_dragging}
            >
                <g style={content_transform}>
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
                            let meta = meta.clone();
                            Callback::from(move |_: MouseEvent| on_page_selected.emit(meta.clone()))
                        };
                        // Ciclo 126: nós de um grafo SVG não são
                        // focáveis/operáveis por teclado por padrão — só
                        // `onclick` não alcança quem navega só com Tab.
                        // `tabindex="0"` bota o nó na ordem de tab (na
                        // ordem em que aparecem no círculo); Enter/Espaço
                        // reaproveita o mesmo callback do clique.
                        let onkeydown = {
                            let on_page_selected = on_page_selected.clone();
                            let meta = meta.clone();
                            Callback::from(move |e: KeyboardEvent| {
                                // `.key()` é o certo pra espaço (" ", literal) num
                                // navegador de verdade; `.code() == "Space"` é
                                // reforço pra ferramentas de automação/drivers que
                                // mandam o nome do código em vez do caractere.
                                if e.key() == "Enter" || e.key() == " " || e.code() == "Space" {
                                    e.prevent_default();
                                    on_page_selected.emit(meta.clone());
                                }
                            })
                        };
                        html! {
                            <g class="graph-view__node" tabindex="0" {onclick} {onkeydown}>
                                <circle cx={node.x.to_string()} cy={node.y.to_string()} r="8" />
                                <text x={(node.x + 12.0).to_string()} y={(node.y + 4.0).to_string()}>{ &node.title }</text>
                            </g>
                        }
                    }) }
                </g>
            </svg>
        </div>
    }
}
