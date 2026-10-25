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

use anotadinho_core::diagrama::{self, so_ver, texto, texto::Papel};

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
    caixa_com(rotulo, aviso, miolo, nivel, largura, max_linhas, Realce::Marca, None, tema)
}

/// A [`caixa`] com a cor da moldura e um texto no fundo: a vista de dentro
/// do bloco (ciclo 498) acende a moldura e diz no fundo como se sai.
#[allow(clippy::too_many_arguments)]
fn caixa_com(
    rotulo: &str,
    aviso: Option<&str>,
    miolo: Vec<Vec<Span<'static>>>,
    nivel: usize,
    largura: usize,
    max_linhas: usize,
    cor: Realce,
    fundo: Option<&str>,
    tema: &Tema,
) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(nivel);
    let util = largura.saturating_sub(recuo.chars().count()).max(8);
    let moldura = tema.estilo(cor);
    let topo = cortar(&format!("┌ {rotulo} "), util);
    let topo_w: usize = topo.chars().map(|c| c.width().unwrap_or(0)).sum();
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
            format!("… mais {} linha(s) — Enter navega", total - mostra),
            tema.estilo(Realce::Dica),
        )]));
    }
    if total == 0 {
        fora.push(linha(vec![Span::styled("(vazio)", tema.estilo(Realce::Dica))]));
    }
    let fundo = match fundo {
        Some(t) => {
            let t = cortar(&format!("└─ {t} "), util);
            let w: usize = t.chars().map(|c| c.width().unwrap_or(0)).sum();
            format!("{t}{}", "─".repeat(util.saturating_sub(w)))
        }
        None => format!("└{}", "─".repeat(util.saturating_sub(1))),
    };
    fora.push(Line::from(vec![Span::raw(recuo), Span::styled(fundo, moldura)]));
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
    let c = conteudo_mermaid(fonte, cabe, tema);
    if c.desenho {
        // Largo demais (498): o desenho inteiro, recortado na borda, e o
        // aviso de que dá pra entrar e deslizar. Antes virava código, e o
        // diagrama que mais precisava ser visto era o que não aparecia.
        let aviso = (c.largura > cabe)
            .then(|| format!("desenho de {} colunas · Enter navega", c.largura));
        let miolo = c.linhas.iter().map(|l| recortar(l, 0, cabe)).collect();
        return caixa(&c.rotulo, aviso.as_deref(), miolo, nivel, largura, max_linhas, tema);
    }
    let miolo = miolo_de_codigo(fonte, Some("mermaid"), cabe, tema);
    caixa(&c.rotulo, c.aviso.as_deref(), miolo, nivel, largura, max_linhas, tema)
}

/// O que um bloco tem pra mostrar, sem corte nenhum (ciclo 498): a vista
/// de dentro do bloco recorta daqui.
struct Conteudo {
    rotulo: String,
    aviso: Option<String>,
    linhas: Vec<Vec<Span<'static>>>,
    /// A linha mais larga, em células.
    largura: usize,
    /// Saiu desenhado (e não como código).
    desenho: bool,
}

fn largura_dos(spans: &[Span]) -> usize {
    spans.iter().flat_map(|s| s.content.chars()).map(|c| c.width().unwrap_or(0)).sum()
}

fn conteudo(linhas: Vec<Vec<Span<'static>>>, rotulo: String, aviso: Option<String>, desenho: bool) -> Conteudo {
    let largura = linhas.iter().map(|l| largura_dos(l)).max().unwrap_or(0);
    Conteudo { rotulo, aviso, linhas, largura, desenho }
}

