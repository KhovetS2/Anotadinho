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

/// Classe do indicador de item focado do nav-mode — ver `focus_item`.
const ITEM_ACTIVE_CLASS: &str = "nav-mode__item-active";

/// Foca um item e rola ele pra dentro da área visível (`Nearest` —
/// mesmo critério já usado por sidebar/paleta/vim mode, rola o mínimo
/// necessário em vez de centralizar à toa a cada tecla). Tenta
/// `HtmlElement` primeiro (a grande maioria) E `SvgElement` (nós do
/// grafo, ciclo 126, são `<g>` — SVG e HTML são ramos SEPARADOS da
/// hierarquia de elementos do DOM; `dyn_ref::<HtmlElement>()` falha
/// silenciosamente pra SVG, então sem esse segundo braço o `.focus()`
/// simplesmente não acontecia pros nós do grafo — achado ao vivo na
/// validação deste ciclo).
///
/// Também marca o item com `ITEM_ACTIVE_CLASS` (ciclo 139, pedido do
/// usuário) — o `:focus-visible` genérico (ciclo 123, só contorno)
/// nem sempre é visível o bastante em itens grandes/com pouco
/// contraste de fundo (ex: o `<header>` inteiro como item de nível
/// raiz); a classe soma um fundo + borda interna mais robustos contra
/// isso, na mesma cor do nível atual (`--nav-mode-depth-color`,
/// ciclo 136). Limpa a marca do item anterior antes — mesmo padrão de
/// "consultar e substituir" já usado pro destaque de região.
pub fn focus_item(el: &web_sys::Element) {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Ok(stale) = doc.query_selector_all(&format!(".{}", ITEM_ACTIVE_CLASS)) {
            for i in 0..stale.length() {
                if let Some(stale_el) = stale.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
                    let _ = stale_el.class_list().remove_1(ITEM_ACTIVE_CLASS);
                }
            }
        }
    }
    let _ = el.class_list().add_1(ITEM_ACTIVE_CLASS);

    if let Some(html_el) = el.dyn_ref::<web_sys::HtmlElement>() {
        let _ = html_el.focus();
    } else if let Some(svg_el) = el.dyn_ref::<web_sys::SvgElement>() {
        let _ = svg_el.focus();
    }
    let opts = web_sys::ScrollIntoViewOptions::new();
    opts.set_block(web_sys::ScrollLogicalPosition::Nearest);
    el.scroll_into_view_with_scroll_into_view_options(&opts);
}

/// Remove `ITEM_ACTIVE_CLASS` de qualquer elemento que a tenha — usado
/// quando a sessão inteira do nav-mode termina (delegate ou saída via
/// Escape), pra não deixar o indicador "preso" num item depois que o
/// foco já foi pra outro lugar que `focus_item` não gerencia mais.
pub fn clear_item_highlight() {
    if let Some(doc) = web_sys::window().and_then(|w| w.document()) {
        if let Ok(stale) = doc.query_selector_all(&format!(".{}", ITEM_ACTIVE_CLASS)) {
            for i in 0..stale.length() {
                if let Some(stale_el) = stale.item(i).and_then(|n| n.dyn_into::<web_sys::Element>().ok()) {
                    let _ = stale_el.class_list().remove_1(ITEM_ACTIVE_CLASS);
                }
            }
        }
    }
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

/// Cor do nível atual da sessão (ciclo 136, pedido do usuário) — um
/// gradiente azul→roxo (as duas cores de destaque já usadas em outros
/// lugares do app, ex. o logo) conforme a pilha fica mais funda, pra
/// dar uma pista visual de profundidade além do texto do badge.
/// `depth` é `nav_stack.len()` (0 = raiz, sem cor própria — o
/// indicador só aparece quando HÁ um grupo atual). Nível 1 = azul
/// puro, nível 5+ satura em roxo puro; os do meio interpolam via
/// `color-mix()` do CSS (suportado pelo WebKitGTK do Tauri, já usado
/// em outras regras deste app).
pub fn depth_color_css(depth: usize) -> String {
    let blue_pct = 100i64.saturating_sub(((depth.max(1) - 1) as i64) * 25).clamp(0, 100);
    if blue_pct >= 100 {
        return "var(--accent-blue)".to_string();
    }
    if blue_pct <= 0 {
        return "var(--accent-purple)".to_string();
    }
    format!(
        "color-mix(in srgb, var(--accent-blue) {}%, var(--accent-purple) {}%)",
        blue_pct, 100 - blue_pct
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escape_attr_value_strips_double_quotes() {
        assert_eq!(escape_attr_value(r#"a"b"#), "ab");
        assert_eq!(escape_attr_value("plain"), "plain");
    }

    #[test]
    fn depth_color_css_pure_blue_at_shallowest() {
        assert_eq!(depth_color_css(1), "var(--accent-blue)");
        // depth 0 não deveria acontecer na prática (nav-mode só marca
        // profundidade quando HÁ pilha), mas trata como nível 1 em vez
        // de estourar por causa do `.max(1)`.
        assert_eq!(depth_color_css(0), "var(--accent-blue)");
    }

    #[test]
    fn depth_color_css_saturates_purple_at_depth_five_and_beyond() {
        assert_eq!(depth_color_css(5), "var(--accent-purple)");
        assert_eq!(depth_color_css(9), "var(--accent-purple)");
    }

    #[test]
    fn depth_color_css_interpolates_between() {
        assert_eq!(
            depth_color_css(2),
            "color-mix(in srgb, var(--accent-blue) 75%, var(--accent-purple) 25%)"
        );
        assert_eq!(
            depth_color_css(3),
            "color-mix(in srgb, var(--accent-blue) 50%, var(--accent-purple) 50%)"
        );
        assert_eq!(
            depth_color_css(4),
            "color-mix(in srgb, var(--accent-blue) 25%, var(--accent-purple) 75%)"
        );
    }
}
