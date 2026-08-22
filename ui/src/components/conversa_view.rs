//! Painel de conversa com o agente (ciclo 202).
//!
//! Aparece quando a página tem `type: conversa`, do mesmo jeito que
//! `type: kanban` mostra um board — a conversa É uma página, e o
//! markdown dela continua legível fora do app.
//!
//! O que dá fluidez, e é o motivo de existir:
//!
//! - **Contexto automático**: a página que estava aberta antes vai junto
//!   no prompt, sem copiar e colar.
//! - **Histórico como arquivo**: o que se manda pro modelo sai do
//!   próprio `.md`, então a conversa sobrevive a fechar o app, entra no
//!   git e pode ser lida pela consulta como qualquer página.

use crate::api;
use crate::components::icon::Icon;
use anotadinho_core::agente::Adaptador;
use anotadinho_core::conversa::{self, Autor, Mensagem};
use anotadinho_core::fluxo::{self, Artefato};
use yew::prelude::*;

/// Quantas mensagens do histórico vão no prompt. Corta as mais ANTIGAS.
const HISTORICO_NO_PROMPT: usize = 12;

#[derive(Properties, PartialEq, Clone)]
pub struct ConversaViewProps {
    /// Abre a página criada ao promover uma mensagem (ciclo 203).
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
    pub vault_path: String,
    pub page: api::PageMeta,
    /// Página que estava aberta antes — vai como contexto.
    #[prop_or_default]
    pub contexto_path: Option<String>,
}

