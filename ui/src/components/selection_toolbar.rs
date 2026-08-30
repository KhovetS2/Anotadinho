//! Barra flutuante que aparece ao selecionar texto (ciclo 234).
//!
//! Até aqui o editor não tinha formatação nenhuma por interface: negrito
//! e itálico só saíam digitando `**` e `*`, e quem não sabia markdown não
//! tinha como descobrir que existiam. O menu `/` só oferece BLOCOS.
//!
//! Marca aplicada é HTML no DOM, e o markdown é recomposto do DOM ao
//! salvar — então só entra aqui marca que o `html_to_md` sabe devolver
//! pro arquivo (`strong`, `em`, `s`, `code`, `a`). Marca que não
//! sobrevive ao autosave é pior que marca nenhuma: some sozinha três
//! segundos depois, e ninguém entende por quê.
//!
//! Nada de `execCommand`: ele inventa `<font>` e `<span style>` conforme
//! o motor, e o `html_to_md` não os reconhece. Aqui a seleção é embrulhada
//! na mão, pela API de `Range`.

use wasm_bindgen::JsCast;
use web_sys::{Element, Range};
use yew::prelude::*;

use crate::dialog::PendingDialog;

/// Quanto a barra fica acima da seleção.
const FOLGA: f64 = 10.0;

/// A paleta nomeada (ciclo 235). Vira `class="cor--X"` / `fundo--X`, que
/// são tokens do tema — a cor escolhida no escuro continua legível no
/// claro. Gravar `#ffee00` no arquivo não teria essa propriedade.
const PALETA: &[(&str, &str)] = &[
    ("vermelho", "Vermelho"),
    ("ambar", "Âmbar"),
    ("verde", "Verde"),
    ("azul", "Azul"),
    ("roxo", "Roxo"),
    ("rosa", "Rosa"),
    ("cinza", "Cinza"),
];

#[derive(Properties, PartialEq, Clone)]
pub struct SelectionToolbarProps {
    /// Abre o diálogo do app (usado pra pedir a URL do link).
    pub open_dialog: Callback<PendingDialog>,
}

