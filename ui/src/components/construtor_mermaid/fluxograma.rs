//! O fluxograma no canvas (ciclo 490): os nós arrastáveis, as ligações
//! puxadas pela alça e o painel do que está selecionado.

use anotadinho_core::diagrama::fluxograma::{Direcao, Fluxograma, Forma, Ligacao, No, TipoLigacao};
use yew::prelude::*;

use super::canvas::{completar, posicao, posicao_livre, CanvasGrafo, EstiloLigacao, FormaNo, LigacaoCanvas, NoCanvas, Selecao};
use super::linhas::{com, texto_do_input, valor_do_select};
use crate::components::icon::Icon;

#[derive(Properties, PartialEq)]
pub struct EditorFluxogramaProps {
    pub modelo: Fluxograma,
    pub on_mudar: Callback<Fluxograma>,
}

fn nome_da_forma(f: Forma) -> &'static str {
    match f {
        Forma::Caixa => "caixa",
        Forma::Arredondado => "arredondado",
        Forma::Decisao => "decisão",
        Forma::Circulo => "círculo",
    }
}

fn nome_da_ligacao(t: TipoLigacao) -> &'static str {
    match t {
        TipoLigacao::Seta => "seta",
        TipoLigacao::Linha => "linha sem seta",
        TipoLigacao::Pontilhada => "seta tracejada",
        TipoLigacao::Grossa => "seta grossa",
    }
}

fn nome_da_direcao(d: Direcao) -> &'static str {
    match d {
        Direcao::TD => "de cima pra baixo",
        Direcao::TB => "de cima pra baixo (TB)",
        Direcao::LR => "da esquerda pra direita",
        Direcao::RL => "da direita pra esquerda",
        Direcao::BT => "de baixo pra cima",
    }
}

fn para_canvas(f: &Fluxograma) -> (Vec<NoCanvas>, Vec<LigacaoCanvas>) {
    let nos = f
        .nos
        .iter()
        .map(|n| NoCanvas {
            chave: n.id.clone(),
            rotulo: n.texto.clone(),
            forma: match n.forma {
                Forma::Caixa => FormaNo::Caixa,
                Forma::Arredondado => FormaNo::Arredondado,
                Forma::Decisao => FormaNo::Decisao,
                Forma::Circulo => FormaNo::Circulo,
            },
            sai: true,
            entra: true,
        })
        .collect();
    let ligacoes = f
        .ligacoes
        .iter()
        .map(|l| LigacaoCanvas {
            de: l.de.clone(),
            para: l.para.clone(),
            rotulo: l.rotulo.clone(),
            estilo: match l.tipo {
                TipoLigacao::Seta => EstiloLigacao::Seta,
                TipoLigacao::Linha => EstiloLigacao::Linha,
                TipoLigacao::Pontilhada => EstiloLigacao::Pontilhada,
                TipoLigacao::Grossa => EstiloLigacao::Grossa,
            },
        })
        .collect();
    (nos, ligacoes)
}

fn proximo_id(f: &Fluxograma) -> String {
    (1..).map(|n| format!("N{n}")).find(|id| !f.nos.iter().any(|n| &n.id == id)).expect("infinito")
}

/// Apaga o que está selecionado. Um nó leva as ligações dele junto: uma
/// ligação para um nó que não existe viraria um nó novo no mermaid.
fn apagar(m: &mut Fluxograma, s: Selecao) {
    match s {
        Selecao::No(i) if i < m.nos.len() => {
            let id = m.nos.remove(i).id;
            m.ligacoes.retain(|l| l.de != id && l.para != id);
            m.posicoes.remove(&id);
        }
        Selecao::Ligacao(k) if k < m.ligacoes.len() => {
            m.ligacoes.remove(k);
        }
        _ => {}
    }
}

