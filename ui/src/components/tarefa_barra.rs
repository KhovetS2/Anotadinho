//! A barra de etapa de uma página `type: tarefa` (ciclo 460).
//!
//! Fica EM CIMA do editor de sempre: a tarefa é texto que a pessoa
//! escreve (objetivo, critérios, abordagem), e a etapa é um campo do
//! frontmatter. Mover a etapa daqui pede ao `PageView` pra recarregar o
//! editor: ele guarda o frontmatter da abertura, e um salvamento depois
//! da mudança gravaria o `status` velho por cima.

use anotadinho_core::projeto::{self, EtapaTarefa, Quem, Tarefa};
use yew::prelude::*;

use crate::api;
use crate::components::conversa_view::PerguntaInicial;
use crate::components::icon::Icon;

#[derive(Properties, PartialEq, Clone)]
pub struct TarefaBarraProps {
    pub vault_path: String,
    pub page: api::PageMeta,
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
    /// Abre uma conversa com a pergunta escrita, sem enviar.
    #[prop_or_default]
    pub on_perguntar: Callback<PerguntaInicial>,
    /// A etapa mudou: o editor abaixo precisa reler a página.
    #[prop_or_default]
    pub on_mudou: Callback<()>,
}

#[function_component(TarefaBarra)]
pub fn tarefa_barra(props: &TarefaBarraProps) -> Html {
    let tarefa = use_state(|| None::<Tarefa>);
    let erro = use_state(|| None::<String>);
    {
        let (tarefa, erro) = (tarefa.clone(), erro.clone());
        let (vault_path, path) = (props.vault_path.clone(), props.page.path.clone());
        use_effect_with(props.page.path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api::read_page(&vault_path, &path).await {
                    Ok(texto) => tarefa.set(Tarefa::ler(&path, &texto)),
                    Err(e) => erro.set(Some(e)),
                }
            });
            || ()
        });
    }
    let Some(t) = (*tarefa).clone() else {
        return html! {};
    };

    let mover = {
        let (vault_path, caminho, erro, on_mudou) =
            (props.vault_path.clone(), t.caminho.clone(), erro.clone(), props.on_mudou.clone());
        Callback::from(move |destino: EtapaTarefa| {
            let (vault_path, caminho, erro, on_mudou) =
                (vault_path.clone(), caminho.clone(), erro.clone(), on_mudou.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::mudar_etapa_tarefa(&vault_path, &caminho, destino).await {
                    Ok(_) => on_mudou.emit(()),
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    // As duas ações de conversa vão pra MESMA página — a conversa da
    // tarefa, onde a execução também fica registrada —, uma com a
    // pergunta de detalhar escrita e a outra sem nada.
    let conversar = {
        let (vault_path, t, erro) = (props.vault_path.clone(), t.clone(), erro.clone());
        let (on_page_selected, on_perguntar) = (props.on_page_selected.clone(), props.on_perguntar.clone());
        Callback::from(move |detalhar: bool| {
            let (vault_path, t, erro) = (vault_path.clone(), t.clone(), erro.clone());
            let (on_page_selected, on_perguntar) = (on_page_selected.clone(), on_perguntar.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match api::conversa_da_tarefa(&vault_path, &t.caminho).await {
                    Ok(conversa) if detalhar => on_perguntar.emit(PerguntaInicial {
                        conversa,
                        texto: projeto::pergunta_de_detalhar(&t),
                    }),
                    Ok(conversa) => on_page_selected.emit(api::PageMeta {
                        title: format!("Tarefa {:03}", t.numero),
                        path: conversa,
                        section: "pages".to_string(),
                    }),
                    Err(e) => erro.set(Some(e)),
                }
            });
        })
    };

    let abrir_projeto = {
        let on_page_selected = props.on_page_selected.clone();
        let slug = t.projeto.clone();
        Callback::from(move |_: MouseEvent| {
            on_page_selected.emit(api::PageMeta {
                path: projeto::caminho_do_projeto(&slug),
                title: slug.clone(),
                section: "pages".to_string(),
            })
        })
    };

    let classe_etapa = format!("tarefa-barra__etapa tarefa-barra__etapa--{}", t.etapa.slug());
    html! {
        <div class="tarefa-barra" data-nav-group="tarefa-barra">
            <button class="tarefa-barra__projeto" data-nav-item="true" data-nav-parent="tarefa-barra"
                onclick={abrir_projeto} title="Abrir o quadro do projeto">
                { &t.projeto }
            </button>
            <span class={classe_etapa}>{ t.etapa.label() }</span>
            <div class="tarefa-barra__mover">
                { for t.etapa.proximas_para(Quem::Humano).into_iter().map(|destino| {
                    let mover = mover.clone();
                    html! {
                        <button class="btn btn--ghost btn--xs tarefa-barra__destino" data-destino={destino.slug()}
                            data-nav-item="true" data-nav-parent="tarefa-barra"
                            onclick={Callback::from(move |_: MouseEvent| mover.emit(destino))}>
                            { format!("→ {}", destino.label()) }
                        </button>
                    }
                }) }
            </div>
            <div class="tarefa-barra__conversa">
                if matches!(t.etapa, EtapaTarefa::Ideia | EtapaTarefa::Detalhando | EtapaTarefa::Bloqueada) {
                    <button class="btn btn--ghost btn--xs tarefa-barra__detalhar" data-nav-item="true"
                        data-nav-parent="tarefa-barra" title="Abre a conversa da tarefa com o pedido de detalhar escrito"
                        onclick={{ let c = conversar.clone(); Callback::from(move |_: MouseEvent| c.emit(true)) }}>
                        <Icon name="message-circle" />{ " Detalhar com o agente" }
                    </button>
                }
                <button class="btn btn--ghost btn--xs tarefa-barra__abrir-conversa" data-nav-item="true"
                    data-nav-parent="tarefa-barra" title="A conversa onde o detalhe e as execuções desta tarefa ficam"
                    onclick={{ let c = conversar.clone(); Callback::from(move |_: MouseEvent| c.emit(false)) }}>
                    { "Conversa da tarefa" }
                </button>
            </div>
            if let Some(e) = (*erro).clone() {
                <p class="tarefa-barra__erro">{ e }</p>
            }
        </div>
    }
}
