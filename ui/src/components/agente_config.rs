//! Configurar como um agente é chamado (ciclo 239).
//!
//! Até aqui não havia campo nenhum: dava pra trocar de preset e escolher
//! pastas, e só. Quem precisasse apontar outro executável — ou chamar um
//! modelo que não fosse claude, codex ou opencode — tinha que editar o
//! `localStorage` na mão.
//!
//! Isto é o que o diagnóstico do Windows chama de B3, e é o item que
//! também melhora o Linux de hoje: o presets funcionarem por acaso não é
//! o mesmo que dar controle.
//!
//! A configuração mora nas PREFERÊNCIAS, nunca no vault. Uma página que
//! chegue de terceiro não pode escolher o que será executado.

use anotadinho_core::agente::{Adaptador, FormatoSaida, TIMEOUT_MINIMO_S};
use web_sys::{HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement};
use yew::prelude::*;

use crate::components::icon::Icon;
use crate::components::modal::Modal;

#[derive(Properties, PartialEq, Clone)]
pub struct AgenteConfigProps {
    pub aberto: bool,
    pub on_fechar: Callback<()>,
    /// Devolve o agente escolhido, já gravado, pra virar o ativo.
    pub on_usar: Callback<Adaptador>,
}

#[function_component(AgenteConfig)]
pub fn agente_config(props: &AgenteConfigProps) -> Html {
    // Quem está sendo editado. Começa no ativo.
    let rascunho = use_state(crate::state::load_adaptador);
    // Nome de quando o formulário abriu: renomear um agente é criar
    // outro, e o antigo precisa ser esquecido pelo nome de antes.
    let nome_original = use_state(|| crate::state::load_adaptador().nome);
    let versao = use_state(|| 0u32);

    // Reabrir tem que mostrar o estado atual, não o de duas aberturas
    // atrás.
    {
        let rascunho = rascunho.clone();
        let nome_original = nome_original.clone();
        use_effect_with(props.aberto, move |aberto| {
            if *aberto {
                let atual = crate::state::load_adaptador();
                nome_original.set(atual.nome.clone());
                rascunho.set(atual);
            }
            || ()
        });
    }

    if !props.aberto {
        return html! {};
    }

    let atual = (*rascunho).clone();
    let problema = atual.validar();
    // Aviso não impede de salvar: é a máquina de quem configura, e o que
    // dá errado ali é responsabilidade de quem escreveu (ciclo 241).
    let aviso = atual.aviso();
    let e_preset = crate::state::e_preset(&nome_original);

    let editar = |aplicar: fn(&mut Adaptador, String)| {
        let rascunho = rascunho.clone();
        Callback::from(move |valor: String| {
            let mut novo = (*rascunho).clone();
            aplicar(&mut novo, valor);
            rascunho.set(novo);
        })
    };
    let de_input = |cb: Callback<String>| {
        Callback::from(move |e: InputEvent| {
            let Some(el) = e.target_dyn_into::<HtmlInputElement>() else { return };
            cb.emit(el.value());
        })
    };

    let escolher = {
        let rascunho = rascunho.clone();
        let nome_original = nome_original.clone();
        Callback::from(move |a: Adaptador| {
            nome_original.set(a.nome.clone());
            rascunho.set(a);
        })
    };

    let novo = {
        let rascunho = rascunho.clone();
        let nome_original = nome_original.clone();
        Callback::from(move |_: MouseEvent| {
            // Nasce com o marcador já no lugar: sem `{prompt}` a
            // configuração é inválida, e um formulário que abre inválido
            // parece quebrado.
            nome_original.set(String::new());
            rascunho.set(Adaptador {
                nome: "Meu agente".to_string(),
                binario: String::new(),
                args: vec!["{prompt}".to_string()],
                timeout_s: TIMEOUT_MINIMO_S,
                ..Default::default()
            });
        })
    };

    let salvar = {
        let rascunho = rascunho.clone();
        let nome_original = nome_original.clone();
        let on_usar = props.on_usar.clone();
        Callback::from(move |_: MouseEvent| {
            let a = (*rascunho).clone();
            if a.validar().is_some() {
                return;
            }
            // Renomear é criar outro: o de antes some, senão a lista
            // ficaria com os dois e ninguém saberia qual está valendo.
            if !nome_original.is_empty() && *nome_original != a.nome {
                crate::state::remover_adaptador(&nome_original);
            }
            crate::state::save_adaptador(&a);
            nome_original.set(a.nome.clone());
            on_usar.emit(a);
        })
    };

    let remover = {
        let rascunho = rascunho.clone();
        let nome_original = nome_original.clone();
        let versao = versao.clone();
        Callback::from(move |_: MouseEvent| {
            crate::state::remover_adaptador(&nome_original);
            let volta = crate::state::load_adaptador();
            nome_original.set(volta.nome.clone());
            rascunho.set(volta);
            versao.set(*versao + 1);
        })
    };

    let opcoes = crate::state::opcoes_de_agente();
    html! {
        <Modal title="Agentes" open={true} on_close={props.on_fechar.reform(|_| ())} wide=true>
            <div class="agente-config">
                <div class="agente-config__lista">
                    { for opcoes.into_iter().map(|a| {
                        let selecionado = a.nome == *nome_original;
                        let escolher = escolher.clone();
                        let alvo = a.clone();
                        html! {
                            <button class={classes!("agente-config__item",
                                    selecionado.then_some("agente-config__item--atual"))}
                                data-nav-item="true"
                                onclick={Callback::from(move |_: MouseEvent| escolher.emit(alvo.clone()))}>
                                <Icon name="zap" />{ a.nome.clone() }
                            </button>
                        }
                    }) }
                    <button class="agente-config__item agente-config__novo" data-nav-item="true"
                        onclick={novo}>{ "+ novo agente" }</button>
                </div>

                <div class="agente-config__form">
                    <label class="agente-config__campo">
                        <span>{ "Nome" }</span>
                        <input class="input" value={atual.nome.clone()} data-nav-item="true"
                            oninput={de_input(editar(|a, v| a.nome = v))} />
                    </label>

                    <label class="agente-config__campo">
                        <span>{ "Executável" }</span>
                        <input class="input" value={atual.binario.clone()} data-nav-item="true"
                            placeholder="claude, codex, ou o caminho completo"
                            oninput={de_input(editar(|a, v| a.binario = v))} />
                        <small>
                            { "Só o executável. O que vem depois vai nos argumentos — \
                               não existe shell no meio, de propósito." }
                        </small>
                    </label>

                    <label class="agente-config__campo">
                        <span>{ "Argumentos, um por linha" }</span>
                        <textarea class="input agente-config__args" rows="6" data-nav-item="true"
                            value={atual.args.join("\n")}
                            oninput={{
                                let rascunho = rascunho.clone();
                                Callback::from(move |e: InputEvent| {
                                    let Some(el) = e.target_dyn_into::<HtmlTextAreaElement>() else { return };
                                    let mut novo = (*rascunho).clone();
                                    novo.args = el.value().lines().map(str::to_string).collect();
                                    rascunho.set(novo);
                                })
                            }} />
                        <small>
                            { "Exatamente um deles precisa conter " }
                            <code>{ "{prompt}" }</code>
                            { ". O prompt entra como UM argumento, então aspas e quebras \
                               de linha dentro dele são texto, não comando." }
                        </small>
                    </label>

                    <div class="agente-config__linha">
                        <label class="agente-config__campo">
                            <span>{ "Formato da saída" }</span>
                            <select class="input" data-nav-item="true"
                                onchange={{
                                    let rascunho = rascunho.clone();
                                    Callback::from(move |e: Event| {
                                        let Some(el) = e.target_dyn_into::<HtmlSelectElement>() else { return };
                                        let mut novo = (*rascunho).clone();
                                        novo.formato = if el.value() == "stream" {
                                            FormatoSaida::StreamJson
                                        } else {
                                            FormatoSaida::Texto
                                        };
                                        rascunho.set(novo);
                                    })
                                }}>
                                <option value="texto" selected={atual.formato == FormatoSaida::Texto}>
                                    { "Texto — a saída inteira é a resposta" }
                                </option>
                                <option value="stream" selected={atual.formato == FormatoSaida::StreamJson}>
                                    { "JSON por linha — mostra o progresso enquanto trabalha" }
                                </option>
                            </select>
                        </label>

                        <label class="agente-config__campo agente-config__campo--curto">
                            <span>{ "Tempo limite (minutos)" }</span>
                            <input class="input" type="number" min="30" data-nav-item="true"
                                value={(atual.timeout_s / 60).to_string()}
                                oninput={{
                                    let rascunho = rascunho.clone();
                                    Callback::from(move |e: InputEvent| {
                                        let Some(el) = e.target_dyn_into::<HtmlInputElement>() else { return };
                                        let minutos: u64 = el.value().parse().unwrap_or(30);
                                        let mut novo = (*rascunho).clone();
                                        novo.timeout_s = minutos.max(1) * 60;
                                        rascunho.set(novo);
                                    })
                                }} />
                        </label>
                    </div>

                    <label class="agente-config__campo">
                        <span>{ "Argumento de pasta extra" }</span>
                        <input class="input" value={atual.arg_pasta_extra.clone()} data-nav-item="true"
                            placeholder="--add-dir"
                            oninput={de_input(editar(|a, v| a.arg_pasta_extra = v))} />
                        <small>
                            { "Como este agente recebe outra pasta além da de trabalho. \
                               Vazio esconde o botão de acrescentar pasta." }
                        </small>
                    </label>

                    if let Some(p) = &problema {
                        <p class="agente-config__erro" role="alert">{ p.mensagem() }</p>
                    } else if let Some(a) = &aviso {
                        <p class="agente-config__aviso">{ a.mensagem() }</p>
                    }

                    <div class="modal__actions">
                        if !e_preset && !nome_original.is_empty() {
                            <button class="btn btn--ghost agente-config__remover" onclick={remover}>
                                { "Remover" }
                            </button>
                        }
                        <button class="btn" onclick={props.on_fechar.reform(|_| ())}>{ "Fechar" }</button>
                        <button class="btn btn--primary agente-config__salvar" onclick={salvar}
                            disabled={problema.is_some()}>{ "Salvar e usar" }</button>
                    </div>
                </div>
            </div>
        </Modal>
    }
}