#[function_component(SelectionToolbar)]
pub fn selection_toolbar(props: &SelectionToolbarProps) -> Html {
    // Onde desenhar. `None` = sem seleção de texto, barra escondida.
    let posicao = use_state(|| None::<(f64, f64)>);
    // A paleta fica fechada por padrão: sete cores de texto e sete de
    // fundo abertas o tempo todo virariam uma parede na frente do texto.
    //
    // E fecha sozinha quando a seleção some: sem isso, selecionar outro
    // trecho trazia a barra já com a paleta aberta por cima do texto, de
    // uma interação anterior que a pessoa nem lembrava.
    let mostrar_cores = use_state(|| false);
    // A seleção de antes de abrir um modal. Abrir diálogo tira o foco do
    // editor e a seleção se perde; sem guardar, o link seria aplicado
    // em lugar nenhum. Mesmo cuidado do modal de imagens (ciclo 226).
    let guardada = use_mut_ref(|| None::<Range>);

    // O último valor conhecido, pra não chamar `set` quando nada mudou.
    //
    // `selectionchange` dispara a CADA movimento de cursor, inclusive ao
    // digitar. Sem esta comparação, cada tecla causava um re-render deste
    // componente pra escrever `None` por cima de `None`.
    //
    // `use_mut_ref` e não `use_state`: quem lê é o ouvinte, e handle de
    // `use_state` capturado em closure fica congelado no valor da
    // criação (ciclos 155, 157, 201, 213, 218).
    let ultimo = use_mut_ref(|| None::<(f64, f64)>);

    {
        let posicao = posicao.clone();
        let ultimo = ultimo.clone();
        let mostrar_cores = mostrar_cores.clone();
        use_effect_with((), move |_| {
            let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
                return Box::new(|| ()) as Box<dyn FnOnce()>;
            };
            let ouvinte = wasm_bindgen::closure::Closure::<dyn Fn()>::new(move || {
                let agora = medir_selecao();
                if *ultimo.borrow() == agora {
                    return;
                }
                // A barra sumindo é o fim de uma interação: o que estava
                // aberto nela não deve reaparecer na próxima.
                if agora.is_none() {
                    mostrar_cores.set(false);
                }
                *ultimo.borrow_mut() = agora;
                posicao.set(agora);
            });
            let _ = doc.add_event_listener_with_callback(
                "selectionchange",
                ouvinte.as_ref().unchecked_ref(),
            );
            let doc2 = doc.clone();
            Box::new(move || {
                let _ = doc2.remove_event_listener_with_callback(
                    "selectionchange",
                    ouvinte.as_ref().unchecked_ref(),
                );
                drop(ouvinte);
            }) as Box<dyn FnOnce()>
        });
    }

    let Some((x, y)) = *posicao else {
        return html! {};
    };
    let cores_abertas = *mostrar_cores;

    // Aplicar marca não avisa o editor por callback: dispara um `input`
    // no bloco, que borbulha até o `oninput` do contêiner. É o mesmo
    // caminho de quando a pessoa digita, com autosave e recomposição do
    // markdown já pendurados — um segundo caminho só divergiria do
    // primeiro com o tempo.
    let marcar = move |tag: &'static str| {
        Callback::from(move |e: MouseEvent| {
            // O clique não pode roubar a seleção antes de a gente usá-la.
            e.prevent_default();
            aplicar_marca(tag);
        })
    };

    let link = {
        let open_dialog = props.open_dialog.clone();
        let guardada = guardada.clone();
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            *guardada.borrow_mut() = selecao_atual();
            let guardada = guardada.clone();
            open_dialog.emit(PendingDialog::Prompt {
                title: "Link para onde?".to_string(),
                default: "https://".to_string(),
                on_submit: Callback::from(move |url: String| {
                    let url = url.trim().to_string();
                    if url.is_empty() {
                        return;
                    }
                    if let Some(r) = guardada.borrow().clone() {
                        restaurar(&r);
                    }
                    aplicar_link(&url);
                }),
            });
        })
    };

    // Pintar pela PALETA grava a classe do tema; a cor livre grava o
    // hex. As duas convivem porque cada uma resolve uma coisa: a do tema
    // sobrevive à troca de claro/escuro, a livre atende quem quer
    // exatamente aquele tom.
    let pintar = move |eixo: &'static str, slug: &'static str| {
        Callback::from(move |e: MouseEvent| {
            e.prevent_default();
            aplicar_cor(Cor::DaPaleta { eixo, slug });
        })
    };
    let cor_livre = Callback::from(|e: Event| {
        let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() else { return };
        aplicar_cor(Cor::Livre(input.value()));
    });
    let limpar_cor = Callback::from(|e: MouseEvent| {
        e.prevent_default();
        aplicar_cor(Cor::Nenhuma);
    });

    let estilo = format!("left: {x}px; top: {y}px;");
    html! {
        <div class="selecao-barra" style={estilo} data-nav-group="selecao-barra"
            // `mousedown` no próprio botão desfaria a seleção antes do
            // clique chegar — por isso o default morre já aqui.
            onmousedown={Callback::from(|e: MouseEvent| e.prevent_default())}>
            { for [
                ("strong", "N", "Negrito", "selecao-barra__negrito"),
                ("em", "I", "Itálico", "selecao-barra__italico"),
                ("s", "S", "Tachado", "selecao-barra__tachado"),
                ("code", "‹›", "Código", "selecao-barra__codigo"),
            ].into_iter().map(|(tag, rotulo, titulo, classe)| html! {
                <button class={classes!("selecao-barra__botao", classe)}
                    title={titulo} data-nav-item="true" onclick={marcar(tag)}>
                    { rotulo }
                </button>
            }) }
            <span class="selecao-barra__separador"></span>
            <button class="selecao-barra__botao selecao-barra__link" title="Link"
                data-nav-item="true" onclick={link}>{ "🔗" }</button>
            <span class="selecao-barra__separador"></span>
            <button class={classes!("selecao-barra__botao", "selecao-barra__cores",
                    cores_abertas.then_some("selecao-barra__cores--aberto"))}
                title="Cor e realce" data-nav-item="true"
                onclick={{
                    let abrir = mostrar_cores.clone();
                    Callback::from(move |e: MouseEvent| {
                        e.prevent_default();
                        abrir.set(!*abrir);
                    })
                }}>{ "A" }</button>
            if cores_abertas {
                <div class="selecao-barra__paleta">
                    <p class="selecao-barra__paleta-titulo">{ "Cor do texto" }</p>
                    <div class="selecao-barra__amostras">
                        { for PALETA.iter().map(|(slug, nome)| html! {
                            <button class={classes!("selecao-barra__amostra", format!("cor--{slug}"))}
                                title={*nome} data-nav-item="true"
                                onclick={pintar("cor", slug)}>{ "A" }</button>
                        }) }
                    </div>
                    <p class="selecao-barra__paleta-titulo">{ "Realce" }</p>
                    <div class="selecao-barra__amostras">
                        { for PALETA.iter().map(|(slug, nome)| html! {
                            <button class={classes!("selecao-barra__amostra", format!("fundo--{slug}"))}
                                title={*nome} data-nav-item="true"
                                onclick={pintar("fundo", slug)}>{ "A" }</button>
                        }) }
                    </div>
                    <label class="selecao-barra__livre">
                        { "Cor personalizada" }
                        <input type="color" data-nav-item="true"
                            onchange={cor_livre} />
                    </label>
                    <button class="selecao-barra__limpar" data-nav-item="true"
                        onclick={limpar_cor}>{ "Tirar a cor" }</button>
                </div>
            }
        </div>
    }
}

