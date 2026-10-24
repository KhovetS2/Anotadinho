//! O bloco de código desenhado em várias linhas (ciclo 494).
//!
//! Até aqui o bloco era uma linha só: o `corpo()` da tela troca toda quebra
//! por espaço, e um trecho de vinte linhas saía como "```" seguido do
//! código emendado, sem indentação. A spec `tui-codigo-e-diagramas` (RF1–
//! RF5) pede as linhas, a indentação, a linguagem no topo, o corte por
//! altura com a contagem do resto e o corte por largura com marca.
//!
//! O bloco continua sendo UMA unidade e UMA parada do cursor: o desenho é
//! um grupo de linhas da tela saído de uma linha da página, como a grade
//! do mês do calendário (ciclo 306). Linha de enfeite por linha de código
//! também daria, mas cada uma entraria na conta de rolagem e de dobra.

use ratatui::style::Style;
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthChar;

use crate::realce_codigo::{self, Categoria, Trecho};
use crate::tema::{Realce, Tema};

/// Quantas linhas de código o bloco mostra antes de cortar (RF4). O grupo
/// é indivisível na tela (ou cabe inteiro, ou fica pro próximo quadro), então
/// um bloco mais alto que o terminal nunca apareceria: quem chama ainda
/// limita pela altura do painel.
pub const LINHAS_DO_CODIGO: usize = 30;

/// A tabulação vira espaços: a largura de um `\t` depende do terminal, e o
/// corte por largura contaria errado.
const TAB: &str = "    ";

/// As linhas do código como estão no arquivo, com tabulação expandida.
pub fn linhas_de(codigo: &str) -> Vec<String> {
    let codigo = codigo.strip_suffix('\n').unwrap_or(codigo);
    if codigo.is_empty() {
        return Vec::new();
    }
    codigo.lines().map(|l| l.replace('\t', TAB)).collect()
}

/// Corta `texto` pra caber em `largura` células, com `…` no fim quando
/// cortou (RF5). Código não quebra: quebrado em duas linhas, parece duas
/// instruções.
pub fn cortar(texto: &str, largura: usize) -> String {
    let total: usize = texto.chars().map(|c| c.width().unwrap_or(0)).sum();
    if total <= largura {
        return texto.to_string();
    }
    let mut fora = String::new();
    let mut usado = 0;
    for c in texto.chars() {
        let w = c.width().unwrap_or(0);
        if usado + w + 1 > largura {
            break;
        }
        fora.push(c);
        usado += w;
    }
    fora.push('…');
    fora
}

/// O realce de cada categoria (ciclo 495).
fn papel(c: Categoria) -> Realce {
    match c {
        Categoria::Texto => Realce::CodigoTexto,
        Categoria::Palavra => Realce::CodigoPalavra,
        Categoria::Cadeia => Realce::CodigoCadeia,
        Categoria::Comentario => Realce::CodigoComentario,
        Categoria::Numero => Realce::CodigoNumero,
        Categoria::Tipo => Realce::CodigoTipo,
    }
}

/// Os trechos de uma linha cortados em `largura` células, com `…` no fim
/// quando cortou, na cor do último trecho que coube.
fn cortar_trechos(trechos: &[Trecho], largura: usize, tema: &Tema) -> Vec<Span<'static>> {
    let total: usize = trechos.iter().flat_map(|(_, t)| t.chars()).map(|c| c.width().unwrap_or(0)).sum();
    if total <= largura {
        return trechos.iter().map(|(c, t)| Span::styled(t.clone(), tema.estilo(papel(*c)))).collect();
    }
    let mut fora = Vec::new();
    let mut usado = 0;
    let mut ultima = Categoria::Texto;
    'trechos: for (c, t) in trechos {
        let mut pedaco = String::new();
        for ch in t.chars() {
            let w = ch.width().unwrap_or(0);
            if usado + w + 1 > largura {
                if !pedaco.is_empty() {
                    fora.push(Span::styled(pedaco, tema.estilo(papel(*c))));
                }
                ultima = *c;
                break 'trechos;
            }
            pedaco.push(ch);
            usado += w;
        }
        ultima = *c;
        fora.push(Span::styled(pedaco, tema.estilo(papel(*c))));
    }
    fora.push(Span::styled("…", tema.estilo(papel(ultima))));
    fora
}

