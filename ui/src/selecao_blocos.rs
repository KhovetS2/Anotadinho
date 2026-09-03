//! Seleção de MÚLTIPLOS blocos (ciclo 251).
//!
//! Resolve a única pendência que o ciclo 175 deixou de propósito: com um
//! `contenteditable` por bloco, o navegador não estende seleção entre
//! blocos. Arrastar o mouse de um parágrafo até o seguinte não pega os
//! dois, `Ctrl+A` pega só um, e copiar dois parágrafos não funciona.
//!
//! A spec registra que Notion e Logseq resolvem isso reimplementando
//! seleção do zero — rastrear âncora e foco em coordenadas próprias,
//! desenhar o realce, interceptar copiar/colar. Este ciclo faz a versão
//! que a spec pediu e só ela: seleção por BLOCO INTEIRO. Seleção parcial
//! atravessando blocos (metade de um parágrafo até a metade do próximo)
//! continua fora — é ela que exigiria o motor próprio, e é não-objetivo
//! declarado.
//!
//! Por que por bloco basta: os três requisitos da spec são levar, apagar
//! e mover um CONJUNTO de blocos. Nenhum deles precisa de meio bloco.
//!
//! O estado mora no DOM (a classe e o atributo de âncora), como no
//! `nav_mode`: um estado Rust espelhando a estrutura ficaria desatualizado
//! a cada re-render, e o re-render aqui é constante.

use wasm_bindgen::JsCast;

/// Classe do bloco que faz parte da seleção múltipla.
pub const CLASSE_SELECIONADO: &str = "editor__bloco--selecionado";

/// Marca o bloco onde a seleção começou. A extensão sempre vai DELE até
/// o bloco focado — é o que faz encolher funcionar tão bem quanto
/// crescer, em vez de só acumular.
const ATTR_ANCORA: &str = "data-sel-ancora";

fn documento() -> Option<web_sys::Document> {
    web_sys::window()?.document()
}

/// Todos os blocos de TEXTO da página, em ordem de documento.
///
/// `[data-nav-block]` e não os itens do grupo de navegação: embed é
/// componente Yew, não markdown num contêiner, então não entra numa
/// seleção que existe pra ser serializada, apagada e movida.
pub fn blocos_do_documento() -> Vec<web_sys::Element> {
    let Some(doc) = documento() else {
        return Vec::new();
    };
    let Ok(lista) = doc.query_selector_all(&format!("[{}]", crate::nav_mode::ATTR_BLOCO_TEXTO))
    else {
        return Vec::new();
    };
    let mut out = Vec::with_capacity(lista.length() as usize);
    for i in 0..lista.length() {
        if let Some(el) = lista
            .item(i)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        {
            out.push(el);
        }
    }
    out
}

/// Os blocos marcados agora, em ordem de documento.
pub fn selecionados() -> Vec<web_sys::Element> {
    blocos_do_documento()
        .into_iter()
        .filter(|el| el.class_list().contains(CLASSE_SELECIONADO))
        .collect()
}

/// Existe seleção múltipla ativa? (Dois ou mais blocos, ou um bloco com
/// âncora — que é uma seleção de um item só, ainda crescendo.)
pub fn ativa() -> bool {
    ancora().is_some()
}

fn ancora() -> Option<web_sys::Element> {
    documento()?
        .query_selector(&format!("[{ATTR_ANCORA}]"))
        .ok()
        .flatten()
}

/// Desfaz a seleção inteira: marca, âncora e tudo.
pub fn limpar() {
    for el in blocos_do_documento() {
        let _ = el.class_list().remove_1(CLASSE_SELECIONADO);
        let _ = el.remove_attribute(ATTR_ANCORA);
    }
}

/// Começa uma seleção ancorada em `bloco` (ele já entra selecionado).
pub fn iniciar(bloco: &web_sys::Element) {
    limpar();
    let _ = bloco.set_attribute(ATTR_ANCORA, "1");
    let _ = bloco.class_list().add_1(CLASSE_SELECIONADO);
}

/// Marca tudo entre a âncora e `ate`, nos dois sentidos.
pub fn estender_para(ate: &web_sys::Element) {
    let blocos = blocos_do_documento();
    let Some(ancora) = ancora() else { return };
    let Some(i_ancora) = blocos.iter().position(|b| b.is_same_node(Some(&ancora))) else {
        return;
    };
    let Some(i_ate) = blocos.iter().position(|b| b.is_same_node(Some(ate))) else {
        return;
    };
    let (ini, fim) = if i_ancora <= i_ate {
        (i_ancora, i_ate)
    } else {
        (i_ate, i_ancora)
    };
    for (i, bloco) in blocos.iter().enumerate() {
        if i >= ini && i <= fim {
            let _ = bloco.class_list().add_1(CLASSE_SELECIONADO);
        } else {
            let _ = bloco.class_list().remove_1(CLASSE_SELECIONADO);
        }
    }
}

/// Move o foco um bloco pra frente/trás e estende a seleção até lá.
///
/// Ancora no bloco atual se ainda não havia seleção — é o que faz
/// `Shift+seta` começar a selecionar sem precisar de um passo antes.
/// Devolve o bloco que ficou focado.
pub fn mover_e_estender(atual: &web_sys::Element, frente: bool) -> Option<web_sys::Element> {
    if !ativa() {
        iniciar(atual);
    }
    let blocos = blocos_do_documento();
    let i = blocos.iter().position(|b| b.is_same_node(Some(atual)))?;
    let proximo = if frente {
        blocos.get(i + 1)?
    } else {
        blocos.get(i.checked_sub(1)?)?
    };
    estender_para(proximo);
    crate::nav_mode::focus_item(proximo);
    Some(proximo.clone())
}

/// O markdown dos blocos selecionados, na ordem em que aparecem.
///
/// Passa pelo MESMO serializador que todo save usa
/// (`html_to_md::html_to_markdown`), então o que se cola é o que estaria
/// no arquivo — e não uma segunda versão da regra, que divergiria.
pub fn markdown_dos_selecionados() -> String {
    let blocos = selecionados();
    if blocos.is_empty() {
        return String::new();
    }
    // `html_to_markdown` faz `trim()` no fim, então cada bloco volta sem
    // a linha em branco que o separa do próximo. Juntar com `\n\n`
    // devolve essa separação — sem ela o título grudava no parágrafo e o
    // que se colava não era markdown válido.
    blocos
        .iter()
        .map(|el| crate::html_to_md::html_to_markdown(el))
        .filter(|md| !md.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}