#[function_component(ConversaView)]
pub fn conversa_view(props: &ConversaViewProps) -> Html {
    let mensagens = use_state(Vec::<Mensagem>::new);
    let rascunho = use_state(String::new);
    let ocupado = use_state(|| false);
    let erro = use_state(|| None::<String>);
    let usar_contexto = use_state(|| true);

    // Carrega a conversa do arquivo.
    {
        let mensagens = mensagens.clone();
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        use_effect_with(props.page.path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(conteudo) = api::read_page(&vault_path, &path).await else { return };
                let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
                mensagens.set(conversa::parse(corpo));
            });
            || ()
        });
    }

    let enviar = {
        let mensagens = mensagens.clone();
        let rascunho = rascunho.clone();
        let ocupado = ocupado.clone();
        let erro = erro.clone();
        let usar_contexto = usar_contexto.clone();
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        let contexto_path = props.contexto_path.clone();
        Callback::from(move |_: MouseEvent| {
            let pergunta = (*rascunho).trim().to_string();
            if pergunta.is_empty() || *ocupado {
                return;
            }
            let (mensagens, rascunho, ocupado, erro) =
                (mensagens.clone(), rascunho.clone(), ocupado.clone(), erro.clone());
            let (vault_path, path) = (vault_path.clone(), path.clone());
            let contexto_path = if *usar_contexto { contexto_path.clone() } else { None };
            ocupado.set(true);
            erro.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let agora = crate::state::agora_legivel();

                // Grava a pergunta ANTES de chamar o agente: se ele
                // falhar ou o app fechar no meio, o que a pessoa
                // escreveu não se perde.
                let minha = Mensagem { autor: Autor::Voce, quando: agora.clone(), texto: pergunta.clone() };
                let mut lista = (*mensagens).clone();
                lista.push(minha.clone());
                mensagens.set(lista.clone());
                rascunho.set(String::new());
                gravar(&vault_path, &path, &lista).await;

                let contexto = match &contexto_path {
                    Some(p) => api::read_page(&vault_path, p).await.ok(),
                    None => None,
                };
                let prompt = conversa::montar_prompt(
                    &lista[..lista.len() - 1],
                    &pergunta,
                    contexto.as_deref(),
                    HISTORICO_NO_PROMPT,
                );

                let adaptador = crate::state::load_adaptador();
                match api::rodar_agente(&adaptador, &prompt, &vault_path).await {
                    Ok(resposta) => {
                        let dele = Mensagem {
                            autor: Autor::Agente,
                            quando: crate::state::agora_legivel(),
                            texto: resposta,
                        };
                        // A partir de `lista`, NÃO de `(*mensagens)`: o
                        // handle capturado no closure ainda tem o valor
                        // de antes do `set`, então ler dele aqui
                        // descartaria a pergunta que acabou de entrar.
                        // Mesmo padrão dos ciclos 155, 157 e 201.
                        let mut lista2 = lista.clone();
                        lista2.push(dele);
                        mensagens.set(lista2.clone());
                        gravar(&vault_path, &path, &lista2).await;
                    }
                    Err(e) => erro.set(Some(e)),
                }
                ocupado.set(false);
            });
        })
    };

    let on_input = {
        let rascunho = rascunho.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(el) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                rascunho.set(el.value());
            }
        })
    };

    let alternar_contexto = {
        let usar_contexto = usar_contexto.clone();
        Callback::from(move |_: MouseEvent| usar_contexto.set(!*usar_contexto))
    };

    // Promove uma resposta em artefato (ciclo 203).
    //
    // É a ponte entre a conversa e o trabalho estruturado: sem ela o
    // fluxo morre no copiar-e-colar, que é onde a maioria das
    // integrações de chat com "criar tarefa" para.
    let promover = {
        let vault_path = props.vault_path.clone();
        let conversa_path = props.page.path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let erro = erro.clone();
        Callback::from(move |(artefato, texto): (Artefato, String)| {
            let (vault_path, conversa_path) = (vault_path.clone(), conversa_path.clone());
            let (on_page_selected, erro) = (on_page_selected.clone(), erro.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let titulo = fluxo::titulo_sugerido(&texto, 60);
                let hoje = crate::state::agora_legivel();
                let hoje = hoje.split(' ').next().unwrap_or("").to_string();
                let md = fluxo::montar_pagina(
                    artefato,
                    &titulo,
                    &texto,
                    Some(&conversa_path),
                    &hoje,
                );
                let path = format!("{}/{}.md", artefato.pasta(), fluxo::slug_de_titulo(&titulo));
                match api::write_page(&vault_path, &path, &md).await {
                    Ok(_) => on_page_selected.emit(api::PageMeta {
                        path,
                        title: titulo,
                        section: "pages".to_string(),
                    }),
                    Err(e) => erro.set(Some(format!("não consegui criar a página: {e}"))),
                }
            });
        })
    };

    let adaptador = crate::state::load_adaptador();

    html! {
        <main class="conversa">
            <header class="conversa__topo">
                <h2 class="conversa__titulo">{ &props.page.title }</h2>
                <span class="conversa__agente" title={adaptador.binario.clone()}>
                    <Icon name="zap" />{ adaptador.nome.clone() }
                </span>
                if props.contexto_path.is_some() {
                    <button class={classes!("btn", "btn--xs",
                        if *usar_contexto { "btn--primary" } else { "btn--ghost" })}
                        onclick={alternar_contexto}
                        title="Mandar a página anterior junto como contexto">
                        { if *usar_contexto { "com contexto" } else { "sem contexto" } }
                    </button>
                }
            </header>

            <div class="conversa__mensagens">
                if mensagens.is_empty() {
                    <p class="conversa__vazia">
                        { "Pergunte algo, peça uma spec, ou mande analisar a página aberta." }
                    </p>
                }
                { for mensagens.iter().map(|m| {
                    let classe = match m.autor {
                        Autor::Voce => "conversa__msg conversa__msg--voce",
                        Autor::Agente => "conversa__msg conversa__msg--agente",
                    };
                    html! {
                        <article class={classe}>
                            <header class="conversa__msg-topo">
                                <span class="conversa__msg-autor">{ m.autor.slug() }</span>
                                <span class="conversa__msg-quando">{ &m.quando }</span>
                            </header>
                            <div class="conversa__msg-corpo">
                                { Html::from_html_unchecked(
                                    crate::markdown_render::render(&m.texto).into()) }
                            </div>
                            if m.autor == Autor::Agente {
                                <div class="conversa__msg-acoes">
                                    { for [Artefato::Spec, Artefato::Proposta].into_iter().map(|a| {
                                        let promover = promover.clone();
                                        let texto = m.texto.clone();
                                        let onclick = Callback::from(move |_: MouseEvent| {
                                            promover.emit((a, texto.clone()))
                                        });
                                        html! {
                                            <button class="btn btn--ghost btn--xs" {onclick}
                                                title={format!("Criar uma {} a partir desta resposta", a.label())}>
                                                { format!("virar {}", a.label().to_lowercase()) }
                                            </button>
                                        }
                                    }) }
                                </div>
                            }
                        </article>
                    }
                }) }
                if *ocupado {
                    <p class="conversa__pensando"><span class="spinner"></span>{ " pensando..." }</p>
                }
                if let Some(e) = &*erro {
                    <p class="conversa__erro">{ e }</p>
                }
            </div>

            <footer class="conversa__compositor">
                <textarea class="conversa__campo" rows="3"
                    placeholder="Escreva e mande. Shift+Enter quebra linha."
                    value={(*rascunho).clone()}
                    oninput={on_input}
                    disabled={*ocupado} />
                <button class="btn btn--primary" onclick={enviar} disabled={*ocupado || rascunho.trim().is_empty()}>
                    { if *ocupado { "..." } else { "Enviar" } }
                </button>
            </footer>
        </main>
    }
}

/// Reescreve o corpo da página com as mensagens, preservando o
/// frontmatter — que é onde vivem `type: conversa` e o que mais a pessoa
/// tiver posto lá.
async fn gravar(vault_path: &str, path: &str, mensagens: &[Mensagem]) {
    let atual = api::read_page(vault_path, path).await.unwrap_or_default();
    let (frontmatter, _) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
    let corpo = conversa::serializar(mensagens);
    let novo = if frontmatter.is_empty() { corpo } else { format!("{frontmatter}\n{corpo}") };
    let _ = api::write_page(vault_path, path, &novo).await;
}
