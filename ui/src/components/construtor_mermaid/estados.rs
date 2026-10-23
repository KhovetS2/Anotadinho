//! Os estados no canvas (ciclo 490). Início e fim (`[*]`) são pontos fixos
//! do canvas: do início só se sai, no fim só se chega, como no mermaid.
//!
//! O nome do estado vai cru no código, e um `"` digitado vira `'`
//! (`estados::nome_no_codigo`): o mermaid não tem como mostrar aspas ali.

use anotadinho_core::diagrama::estados::{Estado, Estados, Ponta, Transicao};
use yew::prelude::*;

use super::canvas::{completar, posicao, posicao_livre, CanvasGrafo, EstiloLigacao, FormaNo, LigacaoCanvas, NoCanvas, Selecao};
use super::linhas::{com, texto_do_input};
use crate::components::icon::Icon;

#[derive(Properties, PartialEq)]
pub struct EditorEstadosProps {
    pub modelo: Estados,
    pub on_mudar: Callback<Estados>,
}

/// Os dois primeiros nós do canvas são o início e o fim; depois, os
/// estados na ordem do modelo.
const EXTREMOS: usize = 2;

fn ponta_da_chave(chave: &str) -> Ponta {
    match chave {
        c if c == Ponta::Inicio.chave() => Ponta::Inicio,
        c if c == Ponta::Fim.chave() => Ponta::Fim,
        id => Ponta::Estado(id.to_string()),
    }
}

fn para_canvas(e: &Estados) -> (Vec<NoCanvas>, Vec<LigacaoCanvas>) {
    let mut nos = vec![
        NoCanvas { chave: Ponta::Inicio.chave().into(), rotulo: "início".into(), forma: FormaNo::Ponto, sai: true, entra: false },
        NoCanvas { chave: Ponta::Fim.chave().into(), rotulo: "fim".into(), forma: FormaNo::Ponto, sai: false, entra: true },
    ];
    nos.extend(e.estados.iter().map(|s| NoCanvas { chave: s.id.clone(), rotulo: s.texto.clone(), forma: FormaNo::Arredondado, sai: true, entra: true }));
    let ligacoes = e
        .transicoes
        .iter()
        .map(|t| LigacaoCanvas { de: t.de.chave().into(), para: t.para.chave().into(), rotulo: t.rotulo.clone(), estilo: EstiloLigacao::Seta })
        .collect();
    (nos, ligacoes)
}

fn proximo_id(e: &Estados) -> String {
    (1..).map(|n| format!("E{n}")).find(|id| !e.estados.iter().any(|s| &s.id == id)).expect("infinito")
}

/// Apaga o selecionado. Um estado leva as transições dele junto; início e
/// fim não se apagam (são do canvas, não do modelo).
fn apagar(m: &mut Estados, s: Selecao) {
    match s {
        Selecao::No(i) if i >= EXTREMOS && i - EXTREMOS < m.estados.len() => {
            let id = m.estados.remove(i - EXTREMOS).id;
            m.transicoes.retain(|t| t.de.chave() != id && t.para.chave() != id);
            m.posicoes.remove(&id);
        }
        Selecao::Ligacao(k) if k < m.transicoes.len() => {
            m.transicoes.remove(k);
        }
        _ => {}
    }
}