/// O rótulo do topo: a linguagem, ou "código" quando a cerca não diz.
pub fn rotulo(lingua: Option<&str>) -> String {
    match lingua.map(str::trim).filter(|l| !l.is_empty()) {
        Some(l) => l.to_string(),
        None => "código".to_string(),
    }
}

/// O bloco inteiro, pronto pra tela:
///
/// ```text
/// ┌ rust ──────────
/// │ fn main() {
/// │     x();
/// │ }
/// └────────────────
/// ```
///
/// `max_linhas` é o teto já combinado com a altura do painel.
pub fn desenhar(
    codigo: &str,
    lingua: Option<&str>,
    nivel: usize,
    largura: usize,
    max_linhas: usize,
    tema: &Tema,
) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(nivel);
    let util = largura.saturating_sub(recuo.chars().count()).max(8);
    let moldura = tema.estilo(Realce::Marca);
    let texto = tema.estilo(Realce::Codigo);
    let linhas = linhas_de(codigo);
    // Com linguagem conhecida, a cor por trecho; sem, a cor única do
    // código (RF3).
    let realcadas = lingua.and_then(|l| realce_codigo::realcar_guardado(&linhas, l));

    let topo = format!("┌ {} ", rotulo(lingua));
    let topo_w: usize = topo.chars().count();
    let mut fora = vec![Line::from(vec![
        Span::raw(recuo.clone()),
        Span::styled(format!("{topo}{}", "─".repeat(util.saturating_sub(topo_w))), moldura),
    ])];

    // A linha do "… mais N" conta no teto: o bloco nunca passa dele.
    let max_linhas = max_linhas.max(2);
    let mostra = if linhas.len() > max_linhas { max_linhas - 1 } else { linhas.len() };
    for (i, l) in linhas[..mostra].iter().enumerate() {
        let mut spans = vec![Span::raw(recuo.clone()), Span::styled("│ ", moldura)];
        match realcadas.as_ref().and_then(|r| r.get(i)) {
            Some(trechos) => spans.extend(cortar_trechos(trechos, util.saturating_sub(2), tema)),
            None => spans.push(Span::styled(cortar(l, util.saturating_sub(2)), texto)),
        }
        fora.push(Line::from(spans));
    }
    if linhas.len() > mostra {
        fora.push(Line::from(vec![
            Span::raw(recuo.clone()),
            Span::styled("│ ", moldura),
            Span::styled(
                format!("… mais {} linha(s) — Enter abre", linhas.len() - mostra),
                tema.estilo(Realce::Dica),
            ),
        ]));
    }
    if linhas.is_empty() {
        fora.push(Line::from(vec![
            Span::raw(recuo.clone()),
            Span::styled("│ ", moldura),
            Span::styled("(vazio)", tema.estilo(Realce::Dica)),
        ]));
    }
    fora.push(Line::from(vec![
        Span::raw(recuo),
        Span::styled(format!("└{}", "─".repeat(util.saturating_sub(1))), moldura),
    ]));
    fora
}

/// O bloco dobrado: uma linha, com a linguagem e o tamanho.
pub fn dobrado(codigo: &str, lingua: Option<&str>, nivel: usize, tema: &Tema) -> Line<'static> {
    let n = linhas_de(codigo).len();
    Line::from(vec![
        Span::raw("  ".repeat(nivel)),
        Span::styled("▸ ``` ", tema.estilo(Realce::Marca)),
        Span::styled(format!("{} · {n} linha(s)", rotulo(lingua)), Style::default().patch(tema.estilo(Realce::Codigo))),
    ])
}

#[cfg(test)]
mod testes {
    use super::*;

    fn texto(linhas: &[Line]) -> Vec<String> {
        linhas.iter().map(|l| l.spans.iter().map(|s| s.content.as_ref()).collect()).collect()
    }

