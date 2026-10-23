//! O diagrama de um bloco ` ```mermaid ` (ciclo 485).
//!
//! Até o 484 o bloco salvo nunca era desenhado: o mermaid só rodava
//! sobre o `div.mermaid` que o `/mermaid` inseria, e ao reabrir a página
//! a fence voltava a ser `<pre><code>`. Eram 60 MiB de biblioteca
//! carregada para nada.
//!
//! O `<pre>` continua sendo o bloco e a fonte da verdade. O diagrama vai
//! como imagem num `::after`, por variável CSS inline:
//! - um elemento DENTRO do `<pre>` vazaria os rótulos do SVG no
//!   `text_content` do bloco, que o yank, o modo visual e as posições de
//!   cursor do vim leem inteiro;
//! - um elemento IRMÃO entraria em mover bloco, juntar no Backspace e em
//!   toda navegação que anda por `next_element_sibling`.
//!
//! Um pseudo-elemento não é nó: não tem texto, não é irmão e não se
//! seleciona.

use std::cell::Cell;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

/// O `<pre>` tem um diagrama (desenhado ou com erro).
pub const CLASSE: &str = "editor__bloco--mermaid";
/// O diagrama saiu: o CSS esconde o código, com ou sem foco. Até o 490 o
/// foco mostrava o código por cima do desenho; clicar num diagrama pra
/// selecioná-lo despejava a sintaxe na página (ciclo 491). O código se
/// edita no construtor (aba Código); inline, só quando o desenho falhou.
pub const CLASSE_DESENHADO: &str = "editor__bloco--mermaid-desenhado";
/// O bloco está com foco: o desenho aparece selecionado, e o botão do
/// construtor, ao lado. É classe, e não `:focus` no CSS, porque o
/// WebKitGTK só aplica `:focus` com a janela focada pelo sistema: o
/// harness (calço do ciclo 455) dispara os eventos, mas não a pseudo-classe.
/// Os eventos existem nos dois casos.
pub const CLASSE_EDITANDO: &str = "editor__bloco--mermaid-editando";
/// A fonte do último desenho. Igual à atual = nada a fazer.
const ATTR_FONTE: &str = "data-mermaid-fonte";
/// A mensagem do erro de sintaxe, mostrada pelo `::after`.
const ATTR_ERRO: &str = "data-mermaid-erro";

/// O evento que pede o construtor pra um bloco (ciclo 488). Disparado NO
/// `<pre>`, borbulhando: o editor ouve no documento e acha o bloco pelo
/// `target`.
pub const EVENTO_CONSTRUTOR: &str = "anotadinho-construtor-mermaid";

thread_local! {
    static OUVINTES: Cell<bool> = const { Cell::new(false) };
    /// O botão "Editar no construtor" e o bloco a que ele está preso agora.
    static BOTAO: std::cell::RefCell<Option<(web_sys::HtmlElement, Option<web_sys::HtmlElement>)>> =
        const { std::cell::RefCell::new(None) };
    static PROXIMO_ID: Cell<u32> = const { Cell::new(0) };
}

/// Largura e altura do `viewBox` do `<svg>` raiz: é delas que sai a
/// proporção da imagem. O `width` do mermaid é `100%`, que como imagem de
/// fundo não diz tamanho nenhum.
pub fn medidas_do_svg(svg: &str) -> Option<(f64, f64)> {
    let raiz = &svg[svg.find("<svg")?..];
    let raiz = &raiz[..raiz.find('>')?];
    let ini = raiz.find("viewBox=\"")? + "viewBox=\"".len();
    let fim = raiz[ini..].find('"')? + ini;
    let nums: Vec<f64> = raiz[ini..fim]
        .split(|c: char| c == ' ' || c == ',')
        .filter(|s| !s.is_empty())
        .map(|s| s.parse().ok())
        .collect::<Option<_>>()?;
    match nums[..] {
        [_, _, w, h] if w > 0.0 && h > 0.0 => Some((w, h)),
        _ => None,
    }
}