#[function_component(EditorFluxograma)]
pub fn editor_fluxograma(props: &EditorFluxogramaProps) -> Html {
    let f = &props.modelo;
    let on = &props.on_mudar;
    let selecao = use_state(|| None::<Selecao>);
    // Índice que não existe mais (apagado pelo código) não é seleção.
    let sel = (*selecao).filter(|s| match s {
        Selecao::No(i) => *i < f.nos.len(),
        Selecao::Ligacao(k) => *k < f.ligacoes.len(),
    });
    let (nos, ligacoes) = para_canvas(f);
    let horizontal = matches!(f.direcao, Direcao::LR | Direcao::RL);
    let posicoes = completar(&nos, &ligacoes, &f.posicoes, horizontal);

    let selecionar = {
        let selecao = selecao.clone();
        Callback::from(move |s: Option<Selecao>| selecao.set(s))
    };
    let mover = com(f, on, |m: &mut Fluxograma, (chave, p): (String, [f64; 2])| {
        m.posicoes.insert(chave, p);
    });
    let ligar = {
        let selecao = selecao.clone();
        let n = f.ligacoes.len();
        com(f, on, move |m: &mut Fluxograma, (de, para): (String, String)| {
            m.ligacoes.push(Ligacao { de, para, tipo: TipoLigacao::Seta, rotulo: String::new() });
            selecao.set(Some(Selecao::Ligacao(n)));
        })
    };
    let apagar_sel = {
        let selecao = selecao.clone();
        com(f, on, move |m: &mut Fluxograma, s: Selecao| {
            apagar(m, s);
            selecao.set(None);
        })
    };
    let novo_no = {
        let selecao = selecao.clone();
        let ocupadas: Vec<[f64; 2]> = (0..nos.len()).map(|i| posicao(&nos, &posicoes, i)).collect();
        let n = f.nos.len();
        com(f, on, move |m: &mut Fluxograma, _: MouseEvent| {
            let id = proximo_id(m);
            // A posição entra JÁ: sem ela, o nó cairia na grade pela ordem,
            // que pode ser em cima de um nó arrastado pra lá.
            m.posicoes.insert(id.clone(), posicao_livre(&ocupadas));
            m.nos.push(No { texto: format!("Nó {}", n + 1), id, forma: Forma::Caixa });
            selecao.set(Some(Selecao::No(n)));
        })
    };
    let mudar_direcao = com(f, on, |m: &mut Fluxograma, e: Event| {
        let v = valor_do_select(e);
        if let Some(d) = Direcao::TODAS.into_iter().find(|d| d.token() == v) {
            m.direcao = d;
        }
    });

    let painel = match sel {
        Some(Selecao::No(i)) => {
            let n = &f.nos[i];
            let mudar_texto = com(f, on, move |m: &mut Fluxograma, e: InputEvent| {
                let t = texto_do_input(e);
                m.nos[i].texto = if t.trim().is_empty() { m.nos[i].id.clone() } else { t.trim().to_string() };
            });
            let mudar_forma = com(f, on, move |m: &mut Fluxograma, e: Event| {
                let v = valor_do_select(e);
                if let Some(nova) = Forma::TODAS.into_iter().find(|x| nome_da_forma(*x) == v) {
                    m.nos[i].forma = nova;
                }
            });
            let apagar = apagar_sel.reform(move |_: MouseEvent| Selecao::No(i));
            html! {
                <div class="construtor-mermaid__painel" data-nav-group="construtor-painel">
                    <span class="construtor-mermaid__id" title="Id no código">{ &n.id }</span>
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Texto do nó"
                        value={n.texto.clone()} oninput={mudar_texto}
                        data-nav-item="construtor-painel-texto" data-nav-parent="construtor-painel" />
                    <select class="construtor-mermaid__campo" aria-label="Forma do nó" onchange={mudar_forma}
                        data-nav-item="construtor-painel-forma" data-nav-parent="construtor-painel">
                        { for Forma::TODAS.iter().map(|x| html! { <option value={nome_da_forma(*x)} selected={*x == n.forma}>{ nome_da_forma(*x) }</option> }) }
                    </select>
                    <button type="button" class="btn btn--ghost btn--sm" onclick={apagar}
                        data-nav-item="construtor-painel-apagar" data-nav-parent="construtor-painel">
                        <Icon name="x" />{ " Apagar nó" }
                    </button>
                </div>
            }
        }
        Some(Selecao::Ligacao(k)) => {
            let l = &f.ligacoes[k];
            let mudar_rotulo = com(f, on, move |m: &mut Fluxograma, e: InputEvent| m.ligacoes[k].rotulo = texto_do_input(e).trim().to_string());
            let mudar_tipo = com(f, on, move |m: &mut Fluxograma, e: Event| {
                let v = valor_do_select(e);
                if let Some(t) = TipoLigacao::TODAS.into_iter().find(|t| t.token() == v) {
                    m.ligacoes[k].tipo = t;
                }
            });
            let inverter = com(f, on, move |m: &mut Fluxograma, _: MouseEvent| {
                let l = &mut m.ligacoes[k];
                std::mem::swap(&mut l.de, &mut l.para);
            });
            let apagar = apagar_sel.reform(move |_: MouseEvent| Selecao::Ligacao(k));
            let nome = |id: &str| f.nos.iter().find(|n| n.id == id).map(|n| n.texto.clone()).unwrap_or_else(|| id.to_string());
            html! {
                <div class="construtor-mermaid__painel" data-nav-group="construtor-painel">
                    <span class="construtor-mermaid__rotulo">{ format!("{} → {}", nome(&l.de), nome(&l.para)) }</span>
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Rótulo da ligação" placeholder="rótulo"
                        value={l.rotulo.clone()} oninput={mudar_rotulo}
                        data-nav-item="construtor-painel-rotulo" data-nav-parent="construtor-painel" />
                    <select class="construtor-mermaid__campo" aria-label="Tipo da ligação" onchange={mudar_tipo}
                        data-nav-item="construtor-painel-tipo" data-nav-parent="construtor-painel">
                        { for TipoLigacao::TODAS.iter().map(|t| html! { <option value={t.token()} selected={*t == l.tipo}>{ nome_da_ligacao(*t) }</option> }) }
                    </select>
                    <button type="button" class="btn btn--ghost btn--sm" onclick={inverter}
                        data-nav-item="construtor-painel-inverter" data-nav-parent="construtor-painel">{ "Inverter" }</button>
                    <button type="button" class="btn btn--ghost btn--sm" onclick={apagar}
                        data-nav-item="construtor-painel-apagar" data-nav-parent="construtor-painel">
                        <Icon name="x" />{ " Apagar ligação" }
                    </button>
                </div>
            }
        }
        None => html! {
            <p class="construtor-mermaid__dica">
                { "Arraste os nós; puxe pela bolinha à direita para ligar. Pelo teclado: Tab chega no nó, setas movem, L num nó e L noutro liga, Delete apaga." }
            </p>
        },
    };

    html! {
        <div class="construtor-mermaid__grafo">
            <div class="construtor-mermaid__linha">
                <button type="button" class="btn btn--ghost btn--sm" onclick={novo_no} data-nav-item="construtor-fluxo-novo-no">
                    <Icon name="plus" />{ " Nó" }
                </button>
                <select class="construtor-mermaid__campo" aria-label="Direção" onchange={mudar_direcao} data-nav-item="construtor-fluxo-direcao">
                    { for Direcao::TODAS.iter().map(|d| html! { <option value={d.token()} selected={*d == f.direcao}>{ nome_da_direcao(*d) }</option> }) }
                </select>
            </div>
            <CanvasGrafo {nos} {ligacoes} {posicoes} selecao={sel}
                on_selecionar={selecionar} on_mover={mover} on_ligar={ligar} on_apagar={apagar_sel} />
            { painel }
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anotadinho_core::diagrama::{ler, Diagrama};

    fn fluxo(c: &str) -> Fluxograma {
        match ler(c) {
            Ok(Diagrama::Fluxograma(f)) => f,
            o => panic!("{o:?}"),
        }
    }

    #[test]
    fn apagar_no_leva_as_ligacoes_e_a_posicao() {
        let mut f = fluxo("graph TD\n  A\n  B\n  C\n  A --> B\n  B --> C\n  A --> C\n  %% anotadinho-posicoes: {\"B\":[1.0,2.0]}");
        apagar(&mut f, Selecao::No(1));
        assert_eq!(f.nos.iter().map(|n| n.id.as_str()).collect::<Vec<_>>(), ["A", "C"]);
        assert_eq!(f.ligacoes.len(), 1);
        assert!(f.posicoes.is_empty());
    }

    #[test]
    fn id_novo_nao_repete() {
        let f = fluxo("graph TD\n  N1\n  N3");
        assert_eq!(proximo_id(&f), "N2");
    }
}
