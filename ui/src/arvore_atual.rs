//! A árvore da página aberta, onde o teclado alcança (ciclo 281).
//!
//! O editor sabe o markdown; o handler de navegação, que decide para
//! onde o foco vai, vive em `app.rs` e não sabe. Enquanto foi assim, o
//! movimento era decidido por aritmética de índice sobre uma lista de
//! elementos do DOM, e `navegacao::mover` — o modelo — ficava sem
//! consultar ninguém.
//!
//! Aqui o editor PUBLICA o corpo da página e quem precisa pede a
//! árvore. Não é estado novo: é cache do que o markdown já diz.
//!
//! A árvore é construída sob demanda e guardada junto do corpo que a
//! gerou. Publicar é comparar duas strings; analisar só acontece quando
//! alguém pergunta E o corpo mudou — então digitar não paga análise, e
//! andar numa página parada paga uma vez só.

use std::cell::RefCell;
use std::rc::Rc;

use anotadinho_core::unidade::{Caminho, Unidade};

thread_local! {
    /// O corpo publicado, e a árvore dele quando já foi pedida.
    static PAGINA: RefCell<(String, Option<Rc<Unidade>>)> =
        const { RefCell::new((String::new(), None)) };
}

/// O editor diz qual é o corpo da página aberta.
///
/// Corpo igual não invalida nada — é o caso comum, porque o render
/// acontece muito mais vezes do que o texto muda.
pub fn publicar(corpo: &str) {
    PAGINA.with(|p| {
        let mut p = p.borrow_mut();
        if p.0 != corpo {
            p.0 = corpo.to_string();
            p.1 = None;
        }
    });
}

/// A árvore do corpo publicado, construída na primeira pergunta.
pub fn arvore() -> Option<Rc<Unidade>> {
    PAGINA.with(|p| {
        {
            let p = p.borrow();
            if p.0.is_empty() {
                return None;
            }
            if let Some(a) = &p.1 {
                return Some(a.clone());
            }
        }
        let mut p = p.borrow_mut();
        let nova = Rc::new(anotadinho_core::analise::analisar(&p.0));
        p.1 = Some(nova.clone());
        Some(nova)
    })
}

/// Lê `data-nav-caminho` como o `Caminho` do núcleo.
///
/// `"2"` é o terceiro bloco; `"2.1"`, o segundo item dele. Devolve
/// `None` para o que não é caminho — atributo ausente, vazio, ou com
/// pedaço que não é número. Nada de aceitar meio caminho: um endereço
/// mal lido move o foco pro lugar errado, que é pior do que não mover.
pub fn caminho_do_elemento(el: &web_sys::Element) -> Option<Caminho> {
    let cru = el.get_attribute(crate::components::editor::ATTR_CAMINHO)?;
    if cru.is_empty() {
        return None;
    }
    cru.split('.').map(|p| p.parse::<usize>().ok()).collect()
}

/// Escreve um caminho como o DOM o estampa.
pub fn caminho_para_texto(caminho: &Caminho) -> String {
    caminho
        .iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".")
}
