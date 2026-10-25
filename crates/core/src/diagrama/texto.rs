//! O diagrama desenhado em caracteres, pra quem não tem o mermaid: a TUI
//! (ciclos 496–497, spec `tui-codigo-e-diagramas`, RF6–RF8).
//!
//! O desenho sai do MESMO modelo do construtor (RNF3): o que o construtor
//! entende é o que se desenha, e nada é lido duas vezes. Quem chama recebe
//! linhas de trechos com um [`Papel`] cada, e pinta como quiser.
//!
//! Largura é em CÉLULAS de terminal: um caractere CJK ocupa duas. Um
//! desenho que não cabe na largura pedida não é cortado — cortado no meio
//! de uma caixa ele confunde mais que o código (RF8) — e sim recusado com
//! [`NaoDesenha::NaoCabe`].

use unicode_width::UnicodeWidthChar;

use super::camadas;
use super::estados::{Estados, Ponta};
use super::fluxograma::{Direcao, Fluxograma, Forma, TipoLigacao};
use super::gantt::{Fim, Gantt, Inicio, Marca};
use super::sequencia::{Item, Lado, Seta, Sequencia};
use super::Diagrama;
use crate::date_util::{add_days, days_between, parse_date};

/// O que cada pedaço do desenho é: quem pinta escolhe a cor por aqui.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Papel {
    /// Espaço.
    Fundo,
    /// A moldura de uma caixa (participante, nó, estado).
    Caixa,
    /// O texto dentro de uma caixa.
    Nome,
    /// A linha da vida de um participante.
    Vida,
    /// Uma seta ou ligação.
    Seta,
    /// O rótulo de uma seta.
    Rotulo,
    /// Uma nota (moldura e texto).
    Nota,
    /// O nome de uma seção do gantt.
    Secao,
    /// O título do diagrama.
    Titulo,
    /// O eixo de datas.
    Eixo,
    /// Barra de tarefa comum.
    Barra,
    /// Barra de tarefa `done`.
    BarraFeita,
    /// Barra de tarefa `active`.
    BarraAtiva,
    /// Barra de tarefa `crit`.
    BarraCritica,
    /// Marco (`milestone`).
    Marco,
}

/// Uma linha do desenho: trechos de texto com o papel de cada um.
pub type Linha = Vec<(Papel, String)>;

/// Por que um diagrama não sai desenhado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NaoDesenha {
    /// Precisa de mais colunas que as dadas.
    NaoCabe { precisa: usize },
    /// O tipo ainda não tem desenho em caracteres.
    SemDesenho,
    /// O gantt não tem como pôr as tarefas no tempo (sem nenhuma data, ou
    /// dependência de uma tarefa que não existe).
    SemData(String),
}

impl NaoDesenha {
    /// A frase pra quem lê, curta.
    pub fn mensagem(&self) -> String {
        match self {
            NaoDesenha::NaoCabe { precisa } => format!("o desenho não cabe na largura (precisa de {precisa} colunas)"),
            NaoDesenha::SemDesenho => "este tipo ainda não tem desenho no terminal".to_string(),
            NaoDesenha::SemData(m) => format!("o cronograma não tem como ser posto no tempo: {m}"),
        }
    }
}

/// O diagrama em linhas de no máximo `largura` células.
pub fn desenhar(d: &Diagrama, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    match d {
        Diagrama::Sequencia(s) => sequencia(s, largura),
        Diagrama::Gantt(g) => gantt(g, largura),
        Diagrama::Fluxograma(f) => fluxograma(f, largura),
        Diagrama::Estados(e) => estados(e, largura),
    }
}

/// O diagrama na largura dele quando não cabe em `largura` (ciclo 498):
/// quem recorta é quem mostra, numa vista que desliza. Recusar continua
/// valendo pra quem só tem a largura do painel; quem pode rolar quer o
/// desenho inteiro. Cada desenho mede `precisa` na própria arrumação, então
/// uma segunda passada com ela basta — a terceira é só cinto.
pub fn desenhar_inteiro(d: &Diagrama, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let mut largura = largura;
    for _ in 0..3 {
        match desenhar(d, largura) {
            Err(NaoDesenha::NaoCabe { precisa }) if precisa > largura => largura = precisa,
            r => return r,
        }
    }
    desenhar(d, largura)
}

