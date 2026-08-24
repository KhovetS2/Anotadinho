//! Embed `{{ type: "fluxo" }}` — a etapa do artefato (ciclo 201).
//!
//! Desenha onde a página está no fluxo spec → proposta → execução, e é
//! por aqui que o estado avança.
//!
//! A regra que dá segurança ao acoplamento com agentes está no core
//! (`fluxo::Etapa::proximas`): **nenhuma transição acontece sozinha**.
//! Um agente pode preparar o conteúdo; quem clica é quem lê. E o botão
//! só existe pra transição que a máquina de estados permite — não dá
//! pra pular a revisão nem por engano da UI.
//!
//! Ao avançar, espelha o valor em `status:` do frontmatter da própria
//! página, que é o campo pelo qual as consultas filtram. É o mesmo
//! caminho do `anotadinho-cli set-property` (via
//! `MarkdownCodec::set_frontmatter_field`), pra não haver dois jeitos
//! de escrever a mesma coisa.

use crate::components::icon::Icon;
use anotadinho_core::embed::{EmbedData, FluxoEmbedData};
use anotadinho_core::fluxo::{self, Artefato, Etapa};
use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct InlineFluxoProps {
    pub data: FluxoEmbedData,
    pub vault_path: String,
    pub on_change: Callback<EmbedData>,
    /// Grava um campo no frontmatter da página — o `status` que as
    /// consultas filtram. Vem do EDITOR de propósito: o embed gravando
    /// direto no disco criava dois escritores pro mesmo arquivo, e o
    /// que fosse gravado por último apagava o outro (ciclo 201).
    pub on_set_property: Callback<(String, String)>,
    /// Abre uma conversa de PLANEJAMENTO a partir desta spec aprovada
    /// (ciclo 209): a spec vai anexada, e a pergunta já pede a proposta
    /// de implementação.
    #[prop_or_default]
    pub on_planejar: Callback<fluxo::Pedido>,
    /// Página onde este embed vive.
    #[prop_or_default]
    pub page_path: String,
    pub on_page_selected: Callback<crate::api::PageMeta>,
    pub nav_group: String,
}

