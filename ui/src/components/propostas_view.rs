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
use crate::components::modal::Modal;
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
    // Os trechos TIRADOS da aplicação, por `id#índice` (ciclo 424, como
    // a TUI faz desde o 404): revisar proposta grande tudo-ou-nada é o
    // que faz aceitar mudança que ninguém leu.
    let fora = use_state(std::collections::HashSet::<String>::new);
    // Editar antes de aplicar (411) e recusar com motivo (404), que só
    // existiam na TUI; e as marcadas pra decidir em lote (409).
    let editando = use_state(|| None::<(String, String)>);
    let recusando = use_state(|| None::<(String, String)>);
    let marcadas = use_state(std::collections::HashSet::<String>::new);

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

    let alternar_trecho = {
        let fora = fora.clone();
        Callback::from(move |chave: String| {
            let mut novo = (*fora).clone();
            if !novo.remove(&chave) {
                novo.insert(chave);
            }
            fora.set(novo);
        })
    };

    // Aplica só os trechos escolhidos e registra a decisão como parcial.
    let aplicar_parcial = {
        let vault_path = props.vault_path.clone();
        let recarregar = recarregar.clone();
        let erro = erro.clone();
        let fora = fora.clone();
        let on_fila_mudou = props.on_fila_mudou.clone();
        Callback::from(move |(p, conteudo, aceitos, de): (Proposta, String, usize, usize)| {
            let (vault_path, recarregar, erro) = (vault_path.clone(), recarregar.clone(), erro.clone());
            let (fora, on_fila_mudou) = (fora.clone(), on_fila_mudou.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::aplicar_proposta_parcial(&vault_path, &p.id, &conteudo).await {
                    Ok(_) => {
                        let decisao = anotadinho_core::decisao::Decisao {
                            quando: crate::state::agora_legivel(),
                            proposta: p.id.clone(),
                            alvo: p.alvo.clone(),
                            autor: p.autor.clone(),
                            acao: anotadinho_core::decisao::Acao::Parcial { aceitos, de },
                            motivo: String::new(),
                        };
                        let _ = api::registrar_decisao(&vault_path, &decisao).await;
                        // Os trechos daquela proposta somem junto com ela.
                        let mut limpo = (*fora).clone();
                        limpo.retain(|c| !c.starts_with(&format!("{}#", p.id)));
                        fora.set(limpo);
                    }
                    Err(e) => erro.set(Some(e)),
                }
                recarregar.set(*recarregar + 1);
                on_fila_mudou.emit(());
            });
        })
    };

    // Recusar registrando o motivo: é a parte que ENSINA o agente na
    // volta seguinte (ciclo 421), e sem registro ela se perde.
    let confirmar_recusa = {
        let vault_path = props.vault_path.clone();
        let (recarregar, erro, recusando) = (recarregar.clone(), erro.clone(), recusando.clone());
        let propostas = propostas.clone();
        let on_fila_mudou = props.on_fila_mudou.clone();
        Callback::from(move |_: MouseEvent| {
            let Some((id, motivo)) = (*recusando).clone() else { return };
            let Some(p) = propostas.iter().find(|x| x.id == id).cloned() else { return };
            let (vault_path, recarregar, erro) = (vault_path.clone(), recarregar.clone(), erro.clone());
            let (recusando, on_fila_mudou) = (recusando.clone(), on_fila_mudou.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::recusar_proposta(&vault_path, &p.id).await {
                    Ok(_) => {
                        let decisao = anotadinho_core::decisao::Decisao {
                            quando: crate::state::agora_legivel(),
                            proposta: p.id.clone(),
                            alvo: p.alvo.clone(),
                            autor: p.autor.clone(),
                            acao: anotadinho_core::decisao::Acao::Recusada,
                            motivo,
                        };
                        let _ = api::registrar_decisao(&vault_path, &decisao).await;
                        recusando.set(None);
                    }
                    Err(e) => erro.set(Some(e)),
                }
                recarregar.set(*recarregar + 1);
                on_fila_mudou.emit(());
            });
        })
    };

    // Aplicar o conteúdo que a pessoa editou (ciclo 411).
    let aplicar_editada = {
        let vault_path = props.vault_path.clone();
        let (recarregar, erro, editando) = (recarregar.clone(), erro.clone(), editando.clone());
        let propostas = propostas.clone();
        let on_fila_mudou = props.on_fila_mudou.clone();
        Callback::from(move |_: MouseEvent| {
            let Some((id, conteudo)) = (*editando).clone() else { return };
            if conteudo.trim().is_empty() {
                erro.set(Some("conteúdo vazio: recuse a proposta em vez de gravar nada".into()));
                return;
            }
            let Some(p) = propostas.iter().find(|x| x.id == id).cloned() else { return };
            let (vault_path, recarregar, erro) = (vault_path.clone(), recarregar.clone(), erro.clone());
            let (editando, on_fila_mudou) = (editando.clone(), on_fila_mudou.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::aplicar_proposta_parcial(&vault_path, &p.id, &conteudo).await {
                    Ok(_) => {
                        let decisao = anotadinho_core::decisao::Decisao {
                            quando: crate::state::agora_legivel(),
                            proposta: p.id.clone(),
                            alvo: p.alvo.clone(),
                            autor: p.autor.clone(),
                            // O que entrou não é o que o agente escreveu.
                            acao: anotadinho_core::decisao::Acao::Editada,
                            motivo: String::new(),
                        };
                        let _ = api::registrar_decisao(&vault_path, &decisao).await;
                        editando.set(None);
                    }
                    Err(e) => erro.set(Some(e)),
                }
                recarregar.set(*recarregar + 1);
                on_fila_mudou.emit(());
            });
        })
    };

    // O lote é UMA decisão: aplica inteiro ou não aplica (ciclo 420).
    let decidir_lote = {
        let vault_path = props.vault_path.clone();
        let (recarregar, erro) = (recarregar.clone(), erro.clone());
        let propostas = propostas.clone();
        let on_fila_mudou = props.on_fila_mudou.clone();
        Callback::from(move |(lote, aplicar): (String, bool)| {
            let (vault_path, recarregar, erro) = (vault_path.clone(), recarregar.clone(), erro.clone());
            let on_fila_mudou = on_fila_mudou.clone();
            let do_lote: Vec<Proposta> = propostas
                .iter()
                .filter(|p| p.lote.as_deref() == Some(lote.as_str()))
                .cloned()
                .collect();
            wasm_bindgen_futures::spawn_local(async move {
                let r = if aplicar {
                    api::aplicar_lote(&vault_path, &lote).await.map(|a| a.len())
                } else {
                    api::recusar_lote(&vault_path, &lote).await
                };
                match r {
                    Ok(_) => {
                        // A auditoria continua por página.
                        let acao = if aplicar {
                            anotadinho_core::decisao::Acao::Aplicada
                        } else {
                            anotadinho_core::decisao::Acao::Recusada
                        };
                        for p in do_lote {
                            let decisao = anotadinho_core::decisao::Decisao {
                                quando: crate::state::agora_legivel(),
                                proposta: p.id.clone(),
                                alvo: p.alvo.clone(),
                                autor: p.autor.clone(),
                                acao: acao.clone(),
                                motivo: String::new(),
                            };
                            let _ = api::registrar_decisao(&vault_path, &decisao).await;
                        }
                    }
                    Err(e) => erro.set(Some(e)),
                }
                recarregar.set(*recarregar + 1);
                on_fila_mudou.emit(());
            });
        })
    };

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
                // Trechos (ciclo 404) e pares pro realce fino (410).
                let trechos = anotadinho_core::diff::trechos(&linhas);
                let id_do_trecho = p.id.clone();
                let tirados = (*fora).clone();
                let esta_fora = move |k: usize| tirados.contains(&format!("{id_do_trecho}#{k}"));
                let escolhidos: Vec<bool> = (0..trechos.len()).map(|k| !esta_fora(k)).collect();
                let aceitos = escolhidos.iter().filter(|x| **x).count();
                let parcial = aceitos < trechos.len();
                let conteudo_parcial = anotadinho_core::diff::aplicar_trechos(&linhas, &trechos, &escolhidos);
                let mut par_de: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
                for t in &trechos {
                    for (a, b) in anotadinho_core::diff::pares_do_trecho(&linhas, t) {
                        par_de.insert(a, b);
                        par_de.insert(b, a);
                    }
                }
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
                            if let Some(l) = &p.lote {
                                <span class="propostas__lote" title="Propostas deste lote são UMA decisão: aplicam juntas ou nenhuma">
                                    { format!("⛓ {l}") }
                                </span>
                            }
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
                                { for linhas.iter().enumerate().map(|(i, l)| {
                                    use anotadinho_core::diff::LinhaDiff;
                                    let k = trechos.iter().position(|t| i >= t.inicio && i < t.fim);
                                    let dentro_de_fora = k.is_some_and(|k| esta_fora(k));
                                    let cabecalho = k.filter(|k| trechos[*k].inicio == i).map(|k| {
                                        let t = &trechos[k];
                                        let chave = format!("{}#{k}", p.id);
                                        let alternar = alternar_trecho.clone();
                                        let tirado = esta_fora(k);
                                        html! {
                                            <div class="propostas__trecho">
                                                <label class="propostas__trecho-rot">
                                                    <input type="checkbox" checked={!tirado}
                                                        onchange={Callback::from(move |_: Event| alternar.emit(chave.clone()))} />
                                                    { format!("trecho {}/{} · −{} +{}", k + 1, trechos.len(), t.removidas, t.adicionadas) }
                                                </label>
                                                if tirado { <span class="propostas__trecho-fora">{ "fora" }</span> }
                                            </div>
                                        }
                                    });
                                    let (classe, marca) = match l {
                                        LinhaDiff::Igual { .. } => ("propostas__l", " "),
                                        LinhaDiff::Removida { .. } => ("propostas__l propostas__l--sai", "-"),
                                        LinhaDiff::Adicionada { .. } => ("propostas__l propostas__l--entra", "+"),
                                    };
                                    let classe = classes!(classe, dentro_de_fora.then_some("propostas__l--fora"));
                                    // Realce palavra a palavra (ciclo 410, agora
                                    // aqui): numa linha longa em que mudou uma
                                    // data, pintar tudo faz reler tudo.
                                    let pedacos = par_de.get(&i).filter(|_| l.mudou() && !dentro_de_fora).and_then(|outra| {
                                        let (a, b) = (linhas[i.min(*outra)].texto(), linhas[i.max(*outra)].texto());
                                        anotadinho_core::diff::diff_palavras(a, b)
                                            .map(|(la, lb)| if i < *outra { la } else { lb })
                                    });
                                    html! {
                                        <>
                                            { cabecalho.unwrap_or_default() }
                                            <div class={classe}>
                                                { marca }
                                                { match pedacos {
                                                    Some(ps) => html! { for ps.into_iter().map(|pd| html! {
                                                        <span class={classes!(pd.mudou.then_some("propostas__palavra"))}>{ pd.texto }</span>
                                                    }) },
                                                    None => html! { l.texto().to_string() },
                                                } }
                                            </div>
                                        </>
                                    }
                                }) }
                            </pre>
                        }

                        <div class="propostas__acoes">
                            <button class="btn btn--primary btn--sm" disabled={parcial && aceitos == 0}
                                onclick={{
                                    let aplicar_parcial = aplicar_parcial.clone();
                                    let proposta = p.clone();
                                    let conteudo = conteudo_parcial.clone();
                                    let total = trechos.len();
                                    Callback::from(move |_: MouseEvent| {
                                        if parcial {
                                            aplicar_parcial.emit((proposta.clone(), conteudo.clone(), aceitos, total));
                                        } else {
                                            d1.emit((id_ok.clone(), true));
                                        }
                                    })
                                }}>
                                { if parcial { format!("Aplicar {aceitos} de {}", trechos.len()) } else { "Aplicar".to_string() } }
                            </button>
                            <button class="btn btn--ghost btn--sm" onclick={{
                                let recusando = recusando.clone();
                                let id = id_no.clone();
                                Callback::from(move |_: MouseEvent| recusando.set(Some((id.clone(), String::new()))))
                            }}>
                                { "Recusar…" }
                            </button>
                            <button class="btn btn--ghost btn--sm" onclick={{
                                let editando = editando.clone();
                                let id = p.id.clone();
                                let conteudo = p.conteudo.clone();
                                Callback::from(move |_: MouseEvent| editando.set(Some((id.clone(), conteudo.clone()))))
                            }} title="Trocar uma frase antes de gravar, em vez de recusar e pedir de novo">
                                { "Editar…" }
                            </button>
                            if let Some(lote) = p.lote.clone() {
                                <span class="propostas__lote-acoes">
                                    <button class="btn btn--ghost btn--sm" onclick={{
                                        let d = decidir_lote.clone();
                                        let l = lote.clone();
                                        Callback::from(move |_: MouseEvent| d.emit((l.clone(), true)))
                                    }}>{ format!("Aplicar o lote {lote}") }</button>
                                    <button class="btn btn--ghost btn--sm" onclick={{
                                        let d = decidir_lote.clone();
                                        let l = lote.clone();
                                        Callback::from(move |_: MouseEvent| d.emit((l.clone(), false)))
                                    }}>{ "Recusar o lote" }</button>
                                </span>
                            }
                        </div>
                    </article>
                }
            }) }

            <Modal title="Recusar a proposta" open={recusando.is_some()}
                on_close={{
                    let recusando = recusando.clone();
                    Callback::from(move |_| recusando.set(None))
                }}>
                <p class="propostas__dica">
                    { "Por quê? O motivo entra no registro de decisões e é o que você \
                       conta ao agente na volta seguinte." }
                </p>
                <textarea class="input" rows="3" autofocus=true
                    value={recusando.as_ref().map(|(_, m)| m.clone()).unwrap_or_default()}
                    oninput={{
                        let recusando = recusando.clone();
                        Callback::from(move |e: InputEvent| {
                            use wasm_bindgen::JsCast;
                            let Some(a) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) else { return };
                            if let Some((id, _)) = (*recusando).clone() {
                                recusando.set(Some((id, a.value())));
                            }
                        })
                    }} />
                <div class="modal__actions">
                    <button class="btn btn--primary btn--sm" onclick={confirmar_recusa}>{ "Recusar" }</button>
                </div>
            </Modal>

            <Modal title="Editar antes de aplicar" open={editando.is_some()} wide=true
                on_close={{
                    let editando = editando.clone();
                    Callback::from(move |_| editando.set(None))
                }}>
                <p class="propostas__dica">
                    { "O que entrar aqui é o que vai pro arquivo — e o registro dirá \
                       \"editada\", porque não é mais o texto do agente." }
                </p>
                <textarea class="input propostas__editor" rows="18"
                    value={editando.as_ref().map(|(_, c)| c.clone()).unwrap_or_default()}
                    oninput={{
                        let editando = editando.clone();
                        Callback::from(move |e: InputEvent| {
                            use wasm_bindgen::JsCast;
                            let Some(a) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlTextAreaElement>().ok()) else { return };
                            if let Some((id, _)) = (*editando).clone() {
                                editando.set(Some((id, a.value())));
                            }
                        })
                    }} />
                <div class="modal__actions">
                    <button class="btn btn--primary btn--sm" onclick={aplicar_editada}>{ "Aplicar editada" }</button>
                </div>
            </Modal>
        </main>
    }
}