/// Largura de `s` em células.
pub fn largura_de(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// `s` cortado em `max` células, com `…` quando cortou.
pub(super) fn cortar(s: &str, max: usize) -> String {
    if largura_de(s) <= max {
        return s.to_string();
    }
    let mut fora = String::new();
    let mut usado = 0;
    for c in s.chars() {
        let w = c.width().unwrap_or(0);
        if usado + w + 1 > max {
            break;
        }
        fora.push(c);
        usado += w;
    }
    fora.push('…');
    fora
}

/// Uma grade de células. Um caractere largo ocupa a célula dele e marca a
/// seguinte como continuação, que não sai no texto.
struct Grade {
    celulas: Vec<Vec<Celula>>,
}

#[derive(Clone, Copy, PartialEq)]
enum Celula {
    Vazia,
    Letra(char, Papel),
    Continuacao,
}

impl Grade {
    fn nova() -> Self {
        Grade { celulas: Vec::new() }
    }

    fn em(&mut self, x: usize, y: usize) -> &mut Celula {
        while self.celulas.len() <= y {
            self.celulas.push(Vec::new());
        }
        let linha = &mut self.celulas[y];
        while linha.len() <= x {
            linha.push(Celula::Vazia);
        }
        &mut linha[x]
    }

    fn letra(&mut self, x: usize, y: usize, c: char, p: Papel) {
        *self.em(x, y) = Celula::Letra(c, p);
        if c.width().unwrap_or(1) == 2 {
            *self.em(x + 1, y) = Celula::Continuacao;
        }
    }

    /// Escreve `s` a partir de `x`; devolve a largura escrita.
    fn texto(&mut self, x: usize, y: usize, s: &str, p: Papel) -> usize {
        let mut cx = x;
        for c in s.chars() {
            self.letra(cx, y, c, p);
            cx += c.width().unwrap_or(0).max(1);
        }
        cx - x
    }

    fn repetir(&mut self, x0: usize, x1: usize, y: usize, c: char, p: Papel) {
        for x in x0..x1 {
            self.letra(x, y, c, p);
        }
    }

    fn caixa(&mut self, x: usize, y: usize, largura: usize, texto: &str, cantos: [char; 4], p_moldura: Papel, p_texto: Papel) {
        let [ce, cd, be, bd] = cantos;
        self.letra(x, y, ce, p_moldura);
        self.repetir(x + 1, x + largura - 1, y, '─', p_moldura);
        self.letra(x + largura - 1, y, cd, p_moldura);
        self.letra(x, y + 1, '│', p_moldura);
        self.repetir(x + 1, x + largura - 1, y + 1, ' ', p_texto);
        let w = largura_de(texto);
        self.texto(x + (largura - w) / 2, y + 1, texto, p_texto);
        self.letra(x + largura - 1, y + 1, '│', p_moldura);
        self.letra(x, y + 2, be, p_moldura);
        self.repetir(x + 1, x + largura - 1, y + 2, '─', p_moldura);
        self.letra(x + largura - 1, y + 2, bd, p_moldura);
    }

    fn linhas(self) -> Vec<Linha> {
        self.celulas
            .into_iter()
            .map(|linha| {
                let mut fora: Linha = Vec::new();
                for c in linha {
                    let (ch, p) = match c {
                        Celula::Vazia => (' ', Papel::Fundo),
                        Celula::Letra(ch, p) => (ch, p),
                        Celula::Continuacao => continue,
                    };
                    match fora.last_mut() {
                        Some((ultimo, s)) if *ultimo == p => s.push(ch),
                        _ => fora.push((p, ch.to_string())),
                    }
                }
                // Espaço no fim não diz nada e só atrapalha quem compara.
                while let Some((p, s)) = fora.last_mut() {
                    let t = s.trim_end().len();
                    if t == 0 {
                        fora.pop();
                    } else {
                        if *p == Papel::Fundo || s.len() != t {
                            s.truncate(t);
                        }
                        break;
                    }
                }
                fora
            })
            .collect()
    }
}

const CANTOS_RETOS: [char; 4] = ['┌', '┐', '└', '┘'];
const CANTOS_REDONDOS: [char; 4] = ['╭', '╮', '╰', '╯'];

// ── Sequência ────────────────────────────────────────────────────────

/// Quanto cada participante ocupa à esquerda e à direita do centro.
fn metades(w: usize) -> (usize, usize) {
    let esq = (w - 1) / 2;
    (esq, w - 1 - esq)
}

fn tracejada(seta: Seta) -> bool {
    matches!(seta, Seta::Resposta | Seta::AssincronaPontilhada | Seta::LinhaPontilhada | Seta::CruzPontilhada)
}

/// A ponta da seta, olhando pra direita ou pra esquerda. `None` = sem ponta.
fn ponta(seta: Seta, pra_direita: bool) -> Option<char> {
    match seta {
        Seta::Sincrona | Seta::Resposta => Some(if pra_direita { '▶' } else { '◀' }),
        Seta::Assincrona | Seta::AssincronaPontilhada => Some(if pra_direita { '▷' } else { '◁' }),
        Seta::Cruz | Seta::CruzPontilhada => Some('×'),
        Seta::Linha | Seta::LinhaPontilhada => None,
    }
}

fn sequencia(s: &Sequencia, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let n = s.participantes.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    let idx = |id: &str| s.participantes.iter().position(|p| p.id == id);
    let caixa: Vec<usize> = s.participantes.iter().map(|p| (largura_de(&p.nome) + 4).max(5)).collect();
    let meia: Vec<(usize, usize)> = caixa.iter().map(|w| metades(*w)).collect();

    // Distância entre centros vizinhos, e as folgas nas pontas. Começa no
    // mínimo pras caixas não se tocarem e cresce pelo que precisar caber.
    let mut vao: Vec<usize> = (0..n.saturating_sub(1)).map(|i| meia[i].1 + meia[i + 1].0 + 3).collect();
    let mut folga_esq = meia[0].0;
    let mut folga_dir = meia[n - 1].1;
    let mut a_direita = |vao: &mut Vec<usize>, folga_dir: &mut usize, i: usize, precisa: usize| {
        if i + 1 < n {
            vao[i] = vao[i].max(precisa + meia[i + 1].0 + 1);
        } else {
            *folga_dir = (*folga_dir).max(precisa);
        }
    };
    for item in &s.itens {
        match item {
            Item::Mensagem { de, para, texto, .. } => {
                let (Some(a), Some(b)) = (idx(de), idx(para)) else { continue };
                let rotulo = largura_de(texto);
                if a == b {
                    // Mensagem pra si: um laço à direita, rótulo ao lado.
                    a_direita(&mut vao, &mut folga_dir, a, (rotulo + 2).max(5));
                } else {
                    let (a, b) = (a.min(b), a.max(b));
                    let tem: usize = vao[a..b].iter().sum();
                    let quer = rotulo + 4;
                    if tem < quer {
                        vao[b - 1] += quer - tem;
                    }
                }
            }
            Item::Nota { lado, alvos, texto } => {
                let w = largura_de(texto) + 4;
                let Some(i) = alvos.first().and_then(|a| idx(a)) else { continue };
                match lado {
                    Lado::Direita => a_direita(&mut vao, &mut folga_dir, i, w + 2),
                    Lado::Esquerda => {
                        if i > 0 {
                            vao[i - 1] = vao[i - 1].max(w + 2 + meia[i - 1].1 + 1);
                        } else {
                            folga_esq = folga_esq.max(w + 2);
                        }
                    }
                    Lado::Sobre => {
                        let j = alvos.get(1).and_then(|a| idx(a)).unwrap_or(i);
                        let (a, b) = (i.min(j), i.max(j));
                        let tem: usize = vao[a..b].iter().sum::<usize>() + 4;
                        if w > tem && b > a {
                            vao[b - 1] += w - tem;
                        }
                        let meio = w.max(tem) / 2 + 1;
                        if a == b {
                            if a > 0 {
                                vao[a - 1] = vao[a - 1].max(meio + meia[a - 1].1 + 1);
                            } else {
                                folga_esq = folga_esq.max(meio);
                            }
                            a_direita(&mut vao, &mut folga_dir, a, meio);
                        } else {
                            folga_esq = if a == 0 { folga_esq.max(2) } else { folga_esq };
                            folga_dir = if b == n - 1 { folga_dir.max(2) } else { folga_dir };
                        }
                    }
                }
            }
        }
    }

    let mut x = vec![folga_esq];
    for v in &vao {
        x.push(x.last().unwrap() + v);
    }
    let total = x[n - 1] + folga_dir + 1;
    if total > largura {
        return Err(NaoDesenha::NaoCabe { precisa: total });
    }

    let mut g = Grade::new_sequencia(s, &x, &caixa, &meia);
    let mut y = 3;
    for item in &s.itens {
        match item {
            Item::Mensagem { de, para, seta, texto } => {
                let (Some(a), Some(b)) = (idx(de), idx(para)) else { continue };
                let traco = if tracejada(*seta) { '╌' } else { '─' };
                if a == b {
                    let xa = x[a];
                    g.texto(xa + 2, y, texto, Papel::Rotulo);
                    g.repetir(xa + 1, xa + 4, y + 1, traco, Papel::Seta);
                    g.letra(xa + 4, y + 1, '╮', Papel::Seta);
                    g.letra(xa + 1, y + 2, ponta(*seta, false).unwrap_or(traco), Papel::Seta);
                    g.repetir(xa + 2, xa + 4, y + 2, traco, Papel::Seta);
                    g.letra(xa + 4, y + 2, '╯', Papel::Seta);
                    y += 3;
                } else {
                    let (xa, xb) = (x[a], x[b]);
                    let (esq, dir) = (xa.min(xb), xa.max(xb));
                    let w = largura_de(texto);
                    g.texto(esq + 1 + (dir - esq - 1).saturating_sub(w) / 2, y, texto, Papel::Rotulo);
                    g.repetir(esq + 1, dir, y + 1, traco, Papel::Seta);
                    if let Some(p) = ponta(*seta, xb > xa) {
                        g.letra(if xb > xa { xb - 1 } else { xb + 1 }, y + 1, p, Papel::Seta);
                    }
                    y += 2;
                }
            }
            Item::Nota { lado, alvos, texto } => {
                let Some(i) = alvos.first().and_then(|a| idx(a)) else { continue };
                let w = largura_de(texto) + 4;
                let x0 = match lado {
                    Lado::Direita => x[i] + 2,
                    Lado::Esquerda => x[i] - 1 - w,
                    Lado::Sobre => {
                        let j = alvos.get(1).and_then(|a| idx(a)).unwrap_or(i);
                        let (a, b) = (x[i.min(j)], x[i.max(j)]);
                        let larg = w.max(b - a + 4);
                        let x0 = (a + b) / 2 - larg / 2;
                        g.caixa(x0, y, larg, texto, CANTOS_RETOS, Papel::Nota, Papel::Nota);
                        y += 3;
                        continue;
                    }
                };
                g.caixa(x0, y, w, texto, CANTOS_RETOS, Papel::Nota, Papel::Nota);
                y += 3;
            }
        }
    }
    // As linhas da vida, por baixo de tudo que não as cobriu, até uma
    // linha depois do último item.
    for yy in 3..=y {
        for &xi in &x {
            if *g.em(xi, yy) == Celula::Vazia {
                g.letra(xi, yy, '│', Papel::Vida);
            }
        }
    }
    // As caixas de novo embaixo, como o mermaid faz por padrão
    // (`mirrorActors`). Sem elas, as linhas da vida terminam no vazio e o
    // desenho parece cortado — foi o que a pessoa viu ao comparar com a
    // janela.
    for (i, p) in s.participantes.iter().enumerate() {
        let cantos = if p.ator { CANTOS_REDONDOS } else { CANTOS_RETOS };
        g.caixa(x[i] - meia[i].0, y + 1, caixa[i], &p.nome, cantos, Papel::Caixa, Papel::Nome);
        g.letra(x[i], y + 1, '┴', Papel::Caixa);
    }
    Ok(g.linhas())
}

impl Grade {
    /// A grade com as caixas dos participantes no topo.
    fn new_sequencia(s: &Sequencia, x: &[usize], caixa: &[usize], meia: &[(usize, usize)]) -> Grade {
        let mut g = Grade::nova();
        for (i, p) in s.participantes.iter().enumerate() {
            let x0 = x[i] - meia[i].0;
            let cantos = if p.ator { CANTOS_REDONDOS } else { CANTOS_RETOS };
            g.caixa(x0, 0, caixa[i], &p.nome, cantos, Papel::Caixa, Papel::Nome);
            g.letra(x[i], 2, '┬', Papel::Caixa);
        }
        g
    }
}

// ── Gantt ────────────────────────────────────────────────────────────

/// Dias de uma duração (`3d`, `2w`, `12h`).
fn dias_da_duracao(s: &str) -> Option<f64> {
    let unidade = s.chars().last()?;
    let n: f64 = s[..s.len() - unidade.len_utf8()].parse().ok()?;
    match unidade {
        'd' => Some(n),
        'w' => Some(n * 7.0),
        'h' => Some(n / 24.0),
        _ => None,
    }
}

struct Posta<'a> {
    nome: &'a str,
    secao: usize,
    ini: f64,
    fim: f64,
    marcas: &'a [Marca],
}