/// O `<svg>` raiz com tamanho explícito: `width` e `height` do `viewBox`,
/// sem o `width="100%"` e o `max-width` do mermaid. Como imagem (`<img>` da
/// prévia), aquilo dava um tamanho natural de 46×150 a um diagrama de
/// 400×500 — a prévia saía espremida e cortada (visto no ciclo 490).
pub fn com_tamanho(svg: &str, largura: f64, altura: f64) -> String {
    let Some(ini) = svg.find("<svg") else {
        return svg.to_string();
    };
    let Some(fim) = svg[ini..].find('>').map(|f| ini + f) else {
        return svg.to_string();
    };
    let mut raiz = svg[ini..fim].to_string();
    for attr in ["width", "height", "style"] {
        // ` attr="…"` — com o espaço, pra não pegar `stroke-width`.
        let chave = format!(" {attr}=\"");
        if let Some(i) = raiz.find(&chave) {
            if let Some(f) = raiz[i + chave.len()..].find('"') {
                raiz.replace_range(i..i + chave.len() + f + 1, "");
            }
        }
    }
    raiz.push_str(&format!(" width=\"{largura}\" height=\"{altura}\""));
    format!("{}{}{}", &svg[..ini], raiz, &svg[fim..])
}

/// A primeira linha da mensagem, curta: o `::after` mostra numa linha só,
/// e o mermaid devolve o trecho do código com setas por baixo.
pub fn resumo_do_erro(mensagem: &str) -> String {
    let linha = mensagem.lines().map(str::trim).find(|l| !l.is_empty()).unwrap_or("erro no diagrama");
    let mut curto: String = linha.chars().take(160).collect();
    if curto.chars().count() < linha.chars().count() {
        curto.push('…');
    }
    format!("Diagrama com erro: {curto}")
}

/// Desenha todo bloco mermaid dentro de `container`.
///
/// Sem bloco nenhum, a biblioteca nem é lida (ciclo 486): é o caso de
/// quase toda página, e são 60 MiB.
pub fn desenhar_em(container: &web_sys::Element) {
    let Ok(codigos) = container.query_selector_all("pre > code.language-mermaid") else {
        return;
    };
    let pres: Vec<web_sys::HtmlElement> = (0..codigos.length())
        .filter_map(|i| codigos.item(i))
        .filter_map(|c| c.parent_element())
        .filter_map(|p| p.dyn_into::<web_sys::HtmlElement>().ok())
        .collect();
    if pres.is_empty() {
        return;
    }
    instalar_ouvintes();
    wasm_bindgen_futures::spawn_local(async move {
        if !crate::biblioteca::carregar(crate::biblioteca::Biblioteca::Mermaid).await {
            // Sem a biblioteca, o bloco fica como código: é o que ele é.
            return;
        }
        for pre in pres {
            desenhar_bloco(pre);
        }
    });
}

fn codigo_de(pre: &web_sys::Element) -> Option<web_sys::Element> {
    pre.query_selector(":scope > code.language-mermaid").ok().flatten()
}

fn desenhar_bloco(pre: web_sys::HtmlElement) {
    let Some(code) = codigo_de(&pre) else {
        return;
    };
    let fonte = code.text_content().unwrap_or_default();
    if pre.get_attribute(ATTR_FONTE).as_deref() == Some(fonte.as_str()) {
        return;
    }
    let Some(window) = web_sys::window() else {
        return;
    };
    // Antes de marcar a fonte: sem a biblioteca, marcar faria a próxima
    // tentativa achar que já desenhou.
    let Some(mermaid) = js_sys::Reflect::get(&window, &JsValue::from_str("mermaid"))
        .ok()
        .filter(|m| m.is_object())
    else {
        return;
    };
    let _ = pre.class_list().add_1(CLASSE);
    // Marca ANTES de desenhar: um segundo `focusout` durante o render não
    // dispara outro com a mesma fonte.
    let _ = pre.set_attribute(ATTR_FONTE, &fonte);
    if fonte.trim().is_empty() {
        limpar(&pre);
        return;
    }
    configurar(&mermaid);

    wasm_bindgen_futures::spawn_local(async move {
        let resultado = renderizar_com(&mermaid, &fonte).await;
        // A pessoa mudou o código enquanto desenhava: o próximo `focusout`
        // desenha a versão nova; esta já não vale.
        if codigo_de(&pre).and_then(|c| c.text_content()).as_deref() != Some(fonte.as_str()) {
            return;
        }
        match resultado {
            Ok(desenho) => aplicar(&pre, &desenho),
            Err(msg) => mostrar_erro(&pre, &msg),
        }
    });
}

