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

thread_local! {
    /// Ligado enquanto a barra está mexendo no DOM.
    ///
    /// Aplicar uma marca esvazia a seleção no meio do caminho —
    /// `extract_contents` dispara `selectionchange` com nada
    /// selecionado —, e sem distinguir isso a paleta se fechava sozinha
    /// no meio de um clique nela. O que fecha a paleta é a pessoa
    /// selecionar outra coisa, não a gente reconstruir os nós.
    ///
    /// `thread_local` e não estado do componente porque quem precisa
    /// consultar é o ouvinte global de `selectionchange`, e o WASM roda
    /// numa thread só.
    static MUTANDO: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Roda a cirurgia com o ouvinte avisado de que a mexida é nossa.
///
/// A bandeira baixa no PRÓXIMO tique, não aqui: `selectionchange` é
/// entregue de forma assíncrona, e a cirurgia esvazia a seleção antes de
/// refazê-la. Baixando na hora, o evento da nossa própria mexida chegava
/// com a bandeira já baixada, era lido como "a pessoa selecionou outra
/// coisa", e a paleta se fechava no meio de um clique nela.
fn mexendo<T>(f: impl FnOnce() -> T) -> T {
    MUTANDO.with(|m| m.set(true));
    let r = f();
    gloo_timers::callback::Timeout::new(0, || MUTANDO.with(|m| m.set(false))).forget();
    r
}

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
                // Seleção nova é interação nova: o que estava aberto na
                // anterior não deve reaparecer por cima do texto. Só que
                // isso vale pra seleção que a PESSOA mudou — a nossa
                // própria cirurgia também mexe na seleção.
                if !MUTANDO.with(|m| m.get()) {
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
            mexendo(|| aplicar_marca(tag));
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
                    mexendo(|| aplicar_link(&url));
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
            mexendo(|| aplicar_cor(Cor::DaPaleta { eixo, slug }));
        })
    };
    let cor_livre = Callback::from(|e: Event| {
        let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() else { return };
        mexendo(|| aplicar_cor(Cor::Livre(input.value())));
    });
    let limpar_cor = Callback::from(|e: MouseEvent| {
        e.prevent_default();
        mexendo(|| aplicar_cor(Cor::Nenhuma));
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
pub(crate) fn nos_de_texto(bloco: &Element) -> Vec<(web_sys::Node, u32, u32)> {
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
pub(crate) fn intervalo(bloco: &Element, range: &Range) -> Option<(u32, u32)> {
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
pub(crate) fn selecionar_intervalo(bloco: &Element, ini: u32, fim: u32) -> Option<()> {
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

/// Onde um elemento começa e termina, em caracteres do bloco.
fn span_do_elemento(bloco: &Element, el: &Element) -> Option<(u32, u32)> {
    let mut ini = None;
    let mut fim = 0;
    for (no, de, ate) in nos_de_texto(bloco) {
        if el.contains(Some(&no)) {
            if ini.is_none() {
                ini = Some(de);
            }
            fim = ate;
        }
    }
    ini.map(|i| (i, fim))
}

/// Tira a marca só do TRECHO selecionado, devolvendo as bordas que
/// continuam marcadas (ciclo 244).
///
/// Antes eu desembrulhava a marca inteira: selecionar uma palavra de uma
/// frase em negrito tirava o negrito da frase toda. O que se pede ao
/// clicar em negrito com uma palavra selecionada é sobre AQUELA palavra.
fn tirar_marca_do_trecho(bloco: &Element, tag: &str, ini: u32, fim: u32) -> Vec<(u32, u32)> {
    let mut sobras = Vec::new();
    for marca in marcas_na_selecao(bloco, tag, ini, fim) {
        if let Some((mi, mf)) = span_do_elemento(bloco, &marca) {
            if mi < ini {
                sobras.push((mi, ini));
            }
            if fim < mf {
                sobras.push((fim, mf));
            }
        }
        desembrulhar(&marca);
    }
    bloco.normalize();
    sobras
}

/// Encolhe o trecho até ele não começar nem terminar em espaço.
///
/// Marca com espaço na borda não é markdown válido: `**uma **` não vira
/// negrito em lugar nenhum. Ao partir uma frase marcada, a sobra vira
/// exatamente isso — "uma " e " inteira aqui" —, e o arquivo saía com o
/// negrito grudado na palavra seguinte.
fn aparar(bloco: &Element, ini: u32, fim: u32) -> (u32, u32) {
    let texto: Vec<char> = bloco.text_content().unwrap_or_default().chars().collect();
    let mut a = ini as usize;
    let mut b = (fim as usize).min(texto.len());
    while a < b && texto[a].is_whitespace() {
        a += 1;
    }
    while b > a && texto[b - 1].is_whitespace() {
        b -= 1;
    }
    (a as u32, b as u32)
}

/// Envolve um trecho do bloco na marca, sem levar espaço nas bordas.
fn envolver_intervalo(bloco: &Element, tag: &str, ini: u32, fim: u32) -> Option<()> {
    let (ini, fim) = aparar(bloco, ini, fim);
    if ini >= fim {
        return None;
    }
    let doc = web_sys::window()?.document()?;
    selecionar_intervalo(bloco, ini, fim)?;
    let range = selecao_atual()?;
    let el = doc.create_element(tag).ok()?;
    // `extract_contents` + `insert_node` em vez de `surround_contents`:
    // este último falha quando a seleção atravessa a borda de um
    // elemento, que é o caso comum de selecionar arrastando.
    let conteudo = range.extract_contents().ok()?;
    el.append_child(&conteudo).ok()?;
    range.insert_node(&el).ok()?;
    Some(())
}

/// Envolve a seleção na marca, ou tira a marca se ela já estiver lá.
///
/// A seleção é preservada nos dois caminhos: quem marcou uma palavra
/// costuma querer marcar outra coisa nela em seguida, e perder a seleção
/// no meio obriga a selecionar de novo a cada clique.
fn aplicar_marca(tag: &str) -> Option<()> {
    let range = selecao_atual()?;
    let bloco = bloco_da_selecao(&range)?;
    let (ini, fim) = intervalo(&bloco, &range)?;

    if tudo_marcado(&bloco, tag, ini, fim) {
        // Tira só do trecho; as bordas voltam marcadas.
        for (a, b) in tirar_marca_do_trecho(&bloco, tag, ini, fim) {
            envolver_intervalo(&bloco, tag, a, b);
        }
    } else {
        // Marcar por cima de uma marca parcial ESTENDE a existente em vez
        // de fragmentar: selecionar metade em negrito e a metade seguinte
        // e clicar negrito deve dar um trecho só, não dois grudados.
        let mut de = ini;
        let mut ate = fim;
        for marca in marcas_na_selecao(&bloco, tag, ini, fim) {
            if let Some((mi, mf)) = span_do_elemento(&bloco, &marca) {
                de = de.min(mi);
                ate = ate.max(mf);
            }
            desembrulhar(&marca);
        }
        bloco.normalize();
        envolver_intervalo(&bloco, tag, de, ate)?;
    }
    bloco.normalize();
    selecionar_intervalo(&bloco, ini, fim);
    avisar_edicao_no_bloco(&bloco)
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
/// Uma borda de cor que precisa voltar depois da cirurgia.
struct Borda {
    ini: u32,
    fim: u32,
    classe: String,
    estilo: String,
}

/// Tira os spans de cor do TRECHO, devolvendo as bordas com a cor que
/// cada uma tinha (ciclo 244).
///
/// Mesma história das outras marcas, com um agravante: aqui a borda
/// precisa voltar com a MESMA cor. Reaplicar sem guardar a identidade
/// pintaria a frase inteira da cor nova.
fn tirar_cor_do_trecho(bloco: &Element, ini: u32, fim: u32) -> (Vec<Borda>, Option<Borda>) {
    let mut bordas = Vec::new();
    // O que estava pintado DENTRO da seleção. Texto e realce são eixos
    // independentes: pintar o fundo de um trecho já azul não pode apagar
    // o azul, e o span novo nasce do zero — então ele herda daqui.
    let mut dentro = None;
    for span in marcas_na_selecao(bloco, "span", ini, fim) {
        if !cor_do_span_existe(&span) {
            continue;
        }
        let classe = span.get_attribute("class").unwrap_or_default();
        let estilo = span.get_attribute("style").unwrap_or_default();
        if dentro.is_none() {
            dentro = Some(Borda { ini, fim, classe: classe.clone(), estilo: estilo.clone() });
        }
        if let Some((mi, mf)) = span_do_elemento(bloco, &span) {
            if mi < ini {
                bordas.push(Borda { ini: mi, fim: ini, classe: classe.clone(), estilo: estilo.clone() });
            }
            if fim < mf {
                bordas.push(Borda { ini: fim, fim: mf, classe, estilo });
            }
        }
        desembrulhar(&span);
    }
    bloco.normalize();
    (bordas, dentro)
}

/// Devolve as bordas ao que elas eram.
fn repor_bordas(bloco: &Element, bordas: &[Borda]) {
    for b in bordas {
        if envolver_intervalo(bloco, "span", b.ini, b.fim).is_none() {
            continue;
        }
        // O span recém-criado é o que contém o trecho: acha por posição.
        if let Some(el) = marcas_na_selecao(bloco, "span", b.ini, b.fim).into_iter().next() {
            if !b.classe.is_empty() {
                let _ = el.set_attribute("class", &b.classe);
            }
            if !b.estilo.is_empty() {
                let _ = el.set_attribute("style", &b.estilo);
            }
        }
    }
}

fn aplicar_cor(cor: Cor) -> Option<()> {
    let doc = web_sys::window()?.document()?;
    let range = selecao_atual()?;
    let bloco = bloco_da_selecao(&range)?;
    let (ini, fim) = intervalo(&bloco, &range)?;

    // O que já estava pintado e sobra FORA da seleção volta com a cor de
    // antes; dentro dela vale o que se pediu agora, por cima do que já
    // havia.
    let (bordas, dentro) = tirar_cor_do_trecho(&bloco, ini, fim);

    if !matches!(cor, Cor::Nenhuma) {
        if envolver_intervalo(&bloco, "span", ini, fim).is_some() {
            if let Some(el) = marcas_na_selecao(&bloco, "span", ini, fim).into_iter().next() {
                // Herda o que estava lá antes de aplicar o eixo novo:
                // `aplicar_no_elemento` troca só o eixo que foi pedido.
                if let Some(antes) = &dentro {
                    if !antes.classe.is_empty() {
                        let _ = el.set_attribute("class", &antes.classe);
                    }
                    if !antes.estilo.is_empty() {
                        let _ = el.set_attribute("style", &antes.estilo);
                    }
                }
                aplicar_no_elemento(&el, &cor);
            }
        }
    }
    repor_bordas(&bloco, &bordas);

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
