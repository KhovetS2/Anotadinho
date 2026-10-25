//! A tela de um projeto (ciclo 460): uma página `type: projeto`.
//!
//! Junta num lugar só o que antes era costurado à mão: o quadro das
//! tarefas por etapa, a conversa com o agente já com o contexto do
//! projeto, e o que o agente executou. A regra de quem move pra onde é
//! do núcleo (`EtapaTarefa::proximas_para`): os botões mostram só o que
//! a pessoa PODE fazer, como o embed de fluxo.

use anotadinho_core::projeto::{self, EtapaTarefa, Quem, Tarefa};
use anotadinho_core::sprint::{self, EstadoSprint, Filtro, Sprint};
use gloo_events::EventListener;
use wasm_bindgen::JsCast;
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
    Sprints,
    Conversa,
    Execucoes,
}

/// O filtro no `<select>`: a sprint pelo número, o resto por nome.
fn filtro_em_texto(f: Filtro) -> String {
    match f {
        Filtro::SprintAtiva => "ativa".into(),
        Filtro::Backlog => "backlog".into(),
        Filtro::Todas => "todas".into(),
        Filtro::Sprint(n) => format!("s{n}"),
    }
}

fn filtro_do_texto(s: &str) -> Filtro {
    match s {
        "backlog" => Filtro::Backlog,
        "todas" => Filtro::Todas,
        outro => outro.strip_prefix('s').and_then(|n| n.parse().ok()).map(Filtro::Sprint).unwrap_or(Filtro::SprintAtiva),
    }
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
    // O que o quadro mostra (ciclo 472). A sprint ativa por padrão: é o
    // que está sendo atacado agora, e o resto é recorte que se escolhe.
    let filtro = use_state(|| Filtro::SprintAtiva);
    // A tarefa cujo painel de detalhes está aberto (ciclo 477), pelo
    // CAMINHO: a tarefa em si vem da leitura mais recente, e o painel
    // mostra o que acabou de ser gravado.
    let detalhe = use_state(|| None::<String>);
    // As mudanças no repositório de uma tarefa (ciclo 518): caminho e título.
    let mudancas_de = use_state(|| None::<(String, String)>);
    // O painel de configuração do projeto (518), e o rascunho dele.
    let configurando = use_state(|| false);
    let rascunho = use_state(|| None::<projeto::ConfigProjeto>);
    // A tarefa sendo arrastada (ciclo 465). Por mouse, como o
    // `inline_kanban`: o arrasto nativo do HTML5 é instável no WebKitGTK.
    let arrastando = use_state(|| None::<Tarefa>);
    {
        // Soltar FORA de qualquer coluna (sidebar, cabeçalho) também
        // termina o arrasto: sem isto o estado ficava preso e a próxima
        // coluna clicada recebia a tarefa. A coluna solta primeiro (o
        // evento sobe dela até a janela), então este vem por último.
        let arrastando = arrastando.clone();
        use_effect_with((), move |_| {
            let janela = web_sys::window().expect("sem janela");
            let ouvinte = EventListener::new(&janela, "mouseup", move |_| arrastando.set(None));
            move || drop(ouvinte)
        });
    }
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

    let ver_mudancas = {
        let mudancas_de = mudancas_de.clone();
        Callback::from(move |par: (String, String)| mudancas_de.set(Some(par)))
    };
    // O rascunho só existe depois da primeira mudança: copiado na
    // abertura, ele pegava a config PADRÃO se o clique vinha antes da tela
    // terminar de carregar (visto no 518: o comando aparecia vazio).
    let abrir_config = {
        let (configurando, rascunho) = (configurando.clone(), rascunho.clone());
        Callback::from(move |_: MouseEvent| {
            configurando.set(!*configurando);
            rascunho.set(None);
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

    let mudar_sprint = {
        let (vault_path, erro, recarregar) = (props.vault_path.clone(), erro.clone(), recarregar.clone());
        Callback::from(move |(caminho, sprint): (String, Option<u32>)| {
            let (vault_path, erro, recarregar) = (vault_path.clone(), erro.clone(), recarregar.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::mudar_sprint_da_tarefa(&vault_path, &caminho, sprint).await {
                    Ok(()) => {
                        erro.set(None);
                        recarregar.emit(())
                    }
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let abrir = props.on_page_selected.clone();
    let titulo = estado.as_ref().map(|e| e.projeto.titulo.clone()).unwrap_or_else(|| props.page.title.clone());
    let tarefas: Vec<Tarefa> = estado.as_ref().map(|e| e.tarefas.clone()).unwrap_or_default();
    let sprints: Vec<Sprint> = estado.as_ref().map(|e| e.sprints.clone()).unwrap_or_default();
    let ativa = sprint::ativa(&sprints).cloned();
    // As sprints em que dá pra pôr uma tarefa: a ativa e as planejadas.
    // Encerrada é histórico.
    let abertas: Vec<Sprint> = sprints.iter().filter(|s| s.estado != EstadoSprint::Encerrada).cloned().collect();
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
                    <button class={classes!("btn", "btn--ghost", "btn--sm", "projeto__configurar", configurando.then_some("projeto__configurar--aberto"))}
                        data-nav-item="true" data-nav-parent="projeto-topo" onclick={abrir_config}
                        title="Verificação, branch por tarefa e commit ao aceitar">
                        <Icon name="settings" />{ " Configurar" }
                    </button>
                </div>
                // O que o app faz no repositório, à vista (518): sem isto,
                // "concluída" é a palavra do agente e ninguém sabia.
                <p class="projeto__regras">
                    { match config_atual.comando_de_verificacao() {
                        Some(cmd) => html! { <span class="projeto__regra projeto__regra--verificar"><Icon name="check-square" />{ " verificação " }<code>{ cmd }</code></span> },
                        None => html! { <span class="projeto__regra projeto__regra--sem">{ "sem verificação — vale o que o agente disser" }</span> },
                    } }
                    if config_atual.branch_por_tarefa {
                        <span class="projeto__regra"><Icon name="git-branch" />{ " branch por tarefa" }</span>
                    }
                    if config_atual.commitar_ao_aceitar {
                        <span class="projeto__regra"><Icon name="check" />{ " commit ao aceitar" }</span>
                    }
                </p>
                if *configurando && estado.is_some() {
                    { painel_de_config((*rascunho).clone().unwrap_or_else(|| config_atual.clone()), rascunho.clone(), configurando.clone(), gravar_config.clone()) }
                }
                <div class="projeto__sprint" data-nav-group="projeto-sprint">
                    <select class="input input--sm projeto__sprint-filtro" data-nav-item="true" data-nav-parent="projeto-sprint"
                        title="O que o quadro mostra"
                        onchange={{
                            let filtro = filtro.clone();
                            Callback::from(move |e: Event| {
                                let alvo: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                filtro.set(filtro_do_texto(&alvo.value()));
                            })
                        }}>
                        { for Filtro::opcoes(&sprints).into_iter().map(|f| html! {
                            <option value={filtro_em_texto(f)} selected={f == *filtro}>{ f.rotulo(&sprints) }</option>
                        }) }
                    </select>
                    if let Some(a) = ativa.as_ref().filter(|_| *filtro == Filtro::SprintAtiva) {
                        <span class="projeto__sprint-info">
                            if !a.objetivo.is_empty() { { &a.objetivo } }
                            if let Some(f) = &a.fim { <span class="projeto__sprint-fim">{ format!(" · até {f}") }</span> }
                        </span>
                    }
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
                { botao(Aba::Sprints, "Sprints") }
                { botao(Aba::Conversa, "Conversa") }
                { botao(Aba::Execucoes, "Execuções") }
            </nav>
        }
    };

    // Um índice pro quadro inteiro (ciclo 462): perguntar
    // `dependencias_pendentes` por cartão era `O(n²)`.
    let indice = projeto::IndiceDeTarefas::novo(&tarefas);
    // O quadro desenha o recorte; as dependências acima continuam olhando
    // o projeto INTEIRO — uma tarefa da sprint pode depender de uma do
    // backlog, e o aviso tem que aparecer.
    let do_quadro: Vec<Tarefa> = sprint::tarefas_do_quadro(&tarefas, &sprints, *filtro).into_iter().cloned().collect();
    let hoje = {
        let agora = crate::state::agora_legivel();
        agora.split(' ').next().unwrap_or("").to_string()
    };
    let abrir_detalhe = {
        let detalhe = detalhe.clone();
        Callback::from(move |c: String| detalhe.set(Some(c)))
    };
    let painel = (*detalhe).as_ref().and_then(|c| tarefas.iter().find(|t| &t.caminho == c)).map(|t| {
        let fechar = {
            let detalhe = detalhe.clone();
            Callback::from(move |_: ()| detalhe.set(None))
        };
        let abrir_pagina = {
            let (abrir, c, titulo) = (abrir.clone(), t.caminho.clone(), t.titulo.clone());
            Callback::from(move |_: ()| abrir.emit(pagina(&c, &titulo)))
        };
        html! {
            <crate::components::tarefa_detalhes::TarefaDetalhes
                vault_path={props.vault_path.clone()}
                tarefa={t.clone()}
                hoje={hoje.clone()}
                outras={tarefas.iter().filter(|o| o.caminho != t.caminho).map(|o| (o.numero, o.titulo.clone())).collect::<Vec<_>>()}
                on_mudancas={{
                    let (ver, par) = (ver_mudancas.clone(), (t.caminho.clone(), format!("{:03} {}", t.numero, t.titulo)));
                    Callback::from(move |_: ()| ver.emit(par.clone()))
                }}
                on_mudou={recarregar.clone()}
                on_fechar={fechar}
                on_abrir_pagina={abrir_pagina} />
        }
    });
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
                        let da_etapa: Vec<&Tarefa> = do_quadro.iter().filter(|t| t.etapa == *etapa).collect();
                        // Quem decide se a coluna aceita é o núcleo, o mesmo
                        // dos botões: `executando` e `revisao` nunca acendem,
                        // porque nenhuma etapa leva a pessoa até elas.
                        let aceita = arrastando
                            .as_ref()
                            .map(|t| t.etapa != *etapa && t.etapa.pode_ir_para(*etapa, Quem::Humano));
                        let classe = classes!(
                            "projeto__coluna",
                            (aceita == Some(true)).then_some("projeto__coluna--destino"),
                            (aceita == Some(false)
                                && arrastando.as_ref().is_some_and(|t| t.etapa != *etapa))
                            .then_some("projeto__coluna--proibida"),
                        );
                        let soltar = {
                            let (arrastando, mover, destino) = (arrastando.clone(), mover.clone(), *etapa);
                            Callback::from(move |_: MouseEvent| {
                                if let Some(t) = (*arrastando).clone() {
                                    if t.etapa != destino && t.etapa.pode_ir_para(destino, Quem::Humano) {
                                        mover.emit((t.caminho, destino));
                                    }
                                }
                                arrastando.set(None);
                            })
                        };
                        html! {
                            <div class={classe} data-etapa={etapa.slug()} onmouseup={soltar}>
                                <h3 class="projeto__coluna-titulo">
                                    { etapa.label() }
                                    <span class="projeto__contagem">{ da_etapa.len() }</span>
                                </h3>
                                { for da_etapa.into_iter().map(|t| cartao(t, &Cartao {
                                    indice: &indice, abrir: &abrir, mover: &mover, arrastando: &arrastando,
                                    sprints: &abertas, mudar_sprint: &mudar_sprint, hoje: &hoje,
                                    abrir_detalhe: &abrir_detalhe, detalhe_aberto: detalhe.as_deref(),
                                    ver_mudancas: &ver_mudancas,
                                })) }
                            </div>
                        }
                    }) }
                </div>
            </section>
        }
    };

    let corpo = match *aba {
        // O painel fica AO LADO do quadro, e não por cima: dá pra marcar
        // um critério olhando as outras colunas.
        Aba::Quadro => html! {
            <div class={classes!("projeto__com-painel", painel.is_some().then_some("projeto__com-painel--aberto"))}>
                { quadro }
                { painel.unwrap_or_default() }
            </div>
        },
        Aba::Sprints => html! {
            <crate::components::sprints_do_projeto::SprintsDoProjeto
                vault_path={props.vault_path.clone()}
                slug={slug.clone()}
                sprints={sprints.clone()}
                tarefas={tarefas.clone()}
                on_mudou={recarregar.clone()}
                on_page_selected={props.on_page_selected.clone()} />
        },
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
            if let Some((caminho, titulo)) = (*mudancas_de).clone() {
                <crate::components::mudancas_da_tarefa::MudancasDaTarefa
                    vault_path={props.vault_path.clone()} caminho={caminho} titulo={titulo}
                    on_fechar={{ let m = mudancas_de.clone(); Callback::from(move |_| m.set(None)) }} />
            }
        </main>
    }
}

/// O painel "Configurar" do projeto (ciclo 518): o que o app faz no
/// repositório. Um rascunho, gravado só no botão — mexer num campo não
/// pode disparar uma gravação por letra no `projetos.json`.
fn painel_de_config(
    r: projeto::ConfigProjeto,
    rascunho: UseStateHandle<Option<projeto::ConfigProjeto>>,
    configurando: UseStateHandle<bool>,
    gravar: Callback<projeto::ConfigProjeto>,
) -> Html {
    let mudar = |f: fn(&mut projeto::ConfigProjeto, String)| {
        let (rascunho, r) = (rascunho.clone(), r.clone());
        Callback::from(move |e: Event| {
            let alvo: web_sys::HtmlInputElement = e.target_unchecked_into();
            let valor = if alvo.type_() == "checkbox" { alvo.checked().to_string() } else { alvo.value() };
            let mut novo = r.clone();
            f(&mut novo, valor);
            rascunho.set(Some(novo));
        })
    };
    let salvar = {
        let (r, gravar, configurando) = (r.clone(), gravar.clone(), configurando.clone());
        Callback::from(move |_: MouseEvent| {
            gravar.emit(r.clone());
            configurando.set(false);
        })
    };
    let cancelar = {
        let configurando = configurando.clone();
        Callback::from(move |_: MouseEvent| configurando.set(false))
    };
    html! {
        <section class="projeto__config-painel" data-nav-group="projeto-config">
            <label class="projeto__campo">
                <span class="projeto__campo-rotulo">{ "Comando de verificação" }</span>
                <input class="input input--sm projeto__verificar" type="text" placeholder="npm test, cargo test… (vazio: sem verificação)"
                    data-nav-item="true" data-nav-parent="projeto-config" value={r.verificar.clone().unwrap_or_default()}
                    onchange={mudar(|c, v| c.verificar = Some(v).filter(|v| !v.trim().is_empty()))} />
                <span class="projeto__campo-dica">{ "Roda no repositório depois do agente; se falhar, a tarefa vai pra bloqueada com a saída." }</span>
            </label>
            <label class="projeto__campo">
                <span class="projeto__campo-rotulo">{ "Prazo da verificação (minutos)" }</span>
                <input class="input input--sm projeto__prazo-verificacao" type="number" min="1"
                    data-nav-item="true" data-nav-parent="projeto-config" value={(r.prazo_da_verificacao / 60).max(1).to_string()}
                    onchange={mudar(|c, v| if let Ok(m) = v.trim().parse::<u64>() { c.prazo_da_verificacao = m.max(1) * 60 })} />
            </label>
            <label class="projeto__marca">
                <input type="checkbox" class="projeto__branch" checked={r.branch_por_tarefa} data-nav-item="true" data-nav-parent="projeto-config"
                    onchange={mudar(|c, v| c.branch_por_tarefa = v == "true")} />
                { " Cada tarefa no branch dela (tarefa/003-…)" }
            </label>
            <label class="projeto__marca">
                <input type="checkbox" class="projeto__commitar" checked={r.commitar_ao_aceitar} data-nav-item="true" data-nav-parent="projeto-config"
                    onchange={mudar(|c, v| c.commitar_ao_aceitar = v == "true")} />
                { " Aceitar (revisão → feita) commita o que a tarefa deixou" }
            </label>
            <div class="projeto__config-acoes">
                <button class="btn btn--ghost btn--sm" data-nav-item="true" data-nav-parent="projeto-config" onclick={cancelar}>{ "Cancelar" }</button>
                <button class="btn btn--primary btn--sm projeto__config-gravar" data-nav-item="true" data-nav-parent="projeto-config" onclick={salvar}>{ "Gravar" }</button>
            </div>
        </section>
    }
}

/// Um cartão do quadro: abre a tarefa, e mostra só as transições que a
/// pessoa pode fazer daqui.
/// O que o cartão precisa da tela, num lugar só: a lista de argumentos
/// já tinha passado de sete.
struct Cartao<'a> {
    indice: &'a projeto::IndiceDeTarefas,
    abrir: &'a Callback<api::PageMeta>,
    mover: &'a Callback<(String, EtapaTarefa)>,
    arrastando: &'a UseStateHandle<Option<Tarefa>>,
    sprints: &'a [Sprint],
    mudar_sprint: &'a Callback<(String, Option<u32>)>,
    hoje: &'a str,
    abrir_detalhe: &'a Callback<String>,
    detalhe_aberto: Option<&'a str>,
    ver_mudancas: &'a Callback<(String, String)>,
}