#[function_component(EditorEstados)]
pub fn editor_estados(props: &EditorEstadosProps) -> Html {
    let e = &props.modelo;
    let on = &props.on_mudar;
    let selecao = use_state(|| None::<Selecao>);
    let sel = (*selecao).filter(|s| match s {
        Selecao::No(i) => *i < EXTREMOS + e.estados.len(),
        Selecao::Ligacao(k) => *k < e.transicoes.len(),
    });
    let (nos, ligacoes) = para_canvas(e);
    let posicoes = completar(&nos, &ligacoes, &e.posicoes, false);

    let selecionar = {
        let selecao = selecao.clone();
        Callback::from(move |s: Option<Selecao>| selecao.set(s))
    };
    let mover = com(e, on, |m: &mut Estados, (chave, p): (String, [f64; 2])| {
        m.posicoes.insert(chave, p);
    });
    let ligar = {
        let selecao = selecao.clone();
        let n = e.transicoes.len();
        com(e, on, move |m: &mut Estados, (de, para): (String, String)| {
            let (de, para) = (ponta_da_chave(&de), ponta_da_chave(&para));
            // Início direto no fim não é transição que o mermaid desenhe.
            if de == Ponta::Inicio && para == Ponta::Fim {
                return;
            }
            m.transicoes.push(Transicao { de, para, rotulo: String::new() });
            selecao.set(Some(Selecao::Ligacao(n)));
        })
    };
    let apagar_sel = {
        let selecao = selecao.clone();
        com(e, on, move |m: &mut Estados, s: Selecao| {
            apagar(m, s);
            selecao.set(None);
        })
    };
    let novo_estado = {
        let selecao = selecao.clone();
        let ocupadas: Vec<[f64; 2]> = (0..nos.len()).map(|i| posicao(&nos, &posicoes, i)).collect();
        let n = e.estados.len();
        com(e, on, move |m: &mut Estados, _: MouseEvent| {
            let id = proximo_id(m);
            m.posicoes.insert(id.clone(), posicao_livre(&ocupadas));
            m.estados.push(Estado { texto: format!("Estado {}", n + 1), id });
            selecao.set(Some(Selecao::No(EXTREMOS + n)));
        })
    };

    let painel = match sel {
        Some(Selecao::No(i)) if i >= EXTREMOS => {
            let j = i - EXTREMOS;
            let s = &e.estados[j];
            let mudar_texto = com(e, on, move |m: &mut Estados, ev: InputEvent| {
                let t = texto_do_input(ev);
                m.estados[j].texto = if t.trim().is_empty() { m.estados[j].id.clone() } else { t.trim().to_string() };
            });
            let apagar = apagar_sel.reform(move |_: MouseEvent| Selecao::No(i));
            html! {
                <div class="construtor-mermaid__painel" data-nav-group="construtor-painel">
                    <span class="construtor-mermaid__id" title="Id no código">{ &s.id }</span>
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Nome do estado"
                        value={s.texto.clone()} oninput={mudar_texto}
                        data-nav-item="construtor-painel-texto" data-nav-parent="construtor-painel" />
                    <button type="button" class="btn btn--ghost btn--sm" onclick={apagar}
                        data-nav-item="construtor-painel-apagar" data-nav-parent="construtor-painel">
                        <Icon name="x" />{ " Apagar estado" }
                    </button>
                </div>
            }
        }
        Some(Selecao::No(_)) => html! {
            <p class="construtor-mermaid__dica">{ "Início e fim são fixos: do início só se sai, no fim só se chega." }</p>
        },
        Some(Selecao::Ligacao(k)) => {
            let t = &e.transicoes[k];
            let mudar_rotulo = com(e, on, move |m: &mut Estados, ev: InputEvent| m.transicoes[k].rotulo = texto_do_input(ev).trim().to_string());
            let apagar = apagar_sel.reform(move |_: MouseEvent| Selecao::Ligacao(k));
            let nome = |p: &Ponta| match p {
                Ponta::Inicio => "início".to_string(),
                Ponta::Fim => "fim".to_string(),
                Ponta::Estado(id) => e.estados.iter().find(|s| &s.id == id).map(|s| s.texto.clone()).unwrap_or_else(|| id.clone()),
            };
            html! {
                <div class="construtor-mermaid__painel" data-nav-group="construtor-painel">
                    <span class="construtor-mermaid__rotulo">{ format!("{} → {}", nome(&t.de), nome(&t.para)) }</span>
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Rótulo da transição" placeholder="rótulo"
                        value={t.rotulo.clone()} oninput={mudar_rotulo}
                        data-nav-item="construtor-painel-rotulo" data-nav-parent="construtor-painel" />
                    <button type="button" class="btn btn--ghost btn--sm" onclick={apagar}
                        data-nav-item="construtor-painel-apagar" data-nav-parent="construtor-painel">
                        <Icon name="x" />{ " Apagar transição" }
                    </button>
                </div>
            }
        }
        None => html! {
            <p class="construtor-mermaid__dica">
                { "Arraste os estados; puxe pela bolinha à direita para criar a transição. Pelo teclado: Tab chega no estado, setas movem, L num e L noutro liga, Delete apaga." }
            </p>
        },
    };

    html! {
        <div class="construtor-mermaid__grafo">
            <div class="construtor-mermaid__linha">
                <button type="button" class="btn btn--ghost btn--sm" onclick={novo_estado} data-nav-item="construtor-estados-novo">
                    <Icon name="plus" />{ " Estado" }
                </button>
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

    #[test]
    fn apagar_estado_leva_as_transicoes_e_poupa_os_extremos() {
        let mut e = match ler("stateDiagram-v2\n  A\n  B\n  [*] --> A\n  A --> B\n  B --> [*]") {
            Ok(Diagrama::Estados(e)) => e,
            o => panic!("{o:?}"),
        };
        apagar(&mut e, Selecao::No(0)); // o início: nada
        assert_eq!(e.estados.len(), 2);
        apagar(&mut e, Selecao::No(EXTREMOS)); // A
        assert_eq!(e.estados.iter().map(|s| s.id.as_str()).collect::<Vec<_>>(), ["B"]);
        assert_eq!(e.transicoes.len(), 1);
    }

    #[test]
    fn chave_volta_a_ser_ponta() {
        assert_eq!(ponta_da_chave("[*]inicio"), Ponta::Inicio);
        assert_eq!(ponta_da_chave("[*]fim"), Ponta::Fim);
        assert_eq!(ponta_da_chave("A"), Ponta::Estado("A".into()));
    }
}