/// Um diagrama desenhado: a imagem e o tamanho natural dela.
#[derive(Debug, Clone, PartialEq)]
pub struct Desenho {
    /// `url("data:image/svg+xml…")`, pronto pra CSS.
    pub url_css: String,
    /// O mesmo, sem o `url(…)`: pra `<img src>`.
    pub data_url: String,
    pub largura: f64,
    pub altura: f64,
}

/// Desenha `fonte` com o mermaid (lendo a biblioteca se precisar). O erro
/// é a mensagem do mermaid, crua: quem mostra resume.
///
/// O bloco (485) e a prévia do construtor (488) passam por aqui, pra que
/// a prévia seja exatamente o que a página vai mostrar.
pub async fn renderizar(fonte: &str) -> Result<Desenho, String> {
    if !crate::biblioteca::carregar(crate::biblioteca::Biblioteca::Mermaid).await {
        return Err("o mermaid não carregou".into());
    }
    let mermaid = web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str("mermaid")).ok())
        .filter(|m| m.is_object())
        .ok_or_else(|| "o mermaid não carregou".to_string())?;
    configurar(&mermaid);
    renderizar_com(&mermaid, fonte).await
}

async fn renderizar_com(mermaid: &JsValue, fonte: &str) -> Result<Desenho, String> {
    let id = PROXIMO_ID.with(|c| {
        c.set(c.get() + 1);
        format!("mermaid-{}", c.get())
    });
    let render = js_sys::Reflect::get(mermaid, &JsValue::from_str("render"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
        .ok_or_else(|| "o mermaid não tem `render`".to_string())?;
    let promessa = match render.call2(mermaid, &JsValue::from_str(&id), &JsValue::from_str(fonte)) {
        Ok(p) => js_sys::Promise::resolve(&p),
        Err(e) => js_sys::Promise::reject(&e),
    };
    let resultado = wasm_bindgen_futures::JsFuture::from(promessa).await;
    // Com erro, o mermaid 10 deixa o rascunho do desenho (`#d<id>`)
    // pendurado no `<body>` — um a cada tentativa.
    if let Some(sobra) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id(&format!("d{id}")))
    {
        sobra.remove();
    }
    match resultado {
        Ok(v) => {
            let svg = js_sys::Reflect::get(&v, &JsValue::from_str("svg"))
                .ok()
                .and_then(|s| s.as_string())
                .ok_or_else(|| "o mermaid não devolveu desenho".to_string())?;
            let (largura, altura) = medidas_do_svg(&svg).ok_or_else(|| "o desenho veio sem medidas".to_string())?;
            let svg = com_tamanho(&svg, largura, altura);
            let data_url = format!(
                "data:image/svg+xml;charset=utf-8,{}",
                String::from(js_sys::encode_uri_component(&svg))
            );
            Ok(Desenho { url_css: format!("url(\"{data_url}\")"), data_url, largura, altura })
        }
        Err(e) => Err(js_sys::Reflect::get(&e, &JsValue::from_str("message"))
            .ok()
            .and_then(|m| m.as_string())
            .or_else(|| e.as_string())
            .unwrap_or_default()),
    }
}

fn aplicar(pre: &web_sys::HtmlElement, d: &Desenho) {
    let estilo = pre.style();
    let _ = estilo.set_property("--mermaid-previa", &d.url_css);
    let _ = estilo.set_property("--mermaid-proporcao", &format!("{} / {}", d.largura, d.altura));
    let _ = estilo.set_property("--mermaid-largura", &format!("{}px", d.largura));
    let _ = pre.remove_attribute(ATTR_ERRO);
    let _ = pre.class_list().add_1(CLASSE_DESENHADO);
}

fn mostrar_erro(pre: &web_sys::HtmlElement, mensagem: &str) {
    limpar(pre);
    let _ = pre.set_attribute(ATTR_ERRO, &resumo_do_erro(mensagem));
}

fn limpar(pre: &web_sys::HtmlElement) {
    let estilo = pre.style();
    for p in ["--mermaid-previa", "--mermaid-proporcao", "--mermaid-largura"] {
        let _ = estilo.remove_property(p);
    }
    let _ = pre.remove_attribute(ATTR_ERRO);
    let _ = pre.class_list().remove_1(CLASSE_DESENHADO);
}

/// `mermaid.initialize` com as cores do tema ATUAL. Chamada a cada
/// desenho: o tema muda sem recarregar a página, e é barato.
///
/// Antes do 485 isto era `Reflect::set(mermaid, "initialize", config)`:
/// trocava a função pelo objeto, e o mermaid nunca era configurado.
fn configurar(mermaid: &JsValue) {
    let Some(window) = web_sys::window() else {
        return;
    };
    let token = |nome: &str| -> String {
        window
            .document()
            .and_then(|d| d.document_element())
            .and_then(|raiz| window.get_computed_style(&raiz).ok().flatten())
            .and_then(|s| s.get_property_value(nome).ok())
            .map(|v| v.trim().to_string())
            .unwrap_or_default()
    };
    let config = serde_json::json!({
        "startOnLoad": false,
        // `strict`: o diagrama vem da página, e uma página pode vir de
        // fora (vault sincronizado). Nada de HTML nem clique nos rótulos.
        "securityLevel": "strict",
        "theme": "base",
        // Rótulo em `<foreignObject>` depende das fontes da página, que
        // uma imagem SVG não enxerga: texto SVG puro desenha igual.
        "flowchart": { "htmlLabels": false },
        "themeVariables": {
            "background": token("--bg-surface"),
            "primaryColor": token("--bg-elevated"),
            "primaryTextColor": token("--text-primary"),
            "primaryBorderColor": token("--accent-blue"),
            "secondaryColor": token("--fundo-azul"),
            "tertiaryColor": token("--bg-surface"),
            "lineColor": token("--text-muted"),
            "textColor": token("--text-primary"),
            "fontFamily": token("--font-sans"),
        },
    });
    let Ok(config) = js_sys::JSON::parse(&config.to_string()) else {
        return;
    };
    if let Some(initialize) = js_sys::Reflect::get(mermaid, &JsValue::from_str("initialize"))
        .ok()
        .and_then(|f| f.dyn_into::<js_sys::Function>().ok())
    {
        let _ = initialize.call1(mermaid, &config);
    }
}

/// Um par de ouvintes no documento, para o app inteiro: um closure por
/// página seria memória presa a cada visita (o que o 482 tirou do grafo).
fn instalar_ouvintes() {
    if OUVINTES.with(|c| c.replace(true)) {
        return;
    }
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };
    let pre_mermaid = |e: &web_sys::Event| -> Option<web_sys::HtmlElement> {
        let alvo = e.target()?.dyn_into::<web_sys::Element>().ok()?;
        let pre = alvo.closest("pre").ok()??;
        codigo_de(&pre)?;
        pre.dyn_into::<web_sys::HtmlElement>().ok()
    };

    // Saiu do bloco: redesenha se o código mudou.
    let ao_sair = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        if let Some(pre) = pre_mermaid(&e) {
            let _ = pre.class_list().remove_1(CLASSE_EDITANDO);
            esconder_botao();
            desenhar_bloco(pre);
        }
    });
    let _ = doc.add_event_listener_with_callback("focusout", ao_sair.as_ref().unchecked_ref());
    ao_sair.forget();

    // Entrou no bloco desenhado: o `<code>` estava escondido, então o
    // cursor cai no `<pre>`, FORA dele — e o que se digitasse ali o save
    // jogaria fora (o `html_to_md` lê só o `<code>`). Leva o cursor pro
    // fim do código.
    let ao_entrar = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        let Some(pre) = pre_mermaid(&e) else {
            return;
        };
        let _ = pre.class_list().add_1(CLASSE_EDITANDO);
        mostrar_botao(&pre);
        let Some(code) = codigo_de(&pre) else {
            return;
        };
        let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) else {
            return;
        };
        let dentro = sel
            .anchor_node()
            .map(|n| code.contains(Some(&n)))
            .unwrap_or(false);
        if dentro {
            return;
        }
        if let Some(r) = web_sys::window()
            .and_then(|w| w.document())
            .and_then(|d| d.create_range().ok())
        {
            let _ = r.select_node_contents(&code);
            r.collapse_with_to_start(false);
            let _ = sel.remove_all_ranges();
            let _ = sel.add_range(&r);
        }
    });
    let _ = doc.add_event_listener_with_callback("focusin", ao_entrar.as_ref().unchecked_ref());
    ao_entrar.forget();

    // `Ctrl+Enter` dentro do bloco: o construtor sem precisar do mouse. No
    // desenho selecionado, o Enter sozinho também — não há texto à vista
    // em que ele quebrasse linha.
    let ao_teclar = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |e: web_sys::KeyboardEvent| {
        if e.key() != "Enter" || e.shift_key() || e.alt_key() {
            return;
        }
        let Some(pre) = pre_mermaid(&e) else {
            return;
        };
        if e.ctrl_key() || e.meta_key() || pre.class_list().contains(CLASSE_DESENHADO) {
            e.prevent_default();
            e.stop_propagation();
            pedir_construtor(&pre);
        }
    });
    let _ = doc.add_event_listener_with_callback_and_bool("keydown", ao_teclar.as_ref().unchecked_ref(), true);
    ao_teclar.forget();

    // Com o desenho à vista, o cursor está no código ESCONDIDO: o que se
    // digitasse, colasse ou apagasse mudaria um texto que ninguém vê. O
    // `beforeinput` cobre teclado, colar, arrastar e IME de uma vez.
    let ao_editar = Closure::<dyn Fn(web_sys::Event)>::new(move |e: web_sys::Event| {
        if pre_mermaid(&e).is_some_and(|pre| pre.class_list().contains(CLASSE_DESENHADO)) {
            e.prevent_default();
            e.stop_propagation();
        }
    });
    let _ = doc.add_event_listener_with_callback_and_bool("beforeinput", ao_editar.as_ref().unchecked_ref(), true);
    ao_editar.forget();

    // O botão é `fixed`: acompanha o bloco quando a página rola.
    if let Some(w) = web_sys::window() {
        let reposicionar = Closure::<dyn Fn()>::new(posicionar_botao);
        let _ = doc.add_event_listener_with_callback_and_bool("scroll", reposicionar.as_ref().unchecked_ref(), true);
        let _ = w.add_event_listener_with_callback("resize", reposicionar.as_ref().unchecked_ref());
        reposicionar.forget();
    }
}

