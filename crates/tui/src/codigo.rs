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

use anotadinho_core::diagrama::{self, texto, texto::Papel};

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

/// A caixa de um bloco: topo com `rotulo`, um `aviso` opcional logo
/// abaixo, o miolo (cortado em `max_linhas`, contando a linha do "… mais
/// N") e o fundo:
///
/// ```text
/// ┌ rust ──────────
/// │ fn main() {
/// │     x();
/// │ }
/// └────────────────
/// ```
fn caixa(
    rotulo: &str,
    aviso: Option<&str>,
    miolo: Vec<Vec<Span<'static>>>,
    nivel: usize,
    largura: usize,
    max_linhas: usize,
    tema: &Tema,
) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(nivel);
    let util = largura.saturating_sub(recuo.chars().count()).max(8);
    let moldura = tema.estilo(Realce::Marca);
    let topo = format!("┌ {rotulo} ");
    let topo_w: usize = topo.chars().count();
    let mut fora = vec![Line::from(vec![
        Span::raw(recuo.clone()),
        Span::styled(format!("{topo}{}", "─".repeat(util.saturating_sub(topo_w))), moldura),
    ])];
    let linha = |spans: Vec<Span<'static>>| {
        let mut v = vec![Span::raw(recuo.clone()), Span::styled("│ ", moldura)];
        v.extend(spans);
        Line::from(v)
    };
    if let Some(a) = aviso {
        fora.push(linha(vec![Span::styled(cortar(&format!("⚠ {a}"), util.saturating_sub(2)), tema.estilo(Realce::Dica))]));
    }
    // A linha do "… mais N" conta no teto: o bloco nunca passa dele.
    let max_linhas = max_linhas.max(2);
    let total = miolo.len();
    let mostra = if total > max_linhas { max_linhas - 1 } else { total };
    for spans in miolo.into_iter().take(mostra) {
        fora.push(linha(spans));
    }
    if total > mostra {
        fora.push(linha(vec![Span::styled(
            format!("… mais {} linha(s) — Enter abre", total - mostra),
            tema.estilo(Realce::Dica),
        )]));
    }
    if total == 0 {
        fora.push(linha(vec![Span::styled("(vazio)", tema.estilo(Realce::Dica))]));
    }
    fora.push(Line::from(vec![
        Span::raw(recuo),
        Span::styled(format!("└{}", "─".repeat(util.saturating_sub(1))), moldura),
    ]));
    fora
}

/// A largura do miolo da caixa: a do painel menos o recuo e o `│ `.
fn largura_do_miolo(nivel: usize, largura: usize) -> usize {
    largura.saturating_sub(2 * nivel).max(8).saturating_sub(2)
}

/// As linhas do código, com cor quando a linguagem é conhecida (RF2) e na
/// cor única do código quando não é (RF3), já cortadas na largura.
fn miolo_de_codigo(codigo: &str, lingua: Option<&str>, cabe: usize, tema: &Tema) -> Vec<Vec<Span<'static>>> {
    let linhas = linhas_de(codigo);
    let realcadas = lingua.and_then(|l| realce_codigo::realcar_guardado(&linhas, l));
    linhas
        .iter()
        .enumerate()
        .map(|(i, l)| match realcadas.as_ref().and_then(|r| r.get(i)) {
            Some(trechos) => cortar_trechos(trechos, cabe, tema),
            None => vec![Span::styled(cortar(l, cabe), tema.estilo(Realce::Codigo))],
        })
        .collect()
}

/// O bloco de código inteiro, pronto pra tela. `max_linhas` é o teto já
/// combinado com a altura do painel.
pub fn desenhar(
    codigo: &str,
    lingua: Option<&str>,
    nivel: usize,
    largura: usize,
    max_linhas: usize,
    tema: &Tema,
) -> Vec<Line<'static>> {
    let miolo = miolo_de_codigo(codigo, lingua, largura_do_miolo(nivel, largura), tema);
    caixa(&rotulo(lingua), None, miolo, nivel, largura, max_linhas, tema)
}

