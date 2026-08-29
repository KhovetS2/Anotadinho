//! Revisão das propostas do agente (ciclo 204).
//!
//! O agente não escreve no vault: ele PROPÕE, e a mudança só vira
//! arquivo depois de alguém ver o diff e aprovar.
//!
//! Essa é a defesa que sustenta todo o acoplamento com modelos. As
//! outras — não ter shell, blindar o contexto no prompt — reduzem a
//! chance de o agente ser enganado. Esta aqui é a que continua valendo
//! MESMO se ele for: o estrago para nesta tela.
//!
//! O diff é o mesmo motor do ciclo 190, então a pessoa lê a mudança do
//! agente no formato que já conhece da barra de conflito.

use crate::api;
use crate::components::icon::Icon;
use crate::components::pagina_preview::PaginaPreview;
use anotadinho_core::proposta::{Operacao, Proposta};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct PropostasViewProps {
    pub vault_path: String,
    pub on_page_selected: Callback<api::PageMeta>,
    /// Avisa que a fila mudou (ciclo 210) — sem isto o aviso do
    /// cabeçalho fica preso no número de antes, porque aplicar ou
    /// recusar não mexe na lista de páginas.
    #[prop_or_default]
    pub on_fila_mudou: Callback<()>,
}

#[function_component(PropostasView)]
pub fn propostas_view(props: &PropostasViewProps) -> Html {
    let propostas = use_state(Vec::<Proposta>::new);
    let atuais = use_state(|| std::collections::HashMap::<String, String>::new());
    let erro = use_state(|| None::<String>);
    let recarregar = use_state(|| 0u32);
    // Quais propostas estão sendo vistas RENDERIZADAS em vez de como
    // diff. Diff continua o padrão: é o que responde "o que mudou". A
    // visualização responde "como vai ficar", que é outra pergunta —
    // e a única que quem não escreve embed consegue responder.
    let visualizando = use_state(std::collections::HashSet::<String>::new);

    {
        let propostas = propostas.clone();
        let atuais = atuais.clone();
        let vault_path = props.vault_path.clone();
        use_effect_with(*recarregar, move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let lista = api::listar_propostas(&vault_path).await.unwrap_or_default();
                // Lê o conteúdo ATUAL de cada alvo pra montar o diff —
                // sem isso a revisão mostraria só o texto novo, e a
                // pergunta "o que muda?" ficaria sem resposta.
                let mut mapa = std::collections::HashMap::new();
                for p in &lista {
                    let atual = api::read_page(&vault_path, &p.alvo).await.unwrap_or_default();
                    mapa.insert(p.alvo.clone(), atual);
                }
                atuais.set(mapa);
                propostas.set(lista);
            });
            || ()
        });
    }

    let alternar_modo = {
        let visualizando = visualizando.clone();
        Callback::from(move |id: String| {
            let mut novo = (*visualizando).clone();
            if !novo.remove(&id) {
                novo.insert(id);
            }
            visualizando.set(novo);
        })
    };

    let decidir = {
        let vault_path = props.vault_path.clone();
        let recarregar = recarregar.clone();
        let erro = erro.clone();
        let on_page_selected = props.on_page_selected.clone();
        let on_fila_mudou = props.on_fila_mudou.clone();
        Callback::from(move |(id, aplicar): (String, bool)| {
            let (vault_path, recarregar, erro) =
                (vault_path.clone(), recarregar.clone(), erro.clone());
            let on_page_selected = on_page_selected.clone();
            let on_fila_mudou = on_fila_mudou.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let r = if aplicar {
                    api::aplicar_proposta(&vault_path, &id).await
                } else {
                    api::recusar_proposta(&vault_path, &id).await
                };
                match r {
                    Ok(alvo) if aplicar && !alvo.is_empty() => {
                        let title = std::path::Path::new(&alvo)
                            .file_stem()
                            .map(|s| s.to_string_lossy().to_string())
                            .unwrap_or_else(|| alvo.clone());
                        on_page_selected.emit(api::PageMeta {
                            path: alvo,
                            title,
                            section: "pages".to_string(),
                        });
                    }
                    Ok(_) => {}
                    Err(e) => erro.set(Some(e)),
                }
                recarregar.set(*recarregar + 1);
                on_fila_mudou.emit(());
            });
        })
    };

    html! {
        <main class="propostas">
            <header class="propostas__topo">
                <h2 class="propostas__titulo">{ "Propostas do agente" }</h2>
                <span class="propostas__contagem">{ format!("{} pendente(s)", propostas.len()) }</span>
            </header>

            if let Some(e) = &*erro {
                <p class="propostas__erro">{ e }</p>
            }

            if propostas.is_empty() {
                <p class="propostas__vazio">
                    { "Nada pendente. O agente grava aqui em vez de escrever no vault; \
                       o que ele propuser aparece nesta tela pra você aprovar." }
                </p>
            }

            { for propostas.iter().map(|p| {
                let atual = atuais.get(&p.alvo).cloned().unwrap_or_default();
                let linhas = p.diff(&atual);
                let (removidas, adicionadas) = anotadinho_core::diff::contar(&linhas);
                let id_ok = p.id.clone();
                let id_no = p.id.clone();
                let preview = visualizando.contains(&p.id);
                let d1 = decidir.clone();
                let d2 = decidir.clone();
                html! {
                    <article class="propostas__item">
                        <header class="propostas__item-topo">
                            <span class={classes!("propostas__op",
                                if p.operacao == Operacao::Criar { "propostas__op--criar" } else { "propostas__op--substituir" })}>
                                { if p.operacao == Operacao::Criar { "criar" } else { "substituir" } }
                            </span>
                            <code class="propostas__alvo">{ &p.alvo }</code>
                            <span class="propostas__autor"><Icon name="zap" />{ &p.autor }</span>
                            <span class="propostas__quando">{ &p.quando }</span>
                        </header>

                        if !p.motivo.trim().is_empty() {
                            <p class="propostas__motivo">{ &p.motivo }</p>
                        }

                        <div class="propostas__cabecalho-modo">
                            <p class="propostas__resumo">
                                { format!("{removidas} linha(s) removida(s) · {adicionadas} adicionada(s)") }
                            </p>
                            <div class="propostas__modos" role="group" aria-label="Como ver a proposta">
                                { for [(false, "Diff"), (true, "Visualização")].into_iter().map(|(modo, rotulo)| {
                                    let alternar = alternar_modo.clone();
                                    let id = p.id.clone();
                                    html! {
                                        <button
                                            class={classes!("propostas__modo",
                                                (modo == preview).then_some("propostas__modo--atual"))}
                                            aria-pressed={(modo == preview).to_string()}
                                            onclick={Callback::from(move |_: MouseEvent| {
                                                if modo != preview {
                                                    alternar.emit(id.clone());
                                                }
                                            })}>
                                            { rotulo }
                                        </button>
                                    }
                                }) }
                            </div>
                        </div>

                        if preview {
                            // O conteúdo PROPOSTO, como a página fica se
                            // for aplicada.
                            <div class="propostas__preview">
                                <PaginaPreview conteudo={p.conteudo.clone()}
                                    vault_path={props.vault_path.clone()}
                                    page_path={p.alvo.clone()}
                                    nav_prefixo={p.id.clone()} />
                            </div>
                        } else {
                            <pre class="propostas__diff">
                                { for linhas.iter().map(|l| {
                                    let (classe, marca) = match l {
                                        anotadinho_core::diff::LinhaDiff::Igual { .. } => ("propostas__l", " "),
                                        anotadinho_core::diff::LinhaDiff::Removida { .. } => ("propostas__l propostas__l--sai", "-"),
                                        anotadinho_core::diff::LinhaDiff::Adicionada { .. } => ("propostas__l propostas__l--entra", "+"),
                                    };
                                    html! { <div class={classe}>{ format!("{marca}{}", l.texto()) }</div> }
                                }) }
                            </pre>
                        }

                        <div class="propostas__acoes">
                            <button class="btn btn--primary btn--sm"
                                onclick={Callback::from(move |_: MouseEvent| d1.emit((id_ok.clone(), true)))}>
                                { "Aplicar" }
                            </button>
                            <button class="btn btn--ghost btn--sm"
                                onclick={Callback::from(move |_: MouseEvent| d2.emit((id_no.clone(), false)))}>
                                { "Recusar" }
                            </button>
                        </div>
                    </article>
                }
            }) }
        </main>
    }
}
