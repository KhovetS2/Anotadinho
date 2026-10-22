//! A tela de um projeto (ciclo 460): uma página `type: projeto`.
//!
//! Junta num lugar só o que antes era costurado à mão: o quadro das
//! tarefas por etapa, a conversa com o agente já com o contexto do
//! projeto, e o que o agente executou. A regra de quem move pra onde é
//! do núcleo (`EtapaTarefa::proximas_para`): os botões mostram só o que
//! a pessoa PODE fazer, como o embed de fluxo.

use anotadinho_core::projeto::{self, EtapaTarefa, Quem, Tarefa};
use yew::prelude::*;

use crate::api;
use crate::components::icon::Icon;

#[derive(Properties, PartialEq, Clone)]
pub struct ProjetoViewProps {
    pub vault_path: String,
    pub page: api::PageMeta,
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
    #[prop_or_default]
    pub open_dialog: Callback<crate::dialog::PendingDialog>,
    /// Repassados à conversa do projeto, pra "Detalhar com o agente"
    /// chegar com a pergunta escrita.
    #[prop_or_default]
    pub pergunta_inicial: Option<crate::components::conversa_view::PerguntaInicial>,
    #[prop_or_default]
    pub on_pergunta_consumida: Callback<()>,
}

#[derive(Clone, Copy, PartialEq)]
enum Aba {
    Quadro,
    Conversa,
    Execucoes,
}

fn pagina(path: &str, title: &str) -> api::PageMeta {
    api::PageMeta { path: path.to_string(), title: title.to_string(), section: "pages".to_string() }
}

