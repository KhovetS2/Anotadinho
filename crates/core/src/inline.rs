//! O markdown DENTRO de um bloco, em trechos com marca (ciclo 287).
//!
//! A árvore diz que um bloco é um parágrafo; ela não diz que "urgente"
//! ali no meio está em negrito. Pra desenhar `**urgente**` como negrito
//! em vez de mostrar os asteriscos, alguém precisa quebrar o texto do
//! bloco em pedaços e dizer o que cada um é.
//!
//! Isto mora no NÚCLEO e não no renderizador pelo mesmo motivo de tudo
//! nesta sequência: analisador inline em dois lugares volta a divergir,
//! como a política do bloco divergiu até o ciclo 278. O terminal mapeia
//! trecho → `Span`; a janela pode mapear trecho → `<strong>`.
//!
//! ## Byte e coluna são coisas diferentes
//!
//! Esconder os marcadores muda o tamanho: `**a**` são 5 bytes no arquivo
//! e 1 coluna na tela. As operações de edição (ciclo 285) trabalham em
//! BYTE — `dividir(raiz, caminho, em)` corta no byte `em` — e o cursor
//! de um editor vive em COLUNA.
//!
//! `byte_da_coluna` e `coluna_do_byte` fazem a ponte. Sem elas, estilizar
//! e editar não convivem: o corte cairia no lugar errado assim que
//! houvesse um marcador antes do cursor.

use std::ops::Range;

use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};