fn pedir_construtor(pre: &web_sys::HtmlElement) {
    let init = web_sys::EventInit::new();
    init.set_bubbles(true);
    if let Ok(ev) = web_sys::Event::new_with_event_init_dict(EVENTO_CONSTRUTOR, &init) {
        let _ = pre.dispatch_event(&ev);
    }
}

/// O botão "Editar no construtor" vive no `<body>`, FORA de qualquer
/// `contenteditable` e da fila de irmãos dos blocos: pelo mesmo motivo do
/// `::after`, um botão dentro do bloco entraria no `text_content` e no
/// `html_to_md` (ciclo 488).
fn botao() -> Option<web_sys::HtmlElement> {
    if let Some(b) = BOTAO.with(|b| b.borrow().as_ref().map(|(b, _)| b.clone())) {
        return Some(b);
    }
    let doc = web_sys::window()?.document()?;
    let b = doc.create_element("button").ok()?.dyn_into::<web_sys::HtmlElement>().ok()?;
    b.set_class_name("btn btn--secondary btn--xs construtor-mermaid__abrir");
    let _ = b.set_attribute("type", "button");
    let _ = b.set_attribute("data-nav-item", "construtor-mermaid-abrir");
    let _ = b.set_attribute("title", "Editar no construtor (Ctrl+Enter)");
    b.set_text_content(Some("Editar no construtor"));
    // `mousedown` sem default: o foco fica no bloco, e o `focusout` não
    // esconde o botão antes do clique chegar.
    let segurar = Closure::<dyn Fn(web_sys::Event)>::new(|e: web_sys::Event| e.prevent_default());
    let _ = b.add_event_listener_with_callback("mousedown", segurar.as_ref().unchecked_ref());
    segurar.forget();
    let clicar = Closure::<dyn Fn()>::new(|| {
        let alvo = BOTAO.with(|b| b.borrow().as_ref().and_then(|(_, p)| p.clone()));
        if let Some(pre) = alvo {
            pedir_construtor(&pre);
        }
    });
    let _ = b.add_event_listener_with_callback("click", clicar.as_ref().unchecked_ref());
    clicar.forget();
    let _ = b.style().set_property("display", "none");
    doc.body()?.append_child(&b).ok()?;
    BOTAO.with(|slot| *slot.borrow_mut() = Some((b.clone(), None)));
    Some(b)
}

