//! O painel de detalhes de uma tarefa, aberto do cartão do quadro
//! (ciclo 477).
//!
//! Faz pela tarefa o que o modal de detalhes faz pelo card do kanban:
//! tags, prazo, checklist, comentários e anexos. A diferença é onde isso
//! mora — no texto da PÁGINA da tarefa (`core::detalhes`) —, então cada
//! ação grava e a tela de projeto relê, em vez de editar uma cópia e
//! salvar no fim.

use anotadinho_core::projeto::Tarefa;
use yew::prelude::*;

use crate::api;
use crate::components::icon::Icon;

#[derive(Properties, PartialEq, Clone)]
pub struct TarefaDetalhesProps {
    pub vault_path: String,
    /// A tarefa como a última leitura do projeto a viu.
    pub tarefa: Tarefa,
    /// `AAAA-MM-DD`, pra o prazo vencido.
    pub hoje: String,
    /// As outras tarefas do projeto (número e título): de quem esta pode
    /// depender (ciclo 513).
    #[prop_or_default]
    pub outras: Vec<(u32, String)>,
    /// Abrir as mudanças da tarefa no repositório (ciclo 518).
    #[prop_or_default]
    pub on_mudancas: Option<Callback<()>>,
    /// Gravou: a tela relê o projeto.
    pub on_mudou: Callback<()>,
    pub on_fechar: Callback<()>,
    pub on_abrir_pagina: Callback<()>,
}

fn valor_do_input(e: &InputEvent) -> String {
    let alvo: web_sys::HtmlInputElement = e.target_unchecked_into();
    alvo.value()
}