/// As tarefas no tempo, em dias desde `base`.
fn por_no_tempo(g: &Gantt) -> Result<(String, Vec<Posta<'_>>), NaoDesenha> {
    let base = g
        .secoes
        .iter()
        .flat_map(|s| &s.tarefas)
        .find_map(|t| match (&t.inicio, &t.fim) {
            (Inicio::Data(d), _) => Some(d.clone()),
            (_, Fim::Data(d)) => Some(d.clone()),
            _ => None,
        })
        .filter(|d| parse_date(d).is_some())
        .ok_or_else(|| NaoDesenha::SemData("nenhuma tarefa tem data".into()))?;
    let dia = |d: &str| -> Result<f64, NaoDesenha> {
        days_between(&base, d).map(|n| n as f64).ok_or_else(|| NaoDesenha::SemData(format!("data inválida: {d}")))
    };
    let mut fim_de: std::collections::HashMap<&str, f64> = Default::default();
    let mut anterior: Option<f64> = None;
    let mut fora = Vec::new();
    for (k, sec) in g.secoes.iter().enumerate() {
        for t in &sec.tarefas {
            let ini = match &t.inicio {
                Inicio::Data(d) => dia(d)?,
                Inicio::Anterior => anterior.ok_or_else(|| NaoDesenha::SemData(format!("\"{}\" começa depois de nada", t.nome)))?,
                Inicio::Depois(ids) => {
                    let mut m = f64::MIN;
                    for id in ids {
                        let f = fim_de
                            .get(id.as_str())
                            .ok_or_else(|| NaoDesenha::SemData(format!("\"{}\" depende de \"{id}\", que não veio antes", t.nome)))?;
                        m = m.max(*f);
                    }
                    m
                }
            };
            let fim = if t.marcas.contains(&Marca::Marco) {
                ini
            } else {
                match &t.fim {
                    // A data de fim do mermaid é o dia em que a tarefa acaba,
                    // exclusivo: `2024-01-01, 2024-01-03` são dois dias.
                    Fim::Data(d) => dia(d)?.max(ini),
                    Fim::Duracao(s) => ini + dias_da_duracao(s).ok_or_else(|| NaoDesenha::SemData(format!("duração inválida: {s}")))?,
                }
            };
            if let Some(id) = &t.id {
                fim_de.insert(id.as_str(), fim);
            }
            anterior = Some(fim);
            fora.push(Posta { nome: &t.nome, secao: k, ini, fim, marcas: &t.marcas });
        }
    }
    Ok((base, fora))
}

fn dd_mm(base: &str, dia: f64) -> String {
    add_days(base, dia.floor() as i64)
        .and_then(|d| parse_date(&d))
        .map(|(_, m, d)| format!("{d:02}/{m:02}"))
        .unwrap_or_default()
}

fn gantt(g: &Gantt, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let (base, postas) = por_no_tempo(g)?;
    if postas.is_empty() {
        return Ok(Vec::new());
    }
    let ini = postas.iter().map(|p| p.ini).fold(f64::MAX, f64::min);
    let fim = postas.iter().map(|p| p.fim).fold(f64::MIN, f64::max);
    let duracao = (fim - ini).max(1.0);

    let nome_w = postas.iter().map(|p| largura_de(p.nome)).max().unwrap_or(0).clamp(4, 24);
    let col0 = 2 + nome_w + 1;
    const BARRA_MIN: usize = 12;
    if largura < col0 + BARRA_MIN {
        return Err(NaoDesenha::NaoCabe { precisa: col0 + BARRA_MIN });
    }
    // Até três colunas por dia: mais que isso, um cronograma curto vira
    // tijolos enormes.
    let por_dia = ((largura - col0) as f64 / duracao).min(3.0);
    let col = |d: f64| col0 + ((d - ini) * por_dia).round() as usize;
    let fim_x = col(fim);

    let mut grade = Grade::nova();
    let mut y = 0;
    if let Some(t) = &g.titulo {
        grade.texto(0, y, &cortar(t, largura), Papel::Titulo);
        y += 1;
    }
    // O eixo: datas de tempos em tempos, sem se sobrepor, e a régua embaixo.
    let marcas_n = ((fim_x - col0) / 12).max(1);
    let mut livre = col0;
    grade.repetir(col0, fim_x + 1, y + 1, '─', Papel::Eixo);
    for j in 0..=marcas_n {
        let d = ini + duracao * j as f64 / marcas_n as f64;
        let x = col(d).min(fim_x);
        let rotulo = dd_mm(&base, d);
        let w = largura_de(&rotulo);
        // O último rótulo encosta na direita em vez de passar dela.
        let x0 = if x + w > largura { largura - w } else { x };
        if x0 < livre {
            continue;
        }
        grade.texto(x0, y, &rotulo, Papel::Eixo);
        grade.letra(x, y + 1, '┬', Papel::Eixo);
        livre = x0 + w + 1;
    }
    y += 2;

    let mut secao_atual = usize::MAX;
    for p in &postas {
        if p.secao != secao_atual {
            secao_atual = p.secao;
            if let Some(n) = &g.secoes[p.secao].nome {
                grade.texto(0, y, &cortar(n, largura), Papel::Secao);
                y += 1;
            }
        }
        grade.texto(2, y, &cortar(p.nome, nome_w), Papel::Nome);
        let papel = if p.marcas.contains(&Marca::Critica) {
            Papel::BarraCritica
        } else if p.marcas.contains(&Marca::Ativa) {
            Papel::BarraAtiva
        } else if p.marcas.contains(&Marca::Feita) {
            Papel::BarraFeita
        } else {
            Papel::Barra
        };
        if p.marcas.contains(&Marca::Marco) {
            grade.letra(col(p.ini).min(largura - 1), y, '◆', Papel::Marco);
        } else {
            let (a, b) = (col(p.ini), col(p.fim).max(col(p.ini) + 1).min(largura));
            // Feita tem textura própria: sem cor, ainda se distingue.
            let c = if papel == Papel::BarraFeita { '▓' } else { '█' };
            grade.repetir(a, b, y, c, papel);
        }
        y += 1;
    }
    Ok(grade.linhas())
}

