//! Página `type: graph` — o grafo das conexões via `[[wikilinks]]`
//! (ciclo 120, refeito no 301).
//!
//! ## Por que uma biblioteca
//!
//! O ciclo 120 desenhou os nós num CÍRCULO, à mão, e escreveu o motivo:
//! "evita dependência nova". O ciclo 300 trocou o círculo por força
//! dirigida em três dimensões, também à mão — e continuava sendo uma
//! implementação mínima de algo que tem solução madura.
//!
//! `3d-force-graph` é essa solução: junta `d3-force-3d` (layout com
//! Barnes-Hut, `O(n log n)`) com `three.js` (WebGL). O que ela dá e o
//! desenho à mão não dava, em ordem de importância:
//!
//! - **WebGL**: com 239 nós e giro contínuo, o SVG redesenhava 400
//!   elementos por quadro. WebGL não sente;
//! - órbita, zoom e enquadramento automático de câmera;
//! - oclusão de rótulo de verdade, em vez da heurística de "só quem
//!   está na frente".
//!
//! O argumento de não introduzir dependência não vale mais num projeto
//! que já vendoriza 3,2 MB de mermaid; esta tem 1,3 MB e traz o three
//! dentro.
//!
//! ## O que fica aqui
//!
//! Montar o grafo (nós e arestas a partir dos wikilinks) e entregar. O
//! layout, o desenho e a câmera são dela. O clique num nó volta pra cá,
//! que é o que liga o grafo à navegação do app.

use std::collections::{HashMap, HashSet};

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use yew::prelude::*;

use crate::api::{self, PageMeta};

/// Props da `GraphView`.
#[derive(Properties, PartialEq, Clone)]
pub struct GraphViewProps {
    /// Path do vault.
    pub vault_path: String,
    /// Navega pra uma página ao clicar num nó.
    pub on_page_selected: Callback<PageMeta>,
}

#[function_component(GraphView)]
pub fn graph_view(props: &GraphViewProps) -> Html {
    let container = use_node_ref();
    let carregando = use_state(|| true);

    {
        let vault_path = props.vault_path.clone();
        let container = container.clone();
        let carregando = carregando.clone();
        let on_page_selected = props.on_page_selected.clone();
        use_effect_with(vault_path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                // Uma varredura só traz título e wikilinks de todas as
                // páginas — a alternativa seria N+1 travessias
                // WASM↔Tauri lendo arquivo por arquivo (ciclo 120).
                let paginas = api::scan_vault(&vault_path).await.unwrap_or_default();
                let Some(el) = container.cast::<web_sys::Element>() else {
                    return;
                };

                let (nos, arestas) = montar(&paginas);
                desenhar(&el, &paginas, &nos, &arestas, on_page_selected);
                carregando.set(false);
            });
            || {}
        });
    }

    html! {
        <div class="graph-view">
            <p class="graph-view__hint graph-view__hint--muted">
                { "Arraste pra girar, scroll pra zoom, clique num nó pra abrir a página" }
            </p>
            if *carregando {
                <p class="editor__status">{ "Carregando..." }</p>
            }
            <div ref={container} class="graph-view__canvas" />
        </div>
    }
}

/// Os nós (índice → página) e as arestas sem direção nem duplicata.
///
/// Sem direção porque a pergunta que o grafo responde é "estas duas
/// páginas se falam?", e A→B e B→A são a mesma resposta.
fn montar(paginas: &[api::PageIndexEntry]) -> (Vec<usize>, Vec<(usize, usize)>) {
    let mut por_titulo: HashMap<String, usize> = HashMap::new();
    for (i, p) in paginas.iter().enumerate() {
        por_titulo.insert(p.title.to_lowercase(), i);
    }
    let mut arestas = Vec::new();
    let mut vistas: HashSet<(usize, usize)> = HashSet::new();
    for (i, p) in paginas.iter().enumerate() {
        for titulo in &p.wikilinks {
            if let Some(&j) = por_titulo.get(&titulo.to_lowercase()) {
                if j != i {
                    let par = if i < j { (i, j) } else { (j, i) };
                    if vistas.insert(par) {
                        arestas.push(par);
                    }
                }
            }
        }
    }
    ((0..paginas.len()).collect(), arestas)
}