fn mostrar_botao(pre: &web_sys::HtmlElement) {
    let Some(b) = botao() else {
        return;
    };
    BOTAO.with(|slot| {
        if let Some((_, alvo)) = slot.borrow_mut().as_mut() {
            *alvo = Some(pre.clone());
        }
    });
    let _ = b.style().remove_property("display");
    posicionar_botao();
}

fn esconder_botao() {
    BOTAO.with(|slot| {
        if let Some((b, alvo)) = slot.borrow_mut().as_mut() {
            let _ = b.style().set_property("display", "none");
            *alvo = None;
        }
    });
}

fn posicionar_botao() {
    let Some((b, Some(pre))) = BOTAO.with(|s| s.borrow().clone()) else {
        return;
    };
    // O bloco saiu da página (trocou de nota com ele em foco): o
    // `focusout` não vem de um nó removido.
    if !pre.is_connected() {
        esconder_botao();
        return;
    }
    let r = pre.get_bounding_client_rect();
    let estilo = b.style();
    let _ = estilo.set_property("top", &format!("{}px", r.top() + 6.0));
    let _ = estilo.set_property("left", &format!("{}px", r.right() - 6.0));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn medidas_vem_do_viewbox_da_raiz() {
        let svg = r#"<svg id="m1" width="100%" viewBox="-8 -8 316.5 174" style="max-width: 316.5px;"><g><svg viewBox="0 0 1 1"></svg></g></svg>"#;
        assert_eq!(medidas_do_svg(svg), Some((316.5, 174.0)));
    }

    #[test]
    fn sem_viewbox_ou_com_zero_nao_tem_medida() {
        assert_eq!(medidas_do_svg(r#"<svg width="100%"><g viewBox="0 0 5 5"/></svg>"#), None);
        assert_eq!(medidas_do_svg(r#"<svg viewBox="0 0 0 10"></svg>"#), None);
        assert_eq!(medidas_do_svg(r#"<svg viewBox="0 0 x 10"></svg>"#), None);
    }

    #[test]
    fn raiz_ganha_tamanho_e_perde_o_cem_por_cento() {
        let svg = r#"<svg id="m" width="100%" xmlns="http://www.w3.org/2000/svg" style="max-width: 316.5px;" viewBox="0 0 316.5 174"><rect stroke-width="2" width="10"/></svg>"#;
        let novo = com_tamanho(svg, 316.5, 174.0);
        let raiz = &novo[..novo.find('>').unwrap()];
        assert!(!raiz.contains("100%") && !raiz.contains("max-width"), "{raiz}");
        assert!(raiz.contains(r#"width="316.5" height="174""#), "{raiz}");
        // Só a raiz: o resto do desenho fica intocado.
        assert!(novo.contains(r#"<rect stroke-width="2" width="10"/>"#), "{novo}");
    }

    #[test]
    fn erro_vira_uma_linha_curta() {
        let msg = "\n  Parse error on line 2:\n...A --> \n-------^\nExpecting 'X'";
        assert_eq!(resumo_do_erro(msg), "Diagrama com erro: Parse error on line 2:");
        let longo = "x".repeat(300);
        assert!(resumo_do_erro(&longo).ends_with('…'));
        assert_eq!(resumo_do_erro(""), "Diagrama com erro: erro no diagrama");
    }
}
