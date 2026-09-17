//! Formatar o texto na inserção (ciclo 357), o que a barra de seleção da
//! janela faz: negrito, itálico, riscado, código, link e cor.
//!
//! Terminal não tem seleção de mouse no texto sendo digitado, então a
//! marca vale pra PALAVRA do cursor (a que ele toca); sem palavra, entra
//! o par vazio com o cursor no meio. Aplicar de novo tira a marca.
//! `Ctrl+B` é negrito direto; `Ctrl+T` abre o menu com o resto. As marcas
//! são as mesmas que a janela grava: `**`, `*`, `~~`, `` ` ``, `[x](url)`
//! e `<span class="cor--…">`.

use super::edicao::Pergunta;
use super::modais::{AcaoDaEntrada, AcaoDaEscolha, Modal};
use super::Estado;
use crate::componentes::{Campo, Item, Lista};

/// A paleta nomeada da janela: vira classe do tema.
const PALETA: &[(&str, &str)] = &[
    ("vermelho", "Vermelho"),
    ("ambar", "Âmbar"),
    ("verde", "Verde"),
    ("azul", "Azul"),
    ("roxo", "Roxo"),
    ("rosa", "Rosa"),
    ("cinza", "Cinza"),
];

/// Onde fica a palavra que o cursor toca, em caracteres.
fn palavra(texto: &[char], cursor: usize) -> Option<(usize, usize)> {
    let espaco = |c: char| c.is_whitespace();
    let mut inicio = cursor.min(texto.len());
    while inicio > 0 && !espaco(texto[inicio - 1]) {
        inicio -= 1;
    }
    let mut fim = cursor.min(texto.len());
    while fim < texto.len() && !espaco(texto[fim]) {
        fim += 1;
    }
    (fim > inicio).then_some((inicio, fim))
}

/// Põe (ou tira) `abre`…`fecha` em volta da palavra do cursor.
pub(super) fn embrulhar(p: &mut Pergunta, abre: &str, fecha: &str) {
    let chars: Vec<char> = p.texto.chars().collect();
    let (na, nf) = (abre.chars().count(), fecha.chars().count());
    let junta = |a: &[char]| a.iter().collect::<String>();
    let Some((inicio, fim)) = palavra(&chars, p.cursor) else {
        p.texto = format!("{}{abre}{fecha}{}", junta(&chars[..p.cursor]), junta(&chars[p.cursor..]));
        p.cursor += na;
        return;
    };
    let miolo = junta(&chars[inicio..fim]);
    // Já marcada: tira.
    if miolo.chars().count() >= na + nf && miolo.starts_with(abre) && miolo.ends_with(fecha) {
        let sem: String = miolo.chars().skip(na).take(miolo.chars().count() - na - nf).collect();
        p.texto = format!("{}{sem}{}", junta(&chars[..inicio]), junta(&chars[fim..]));
        p.cursor = inicio + sem.chars().count();
        return;
    }
    p.texto = format!("{}{abre}{miolo}{fecha}{}", junta(&chars[..inicio]), junta(&chars[fim..]));
    p.cursor = fim + na + nf;
}

/// `Ctrl+T`: o menu, com a inserção guardada enquanto ele está aberto.
pub(super) fn abrir_menu(e: &mut Estado) {
    let Some(p) = e.pergunta.take() else { return };
    e.pergunta_suspensa = Some(p);
    let mut itens = vec![
        Item::novo("B", "Negrito", "negrito").com_detalhe("Ctrl+B"),
        Item::novo("I", "Itálico", "italico"),
        Item::novo("S", "Riscado", "riscado"),
        Item::novo("‹›", "Código", "codigo"),
        Item::novo("↗", "Link…", "link"),
    ];
    itens.extend(PALETA.iter().map(|(slug, nome)| Item::novo("●", format!("Cor: {nome}"), format!("cor:{slug}"))));
    itens.extend(PALETA.iter().map(|(slug, nome)| Item::novo("▆", format!("Fundo: {nome}"), format!("fundo:{slug}"))));
    e.modal = Some(Modal::Escolha { titulo: "Formatar".into(), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::Formatar });
}

/// Devolve a inserção guardada.
pub(super) fn retomar(e: &mut Estado) {
    if let Some(p) = e.pergunta_suspensa.take() {
        e.pergunta = Some(p);
    }
}

/// O item escolhido no menu.
pub(super) fn escolher(e: &mut Estado, chave: &str) {
    if chave == "link" {
        e.modal = Some(Modal::Entrada { titulo: "Link (URL)".into(), campo: Campo::default(), acao: AcaoDaEntrada::Link });
        return;
    }
    retomar(e);
    let Some(p) = e.pergunta.as_mut() else { return };
    match chave {
        "negrito" => embrulhar(p, "**", "**"),
        "italico" => embrulhar(p, "*", "*"),
        "riscado" => embrulhar(p, "~~", "~~"),
        "codigo" => embrulhar(p, "`", "`"),
        outra => {
            if let Some((eixo, slug)) = outra.split_once(':') {
                embrulhar(p, &format!("<span class=\"{eixo}--{slug}\">"), "</span>");
            }
        }
    }
}

/// O link digitado: `[palavra](url)`.
pub(super) fn aplicar_link(e: &mut Estado, url: &str) {
    retomar(e);
    let Some(p) = e.pergunta.as_mut() else { return };
    embrulhar(p, "[", &format!("]({url})"));
}
