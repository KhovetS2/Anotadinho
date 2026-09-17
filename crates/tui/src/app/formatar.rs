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
    // A "Cor personalizada" e o "Tirar a cor" da janela (ciclo 394).
    itens.push(Item::novo("●", "Cor personalizada…", "cor-livre").com_detalhe("#rrggbb"));
    itens.push(Item::novo("○", "Tirar a cor", "tirar-cor"));
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
    if chave == "cor-livre" {
        e.modal = Some(Modal::Entrada { titulo: "Cor (#rrggbb)".into(), campo: Campo::com("#"), acao: AcaoDaEntrada::CorLivre });
        return;
    }
    if chave == "tirar-cor" {
        retomar(e);
        if let Some(p) = e.pergunta.as_mut() {
            tirar_cor(p);
        }
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

/// A cor personalizada digitada: `<span style="color:#hex">`.
pub(super) fn aplicar_cor_livre(e: &mut Estado, hex: &str) {
    let hex = hex.trim().trim_start_matches('#');
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        retomar(e);
        e.aviso = Some("cor: use #rrggbb".into());
        return;
    }
    retomar(e);
    let Some(p) = e.pergunta.as_mut() else { return };
    embrulhar(p, &format!("<span style=\"color:#{}\">", hex.to_lowercase()), "</span>");
}

/// Tira o `<span …>` de cor que envolve o cursor.
pub(super) fn tirar_cor(p: &mut Pergunta) {
    let chars: Vec<char> = p.texto.chars().collect();
    let texto: String = chars.iter().collect();
    let byte = |c: usize| texto.char_indices().nth(c).map_or(texto.len(), |(i, _)| i);
    let aqui = byte(p.cursor.min(chars.len()));
    let Some(abre) = texto[..aqui].rfind("<span") else { return };
    let Some(fim_abre) = texto[abre..].find('>').map(|i| abre + i + 1) else { return };
    let Some(fecha) = texto[fim_abre..].find("</span>").map(|i| fim_abre + i) else { return };
    if fecha + 7 < aqui {
        return;
    }
    let miolo = &texto[fim_abre..fecha];
    let novo = format!("{}{miolo}{}", &texto[..abre], &texto[fecha + 7..]);
    p.cursor = texto[..abre].chars().count() + miolo.chars().count();
    p.texto = novo;
}
