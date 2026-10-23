//! A aba Sprints da tela de projeto (ciclo 472).
//!
//! Criar, iniciar e encerrar — e o histórico das encerradas, que é onde
//! as tarefas feitas vão parar em vez de lotar o quadro (RF5 da spec).
//! Tudo grava pela PESSOA: a regra (uma ativa, pra onde vão as
//! pendentes) é do `core::sprint`, e o erro que ela dá aparece aqui.

use anotadinho_core::projeto::{EtapaTarefa, Tarefa};
use anotadinho_core::sprint::{self, DestinoDasPendentes, EstadoSprint, Sprint};
use yew::prelude::*;

use crate::api;

#[derive(Properties, PartialEq, Clone)]
pub struct SprintsDoProjetoProps {
    pub vault_path: String,
    pub slug: String,
    pub sprints: Vec<Sprint>,
    pub tarefas: Vec<Tarefa>,
    /// Algo mudou: a tela de projeto relê o estado.
    pub on_mudou: Callback<()>,
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
}

fn valor_do_input(e: &InputEvent) -> String {
    let alvo: web_sys::HtmlInputElement = e.target_unchecked_into();
    alvo.value()
}

#[function_component(SprintsDoProjeto)]
pub fn sprints_do_projeto(props: &SprintsDoProjetoProps) -> Html {
    let nome = use_state(String::new);
    let objetivo = use_state(String::new);
    let inicio = use_state(String::new);
    let fim = use_state(String::new);
    let erro = use_state(|| None::<String>);
    // A sprint cujo encerramento está sendo decidido (o painel da pergunta).
    let encerrando = use_state(|| None::<String>);

    let acao = {
        let (erro, on_mudou) = (erro.clone(), props.on_mudou.clone());
        move |r: Result<(), String>| match r {
            Ok(()) => {
                erro.set(None);
                on_mudou.emit(());
            }
            Err(e) => erro.set(Some(e)),
        }
    };

    let criar = {
        let (vault_path, slug) = (props.vault_path.clone(), props.slug.clone());
        let (nome, objetivo, inicio, fim, erro) = (nome.clone(), objetivo.clone(), inicio.clone(), fim.clone(), erro.clone());
        let acao = acao.clone();
        Callback::from(move |_: MouseEvent| {
            if nome.trim().is_empty() {
                erro.set(Some("a sprint precisa de um nome".into()));
                return;
            }
            let (vault_path, slug, acao) = (vault_path.clone(), slug.clone(), acao.clone());
            let (n, o) = ((*nome).clone(), (*objetivo).clone());
            let opcional = |s: &UseStateHandle<String>| Some((**s).clone()).filter(|v| !v.trim().is_empty());
            let (i, f) = (opcional(&inicio), opcional(&fim));
            let limpar = (nome.clone(), objetivo.clone(), inicio.clone(), fim.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let hoje = crate::state::agora_legivel();
                let hoje = hoje.split(' ').next().unwrap_or("").to_string();
                let r = api::criar_sprint(&vault_path, &slug, &n, &o, i.as_deref(), f.as_deref(), &hoje).await.map(|_| ());
                if r.is_ok() {
                    limpar.0.set(String::new());
                    limpar.1.set(String::new());
                    limpar.2.set(String::new());
                    limpar.3.set(String::new());
                }
                acao(r);
            });
        })
    };

    let iniciar = {
        let vault_path = props.vault_path.clone();
        let acao = acao.clone();
        Callback::from(move |caminho: String| {
            let (vault_path, acao) = (vault_path.clone(), acao.clone());
            wasm_bindgen_futures::spawn_local(async move {
                acao(api::iniciar_sprint(&vault_path, &caminho).await);
            });
        })
    };

    let encerrar = {
        let (vault_path, encerrando) = (props.vault_path.clone(), encerrando.clone());
        let acao = acao.clone();
        Callback::from(move |(caminho, destino): (String, DestinoDasPendentes)| {
            let (vault_path, acao, encerrando) = (vault_path.clone(), acao.clone(), encerrando.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let r = api::encerrar_sprint(&vault_path, &caminho, destino).await.map(|_| ());
                if r.is_ok() {
                    encerrando.set(None);
                }
                acao(r);
            });
        })
    };

    let mut abertas: Vec<&Sprint> = props.sprints.iter().filter(|s| s.estado != EstadoSprint::Encerrada).collect();
    // A ativa primeiro, depois as planejadas na ordem em que serão feitas.
    abertas.sort_by_key(|s| (s.estado != EstadoSprint::Ativa, s.numero));
    let mut encerradas: Vec<&Sprint> = props.sprints.iter().filter(|s| s.estado == EstadoSprint::Encerrada).collect();
    encerradas.sort_by_key(|s| std::cmp::Reverse(s.numero));
    let proxima = props
        .sprints
        .iter()
        .filter(|s| s.estado == EstadoSprint::Planejada)
        .min_by_key(|s| s.numero)
        .cloned();

    let datas = |s: &Sprint| match (&s.inicio, &s.fim) {
        (Some(i), Some(f)) => format!("{i} → {f}"),
        (Some(i), None) => format!("desde {i}"),
        (None, Some(f)) => format!("até {f}"),
        (None, None) => String::new(),
    };

    let linha = |s: &Sprint| {
        let (feitas, total) = sprint::resumo(s, &props.tarefas);
        let caminho = s.caminho.clone();
        let botao = match s.estado {
            EstadoSprint::Planejada => {
                let iniciar = iniciar.clone();
                html! {
                    <button class="btn btn--sm sprints__iniciar" data-nav-item="true" data-nav-parent="sprints"
                        onclick={Callback::from(move |_: MouseEvent| iniciar.emit(caminho.clone()))}>
                        { "Iniciar" }
                    </button>
                }
            }
            EstadoSprint::Ativa => {
                let encerrando = encerrando.clone();
                html! {
                    <button class="btn btn--sm sprints__encerrar" data-nav-item="true" data-nav-parent="sprints"
                        onclick={Callback::from(move |_: MouseEvent| encerrando.set(Some(caminho.clone())))}>
                        { "Encerrar" }
                    </button>
                }
            }
            EstadoSprint::Encerrada => html! {},
        };
        let pergunta = (encerrando.as_deref() == Some(s.caminho.as_str())).then(|| {
            let pendentes = total - feitas;
            let (e1, e2, c1, c2) = (encerrar.clone(), encerrar.clone(), s.caminho.clone(), s.caminho.clone());
            let cancelar = {
                let encerrando = encerrando.clone();
                Callback::from(move |_: MouseEvent| encerrando.set(None))
            };
            html! {
                <div class="sprints__pergunta" data-nav-group="sprints-pergunta">
                    <p class="sprints__pergunta-texto">
                        { format!("{pendentes} tarefa(s) não feita(s). Pra onde vão?") }
                    </p>
                    <button class="btn btn--sm sprints__pendentes-proxima" data-nav-item="true" data-nav-parent="sprints-pergunta"
                        disabled={proxima.is_none()}
                        title={if proxima.is_none() { "Não há sprint planejada: crie uma, ou mande pro backlog" } else { "" }}
                        onclick={Callback::from(move |_: MouseEvent| e1.emit((c1.clone(), DestinoDasPendentes::ProximaSprint)))}>
                        { match &proxima { Some(p) => format!("→ próxima sprint ({})", p.nome), None => "→ próxima sprint".to_string() } }
                    </button>
                    <button class="btn btn--sm sprints__pendentes-backlog" data-nav-item="true" data-nav-parent="sprints-pergunta"
                        onclick={Callback::from(move |_: MouseEvent| e2.emit((c2.clone(), DestinoDasPendentes::Backlog)))}>
                        { "→ backlog" }
                    </button>
                    <button class="btn btn--ghost btn--sm" data-nav-item="true" data-nav-parent="sprints-pergunta" onclick={cancelar}>
                        { "Cancelar" }
                    </button>
                </div>
            }
        });
        html! {
            <li class={classes!("sprints__item", format!("sprints__item--{}", s.estado.slug()))} data-sprint={s.caminho.clone()}>
                <div class="sprints__cabeca">
                    <span class="sprints__numero">{ format!("{:02}", s.numero) }</span>
                    <span class="sprints__nome">{ &s.nome }</span>
                    <span class="sprints__estado">{ s.estado.label() }</span>
                    <span class="sprints__datas">{ datas(s) }</span>
                    <span class="sprints__resumo">{ format!("{feitas}/{total} feitas") }</span>
                    { botao }
                </div>
                if !s.objetivo.is_empty() {
                    <p class="sprints__objetivo">{ &s.objetivo }</p>
                }
                { pergunta.unwrap_or_default() }
            </li>
        }
    };

    html! {
        <section class="sprints" data-nav-group="sprints">
            <div class="sprints__nova" data-nav-group="sprints-nova">
                <input class="input input--sm sprints__nova-nome" type="text" placeholder="Nome da sprint" data-nav-item="true"
                    data-nav-parent="sprints-nova" value={(*nome).clone()}
                    oninput={{ let n = nome.clone(); Callback::from(move |e: InputEvent| n.set(valor_do_input(&e))) }} />
                <input class="input input--sm sprints__nova-objetivo" type="text" placeholder="Objetivo" data-nav-item="true"
                    data-nav-parent="sprints-nova" value={(*objetivo).clone()}
                    oninput={{ let o = objetivo.clone(); Callback::from(move |e: InputEvent| o.set(valor_do_input(&e))) }} />
                <input class="input input--sm sprints__nova-inicio" type="date" title="Início" data-nav-item="true"
                    data-nav-parent="sprints-nova" value={(*inicio).clone()}
                    oninput={{ let i = inicio.clone(); Callback::from(move |e: InputEvent| i.set(valor_do_input(&e))) }} />
                <input class="input input--sm sprints__nova-fim" type="date" title="Fim" data-nav-item="true"
                    data-nav-parent="sprints-nova" value={(*fim).clone()}
                    oninput={{ let f = fim.clone(); Callback::from(move |e: InputEvent| f.set(valor_do_input(&e))) }} />
                <button class="btn btn--sm sprints__nova-criar" data-nav-item="true" data-nav-parent="sprints-nova" onclick={criar}>
                    { "Criar sprint" }
                </button>
            </div>
            if let Some(e) = (*erro).clone() {
                <p class="sprints__erro">{ e }</p>
            }
            <ul class="sprints__lista">
                if abertas.is_empty() {
                    <li class="sprints__vazio">{ "Nenhuma sprint aberta. Sem sprint ativa, o quadro mostra todas as tarefas." }</li>
                }
                { for abertas.into_iter().map(linha) }
            </ul>
            if !encerradas.is_empty() {
                <h3 class="sprints__titulo-historico">{ "Histórico" }</h3>
                <ul class="sprints__historico">
                    { for encerradas.into_iter().map(|s| {
                        let (feitas, total) = sprint::resumo(s, &props.tarefas);
                        let mut titulos: Vec<&Tarefa> = props
                            .tarefas
                            .iter()
                            .filter(|t| t.sprint == Some(s.numero) && t.etapa == EtapaTarefa::Feita)
                            .collect();
                        titulos.sort_by_key(|t| t.numero);
                        html! {
                            <li class="sprints__encerrada" data-sprint={s.caminho.clone()}>
                                <div class="sprints__cabeca">
                                    <span class="sprints__numero">{ format!("{:02}", s.numero) }</span>
                                    <span class="sprints__nome">{ &s.nome }</span>
                                    <span class="sprints__datas">{ datas(s) }</span>
                                    <span class="sprints__resumo">{ format!("{feitas}/{total} feitas") }</span>
                                </div>
                                if !s.objetivo.is_empty() {
                                    <p class="sprints__objetivo">{ &s.objetivo }</p>
                                }
                                <ul class="sprints__feitas">
                                    { for titulos.into_iter().map(|t| {
                                        let abrir = props.on_page_selected.clone();
                                        let (c, ti) = (t.caminho.clone(), t.titulo.clone());
                                        html! {
                                            <li>
                                                <button class="projeto__link" data-nav-item="true" data-nav-parent="sprints"
                                                    onclick={Callback::from(move |_: MouseEvent| abrir.emit(api::PageMeta {
                                                        path: c.clone(), title: ti.clone(), section: "pages".into() }))}>
                                                    { format!("{:03} {}", t.numero, t.titulo) }
                                                </button>
                                            </li>
                                        }
                                    }) }
                                </ul>
                            </li>
                        }
                    }) }
                </ul>
            }
        </section>
    }
}