/// Entrega o grafo pra biblioteca e liga o clique de volta no app.
fn desenhar(
    el: &web_sys::Element,
    paginas: &[api::PageIndexEntry],
    nos: &[usize],
    arestas: &[(usize, usize)],
    ao_escolher: Callback<PageMeta>,
) {
    let Some(window) = web_sys::window() else { return };
    let Some(fabrica) = js_sys::Reflect::get(&window, &JsValue::from_str("ForceGraph3D"))
        .ok()
        .and_then(|v| v.dyn_into::<js_sys::Function>().ok())
    else {
        // Sem a biblioteca o grafo não desenha, e isso precisa ser
        // visível: uma tela em branco sem explicação é o pior modo de
        // falhar.
        el.set_text_content(Some(
            "3d-force-graph não carregou — o grafo precisa dele pra desenhar.",
        ));
        return;
    };

    let dados = js_sys::Object::new();
    let js_nos = js_sys::Array::new();
    for &i in nos {
        let n = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&n, &"id".into(), &JsValue::from_f64(i as f64));
        let _ = js_sys::Reflect::set(&n, &"name".into(), &paginas[i].title.clone().into());
        let _ = js_sys::Reflect::set(&n, &"path".into(), &paginas[i].path.clone().into());
        let _ = js_sys::Reflect::set(&n, &"section".into(), &paginas[i].section.clone().into());
        js_nos.push(&n);
    }
    let js_arestas = js_sys::Array::new();
    for (a, b) in arestas {
        let l = js_sys::Object::new();
        let _ = js_sys::Reflect::set(&l, &"source".into(), &JsValue::from_f64(*a as f64));
        let _ = js_sys::Reflect::set(&l, &"target".into(), &JsValue::from_f64(*b as f64));
        js_arestas.push(&l);
    }
    let _ = js_sys::Reflect::set(&dados, &"nodes".into(), &js_nos);
    let _ = js_sys::Reflect::set(&dados, &"links".into(), &js_arestas);

    // `ForceGraph3D()(elemento)` devolve o grafo; daí pra frente é
    // encadeamento de configuração, no estilo do d3.
    let Ok(grafo) = fabrica.call0(&JsValue::NULL) else { return };
    let Ok(grafo) = js_sys::Reflect::apply(
        &grafo.dyn_into::<js_sys::Function>().unwrap_or_else(|_| js_sys::Function::new_no_args("")),
        &JsValue::NULL,
        &js_sys::Array::of1(el),
    ) else {
        return;
    };

    chamar(&grafo, "graphData", &dados);
    chamar(&grafo, "nodeLabel", &"name".into());
    // A biblioteca escreve a própria legenda de navegação, em inglês, e
    // o app já tem a dele em português logo acima. Duas legendas em duas
    // línguas dizendo a mesma coisa é pior que nenhuma.
    chamar(&grafo, "showNavInfo", &JsValue::FALSE);
    // As cores saem do CSS da janela, não de um tema da biblioteca: é a
    // mesma paleta que a TUI lê no ciclo 288, e ter duas fontes de cor
    // é como a política do bloco divergiu até o ciclo 278.
    chamar(&grafo, "backgroundColor", &cor_css("--bg-base", "#272930").into());
    chamar(&grafo, "nodeColor", &JsValue::from(js_sys::Function::new_with_args(
        "n",
        &format!(
            "return n.section === 'journals' ? '{}' : '{}';",
            cor_css("--accent-purple", "#9327FF"),
            cor_css("--accent-blue", "#00B5FF"),
        ),
    )));
    chamar(&grafo, "linkColor", &JsValue::from(js_sys::Function::new_with_args(
        "_l",
        &format!("return '{}';", cor_css("--border", "#3D404F")),
    )));

    // O clique volta pro app: é o que liga o grafo à navegação, e sem
    // isso ele seria um desenho bonito e inútil.
    let paginas: Vec<PageMeta> = paginas
        .iter()
        .map(|p| PageMeta {
            path: p.path.clone(),
            title: p.title.clone(),
            section: p.section.clone(),
        })
        .collect();
    let ao_clicar = Closure::<dyn Fn(JsValue)>::new(move |no: JsValue| {
        let indice = js_sys::Reflect::get(&no, &"id".into())
            .ok()
            .and_then(|v| v.as_f64())
            .map(|f| f as usize);
        if let Some(p) = indice.and_then(|i| paginas.get(i)) {
            ao_escolher.emit(p.clone());
        }
    });
    chamar(&grafo, "onNodeClick", ao_clicar.as_ref());
    // O `Closure` morre com esta função e o callback vira ponteiro
    // solto; `forget` entrega a posse pro JS, que é dono do grafo
    // enquanto a página existir.
    ao_clicar.forget();
}

/// Chama um método encadeável do grafo, ignorando o retorno.
fn chamar(grafo: &JsValue, metodo: &str, arg: &JsValue) {
    if let Ok(f) = js_sys::Reflect::get(grafo, &JsValue::from_str(metodo)) {
        if let Ok(f) = f.dyn_into::<js_sys::Function>() {
            let _ = f.call1(grafo, arg);
        }
    }
}

/// Lê uma variável de cor do CSS da janela, com socorro.
fn cor_css(nome: &str, socorro: &str) -> String {
    web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.document_element())
        .and_then(|raiz| web_sys::window()?.get_computed_style(&raiz).ok().flatten())
        .and_then(|estilo| estilo.get_property_value(nome).ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .unwrap_or_else(|| socorro.to_string())
}
