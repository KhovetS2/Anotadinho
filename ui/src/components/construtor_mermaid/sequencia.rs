//! Formulário da sequência (ciclo 489): participantes e, em ordem,
//! mensagens e notas. Num canvas a posição de uma mensagem não diria nada;
//! a ordem é que diz (ambiguidade da spec, resolvida assim).

use anotadinho_core::diagrama::sequencia::{Item, Lado, Participante, Seta, Sequencia};
use yew::prelude::*;

use super::linhas::{aplicar, botoes, com, marcado, texto_do_input, valor_do_select, AcaoLinha};
use crate::components::icon::Icon;

#[derive(Properties, PartialEq)]
pub struct EditorSequenciaProps {
    pub modelo: Sequencia,
    pub on_mudar: Callback<Sequencia>,
}

fn nome_da_seta(s: Seta) -> &'static str {
    match s {
        Seta::Sincrona => "pedido (->>)",
        Seta::Resposta => "resposta (-->>)",
        Seta::Assincrona => "sem esperar (-))",
        Seta::AssincronaPontilhada => "sem esperar, tracejada (--))",
        Seta::Linha => "linha (->)",
        Seta::LinhaPontilhada => "linha tracejada (-->)",
        Seta::Cruz => "falhou (-x)",
        Seta::CruzPontilhada => "falhou, tracejada (--x)",
    }
}

fn nome_do_lado(l: Lado) -> &'static str {
    match l {
        Lado::Direita => "à direita de",
        Lado::Esquerda => "à esquerda de",
        Lado::Sobre => "sobre",
    }
}

const LADOS: [Lado; 3] = [Lado::Direita, Lado::Esquerda, Lado::Sobre];

/// Quantas mensagens e notas citam o participante: apagar um citado
/// deixaria mensagem apontando pro nada.
fn citacoes(s: &Sequencia, id: &str) -> usize {
    s.itens
        .iter()
        .filter(|i| match i {
            Item::Mensagem { de, para, .. } => de == id || para == id,
            Item::Nota { alvos, .. } => alvos.iter().any(|a| a == id),
        })
        .count()
}

fn proximo_id(s: &Sequencia) -> String {
    (1..).map(|n| format!("P{n}")).find(|id| !s.participantes.iter().any(|p| &p.id == id)).expect("infinito")
}

fn select_de_participante(s: &Sequencia, atual: &str, nav: String, on: Callback<String>) -> Html {
    html! {
        <select class="construtor-mermaid__campo" data-nav-item={nav} data-nav-parent="construtor-seq-itens"
            onchange={Callback::from(move |e: Event| on.emit(valor_do_select(e)))}>
            { for s.participantes.iter().map(|p| html! {
                <option value={p.id.clone()} selected={p.id == atual}>{ &p.nome }</option>
            }) }
        </select>
    }
}