fn cartao(t: &Tarefa, c: &Cartao) -> Html {
    let Cartao { indice, abrir, mover, arrastando, sprints, mudar_sprint, .. } = *c;
    let pegar = {
        let (arrastando, t) = (arrastando.clone(), t.clone());
        Callback::from(move |e: MouseEvent| {
            if e.button() != 0 {
                return;
            }
            // Nos botões de mover, o gesto é o clique: começar um arrasto
            // ali faria um clique trêmulo virar soltura na coluna vizinha.
            let num_botao = e
                .target()
                .and_then(|alvo| alvo.dyn_into::<web_sys::Element>().ok())
                // O seletor de sprint também é gesto de clique (ciclo 472).
                .and_then(|el| el.closest(".projeto__mover, .projeto__cartao-sprint, .projeto__detalhes-abrir, .projeto__mudancas-abrir").ok().flatten())
                .is_some();
            if num_botao {
                return;
            }
            // Sem isto, segurar e mover seleciona o texto embaixo e o
            // navegador começa o arrasto NATIVO de conteúdo por cima do
            // nosso (a mesma "sombra" que o kanban teve).
            e.prevent_default();
            arrastando.set(Some(t.clone()));
        })
    };
    let sendo_arrastado = arrastando.as_ref().is_some_and(|a| a.caminho == t.caminho);
    let pendentes = indice.pendentes(t);
    let abrir_tarefa = {
        let abrir = abrir.clone();
        let (c, titulo) = (t.caminho.clone(), t.titulo.clone());
        Callback::from(move |_: MouseEvent| abrir.emit(pagina(&c, &titulo)))
    };
    let (feitos, total) = t.progresso();
    let vencida = t.vencida(c.hoje);
    // A prioridade só aparece quando foge da média (ciclo 513): o selo em
    // toda tarefa diria "média" mil vezes e esconderia as que importam.
    let prioridade = t.prioridade.as_deref().map(str::to_lowercase).filter(|p| p == "alta" || p == "baixa");
    let tem_selos = total > 0 || t.prazo.is_some() || !t.comentarios.is_empty() || !t.anexos.is_empty() || prioridade.is_some() || t.verificacao.is_some();
    let detalhar = {
        let (abrir_detalhe, caminho) = (c.abrir_detalhe.clone(), t.caminho.clone());
        Callback::from(move |_: MouseEvent| abrir_detalhe.emit(caminho.clone()))
    };
    let aberto = c.detalhe_aberto == Some(t.caminho.as_str());
    html! {
        <article class={classes!(
            "projeto__cartao",
            sendo_arrastado.then_some("projeto__cartao--arrastando"),
            aberto.then_some("projeto__cartao--detalhado"),
        )}
            data-tarefa={t.caminho.clone()} onmousedown={pegar}>
            <div class="projeto__cartao-cabeca">
                <button class="projeto__cartao-titulo" data-nav-item="true" data-nav-parent="projeto-quadro"
                    onclick={abrir_tarefa}>
                    <span class="projeto__numero">{ format!("{:03}", t.numero) }</span>
                    { " " }{ &t.titulo }
                </button>
                // O que a tarefa mudou no código (518): só depois de ela ter
                // rodado pelo app, que é quando há de onde comparar.
                if t.base_git.is_some() {
                    <button class="btn btn--ghost btn--xs projeto__mudancas-abrir" data-nav-item="true" data-nav-parent="projeto-quadro"
                        title="O que esta tarefa mudou no repositório"
                        onclick={{
                            let (ver, par) = (c.ver_mudancas.clone(), (t.caminho.clone(), format!("{:03} {}", t.numero, t.titulo)));
                            Callback::from(move |_: MouseEvent| ver.emit(par.clone()))
                        }}>
                        <Icon name="git-branch" />
                    </button>
                }
                <button class="btn btn--ghost btn--xs projeto__detalhes-abrir" data-nav-item="true" data-nav-parent="projeto-quadro"
                    title="Detalhes: tags, prazo, critérios, comentários e anexos" onclick={detalhar}>
                    <Icon name="more-horizontal" />
                </button>
            </div>
            // Os selos do card do kanban (ciclo 477): o que só se via
            // abrindo a página.
            if tem_selos {
                <div class="projeto__selos">
                    // O resultado da verificação do projeto (517).
                    if let Some(v) = &t.verificacao {
                        <span class={classes!("projeto__selo", "projeto__selo--verificacao", if v == "passou" { "projeto__selo--passou" } else { "projeto__selo--falhou" })}
                            title={if v == "passou" { "A verificação do projeto passou" } else { "A verificação do projeto falhou: veja a seção Verificação da tarefa" }}>
                            <Icon name={if v == "passou" { "check" } else { "x" }} />{ " teste" }
                        </span>
                    }
                    if let Some(p) = &prioridade {
                        <span class={classes!("projeto__selo", "projeto__selo--prioridade", format!("projeto__selo--{p}"))}
                            title={format!("Prioridade {p}")}>
                            <Icon name={if p == "alta" { "arrow-up" } else { "arrow-down" }} />{ format!(" {p}") }
                        </span>
                    }
                    if total > 0 {
                        <span class={classes!("projeto__selo", "projeto__selo--checklist", (feitos == total).then_some("projeto__selo--completo"))}>
                            <Icon name="check-square" />{ format!(" {feitos}/{total}") }
                        </span>
                    }
                    if let Some(p) = &t.prazo {
                        <span class={classes!("projeto__selo", "projeto__selo--prazo", vencida.then_some("projeto__selo--vencido"))}
                            title={if vencida { "Prazo vencido" } else { "Prazo" }}>
                            <Icon name="calendar" />{ format!(" {p}") }
                        </span>
                    }
                    if !t.comentarios.is_empty() {
                        <span class="projeto__selo projeto__selo--comentarios"><Icon name="message-circle" />{ format!(" {}", t.comentarios.len()) }</span>
                    }
                    if !t.anexos.is_empty() {
                        <span class="projeto__selo projeto__selo--anexos"><Icon name="paperclip" />{ format!(" {}", t.anexos.len()) }</span>
                    }
                </div>
            }
            if !t.tags.is_empty() {
                <div class="projeto__tags">
                    { for t.tags.iter().map(|tag| html! { <span class="badge badge--info projeto__tag">{ tag }</span> }) }
                </div>
            }
            if !t.depende.is_empty() {
                <p class={classes!("projeto__depende", (!pendentes.is_empty()).then_some("projeto__depende--pendente"))}>
                    { format!("depende de {}", t.depende.join(", ")) }
                </p>
            }
            // A sprint da tarefa (ciclo 472). Só aparece com sprint aberta
            // no projeto: sem nenhuma, o seletor seria uma opção só.
            if !sprints.is_empty() {
                <select class="input input--xs projeto__cartao-sprint" data-nav-item="true" data-nav-parent="projeto-quadro"
                    title="A sprint desta tarefa"
                    onchange={{
                        let (mudar_sprint, c) = (mudar_sprint.clone(), t.caminho.clone());
                        Callback::from(move |e: Event| {
                            let alvo: web_sys::HtmlSelectElement = e.target_unchecked_into();
                            mudar_sprint.emit((c.clone(), alvo.value().parse().ok()));
                        })
                    }}>
                    <option value="backlog" selected={t.sprint.is_none()}>{ "Backlog" }</option>
                    { for sprints.iter().map(|s| html! {
                        <option value={s.numero.to_string()} selected={t.sprint == Some(s.numero)}>
                            { format!("{} · {}", s.nome, s.estado.label().to_lowercase()) }
                        </option>
                    }) }
                </select>
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