// ── Grafo: fluxograma e estados (ciclo 497) ──────────────────────────
//
// As camadas vêm de `camadas` (a mesma arrumação do canvas). Entre duas
// camadas há um CANAL: uma faixa de linhas (ou colunas, na horizontal) por
// onde as ligações passam. Cada ligação que precisa andar de lado ganha a
// sua PISTA no canal, e nenhuma divide trilho com outra. Ligação que volta,
// que pula camada ou que fica na mesma camada sai pelo canal de baixo da
// origem, corre por uma pista LATERAL por fora de todas as caixas e entra
// pelo canal de cima do destino: nunca atravessa uma caixa.
//
// As linhas são desenhadas como conexões por célula (cima, baixo,
// esquerda, direita), e o caractere sai do conjunto: dois traços que se
// cruzam viram `┼`, um que encontra outro de lado vira `├`.
//
// Desenha-se sempre de cima pra baixo (TD) ou da esquerda pra direita
// (LR); BT e RL são o mesmo desenho espelhado no fim, com o texto posto
// depois do espelho pra não sair de trás pra frente.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum FormaG {
    Caixa,
    Arredondado,
    Decisao,
    Circulo,
    Inicio,
    Fim,
}

pub(super) struct NoG {
    pub(super) rotulo: String,
    pub(super) forma: FormaG,
    /// Linhas debaixo do nome, depois de um separador: os membros de uma
    /// classe, os atributos de uma entidade (ciclo 499). Vazio é a caixa
    /// de uma linha só do fluxograma.
    pub(super) corpo: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Traco {
    Cheio,
    Pontilhado,
    Grosso,
}

pub(super) struct LigG {
    pub(super) de: usize,
    pub(super) para: usize,
    pub(super) rotulo: String,
    pub(super) traco: Traco,
    pub(super) ponta: bool,
}

/// Rótulo de nó mais comprido que isto é cortado: uma caixa de 60
/// colunas come a tela sozinha.
pub(super) const ROTULO_MAX: usize = 28;

const CIMA: u8 = 1;
const BAIXO: u8 = 2;
const ESQ: u8 = 4;
const DIR: u8 = 8;

#[derive(Clone, Copy, PartialEq)]
enum Geo {
    Vazia,
    Letra(char, Papel),
    Traco { lados: u8, traco: Traco },
}

/// A grade da geometria: caixas, traços e pontas. O texto vem depois.
struct GradeGeo {
    celulas: Vec<Vec<Geo>>,
}

impl GradeGeo {
    fn em(&mut self, x: usize, y: usize) -> &mut Geo {
        while self.celulas.len() <= y {
            self.celulas.push(Vec::new());
        }
        let l = &mut self.celulas[y];
        while l.len() <= x {
            l.push(Geo::Vazia);
        }
        &mut l[x]
    }

    fn letra(&mut self, x: usize, y: usize, c: char, p: Papel) {
        *self.em(x, y) = Geo::Letra(c, p);
    }

    fn lado(&mut self, x: usize, y: usize, lados: u8, traco: Traco) {
        let c = self.em(x, y);
        match c {
            Geo::Traco { lados: l, traco: t } => {
                *l |= lados;
                // Cruzamento de cheio com pontilhado vira cheio: o
                // pontilhado não tem junção.
                if *t != traco {
                    *t = Traco::Cheio;
                }
            }
            Geo::Vazia => *c = Geo::Traco { lados, traco },
            // Traço não passa por cima de caixa.
            Geo::Letra(..) => {}
        }
    }

    /// Um segmento reto de (x0, y0) a (x1, y1).
    fn segmento(&mut self, (x0, y0): (usize, usize), (x1, y1): (usize, usize), traco: Traco) {
        if (x0, y0) == (x1, y1) {
            return;
        }
        if x0 == x1 {
            let (a, b) = (y0.min(y1), y0.max(y1));
            for y in a..=b {
                let mut l = 0;
                if y > a {
                    l |= CIMA;
                }
                if y < b {
                    l |= BAIXO;
                }
                self.lado(x0, y, l, traco);
            }
        } else {
            let (a, b) = (x0.min(x1), x0.max(x1));
            for x in a..=b {
                let mut l = 0;
                if x > a {
                    l |= ESQ;
                }
                if x < b {
                    l |= DIR;
                }
                self.lado(x, y0, l, traco);
            }
        }
    }

    fn altura(&self) -> usize {
        self.celulas.len()
    }

    fn largura(&self) -> usize {
        self.celulas.iter().map(|l| l.len()).max().unwrap_or(0)
    }
}

fn caractere_do_traco(lados: u8, traco: Traco) -> char {
    let vertical = lados & !(CIMA | BAIXO) == 0;
    let horizontal = lados & !(ESQ | DIR) == 0;
    match traco {
        Traco::Pontilhado if vertical => return '╎',
        Traco::Pontilhado if horizontal => return '╌',
        Traco::Grosso if vertical => return '┃',
        Traco::Grosso if horizontal => return '━',
        _ => {}
    }
    match lados {
        0 => ' ',
        l if l == CIMA || l == BAIXO || l == CIMA | BAIXO => '│',
        l if l == ESQ || l == DIR || l == ESQ | DIR => '─',
        l if l == BAIXO | DIR => '┌',
        l if l == BAIXO | ESQ => '┐',
        l if l == CIMA | DIR => '└',
        l if l == CIMA | ESQ => '┘',
        l if l == CIMA | BAIXO | DIR => '├',
        l if l == CIMA | BAIXO | ESQ => '┤',
        l if l == BAIXO | ESQ | DIR => '┬',
        l if l == CIMA | ESQ | DIR => '┴',
        _ => '┼',
    }
}

/// O caractere visto no espelho de cima pra baixo (`vertical`) ou da
/// esquerda pra direita.
fn espelhar(c: char, vertical: bool) -> char {
    let pares: &[(char, char)] = if vertical {
        &[('▼', '▲'), ('┬', '┴'), ('┌', '└'), ('┐', '┘'), ('╭', '╰'), ('╮', '╯'), ('╱', '╲')]
    } else {
        &[('▶', '◀'), ('├', '┤'), ('┌', '┐'), ('└', '┘'), ('╭', '╮'), ('╰', '╯'), ('╱', '╲'), ('<', '>'), ('(', ')')]
    };
    for (a, b) in pares {
        if c == *a {
            return *b;
        }
        if c == *b {
            return *a;
        }
    }
    c
}

/// Tamanho de um nó: (largura, altura) na tela.
fn tamanho(no: &NoG) -> (usize, usize) {
    match no.forma {
        FormaG::Inicio | FormaG::Fim => (1, 1),
        _ => {
            let w = no.corpo.iter().map(|l| largura_de(l)).chain([largura_de(&no.rotulo)]).max().unwrap_or(0);
            (w + 4, if no.corpo.is_empty() { 3 } else { 4 + no.corpo.len() })
        }
    }
}

/// Onde cada ligação passa. `Normal`: da camada k pra k+1, pelo canal entre
/// elas. `Lateral`: por fora.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Rota {
    Normal,
    Lateral,
}

