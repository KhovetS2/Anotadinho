//! Autocompletar wikilink ao digitar (ciclo 352), como o popup `[[` da
//! janela: numa inserção, `[[` seguido de texto abre as páginas que
//! batem; `↑`/`↓` (ou `Ctrl+P`/`Ctrl+N`) escolhem, `Enter` ou `Tab`
//! completam `[[Título]]` e `Esc` fecha a lista sem sair da inserção.

use ratatui::layout::Rect;
use ratatui::Frame;

use super::edicao::Pergunta;
use super::Estado;
use crate::componentes::{self, Item, Lista};

/// Quantas sugestões aparecem.
const MAXIMO: usize = 8;

/// Onde começa o `[[` aberto antes do cursor (em caracteres) e o que foi
/// digitado depois dele.
pub(super) fn contexto(p: &Pergunta) -> Option<(usize, String)> {
    let antes: Vec<char> = p.texto.chars().take(p.cursor).collect();
    let texto: String = antes.iter().collect();
    let abre = texto.rfind("[[")?;
    let depois = &texto[abre + 2..];
    if depois.contains("]]") || depois.contains('\n') {
        return None;
    }
    Some((texto[..abre].chars().count(), depois.to_string()))
}

/// As páginas que batem com o que foi digitado, `(caminho, título)`.
pub(super) fn sugestoes(e: &Estado) -> Vec<(String, String)> {
    let Some(p) = &e.pergunta else { return Vec::new() };
    let Some((inicio, consulta)) = contexto(p) else { return Vec::new() };
    if e.wikilink_dispensado == Some(inicio) {
        return Vec::new();
    }
    let alvo = componentes::sem_acento(&consulta.to_lowercase());
    let mut v: Vec<(usize, String, String)> = e
        .paginas
        .iter()
        .filter_map(|pg| {
            let t = componentes::sem_acento(&pg.title.to_lowercase());
            let pos = t.find(&alvo)?;
            Some((if pos == 0 { 0 } else { 1 }, pg.path.clone(), pg.title.clone()))
        })
        .collect();
    v.sort_by(|a, b| a.0.cmp(&b.0).then(a.2.len().cmp(&b.2.len())));
    v.into_iter().take(MAXIMO).map(|(_, p, t)| (p, t)).collect()
}

/// Uma tecla com a lista aberta. `true` é "usei".
pub(super) fn tecla(e: &mut Estado, tecla: &str) -> bool {
    let lista = sugestoes(e);
    if lista.is_empty() {
        return false;
    }
    match tecla {
        "ArrowDown" | "Ctrl+n" => e.wikilink_sel = (e.wikilink_sel + 1) % lista.len(),
        "ArrowUp" | "Ctrl+p" => e.wikilink_sel = (e.wikilink_sel + lista.len() - 1) % lista.len(),
        "Escape" => {
            if let Some((inicio, _)) = e.pergunta.as_ref().and_then(contexto) {
                e.wikilink_dispensado = Some(inicio);
            }
        }
        "Enter" | "Tab" => {
            let (_, titulo) = lista[e.wikilink_sel.min(lista.len() - 1)].clone();
            completar(e, &titulo);
        }
        _ => {
            e.wikilink_sel = 0;
            return false;
        }
    }
    true
}

/// Troca `[[consulta` por `[[Título]]` e põe o cursor depois.
fn completar(e: &mut Estado, titulo: &str) {
    let Some(p) = e.pergunta.as_mut() else { return };
    let Some((inicio, _)) = contexto(p) else { return };
    let chars: Vec<char> = p.texto.chars().collect();
    let link = format!("[[{}]]", anotadinho_core::links::escapar_barra(titulo));
    let novo: String = chars[..inicio].iter().collect::<String>() + &link + &chars[p.cursor..].iter().collect::<String>();
    p.cursor = inicio + link.chars().count();
    p.texto = novo;
    e.wikilink_sel = 0;
}

/// A lista, embaixo da área do conteúdo, perto do rodapé da inserção.
pub(super) fn desenhar(f: &mut Frame, e: &Estado, area: Rect) {
    let lista = sugestoes(e);
    if lista.is_empty() {
        return;
    }
    let altura = lista.len() as u16 + 2;
    let largura = 56.min(area.width.saturating_sub(4));
    if area.height < altura + 2 || largura < 20 {
        return;
    }
    let caixa = Rect { x: area.x + 2, y: area.y + area.height - altura - 1, width: largura, height: altura };
    let mut l = Lista::menu(lista.into_iter().map(|(path, titulo)| Item::novo("≡", titulo, path.clone()).com_detalhe(path)).collect());
    l.selecionado = e.wikilink_sel;
    let dentro = componentes::desenhar_modal(f, caixa, "Wikilink", "↑↓ · Enter completa · Esc", &e.tema);
    componentes::desenhar_lista(f, dentro, &l, "", &e.tema);
}