#[function_component(ProjetoView)]
pub fn projeto_view(props: &ProjetoViewProps) -> Html {
    let slug = projeto::projeto_do_caminho(&props.page.path).unwrap_or_default().to_string();
    let estado = use_state(|| None::<api::EstadoDoProjeto>);
    // Dois erros, e não um: o recarregamento de 3 s limpava o da LEITURA
    // a cada volta boa, e junto ia embora o de um clique que falhou —
    // o botão "não fazia nada" e a mensagem sumia antes de alguém ler
    // (achado na tela, ciclo 460). O da ação só sai na próxima ação.
    let erro_leitura = use_state(|| None::<String>);
    let erro = use_state(|| None::<String>);
    let execucoes = use_state(Vec::<anotadinho_core::execucao::Execucao>::new);
    let nova = use_state(String::new);
    // Uma pergunta pra conversa geral faz a tela abrir NELA: quem apertou
    // "Detalhar" na tarefa quer ver o campo preenchido, não o quadro.
    let aba_inicial = if props
        .pergunta_inicial
        .as_ref()
        .is_some_and(|p| p.conversa == projeto::caminho_da_conversa_geral(&slug))
    {
        Aba::Conversa
    } else {
        Aba::Quadro
    };
    let aba = use_state(move || aba_inicial);
    {
        // A pergunta pode chegar com a tela JÁ aberta (a tarefa estava
        // noutra aba); o estado inicial acima só vale na montagem.
        let aba = aba.clone();
        use_effect_with(aba_inicial, move |a| {
            if *a == Aba::Conversa {
                aba.set(Aba::Conversa);
            }
            || ()
        });
    }
    let recarga = use_state(|| 0u32);
    // O último estado lido, pra o intervalo comparar. `use_mut_ref`, e
    // não o `use_state` acima: capturado na closure do intervalo, o
    // handle fica CONGELADO no valor da montagem (ciclos 155, 157, 201,
    // 213), e "voltou a ser igual ao de antes" não re-renderizaria.
    let visto = use_mut_ref(|| None::<api::EstadoDoProjeto>);
    let vistas = use_mut_ref(Vec::<anotadinho_core::execucao::Execucao>::new);

    // Lê o estado ao abrir, a cada mudança feita daqui, e de 3 em 3 s: o
    // agente move tarefas sozinho (executando → revisão), e um quadro que
    // só atualizasse no clique mentiria sobre o que está rodando.
    {
        let (estado, erro, execucoes) = (estado.clone(), erro_leitura.clone(), execucoes.clone());
        let (visto, vistas) = (visto.clone(), vistas.clone());
        let (vault_path, slug) = (props.vault_path.clone(), slug.clone());
        use_effect_with((props.page.path.clone(), *recarga), move |_| {
            let carregar = {
                let (estado, erro, execucoes) = (estado.clone(), erro.clone(), execucoes.clone());
                let (visto, vistas) = (visto.clone(), vistas.clone());
                let (vault_path, slug) = (vault_path.clone(), slug.clone());
                move || {
                    let (estado, erro, execucoes) = (estado.clone(), erro.clone(), execucoes.clone());
                    let (visto, vistas) = (visto.clone(), vistas.clone());
                    let (vault_path, slug) = (vault_path.clone(), slug.clone());
                    wasm_bindgen_futures::spawn_local(async move {
                        match api::estado_do_projeto(&vault_path, &slug).await {
                            Ok(e) => {
                                // Só re-renderiza se mudou: o intervalo roda
                                // com a pessoa digitando o título da tarefa.
                                if visto.borrow().as_ref() != Some(&e) {
                                    *visto.borrow_mut() = Some(e.clone());
                                    estado.set(Some(e));
                                }
                                erro.set(None);
                            }
                            Err(e) => erro.set(Some(e)),
                        }
                        let pasta = format!("{}/", projeto::pasta_do_projeto(&slug));
                        if let Ok(lista) = api::listar_execucoes(&vault_path).await {
                            let do_projeto: Vec<_> =
                                lista.into_iter().filter(|x| x.conversa.starts_with(&pasta)).collect();
                            if *vistas.borrow() != do_projeto {
                                *vistas.borrow_mut() = do_projeto.clone();
                                execucoes.set(do_projeto);
                            }
                        }
                    });
                }
            };
            carregar();
            let intervalo = gloo_timers::callback::Interval::new(3_000, carregar);
            move || drop(intervalo)
        });
    }

    let recarregar = {
        let recarga = recarga.clone();
        Callback::from(move |_: ()| recarga.set(*recarga + 1))
    };

    let mover = {
        let (vault_path, erro, recarregar) = (props.vault_path.clone(), erro.clone(), recarregar.clone());
        Callback::from(move |(caminho, destino): (String, EtapaTarefa)| {
            let (vault_path, erro, recarregar) = (vault_path.clone(), erro.clone(), recarregar.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::mudar_etapa_tarefa(&vault_path, &caminho, destino).await {
                    Ok(_) => {
                        erro.set(None);
                        recarregar.emit(())
                    }
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let gravar_config = {
        let (vault_path, slug, erro, recarregar) =
            (props.vault_path.clone(), slug.clone(), erro.clone(), recarregar.clone());
        Callback::from(move |config: projeto::ConfigProjeto| {
            let (vault_path, slug, erro, recarregar) =
                (vault_path.clone(), slug.clone(), erro.clone(), recarregar.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::gravar_config_do_projeto(&vault_path, &slug, &config).await {
                    Ok(()) => {
                        erro.set(None);
                        recarregar.emit(())
                    }
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let config_atual = estado.as_ref().map(|e| e.config.clone()).unwrap_or_default();

    let escolher_repositorio = {
        let (gravar_config, config) = (gravar_config.clone(), config_atual.clone());
        let erro = erro.clone();
        Callback::from(move |_: MouseEvent| {
            let (gravar_config, mut config, erro) = (gravar_config.clone(), config.clone(), erro.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::escolher_pasta().await {
                    Ok(Some(pasta)) => {
                        config.repositorio = Some(pasta);
                        gravar_config.emit(config);
                    }
                    Ok(None) => {}
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let alternar_pausa = {
        let (gravar_config, config) = (gravar_config.clone(), config_atual.clone());
        Callback::from(move |_: MouseEvent| {
            let mut c = config.clone();
            c.pausado = !c.pausado;
            gravar_config.emit(c);
        })
    };

    let criar = {
        let (vault_path, slug, nova, erro, recarregar) =
            (props.vault_path.clone(), slug.clone(), nova.clone(), erro.clone(), recarregar.clone());
        Callback::from(move |_: ()| {
            let titulo = nova.trim().to_string();
            if titulo.is_empty() {
                return;
            }
            let (vault_path, slug, nova, erro, recarregar) =
                (vault_path.clone(), slug.clone(), nova.clone(), erro.clone(), recarregar.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let hoje = crate::state::agora_legivel();
                let hoje = hoje.split(' ').next().unwrap_or("").to_string();
                match api::criar_tarefa(&vault_path, &slug, &titulo, &hoje).await {
                    Ok(_) => {
                        erro.set(None);
                        nova.set(String::new());
                        recarregar.emit(());
                    }
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let abrir = props.on_page_selected.clone();
    let titulo = estado.as_ref().map(|e| e.projeto.titulo.clone()).unwrap_or_else(|| props.page.title.clone());
    let tarefas: Vec<Tarefa> = estado.as_ref().map(|e| e.tarefas.clone()).unwrap_or_default();
    let rodando = tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Executando).count();

    let cabecalho = {
        let arq = projeto::caminho_da_arquitetura(&slug);
        let abrir_arq = {
            let abrir = abrir.clone();
            let titulo = titulo.clone();
            Callback::from(move |_: MouseEvent| abrir.emit(pagina(&arq, &format!("Arquitetura — {titulo}"))))
        };
        html! {
            <header class="projeto__topo" data-nav-group="projeto-topo">
                <div class="projeto__titulo-linha">
                    <h2 class="projeto__titulo">{ &titulo }</h2>
                    if rodando > 0 {
                        <span class="projeto__rodando"><span class="spinner"></span>
                            { format!(" {rodando} executando") }</span>
                    }
                </div>
                <div class="projeto__config">
                    <span class="projeto__repo" title="Onde o agente trabalha nas tarefas deste projeto">
                        <Icon name="folder" />
                        { match &config_atual.repositorio {
                            Some(r) => html! { <code>{ r }</code> },
                            None => html! { <em>{ "sem repositório" }</em> },
                        } }
                    </span>
                    <button class="btn btn--ghost btn--sm projeto__escolher-repo" data-nav-item="true"
                        data-nav-parent="projeto-topo" onclick={escolher_repositorio}>
                        { "Escolher repositório" }
                    </button>
                    <button class="btn btn--ghost btn--sm projeto__pausa" data-nav-item="true"
                        data-nav-parent="projeto-topo" onclick={alternar_pausa}
                        title="Com a fila pausada, nada sai de \"executar\"">
                        if config_atual.pausado {
                            <Icon name="play" />{ " Retomar fila" }
                        } else {
                            <Icon name="pause" />{ " Pausar fila" }
                        }
                    </button>
                    <button class="btn btn--ghost btn--sm projeto__arquitetura" data-nav-item="true"
                        data-nav-parent="projeto-topo" onclick={abrir_arq}>
                        <Icon name="layout" />{ " Arquitetura" }
                    </button>
                </div>
                if let Some(e) = estado.as_ref().filter(|e| !e.avisos.is_empty()) {
                    <ul class="projeto__avisos">
                        { for e.avisos.iter().map(|a| html! {
                            <li class="projeto__aviso"><Icon name="alert-triangle" />{ " " }{ a }</li>
                        }) }
                    </ul>
                }
                if let Some(e) = (*erro).clone().or_else(|| (*erro_leitura).clone()) {
                    <p class="projeto__erro">{ e }</p>
                }
            </header>
        }
    };

    let abas = {
        let botao = |alvo: Aba, rotulo: &'static str| {
            let aba = aba.clone();
            let classe = classes!("projeto__aba", (*aba == alvo).then_some("projeto__aba--ativa"));
            html! {
                <button class={classe} data-nav-item="true" data-nav-parent="projeto-abas"
                    onclick={Callback::from(move |_: MouseEvent| aba.set(alvo))}>
                    { rotulo }
                </button>
            }
        };
        html! {
            <nav class="projeto__abas" data-nav-group="projeto-abas">
                { botao(Aba::Quadro, "Quadro") }
                { botao(Aba::Conversa, "Conversa") }
                { botao(Aba::Execucoes, "Execuções") }
            </nav>
        }
    };

    let quadro = {
        let oninput = {
            let nova = nova.clone();
            Callback::from(move |e: InputEvent| {
                let alvo: web_sys::HtmlInputElement = e.target_unchecked_into();
                nova.set(alvo.value());
            })
        };
        let onkeydown = {
            let criar = criar.clone();
            Callback::from(move |e: KeyboardEvent| {
                if e.key() == "Enter" {
                    e.prevent_default();
                    criar.emit(());
                }
            })
        };
        let clique_criar = {
            let criar = criar.clone();
            Callback::from(move |_: MouseEvent| criar.emit(()))
        };
        html! {
            <section class="projeto__quadro-area">
                <div class="projeto__nova" data-nav-group="projeto-nova">
                    <input class="input input--sm projeto__nova-titulo" type="text" data-nav-item="true"
                        data-nav-parent="projeto-nova" placeholder="Título da tarefa nova"
                        value={(*nova).clone()} {oninput} {onkeydown} />
                    <button class="btn btn--sm projeto__nova-criar" data-nav-item="true"
                        data-nav-parent="projeto-nova" onclick={clique_criar}>
                        <Icon name="plus" />{ " Criar tarefa" }
                    </button>
                </div>
                <div class="projeto__quadro" data-nav-group="projeto-quadro">
                    { for EtapaTarefa::all().iter().map(|etapa| {
                        let da_etapa: Vec<&Tarefa> = tarefas.iter().filter(|t| t.etapa == *etapa).collect();
                        html! {
                            <div class="projeto__coluna" data-etapa={etapa.slug()}>
                                <h3 class="projeto__coluna-titulo">
                                    { etapa.label() }
                                    <span class="projeto__contagem">{ da_etapa.len() }</span>
                                </h3>
                                { for da_etapa.into_iter().map(|t| cartao(t, &tarefas, &abrir, &mover)) }
                            </div>
                        }
                    }) }
                </div>
            </section>
        }
    };

    let corpo = match *aba {
        Aba::Quadro => quadro,
        Aba::Conversa => {
            let geral = projeto::caminho_da_conversa_geral(&slug);
            html! {
                <div class="projeto__conversa">
                    <crate::components::conversa_view::ConversaView
                        vault_path={props.vault_path.clone()}
                        page={pagina(&geral, &format!("Conversa — {titulo}"))}
                        on_page_selected={props.on_page_selected.clone()}
                        pergunta_inicial={props.pergunta_inicial.clone()}
                        on_pergunta_consumida={props.on_pergunta_consumida.clone()}
                        open_dialog={props.open_dialog.clone()} />
                </div>
            }
        }
        Aba::Execucoes => html! {
            <ul class="projeto__execucoes" data-nav-group="projeto-execucoes">
                if execucoes.is_empty() {
                    <li class="projeto__vazio">{ "Nenhuma tarefa executada ainda." }</li>
                }
                { for execucoes.iter().map(|x| {
                    let abrir = abrir.clone();
                    let conversa = x.conversa.clone();
                    html! {
                        <li class="projeto__execucao">
                            <span class="projeto__quando">{ &x.quando }</span>
                            <button class="projeto__link" data-nav-item="true" data-nav-parent="projeto-execucoes"
                                onclick={Callback::from(move |_: MouseEvent| abrir.emit(pagina(&conversa, &conversa)))}>
                                { &x.conversa }
                            </button>
                            <span class="projeto__fim">{ x.fim.rotulo() }</span>
                        </li>
                    }
                }) }
            </ul>
        },
    };

    html! {
        <main class="projeto">
            { cabecalho }
            { abas }
            { corpo }
        </main>
    }
}

/// Um cartão do quadro: abre a tarefa, e mostra só as transições que a
/// pessoa pode fazer daqui.
fn cartao(
    t: &Tarefa,
    todas: &[Tarefa],
    abrir: &Callback<api::PageMeta>,
    mover: &Callback<(String, EtapaTarefa)>,
) -> Html {
    let pendentes = projeto::dependencias_pendentes(t, todas);
    let abrir_tarefa = {
        let abrir = abrir.clone();
        let (c, titulo) = (t.caminho.clone(), t.titulo.clone());
        Callback::from(move |_: MouseEvent| abrir.emit(pagina(&c, &titulo)))
    };
    html! {
        <article class="projeto__cartao" data-tarefa={t.caminho.clone()}>
            <button class="projeto__cartao-titulo" data-nav-item="true" data-nav-parent="projeto-quadro"
                onclick={abrir_tarefa}>
                <span class="projeto__numero">{ format!("{:03}", t.numero) }</span>
                { " " }{ &t.titulo }
            </button>
            if !t.depende.is_empty() {
                <p class={classes!("projeto__depende", (!pendentes.is_empty()).then_some("projeto__depende--pendente"))}>
                    { format!("depende de {}", t.depende.join(", ")) }
                </p>
            }
            <div class="projeto__cartao-acoes">
                { for t.etapa.proximas_para(Quem::Humano).into_iter().map(|destino| {
                    let mover = mover.clone();
                    let c = t.caminho.clone();
                    html! {
                        <button class="btn btn--ghost btn--xs projeto__mover" data-destino={destino.slug()}
                            data-nav-item="true" data-nav-parent="projeto-quadro"
                            onclick={Callback::from(move |_: MouseEvent| mover.emit((c.clone(), destino)))}>
                            { format!("→ {}", destino.label()) }
                        </button>
                    }
                }) }
            </div>
        </article>
    }
}
