//! Renderiza o conteúdo de uma página como ela FICA, sem poder editar
//! (ciclo 230).
//!
//! Existe porque a tela de revisão mostrava só o diff em texto cru. Para
//! quem escreveu o embed isso basta; para quem só quer decidir, um
//! `{{ type: "query" }}` com dez linhas de YAML é ilegível — e decidir é
//! justamente o que aquela tela existe para permitir.
//!
//! Desde o ciclo 452 a árvore é a MESMA do editor
//! (`.editor__wysiwyg-segments` > `.editor__wysiwyg`) e o markdown passa
//! pelo mesmo pipeline (`editor::preparar_leitura`). Antes era um
//! renderizador ao lado, e "como fica" divergia da página de verdade em
//! tudo que o editor faz depois do render: espaçamento, recuo de lista,
//! imagem do vault, transclusão.
//!
//! Somente leitura de verdade: todo callback de mutação é `noop`, e
//! nenhum bloco é `contenteditable`. Nada aqui pode escrever no vault,
//! nem sem querer.

use yew::prelude::*;

use crate::api::PageMeta;
use crate::components::embeds::InlineEmbed;
use crate::embed::DocSegment;

#[derive(Properties, PartialEq, Clone)]
pub struct PaginaPreviewProps {
    /// Conteúdo completo da página, com frontmatter.
    pub conteudo: String,
    pub vault_path: String,
    /// Caminho da página, usado pelos embeds que precisam saber onde vivem.
    #[prop_or_default]
    pub page_path: String,
    /// Prefixo do grupo de navegação, para dois previews na mesma tela
    /// não compartilharem grupo.
    #[prop_or_default]
    pub nav_prefixo: String,
}

#[function_component(PaginaPreview)]
pub fn pagina_preview(props: &PaginaPreviewProps) -> Html {
    let raiz = use_node_ref();
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&props.conteudo);
    let segmentos = crate::embed::segment(corpo);

    {
        let raiz = raiz.clone();
        let segmentos = segmentos.clone();
        let vault = props.vault_path.clone();
        let pagina = props.page_path.clone();
        // O markdown entra por `set_inner_html`, como no editor: o Yew não
        // tem filhos nesses divs, então não briga com o conteúdo
        // imperativo. Roda de novo quando o conteúdo muda — trocar um
        // trecho na revisão muda o que a Visualização mostra.
        use_effect_with((props.conteudo.clone(), vault.clone(), pagina.clone()), move |_| {
            if let Some(raiz) = raiz.cast::<web_sys::Element>() {
                for (i, seg) in segmentos.iter().enumerate() {
                    let DocSegment::Markdown(texto) = seg else { continue };
                    let seletor = format!("[data-preview-md=\"{i}\"]");
                    if let Ok(Some(div)) = raiz.query_selector(&seletor) {
                        crate::components::editor::preparar_leitura(&div, texto, vault.clone(), pagina.clone());
                    }
                }
            }
            || ()
        });
    }

    html! {
        <div class="pagina-preview editor__wysiwyg-segments" ref={raiz}>
            { for segmentos.iter().enumerate().map(|(i, seg)| {
                // A chave leva o TIPO e a contagem, como no editor (ciclo
                // 079): um trecho de markdown que vira embed no mesmo
                // índice tem que desmontar o div, senão o conteúdo
                // imperativo de antes fica grudado ao lado do novo.
                let tipo = match seg { DocSegment::Markdown(_) => "md", DocSegment::Embed(_) => "embed" };
                let key = format!("{i}-{tipo}-{}", segmentos.len());
                match seg {
                    DocSegment::Markdown(_) => html! {
                        <div class="editor__wysiwyg" {key} data-preview-md={i.to_string()} contenteditable="false" />
                    },
                    DocSegment::Embed(dados) => html! {
                        <div class="pagina-preview__embed" {key}>
                            <InlineEmbed
                                data={dados.clone()}
                                vault_path={props.vault_path.clone()}
                                page_path={props.page_path.clone()}
                                nav_group={format!("{}-preview-{i}", props.nav_prefixo)}
                                on_change={Callback::noop()}
                                open_dialog={Callback::noop()}
                                on_page_selected={Callback::<PageMeta>::noop()} />
                        </div>
                    },
                }
            }) }
        </div>
    }
}