pub(super) fn grafo(nos: &[NoG], ligs: &[LigG], horizontal: bool, invertido: bool, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let n = nos.len();
    if n == 0 {
        return Ok(Vec::new());
    }
    let arestas: Vec<(usize, usize)> = ligs.iter().map(|l| (l.de, l.para)).collect();
    let niv = camadas::niveis(n, &arestas);
    let lista = camadas::camadas(n, &arestas, &niv);
    let nl = lista.len();
    let rota: Vec<Rota> = ligs
        .iter()
        .map(|l| if l.de != l.para && niv[l.para] == niv[l.de] + 1 { Rota::Normal } else { Rota::Lateral })
        .collect();

    // Eixo "maior" é o das camadas (y em TD, x em LR); "menor", o de lado.
    let tam: Vec<(usize, usize)> = nos
        .iter()
        .map(|no| {
            let (w, h) = tamanho(no);
            if horizontal { (w, h) } else { (h, w) }
        })
        .collect(); // (maior, menor)
    let vao_menor = if horizontal { 1 } else { 3 };

    // Posição no eixo menor: camada por camada, centradas.
    let comprimento: Vec<usize> = lista
        .iter()
        .map(|c| c.iter().map(|i| tam[*i].1).sum::<usize>() + vao_menor * c.len().saturating_sub(1))
        .collect();
    let menor_max = comprimento.iter().copied().max().unwrap_or(0);
    // Toda camada centra na MESMA coluna: centrar cada uma pela sobra dava
    // centros a uma coluna de distância entre caixas de larguras pares e
    // ímpares, e a seta entre elas fazia um degrau (`┌┘`) à toa.
    let eixo = menor_max.saturating_sub(1) / 2;
    let mut ini_menor = vec![0usize; n];
    for (k, c) in lista.iter().enumerate() {
        let mut m = eixo.saturating_sub(comprimento[k].saturating_sub(1) / 2);
        for &i in c {
            ini_menor[i] = m;
            m += tam[i].1 + vao_menor;
        }
    }
    let centro_menor = |i: usize| ini_menor[i] + (tam[i].1.saturating_sub(1)) / 2;
    // Em TD os rótulos ficam à direita da ponta, na linha dela: as pistas
    // laterais começam depois do mais comprido, senão o rótulo escreve por
    // cima da pista.
    let mut rotulos_por_destino: std::collections::BTreeMap<usize, Vec<String>> = Default::default();
    if !horizontal {
        for l in ligs.iter().filter(|l| !l.rotulo.is_empty()) {
            rotulos_por_destino.entry(l.para).or_default().push(l.rotulo.clone());
        }
    }
    let alcance_dos_rotulos = rotulos_por_destino
        .iter()
        .map(|(i, r)| centro_menor(*i) + 2 + largura_de(&r.join(" / ")))
        .max()
        .unwrap_or(0);
    let base_lateral = menor_max.max(ini_menor.iter().zip(&tam).map(|(m, t)| m + t.1).max().unwrap_or(0)).max(alcance_dos_rotulos);
    let laterais: Vec<usize> = (0..ligs.len()).filter(|k| rota[*k] == Rota::Lateral).collect();
    let pista_lateral = |k: usize| base_lateral + 1 + 2 * laterais.iter().position(|x| *x == k).unwrap_or(0);

    // Os canais: o canal c fica antes da camada c (c = nl: depois da
    // última). Quem sai da camada k usa o canal k+1; quem entra na k, o k.
    let mut sai = vec![false; nl + 1];
    let mut entra = vec![false; nl + 1];
    let mut pistas: Vec<Vec<(usize, bool)>> = vec![Vec::new(); nl + 1]; // (ligação, é a de saída)
    let mut espaco_rotulo = vec![0usize; nl + 1];
    for (k, l) in ligs.iter().enumerate() {
        let (cs, ce) = (niv[l.de] + 1, niv[l.para]);
        sai[cs] = true;
        entra[ce] = true;
        if horizontal && !l.rotulo.is_empty() {
            espaco_rotulo[ce] = espaco_rotulo[ce].max(largura_de(&l.rotulo) + 2);
        }
        match rota[k] {
            Rota::Normal => {
                if centro_menor(l.de) != centro_menor(l.para) {
                    pistas[cs].push((k, true));
                }
            }
            Rota::Lateral => {
                pistas[cs].push((k, true));
                pistas[ce].push((k, false));
            }
        }
    }
    // Posição no eixo maior: canal, camada, canal, camada...
    let mut ini_canal = vec![0usize; nl + 1];
    let mut ini_camada = vec![0usize; nl];
    let mut grossura = vec![0usize; nl];
    let mut m = 0;
    for c in 0..=nl {
        ini_canal[c] = m;
        let tamanho_canal = usize::from(sai[c]) + pistas[c].len() + espaco_rotulo[c] + usize::from(entra[c]);
        // Entre duas camadas sempre há ao menos uma de folga.
        let minimo = if c > 0 && c < nl { 1 } else { 0 };
        m += tamanho_canal.max(minimo);
        if c < nl {
            ini_camada[c] = m;
            grossura[c] = lista[c].iter().map(|i| tam[*i].0).max().unwrap_or(1);
            m += grossura[c];
        }
    }
    // Em LR a caixa estreita centra na faixa da camada. Em TD as caixas de
    // alturas diferentes (classes e entidades, 499) alinham pelo topo: pelo
    // centro, as setas que descem pra mesma camada chegavam em alturas
    // diferentes, e a caixa baixa parecia de outra camada.
    let ini_maior = |i: usize| ini_camada[niv[i]] + if horizontal { (grossura[niv[i]] - tam[i].0) / 2 } else { 0 };
    let fim_maior = |i: usize| ini_maior(i) + tam[i].0 - 1;
    let pista = |c: usize, k: usize, saida: bool| -> usize {
        ini_canal[c] + usize::from(sai[c]) + pistas[c].iter().position(|p| *p == (k, saida)).unwrap_or(0)
    };

    // (maior, menor) → (x, y)
    let tela = |maior: usize, menor: usize| if horizontal { (maior, menor) } else { (menor, maior) };

    let mut geo = GradeGeo { celulas: Vec::new() };
    let mut textos: Vec<(usize, usize, String, Papel)> = Vec::new();

    // As caixas.
    for (i, no) in nos.iter().enumerate() {
        let (x0, y0) = tela(ini_maior(i), ini_menor[i]);
        match no.forma {
            FormaG::Inicio => geo.letra(x0, y0, '●', Papel::Caixa),
            FormaG::Fim => geo.letra(x0, y0, '◉', Papel::Caixa),
            forma => {
                let (w, h) = tamanho(no);
                let (ce, cd, be, bd, le, ld) = match forma {
                    FormaG::Arredondado => ('╭', '╮', '╰', '╯', '│', '│'),
                    FormaG::Decisao => ('╱', '╲', '╲', '╱', '<', '>'),
                    FormaG::Circulo => ('╭', '╮', '╰', '╯', '(', ')'),
                    _ => ('┌', '┐', '└', '┘', '│', '│'),
                };
                let fundo = y0 + h - 1;
                geo.letra(x0, y0, ce, Papel::Caixa);
                geo.letra(x0 + w - 1, y0, cd, Papel::Caixa);
                geo.letra(x0, fundo, be, Papel::Caixa);
                geo.letra(x0 + w - 1, fundo, bd, Papel::Caixa);
                for x in x0 + 1..x0 + w - 1 {
                    geo.letra(x, y0, '─', Papel::Caixa);
                    geo.letra(x, fundo, '─', Papel::Caixa);
                    for y in y0 + 1..fundo {
                        geo.letra(x, y, ' ', Papel::Nome);
                    }
                }
                for y in y0 + 1..fundo {
                    geo.letra(x0, y, le, Papel::Caixa);
                    geo.letra(x0 + w - 1, y, ld, Papel::Caixa);
                }
                textos.push((x0 + 2, y0 + 1, no.rotulo.clone(), Papel::Nome));
                if !no.corpo.is_empty() {
                    // O separador entre o nome e o corpo, como a classe da
                    // UML: `├───┤`.
                    geo.letra(x0, y0 + 2, '├', Papel::Caixa);
                    geo.letra(x0 + w - 1, y0 + 2, '┤', Papel::Caixa);
                    for x in x0 + 1..x0 + w - 1 {
                        geo.letra(x, y0 + 2, '─', Papel::Caixa);
                    }
                    for (k, l) in no.corpo.iter().enumerate() {
                        textos.push((x0 + 2, y0 + 3 + k, l.clone(), Papel::Rotulo));
                    }
                }
            }
        }
    }

    // As ligações: o caminho em (maior, menor), depois na tela.
    for (k, l) in ligs.iter().enumerate() {
        let (cs, ce) = (niv[l.de] + 1, niv[l.para]);
        let (ms, mt) = (centro_menor(l.de), centro_menor(l.para));
        // Sai da borda de baixo (de trás, em LR) da origem e chega na
        // célula antes da borda de cima do destino.
        let borda_origem = fim_maior(l.de);
        let antes_do_destino = ini_maior(l.para) - 1;
        let mut pontos: Vec<(usize, usize)> = vec![(borda_origem, ms)];
        match rota[k] {
            Rota::Normal => {
                if ms != mt {
                    let p = pista(cs, k, true);
                    pontos.push((p, ms));
                    pontos.push((p, mt));
                }
            }
            Rota::Lateral => {
                let (a, b) = (pista(cs, k, true), pista(ce, k, false));
                let lat = pista_lateral(k);
                pontos.extend([(a, ms), (a, lat), (b, lat), (b, mt)]);
            }
        }
        pontos.push((antes_do_destino, mt));
        let na_tela: Vec<(usize, usize)> = pontos.iter().map(|(a, b)| tela(*a, *b)).collect();
        for par in na_tela.windows(2) {
            geo.segmento(par[0], par[1], l.traco);
        }
        // A saída marca a borda; a chegada, a ponta.
        let origem_ponto = matches!(nos[l.de].forma, FormaG::Inicio | FormaG::Fim);
        let (bx, by) = na_tela[0];
        if !origem_ponto {
            geo.letra(bx, by, if horizontal { '├' } else { '┬' }, Papel::Caixa);
        } else {
            let ch = if nos[l.de].forma == FormaG::Inicio { '●' } else { '◉' };
            geo.letra(bx, by, ch, Papel::Caixa);
        }
        let (ax, ay) = *na_tela.last().unwrap();
        if l.ponta {
            geo.letra(ax, ay, if horizontal { '▶' } else { '▼' }, Papel::Seta);
        }
        if !l.rotulo.is_empty() {
            if horizontal {
                // Em LR o rótulo vai em cima do trecho que entra no destino,
                // no espaço que o canal reservou.
                let m0 = ini_canal[ce] + usize::from(sai[ce]) + pistas[ce].len() + 1;
                let (x, y) = tela(m0, mt);
                textos.push((x, y, l.rotulo.clone(), Papel::Rotulo));
            }
        }
    }
    // Em TD, os rótulos ao lado da ponta, juntos quando várias chegam no
    // mesmo destino.
    for (i, rots) in rotulos_por_destino {
        let (x, y) = tela(ini_maior(i) - 1, centro_menor(i));
        textos.push((x + 2, y, rots.join(" / "), Papel::Rotulo));
    }

    // Espelho, pra BT e RL: a geometria vira, o texto é posto depois.
    let (w_geo, h_geo) = (geo.largura(), geo.altura());
    let mut final_ = Grade::nova();
    for (y, linha) in geo.celulas.iter().enumerate() {
        for (x, c) in linha.iter().enumerate() {
            let (xx, yy) = match (invertido, horizontal) {
                (false, _) => (x, y),
                (true, false) => (x, h_geo - 1 - y),
                (true, true) => (w_geo - 1 - x, y),
            };
            let virar = |ch: char| if invertido { espelhar(ch, !horizontal) } else { ch };
            match *c {
                Geo::Vazia => {}
                Geo::Letra(ch, p) => final_.letra(xx, yy, virar(ch), p),
                Geo::Traco { lados, traco } => {
                    let lados = if !invertido {
                        lados
                    } else if horizontal {
                        (lados & (CIMA | BAIXO)) | if lados & ESQ != 0 { DIR } else { 0 } | if lados & DIR != 0 { ESQ } else { 0 }
                    } else {
                        (lados & (ESQ | DIR)) | if lados & CIMA != 0 { BAIXO } else { 0 } | if lados & BAIXO != 0 { CIMA } else { 0 }
                    };
                    final_.letra(xx, yy, caractere_do_traco(lados, traco), Papel::Seta);
                }
            }
        }
    }
    for (x, y, t, p) in textos {
        let w = largura_de(&t);
        let (xx, yy) = match (invertido, horizontal) {
            (false, _) => (x, y),
            (true, false) => (x, h_geo - 1 - y),
            (true, true) => (w_geo.saturating_sub(x + w), y),
        };
        final_.texto(xx, yy, &t, p);
    }
    let linhas = final_.linhas();
    let precisa = linhas.iter().map(|l| l.iter().map(|(_, s)| largura_de(s)).sum::<usize>()).max().unwrap_or(0);
    if precisa > largura {
        return Err(NaoDesenha::NaoCabe { precisa });
    }
    Ok(linhas)
}