/// O papel de cada pedaço do diagrama, em cor do tema (ciclo 496).
fn papel_do_desenho(p: Papel, tema: &Tema) -> Style {
    let r = match p {
        Papel::Fundo | Papel::Nome => Realce::CodigoTexto,
        Papel::Caixa => Realce::CodigoTipo,
        Papel::Vida | Papel::Eixo | Papel::BarraFeita => Realce::Marca,
        Papel::Seta => Realce::CodigoPalavra,
        Papel::Rotulo => Realce::CodigoCadeia,
        Papel::Nota | Papel::Marco => Realce::Parte,
        Papel::Secao => Realce::CodigoPalavra,
        Papel::Titulo => Realce::Titulo(3),
        Papel::Barra => Realce::Barra,
        Papel::BarraAtiva => Realce::CodigoTipo,
        Papel::BarraCritica => Realce::BadgeErro,
    };
    tema.estilo(r)
}

/// Um bloco ` ```mermaid ` (ciclo 496, RF6–RF8): desenhado em caracteres
/// quando o construtor entende o diagrama e ele cabe na largura. Senão, o
/// código, com uma linha dizendo por quê.
pub fn desenhar_mermaid(fonte: &str, nivel: usize, largura: usize, max_linhas: usize, tema: &Tema) -> Vec<Line<'static>> {
    let cabe = largura_do_miolo(nivel, largura);
    let motivo = match diagrama::ler(fonte) {
        Ok(d) => match texto::desenhar(&d, cabe) {
            Ok(linhas) => {
                let miolo = linhas
                    .into_iter()
                    .map(|l| l.into_iter().map(|(p, t)| Span::styled(t, papel_do_desenho(p, tema))).collect())
                    .collect();
                let rotulo = format!("mermaid · {}", d.tipo().nome().to_lowercase());
                return caixa(&rotulo, None, miolo, nivel, largura, max_linhas, tema);
            }
            Err(nao) => nao.mensagem(),
        },
        Err(fora) => format!("o construtor não monta este diagrama — {}", fora.mensagem()),
    };
    let miolo = miolo_de_codigo(fonte, Some("mermaid"), cabe, tema);
    caixa("mermaid", Some(&motivo), miolo, nivel, largura, max_linhas, tema)
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
    fn mermaid_dentro_do_modelo_sai_desenhado() {
        let fonte = "sequenceDiagram\n  A->>B: oi\n";
        let t = texto(&desenhar_mermaid(fonte, 0, 60, 30, &Tema::novo("mocha")));
        assert!(t[0].starts_with("┌ mermaid · sequência "), "{t:?}");
        assert!(t.iter().any(|l| l.contains("│ A │")), "{t:?}");
        assert!(t.iter().any(|l| l.contains("▶│")), "{t:?}");
        assert!(!t.iter().any(|l| l.contains("A->>B")), "o código apareceu junto:\n{}", t.join("\n"));
    }

    #[test]
    fn mermaid_fora_do_modelo_ou_largo_demais_mostra_o_codigo_e_o_motivo() {
        let tema = Tema::novo("mocha");
        let t = texto(&desenhar_mermaid("pie\n  \"a\" : 1\n", 0, 60, 30, &tema));
        assert!(t[1].starts_with("│ ⚠ o construtor não monta este diagrama"), "{t:?}");
        assert_eq!(t[2], "│ pie");
        let largo = "sequenceDiagram\n  A->>B: uma mensagem que não cabe em vinte colunas\n";
        let t = texto(&desenhar_mermaid(largo, 0, 24, 30, &tema));
        assert!(t[1].contains("não cabe"), "{t:?}");
        assert!(t.iter().all(|l| l.chars().count() <= 24), "passou da largura:\n{}", t.join("\n"));
    }

    #[test]
    fn vazio_e_dobrado() {
        let t = texto(&desenhar("", Some("sh"), 0, 20, 30, &Tema::novo("mocha")));
        assert_eq!(t[1], "│ (vazio)");
        let d: String = dobrado("a\nb\nc\n", Some("py"), 0, &Tema::novo("mocha")).spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(d, "▸ ``` py · 3 linha(s)");
    }
}
