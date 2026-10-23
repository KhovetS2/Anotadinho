//! O canvas dos diagramas que são grafo: fluxograma e estados (ciclo 490).
//!
//! SVG próprio, sem biblioteca: arrastar um nó, puxar uma ligação pela
//! alça e selecionar. O canvas não conhece o modelo do mermaid; quem o usa
//! traduz nós e ligações para cá e recebe de volta o que a pessoa fez.
//!
//! A posição aqui é só arrumação (RNF2): o mermaid recalcula o desenho.

use std::cell::RefCell;
use std::rc::Rc;

use anotadinho_core::diagrama::Posicoes;
use yew::prelude::*;

/// O tamanho lógico do canvas. O SVG escala para caber; as posições são
/// guardadas nesta unidade.
/// 640 e não 900: o canvas aparece com ~560 px de largura, e a 900 o texto
/// dos nós ficava com ~8 px na tela (visto no ciclo 490).
pub const LARGURA: f64 = 640.0;
pub const ALTURA: f64 = 400.0;
const NO_L: f64 = 132.0;
const NO_A: f64 = 44.0;
const PONTO_R: f64 = 11.0;
/// Quanto um nó anda por tecla de seta.
const PASSO: f64 = 20.0;
/// Menos que isto entre apertar e soltar é clique, não arraste.
const LIMIAR_DE_ARRASTE: f64 = 4.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FormaNo {
    Caixa,
    Arredondado,
    Decisao,
    Circulo,
    /// Início e fim dos estados.
    Ponto,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EstiloLigacao {
    Seta,
    Linha,
    Pontilhada,
    Grossa,
}

#[derive(Clone, PartialEq, Debug)]
pub struct NoCanvas {
    pub chave: String,
    pub rotulo: String,
    pub forma: FormaNo,
    /// Pode ser ORIGEM de ligação (o fim dos estados não pode).
    pub sai: bool,
    /// Pode ser DESTINO (o início não pode).
    pub entra: bool,
}

#[derive(Clone, PartialEq, Debug)]
pub struct LigacaoCanvas {
    pub de: String,
    pub para: String,
    pub rotulo: String,
    pub estilo: EstiloLigacao,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Selecao {
    No(usize),
    Ligacao(usize),
}

#[derive(Properties, PartialEq)]
pub struct CanvasProps {
    pub nos: Vec<NoCanvas>,
    pub ligacoes: Vec<LigacaoCanvas>,
    pub posicoes: Posicoes,
    pub selecao: Option<Selecao>,
    pub on_selecionar: Callback<Option<Selecao>>,
    pub on_mover: Callback<(String, [f64; 2])>,
    pub on_ligar: Callback<(String, String)>,
    pub on_apagar: Callback<Selecao>,
}

/// Onde o nó `i` fica: a posição guardada, ou uma grade pela ordem (os
/// nós ainda sem lugar ficam arrumados, e não empilhados no canto).
pub fn posicao(nos: &[NoCanvas], posicoes: &Posicoes, i: usize) -> [f64; 2] {
    if let Some(p) = posicoes.get(&nos[i].chave) {
        return *p;
    }
    let colunas = 4usize;
    let (c, l) = (i % colunas, i / colunas);
    [80.0 + c as f64 * 155.0, 50.0 + l as f64 * 90.0]
}

/// As posições de todos os nós: as guardadas e, pros que não têm, uma
/// arrumação em camadas pela ligação (busca em largura a partir de quem não
/// tem entrada), na direção do diagrama. A grade pela ordem deixava um
/// fluxograma de quatro passos numa linha só, com as setas espremidas
/// (visto no ciclo 490). O resultado não vai pro modelo: só o que a pessoa
/// arrasta vira posição guardada.
pub fn completar(nos: &[NoCanvas], ligacoes: &[LigacaoCanvas], guardadas: &Posicoes, horizontal: bool) -> Posicoes {
    let n = nos.len();
    let idx = |chave: &str| nos.iter().position(|x| x.chave == chave);
    let arestas: Vec<(usize, usize)> = ligacoes
        .iter()
        .filter_map(|l| Some((idx(&l.de)?, idx(&l.para)?)))
        .filter(|(a, b)| a != b)
        .collect();
    let mut nivel: Vec<Option<usize>> = vec![None; n];
    let tem_entrada: Vec<bool> = (0..n).map(|i| arestas.iter().any(|(_, b)| *b == i)).collect();
    // Raízes primeiro (sem entrada); depois, quem sobrou (ciclos, soltos).
    let ordem: Vec<usize> = (0..n).filter(|i| !tem_entrada[*i]).chain((0..n).filter(|i| tem_entrada[*i])).collect();
    for &raiz in &ordem {
        if nivel[raiz].is_some() {
            continue;
        }
        nivel[raiz] = Some(0);
        let mut fila = std::collections::VecDeque::from([raiz]);
        while let Some(u) = fila.pop_front() {
            let nu = nivel[u].unwrap_or(0);
            for &(a, b) in &arestas {
                if a == u && nivel[b].is_none() {
                    nivel[b] = Some(nu + 1);
                    fila.push_back(b);
                }
            }
        }
    }
    let mut por_nivel: std::collections::BTreeMap<usize, Vec<usize>> = Default::default();
    for i in 0..n {
        por_nivel.entry(nivel[i].unwrap_or(0)).or_default().push(i);
    }
    let mut todas = guardadas.clone();
    for (camada, membros) in por_nivel {
        let k = membros.len() as f64;
        for (j, &i) in membros.iter().enumerate() {
            if guardadas.contains_key(&nos[i].chave) {
                continue;
            }
            let desvio = j as f64 - (k - 1.0) / 2.0;
            let p = if horizontal {
                [80.0 + camada as f64 * 165.0, ALTURA / 2.0 + desvio * 80.0]
            } else {
                [LARGURA / 2.0 + desvio * 160.0, 40.0 + camada as f64 * 85.0]
            };
            todas.insert(nos[i].chave.clone(), presa(p));
        }
    }
    todas
}

/// Uma posição livre pra um nó novo: a primeira célula da grade que não
/// está em cima de ninguém.
pub fn posicao_livre(ocupadas: &[[f64; 2]]) -> [f64; 2] {
    for l in 0..8 {
        for c in 0..4 {
            let p = [80.0 + c as f64 * 155.0, 50.0 + l as f64 * 90.0];
            if ocupadas.iter().all(|o| (o[0] - p[0]).abs() > NO_L * 0.6 || (o[1] - p[1]).abs() > NO_A * 1.2) {
                return p;
            }
        }
    }
    [LARGURA / 2.0, ALTURA / 2.0]
}

fn presa(p: [f64; 2]) -> [f64; 2] {
    [p[0].clamp(NO_L / 2.0, LARGURA - NO_L / 2.0), p[1].clamp(NO_A / 2.0, ALTURA - NO_A / 2.0)]
}

fn meia_caixa(f: FormaNo) -> (f64, f64) {
    match f {
        FormaNo::Ponto => (PONTO_R, PONTO_R),
        FormaNo::Circulo => (NO_A * 0.75, NO_A * 0.75),
        _ => (NO_L / 2.0, NO_A / 2.0),
    }
}

/// O ponto da borda do nó (aproximada por retângulo) na direção de `alvo`:
/// a seta começa e termina na borda, não no centro escondido.
fn borda(centro: [f64; 2], forma: FormaNo, alvo: [f64; 2]) -> [f64; 2] {
    let (dx, dy) = (alvo[0] - centro[0], alvo[1] - centro[1]);
    if dx.abs() < f64::EPSILON && dy.abs() < f64::EPSILON {
        return centro;
    }
    let (mx, my) = meia_caixa(forma);
    let t = (mx / dx.abs().max(f64::EPSILON)).min(my / dy.abs().max(f64::EPSILON));
    [centro[0] + dx * t, centro[1] + dy * t]
}

/// Qual nó está sob o ponto (em unidades do canvas), do de cima pro de baixo.
fn no_sob(nos: &[NoCanvas], posicoes: &Posicoes, p: [f64; 2]) -> Option<usize> {
    (0..nos.len()).rev().find(|&i| {
        let c = posicao(nos, posicoes, i);
        let (mx, my) = meia_caixa(nos[i].forma);
        (p[0] - c[0]).abs() <= mx && (p[1] - c[1]).abs() <= my
    })
}

/// O ponto de controle da curva quando existe a ligação de volta. O lado
/// sai da direção (de → para): a volta, com a direção ao contrário, cai do
/// outro lado sozinha.
fn controle_da_volta(todas: &[LigacaoCanvas], l: &LigacaoCanvas, pa: [f64; 2], pb: [f64; 2]) -> Option<[f64; 2]> {
    if l.de == l.para || !todas.iter().any(|o| o.de == l.para && o.para == l.de) {
        return None;
    }
    let (dx, dy) = (pb[0] - pa[0], pb[1] - pa[1]);
    let tam = (dx * dx + dy * dy).sqrt().max(1.0);
    let afasta = 34.0;
    Some([(pa[0] + pb[0]) / 2.0 - dy / tam * afasta, (pa[1] + pb[1]) / 2.0 + dx / tam * afasta])
}

#[derive(Clone, Debug)]
enum Gesto {
    Arrasto { i: usize, desvio: [f64; 2], inicio: [f64; 2], agora: [f64; 2] },
    Ligando { i: usize, ponta: [f64; 2] },
}

/// Ponteiro → unidades do canvas.
fn no_canvas(svg: &NodeRef, e: &PointerEvent) -> Option<[f64; 2]> {
    let el = svg.cast::<web_sys::Element>()?;
    let r = el.get_bounding_client_rect();
    if r.width() <= 0.0 || r.height() <= 0.0 {
        return None;
    }
    Some([
        (e.client_x() as f64 - r.left()) * LARGURA / r.width(),
        (e.client_y() as f64 - r.top()) * ALTURA / r.height(),
    ])
}

#[function_component(CanvasGrafo)]
pub fn canvas_grafo(props: &CanvasProps) -> Html {
    let svg = use_node_ref();
    // O gesto em curso, lido pelo `pointermove` entre um render e outro:
    // `use_mut_ref`, não `use_state` (AGENTS.md — closure congela estado).
    let gesto: Rc<RefCell<Option<Gesto>>> = use_mut_ref(|| None);
    // A ligação começada pelo teclado (`L` num nó, `L` noutro).
    let ligar_de: Rc<RefCell<Option<usize>>> = use_mut_ref(|| None);
    let redesenhar = use_force_update();

    let nos = &props.nos;
    let pos_atual = |i: usize| -> [f64; 2] {
        match &*gesto.borrow() {
            Some(Gesto::Arrasto { i: j, agora, .. }) if *j == i => *agora,
            _ => posicao(nos, &props.posicoes, i),
        }
    };

    let ao_mover = {
        let gesto = gesto.clone();
        let svg = svg.clone();
        let redesenhar = redesenhar.clone();
        Callback::from(move |e: PointerEvent| {
            let Some(p) = no_canvas(&svg, &e) else { return };
            let mut g = gesto.borrow_mut();
            match g.as_mut() {
                Some(Gesto::Arrasto { desvio, agora, .. }) => *agora = presa([p[0] - desvio[0], p[1] - desvio[1]]),
                Some(Gesto::Ligando { ponta, .. }) => *ponta = p,
                None => return,
            }
            drop(g);
            redesenhar.force_update();
        })
    };
    let ao_soltar = {
        let gesto = gesto.clone();
        let svg = svg.clone();
        let nos = props.nos.clone();
        let posicoes = props.posicoes.clone();
        let on_mover = props.on_mover.clone();
        let on_ligar = props.on_ligar.clone();
        let on_selecionar = props.on_selecionar.clone();
        let redesenhar = redesenhar.clone();
        Callback::from(move |e: PointerEvent| {
            let Some(feito) = gesto.borrow_mut().take() else { return };
            match feito {
                Gesto::Arrasto { i, inicio, agora, .. } => {
                    let andou = ((agora[0] - inicio[0]).powi(2) + (agora[1] - inicio[1]).powi(2)).sqrt();
                    if andou >= LIMIAR_DE_ARRASTE {
                        // Só aqui a posição vira modelo (e código): emitir a
                        // cada movimento redesenharia a prévia o tempo todo.
                        on_mover.emit((nos[i].chave.clone(), [agora[0].round(), agora[1].round()]));
                    }
                    on_selecionar.emit(Some(Selecao::No(i)));
                }
                Gesto::Ligando { i, .. } => {
                    if let Some(j) = no_canvas(&svg, &e).and_then(|p| no_sob(&nos, &posicoes, p)) {
                        if j != i && nos[j].entra {
                            on_ligar.emit((nos[i].chave.clone(), nos[j].chave.clone()));
                        }
                    }
                }
            }
            redesenhar.force_update();
        })
    };
    let ao_cancelar = {
        let gesto = gesto.clone();
        let redesenhar = redesenhar.clone();
        Callback::from(move |_: PointerEvent| {
            *gesto.borrow_mut() = None;
            redesenhar.force_update();
        })
    };
    let fundo = {
        let on = props.on_selecionar.clone();
        Callback::from(move |_: MouseEvent| on.emit(None))
    };

    let ligacoes = props.ligacoes.iter().enumerate().map(|(k, l)| {
        let (Some(a), Some(b)) = (nos.iter().position(|n| n.chave == l.de), nos.iter().position(|n| n.chave == l.para)) else {
            return html! {};
        };
        let (pa, pb) = (pos_atual(a), pos_atual(b));
        let d = if a == b {
            // Laço: um arco por cima do próprio nó.
            let (mx, my) = meia_caixa(nos[a].forma);
            let (x1, y) = (pa[0] - mx * 0.4, pa[1] - my);
            let x2 = pa[0] + mx * 0.4;
            format!("M {x1} {y} C {x1} {} {x2} {} {x2} {y}", y - 50.0, y - 50.0)
        } else if let Some(c) = controle_da_volta(&props.ligacoes, l, pa, pb) {
            // Ida e volta entre os mesmos nós: duas curvas, uma de cada
            // lado, em vez de duas retas uma em cima da outra.
            let (o, f) = (borda(pa, nos[a].forma, c), borda(pb, nos[b].forma, c));
            format!("M {} {} Q {} {} {} {}", o[0], o[1], c[0], c[1], f[0], f[1])
        } else {
            let (o, f) = (borda(pa, nos[a].forma, pb), borda(pb, nos[b].forma, pa));
            format!("M {} {} L {} {}", o[0], o[1], f[0], f[1])
        };
        let meio = if a == b {
            [pa[0], pa[1] - meia_caixa(nos[a].forma).1 - 42.0]
        } else if let Some(c) = controle_da_volta(&props.ligacoes, l, pa, pb) {
            [(pa[0] + 2.0 * c[0] + pb[0]) / 4.0, (pa[1] + 2.0 * c[1] + pb[1]) / 4.0]
        } else {
            [(pa[0] + pb[0]) / 2.0, (pa[1] + pb[1]) / 2.0]
        };
        let selecionada = props.selecao == Some(Selecao::Ligacao(k));
        let classe = classes!(
            "construtor-canvas__ligacao",
            match l.estilo {
                EstiloLigacao::Pontilhada => Some("construtor-canvas__ligacao--pontilhada"),
                EstiloLigacao::Grossa => Some("construtor-canvas__ligacao--grossa"),
                _ => None,
            },
            selecionada.then_some("construtor-canvas__ligacao--selecionada"),
        );
        let ponta = (l.estilo != EstiloLigacao::Linha).then_some("url(#construtor-canvas-ponta)");
        let selecionar = {
            let on = props.on_selecionar.clone();
            Callback::from(move |e: MouseEvent| {
                e.stop_propagation();
                on.emit(Some(Selecao::Ligacao(k)));
            })
        };
        let teclar = {
            let on_sel = props.on_selecionar.clone();
            let on_apagar = props.on_apagar.clone();
            Callback::from(move |e: KeyboardEvent| match e.key().as_str() {
                "Enter" | " " => {
                    e.prevent_default();
                    on_sel.emit(Some(Selecao::Ligacao(k)));
                }
                "Delete" | "Backspace" => {
                    e.prevent_default();
                    on_apagar.emit(Selecao::Ligacao(k));
                }
                _ => {}
            })
        };
        html! {
            <g class={classe} data-ligacao={k.to_string()}>
                // A linha visível é fina; esta, larga e transparente, é o
                // alvo do clique.
                <path class="construtor-canvas__ligacao-alvo" d={d.clone()} tabindex="0"
                    role="button" aria-label={format!("Ligação de {} para {}", nos[a].rotulo, nos[b].rotulo)}
                    data-nav-item={format!("construtor-canvas-ligacao-{k}")} data-nav-parent="construtor-canvas"
                    onclick={selecionar} onkeydown={teclar} />
                <path class="construtor-canvas__ligacao-linha" d={d} marker-end={ponta} />
                if !l.rotulo.is_empty() {
                    <text class="construtor-canvas__ligacao-rotulo" x={meio[0].to_string()} y={(meio[1] - 4.0).to_string()} text-anchor="middle">{ &l.rotulo }</text>
                }
            </g>
        }
    });

    let nos_html = nos.iter().enumerate().map(|(i, n)| {
        let c = pos_atual(i);
        let selecionado = props.selecao == Some(Selecao::No(i));
        let ligando_daqui = *ligar_de.borrow() == Some(i);
        let classe = classes!(
            "construtor-canvas__no",
            selecionado.then_some("construtor-canvas__no--selecionado"),
            ligando_daqui.then_some("construtor-canvas__no--ligando"),
        );
        let (mx, my) = meia_caixa(n.forma);
        let corpo = match n.forma {
            FormaNo::Caixa => html! { <rect x={(c[0] - mx).to_string()} y={(c[1] - my).to_string()} width={(2.0 * mx).to_string()} height={(2.0 * my).to_string()} rx="3" /> },
            FormaNo::Arredondado => html! { <rect x={(c[0] - mx).to_string()} y={(c[1] - my).to_string()} width={(2.0 * mx).to_string()} height={(2.0 * my).to_string()} rx="18" /> },
            FormaNo::Decisao => html! { <polygon points={format!("{},{} {},{} {},{} {},{}", c[0], c[1] - my, c[0] + mx, c[1], c[0], c[1] + my, c[0] - mx, c[1])} /> },
            FormaNo::Circulo => html! { <circle cx={c[0].to_string()} cy={c[1].to_string()} r={mx.to_string()} /> },
            FormaNo::Ponto => html! { <circle class="construtor-canvas__ponto" cx={c[0].to_string()} cy={c[1].to_string()} r={mx.to_string()} /> },
        };
        let apertar = {
            let gesto = gesto.clone();
            let svg = svg.clone();
            let base = c;
            Callback::from(move |e: PointerEvent| {
                if e.button() != 0 {
                    return;
                }
                e.stop_propagation();
                let Some(p) = no_canvas(&svg, &e) else { return };
                // A captura mantém o arraste mesmo com o ponteiro saindo do
                // nó (ou do canvas) no meio do gesto.
                if let Some(el) = svg.cast::<web_sys::Element>() {
                    let _ = el.set_pointer_capture(e.pointer_id());
                }
                *gesto.borrow_mut() = Some(Gesto::Arrasto { i, desvio: [p[0] - base[0], p[1] - base[1]], inicio: base, agora: base });
            })
        };
        let puxar = {
            let gesto = gesto.clone();
            let svg = svg.clone();
            Callback::from(move |e: PointerEvent| {
                if e.button() != 0 {
                    return;
                }
                e.stop_propagation();
                let Some(p) = no_canvas(&svg, &e) else { return };
                if let Some(el) = svg.cast::<web_sys::Element>() {
                    let _ = el.set_pointer_capture(e.pointer_id());
                }
                *gesto.borrow_mut() = Some(Gesto::Ligando { i, ponta: p });
            })
        };
        let teclar = {
            let on_mover = props.on_mover.clone();
            let on_sel = props.on_selecionar.clone();
            let on_ligar = props.on_ligar.clone();
            let on_apagar = props.on_apagar.clone();
            let ligar_de = ligar_de.clone();
            let redesenhar = redesenhar.clone();
            let nos = props.nos.clone();
            Callback::from(move |e: KeyboardEvent| {
                let passo = match e.key().as_str() {
                    "ArrowLeft" => Some([-PASSO, 0.0]),
                    "ArrowRight" => Some([PASSO, 0.0]),
                    "ArrowUp" => Some([0.0, -PASSO]),
                    "ArrowDown" => Some([0.0, PASSO]),
                    _ => None,
                };
                if let Some([dx, dy]) = passo {
                    e.prevent_default();
                    e.stop_propagation();
                    on_mover.emit((nos[i].chave.clone(), presa([c[0] + dx, c[1] + dy])));
                    return;
                }
                match e.key().as_str() {
                    "Enter" | " " => {
                        e.prevent_default();
                        on_sel.emit(Some(Selecao::No(i)));
                    }
                    "l" | "L" => {
                        e.prevent_default();
                        let origem = ligar_de.borrow_mut().take();
                        match origem {
                            Some(o) if o != i && nos[o].sai && nos[i].entra => on_ligar.emit((nos[o].chave.clone(), nos[i].chave.clone())),
                            Some(_) => {}
                            None if nos[i].sai => *ligar_de.borrow_mut() = Some(i),
                            None => {}
                        }
                        redesenhar.force_update();
                    }
                    "Escape" if ligar_de.borrow().is_some() => {
                        // Cancela a ligação sem fechar o modal.
                        e.prevent_default();
                        e.stop_propagation();
                        *ligar_de.borrow_mut() = None;
                        redesenhar.force_update();
                    }
                    "Delete" | "Backspace" => {
                        e.prevent_default();
                        on_apagar.emit(Selecao::No(i));
                    }
                    _ => {}
                }
            })
        };
        html! {
            <g class={classe} data-chave={n.chave.clone()} tabindex="0" role="button"
                aria-label={format!("Nó {}", n.rotulo)} aria-pressed={selecionado.to_string()}
                data-nav-item={format!("construtor-canvas-no-{i}")} data-nav-parent="construtor-canvas"
                onpointerdown={apertar} onkeydown={teclar}
                onclick={Callback::from(|e: MouseEvent| e.stop_propagation())}>
                { corpo }
                if n.forma != FormaNo::Ponto {
                    <text class="construtor-canvas__no-rotulo" x={c[0].to_string()} y={(c[1] + 4.0).to_string()} text-anchor="middle">{ &n.rotulo }</text>
                } else {
                    <text class="construtor-canvas__no-rotulo" x={c[0].to_string()} y={(c[1] + mx + 14.0).to_string()} text-anchor="middle">{ &n.rotulo }</text>
                }
                if n.sai {
                    <circle class="construtor-canvas__alca" cx={(c[0] + mx).to_string()} cy={c[1].to_string()} r="6"
                        aria-hidden="true" onpointerdown={puxar} />
                }
            </g>
        }
    });

    let rascunho = match &*gesto.borrow() {
        Some(Gesto::Ligando { i, ponta }) => {
            let o = borda(posicao(nos, &props.posicoes, *i), nos[*i].forma, *ponta);
            html! { <line class="construtor-canvas__rascunho" x1={o[0].to_string()} y1={o[1].to_string()} x2={ponta[0].to_string()} y2={ponta[1].to_string()} /> }
        }
        _ => html! {},
    };

    html! {
        <svg class="construtor-canvas" ref={svg.clone()} viewBox={format!("0 0 {LARGURA} {ALTURA}")}
            role="group" aria-label="Canvas do diagrama" data-nav-group="construtor-canvas"
            onpointermove={ao_mover} onpointerup={ao_soltar} onpointercancel={ao_cancelar} onclick={fundo}>
            <defs>
                <marker id="construtor-canvas-ponta" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse">
                    <path class="construtor-canvas__ponta" d="M 0 0 L 10 5 L 0 10 z" />
                </marker>
            </defs>
            { for ligacoes }
            { for nos_html }
            { rascunho }
        </svg>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn no(chave: &str) -> NoCanvas {
        NoCanvas { chave: chave.into(), rotulo: chave.into(), forma: FormaNo::Caixa, sai: true, entra: true }
    }

    #[test]
    fn sem_posicao_guardada_vai_pra_grade_na_ordem() {
        let nos = vec![no("a"), no("b")];
        let mut p = Posicoes::new();
        assert_eq!(posicao(&nos, &p, 0), [80.0, 50.0]);
        assert_eq!(posicao(&nos, &p, 1), [235.0, 50.0]);
        p.insert("b".into(), [400.0, 300.0]);
        assert_eq!(posicao(&nos, &p, 1), [400.0, 300.0]);
    }

    #[test]
    fn posicao_livre_pula_a_ocupada() {
        assert_eq!(posicao_livre(&[]), [80.0, 50.0]);
        assert_eq!(posicao_livre(&[[80.0, 50.0]]), [235.0, 50.0]);
    }

    #[test]
    fn camadas_seguem_as_ligacoes_e_respeitam_o_guardado() {
        let nos = vec![no("a"), no("b"), no("c"), no("d")];
        let lig = |de: &str, para: &str| LigacaoCanvas { de: de.into(), para: para.into(), rotulo: String::new(), estilo: EstiloLigacao::Seta };
        let ligacoes = vec![lig("a", "b"), lig("b", "c"), lig("b", "d"), lig("c", "a")];
        let p = completar(&nos, &ligacoes, &Posicoes::new(), false);
        // a no topo; b embaixo; c e d lado a lado na terceira camada. A volta
        // c → a não puxa o a pra baixo.
        assert!(p["a"][1] < p["b"][1] && p["b"][1] < p["c"][1]);
        assert_eq!(p["c"][1], p["d"][1]);
        assert_ne!(p["c"][0], p["d"][0]);
        let mut guardadas = Posicoes::new();
        guardadas.insert("b".into(), [700.0, 400.0]);
        assert_eq!(completar(&nos, &ligacoes, &guardadas, true)["b"], [700.0, 400.0]);
        let h = completar(&nos, &ligacoes, &Posicoes::new(), true);
        assert!(h["a"][0] < h["b"][0], "horizontal anda em x");
    }

    #[test]
    fn so_ida_e_volta_vira_curva_e_cada_uma_de_um_lado() {
        let lig = |de: &str, para: &str| LigacaoCanvas { de: de.into(), para: para.into(), rotulo: String::new(), estilo: EstiloLigacao::Seta };
        let todas = vec![lig("a", "b"), lig("b", "a"), lig("b", "c")];
        let (pa, pb) = ([0.0, 0.0], [100.0, 0.0]);
        let ida = controle_da_volta(&todas, &todas[0], pa, pb).unwrap();
        let volta = controle_da_volta(&todas, &todas[1], pb, pa).unwrap();
        assert!(ida[1] * volta[1] < 0.0, "as duas curvas do mesmo lado: {ida:?} {volta:?}");
        assert_eq!(controle_da_volta(&todas, &todas[2], pa, pb), None);
    }

    #[test]
    fn borda_fica_na_caixa() {
        let b = borda([100.0, 100.0], FormaNo::Caixa, [400.0, 100.0]);
        assert_eq!(b, [100.0 + NO_L / 2.0, 100.0]);
        let b = borda([100.0, 100.0], FormaNo::Caixa, [100.0, 400.0]);
        assert_eq!(b, [100.0, 100.0 + NO_A / 2.0]);
    }

    #[test]
    fn no_sob_acha_pelo_retangulo() {
        let nos = vec![no("a"), no("b")];
        let p = Posicoes::new();
        assert_eq!(no_sob(&nos, &p, [85.0, 60.0]), Some(0));
        assert_eq!(no_sob(&nos, &p, [235.0, 50.0]), Some(1));
        assert_eq!(no_sob(&nos, &p, [600.0, 400.0]), None);
    }
}
