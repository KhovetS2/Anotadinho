//! Motor genérico do modo de navegação hierárquico por teclado (ciclo
//! 133) — camada fina sobre atributos `data-nav-*` do DOM, consultados
//! ao vivo a cada tecla (sem árvore Rust espelhando a estrutura da UI,
//! que ficaria desatualizada a cada página/embed novo). Um componente
//! participa só marcando seu próprio markup:
//!
//! - `data-nav-item="<id>" data-nav-parent="<group-id>"` — um item
//!   navegável (uma "parada") dentro do grupo `<group-id>`.
//! - `data-nav-group="<id>"` (opcional, junto de `data-nav-item`) —
//!   esse item TAMBÉM é um grupo: Enter desce nele, e os filhos
//!   marcam `data-nav-parent="<id>"` pra aparecer nesse nível.
//! - `data-nav-delegate="<nome>"` (opcional, junto de `data-nav-item`,
//!   em vez de `data-nav-group`) — em vez do motor genérico navegar
//!   dentro desse item, ele aciona um sistema de navegação já
//!   existente (ex: a sidebar, ciclo 106) e sai da frente; ver
//!   `app.rs` pro dispatch de cada nome de delegate.
//!
//! O nível raiz (topo da hierarquia — header/sidebar/tabbar/editor)
//! usa o id de grupo fixo `"root"`, sem elemento contêiner próprio no
//! DOM (os itens de topo têm `data-nav-parent="root"` cada um no seu
//! próprio elemento raiz, não precisam de um wrapper comum).

use wasm_bindgen::JsCast;

/// Itens (`[data-nav-item]`) cujo `data-nav-parent` é exatamente
/// `group_id` — os "irmãos" navegáveis por seta no nível atual, na
/// ordem em que aparecem no documento.
pub fn items_in_group(document: &web_sys::Document, group_id: &str) -> Vec<web_sys::Element> {
    let selector = format!(
        "[data-nav-item][data-nav-parent=\"{}\"]",
        escape_attr_value(group_id)
    );
    let Ok(list) = document.query_selector_all(&selector) else { return Vec::new() };
    let mut out = Vec::with_capacity(list.length() as usize);
    for i in 0..list.length() {
        if let Some(node) = list.item(i) {
            if let Ok(el) = node.dyn_into::<web_sys::Element>() {
                out.push(el);
            }
        }
    }
    out
}

/// Índice de `active` dentro de `items`, comparando por identidade de
/// nó (mesmo critério do trap de foco do `Modal`) — `None` se `active`
/// não for nenhum deles (ex: acabou de entrar no nível, nada focado
/// ainda).
pub fn index_of(items: &[web_sys::Element], active: Option<&web_sys::Element>) -> Option<usize> {
    let active = active?;
    items.iter().position(|el| active.is_same_node(Some(el)))
}

/// Foca um item e rola ele pra dentro da área visível (`Nearest` —
/// mesmo critério já usado por sidebar/paleta/vim mode, rola o mínimo
/// necessário em vez de centralizar à toa a cada tecla).
pub fn focus_item(el: &web_sys::Element) {
    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html_el.focus();
    }
    let opts = web_sys::ScrollIntoViewOptions::new();
    opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
    el.scroll_into_view_with_scroll_into_view_options(&opts);
}

/// Nome do sistema de navegação bespoke que deve assumir o teclado a
/// partir desse item (`None` = não é um delegate).
pub fn delegate_of(el: &web_sys::Element) -> Option<String> {
    el.get_attribute("data-nav-delegate")
}

/// Id do grupo que esse item também é (`None` = é uma folha — Enter
/// ativa em vez de descer).
pub fn group_of(el: &web_sys::Element) -> Option<String> {
    el.get_attribute("data-nav-group")
}

/// Escapa aspas duplas num valor de atributo antes de compor um
/// seletor CSS — os ids usados no app são todos literais fixos
/// (`"header"`, `"sidebar"`, etc., nunca texto livre do usuário), mas
/// blindar aqui é trivial e evita um seletor quebrado se algum id
/// futuro tiver aspas por engano.
fn escape_attr_value(value: &str) -> String {
    value.replace('"', "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_attr_value_strips_double_quotes() {
        assert_eq!(escape_attr_value(r#"a"b"#), "ab");
        assert_eq!(escape_attr_value("plain"), "plain");
    }
}
