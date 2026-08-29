//! Renderiza o conteúdo de uma página como ela FICA, sem poder editar
//! (ciclo 230).
//!
//! Existe porque a tela de revisão mostrava só o diff em texto cru. Para
//! quem escreveu o embed isso basta; para quem só quer decidir, um
//! `{{ type: "query" }}` com dez linhas de YAML é ilegível — e decidir é
//! justamente o que aquela tela existe para permitir.
//!
//! Somente leitura de verdade: todo callback de mutação é `noop`. Nada
//! aqui pode escrever no vault, nem sem querer.

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
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&props.conteudo);
    let segmentos = crate::embed::segment(corpo);

    html! {
        <div class="pagina-preview">
            { for segmentos.iter().enumerate().map(|(i, seg)| match seg {
                DocSegment::Markdown(texto) => {
                    let html = crate::markdown_render::render(texto);
                    html! {
                        <div class="pagina-preview__md">
                            { Html::from_html_unchecked(AttrValue::from(html)) }
                        </div>
                    }
                }
                DocSegment::Embed(dados) => html! {
                    <div class="pagina-preview__embed">
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
            }) }
        </div>
    }
}