/// O diagrama na largura dele, ou o código com o motivo de não ter saído.
fn conteudo_mermaid(fonte: &str, cabe: usize, tema: &Tema) -> Conteudo {
    // Pra ver, comentário e cabeçalho `---` não fazem falta (499): o leitor
    // do construtor recusa os dois, pra não apagá-los ao reescrever.
    let (limpo, titulo) = so_ver::limpar(fonte);
    let pintar = |linhas: Vec<texto::Linha>| -> Vec<Vec<Span<'static>>> {
        let mut fora: Vec<Vec<Span<'static>>> = titulo
            .iter()
            .map(|t| vec![Span::styled(t.clone(), papel_do_desenho(Papel::Titulo, tema))])
            .collect();
        fora.extend(linhas.into_iter().map(|l| l.into_iter().map(|(p, t)| Span::styled(t, papel_do_desenho(p, tema))).collect()));
        fora
    };
    let motivo = match diagrama::ler(&limpo) {
        Ok(d) => match texto::desenhar_inteiro(&d, cabe) {
            Ok(linhas) => return conteudo(pintar(linhas), format!("mermaid · {}", d.tipo().nome().to_lowercase()), None, true),
            Err(nao) => nao.mensagem(),
        },
        // Os tipos que o construtor não monta, mas que a janela desenha
        // (499): pizza, mapa mental, linha do tempo, jornada, classes, ER.
        Err(fora) => match (so_ver::tipo(&limpo), so_ver::desenhar_inteiro(&limpo, cabe)) {
            (Some(nome), Some(Ok(linhas))) => return conteudo(pintar(linhas), format!("mermaid · {nome}"), None, true),
            (_, Some(Err(nao))) => nao.mensagem(),
            _ => {
                let palavra = limpo.split_whitespace().next().unwrap_or("");
                if ["graph", "flowchart", "sequenceDiagram", "stateDiagram", "stateDiagram-v2", "gantt"].contains(&palavra) {
                    format!("o construtor não monta este diagrama — {}", fora.mensagem())
                } else {
                    format!("o tipo `{palavra}` ainda não tem desenho no terminal")
                }
            }
        },
    };
    conteudo(codigo_inteiro(fonte, Some("mermaid"), tema), "mermaid".into(), Some(motivo), false)
}

/// A prévia do construtor (ciclo 500): o MESMO conteúdo que o bloco mostra
/// na página — desenho inteiro, ou o código com o motivo —, pra quem vai
/// recortar numa janela própria. Uma prévia com regra própria divergiria
/// da página, e o construtor mostraria um desenho que o bloco não mostra.
/// Devolve as linhas, a mais larga em células e o aviso.
pub fn previa_mermaid(fonte: &str, largura: usize, tema: &Tema) -> (Vec<Vec<Span<'static>>>, usize, Option<String>) {
    let c = conteudo_mermaid(fonte, largura, tema);
    (c.linhas, c.largura, c.aviso)
}

/// As linhas do código sem o corte por largura.
fn codigo_inteiro(codigo: &str, lingua: Option<&str>, tema: &Tema) -> Vec<Vec<Span<'static>>> {
    let linhas = linhas_de(codigo);
    let realcadas = lingua.and_then(|l| realce_codigo::realcar_guardado(&linhas, l));
    linhas
        .iter()
        .enumerate()
        .map(|(i, l)| match realcadas.as_ref().and_then(|r| r.get(i)) {
            Some(trechos) => trechos.iter().map(|(c, t)| Span::styled(t.clone(), tema.estilo(papel(*c)))).collect(),
            None => vec![Span::styled(l.clone(), tema.estilo(Realce::Codigo))],
        })
        .collect()
}