/// O que um trecho é, além de texto.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Marca {
    /// `**assim**`.
    Negrito,
    /// `*assim*`.
    Italico,
    /// `~~assim~~`.
    Tachado,
    /// `` `assim` `` — literal, nada dentro é interpretado.
    Codigo,
    /// `[texto](destino)`.
    Link,
    /// `[[Página]]` — o link do vault, que o pulldown-cmark não conhece.
    Wikilink,
    /// `<span class="cor--vermelho">` — a cor da paleta da janela (ciclo
    /// 357), pelo nome (`vermelho`).
    Cor(&'static str),
    /// `<span class="fundo--vermelho">` — o realce.
    Fundo(&'static str),
    /// `<span style="color:#rrggbb">` — a "Cor personalizada" da janela
    /// (ciclo 394).
    CorLivre(u8, u8, u8),
}

/// Os nomes da paleta que a barra de seleção da janela grava.
pub const PALETA: &[&str] = &["vermelho", "ambar", "verde", "azul", "roxo", "rosa", "cinza"];

/// As marcas de cor de um `<span class="…">`.
fn cores_do_span(html: &str) -> Vec<Marca> {
    // A cor livre vem no `style`.
    let livre = html.split("style=\"").nth(1).and_then(|r| r.split('"').next()).and_then(|estilo| {
        let valor = estilo.split(';').find_map(|d| {
            let (k, v) = d.split_once(':')?;
            (k.trim().eq_ignore_ascii_case("color")).then(|| v.trim().trim_start_matches('#').to_string())
        })?;
        let canal = |i: usize| u8::from_str_radix(valor.get(i..i + 2)?, 16).ok();
        (valor.len() == 6).then_some(())?;
        Some(Marca::CorLivre(canal(0)?, canal(2)?, canal(4)?))
    });
    let Some(classe) = html.split("class=\"").nth(1).and_then(|r| r.split('"').next()) else { return livre.into_iter().collect() };
    livre.into_iter().collect::<Vec<_>>().into_iter().chain(classe
        .split_whitespace()
        .filter_map(|c| {
            let achar = |slug: &str| PALETA.iter().copied().find(|p| *p == slug);
            if let Some(s) = c.strip_prefix("cor--") {
                achar(s).map(Marca::Cor)
            } else {
                c.strip_prefix("fundo--").and_then(achar).map(Marca::Fundo)
            }
        }))
        .collect()
}

/// Um pedaço de texto visível, com o que ele é e de onde veio.
#[derive(Debug, Clone, PartialEq)]
pub struct Trecho {
    /// O que se VÊ — sem os marcadores.
    pub texto: String,
    /// As marcas ativas, de fora pra dentro. Um `**_a_**` tem as duas.
    pub marcas: Vec<Marca>,
    /// Onde este pedaço estava no texto do bloco.
    pub byte: Range<usize>,
}

impl Trecho {
    /// Tem esta marca?
    pub fn tem(&self, m: Marca) -> bool {
        self.marcas.contains(&m)
    }
}

/// Quebra o texto de um bloco em trechos.
///
/// Texto sem marcador nenhum sai como um trecho só — é o caso comum, e
/// vale a pena não pagar nada por ele.
pub fn trechos(texto: &str) -> Vec<Trecho> {
    if texto.is_empty() {
        return Vec::new();
    }

    let mut opcoes = Options::empty();
    opcoes.insert(Options::ENABLE_STRIKETHROUGH);
    let mut fora = Vec::new();
    let mut pilha: Vec<Marca> = Vec::new();
    // Quantas marcas cada `<span>` aberto empilhou.
    let mut spans: Vec<usize> = Vec::new();

    for (evento, faixa) in Parser::new_ext(texto, opcoes).into_offset_iter() {
        match evento {
            Event::Start(Tag::Strong) => pilha.push(Marca::Negrito),
            Event::Start(Tag::Emphasis) => pilha.push(Marca::Italico),
            Event::Start(Tag::Strikethrough) => pilha.push(Marca::Tachado),
            Event::Start(Tag::Link { .. }) => pilha.push(Marca::Link),
            Event::End(TagEnd::Strong | TagEnd::Emphasis | TagEnd::Strikethrough | TagEnd::Link) => {
                pilha.pop();
            }
            Event::InlineHtml(h) if h.trim_start().starts_with("<span") => {
                let cores = cores_do_span(&h);
                spans.push(cores.len());
                pilha.extend(cores);
            }
            Event::InlineHtml(h) if h.trim() == "</span>" => {
                let n = spans.pop().unwrap_or(0);
                pilha.truncate(pilha.len().saturating_sub(n));
            }
            Event::Code(t) => {
                let mut marcas = pilha.clone();
                marcas.push(Marca::Codigo);
                fora.push(Trecho { texto: t.to_string(), marcas, byte: faixa });
            }
            Event::Text(t) => fora.push(Trecho {
                texto: t.to_string(),
                marcas: pilha.clone(),
                byte: faixa,
            }),
            // Um bloco é uma linha: quebra macia vira espaço, como em
            // qualquer leitor.
            Event::SoftBreak | Event::HardBreak => fora.push(Trecho {
                texto: " ".into(),
                marcas: pilha.clone(),
                byte: faixa,
            }),
            _ => {}
        }
    }
    costurar_wikilinks(fora, texto)
}

/// Junta os trechos que caem dentro de um `[[wikilink]]`.
///
/// O pulldown-cmark não conhece wikilink e ainda por cima quebra `[[` em
/// DOIS eventos de um colchete cada — medido, não suposto. Então não dá
/// pra reconhecê-lo evento a evento: o jeito é varrer a fonte, achar os
/// pares, e recolher o que caiu dentro.
///
/// Dentro de código não vale, e isso é regra antiga do vault
/// (`links.rs`): `[[exemplo]]` num trecho de código é texto.
fn costurar_wikilinks(trechos: Vec<Trecho>, fonte: &str) -> Vec<Trecho> {
    let vaos = vaos_de_wikilink(fonte);
    if vaos.is_empty() {
        return trechos;
    }
    let mut fora: Vec<Trecho> = Vec::with_capacity(trechos.len());
    for t in trechos {
        let dentro = vaos
            .iter()
            .find(|v| v.start <= t.byte.start && t.byte.end <= v.end);
        match dentro {
            Some(v) if !t.tem(Marca::Codigo) => {
                // Já recolhi este vão? Então este trecho é continuação
                // dele e some — o texto do link vem do miolo, não da
                // soma dos pedaços.
                if fora.last().is_some_and(|u| u.byte == *v) {
                    continue;
                }
                let (alvo, alias) = crate::links::split_wikilink(&fonte[v.start + 2..v.end - 2]);
                let mut marcas = t.marcas.clone();
                marcas.push(Marca::Wikilink);
                fora.push(Trecho {
                    texto: alias.unwrap_or(alvo),
                    marcas,
                    byte: v.clone(),
                });
            }
            _ => fora.push(t),
        }
    }
    fora
}

/// Os pares `[[` … `]]` da fonte, sem aninhar.
fn vaos_de_wikilink(fonte: &str) -> Vec<Range<usize>> {
    let mut fora = Vec::new();
    let mut cursor = 0usize;
    while let Some(abre) = fonte[cursor..].find("[[") {
        let abre = cursor + abre;
        let Some(fecha) = fonte[abre + 2..].find("]]").map(|f| abre + 2 + f) else {
            break;
        };
        fora.push(abre..(fecha + 2));
        cursor = fecha + 2;
    }
    fora
}

/// O texto que aparece na tela — os trechos emendados.
pub fn visivel(trechos: &[Trecho]) -> String {
    trechos.iter().map(|t| t.texto.as_str()).collect()
}

/// Em que byte do markdown está a coluna `coluna` da tela.
///
/// Coluna conta CARACTERES, não bytes nem largura de célula: acento
/// ocupa uma coluna e dois bytes, e é o caso que o português traz todo
/// dia. Ideograma e emoji ocupam duas células e continuam contando um —
/// limite conhecido, e o dia em que alguém escrever japonês aqui o
/// cursor vai andar torto antes de errar o corte.
///
/// Dentro de `**abc**` o mapeamento é EXATO: o pulldown entrega o texto
/// sem os asteriscos e com o intervalo do miolo (medido: `2..5`), então
/// a coluna do `b` vira o byte do `b`. É o que um cursor precisa.
///
/// Onde não dá é quando o marcador vive DENTRO do intervalo — o
/// wikilink, cujo trecho mostra `o alvo` e cobre `[[Página|o alvo]]`.
/// Ali a resposta é o começo: não existe byte "no meio do link" que
/// signifique alguma coisa, e chutar um seria pior.
pub fn byte_da_coluna(trechos: &[Trecho], coluna: usize) -> usize {
    let mut andadas = 0usize;
    for t in trechos {
        let quantas = t.texto.chars().count();
        if coluna < andadas + quantas {
            let dentro = coluna - andadas;
            // Trecho sem marcador: o offset dentro dele é byte a byte.
            if t.texto.len() == t.byte.len() {
                let deslocamento = t
                    .texto
                    .char_indices()
                    .nth(dentro)
                    .map(|(i, _)| i)
                    .unwrap_or(t.texto.len());
                return t.byte.start + deslocamento;
            }
            return t.byte.start;
        }
        andadas += quantas;
    }
    trechos.last().map(|t| t.byte.end).unwrap_or(0)
}

/// Em que coluna da tela está o byte `byte` do markdown.
pub fn coluna_do_byte(trechos: &[Trecho], byte: usize) -> usize {
    let mut andadas = 0usize;
    for t in trechos {
        if byte < t.byte.end {
            if byte <= t.byte.start {
                return andadas;
            }
            if t.texto.len() == t.byte.len() {
                let dentro = byte - t.byte.start;
                return andadas + t.texto[..dentro.min(t.texto.len())].chars().count();
            }
            return andadas;
        }
        andadas += t.texto.chars().count();
    }
    andadas
}

#[cfg(test)]
mod testes {
    use super::*;

    fn marcado(texto: &str) -> Vec<(String, Vec<Marca>)> {
        trechos(texto)
            .into_iter()
            .map(|t| (t.texto, t.marcas))
            .collect()
    }

    #[test]
    fn texto_sem_marcador_sai_inteiro() {
        assert_eq!(marcado("só texto"), [("só texto".into(), vec![])]);
    }

    #[test]
    fn negrito_italico_e_codigo_viram_marca_e_perdem_o_marcador() {
        assert_eq!(
            marcado("um **forte** e `cod`"),
            [
                ("um ".to_string(), vec![]),
                ("forte".to_string(), vec![Marca::Negrito]),
                (" e ".to_string(), vec![]),
                ("cod".to_string(), vec![Marca::Codigo]),
            ]
        );
    }

    #[test]
    fn marca_dentro_de_marca_acumula() {
        let t = trechos("**a *b* c**");
        let meio = t.iter().find(|t| t.texto == "b").unwrap();
        assert!(meio.tem(Marca::Negrito) && meio.tem(Marca::Italico), "{meio:?}");
    }

    #[test]
    fn o_wikilink_mostra_o_alias_e_guarda_o_tamanho_do_original() {
        // O pulldown-cmark não conhece `[[...]]`: entrega como texto
        // comum, e quem separa é este módulo.
        let t = trechos("veja [[Página|o alvo]] agora");
        let link = t.iter().find(|t| t.tem(Marca::Wikilink)).unwrap();
        assert_eq!(link.texto, "o alvo");
        // O intervalo cobre o `[[...]]` INTEIRO — é o que ele ocupa no
        // arquivo, e é isso que a edição precisa saber.
        assert_eq!(&"veja [[Página|o alvo]] agora"[link.byte.clone()], "[[Página|o alvo]]");
    }

    #[test]
    fn wikilink_sem_alias_mostra_o_alvo() {
        let t = trechos("[[Sobre]]");
        assert_eq!(t[0].texto, "Sobre");
        assert!(t[0].tem(Marca::Wikilink));
    }

    #[test]
    fn o_visivel_e_o_texto_sem_os_marcadores() {
        assert_eq!(visivel(&trechos("um **forte** e `cod`")), "um forte e cod");
        assert_eq!(visivel(&trechos("veja [[A|b]] fim")), "veja b fim");
    }

    #[test]
    fn coluna_e_byte_conversam_nos_dois_sentidos() {
        // `um **forte** e`: a coluna 3 é o "f" de forte, que no arquivo
        // está depois de dois asteriscos.
        let md = "um **forte** e";
        let t = trechos(md);
        assert_eq!(visivel(&t), "um forte e");

        let b = byte_da_coluna(&t, 3);
        assert_eq!(&md[b..b + 5], "forte", "a coluna 3 não caiu no 'f'");
        assert_eq!(coluna_do_byte(&t, b), 3);

        // E o começo continua sendo o começo.
        assert_eq!(byte_da_coluna(&t, 0), 0);
        assert_eq!(coluna_do_byte(&t, 0), 0);
    }

    #[test]
    fn acento_ocupa_uma_coluna_e_dois_bytes() {
        // O caso que o português traz todo dia, e o que separa contar
        // caractere de contar byte.
        let md = "ação e paz";
        let t = trechos(md);
        // "ação" tem 4 letras e 6 bytes. A coluna 5 é o "e"; em bytes
        // ele está no 7, porque "ç" e "ã" ocupam dois cada.
        assert_eq!(byte_da_coluna(&t, 5), 7);
        assert_eq!(&md[7..8], "e");
        assert_eq!(coluna_do_byte(&t, 7), 5);
    }

    #[test]
    fn dentro_do_negrito_o_mapeamento_e_exato() {
        // O pulldown entrega "abc" com o intervalo do MIOLO (2..5), sem
        // os asteriscos — então a coluna do "b" vira o byte do "b", que
        // é o que um cursor precisa.
        let md = "**abc**";
        let t = trechos(md);
        assert_eq!(byte_da_coluna(&t, 1), 3);
        assert_eq!(&md[3..4], "b");
        assert_eq!(coluna_do_byte(&t, 3), 1);
    }

    #[test]
    fn dentro_do_wikilink_nao_ha_byte_que_signifique_algo() {
        // Aqui o marcador vive DENTRO do intervalo: o trecho mostra
        // "o alvo" e cobre `[[Página|o alvo]]`. Cair no meio devolve o
        // começo — chutar um byte ali seria pior que recusar.
        let md = "veja [[Página|o alvo]] fim";
        let t = trechos(md);
        let inicio = md.find("[[").unwrap();
        assert_eq!(byte_da_coluna(&t, 6), inicio);
        assert_eq!(byte_da_coluna(&t, 8), inicio);
    }

    #[test]
    fn coluna_depois_do_fim_cai_no_fim() {
        let md = "abc";
        let t = trechos(md);
        assert_eq!(byte_da_coluna(&t, 99), md.len());
    }

    #[test]
    fn texto_vazio_nao_tem_trecho() {
        assert!(trechos("").is_empty());
        assert_eq!(visivel(&[]), "");
        assert_eq!(byte_da_coluna(&[], 0), 0);
    }

    #[test]
    fn dentro_de_codigo_nada_e_interpretado() {
        // `` `**a**` `` é código com asteriscos, não negrito — e o
        // ciclo 276 já tinha aprendido que dentro de código espaço é
        // conteúdo. Aqui vale a mesma ideia.
        let t = trechos("`**a**`");
        assert_eq!(t.len(), 1);
        assert_eq!(t[0].texto, "**a**");
        assert!(t[0].tem(Marca::Codigo));
    }

    #[test]
    fn span_com_classe_da_paleta_vira_cor() {
        let t = trechos("um <span class=\"cor--vermelho fundo--ambar\">alerta</span> aqui");
        assert_eq!(visivel(&t), "um alerta aqui");
        let alerta = t.iter().find(|x| x.texto == "alerta").unwrap();
        assert_eq!(alerta.marcas, [Marca::Cor("vermelho"), Marca::Fundo("ambar")]);
        assert!(t.iter().find(|x| x.texto == " aqui").unwrap().marcas.is_empty());
    }

    #[test]
    fn span_com_cor_livre_vira_rgb() {
        let t = trechos("a <span style=\"color:#ff8800\">quente</span>");
        let q = t.iter().find(|x| x.texto == "quente").unwrap();
        assert_eq!(q.marcas, [Marca::CorLivre(0xff, 0x88, 0x00)]);
    }
}