/// A seleção atual, se houver texto selecionado.
fn selecao_atual() -> Option<Range> {
    let sel = web_sys::window()?.get_selection().ok()??;
    if sel.is_collapsed() || sel.range_count() == 0 {
        return None;
    }
    sel.get_range_at(0).ok()
}

fn restaurar(r: &Range) {
    if let Some(sel) = web_sys::window().and_then(|w| w.get_selection().ok().flatten()) {
        let _ = sel.remove_all_ranges();
        let _ = sel.add_range(r);
    }
}

/// Onde desenhar a barra, ou `None` se não há seleção editável.
fn medir_selecao() -> Option<(f64, f64)> {
    let range = selecao_atual()?;

    // Só dentro de um bloco editável do editor. Selecionar texto na
    // barra lateral, numa proposta ou numa conversa não pode abrir uma
    // barra que não vai conseguir escrever em lugar nenhum.
    //
    // A âncora é `.editor`, não a raiz do `contenteditable`: numa página
    // COM embeds o editor não tem raiz única — cada segmento de markdown
    // é seu próprio `.editor__wysiwyg`. A primeira versão pedia a raiz e
    // por isso a barra simplesmente não aparecia em nenhuma página com
    // embed, que é justamente onde mora quase todo conteúdo do vault.
    let no = range.common_ancestor_container().ok()?;
    let alvo: Element = match no.dyn_ref::<Element>() {
        Some(e) => e.clone(),
        None => no.parent_element()?,
    };
    alvo.closest("[contenteditable=\"true\"]").ok()??;
    alvo.closest(".editor").ok()??;

    let r = range.get_bounding_client_rect();
    if r.width() == 0.0 && r.height() == 0.0 {
        return None;
    }
    Some((r.x() + r.width() / 2.0, r.y() - FOLGA))
}

/// O bloco editável onde a seleção está.
fn bloco_da_selecao(range: &Range) -> Option<Element> {
    let no = range.common_ancestor_container().ok()?;
    let de = match no.dyn_ref::<Element>() {
        Some(e) => e.clone(),
        None => no.parent_element()?,
    };
    de.closest("[contenteditable=\"true\"]").ok()?
}

/// Todos os nós de texto do bloco, com onde cada um começa e termina em
/// caracteres.
///
/// É a régua de tudo aqui. Guardar a seleção como um `Range` não serve:
/// aplicar ou tirar uma marca reconstrói os nós, e o `Range` antigo passa
/// a apontar pra nada. Já "do caractere 4 ao 9 deste bloco" continua
/// significando a mesma coisa depois da cirurgia.
fn nos_de_texto(bloco: &Element) -> Vec<(web_sys::Node, u32, u32)> {
    fn andar(no: &web_sys::Node, pos: &mut u32, saida: &mut Vec<(web_sys::Node, u32, u32)>) {
        if no.node_type() == web_sys::Node::TEXT_NODE {
            let tamanho = no.text_content().unwrap_or_default().chars().count() as u32;
            saida.push((no.clone(), *pos, *pos + tamanho));
            *pos += tamanho;
            return;
        }
        let filhos = no.child_nodes();
        for i in 0..filhos.length() {
            if let Some(f) = filhos.item(i) {
                andar(&f, pos, saida);
            }
        }
    }
    let mut saida = Vec::new();
    let mut pos = 0;
    andar(bloco, &mut pos, &mut saida);
    saida
}

/// Onde a seleção começa e termina, em caracteres do bloco.
fn intervalo(bloco: &Element, range: &Range) -> Option<(u32, u32)> {
    let doc = web_sys::window()?.document()?;
    let antes = doc.create_range().ok()?;
    antes.select_node_contents(bloco).ok()?;
    antes
        .set_end(&range.start_container().ok()?, range.start_offset().ok()?)
        .ok()?;
    let ini = antes.to_string().as_string().unwrap_or_default().chars().count() as u32;
    let tam = range.to_string().as_string().unwrap_or_default().chars().count() as u32;
    Some((ini, ini + tam))
}

