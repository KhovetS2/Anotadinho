//! As bibliotecas JS pesadas, carregadas quando a página precisa (ciclo 486).
//!
//! Até o 485 o `index.html` carregava as três na abertura. O 484 mediu
//! ~79 MiB no processo da página só por elas estarem ali, 60 MiB só o
//! mermaid, pagos por quem nunca abre um diagrama nem o grafo.
//!
//! O arquivo continua em `ui/vendor/` e é copiado pelo trunk (ciclo 243):
//! nada vem de fora, só deixa de ser lido antes da hora.

use std::cell::RefCell;
use std::collections::HashMap;

use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Biblioteca {
    Mermaid,
    Grafo,
    Realce,
}

impl Biblioteca {
    /// O caminho servido, o mesmo que o `index.html` usava.
    pub fn arquivo(self) -> &'static str {
        match self {
            Biblioteca::Mermaid => "/mermaid.min.js",
            Biblioteca::Grafo => "/3d-force-graph.min.js",
            Biblioteca::Realce => "/highlight.min.js",
        }
    }

    /// O global que prova que ela carregou.
    pub fn global(self) -> &'static str {
        match self {
            Biblioteca::Mermaid => "mermaid",
            Biblioteca::Grafo => "ForceGraph3D",
            Biblioteca::Realce => "hljs",
        }
    }
}

thread_local! {
    /// Um carregamento por biblioteca: duas páginas pedindo ao mesmo tempo
    /// esperam o MESMO `<script>`, em vez de injetar dois.
    static EM_CURSO: RefCell<HashMap<Biblioteca, js_sys::Promise>> = RefCell::new(HashMap::new());
}

fn presente(b: Biblioteca) -> bool {
    web_sys::window()
        .and_then(|w| js_sys::Reflect::get(&w, &JsValue::from_str(b.global())).ok())
        .map(|v| !v.is_undefined() && !v.is_null())
        .unwrap_or(false)
}

fn injetar(b: Biblioteca) -> js_sys::Promise {
    js_sys::Promise::new(&mut |resolver: js_sys::Function, rejeitar: js_sys::Function| {
        let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
            let _ = rejeitar.call0(&JsValue::NULL);
            return;
        };
        let Ok(script) = doc.create_element("script") else {
            let _ = rejeitar.call0(&JsValue::NULL);
            return;
        };
        let _ = script.set_attribute("src", b.arquivo());
        let _ = script.set_attribute("data-biblioteca", b.global());
        // `once_into_js`: o closure se solta depois de rodar, em vez de
        // ficar preso como um `forget`.
        let ok = Closure::once_into_js(move || {
            let _ = resolver.call0(&JsValue::NULL);
        });
        let erro = Closure::once_into_js(move || {
            let _ = rejeitar.call0(&JsValue::NULL);
        });
        let _ = script.add_event_listener_with_callback("load", ok.unchecked_ref());
        let _ = script.add_event_listener_with_callback("error", erro.unchecked_ref());
        if let Some(head) = doc.head() {
            let _ = head.append_child(&script);
        }
    })
}

/// Carrega `b` se ainda não carregou. Devolve se o global existe no fim.
pub async fn carregar(b: Biblioteca) -> bool {
    if presente(b) {
        return true;
    }
    let promessa = EM_CURSO.with(|m| m.borrow_mut().entry(b).or_insert_with(|| injetar(b)).clone());
    let carregou = wasm_bindgen_futures::JsFuture::from(promessa).await.is_ok();
    if !carregou {
        // Falhou (arquivo faltando no pacote): a próxima página tenta de
        // novo, em vez de herdar a promessa rejeitada para sempre.
        EM_CURSO.with(|m| m.borrow_mut().remove(&b));
    }
    presente(b)
}
