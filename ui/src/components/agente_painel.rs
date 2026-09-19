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
    let execucoes = use_state(Vec::<anotadinho_core::execucao::Execucao>::new);
    let erro = use_state(|| None::<String>);
    let salvo = use_state(|| false);

    {
        let (permissoes, execucoes, erro) = (permissoes.clone(), execucoes.clone(), erro.clone());
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
