//! Peças comuns aos editores do construtor: as ações de uma linha de lista
//! e os leitores de valor dos campos.

use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::components::date_picker::DatePicker;
use crate::components::icon::Icon;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AcaoLinha {
    Subir,
    Descer,
    Apagar,
}

/// Aplica a ação na lista. Fora dos limites, não faz nada.
pub fn aplicar<T>(v: &mut Vec<T>, i: usize, a: AcaoLinha) {
    match a {
        AcaoLinha::Subir if i > 0 && i < v.len() => v.swap(i, i - 1),
        AcaoLinha::Descer if i + 1 < v.len() => v.swap(i, i + 1),
        AcaoLinha::Apagar if i < v.len() => {
            v.remove(i);
        }
        _ => {}
    }
}

/// Subir, descer e apagar. `bloqueio` desabilita o apagar e diz por quê.
pub fn botoes(grupo: &str, i: usize, n: usize, bloqueio: Option<String>, on: Callback<AcaoLinha>) -> Html {
    let item = |acao: &str| format!("{grupo}-{i}-{acao}");
    html! {
        <div class="construtor-mermaid__linha-acoes">
            <button type="button" class="btn btn--ghost btn--icon btn--xs" title="Subir" aria-label="Subir"
                disabled={i == 0} data-nav-item={item("subir")} data-nav-parent={grupo.to_string()}
                onclick={on.reform(|_| AcaoLinha::Subir)}><Icon name="arrow-up" /></button>
            <button type="button" class="btn btn--ghost btn--icon btn--xs" title="Descer" aria-label="Descer"
                disabled={i + 1 >= n} data-nav-item={item("descer")} data-nav-parent={grupo.to_string()}
                onclick={on.reform(|_| AcaoLinha::Descer)}><Icon name="arrow-down" /></button>
            <button type="button" class="btn btn--ghost btn--icon btn--xs construtor-mermaid__apagar"
                title={bloqueio.clone().unwrap_or_else(|| "Apagar".into())} aria-label="Apagar"
                disabled={bloqueio.is_some()} data-nav-item={item("apagar")} data-nav-parent={grupo.to_string()}
                onclick={on.reform(|_| AcaoLinha::Apagar)}><Icon name="x" /></button>
        </div>
    }
}

/// Um callback que aplica `f` numa cópia do modelo e emite a cópia.
pub fn com<M: Clone + 'static, V: 'static>(m: &M, on: &Callback<M>, f: impl Fn(&mut M, V) + 'static) -> Callback<V> {
    let m = m.clone();
    let on = on.clone();
    Callback::from(move |v: V| {
        let mut novo = m.clone();
        f(&mut novo, v);
        on.emit(novo);
    })
}

pub fn texto_do_input(e: InputEvent) -> String {
    e.target_dyn_into::<web_sys::HtmlInputElement>().map(|t| t.value()).unwrap_or_default()
}

pub fn valor_do_select(e: Event) -> String {
    e.target().and_then(|t| t.dyn_into::<web_sys::HtmlSelectElement>().ok()).map(|s| s.value()).unwrap_or_default()
}

pub fn marcado(e: Event) -> bool {
    e.target_dyn_into::<web_sys::HtmlInputElement>().map(|t| t.checked()).unwrap_or(false)
}

#[derive(Properties, PartialEq)]
pub struct CampoDataProps {
    pub valor: Option<String>,
    pub on_escolher: Callback<String>,
    pub nav: String,
    pub grupo: String,
}

/// Uma data pelo seletor do app (o mesmo do calendário): só sai data
/// válida, então o diagrama nunca fica ilegível no meio da escolha.
#[function_component(CampoData)]
pub fn campo_data(props: &CampoDataProps) -> Html {
    let aberto = use_state(|| false);
    let alternar = {
        let aberto = aberto.clone();
        Callback::from(move |_: MouseEvent| aberto.set(!*aberto))
    };
    let fechar = {
        let aberto = aberto.clone();
        Callback::from(move |_: ()| aberto.set(false))
    };
    let escolher = {
        let aberto = aberto.clone();
        let on = props.on_escolher.clone();
        Callback::from(move |d: String| {
            aberto.set(false);
            on.emit(d);
        })
    };
    html! {
        <span class="construtor-mermaid__data">
            <button type="button" class="btn btn--secondary btn--xs" data-nav-item={props.nav.clone()} data-nav-parent={props.grupo.clone()}
                onclick={alternar}>
                { props.valor.clone().unwrap_or_else(|| "escolher data".into()) }
            </button>
            if *aberto {
                <DatePicker value={props.valor.clone()} on_pick={escolher} on_close={fechar} />
            }
        </span>
    }
}