fn fluxograma(f: &Fluxograma, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    let idx = |id: &str| f.nos.iter().position(|n| n.id == id);
    let nos: Vec<NoG> = f
        .nos
        .iter()
        .map(|n| NoG {
            rotulo: cortar(&n.texto, ROTULO_MAX),
            forma: match n.forma {
                Forma::Caixa => FormaG::Caixa,
                Forma::Arredondado => FormaG::Arredondado,
                Forma::Decisao => FormaG::Decisao,
                Forma::Circulo => FormaG::Circulo,
            },
            corpo: Vec::new(),
        })
        .collect();
    let ligs: Vec<LigG> = f
        .ligacoes
        .iter()
        .filter_map(|l| {
            Some(LigG {
                de: idx(&l.de)?,
                para: idx(&l.para)?,
                rotulo: cortar(&l.rotulo, ROTULO_MAX),
                traco: match l.tipo {
                    TipoLigacao::Pontilhada => Traco::Pontilhado,
                    TipoLigacao::Grossa => Traco::Grosso,
                    _ => Traco::Cheio,
                },
                ponta: l.tipo != TipoLigacao::Linha,
            })
        })
        .collect();
    let (horizontal, invertido) = match f.direcao {
        Direcao::TD | Direcao::TB => (false, false),
        Direcao::BT => (false, true),
        Direcao::LR => (true, false),
        Direcao::RL => (true, true),
    };
    grafo(&nos, &ligs, horizontal, invertido, largura)
}