/// As células `x..x+largura` de uma linha, com as cores. Um caractere
/// largo partido pela borda vira espaço: meio caractere CJK o terminal
/// desenha inteiro, e a linha passaria da largura.
pub fn recortar(spans: &[Span<'static>], x: usize, largura: usize) -> Vec<Span<'static>> {
    let mut fora = Vec::new();
    let mut col = 0;
    let fim = x + largura;
    for s in spans {
        let mut pedaco = String::new();
        for ch in s.content.chars() {
            let w = ch.width().unwrap_or(0);
            if col >= fim {
                break;
            }
            if col >= x && col + w <= fim {
                pedaco.push(ch);
            } else if col + w > x && w > 0 {
                // Partido na borda (de um lado ou do outro).
                for _ in col.max(x)..(col + w).min(fim) {
                    pedaco.push(' ');
                }
            }
            col += w;
        }
        if !pedaco.is_empty() {
            fora.push(Span::styled(pedaco, s.style));
        }
        if col >= fim {
            break;
        }
    }
    fora
}

/// Onde a vista de dentro de um bloco está (ciclo 498): a primeira linha e
/// a primeira coluna à vista.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Vista {
    pub x: usize,
    pub y: usize,
    /// O termo da busca dentro do bloco (ciclo 509), vazio sem busca.
    pub busca: String,
    /// Digitando o termo (depois do `/`, antes do Enter).
    pub digitando: bool,
    /// O achado em que se está: linha e coluna (em células).
    pub atual: Option<(usize, usize)>,
}

/// Onde `termo` aparece nas linhas, sem diferença de maiúscula: linha,
/// coluna e largura, em células.
fn achados_nas(linhas: &[Vec<Span<'static>>], termo: &str) -> Vec<(usize, usize, usize)> {
    let termo: Vec<char> = termo.to_lowercase().chars().collect();
    if termo.is_empty() {
        return Vec::new();
    }
    let mut fora = Vec::new();
    for (n, l) in linhas.iter().enumerate() {
        let cs: Vec<char> = l.iter().flat_map(|s| s.content.chars()).collect();
        let baixo: Vec<char> = cs.iter().map(|c| c.to_lowercase().next().unwrap_or(*c)).collect();
        let mut i = 0;
        while i + termo.len() <= baixo.len() {
            if baixo[i..i + termo.len()] == termo[..] {
                let col: usize = cs[..i].iter().map(|c| c.width().unwrap_or(0)).sum();
                let w: usize = cs[i..i + termo.len()].iter().map(|c| c.width().unwrap_or(0)).sum();
                fora.push((n, col, w));
                i += termo.len();
            } else {
                i += 1;
            }
        }
    }
    fora
}

/// A linha com as células de `faixas` (coluna, largura) no estilo `marca`.
fn marcar(spans: &[Span<'static>], faixas: &[(usize, usize, Style)]) -> Vec<Span<'static>> {
    if faixas.is_empty() {
        return spans.to_vec();
    }
    let mut fora: Vec<Span<'static>> = Vec::new();
    let mut col = 0;
    for s in spans {
        for ch in s.content.chars() {
            let w = ch.width().unwrap_or(0);
            let estilo = faixas.iter().find(|(c, l, _)| col >= *c && col < c + l).map_or(s.style, |(_, _, e)| s.style.patch(*e));
            match fora.last_mut() {
                Some(u) if u.style == estilo => u.content.to_mut().push(ch),
                _ => fora.push(Span::styled(ch.to_string(), estilo)),
            }
            col += w;
        }
    }
    fora
}

/// O achado seguinte (ou o anterior) a `desde`, dando a volta; sem
/// `desde`, o primeiro. `None` se o termo não aparece.
pub fn achar(fonte: &str, lingua: Option<&str>, cabe: usize, tema: &Tema, termo: &str, desde: Option<(usize, usize)>, frente: bool) -> Option<(usize, usize)> {
    let c = conteudo_do_bloco(fonte, lingua, cabe, tema);
    let todos: Vec<(usize, usize)> = achados_nas(&c.linhas, termo).into_iter().map(|(l, x, _)| (l, x)).collect();
    if todos.is_empty() {
        return None;
    }
    let Some(d) = desde else { return todos.first().copied() };
    if frente {
        todos.iter().find(|p| **p > d).or(todos.first()).copied()
    } else {
        todos.iter().rev().find(|p| **p < d).or(todos.last()).copied()
    }
}

/// O tamanho do que o bloco tem e do que cabe dele, medido no último
/// quadro: é com isto que a tecla sabe onde a vista para.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Medida {
    pub linhas: usize,
    pub colunas: usize,
    pub alto: usize,
    pub largo: usize,
}

impl Medida {
    pub fn max_y(&self) -> usize {
        self.linhas.saturating_sub(self.alto)
    }
    pub fn max_x(&self) -> usize {
        self.colunas.saturating_sub(self.largo)
    }
}

/// As medidas do bloco na vista, sem desenhar.
pub fn medir(fonte: &str, lingua: Option<&str>, nivel: usize, largura: usize, alto: usize, tema: &Tema) -> Medida {
    let cabe = largura_do_miolo(nivel, largura);
    let c = conteudo_do_bloco(fonte, lingua, cabe, tema);
    // O aviso, quando há, toma uma linha do miolo.
    let alto = alto.saturating_sub(usize::from(c.aviso.is_some())).max(1);
    Medida { linhas: c.linhas.len(), colunas: c.largura, alto, largo: cabe }
}

fn conteudo_do_bloco(fonte: &str, lingua: Option<&str>, cabe: usize, tema: &Tema) -> Conteudo {
    if lingua.is_some_and(|l| l.eq_ignore_ascii_case("mermaid")) {
        conteudo_mermaid(fonte, cabe, tema)
    } else {
        conteudo(codigo_inteiro(fonte, lingua, tema), rotulo(lingua), None, false)
    }
}

/// O bloco visto de dentro (ciclo 498): `alto` linhas do miolo a partir de
/// `vista.y`, recortadas a partir da coluna `vista.x`, numa moldura acesa
/// que diz onde se está. A vista já vem dentro dos limites ([`Medida`]).
pub fn desenhar_na_vista(
    fonte: &str,
    lingua: Option<&str>,
    nivel: usize,
    largura: usize,
    alto: usize,
    vista: &Vista,
    tema: &Tema,
) -> Vec<Line<'static>> {
    let cabe = largura_do_miolo(nivel, largura);
    let c = conteudo_do_bloco(fonte, lingua, cabe, tema);
    let alto = alto.saturating_sub(usize::from(c.aviso.is_some())).max(1);
    let total = c.linhas.len();
    let y = vista.y.min(total.saturating_sub(alto));
    let x = vista.x.min(c.largura.saturating_sub(cabe));
    // Os achados da busca (509) acendem; o atual, mais forte.
    let achados = achados_nas(&c.linhas, &vista.busca);
    let fraco = tema.estilo(Realce::Barra);
    let forte = tema.estilo(Realce::Cursor);
    let miolo: Vec<Vec<Span<'static>>> = c
        .linhas
        .iter()
        .enumerate()
        .skip(y)
        .take(alto)
        .map(|(n, l)| {
            let faixas: Vec<(usize, usize, Style)> = achados
                .iter()
                .filter(|(ln, _, _)| *ln == n)
                .map(|(_, col, w)| (*col, *w, if vista.atual == Some((n, *col)) { forte } else { fraco }))
                .collect();
            recortar(&marcar(l, &faixas), x, cabe)
        })
        .collect();
    // Onde se está vai no topo, curto (`↕` linhas, `↔` colunas): num painel
    // estreito o rótulo é cortado, e o que não pode sumir é a posição.
    let mut rotulo = c.rotulo.clone();
    if total > alto {
        rotulo.push_str(&format!(" · ↕ {}–{}/{total}", y + 1, (y + alto).min(total)));
    }
    if c.largura > cabe {
        rotulo.push_str(&format!(" · ↔ {}–{}/{}", x + 1, (x + cabe).min(c.largura), c.largura));
    }
    let n = miolo.len();
    let fundo = if vista.digitando {
        format!("/{}▏ Enter procura · Esc desiste", vista.busca)
    } else if !vista.busca.is_empty() {
        match vista.atual.and_then(|a| achados.iter().position(|(l, x, _)| (*l, *x) == a)) {
            Some(k) => format!("/{} · {}/{} · n N próximo · Esc limpa", vista.busca, k + 1, achados.len()),
            None if achados.is_empty() => format!("/{} · nada · Esc limpa", vista.busca),
            None => format!("/{} · {} achado(s) · n próximo · Esc limpa", vista.busca, achados.len()),
        }
    } else {
        "navegando: hjkl rola · / busca · i edita · Esc sai".to_string()
    };
    caixa_com(&rotulo, c.aviso.as_deref(), miolo, nivel, largura, n.max(2), Realce::BordaFoco, Some(&fundo), tema)
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
        assert_eq!(t[10], "│ … mais 41 linha(s) — Enter navega");
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
    fn mermaid_fora_do_modelo_mostra_o_codigo_e_o_largo_sai_recortado() {
        let tema = Tema::novo("mocha");
        let t = texto(&desenhar_mermaid("gitGraph\n  commit\n", 0, 60, 30, &tema));
        assert_eq!(t[1], "│ ⚠ o tipo `gitGraph` ainda não tem desenho no terminal");
        assert_eq!(t[2], "│ gitGraph");
        let t = texto(&desenhar_mermaid("graph TD\n  A --> B\n  click A callback\n", 0, 60, 30, &tema));
        assert!(t[1].starts_with("│ ⚠ o construtor não monta este diagrama — linha 3"), "{t:?}");
        let largo = "sequenceDiagram\n  A->>B: uma mensagem que não cabe em vinte colunas\n";
        // Largo demais, desde o 498, sai desenhado e recortado na borda.
        let t = texto(&desenhar_mermaid(largo, 0, 40, 30, &tema));
        assert!(t[1].starts_with("│ ⚠ desenho de 51 colunas · Enter navega"), "{t:?}");
        assert!(t.iter().any(|l| l.contains("│ A │")), "{t:?}");
        assert!(t.iter().all(|l| l.chars().count() <= 40), "passou da largura:\n{}", t.join("\n"));
    }

    #[test]
    fn vazio_e_dobrado() {
        let t = texto(&desenhar("", Some("sh"), 0, 20, 30, &Tema::novo("mocha")));
        assert_eq!(t[1], "│ (vazio)");
        let d: String = dobrado("a\nb\nc\n", Some("py"), 0, &Tema::novo("mocha")).spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(d, "▸ ``` py · 3 linha(s)");
    }
}
