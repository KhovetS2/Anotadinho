//! O construtor de diagramas mermaid (spec `construtor-mermaid`, ciclos
//! 488–490).
//!
//! A fonte da verdade enquanto o modal está aberto é o TEXTO do código.
//! A aba Construtor edita o modelo do núcleo (`anotadinho_core::diagrama`) e
//! reescreve o texto; a aba Código edita o texto direto. Por isso abrir e
//! aplicar sem mexer devolve o código exatamente como veio (RF8): ninguém
//! o regenerou.

use anotadinho_core::diagrama::{self, Diagrama, Tipo};
use yew::prelude::*;

use crate::components::modal::Modal;
use crate::mermaid::Desenho;

#[derive(Properties, PartialEq)]
pub struct ConstrutorMermaidProps {
    /// O código do bloco. Vazio = diagrama novo (`/mermaid`).
    pub inicial: String,
    pub on_aplicar: Callback<String>,
    pub on_fechar: Callback<()>,
}

#[derive(Clone, Copy, PartialEq)]
enum Aba {
    Construtor,
    Codigo,
}

#[derive(Clone, PartialEq)]
enum Previa {
    Desenhando,
    Pronta(Desenho),
    Erro(String),
}

/// Quanto esperar a pessoa parar de digitar antes de redesenhar a prévia.
const ESPERA_DA_PREVIA_MS: u32 = 250;

