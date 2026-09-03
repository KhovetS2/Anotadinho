//! Tela de aparência (ciclo 253).
//!
//! O sistema de tokens do `main.css` já era a metade difícil de um tema
//! configurável — a spec diz isso com todas as letras. O que faltava era
//! expor: até aqui, trocar qualquer coisa além de claro/escuro exigia
//! editar CSS e recompilar.
//!
//! Três escolhas independentes, porque são independentes na cabeça de
//! quem escolhe: o TEMA (o conjunto de cores), o DESTAQUE (a cor de
//! ênfase, que combina com qualquer tema) e a forma dos BOTÕES.
//!
//! Cada tema mostra uma prévia — três amostras de cor — pra dar pra
//! escolher SEM aplicar. Aplicar troca só um atributo no `<html>`: não
//! recarrega a janela, não perde trabalho não salvo (RNF3).

use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::state::{Aparencia, BOTOES, DESTAQUES, TEMAS};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct AparenciaModalProps {
    pub aparencia: Aparencia,
    pub on_change: Callback<Aparencia>,
    pub on_close: Callback<()>,
}

#[function_component(AparenciaModal)]
pub fn aparencia_modal(props: &AparenciaModalProps) -> Html {
    let atual = props.aparencia.clone();

    let trocar = {
        let on_change = props.on_change.clone();
        let atual = atual.clone();
        move |campo: &'static str, valor: String| {
            let on_change = on_change.clone();
            let atual = atual.clone();
            Callback::from(move |_: MouseEvent| {
                let mut nova = atual.clone();
                match campo {
                    "tema" => nova.tema = valor.clone(),
                    "destaque" => nova.destaque = valor.clone(),
                    _ => nova.botoes = valor.clone(),
                }
                on_change.emit(nova);
            })
        }
    };

    let restaurar = {
        let on_change = props.on_change.clone();
        Callback::from(move |_: MouseEvent| on_change.emit(Aparencia::default()))
    };

    html! {
        <Modal title="Aparência" open={true} on_close={props.on_close.reform(|_| ())}>
            <div class="aparencia" data-nav-group="aparencia">
                <section class="aparencia__secao">
                    <h4 class="aparencia__titulo">{ "Tema" }</h4>
                    <div class="aparencia__temas">
                        { for TEMAS.iter().map(|t| {
                            let escolhido = atual.tema == t.id;
                            html! {
                                <button
                                    class={classes!("aparencia__tema",
                                        escolhido.then_some("aparencia__tema--atual"))}
                                    data-tema={t.id}
                                    data-nav-item={format!("tema-{}", t.id)}
                                    data-nav-parent="aparencia"
                                    aria-pressed={escolhido.to_string()}
                                    onclick={trocar("tema", t.id.to_string())}>
                                    // A prévia: dá pra ver como fica antes
                                    // de aplicar, que é o RF2.
                                    <span class="aparencia__amostra" aria-hidden="true">
                                        { for t.amostra.iter().map(|cor| html! {
                                            <span class="aparencia__cor"
                                                style={format!("background:{cor}")} />
                                        }) }
                                    </span>
                                    <span class="aparencia__nome">{ t.nome }</span>
                                    if escolhido { <Icon name="check" /> }
                                </button>
                            }
                        }) }
                    </div>
                </section>

                <section class="aparencia__secao">
                    <h4 class="aparencia__titulo">{ "Cor de destaque" }</h4>
                    <p class="aparencia__nota">
                        { "Muda botões, foco e seleção. Vale sobre qualquer tema." }
                    </p>
                    <div class="aparencia__destaques">
                        <button
                            class={classes!("aparencia__destaque",
                                atual.destaque.is_empty().then_some("aparencia__destaque--atual"))}
                            data-destaque="tema"
                            data-nav-item="destaque-tema"
                            data-nav-parent="aparencia"
                            title="A cor que vem com o tema"
                            onclick={trocar("destaque", String::new())}>
                            { "Do tema" }
                        </button>
                        { for DESTAQUES.iter().map(|(id, nome, cor)| {
                            let escolhido = atual.destaque == *id;
                            html! {
                                <button
                                    class={classes!("aparencia__destaque",
                                        escolhido.then_some("aparencia__destaque--atual"))}
                                    data-destaque={*id}
                                    data-nav-item={format!("destaque-{id}")}
                                    data-nav-parent="aparencia"
                                    title={*nome}
                                    aria-label={*nome}
                                    style={format!("--amostra:{cor}")}
                                    onclick={trocar("destaque", id.to_string())}>
                                    <span class="aparencia__bolinha" aria-hidden="true" />
                                    { *nome }
                                </button>
                            }
                        }) }
                    </div>
                </section>

                <section class="aparencia__secao">
                    <h4 class="aparencia__titulo">{ "Botões" }</h4>
                    <div class="aparencia__botoes">
                        { for BOTOES.iter().map(|(id, nome)| {
                            let escolhido = atual.botoes == *id;
                            html! {
                                <button
                                    class={classes!("aparencia__forma", format!("aparencia__forma--{id}"),
                                        escolhido.then_some("aparencia__forma--atual"))}
                                    data-botoes={*id}
                                    data-nav-item={format!("botoes-{id}")}
                                    data-nav-parent="aparencia"
                                    aria-pressed={escolhido.to_string()}
                                    onclick={trocar("botoes", id.to_string())}>
                                    { *nome }
                                </button>
                            }
                        }) }
                    </div>
                </section>

                <div class="aparencia__rodape">
                    <button class="btn btn--ghost btn--sm"
                        data-nav-item="aparencia-padrao" data-nav-parent="aparencia"
                        onclick={restaurar}>
                        { "Voltar ao padrão" }
                    </button>
                </div>
            </div>
        </Modal>
    }
}
