//! Dentro de um bloco de código ou diagrama (ciclo 498).
//!
//! O bloco é UMA parada do cursor e um grupo indivisível na tela, com teto
//! na altura do painel: um diagrama mais alto que o painel saía cortado em
//! "… mais N", e um mais largo virava código. Entrar no bloco troca o que as
//! teclas movem — a vista, não o cursor —, e o desenho inteiro passa a ser
//! alcançável nas duas direções.

use anotadinho_core::unidade::Tipo;

use super::{markdown, Estado, VistaDoBloco};

/// A página aberta, pelo caminho.
fn pagina_aberta(e: &Estado) -> String {
    e.paginas.get(e.pagina).map(|p| p.path.clone()).unwrap_or_default()
}

/// O bloco de código no cursor, se é nele que se está.
fn bloco_no_cursor(e: &Estado) -> bool {
    e.arvore.em(&e.cursor).is_some_and(|u| matches!(u.tipo, Tipo::Codigo(_)))
        && e.linhas.iter().any(|l| l.caminho == e.cursor && !l.enfeite)
}

/// A vista ainda é do bloco que está na tela.
pub(super) fn valida(e: &Estado) -> bool {
    e.vista_bloco
        .as_ref()
        .is_some_and(|v| v.pagina == pagina_aberta(e) && v.caminho == e.cursor && bloco_no_cursor(e))
}

/// Enter num bloco de código: entra. Dobrado, desdobra junto — entrar num
/// bloco que não se vê não teria o que rolar.
pub(super) fn entrar(e: &mut Estado) -> bool {
    if !bloco_no_cursor(e) {
        return false;
    }
    e.dobrados.remove(&e.cursor);
    e.vista_bloco = Some(VistaDoBloco { pagina: pagina_aberta(e), caminho: e.cursor.clone(), ..Default::default() });
    true
}

/// Uma tecla com a vista aberta. `false` é "não é da vista": a tecla segue
/// pro resto (abas, foco, `Ctrl+S`), e a vista continua.
pub(super) fn tecla(e: &mut Estado, tecla: &str) -> bool {
    if !valida(e) {
        e.vista_bloco = None;
        return false;
    }
    let Some(v) = e.vista_bloco.as_mut() else { return false };
    let m = v.medida.get();
    let meia = (m.alto / 2).max(1);
    // De lado anda de quatro em quatro: uma coluna por tecla num desenho
    // de 200 colunas é tecla demais. As setas andam uma, pro ajuste fino.
    let passo = 4;
    let (x, y) = (v.vista.x, v.vista.y);
    let (nx, ny) = match tecla {
        "j" | "ArrowDown" => (x, y + 1),
        "k" | "ArrowUp" => (x, y.saturating_sub(1)),
        "l" => (x + passo, y),
        "h" => (x.saturating_sub(passo), y),
        "ArrowRight" => (x + 1, y),
        "ArrowLeft" => (x.saturating_sub(1), y),
        "Ctrl+d" | "PageDown" => (x, y + meia),
        "Ctrl+u" | "PageUp" => (x, y.saturating_sub(meia)),
        "L" => (x + m.largo / 2, y),
        "H" => (x.saturating_sub(m.largo / 2), y),
        "g" | "Home" => (x, 0),
        "G" | "End" => (x, m.max_y()),
        "0" => (0, y),
        "$" => (m.max_x(), y),
        "Escape" | "q" => {
            e.vista_bloco = None;
            return true;
        }
        "Enter" | "i" | "A" => {
            // Editar sai da vista: o editor de várias linhas é o modal de
            // 390, e ao voltar dele o bloco é outro texto.
            e.vista_bloco = None;
            markdown::enter_na_tabela_md(e);
            return true;
        }
        _ => return false,
    };
    v.vista.x = nx.min(m.max_x());
    v.vista.y = ny.min(m.max_y());
    true
}