#[function_component(ConstrutorMermaid)]
pub fn construtor_mermaid(props: &ConstrutorMermaidProps) -> Html {
    let novo = props.inicial.trim().is_empty();
    let codigo = {
        let inicial = props.inicial.clone();
        use_state(move || if novo { diagrama::escrever(&Diagrama::exemplo(Tipo::Fluxograma)) } else { inicial })
    };
    let lido = diagrama::ler(&codigo);
    // Fora do modelo, abre no código (RF7). Dentro, no construtor.
    let aba = {
        let abre_no_construtor = novo || lido.is_ok();
        use_state(move || if abre_no_construtor { Aba::Construtor } else { Aba::Codigo })
    };
    let aba_atual = if lido.is_err() { Aba::Codigo } else { *aba };

    // Prévia: redesenha um pouco depois da última mudança. A geração
    // descarta um desenho que chegou depois de outro mais novo.
    let previa = use_state(|| Previa::Desenhando);
    let geracao = use_mut_ref(|| 0u32);
    {
        let previa = previa.clone();
        let geracao = geracao.clone();
        use_effect_with((*codigo).clone(), move |fonte| {
            let minha = {
                let mut g = geracao.borrow_mut();
                *g += 1;
                *g
            };
            let fonte = fonte.clone();
            wasm_bindgen_futures::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(ESPERA_DA_PREVIA_MS).await;
                if *geracao.borrow() != minha {
                    return;
                }
                let r = crate::mermaid::renderizar(&fonte).await;
                if *geracao.borrow() != minha {
                    return;
                }
                previa.set(match r {
                    Ok(d) => Previa::Pronta(d),
                    Err(e) => Previa::Erro(crate::mermaid::resumo_do_erro(&e)),
                });
            });
            || ()
        });
    }

    let mudar_codigo = {
        let codigo = codigo.clone();
        Callback::from(move |novo: String| codigo.set(novo))
    };
    let ao_digitar = {
        let mudar = mudar_codigo.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(t) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                mudar.emit(t.value());
            }
        })
    };
    let ir_para = |destino: Aba| {
        let aba = aba.clone();
        Callback::from(move |_: MouseEvent| aba.set(destino))
    };
    let aplicar = {
        let codigo = codigo.clone();
        let on_aplicar = props.on_aplicar.clone();
        Callback::from(move |_: MouseEvent| on_aplicar.emit((*codigo).clone()))
    };
    let fechar = props.on_fechar.clone();

    let corpo_construtor = match &lido {
        Ok(d) => html! {
            <>
                if novo {
                    <div class="construtor-mermaid__tipos" role="radiogroup" aria-label="Tipo de diagrama" data-nav-group="construtor-mermaid-tipos">
                        { for Tipo::TODOS.iter().map(|t| {
                            let t = *t;
                            let ativo = d.tipo() == t;
                            let mudar = mudar_codigo.clone();
                            html! {
                                <button type="button" role="radio" aria-checked={ativo.to_string()}
                                    class={classes!("construtor-mermaid__tipo", ativo.then_some("construtor-mermaid__tipo--ativo"))}
                                    data-nav-item={format!("construtor-mermaid-tipo-{}", t.nome())}
                                    data-nav-parent="construtor-mermaid-tipos"
                                    onclick={Callback::from(move |_: MouseEvent| mudar.emit(diagrama::escrever(&Diagrama::exemplo(t))))}>
                                    { t.nome() }
                                </button>
                            }
                        }) }
                    </div>
                }
                { editor_do_tipo(d, mudar_codigo.clone()) }
            </>
        },
        Err(_) => html! {},
    };

    let titulo = if novo { "Novo diagrama" } else { "Editar diagrama" };
    html! {
        <Modal title={titulo} open={true} on_close={fechar.clone()} classe={classes!("construtor-mermaid")}>
            <div class="construtor-mermaid__abas" role="tablist" data-nav-group="construtor-mermaid-abas">
                <button type="button" role="tab" aria-selected={(aba_atual == Aba::Construtor).to_string()}
                    class={classes!("construtor-mermaid__aba", (aba_atual == Aba::Construtor).then_some("construtor-mermaid__aba--ativa"))}
                    disabled={lido.is_err()}
                    title={lido.as_ref().err().map(|e| format!("O construtor não monta este diagrama: {}", e.mensagem()))}
                    data-nav-item="construtor-mermaid-aba-construtor" data-nav-parent="construtor-mermaid-abas"
                    onclick={ir_para(Aba::Construtor)}>
                    { "Construtor" }
                </button>
                <button type="button" role="tab" aria-selected={(aba_atual == Aba::Codigo).to_string()}
                    class={classes!("construtor-mermaid__aba", (aba_atual == Aba::Codigo).then_some("construtor-mermaid__aba--ativa"))}
                    data-nav-item="construtor-mermaid-aba-codigo" data-nav-parent="construtor-mermaid-abas"
                    onclick={ir_para(Aba::Codigo)}>
                    { "Código" }
                </button>
            </div>
            <div class="construtor-mermaid__corpo">
                <div class="construtor-mermaid__edicao">
                    if aba_atual == Aba::Construtor {
                        { corpo_construtor }
                    } else {
                        if let Err(e) = &lido {
                            <p class="construtor-mermaid__aviso" role="status">
                                { format!("O construtor não monta este diagrama — {}. Edite pelo código.", e.mensagem()) }
                            </p>
                        }
                        <textarea class="construtor-mermaid__codigo" spellcheck="false" aria-label="Código mermaid"
                            data-nav-item="construtor-mermaid-codigo"
                            value={(*codigo).clone()} oninput={ao_digitar} />
                    }
                </div>
                <div class="construtor-mermaid__previa" aria-label="Prévia" aria-live="polite">
                    { match &*previa {
                        Previa::Desenhando => html! { <p class="construtor-mermaid__previa-estado">{ "Desenhando…" }</p> },
                        Previa::Pronta(d) => html! {
                            <img class="construtor-mermaid__previa-imagem" src={d.data_url.clone()} alt="Prévia do diagrama"
                                style={format!("--mermaid-largura: {}px", d.largura)} />
                        },
                        Previa::Erro(e) => html! { <p class="construtor-mermaid__previa-erro" role="alert">{ e }</p> },
                    } }
                </div>
            </div>
            <div class="modal__actions">
                <button type="button" class="btn btn--ghost" data-nav-item="construtor-mermaid-cancelar"
                    onclick={fechar.reform(|_| ())}>{ "Cancelar" }</button>
                <button type="button" class="btn btn--primary construtor-mermaid__aplicar" data-nav-item="construtor-mermaid-aplicar"
                    onclick={aplicar}>{ if novo { "Inserir" } else { "Aplicar" } }</button>
            </div>
        </Modal>
    }
}

/// A edição visual de cada tipo. Até ela existir (489: sequência e gantt;
/// 490: fluxograma e estados), o construtor mostra o que leu e manda para
/// o código.
fn editor_do_tipo(d: &Diagrama, _mudar: Callback<String>) -> Html {
    let resumo = match d {
        Diagrama::Fluxograma(f) => format!("{} nós, {} ligações", f.nos.len(), f.ligacoes.len()),
        Diagrama::Sequencia(s) => format!("{} participantes, {} mensagens e notas", s.participantes.len(), s.itens.len()),
        Diagrama::Estados(e) => format!("{} estados, {} transições", e.estados.len(), e.transicoes.len()),
        Diagrama::Gantt(g) => format!("{} seções, {} tarefas", g.secoes.len(), g.secoes.iter().map(|s| s.tarefas.len()).sum::<usize>()),
    };
    html! {
        <p class="construtor-mermaid__resumo">
            { format!("{}: {resumo}. A edição visual deste tipo ainda não chegou — use a aba Código.", d.tipo().nome()) }
        </p>
    }
}