fn estados(e: &Estados, largura: usize) -> Result<Vec<Linha>, NaoDesenha> {
    // Início e fim só entram se alguma transição os usa: um `[*]` solto
    // seria um ponto sem sentido no desenho.
    let usa = |p: &Ponta| e.transicoes.iter().any(|t| &t.de == p || &t.para == p);
    let mut nos: Vec<NoG> = Vec::new();
    let mut chaves: Vec<Ponta> = Vec::new();
    if usa(&Ponta::Inicio) {
        nos.push(NoG { rotulo: String::new(), forma: FormaG::Inicio, corpo: Vec::new() });
        chaves.push(Ponta::Inicio);
    }
    for s in &e.estados {
        nos.push(NoG { rotulo: cortar(&s.texto, ROTULO_MAX), forma: FormaG::Arredondado, corpo: Vec::new() });
        chaves.push(Ponta::Estado(s.id.clone()));
    }
    if usa(&Ponta::Fim) {
        nos.push(NoG { rotulo: String::new(), forma: FormaG::Fim, corpo: Vec::new() });
        chaves.push(Ponta::Fim);
    }
    let idx = |p: &Ponta| chaves.iter().position(|c| c == p);
    let ligs: Vec<LigG> = e
        .transicoes
        .iter()
        .filter_map(|t| {
            Some(LigG {
                de: idx(&t.de)?,
                para: idx(&t.para)?,
                rotulo: cortar(&t.rotulo, ROTULO_MAX),
                traco: Traco::Cheio,
                ponta: true,
            })
        })
        .collect();
    grafo(&nos, &ligs, false, false, largura)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::diagrama::ler;

    fn texto(linhas: &[Linha]) -> Vec<String> {
        linhas.iter().map(|l| l.iter().map(|(_, s)| s.as_str()).collect()).collect()
    }

    fn desenho(codigo: &str, largura: usize) -> Vec<String> {
        texto(&desenhar(&ler(codigo).expect("dentro do modelo"), largura).expect("desenha"))
    }

    #[test]
    fn sequencia_do_exemplo() {
        let t = desenho("sequenceDiagram\n  actor P as Pessoa\n  participant S as Sistema\n  P->>S: pede\n  S-->>P: responde\n  Note right of S: pensa\n", 80);
        let esperado = [
            "╭────────╮  ┌─────────┐",
            "│ Pessoa │  │ Sistema │",
            "╰───┬────╯  └────┬────┘",
            "    │    pede    │",
            "    │───────────▶│",
            "    │  responde  │",
            "    │◀╌╌╌╌╌╌╌╌╌╌╌│",
            "    │            │ ┌───────┐",
            "    │            │ │ pensa │",
            "    │            │ └───────┘",
            "    │            │",
            "╭───┴────╮  ┌────┴────┐",
            "│ Pessoa │  │ Sistema │",
            "╰────────╯  └─────────┘",
        ];
        assert_eq!(t, esperado, "\n{}", t.join("\n"));
    }

    #[test]
    fn rotulo_comprido_afasta_as_colunas_e_largo_demais_nao_cabe() {
        let codigo = "sequenceDiagram\n  A->>B: uma mensagem bem comprida mesmo\n";
        let t = desenho(codigo, 80);
        assert!(t[3].contains("uma mensagem bem comprida mesmo"), "{t:?}");
        let seta = &t[4];
        assert!(seta.ends_with("▶│"), "{seta:?}");
        let d = desenhar(&ler(codigo).unwrap(), 20);
        assert!(matches!(d, Err(NaoDesenha::NaoCabe { precisa }) if precisa > 20), "{d:?}");
    }

    #[test]
    fn mensagem_pra_si_e_setas() {
        let t = desenho("sequenceDiagram\n  A->>A: pensa\n  A-)B: solta\n  B-xA: falha\n  A--)B: ok\n", 80);
        let tudo = t.join("\n");
        assert!(tudo.contains("│───╮") && tudo.contains("│◀──╯"), "laço:\n{tudo}");
        assert!(tudo.contains("▷"), "assíncrona:\n{tudo}");
        assert!(tudo.contains("×"), "cruz:\n{tudo}");
        assert!(tudo.contains("╌▷"), "assíncrona tracejada:\n{tudo}");
    }

    #[test]
    fn nota_sobre_dois_cobre_os_dois() {
        let t = desenho("sequenceDiagram\n  participant A\n  participant B\n  Note over A,B: juntos\n", 80);
        // Posição em CARACTERES: os de caixa têm vários bytes.
        let pos = |l: &str, c: char| l.chars().enumerate().filter(|(_, x)| *x == c).map(|(i, _)| i).collect::<Vec<_>>();
        let linha = t.iter().find(|l| l.contains("juntos")).unwrap();
        let colunas = pos(&t[2], '┬');
        let bordas = pos(linha, '│');
        assert!(bordas[0] < colunas[0], "a nota não cobre A:\n{}", t.join("\n"));
        assert!(bordas[1] > colunas[1], "a nota não cobre B:\n{}", t.join("\n"));
    }

    #[test]
    fn largura_de_caractere_largo_conta_dois() {
        let t = desenho("sequenceDiagram\n  participant 日本\n  participant B\n  日本->>B: x\n", 80);
        // A caixa de 日本 tem 4 de texto + 4: as bordas alinham.
        assert_eq!(largura_de(&t[0]), largura_de(&t[1]), "\n{}", t.join("\n"));
    }

    const GANTT: &str = "gantt\n  title Plano\n  dateFormat YYYY-MM-DD\n  section Fazer\n    Escrever :done, a1, 2026-01-01, 4d\n    Revisar :active, a2, after a1, 3d\n  section Entregar\n    Publicar :crit, 2026-01-10, 2026-01-12\n    Lançar :milestone, after a2, 1d\n";

    #[test]
    fn gantt_barras_no_tempo() {
        let t = desenho(GANTT, 60);
        assert_eq!(t[0], "Plano");
        // A régua começa depois da coluna dos nomes: 2 de recuo, 8 de
        // "Escrever" (o maior), 1 de folga.
        assert!(t[1].starts_with(&format!("{}01/01", " ".repeat(11))), "eixo:\n{}", t.join("\n"));
        assert_eq!(t[3], "Fazer");
        let escrever = &t[4];
        let revisar = &t[5];
        assert!(escrever.contains("▓"), "{escrever:?}");
        // Revisar começa onde Escrever acaba (after a1).
        let fim_escrever = escrever.rfind('▓').unwrap() + '▓'.len_utf8();
        let ini_revisar = revisar.find('█').unwrap();
        assert_eq!(largura_de(&escrever[..fim_escrever]), largura_de(&revisar[..ini_revisar]), "\n{}", t.join("\n"));
        assert_eq!(t[6], "Entregar");
        assert!(t[8].contains('◆'), "marco:\n{}", t.join("\n"));
        // Ninguém passa da largura.
        assert!(t.iter().all(|l| largura_de(l) <= 60), "\n{}", t.join("\n"));
        // E os papéis vêm por marca.
        let d = desenhar(&ler(GANTT).unwrap(), 60).unwrap();
        assert!(d[7].iter().any(|(p, _)| *p == Papel::BarraCritica));
        assert!(d[5].iter().any(|(p, _)| *p == Papel::BarraAtiva));
    }

    #[test]
    fn gantt_sem_data_ou_estreito_nao_desenha() {
        let sem = ler("gantt\n  Tarefa :a1, after b9, 2d\n").unwrap();
        assert!(matches!(desenhar(&sem, 80), Err(NaoDesenha::SemData(_))));
        assert!(matches!(desenhar(&ler(GANTT).unwrap(), 20), Err(NaoDesenha::NaoCabe { .. })));
    }

    /// O fluxograma do print do ciclo 491: oito passos, uma volta que pula
    /// três camadas e uma decisão que volta pra camada de cima.
    const FLUXO_DO_PRINT: &str = "graph TD\n  A[Ideia] --> B[Detalhando]\n  B --> C[Pronta]\n  C --> D{Executar?}\n  D -->|não| C\n  D -->|sim| E[Executando]\n  E --> F[Revisão]\n  E -->|parou| G[Bloqueada]\n  F --> H[Feita]\n  F -->|voltou| B\n";

    #[test]
    fn as_voltas_correm_por_fora_e_nenhuma_caixa_e_cortada() {
        let t = desenho(FLUXO_DO_PRINT, 80);
        let tudo = t.join("\n");
        for caixa in ["│ Ideia │", "│ Detalhando │", "│ Pronta │", "< Executar? >", "│ Executando │", "│ Revisão │", "│ Bloqueada │", "│ Feita │"] {
            assert!(tudo.contains(caixa), "{caixa} cortada ou sumida:\n{tudo}");
        }
        for rotulo in ["▼ voltou", "▼ não", "▼ sim", "▼ parou"] {
            assert!(tudo.contains(rotulo), "{rotulo}:\n{tudo}");
        }
        // Nós sozinhos na camada ficam na mesma coluna: a seta desce reta,
        // sem degrau.
        assert!(!tudo.contains("┌┘") && !tudo.contains("└┐"), "degrau na seta:\n{tudo}");
        assert!(t.iter().all(|l| largura_de(l) <= 80));
    }

    #[test]
    fn estados_com_inicio_fim_e_volta() {
        let t = desenho("stateDiagram-v2\n  [*] --> Rascunho\n  Rascunho --> Revisao: envia\n  Revisao --> Rascunho: pede mudança\n  Revisao --> Aprovada: aprova\n  Aprovada --> [*]\n", 80);
        let tudo = t.join("\n");
        assert!(t[0].trim() == "●", "o início no topo:\n{tudo}");
        assert!(t.last().unwrap().trim() == "◉", "o fim embaixo:\n{tudo}");
        // O rótulo da volta não é atravessado pela pista lateral.
        assert!(tudo.contains("▼ pede mudança "), "{tudo}");
        assert!(tudo.contains("╭──────────╮") && tudo.contains("│ Aprovada │"), "{tudo}");
    }

    #[test]
    fn da_esquerda_pra_direita_e_espelhados() {
        let lr = desenho("graph LR\n  A[Começo] --> B[Fim]\n", 80);
        let linha = lr.iter().find(|l| l.contains("Começo")).unwrap();
        assert!(linha.find("Começo") < linha.find("Fim") && linha.contains("─▶"), "{lr:?}");
        let rl = desenho("graph RL\n  A[Começo] --> B[Fim]\n", 80);
        let linha = rl.iter().find(|l| l.contains("Começo")).unwrap();
        assert!(linha.find("Fim") < linha.find("Começo") && linha.contains("◀─"), "o texto saiu espelhado ou a seta não virou: {rl:?}");
        let bt = desenho("graph BT\n  A[Base] --> B[Topo]\n", 80);
        let (base, topo) = (bt.iter().position(|l| l.contains("Base")).unwrap(), bt.iter().position(|l| l.contains("Topo")).unwrap());
        assert!(topo < base && bt.join("").contains('▲'), "{bt:?}");
    }

    #[test]
    fn grafo_largo_demais_nao_cabe() {
        let d = ler("graph LR\n  A[um nó com nome comprido] --> B[outro nó com nome comprido]\n").unwrap();
        assert!(matches!(desenhar(&d, 40), Err(NaoDesenha::NaoCabe { precisa }) if precisa > 40));
    }

    #[test]
    fn inteiro_desenha_o_que_nao_cabe_na_largura_dele() {
        // Os quatro tipos, cada um largo demais pra 20 colunas.
        for fonte in [
            "graph LR\n  A[um nó com nome comprido] --> B[outro nó com nome comprido]\n",
            "sequenceDiagram\n  A->>B: uma mensagem que não cabe em vinte colunas\n",
            "stateDiagram-v2\n  s1 : Um estado de nome comprido\n  [*] --> s1\n  s1 --> s2\n  s2 : Outro estado também comprido\n",
            GANTT,
        ] {
            let d = ler(fonte).unwrap();
            let t = desenhar_inteiro(&d, 20).unwrap_or_else(|e| panic!("{fonte}: {e:?}"));
            let larg = t.iter().map(|l| l.iter().map(|(_, s)| largura_de(s)).sum::<usize>()).max().unwrap();
            assert!(larg > 20, "{fonte}: deveria ser mais largo que 20, tem {larg}");
            // Na largura que pede, é o mesmo desenho que `desenhar` daria.
            // O gantt fica de fora: ele estica as barras até a largura dada.
            if fonte != GANTT {
                assert_eq!(desenhar(&d, larg).unwrap(), t, "{fonte}");
            }
        }
        // O que cabe sai igual.
        let d = ler("graph TD\n  A --> B\n").unwrap();
        assert_eq!(desenhar_inteiro(&d, 80), desenhar(&d, 80));
    }

    #[test]
    fn grafos_quaisquer_desenham_sem_estourar() {
        // Grafos de até 7 nós com ligações ao acaso (voltas, laços, saltos,
        // mesma camada), nas quatro direções: nunca entra em pânico, todo
        // nó aparece e nenhuma linha passa da largura.
        let mut semente: u64 = 0x9E37_79B9_7F4A_7C15;
        let mut acaso = |n: u64| {
            semente ^= semente << 13;
            semente ^= semente >> 7;
            semente ^= semente << 17;
            semente % n
        };
        for _ in 0..300 {
            let n = 1 + acaso(7) as usize;
            let dir = ["TD", "LR", "BT", "RL"][acaso(4) as usize];
            let mut codigo = format!("graph {dir}\n");
            for i in 0..n {
                codigo.push_str(&format!("  N{i}[nó {i}]\n"));
            }
            for _ in 0..acaso(10) {
                let (a, b) = (acaso(n as u64), acaso(n as u64));
                let rot = if acaso(2) == 0 { String::new() } else { format!("|r{a}{b}|") };
                codigo.push_str(&format!("  N{a} -->{rot} N{b}\n"));
            }
            let d = ler(&codigo).unwrap_or_else(|e| panic!("{e:?}\n{codigo}"));
            match desenhar(&d, 200) {
                Ok(linhas) => {
                    let t = texto(&linhas);
                    let tudo = t.join("\n");
                    for i in 0..n {
                        assert!(tudo.contains(&format!("nó {i}")), "nó {i} sumiu:\n{codigo}\n{tudo}");
                    }
                    assert!(t.iter().all(|l| largura_de(l) <= 200), "{codigo}\n{tudo}");
                }
                Err(NaoDesenha::NaoCabe { .. }) => {}
                Err(e) => panic!("{e:?}\n{codigo}"),
            }
        }
    }

    #[test]
    fn fluxograma_simples_de_cima_pra_baixo() {
        let t = desenho("graph TD\n  A[Começo] --> B{Decide}\n  B -->|sim| C(Faz)\n  B -->|não| D((Fim))\n", 80);
        let tudo = t.join("\n");
        for nome in ["Começo", "Decide", "Faz", "Fim"] {
            assert!(tudo.contains(nome), "{nome} sumiu:\n{tudo}");
        }
        assert!(tudo.contains("▼ sim") && tudo.contains("▼ não"), "rótulos:\n{tudo}");
        assert!(tudo.contains("< Decide >"), "decisão:\n{tudo}");
        assert!(tudo.contains("( Fim )"), "círculo:\n{tudo}");
        assert!(tudo.contains("╭─────╮"), "arredondado:\n{tudo}");
    }
}
