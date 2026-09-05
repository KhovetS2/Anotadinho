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

/// Grupo de navegação do CONTEÚDO da página (ciclo 174): cada
/// parágrafo, título, lista, citação, bloco de código e cada embed é um
/// item deste grupo. Antes disso o editor era um `data-nav-delegate` —
/// entrar nele jogava o foco no texto e o motor de setas saía de cena,
/// então não havia destaque de região nenhum lá dentro.
pub const GRUPO_BLOCOS: &str = "editor-blocos";

/// Marca que distingue um bloco de TEXTO (onde Enter entra em edição)
/// de um bloco de EMBED (onde Enter desce pros controles).
pub const ATTR_BLOCO_TEXTO: &str = "data-nav-block";

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
                if esta_visivel(&el) {
                    out.push(el);
                }
            }
        }
    }
    out
}

/// Se o elemento ocupa espaço na tela.
///
/// Item escondido com `display: none` (os controles que só aparecem no
/// hover — configurar/remover botão, barra da galeria, toolbar do
/// embed) continua casando com o seletor, mas `focus()` nele não faz
/// nada: a navegação parava naquele índice e as setas ficavam mudas.
///
/// `opacity: 0` NÃO é filtrado de propósito: esses ficam visíveis assim
/// que recebem foco (`:focus-within`), então são alvos legítimos.
fn esta_visivel(el: &web_sys::Element) -> bool {
    let rect = el.get_bounding_client_rect();
    rect.width() > 0.0 || rect.height() > 0.0
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
/// Apaga o realce de item ativo, onde quer que ele esteja.
///
/// Extraído de `focus_item` porque quem SAI de um bloco realçado (o
/// cursor indo de um embed pra um parágrafo) precisa limpar sem acender
/// nada — e sem isto o embed ficava aceso depois de o cursor já ter ido.
pub fn limpar_item_ativo() {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let Ok(antigos) = doc.query_selector_all(&format!(".{}", ITEM_ACTIVE_CLASS)) else {
        return;
    };
    for i in 0..antigos.length() {
        if let Some(el) = antigos
            .item(i)
            .and_then(|n| n.dyn_into::<web_sys::Element>().ok())
        {
            let _ = el.class_list().remove_1(ITEM_ACTIVE_CLASS);
        }
    }
}

pub fn focus_item(el: &web_sys::Element) {
    limpar_item_ativo();
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

// ── alvo de busca (ciclo 188) ────────────────────────────────────────

thread_local! {
    /// Registro que a busca deixa pra o editor ler quando terminar de
    /// renderizar a página.
    ///
    /// Um `thread_local` e não uma prop porque quem grava (sidebar,
    /// paleta) e quem lê (editor) não têm relação de pai/filho, e o
    /// valor precisa sobreviver ao intervalo entre o clique e o fim do
    /// carregamento da página — que é assíncrono e passa por vários
    /// re-renders. WASM é single-thread, então não há disputa aqui.
    static ALVO_DE_BUSCA: std::cell::RefCell<Option<String>> =
        const { std::cell::RefCell::new(None) };
}

/// Guarda a âncora (`"<segmento>:<registro>"`) do resultado clicado.
/// `None` limpa — resultado de texto solto não tem pra onde levar.
pub fn marcar_alvo_de_busca(ancora: Option<&str>) {
    ALVO_DE_BUSCA.with(|a| *a.borrow_mut() = ancora.map(str::to_string));
}

/// Consome o alvo pendente. Consumir (e não só ler) é de propósito: o
/// destaque acontece UMA vez, não a cada re-render da mesma página.
pub fn tomar_alvo_de_busca() -> Option<String> {
    ALVO_DE_BUSCA.with(|a| a.borrow_mut().take())
}

/// Rola até o embed do alvo e o destaca por alguns segundos.
///
/// O destaque é uma classe com animação de CSS, não foco: a pessoa veio
/// da busca pra LER, e roubar o foco atrapalharia quem continuou
/// digitando na caixa de busca.
pub fn revelar_alvo_de_busca(ancora: &str) {
    let Some((seg, _registro)) = ancora.split_once(':') else { return };
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else { return };
    let Ok(Some(el)) = doc.query_selector(&format!("[data-nav-group=\"embed-{seg}\"]")) else {
        return;
    };
    let _ = el.class_list().add_1("busca-alvo");
    let opts = web_sys::ScrollIntoViewOptions::new();
    opts.set_block(web_sys::ScrollLogicalPosition::Center);
    el.scroll_into_view_with_scroll_into_view_options(&opts);

    let el = el.clone();
    wasm_bindgen_futures::spawn_local(async move {
        gloo_timers::future::sleep(std::time::Duration::from_millis(2600)).await;
        let _ = el.class_list().remove_1("busca-alvo");
    });
}

/// Reancora a navegação quando o foco caiu num lugar genérico
/// (ciclo 197).
///
/// O caso real: abrir a paleta e escolher uma página desmonta o overlay
/// e o foco vai parar no `<body>`; a rede de segurança do ciclo 138
/// devolve pro `.app-root`, o que faz os atalhos globais voltarem — mas
/// o nav-mode fica sem item, e as setas param de andar porque não há
/// `data-nav-item` focado.
///
/// A regra é a que o usuário sugeriu: perdeu a referência micro, cai
/// pra uma mais macro em vez de ficar sem nenhuma.
///
/// Só age quando o foco NÃO pertence a ninguém (`<body>`, `.app-root`,
/// ou nada). Se o foco está num campo, num menu ou num delegate, ele é
/// de quem está lá — foi por confundir esses dois casos que o autocuro
/// das setas foi removido no ciclo 140.
pub fn reancorar_se_perdido(doc: &web_sys::Document, grupo: &str) -> bool {
    let ativo = doc.active_element();
    let perdido = match &ativo {
        None => true,
        Some(el) => {
            let tag = el.tag_name().to_lowercase();
            tag == "body" || el.class_list().contains("app-root")
        }
    };
    if !perdido {
        return false;
    }
    let itens = items_in_group(doc, grupo);
    match itens.first() {
        Some(primeiro) => {
            focus_item(primeiro);
            true
        }
        // Grupo sumiu junto (a página mudou): tenta a raiz, que é o
        // nível mais macro que sempre existe.
        None => match items_in_group(doc, "root").first() {
            Some(raiz) => {
                focus_item(raiz);
                true
            }
            None => false,
        },
    }
}

/// Reancora a sessão de navegação nos BLOCOS da página recém-aberta.
///
/// Abrir uma página de dentro de um grupo (um card de "Trabalho
/// recente", por exemplo) troca o conteúdo inteiro, mas a pilha
/// continuava apontando pro grupo ANTIGO — que não existe mais. As setas
/// então caíam no resgate de `reancorar_se_perdido`, cujo último recurso
/// é a raiz: por isso o teclado terminava preso na barra superior, longe
/// do que a pessoa acabou de abrir (ciclo 250).
///
/// O conteúdo chega assíncrono (a página é lida do disco), então isto
/// tenta algumas vezes antes de desistir em vez de apostar num único
/// `sleep`. Devolve na callback se achou os blocos.
pub fn focar_blocos_da_pagina(pronto: impl Fn(bool) + 'static) {
    wasm_bindgen_futures::spawn_local(async move {
        for _ in 0..20 {
            gloo_timers::future::sleep(std::time::Duration::from_millis(50)).await;
            let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                continue;
            };
            let itens = items_in_group(&doc, GRUPO_BLOCOS);
            if let Some(primeiro) = itens.first() {
                focus_item(primeiro);
                pronto(true);
                return;
            }
        }
        pronto(false);
    });
}

/// A tecla é um movimento de navegação? Devolve `true` pra frente.
///
/// `hjkl` valem onde as setas valem (RF3 da spec de teclado): quem navega
/// pelo teclado não devia ter que tirar a mão de casa. Maiúsculas ficam
/// de fora de propósito — `J`/`K` já movem o bloco, e são outra ação.
pub fn direcao_de_navegacao(tecla: &str) -> Option<bool> {
    match tecla {
        "ArrowDown" | "ArrowRight" | "j" | "l" => Some(true),
        "ArrowUp" | "ArrowLeft" | "k" | "h" => Some(false),
        _ => None,
    }
}