    #[test]
    fn linhas_e_indentacao_ficam() {
        let t = texto(&desenhar("fn f() {\n\tx();\n}\n", Some("rust"), 0, 30, 30, &Tema::novo("mocha")));
        assert_eq!(t[0], format!("┌ rust {}", "─".repeat(23)));
        assert_eq!(&t[1..4], ["│ fn f() {", "│     x();", "│ }"]);
        assert!(t[4].starts_with("└─"));
        assert_eq!(t.len(), 5);
    }

    #[test]
    fn sem_linguagem_diz_codigo() {
        let t = texto(&desenhar("a\nb", None, 1, 30, 30, &Tema::novo("mocha")));
        assert!(t[0].starts_with("  ┌ código "), "{t:?}");
        assert_eq!(t[1], "  │ a");
    }

    #[test]
    fn alto_demais_corta_e_conta_o_resto() {
        let codigo: String = (0..50).map(|i| format!("linha {i}\n")).collect();
        let t = texto(&desenhar(&codigo, None, 0, 40, 10, &Tema::novo("mocha")));
        // Topo + 9 linhas + "… mais" + fundo: 10 linhas de miolo, no teto.
        assert_eq!(t.len(), 12);
        assert_eq!(t[9], "│ linha 8");
        assert_eq!(t[10], "│ … mais 41 linha(s) — Enter abre");
    }

    #[test]
    fn largo_demais_corta_com_marca_e_nao_quebra() {
        let t = texto(&desenhar(&"x".repeat(100), None, 0, 20, 30, &Tema::novo("mocha")));
        assert_eq!(t.len(), 3);
        assert_eq!(t[1], format!("│ {}…", "x".repeat(17)));
        assert_eq!(cortar("ação", 4), "ação");
        // Largura dupla (CJK) conta duas células.
        assert_eq!(cortar("日本語です", 5), "日本…");
    }

    #[test]
    fn com_linguagem_cada_trecho_tem_a_sua_cor() {
        let tema = Tema::novo("mocha");
        let l = desenhar("let s = \"oi\"; // c\n", Some("rust"), 0, 40, 30, &tema);
        let cor = |pedaco: &str| l[1].spans.iter().find(|s| s.content.contains(pedaco)).map(|s| s.style).unwrap();
        assert_eq!(cor("let"), tema.estilo(Realce::CodigoPalavra));
        assert_eq!(cor("\"oi\""), tema.estilo(Realce::CodigoCadeia));
        assert_eq!(cor("// c"), tema.estilo(Realce::CodigoComentario));
        assert_ne!(cor("let"), cor("\"oi\""));
        // Sem linguagem (ou desconhecida): uma cor só, a do código.
        for lingua in [None, Some("nao-existe")] {
            let l = desenhar("let s = 1;", lingua, 0, 40, 30, &tema);
            assert_eq!(l[1].spans.len(), 3, "{:?}", l[1]);
            assert_eq!(l[1].spans[2].style, tema.estilo(Realce::Codigo));
        }
    }

    #[test]
    fn com_cor_o_corte_por_largura_continua() {
        let tema = Tema::novo("mocha");
        let t = texto(&desenhar(&format!("let x = \"{}\";", "a".repeat(80)), Some("rust"), 0, 20, 30, &tema));
        assert_eq!(t[1].chars().count(), 20, "{:?}", t[1]);
        assert!(t[1].ends_with('…'), "{:?}", t[1]);
        assert!(t[1].starts_with("│ let x = \"aaa"), "{:?}", t[1]);
    }

    #[test]
    fn vazio_e_dobrado() {
        let t = texto(&desenhar("", Some("sh"), 0, 20, 30, &Tema::novo("mocha")));
        assert_eq!(t[1], "│ (vazio)");
        let d: String = dobrado("a\nb\nc\n", Some("py"), 0, &Tema::novo("mocha")).spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(d, "▸ ``` py · 3 linha(s)");
    }
}
