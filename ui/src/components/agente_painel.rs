//! O painel do agente (ciclo 427) — uma página `type: agente`.
//!
//! A TUI respondia "o que esse bicho pode fazer, e o que ele já fez" por
//! três comandos da barra (405, 406, 407). Na janela não havia resposta
//! nenhuma: quem usa o app pela janela — que é a maioria — não tinha
//! como ver onde o agente pode escrever, nem o que ele rodou, nem quanto
//! custou.
//!
//! As três seções são a mesma pergunta em três tempos: o que ele PODE
//! (permissões), o que ele SABE fazer (contrato), e o que ele JÁ FEZ
//! (execuções, com o custo do dia). Tudo lido dos mesmos handlers da
//! TUI, então as duas telas nunca discordam.

use yew::prelude::*;

use crate::api;
use crate::components::icon::Icon;

#[derive(Properties, PartialEq, Clone)]
pub struct AgentePainelProps {
    pub vault_path: String,
    /// Pra abrir a conversa de uma execução.
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
}

#[function_component(AgentePainel)]
pub fn agente_painel(props: &AgentePainelProps) -> Html {
    let permissoes = use_state(anotadinho_core::permissoes::Permissoes::default);
    let gatilhos = use_state(Vec::<anotadinho_core::gatilho::Gatilho>::new);
    let guardas = use_state(anotadinho_core::guardas::Guardas::default);
    let freio = use_state(|| anotadinho_core::guardas::Freio::Nenhum);
    let execucoes = use_state(Vec::<anotadinho_core::execucao::Execucao>::new);
    let erro = use_state(|| None::<String>);
    let salvo = use_state(|| false);

    {
        let (permissoes, execucoes, erro) = (permissoes.clone(), execucoes.clone(), erro.clone());
        let gatilhos = gatilhos.clone();
        let (guardas, freio) = (guardas.clone(), freio.clone());
        let vault_path = props.vault_path.clone();
        use_effect_with(vault_path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::ler_permissoes(&vault_path).await {
                    Ok(p) => permissoes.set(p),
                    Err(e) => erro.set(Some(e)),
                }
                if let Ok(x) = api::listar_execucoes(&vault_path).await {
                    execucoes.set(x);
                }
                if let Ok(g) = api::ler_gatilhos(&vault_path).await {
                    gatilhos.set(g);
                }
                if let Ok(g) = api::ler_guardas(&vault_path).await {
                    guardas.set(g);
                }
                if let Ok(f) = api::freio_do_dia(&vault_path).await {
                    freio.set(f);
                }
            });
        });
    }

    // Uma lista por linha, como no formulário da TUI: uma pasta por
    // linha é o que se edita sem inventar sintaxe.
    let editar_lista = {
        let permissoes = permissoes.clone();
        let salvo = salvo.clone();
        move |campo: &'static str| {
            let (permissoes, salvo) = (permissoes.clone(), salvo.clone());
            Callback::from(move |e: InputEvent| {
                use wasm_bindgen::JsCast;
                let Some(alvo) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok())
                else {
                    return;
                };
                let itens: Vec<String> = alvo
                    .value()
                    .lines()
                    .map(|l| l.trim().to_string())
                    .filter(|l| !l.is_empty())
                    .collect();
                let mut novo = (*permissoes).clone();
                if campo == "pode" {
                    novo.pode = itens;
                } else {
                    novo.nunca = itens;
                }
                permissoes.set(novo);
                salvo.set(false);
            })
        }
    };

    let salvar = {
        let (permissoes, erro, salvo) = (permissoes.clone(), erro.clone(), salvo.clone());
        let vault_path = props.vault_path.clone();
        Callback::from(move |_: MouseEvent| {
            let (permissoes, erro, salvo) = (permissoes.clone(), erro.clone(), salvo.clone());
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match api::gravar_permissoes(&vault_path, &permissoes).await {
                    Ok(()) => salvo.set(true),
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    // Ligar e desligar um gatilho — a decisão que mais se toma nesta
    // lista, e a que não deve exigir editar arquivo.
    let alternar_gatilho = {
        let (gatilhos, erro) = (gatilhos.clone(), erro.clone());
        let vault_path = props.vault_path.clone();
        Callback::from(move |nome: String| {
            let mut lista = (*gatilhos).clone();
            for g in lista.iter_mut().filter(|g| g.nome == nome) {
                g.ativo = !g.ativo;
            }
            gatilhos.set(lista.clone());
            let (vault_path, erro) = (vault_path.clone(), erro.clone());
            wasm_bindgen_futures::spawn_local(async move {
                if let Err(e) = api::gravar_gatilhos(&vault_path, &lista).await {
                    erro.set(Some(e));
                }
            });
        })
    };

    // As guardas: número por campo, gravado no vault.
    let editar_guarda = {
        let (guardas, erro) = (guardas.clone(), erro.clone());
        let vault_path = props.vault_path.clone();
        move |campo: &'static str| {
            let (guardas, erro, vault_path) = (guardas.clone(), erro.clone(), vault_path.clone());
            Callback::from(move |e: Event| {
                use wasm_bindgen::JsCast;
                let Some(alvo) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok()) else {
                    return;
                };
                let v = alvo.value();
                let mut novo = (*guardas).clone();
                match campo {
                    "execucoes" => novo.teto_execucoes = v.trim().parse().unwrap_or(0),
                    "custo" => novo.teto_custo_usd = v.trim().replace(',', ".").parse().unwrap_or(0.0),
                    "de" => novo.silencio_de = v.trim().to_string(),
                    _ => novo.silencio_ate = v.trim().to_string(),
                }
                guardas.set(novo.clone());
                let (vault_path, erro) = (vault_path.clone(), erro.clone());
                wasm_bindgen_futures::spawn_local(async move {
                    if let Err(e) = api::gravar_guardas(&vault_path, &novo).await {
                        erro.set(Some(e));
                    }
                });
            })
        }
    };

    let hoje = crate::state::agora_legivel();
    let dia = hoje.split_whitespace().next().unwrap_or("").to_string();
    let total = anotadinho_core::execucao::total_do_dia(&execucoes, &dia);

    html! {
        <main class="agente-painel">
            <h2 class="agente-painel__titulo">{ "Painel do agente" }</h2>
            if let Some(e) = &*erro {
                <p class="agente-painel__erro" role="alert">{ e }</p>
            }

            <section class="agente-painel__secao">
                <h3>{ "Onde ele pode propor" }</h3>
                <p class="agente-painel__dica">
                    { "Uma pasta por linha. \"Pode\" vazio significa qualquer lugar; \
                       \"nunca\" ganha de \"pode\", e o agente nunca escreve em \
                       .anotadinho/. Vale pra TUI, janela e CLI — o arquivo mora no vault." }
                </p>
                <div class="agente-painel__permissoes">
                    <label>
                        <span>{ "Pode propor em" }</span>
                        <textarea class="input" rows="3" placeholder="pages/specs"
                            value={permissoes.pode.join("\n")}
                            oninput={editar_lista("pode")} />
                    </label>
                    <label>
                        <span>{ "Nunca" }</span>
                        <textarea class="input" rows="3" placeholder="journals/"
                            value={permissoes.nunca.join("\n")}
                            oninput={editar_lista("nunca")} />
                    </label>
                </div>
                <div class="agente-painel__acoes">
                    <button class="btn btn--primary btn--sm" onclick={salvar}>{ "Salvar no vault" }</button>
                    if *salvo {
                        <span class="agente-painel__ok">{ "salvo" }</span>
                    }
                </div>
            </section>

            <section class="agente-painel__secao">
                <h3>
                    { "Até onde ele pode ir sozinho" }
                    if freio.parou() {
                        <span class="agente-painel__pausado">{ format!("pausado: {}", freio.motivo()) }</span>
                    }
                </h3>
                <p class="agente-painel__dica">
                    { "Os limites valem pro que roda SEM ninguém olhando. O que você manda \
                       na conversa é decisão sua e não passa por aqui." }
                </p>
                <div class="agente-painel__guardas">
                    <label>
                        <span>{ "Disparos por dia" }</span>
                        <input class="input" type="number" min="0" value={guardas.teto_execucoes.to_string()}
                            onchange={editar_guarda("execucoes")} />
                    </label>
                    <label>
                        <span>{ "Custo por dia (US$)" }</span>
                        <input class="input" type="number" min="0" step="0.5"
                            value={format!("{:.2}", guardas.teto_custo_usd)}
                            onchange={editar_guarda("custo")} />
                    </label>
                    <label>
                        <span>{ "Silêncio das" }</span>
                        <input class="input" type="time" value={guardas.silencio_de.clone()}
                            onchange={editar_guarda("de")} />
                    </label>
                    <label>
                        <span>{ "até" }</span>
                        <input class="input" type="time" value={guardas.silencio_ate.clone()}
                            onchange={editar_guarda("ate")} />
                    </label>
                </div>
            </section>

            <section class="agente-painel__secao">
                <h3>{ "Quando ele trabalha sozinho" }</h3>
                <p class="agente-painel__dica">
                    { "Gatilhos disparam quando uma pasta muda, quando uma consulta passa \
                       a ter resultado, ou todo dia a partir de uma hora. Cada disparo abre \
                       uma conversa própria e passa pela mesma fila e revisão." }
                </p>
                if gatilhos.is_empty() {
                    <p class="agente-painel__dica">
                        { "Nenhum gatilho. O arquivo é .anotadinho/gatilhos.json no vault." }
                    </p>
                } else {
                    <ul class="agente-painel__gatilhos">
                        { for gatilhos.iter().map(|g| {
                            let alternar = alternar_gatilho.clone();
                            let nome = g.nome.clone();
                            html! {
                                <li class="agente-painel__gatilho">
                                    <label class="agente-painel__liga">
                                        <input type="checkbox" checked={g.ativo}
                                            onchange={Callback::from(move |_: Event| alternar.emit(nome.clone()))} />
                                        <strong>{ &g.nome }</strong>
                                    </label>
                                    <span class="agente-painel__regra">{ g.quando.rotulo() }</span>
                                    <small>{ g.prompt.lines().next().unwrap_or("") }</small>
                                    if !g.ultimo.trim().is_empty() {
                                        <span class="agente-painel__quando">{ format!("último: {}", g.ultimo) }</span>
                                    }
                                </li>
                            }
                        }) }
                    </ul>
                }
            </section>

            <section class="agente-painel__secao">
                <h3>{ "O que ele sabe fazer" }</h3>
                <p class="agente-painel__dica">
                    { "O conjunto é fechado. Uma única ferramenta escreve — e ela não grava \
                       página: enfileira uma proposta pra você revisar." }
                </p>
                <ul class="agente-painel__ferramentas">
                    { for anotadinho_core::ferramentas::CONTRATO.iter().map(|f| html! {
                        <li class={classes!("agente-painel__ferramenta",
                            f.escreve.then_some("agente-painel__ferramenta--escreve"))}>
                            <code>{ f.assinatura() }</code>
                            <span class="agente-painel__marca">
                                { if f.escreve { "escreve · revisão" } else { "leitura" } }
                            </span>
                            <small>{ f.descricao }</small>
                        </li>
                    }) }
                </ul>
            </section>

            <section class="agente-painel__secao">
                <h3>
                    { "O que ele já fez" }
                    if let Some((uso, contaram, quantas)) = &total {
                        <span class="agente-painel__hoje">
                            { if contaram == quantas {
                                format!("hoje {}", uso.rotulo())
                              } else {
                                format!("hoje {} ({contaram}/{quantas} medidas)", uso.rotulo())
                              } }
                        </span>
                    }
                </h3>
                if execucoes.is_empty() {
                    <p class="agente-painel__dica">{ "Nenhuma execução registrada ainda." }</p>
                } else {
                    <ul class="agente-painel__execucoes">
                        { for execucoes.iter().take(50).map(|x| {
                            let abrir = props.on_page_selected.clone();
                            let conversa = x.conversa.clone();
                            let titulo = std::path::Path::new(&conversa)
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_else(|| conversa.clone());
                            let glifo = match x.fim {
                                anotadinho_core::execucao::Fim::Respondeu => "check",
                                anotadinho_core::execucao::Fim::Falhou(_) => "x",
                                anotadinho_core::execucao::Fim::Interrompida => "square",
                            };
                            html! {
                                <li class="agente-painel__execucao">
                                    <Icon name={glifo} />
                                    <button class="agente-painel__conversa"
                                        onclick={Callback::from(move |_: MouseEvent| abrir.emit(api::PageMeta {
                                            path: conversa.clone(),
                                            title: titulo.clone(),
                                            section: "pages".to_string(),
                                        }))}>
                                        { std::path::Path::new(&x.conversa).file_stem()
                                            .map(|s| s.to_string_lossy().to_string())
                                            .unwrap_or_else(|| x.conversa.clone()) }
                                    </button>
                                    <span class="agente-painel__quando">{ &x.quando }</span>
                                    <span class="agente-painel__fim">{ x.fim.rotulo() }</span>
                                    <span class="agente-painel__dur">{ format!("{}s", x.segundos) }</span>
                                    if let Some(u) = &x.uso {
                                        <span class="agente-painel__uso">{ u.rotulo() }</span>
                                    }
                                </li>
                            }
                        }) }
                    </ul>
                }
            </section>
        </main>
    }
}
