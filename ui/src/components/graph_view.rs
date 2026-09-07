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
    /// A posição no ESPAÇO, do layout do núcleo (ciclo 300).
    ///
    /// A projeção pra tela é feita no desenho, porque ela depende da
    /// rotação — que muda a cada arraste e não pode custar recalcular o
    /// grafo inteiro.
    p: [f64; 3],
}

/// Props da `GraphView`.
#[derive(Properties, PartialEq, Clone)]
pub struct GraphViewProps {
    /// Path do vault.
    pub vault_path: String,
    /// Navega pra uma página ao clicar num nó.
    pub on_page_selected: Callback<PageMeta>,
}

/// Projeta um ponto do espaço na tela, girado.
///
/// Perspectiva fraca (o `1.6` no divisor): forte demais e os nós de trás
/// somem; nenhuma e a rotação não se percebe. Devolve também a
/// profundidade normalizada, que o desenho usa pra tamanho e opacidade —
/// que é o que dá noção de fundo numa tela plana.
fn projetar(p: [f64; 3], giro: (f64, f64), escala: f64) -> (f64, f64, f64) {
    let (gy, gx) = giro;
    // Gira em Y, depois em X.
    let (sy, cy) = gy.sin_cos();
    let x1 = p[0] * cy + p[2] * sy;
    let z1 = -p[0] * sy + p[2] * cy;
    let (sx, cx) = gx.sin_cos();
    let y1 = p[1] * cx - z1 * sx;
    let z2 = p[1] * sx + z1 * cx;

    let perto = 1.0 / (1.6 - z2.clamp(-1.2, 1.2) * 0.35);
    (
        400.0 + x1 * escala * perto,
        400.0 + y1 * escala * perto,
        z2,
    )
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
    /// Rotação em torno de Y e de X, em radianos (ciclo 300).
    ///
    /// É o que faz o layout em três dimensões valer a pena numa tela
    /// plana: parado, profundidade é só um borrão; girando, a estrutura
    /// aparece. Começa levemente torto pra a terceira dimensão ser
    /// visível já na abertura.
    let giro = use_state(|| (0.5f64, -0.3f64));
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
        let giro = giro.clone();
        Callback::from(move |e: MouseEvent| {
            let mut d = dragging.borrow_mut();
            if let Some((last_x, last_y)) = *d {
                let (cx, cy) = (e.client_x() as f64, e.client_y() as f64);
                if e.shift_key() {
                    // Shift arrasta a tela, como antes.
                    let (px, py) = *pan;
                    pan.set((px + (cx - last_x), py + (cy - last_y)));
                } else {
                    // Arrastar GIRA: é o gesto que as pessoas já esperam
                    // de qualquer coisa em três dimensões, e sem ele o
                    // layout 3D seria só um 2D embaralhado.
                    let (gy, gx) = *giro;
                    giro.set((gy + (cx - last_x) * 0.008, gx + (cy - last_y) * 0.008));
                }
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
                // Uma varredura só (ciclo 150): antes era `list_pages()`
                // + um `read_page()` por página só pra achar os
                // wikilinks — N+1 travessias WASM↔Tauri carregando o
                // arquivo inteiro em cada uma.
                let pages = api::scan_vault(&vault_path).await.unwrap_or_default();
                let n = pages.len();
                let mut title_to_index: HashMap<String, usize> = HashMap::new();
                for (i, p) in pages.iter().enumerate() {
                    title_to_index.insert(p.title.to_lowercase(), i);
                }

                // Arestas sem direção (A linka B ou B linka A — mostra a
                // mesma linha) e sem duplicata quando as duas páginas se
                // linkam mutuamente.
                let mut edge_list: Vec<(usize, usize)> = Vec::new();
                let mut seen_pairs: HashSet<(usize, usize)> = HashSet::new();
                for (i, p) in pages.iter().enumerate() {
                    for title in &p.wikilinks {
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

                // O layout vem do núcleo (ciclo 300): força dirigida em
                // três dimensões, no lugar do círculo do ciclo 120.
                //
                // O círculo punha todo nó na borda, e aí TODA aresta
                // virava uma corda cruzando o meio — com 239 páginas e
                // 162 links o desenho é uma bola de linhas que não
                // mostra estrutura nenhuma. Aqui o que se liga fica
                // junto.
                let posicoes = anotadinho_core::grafo::posicionar(
                    &anotadinho_core::grafo::Grafo {
                        nos: n,
                        arestas: edge_list.clone(),
                    },
                );
                let node_list: Vec<Node> = pages
                    .iter()
                    .enumerate()
                    .map(|(i, p)| Node {
                        path: p.path.clone(),
                        title: p.title.clone(),
                        section: p.section.clone(),
                        p: posicoes.get(i).copied().unwrap_or([0.0; 3]),
                    })
                    .collect();

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
                        let (x1, y1, z1) = projetar(nodes[i].p, *giro, 320.0);
                        let (x2, y2, z2) = projetar(nodes[j].p, *giro, 320.0);
                        // Aresta de trás fica mais apagada: é o que
                        // separa "está atrás" de "está longe" numa tela
                        // plana.
                        let fundo = ((z1 + z2) / 2.0).clamp(-1.0, 1.0);
                        let opacidade = 0.15 + 0.35 * (fundo + 1.0) / 2.0;
                        html! {
                            <line class="graph-view__edge"
                                opacity={format!("{opacidade:.2}")}
                                x1={x1.to_string()} y1={y1.to_string()}
                                x2={x2.to_string()} y2={y2.to_string()} />
                        }
                    }) }
                    { for nodes.iter().map(|node| {
                        let (x, y, z) = projetar(node.p, *giro, 320.0);
                        // Perto é maior e mais opaco. É a única pista de
                        // profundidade que um SVG plano oferece de
                        // graça, e sem ela o 3D vira 2D embaralhado.
                        let raio = 5.0 + 4.0 * (z.clamp(-1.0, 1.0) + 1.0) / 2.0;
                        let opacidade = 0.45 + 0.55 * (z.clamp(-1.0, 1.0) + 1.0) / 2.0;
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
                                <circle cx={x.to_string()} cy={y.to_string()}
                                    r={format!("{:.1}", raio)} opacity={format!("{opacidade:.2}")} />
                                // O rótulo só do que está na frente: com
                                // 239 páginas, escrever todos empilha
                                // texto ilegível — e o que está atrás é
                                // justamente o que não se quer ler
                                // agora.
                                if z > 0.15 {
                                    <text x={(x + raio + 4.0).to_string()} y={(y + 4.0).to_string()}>{ &node.title }</text>
                                }
                            </g>
                        }
                    }) }
                </g>
            </svg>
        </div>
    }
}