/// Refaz a seleção a partir dos caracteres — é o que faz a barra
/// continuar servindo depois de aplicar ou tirar uma marca.
fn selecionar_intervalo(bloco: &Element, ini: u32, fim: u32) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let nos = nos_de_texto(bloco);
    let range = doc.create_range().ok()?;
    let mut comecou = false;
    for (no, de, ate) in &nos {
        if !comecou && ini >= *de && ini <= *ate {
            range.set_start(no, ini - de).ok()?;
            comecou = true;
        }
        if comecou && fim >= *de && fim <= *ate {
            range.set_end(no, fim - de).ok()?;
            let sel = web_sys::window()?.get_selection().ok()??;
            sel.remove_all_ranges().ok()?;
            sel.add_range(&range).ok()?;
            return Some(());
        }
    }
    None
}

/// Os elementos da marca que tocam a seleção.
fn marcas_na_selecao(bloco: &Element, tag: &str, ini: u32, fim: u32) -> Vec<Element> {
    let mut achados: Vec<Element> = Vec::new();
    for (no, de, ate) in nos_de_texto(bloco) {
        // Toca de verdade — encostar na borda não conta.
        if ate <= ini || de >= fim {
            continue;
        }
        let Some(pai) = no.parent_element() else { continue };
        let Ok(Some(marca)) = pai.closest(tag) else { continue };
        if !bloco.contains(Some(&marca)) {
            continue;
        }
        if !achados.iter().any(|a| a.is_same_node(Some(&marca))) {
            achados.push(marca);
        }
    }
    achados
}

/// A seleção INTEIRA já está marcada?
///
/// Perguntar pelo ancestral comum não servia: ao reselecionar arrastando,
/// o range costuma começar fora do `<strong>` e o ancestral vira o bloco.
/// A marca existia, a busca não achava, e o clique aninhava outra —
/// depois disso não saía mais (relatado no uso real).
fn tudo_marcado(bloco: &Element, tag: &str, ini: u32, fim: u32) -> bool {
    let mut viu_texto = false;
    for (no, de, ate) in nos_de_texto(bloco) {
        if ate <= ini || de >= fim {
            continue;
        }
        if no.text_content().unwrap_or_default().trim().is_empty() {
            continue;
        }
        viu_texto = true;
        let dentro = no
            .parent_element()
            .and_then(|p| p.closest(tag).ok().flatten())
            .is_some_and(|m| bloco.contains(Some(&m)));
        if !dentro {
            return false;
        }
    }
    viu_texto
}

/// Envolve a seleção na marca, ou tira a marca se ela já estiver lá.
///
/// A seleção é preservada nos dois caminhos: quem marcou uma palavra
/// costuma querer marcar outra coisa nela em seguida, e perder a seleção
/// no meio obriga a selecionar de novo a cada clique.
fn aplicar_marca(tag: &str) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;
    let bloco = bloco_da_selecao(&range)?;
    let (ini, fim) = intervalo(&bloco, &range)?;

    if tudo_marcado(&bloco, tag, ini, fim) {
        for marca in marcas_na_selecao(&bloco, tag, ini, fim) {
            desembrulhar(&marca);
        }
    } else {
        // Tira as marcas parciais antes de envolver: sem isto,
        // `<strong>meia</strong> palavra` viraria `<strong>` dentro de
        // `<strong>`.
        for marca in marcas_na_selecao(&bloco, tag, ini, fim) {
            desembrulhar(&marca);
        }
        bloco.normalize();
        let range = refazer(&bloco, ini, fim)?;
        let el = doc.create_element(tag).ok()?;
        // `extract_contents` + `insert_node` em vez de
        // `surround_contents`: este último falha quando a seleção
        // atravessa a borda de um elemento, que é o caso comum de
        // selecionar arrastando.
        let conteudo = range.extract_contents().ok()?;
        el.append_child(&conteudo).ok()?;
        range.insert_node(&el).ok()?;
    }
    bloco.normalize();
    selecionar_intervalo(&bloco, ini, fim);
    avisar_edicao_no_bloco(&bloco)
}

/// Refaz a seleção e devolve o `Range` correspondente.
fn refazer(bloco: &Element, ini: u32, fim: u32) -> Option<Range> {
    selecionar_intervalo(bloco, ini, fim)?;
    selecao_atual()
}

