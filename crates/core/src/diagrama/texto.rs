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
        Diagrama::Fluxograma(_) | Diagrama::Estados(_) => Err(NaoDesenha::SemDesenho),
    }
}

/// Largura de `s` em células.
pub fn largura_de(s: &str) -> usize {
    s.chars().map(|c| c.width().unwrap_or(0)).sum()
}

/// `s` cortado em `max` células, com `…` quando cortou.
fn cortar(s: &str, max: usize) -> String {
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

    #[test]
    fn fluxograma_e_estados_ainda_sem_desenho() {
        let f = ler("graph TD\n  A --> B\n").unwrap();
        assert_eq!(desenhar(&f, 80), Err(NaoDesenha::SemDesenho));
    }
}