#[function_component(InlineFluxo)]
pub fn inline_fluxo(props: &InlineFluxoProps) -> Html {
    let data = props.data.clone();
    let etapa = data.etapa;

    let ir_para = {
        let data = data.clone();
        let on_change = props.on_change.clone();
        let on_set_property = props.on_set_property.clone();
        Callback::from(move |destino: Etapa| {
            let mut nova = data.clone();
            if !nova.ir_para(destino, None) {
                return;
            }
            on_change.emit(EmbedData::Fluxo(nova));
            // Espelha no `status` do frontmatter, que é o campo pelo
            // qual as consultas filtram. Pelo editor, não pelo disco.
            on_set_property.emit(("status".to_string(), destino.slug().to_string()));
        })
    };

    let planejar = {
        let on_planejar = props.on_planejar.clone();
        Callback::from(move |_: MouseEvent| on_planejar.emit(fluxo::Pedido::Avancar))
    };
    let alterar = {
        let on_planejar = props.on_planejar.clone();
        Callback::from(move |_: MouseEvent| on_planejar.emit(fluxo::Pedido::Alterar))
    };

    let abrir_origem = {
        let origem = data.origem.clone();
        let on_page_selected = props.on_page_selected.clone();
        Callback::from(move |_: MouseEvent| {
            let Some(path) = origem.clone() else { return };
            let title = std::path::Path::new(&path)
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.clone());
            on_page_selected.emit(crate::api::PageMeta {
                path,
                title,
                section: "pages".to_string(),
            });
        })
    };

    // As etapas do caminho feliz, em ordem — as de exceção (bloqueada)
    // não entram na trilha, senão a leitura vira um grafo.
    let trilha = [
        Etapa::Rascunho,
        Etapa::EmRevisao,
        Etapa::Aprovada,
        Etapa::EmExecucao,
        Etapa::Concluida,
    ];
    let indice_atual = trilha.iter().position(|e| *e == etapa);

    html! {
        <div class="fluxo" data-nav-group={props.nav_group.clone()} data-nav-item={props.nav_group.clone()}
            data-nav-parent={crate::nav_mode::GRUPO_BLOCOS} tabindex="-1">
            <div class="fluxo__topo">
                <span class="fluxo__artefato">{ data.artefato.label() }</span>
                <span class={classes!("fluxo__etapa", format!("fluxo__etapa--{}", etapa.slug()))}>
                    { etapa.label() }
                </span>
                if let Some(origem) = &data.origem {
                    <button class="fluxo__origem btn btn--ghost btn--xs" onclick={abrir_origem}
                        title={format!("Abrir {origem}")}>
                        <Icon name="link" />{ "origem" }
                    </button>
                }
            </div>

            <ol class="fluxo__trilha">
                { for trilha.iter().enumerate().map(|(i, e)| {
                    let estado = match indice_atual {
                        Some(atual) if i < atual => "feita",
                        Some(atual) if i == atual => "atual",
                        _ => "futura",
                    };
                    html! {
                        <li class={classes!("fluxo__passo", format!("fluxo__passo--{estado}"))}>
                            { e.label() }
                        </li>
                    }
                }) }
            </ol>

            // Uma spec APROVADA é o ponto onde o trabalho passa do "o
            // quê" pro "como" (ciclo 209). O botão leva pra conversa de
            // planejamento com a spec anexada — e é lá que se anexam os
            // padrões que a implementação deve respeitar.
            if etapa == Etapa::Aprovada
                && matches!(data.artefato, Artefato::Spec | Artefato::Proposta)
            {
                <div class="fluxo__planejar">
                    <button class="btn btn--primary btn--sm" onclick={planejar}>
                        <Icon name="zap" />
                        { if data.artefato == Artefato::Spec {
                            "Planejar implementação"
                        } else {
                            "Executar"
                        } }
                    </button>
                    <span class="fluxo__planejar-dica">
                        { if data.artefato == Artefato::Spec {
                            "Abre uma conversa com esta spec anexada. Anexe também os padrões que a proposta deve seguir."
                        } else {
                            "Abre uma conversa com esta proposta anexada. A abordagem já foi aceita — o que sair daqui vira o registro de execução."
                        } }
                    </span>
                </div>
            }

            // Revisar só tinha duas saídas — aprovar ou mandar pra
            // trás. A terceira, "está quase, muda estes pontos", virava
            // copiar e colar na mão (ciclo 223).
            if etapa == Etapa::EmRevisao
                && matches!(data.artefato, Artefato::Spec | Artefato::Proposta)
            {
                <div class="fluxo__planejar">
                    <button class="btn btn--sm" onclick={alterar}>
                        <Icon name="edit" />
                        { "Pedir alteração" }
                    </button>
                    <span class="fluxo__planejar-dica">
                        { "Abre a conversa com esta página anexada. O agente devolve a mudança como proposta, pra você ver o diff antes de aplicar." }
                    </span>
                </div>
            }

            <div class="fluxo__acoes">
                { for etapa.proximas().into_iter().map(|destino| {
                    let ir = ir_para.clone();
                    let primario = etapa.avanco_natural() == Some(destino);
                    let onclick = Callback::from(move |_: MouseEvent| ir.emit(destino));
                    html! {
                        <button class={classes!("btn", "btn--sm",
                            if primario { "btn--primary" } else { "btn--ghost" })}
                            {onclick}
                            title={format!("Mover pra {}", destino.label())}>
                            { destino.label() }
                        </button>
                    }
                }) }
            </div>

            if let Some(nota) = &data.nota {
                <p class="fluxo__nota">{ nota }</p>
            }

            if !etapa.agente_pode_preparar() {
                <p class="fluxo__aviso">
                    { "Etapa fechada pra edição automática: um agente não altera o conteúdo daqui em diante." }
                </p>
            }
        </div>
    }
}