#[function_component(TarefaDetalhes)]
pub fn tarefa_detalhes(props: &TarefaDetalhesProps) -> Html {
    let erro = use_state(|| None::<String>);
    let tag_nova = use_state(String::new);
    let comentario = use_state(String::new);
    let t = &props.tarefa;

    // Toda ação tem a mesma cara: grava, e em caso de erro mostra AQUI —
    // o erro do cabeçalho do projeto fica longe do que se clicou.
    let rodar = {
        let (erro, on_mudou) = (erro.clone(), props.on_mudou.clone());
        move |futuro: std::pin::Pin<Box<dyn std::future::Future<Output = Result<(), String>>>>| {
            let (erro, on_mudou) = (erro.clone(), on_mudou.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match futuro.await {
                    Ok(()) => {
                        erro.set(None);
                        on_mudou.emit(());
                    }
                    Err(e) => erro.set(Some(e)),
                }
            });
        }
    };

    let gravar_tags = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |tags: Vec<String>| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move { api::tags_da_tarefa(&v, &c, &tags).await }));
        })
    };
    let acrescentar_tag = {
        let (gravar, tag_nova, atuais) = (gravar_tags.clone(), tag_nova.clone(), t.tags.clone());
        Callback::from(move |_: ()| {
            let nova = tag_nova.trim().to_string();
            if nova.is_empty() {
                return;
            }
            let mut tags = atuais.clone();
            tags.push(nova);
            tag_nova.set(String::new());
            gravar.emit(tags);
        })
    };
    let prazo = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |novo: Option<String>| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move { api::prazo_da_tarefa(&v, &c, novo.as_deref()).await }));
        })
    };
    let prioridade = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |nova: Option<String>| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move { api::prioridade_da_tarefa(&v, &c, nova.as_deref()).await }));
        })
    };
    let gravar_deps = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |deps: Vec<String>| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move { api::dependencias_da_tarefa(&v, &c, &deps).await }));
        })
    };
    // O nome de uma dependência: a tarefa que ela cita, quando é um número
    // do projeto; senão, como está escrita (nome de arquivo, wikilink).
    let nome_da_dep = |d: &str| -> String {
        match d.trim().parse::<u32>().ok().and_then(|n| props.outras.iter().find(|(m, _)| *m == n)) {
            Some((n, titulo)) => format!("{n:03} {titulo}"),
            None => d.to_string(),
        }
    };
    let alternar = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |i: usize| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move { api::alternar_item_da_tarefa(&v, &c, i).await }));
        })
    };
    let comentar = {
        let (v, c, rodar, comentario) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone(), comentario.clone());
        Callback::from(move |_: MouseEvent| {
            let texto = comentario.trim().to_string();
            if texto.is_empty() {
                return;
            }
            let (v, c, comentario) = (v.clone(), c.clone(), comentario.clone());
            rodar(Box::pin(async move {
                let quando = crate::state::agora_legivel();
                let r = api::comentar_tarefa(&v, &c, &quando, &texto).await;
                if r.is_ok() {
                    comentario.set(String::new());
                }
                r
            }));
        })
    };
    let anexar = {
        let (v, c, rodar) = (props.vault_path.clone(), t.caminho.clone(), rodar.clone());
        Callback::from(move |_: MouseEvent| {
            let (v, c) = (v.clone(), c.clone());
            rodar(Box::pin(async move {
                match api::escolher_arquivo().await? {
                    Some(origem) => api::anexar_na_tarefa(&v, &c, &origem).await.map(|_| ()),
                    // Cancelar o diálogo não é erro.
                    None => Ok(()),
                }
            }));
        })
    };
    let abrir_anexo = {
        let (v, erro) = (props.vault_path.clone(), erro.clone());
        Callback::from(move |caminho: String| {
            let (v, erro) = (v.clone(), erro.clone());
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api::abrir_anexo(&v, &caminho).await {
                    erro.set(Some(e));
                }
            });
        })
    };

    let (feitos, total) = t.progresso();
    let vencida = t.vencida(&props.hoje);
    let fechar = props.on_fechar.reform(|_: MouseEvent| ());
    let abrir_pagina = props.on_abrir_pagina.reform(|_: MouseEvent| ());

    html! {
        <aside class="tarefa-detalhes" data-nav-group="tarefa-detalhes" data-tarefa={t.caminho.clone()}>
            <header class="tarefa-detalhes__topo">
                <span class="tarefa-detalhes__numero">{ format!("{:03}", t.numero) }</span>
                <h3 class="tarefa-detalhes__titulo">{ &t.titulo }</h3>
                <button class="btn btn--ghost btn--xs tarefa-detalhes__abrir" data-nav-item="true"
                    data-nav-parent="tarefa-detalhes" title="Abrir a página da tarefa" onclick={abrir_pagina}>
                    <Icon name="external-link" />
                </button>
                <button class="btn btn--ghost btn--xs tarefa-detalhes__fechar" data-nav-item="true"
                    data-nav-parent="tarefa-detalhes" title="Fechar" onclick={fechar}>
                    <Icon name="x" />
                </button>
            </header>
            if let Some(e) = (*erro).clone() {
                <p class="tarefa-detalhes__erro">{ e }</p>
            }

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ "Tags" }</h4>
                <div class="tarefa-detalhes__tags">
                    { for t.tags.iter().enumerate().map(|(i, tag)| {
                        let (gravar, mut sem) = (gravar_tags.clone(), t.tags.clone());
                        sem.remove(i);
                        html! {
                            <span class="badge badge--info tarefa-detalhes__tag">{ tag }
                                <button class="tarefa-detalhes__tag-tirar" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                                    title={format!("Tirar {tag}")}
                                    onclick={Callback::from(move |_: MouseEvent| gravar.emit(sem.clone()))}>
                                    <Icon name="x" />
                                </button>
                            </span>
                        }
                    }) }
                    <input class="input input--sm tarefa-detalhes__tag-nova" type="text" placeholder="+ tag" data-nav-item="true"
                        data-nav-parent="tarefa-detalhes" value={(*tag_nova).clone()}
                        oninput={{ let tn = tag_nova.clone(); Callback::from(move |e: InputEvent| tn.set(valor_do_input(&e))) }}
                        onkeydown={{
                            let acrescentar = acrescentar_tag.clone();
                            Callback::from(move |e: KeyboardEvent| if e.key() == "Enter" { e.prevent_default(); acrescentar.emit(()) })
                        }} />
                </div>
            </section>

            // O repositório (ciclo 518): onde a tarefa roda e se passou.
            if t.base_git.is_some() || t.verificacao.is_some() || t.branch_git.is_some() {
                <section class="tarefa-detalhes__secao">
                    <h4 class="tarefa-detalhes__rotulo">{ "Repositório" }</h4>
                    if let Some(b) = &t.branch_git {
                        <p class="tarefa-detalhes__repo">{ "branch " }<code>{ b }</code></p>
                    }
                    if let Some(v) = &t.verificacao {
                        <p class={classes!("tarefa-detalhes__repo", if v == "passou" { "tarefa-detalhes__repo--passou" } else { "tarefa-detalhes__repo--falhou" })}>
                            <Icon name={if v == "passou" { "check" } else { "x" }} />
                            { if v == "passou" { " a verificação passou" } else { " a verificação falhou — a saída está na seção Verificação da página" } }
                        </p>
                    }
                    if let (Some(abrir), true) = (props.on_mudancas.clone(), t.base_git.is_some()) {
                        <button class="btn btn--ghost btn--sm tarefa-detalhes__mudancas" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                            onclick={abrir.reform(|_: MouseEvent| ())}>
                            <Icon name="git-branch" />{ " Ver mudanças" }
                        </button>
                    }
                </section>
            }

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ "Prioridade" }</h4>
                <select class="input input--sm tarefa-detalhes__prioridade" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                    onchange={{
                        let prioridade = prioridade.clone();
                        Callback::from(move |e: Event| {
                            let alvo: web_sys::HtmlSelectElement = e.target_unchecked_into();
                            let v = alvo.value();
                            prioridade.emit(Some(v).filter(|v| !v.is_empty()));
                        })
                    }}>
                    <option value="" selected={t.prioridade.is_none()}>{ "— (média)" }</option>
                    { for anotadinho_core::projeto::PRIORIDADES.iter().map(|p| html! {
                        <option value={*p} selected={t.prioridade.as_deref() == Some(*p)}>{ match *p { "alta" => "alta", "baixa" => "baixa", _ => "média" } }</option>
                    }) }
                </select>
            </section>

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ format!("Depende de · {}", t.depende.len()) }</h4>
                <div class="tarefa-detalhes__deps">
                    { for t.depende.iter().enumerate().map(|(i, d)| {
                        let (gravar, mut sem) = (gravar_deps.clone(), t.depende.clone());
                        sem.remove(i);
                        html! {
                            <span class="badge tarefa-detalhes__dep">{ nome_da_dep(d) }
                                <button class="tarefa-detalhes__tag-tirar" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                                    title={format!("Não depender de {d}")}
                                    onclick={Callback::from(move |_: MouseEvent| gravar.emit(sem.clone()))}>
                                    <Icon name="x" />
                                </button>
                            </span>
                        }
                    }) }
                    <select class="input input--sm tarefa-detalhes__dep-nova" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                        onchange={{
                            let (gravar, atuais) = (gravar_deps.clone(), t.depende.clone());
                            Callback::from(move |e: Event| {
                                let alvo: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                let v = alvo.value();
                                if !v.is_empty() {
                                    let mut deps = atuais.clone();
                                    deps.push(v);
                                    gravar.emit(deps);
                                }
                            })
                        }}>
                        <option value="" selected=true>{ "+ depende de…" }</option>
                        { for props.outras.iter().filter(|(n, _)| !t.depende.iter().any(|d| d.trim() == n.to_string())).map(|(n, titulo)| html! {
                            <option value={n.to_string()}>{ format!("{n:03} {titulo}") }</option>
                        }) }
                    </select>
                </div>
            </section>

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ "Prazo" }</h4>
                <div class={classes!("tarefa-detalhes__prazo", vencida.then_some("tarefa-detalhes__prazo--vencido"))}>
                    <input class="input input--sm tarefa-detalhes__prazo-data" type="date" data-nav-item="true"
                        data-nav-parent="tarefa-detalhes" value={t.prazo.clone().unwrap_or_default()}
                        onchange={{
                            let prazo = prazo.clone();
                            Callback::from(move |e: Event| {
                                let alvo: web_sys::HtmlInputElement = e.target_unchecked_into();
                                let v = alvo.value();
                                prazo.emit(Some(v).filter(|v| !v.is_empty()));
                            })
                        }} />
                    if t.prazo.is_some() {
                        <button class="btn btn--ghost btn--xs tarefa-detalhes__prazo-tirar" data-nav-item="true"
                            data-nav-parent="tarefa-detalhes" onclick={{ let prazo = prazo.clone(); Callback::from(move |_: MouseEvent| prazo.emit(None)) }}>
                            { "Tirar" }
                        </button>
                    }
                    if vencida {
                        <span class="tarefa-detalhes__vencido"><Icon name="alert-triangle" />{ " vencido" }</span>
                    }
                </div>
            </section>

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ format!("Critérios de aceite · {feitos}/{total}") }</h4>
                if t.checklist.is_empty() {
                    <p class="tarefa-detalhes__vazio">{ "Nenhum critério ainda: escreva-os na página da tarefa." }</p>
                }
                <ul class="tarefa-detalhes__checklist">
                    { for t.checklist.iter().enumerate().map(|(i, item)| {
                        let alternar = alternar.clone();
                        html! {
                            <li>
                                <label class={classes!("tarefa-detalhes__item", item.feito.then_some("tarefa-detalhes__item--feito"))}>
                                    <input type="checkbox" class="tarefa-detalhes__marca" checked={item.feito} data-nav-item="true"
                                        data-nav-parent="tarefa-detalhes" data-indice={i.to_string()}
                                        onchange={Callback::from(move |_: Event| alternar.emit(i))} />
                                    { &item.texto }
                                </label>
                            </li>
                        }
                    }) }
                </ul>
            </section>

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ format!("Comentários · {}", t.comentarios.len()) }</h4>
                <ul class="tarefa-detalhes__comentarios">
                    { for t.comentarios.iter().map(|c| html! {
                        <li class="tarefa-detalhes__comentario">
                            <span class="tarefa-detalhes__quando">{ &c.quando }</span>
                            <p class="tarefa-detalhes__texto">{ &c.texto }</p>
                        </li>
                    }) }
                </ul>
                <textarea class="input tarefa-detalhes__comentario-novo" rows="2" placeholder="Escreva um comentário"
                    data-nav-item="true" data-nav-parent="tarefa-detalhes" value={(*comentario).clone()}
                    oninput={{
                        let c = comentario.clone();
                        Callback::from(move |e: InputEvent| {
                            let alvo: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
                            c.set(alvo.value());
                        })
                    }} />
                <button class="btn btn--sm tarefa-detalhes__comentar" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                    onclick={comentar}>
                    <Icon name="message-circle" />{ " Comentar" }
                </button>
            </section>

            <section class="tarefa-detalhes__secao">
                <h4 class="tarefa-detalhes__rotulo">{ format!("Anexos · {}", t.anexos.len()) }</h4>
                <ul class="tarefa-detalhes__anexos">
                    { for t.anexos.iter().map(|a| {
                        let (abrir, caminho) = (abrir_anexo.clone(), a.caminho.clone());
                        html! {
                            <li>
                                <button class="projeto__link tarefa-detalhes__anexo" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                                    title={a.caminho.clone()} onclick={Callback::from(move |_: MouseEvent| abrir.emit(caminho.clone()))}>
                                    <Icon name="paperclip" />{ " " }{ &a.nome }
                                </button>
                            </li>
                        }
                    }) }
                </ul>
                <button class="btn btn--ghost btn--sm tarefa-detalhes__anexar" data-nav-item="true" data-nav-parent="tarefa-detalhes"
                    onclick={anexar}>
                    <Icon name="plus" />{ " Anexar arquivo" }
                </button>
            </section>
        </aside>
    }
}