fn aplicar_link(url: &str) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;
    let bloco = bloco_da_selecao(&range)?;
    let (ini, fim) = intervalo(&bloco, &range)?;
    let el = doc.create_element("a").ok()?;
    el.set_attribute("href", url).ok()?;
    let conteudo = range.extract_contents().ok()?;
    el.append_child(&conteudo).ok()?;
    range.insert_node(&el).ok()?;
    bloco.normalize();
    selecionar_intervalo(&bloco, ini, fim);
    avisar_edicao_no_bloco(&bloco)
}

/// Troca o elemento pelos filhos dele, preservando o texto.
fn desembrulhar(el: &Element) {
    let Some(pai) = el.parent_node() else { return };
    while let Some(filho) = el.first_child() {
        if pai.insert_before(&filho, Some(el)).is_err() {
            return;
        }
    }
    let _ = pai.remove_child(el);
}

/// Dispara um `input` no bloco, pra a edição seguir o MESMO caminho de
/// quando a pessoa digita — autosave, undo e recomposição do markdown já
/// estão pendurados ali, e duplicá-los seria pedir pra divergirem.
fn avisar_edicao_no_bloco(bloco: &Element) -> Option<()> {
    let ev = web_sys::Event::new_with_event_init_dict(
        "input",
        web_sys::EventInit::new().bubbles(true),
    )
    .ok()?;
    bloco.dispatch_event(&ev).ok()?;
    Some(())
}

/// O que aplicar como cor.
enum Cor {
    /// Um tom da paleta do tema: `eixo` é "cor" (texto) ou "fundo".
    DaPaleta { eixo: &'static str, slug: &'static str },
    /// Um hex escolhido no seletor do sistema.
    Livre(String),
    /// Tirar a cor que estiver lá.
    Nenhuma,
}

/// Pinta a seleção, trocando a cor anterior se já houver uma.
fn aplicar_cor(cor: Cor) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;
    let bloco = bloco_da_selecao(&range)?;
    let (ini, fim) = intervalo(&bloco, &range)?;

    // Se a seleção já está num span de cor, ele é reaproveitado em vez de
    // aninhar outro. Sem isso, trocar de cor cinco vezes deixaria cinco
    // spans encaixados, e o arquivo viraria sopa.
    let existente = marcas_na_selecao(&bloco, "span", ini, fim)
        .into_iter()
        .find(cor_do_span_existe);

    match (&cor, existente) {
        (Cor::Nenhuma, Some(el)) => desembrulhar(&el),
        (Cor::Nenhuma, None) => return None,
        (_, Some(el)) => aplicar_no_elemento(&el, &cor),
        (_, None) => {
            let el = doc.create_element("span").ok()?;
            aplicar_no_elemento(&el, &cor);
            let conteudo = range.extract_contents().ok()?;
            el.append_child(&conteudo).ok()?;
            range.insert_node(&el).ok()?;
        }
    }
    bloco.normalize();
    selecionar_intervalo(&bloco, ini, fim);
    avisar_edicao_no_bloco(&bloco)
}

fn aplicar_no_elemento(el: &Element, cor: &Cor) {
    match cor {
        Cor::DaPaleta { eixo, slug } => {
            // Texto e realce são independentes: pintar o fundo não pode
            // apagar a cor da letra.
            let manter: Vec<String> = el
                .get_attribute("class")
                .unwrap_or_default()
                .split_whitespace()
                .filter(|c| !c.starts_with(&format!("{eixo}--")))
                .map(|c| c.to_string())
                .collect();
            let mut classes = manter;
            classes.push(format!("{eixo}--{slug}"));
            let _ = el.set_attribute("class", &classes.join(" "));
            let _ = el.remove_attribute("style");
        }
        Cor::Livre(hex) => {
            let _ = el.set_attribute("style", &format!("color:{hex}"));
            let sem_cor: Vec<String> = el
                .get_attribute("class")
                .unwrap_or_default()
                .split_whitespace()
                .filter(|c| !c.starts_with("cor--"))
                .map(|c| c.to_string())
                .collect();
            let _ = el.set_attribute("class", &sem_cor.join(" "));
        }
        Cor::Nenhuma => {}
    }
}

/// O span já carrega cor? Um `<span>` qualquer não deve ser sequestrado.
fn cor_do_span_existe(el: &Element) -> bool {
    let classe = el.get_attribute("class").unwrap_or_default();
    if classe
        .split_whitespace()
        .any(|c| c.starts_with("cor--") || c.starts_with("fundo--"))
    {
        return true;
    }
    el.get_attribute("style")
        .is_some_and(|s| s.contains("color"))
}