#[function_component(EditorSequencia)]
pub fn editor_sequencia(props: &EditorSequenciaProps) -> Html {
    let s = &props.modelo;
    let on = &props.on_mudar;
    let n_part = s.participantes.len();
    let n_itens = s.itens.len();

    let participantes = s.participantes.iter().enumerate().map(|(i, p)| {
        let usado = citacoes(s, &p.id);
        let bloqueio = (usado > 0).then(|| format!("Usado em {usado} mensagens ou notas"));
        let mudar_nome = com(s, on, move |m: &mut Sequencia, e: InputEvent| {
            let nome = texto_do_input(e);
            // Nome vazio vira o id: `participant P1 as` sem nome não é mermaid.
            m.participantes[i].nome = if nome.trim().is_empty() { m.participantes[i].id.clone() } else { nome.trim().to_string() };
        });
        let mudar_ator = com(s, on, move |m: &mut Sequencia, e: Event| m.participantes[i].ator = marcado(e));
        let acao = com(s, on, move |m: &mut Sequencia, a: AcaoLinha| aplicar(&mut m.participantes, i, a));
        html! {
            <li class="construtor-mermaid__linha" key={p.id.clone()}>
                <span class="construtor-mermaid__id" title="Id no código">{ &p.id }</span>
                <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Nome do participante"
                    value={p.nome.clone()} oninput={mudar_nome}
                    data-nav-item={format!("construtor-seq-part-{i}-nome")} data-nav-parent="construtor-seq-part" />
                <label class="construtor-mermaid__marca">
                    <input type="checkbox" checked={p.ator} onchange={mudar_ator}
                        data-nav-item={format!("construtor-seq-part-{i}-ator")} data-nav-parent="construtor-seq-part" />
                    { "pessoa" }
                </label>
                { botoes("construtor-seq-part", i, n_part, bloqueio, acao) }
            </li>
        }
    });

    let itens = s.itens.iter().enumerate().map(|(i, item)| {
        let acao = com(s, on, move |m: &mut Sequencia, a: AcaoLinha| aplicar(&mut m.itens, i, a));
        let nav = |campo: &str| format!("construtor-seq-item-{i}-{campo}");
        let corpo = match item {
            Item::Mensagem { de, para, seta, texto } => {
                let mudar_de = com(s, on, move |m: &mut Sequencia, v: String| {
                    if let Item::Mensagem { de, .. } = &mut m.itens[i] { *de = v; }
                });
                let mudar_para = com(s, on, move |m: &mut Sequencia, v: String| {
                    if let Item::Mensagem { para, .. } = &mut m.itens[i] { *para = v; }
                });
                let mudar_seta = com(s, on, move |m: &mut Sequencia, e: Event| {
                    let v = valor_do_select(e);
                    if let (Item::Mensagem { seta, .. }, Some(nova)) = (&mut m.itens[i], Seta::TODAS.into_iter().find(|s| s.token() == v)) {
                        *seta = nova;
                    }
                });
                let mudar_texto = com(s, on, move |m: &mut Sequencia, e: InputEvent| {
                    if let Item::Mensagem { texto, .. } = &mut m.itens[i] { *texto = texto_do_input(e).trim().to_string(); }
                });
                html! {
                    <>
                        { select_de_participante(s, de, nav("de"), mudar_de) }
                        <select class="construtor-mermaid__campo" aria-label="Seta" onchange={mudar_seta}
                            data-nav-item={nav("seta")} data-nav-parent="construtor-seq-itens">
                            { for Seta::TODAS.iter().map(|t| html! {
                                <option value={t.token()} selected={t == seta}>{ nome_da_seta(*t) }</option>
                            }) }
                        </select>
                        { select_de_participante(s, para, nav("para"), mudar_para) }
                        <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Texto da mensagem"
                            value={texto.clone()} oninput={mudar_texto}
                            data-nav-item={nav("texto")} data-nav-parent="construtor-seq-itens" />
                    </>
                }
            }
            Item::Nota { lado, alvos, texto } => {
                let mudar_lado = com(s, on, move |m: &mut Sequencia, e: Event| {
                    let v = valor_do_select(e);
                    if let Item::Nota { lado, alvos, .. } = &mut m.itens[i] {
                        if let Some(novo) = LADOS.into_iter().find(|l| nome_do_lado(*l) == v) {
                            *lado = novo;
                            // Dois alvos só existem em "sobre".
                            if novo != Lado::Sobre {
                                alvos.truncate(1);
                            }
                        }
                    }
                });
                let mudar_alvo = com(s, on, move |m: &mut Sequencia, v: String| {
                    if let Item::Nota { alvos, .. } = &mut m.itens[i] { alvos[0] = v; }
                });
                let mudar_segundo = com(s, on, move |m: &mut Sequencia, e: Event| {
                    let v = valor_do_select(e);
                    if let Item::Nota { alvos, .. } = &mut m.itens[i] {
                        alvos.truncate(1);
                        if !v.is_empty() { alvos.push(v); }
                    }
                });
                let mudar_texto = com(s, on, move |m: &mut Sequencia, e: InputEvent| {
                    if let Item::Nota { texto, .. } = &mut m.itens[i] { *texto = texto_do_input(e).trim().to_string(); }
                });
                let segundo = alvos.get(1).cloned().unwrap_or_default();
                html! {
                    <>
                        <span class="construtor-mermaid__rotulo">{ "Nota" }</span>
                        <select class="construtor-mermaid__campo" aria-label="Onde fica a nota" onchange={mudar_lado}
                            data-nav-item={nav("lado")} data-nav-parent="construtor-seq-itens">
                            { for LADOS.iter().map(|l| html! {
                                <option value={nome_do_lado(*l)} selected={l == lado}>{ nome_do_lado(*l) }</option>
                            }) }
                        </select>
                        { select_de_participante(s, &alvos[0], nav("alvo"), mudar_alvo) }
                        if *lado == Lado::Sobre {
                            <select class="construtor-mermaid__campo" aria-label="Até qual participante" onchange={mudar_segundo}
                                data-nav-item={nav("alvo2")} data-nav-parent="construtor-seq-itens">
                                <option value="" selected={segundo.is_empty()}>{ "só ele" }</option>
                                { for s.participantes.iter().map(|p| html! {
                                    <option value={p.id.clone()} selected={p.id == segundo}>{ format!("até {}", p.nome) }</option>
                                }) }
                            </select>
                        }
                        <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Texto da nota"
                            value={texto.clone()} oninput={mudar_texto}
                            data-nav-item={nav("texto")} data-nav-parent="construtor-seq-itens" />
                    </>
                }
            }
        };
        html! {
            <li class="construtor-mermaid__linha">
                { corpo }
                { botoes("construtor-seq-itens", i, n_itens, None, acao) }
            </li>
        }
    });

    let novo_participante = com(s, on, |m: &mut Sequencia, _: MouseEvent| {
        let id = proximo_id(m);
        m.participantes.push(Participante { nome: id.clone(), id, ator: false });
    });
    let nova_mensagem = com(s, on, |m: &mut Sequencia, _: MouseEvent| {
        let de = m.participantes[0].id.clone();
        let para = m.participantes.get(1).unwrap_or(&m.participantes[0]).id.clone();
        m.itens.push(Item::Mensagem { de, para, seta: Seta::Sincrona, texto: "mensagem".into() });
    });
    let nova_nota = com(s, on, |m: &mut Sequencia, _: MouseEvent| {
        let alvo = m.participantes[0].id.clone();
        m.itens.push(Item::Nota { lado: Lado::Direita, alvos: vec![alvo], texto: "nota".into() });
    });
    let sem_participante = n_part == 0;

    html! {
        <div class="construtor-mermaid__form">
            <section class="construtor-mermaid__secao">
                <h4 class="construtor-mermaid__titulo">{ "Participantes" }</h4>
                <ul class="construtor-mermaid__lista" data-nav-group="construtor-seq-part">{ for participantes }</ul>
                <button type="button" class="btn btn--ghost btn--sm" data-nav-item="construtor-seq-novo-part" onclick={novo_participante}>
                    <Icon name="plus" />{ " Participante" }
                </button>
            </section>
            <section class="construtor-mermaid__secao">
                <h4 class="construtor-mermaid__titulo">{ "Mensagens e notas, em ordem" }</h4>
                <ul class="construtor-mermaid__lista" data-nav-group="construtor-seq-itens">{ for itens }</ul>
                <div class="construtor-mermaid__adicionar">
                    <button type="button" class="btn btn--ghost btn--sm" disabled={sem_participante}
                        title={sem_participante.then_some("Primeiro um participante")}
                        data-nav-item="construtor-seq-nova-msg" onclick={nova_mensagem}>
                        <Icon name="plus" />{ " Mensagem" }
                    </button>
                    <button type="button" class="btn btn--ghost btn--sm" disabled={sem_participante}
                        title={sem_participante.then_some("Primeiro um participante")}
                        data-nav-item="construtor-seq-nova-nota" onclick={nova_nota}>
                        <Icon name="plus" />{ " Nota" }
                    </button>
                </div>
            </section>
        </div>
    }
}
