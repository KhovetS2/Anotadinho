//! O editor de código da TUI (ciclo 506).
//!
//! O de 390 era um campo de várias linhas: sem cor, sem indentação, Tab
//! virava quatro espaços onde estivesse, e uma linha mais larga que o modal
//! saía cortada com o cursor fora da tela. Pra quem escreve código, isso
//! obriga a contar espaços na mão. Este editor faz o que um editor de código
//! faz no mínimo:
//!
//! - cor da linguagem (a mesma do bloco na página, 495) e número de linha;
//! - Enter mantém a indentação, e entra um nível depois de `{ ( [ :`;
//!   `}` numa linha só de espaços volta um nível;
//! - Tab e Shift+Tab indentam e desindentam de 4 em 4; Backspace no recuo
//!   apaga até o nível anterior;
//! - Home vai ao primeiro caractere que não é espaço (de novo, à coluna 0);
//! - PgUp/PgDn, Ctrl+Home/End;
//! - desfazer e refazer (Ctrl+Z / Ctrl+Y) do próprio editor;
//! - a vista acompanha o cursor nas duas direções, e o rodapé diz linha,
//!   coluna e linguagem.
//!
//! O estado é puro (texto, cursor, histórico); quem desenha lembra a
//! rolagem num `Cell`, como a vista do bloco (498).

use std::cell::Cell;

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;
use unicode_width::UnicodeWidthChar;

use crate::componentes::{self, Campo};
use crate::realce_codigo::{self, Categoria};
use crate::tema::{Realce, Tema};

/// Um nível de indentação.
const NIVEL: usize = 4;
/// Quantos passos o desfazer guarda.
const HISTORICO: usize = 500;

/// As linguagens do seletor (Ctrl+L), na ordem em que mais aparecem num
/// vault de notas. O nome é o que vai na cerca.
pub const LINGUAGENS: &[(&str, &str)] = &[
    ("", "sem linguagem"),
    ("mermaid", "Mermaid (diagrama)"),
    ("rust", "Rust"),
    ("python", "Python"),
    ("javascript", "JavaScript"),
    ("typescript", "TypeScript"),
    ("bash", "Shell (bash)"),
    ("json", "JSON"),
    ("yaml", "YAML"),
    ("toml", "TOML"),
    ("html", "HTML"),
    ("css", "CSS"),
    ("sql", "SQL"),
    ("markdown", "Markdown"),
    ("go", "Go"),
    ("java", "Java"),
    ("c", "C"),
    ("cpp", "C++"),
    ("csharp", "C#"),
    ("ruby", "Ruby"),
    ("php", "PHP"),
    ("lua", "Lua"),
    ("diff", "Diff"),
    ("xml", "XML"),
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tipo {
    Letra,
    Outro,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct EditorDeCodigo {
    pub campo: Campo,
    /// A linguagem da cerca (`None`: sem linguagem).
    pub lingua: Option<String>,
    desfazer: Vec<(String, usize)>,
    refazer: Vec<(String, usize)>,
    /// O tipo da última edição: letras seguidas viram UM passo do desfazer,
    /// como nos editores — desfazer letra por letra cansa.
    ultima: Option<TipoDaEdicao>,
    /// O seletor de linguagem aberto, com a opção marcada.
    pub seletor: Option<usize>,
    /// A busca (ciclo 509): o termo e se ele ainda está sendo digitado.
    pub busca: Option<(String, bool)>,
    /// A primeira linha e a primeira coluna à vista, e a altura do último
    /// quadro (pro PgUp/PgDn).
    pub topo: Cell<usize>,
    pub esquerda: Cell<usize>,
    pub alto: Cell<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct TipoDaEdicao(Tipo);

/// O que a tecla fez com o editor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Resposta {
    /// Usou a tecla.
    Usou,
    /// Não é tecla do editor: quem contém decide (Esc, Ctrl+S, Ctrl+C…).
    Ignorou,
    /// A linguagem mudou pelo seletor.
    Linguagem,
}

fn chars(s: &str) -> Vec<char> {
    s.chars().collect()
}

impl EditorDeCodigo {
    pub fn novo(texto: impl Into<String>, lingua: Option<String>) -> Self {
        let texto = texto.into();
        // O cursor começa no começo: quem abre um bloco pra editar quer ver
        // o topo dele, não o fim.
        Self { campo: Campo { texto, cursor: 0 }, lingua: lingua.filter(|l| !l.trim().is_empty()), ..Default::default() }
    }

    pub fn texto(&self) -> &str {
        &self.campo.texto
    }

    /// Cola `texto` no cursor como VEIO (ciclo 510): sem o recuo do Enter,
    /// que somado ao recuo do próprio texto colado entortava cada linha, e
    /// como um passo só do desfazer.
    pub fn colar(&mut self, texto: &str) {
        let texto = texto.replace("\r\n", "\n").replace('\r', "\n");
        if texto.is_empty() {
            return;
        }
        self.guardar(Tipo::Outro);
        self.ultima = None;
        self.inserir(&texto);
    }

    /// Troca o texto inteiro (o construtor reescreveu o código pelo
    /// formulário), como UM passo do desfazer.
    pub fn trocar(&mut self, texto: String) {
        if texto == self.campo.texto {
            return;
        }
        self.guardar(Tipo::Outro);
        self.campo.cursor = self.campo.cursor.min(texto.chars().count());
        self.campo.texto = texto;
    }

    /// Linha e coluna do cursor, contando de 0.
    pub fn linha_coluna(&self) -> (usize, usize) {
        let cs = chars(&self.campo.texto);
        let c = self.campo.cursor.min(cs.len());
        let linha = cs[..c].iter().filter(|x| **x == '\n').count();
        let ini = cs[..c].iter().rposition(|x| *x == '\n').map_or(0, |i| i + 1);
        (linha, c - ini)
    }

    fn inicio_da_linha(cs: &[char], p: usize) -> usize {
        cs[..p].iter().rposition(|x| *x == '\n').map_or(0, |i| i + 1)
    }

    fn fim_da_linha(cs: &[char], p: usize) -> usize {
        cs[p..].iter().position(|x| *x == '\n').map_or(cs.len(), |i| p + i)
    }

    /// Guarda o estado de antes de uma edição. Letras seguidas são um passo
    /// só; qualquer outra coisa abre um passo novo.
    fn guardar(&mut self, tipo: Tipo) {
        let novo_passo = tipo != Tipo::Letra || self.ultima != Some(TipoDaEdicao(Tipo::Letra));
        if novo_passo {
            self.desfazer.push((self.campo.texto.clone(), self.campo.cursor));
            if self.desfazer.len() > HISTORICO {
                self.desfazer.remove(0);
            }
        }
        self.refazer.clear();
        self.ultima = Some(TipoDaEdicao(tipo));
    }

    fn inserir(&mut self, s: &str) {
        let i = self.campo.texto.char_indices().nth(self.campo.cursor).map_or(self.campo.texto.len(), |(i, _)| i);
        self.campo.texto.insert_str(i, s);
        self.campo.cursor += s.chars().count();
    }

    /// Apaga `n` caracteres antes do cursor.
    fn apagar_antes(&mut self, n: usize) {
        let mut cs = chars(&self.campo.texto);
        let c = self.campo.cursor.min(cs.len());
        let n = n.min(c);
        cs.drain(c - n..c);
        self.campo.texto = cs.into_iter().collect();
        self.campo.cursor = c - n;
    }

    /// O recuo (espaços) da linha que começa em `ini`.
    fn recuo(cs: &[char], ini: usize) -> usize {
        cs[ini..].iter().take_while(|c| **c == ' ').count()
    }

    /// A tecla é do editor mesmo sendo Esc, Ctrl+S ou Ctrl+C: com o
    /// seletor aberto, com o termo da busca sendo digitado, ou o Esc que
    /// limpa a busca antes de fechar.
    pub fn prende(&self, tecla: &str) -> bool {
        self.seletor.is_some() || self.busca.as_ref().is_some_and(|(_, d)| *d) || (tecla == "Escape" && self.busca.is_some())
    }

    /// Vai ao próximo `termo` depois do cursor (ou antes, pra trás), dando
    /// a volta, sem diferença de maiúscula.
    fn ir_ao_achado(&mut self, frente: bool) -> bool {
        let Some((termo, _)) = &self.busca else { return false };
        let termo: Vec<char> = termo.to_lowercase().chars().collect();
        if termo.is_empty() {
            return false;
        }
        let texto: Vec<char> = self.campo.texto.to_lowercase().chars().collect();
        let n = termo.len();
        let pos: Vec<usize> = (0..texto.len().saturating_sub(n - 1)).filter(|i| texto[*i..*i + n] == termo[..]).collect();
        let c = self.campo.cursor;
        let alvo = if frente { pos.iter().find(|p| **p > c).or(pos.first()) } else { pos.iter().rev().find(|p| **p < c).or(pos.last()) };
        match alvo {
            Some(p) => {
                self.campo.cursor = *p;
                true
            }
            None => false,
        }
    }

    pub fn tecla(&mut self, tecla: &str) -> Resposta {
        if let Some(i) = self.seletor {
            return self.tecla_no_seletor(i, tecla);
        }
        if let Some((termo, true)) = self.busca.as_mut() {
            match tecla {
                "Escape" => self.busca = None,
                "Backspace" => {
                    termo.pop();
                }
                "Enter" => {
                    if let Some((_, d)) = self.busca.as_mut() {
                        *d = false;
                    }
                    // A partir de onde se está: o primeiro achado pode ser
                    // o do próprio cursor.
                    self.campo.cursor = self.campo.cursor.saturating_sub(1);
                    self.ir_ao_achado(true);
                }
                t if t.chars().count() == 1 => termo.push_str(t),
                _ => {}
            }
            return Resposta::Usou;
        }
        match tecla {
            "Ctrl+f" => {
                let termo = self.busca.take().map(|(t, _)| t).unwrap_or_default();
                self.busca = Some((termo, true));
                return Resposta::Usou;
            }
            "Ctrl+g" => {
                self.ir_ao_achado(true);
                return Resposta::Usou;
            }
            "Escape" if self.busca.is_some() => {
                self.busca = None;
                return Resposta::Usou;
            }
            _ => {}
        }
        let cs = chars(&self.campo.texto);
        let c = self.campo.cursor.min(cs.len());
        let ini = Self::inicio_da_linha(&cs, c);
        let fim = Self::fim_da_linha(&cs, c);
        let coluna = c - ini;
        let so_espacos_antes = cs[ini..c].iter().all(|x| *x == ' ');
        match tecla {
            "Enter" => {
                self.guardar(Tipo::Outro);
                let recuo = Self::recuo(&cs, ini).min(coluna);
                let antes = cs[ini..c].iter().collect::<String>();
                let abre = antes.trim_end().chars().last().is_some_and(|x| matches!(x, '{' | '(' | '[' | ':'));
                let fecha_logo = cs.get(c).is_some_and(|x| matches!(x, '}' | ')' | ']'));
                let dentro = recuo + if abre { NIVEL } else { 0 };
                self.inserir(&format!("\n{}", " ".repeat(dentro)));
                // `{|}` + Enter: o `}` desce pra linha dele, no recuo de fora.
                if abre && fecha_logo {
                    let volta = self.campo.cursor;
                    self.inserir(&format!("\n{}", " ".repeat(recuo)));
                    self.campo.cursor = volta;
                }
            }
            "Tab" => {
                self.guardar(Tipo::Outro);
                let faltam = NIVEL - coluna % NIVEL;
                self.inserir(&" ".repeat(faltam));
            }
            "Shift+Tab" => {
                let recuo = Self::recuo(&cs, ini);
                let tirar = if recuo == 0 { 0 } else { (recuo - 1) % NIVEL + 1 };
                if tirar > 0 {
                    self.guardar(Tipo::Outro);
                    let mut v = cs.clone();
                    v.drain(ini..ini + tirar);
                    self.campo.texto = v.into_iter().collect();
                    self.campo.cursor = c.saturating_sub(tirar).max(ini);
                }
            }
            "Backspace" => {
                if c == 0 {
                    return Resposta::Usou;
                }
                self.guardar(Tipo::Outro);
                // No recuo, volta um nível inteiro.
                let n = if so_espacos_antes && coluna > 0 { (coluna - 1) % NIVEL + 1 } else { 1 };
                self.apagar_antes(n);
            }
            "Delete" => {
                if c < cs.len() {
                    self.guardar(Tipo::Outro);
                    self.campo.tecla("Delete");
                }
            }
            "ArrowUp" | "ArrowDown" => componentes::cursor_vertical(&mut self.campo, tecla == "ArrowDown"),
            "ArrowLeft" | "ArrowRight" => {
                self.campo.tecla(tecla);
            }
            "Home" => {
                // O primeiro caractere da linha que não é espaço; se já
                // está nele, a coluna 0.
                let primeiro = ini + Self::recuo(&cs, ini);
                self.campo.cursor = if c == primeiro { ini } else { primeiro };
            }
            "End" => self.campo.cursor = fim,
            "Ctrl+Home" => self.campo.cursor = 0,
            "Ctrl+End" => self.campo.cursor = cs.len(),
            "PageUp" | "PageDown" => {
                for _ in 0..self.alto.get().max(2) - 1 {
                    componentes::cursor_vertical(&mut self.campo, tecla == "PageDown");
                }
            }
            "Ctrl+z" => {
                if let Some((t, k)) = self.desfazer.pop() {
                    self.refazer.push((std::mem::replace(&mut self.campo.texto, t), self.campo.cursor));
                    self.campo.cursor = k;
                    self.ultima = None;
                }
            }
            "Ctrl+y" | "Ctrl+r" => {
                if let Some((t, k)) = self.refazer.pop() {
                    self.desfazer.push((std::mem::replace(&mut self.campo.texto, t), self.campo.cursor));
                    self.campo.cursor = k;
                    self.ultima = None;
                }
            }
            "Ctrl+l" => {
                let atual = self.lingua.as_deref().unwrap_or("");
                self.seletor = Some(LINGUAGENS.iter().position(|(k, _)| k.eq_ignore_ascii_case(atual)).unwrap_or(0));
            }
            // Apaga a linha inteira, como o `dd` do vim.
            "Ctrl+k" => {
                self.guardar(Tipo::Outro);
                let mut v = cs.clone();
                let ate = if fim < v.len() { fim + 1 } else { fim };
                let de = if ate == fim && ini > 0 { ini - 1 } else { ini };
                v.drain(de..ate);
                self.campo.texto = v.into_iter().collect();
                self.campo.cursor = de.min(self.campo.texto.chars().count());
                let cs = chars(&self.campo.texto);
                let ini = Self::inicio_da_linha(&cs, self.campo.cursor);
                self.campo.cursor = ini + Self::recuo(&cs, ini);
            }
            t if t.chars().count() == 1 => {
                let ch = t.chars().next().unwrap();
                self.guardar(if ch.is_alphanumeric() { Tipo::Letra } else { Tipo::Outro });
                // `}` numa linha só de espaços volta um nível antes.
                if matches!(ch, '}' | ')' | ']') && so_espacos_antes && coluna >= NIVEL {
                    self.apagar_antes((coluna - 1) % NIVEL + 1);
                }
                self.inserir(t);
            }
            _ => return Resposta::Ignorou,
        }
        Resposta::Usou
    }

    fn tecla_no_seletor(&mut self, i: usize, tecla: &str) -> Resposta {
        let n = LINGUAGENS.len();
        match tecla {
            "j" | "ArrowDown" => self.seletor = Some((i + 1) % n),
            "k" | "ArrowUp" => self.seletor = Some((i + n - 1) % n),
            "Enter" => {
                self.seletor = None;
                let nova = LINGUAGENS[i].0;
                self.lingua = (!nova.is_empty()).then(|| nova.to_string());
                return Resposta::Linguagem;
            }
            "g" | "Home" => self.seletor = Some(0),
            "G" | "End" => self.seletor = Some(n - 1),
            "Escape" | "Ctrl+l" => self.seletor = None,
            // Digitar procura pela primeira letra, como numa lista do sistema.
            t if t.chars().count() == 1 => {
                let ch = t.chars().next().unwrap().to_ascii_lowercase();
                if let Some(k) = (1..=n).map(|d| (i + d) % n).find(|k| LINGUAGENS[*k].1.to_lowercase().starts_with(ch)) {
                    self.seletor = Some(k);
                }
            }
            _ => {}
        }
        Resposta::Usou
    }

    /// O rodapé: onde está o cursor e a linguagem.
    pub fn situacao(&self) -> String {
        if let Some((termo, digitando)) = &self.busca {
            return if *digitando { format!("buscar: {termo}▏ Enter vai · Esc desiste") } else { format!("“{termo}” · Ctrl+G próximo · Esc limpa") };
        }
        let (l, c) = self.linha_coluna();
        let lingua = self.lingua.as_deref().map(|l| LINGUAGENS.iter().find(|(k, _)| k.eq_ignore_ascii_case(l)).map_or(l, |(_, n)| n)).unwrap_or("sem linguagem");
        format!("linha {}, coluna {} · {lingua}", l + 1, c + 1)
    }

    /// Desenha o editor em `area`: número, código com cor, a linha do
    /// cursor marcada e o cursor em vídeo inverso. A vista anda o mínimo pra
    /// o cursor ficar à vista.
    pub fn desenhar(&self, f: &mut Frame, area: Rect, tema: &Tema) {
        if area.height == 0 || area.width < 8 {
            return;
        }
        let linhas: Vec<String> = self.campo.texto.split('\n').map(str::to_string).collect();
        let total = linhas.len();
        let (lc, cc) = self.linha_coluna();
        let alto = area.height as usize;
        self.alto.set(alto);
        let num_w = total.to_string().len().max(3);
        let largo = (area.width as usize).saturating_sub(num_w + 2).max(1);

        // Rolagem vertical e horizontal mínima.
        let mut topo = self.topo.get().min(total.saturating_sub(1));
        if lc < topo {
            topo = lc;
        } else if lc >= topo + alto {
            topo = lc + 1 - alto;
        }
        self.topo.set(topo);
        let col_cursor: usize = linhas[lc].chars().take(cc).map(|c| c.width().unwrap_or(0)).sum();
        let mut esq = self.esquerda.get();
        if col_cursor < esq {
            esq = col_cursor.saturating_sub(largo / 4);
        } else if col_cursor >= esq + largo {
            esq = col_cursor + 1 - largo + largo / 4;
        }
        self.esquerda.set(esq);

        let realcadas = self.lingua.as_deref().and_then(|l| realce_codigo::realcar(&linhas, l));
        let numero = Style::default().fg(tema.var("text-muted"));
        let numero_atual = Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD);
        let fundo_atual = tema.var("bg-elevated");
        let invertido = Style::default().fg(tema.var("bg-base")).bg(tema.var("text-primary"));
        let papel = |c: Categoria| -> Style {
            tema.estilo(match c {
                Categoria::Texto => Realce::CodigoTexto,
                Categoria::Palavra => Realce::CodigoPalavra,
                Categoria::Cadeia => Realce::CodigoCadeia,
                Categoria::Comentario => Realce::CodigoComentario,
                Categoria::Numero => Realce::CodigoNumero,
                Categoria::Tipo => Realce::CodigoTipo,
            })
        };
        let achado = tema.estilo(Realce::Barra);
        let termo: Vec<char> = self.busca.as_ref().map(|(t, _)| t.to_lowercase().chars().collect()).unwrap_or_default();
        let mut fora: Vec<Line<'static>> = Vec::new();
        for (n, l) in linhas.iter().enumerate().skip(topo).take(alto) {
            // Onde o termo da busca aparece nesta linha, em caracteres.
            let baixo: Vec<char> = l.to_lowercase().chars().collect();
            let mut marcados = vec![false; baixo.len()];
            if !termo.is_empty() && baixo.len() == l.chars().count() {
                let mut i = 0;
                while i + termo.len() <= baixo.len() {
                    if baixo[i..i + termo.len()] == termo[..] {
                        marcados[i..i + termo.len()].iter_mut().for_each(|m| *m = true);
                        i += termo.len();
                    } else {
                        i += 1;
                    }
                }
            }
            let atual = n == lc;
            let mut spans = vec![Span::styled(format!("{:>num_w$} ", n + 1), if atual { numero_atual } else { numero }), Span::raw(" ")];
            // Os caracteres com o estilo de cada um.
            let estilos: Vec<(char, Style)> = match realcadas.as_ref().and_then(|r| r.get(n)) {
                Some(trechos) => trechos.iter().flat_map(|(c, t)| t.chars().map(move |ch| (ch, papel(*c)))).collect(),
                None => l.chars().map(|ch| (ch, tema.estilo(Realce::Codigo))).collect(),
            };
            let mut col = 0;
            let mut usados = 0;
            for (k, (ch, mut estilo)) in estilos.iter().copied().chain(std::iter::once((' ', Style::default()))).enumerate() {
                let w = ch.width().unwrap_or(0).max(1);
                let e_cursor = atual && k == cc;
                if k == estilos.len() && !e_cursor {
                    break;
                }
                if col >= esq && col + w <= esq + largo {
                    if atual {
                        estilo = estilo.bg(fundo_atual);
                    }
                    if marcados.get(k).copied().unwrap_or(false) {
                        estilo = estilo.patch(achado);
                    }
                    if e_cursor {
                        estilo = invertido;
                    }
                    spans.push(Span::styled(ch.to_string(), estilo));
                    usados += w;
                }
                col += w;
            }
            if atual && usados < largo {
                spans.push(Span::styled(" ".repeat(largo - usados), Style::default().bg(fundo_atual)));
            }
            fora.push(Line::from(spans));
        }
        f.render_widget(Paragraph::new(fora), area);

        // O seletor de linguagem, por cima.
        if let Some(i) = self.seletor {
            let alto = (LINGUAGENS.len() as u16 + 2).min(area.height);
            let caixa = componentes::area_do_modal(area, 36, alto);
            let dentro = componentes::desenhar_modal(f, caixa, "Linguagem", "j k · letra · Enter · Esc", tema);
            let visiveis = dentro.height as usize;
            let primeira = i.saturating_sub(visiveis.saturating_sub(1));
            let itens: Vec<Line<'static>> = LINGUAGENS
                .iter()
                .enumerate()
                .skip(primeira)
                .take(visiveis)
                .map(|(k, (_, nome))| {
                    let estilo = if k == i { tema.estilo(Realce::Cursor) } else { Style::default().fg(tema.var("text-primary")) };
                    Line::from(Span::styled(format!(" {nome} "), estilo))
                })
                .collect();
            f.render_widget(Paragraph::new(itens), dentro);
        }
    }
}

/// A cerca com a linguagem trocada, guardando os acentos graves (ou os
/// tis) e quantos eram: "````rust" com 4 continua com 4.
pub fn cerca_com(cerca: &str, lingua: Option<&str>) -> String {
    let marca: String = cerca.chars().take_while(|c| *c == '`' || *c == '~').collect();
    let marca = if marca.is_empty() { "```".to_string() } else { marca };
    format!("{marca}{}", lingua.unwrap_or(""))
}

/// A linguagem de uma cerca (` ```rust {x} ` → `rust`).
pub fn lingua_da_cerca(cerca: &str) -> Option<String> {
    let resto = cerca.trim_start_matches(['`', '~']).trim();
    resto.split_whitespace().next().map(str::to_string).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod testes {
    use super::*;

    fn ed(texto: &str, cursor: usize) -> EditorDeCodigo {
        let mut e = EditorDeCodigo::novo(texto, Some("rust".into()));
        e.campo.cursor = cursor;
        e
    }

    fn digitar(e: &mut EditorDeCodigo, s: &str) {
        for ch in s.chars() {
            e.tecla(&ch.to_string());
        }
    }

    #[test]
    fn enter_mantem_o_recuo_e_entra_um_nivel_depois_da_chave() {
        let mut e = ed("fn f() {", 8);
        e.tecla("Enter");
        assert_eq!(e.texto(), "fn f() {\n    ");
        digitar(&mut e, "if x {");
        e.tecla("Enter");
        assert_eq!(e.texto(), "fn f() {\n    if x {\n        ");
        // `}` numa linha só de espaços volta um nível.
        digitar(&mut e, "}");
        assert_eq!(e.texto(), "fn f() {\n    if x {\n    }");
        // Linha comum: só mantém.
        let mut e = ed("    a = 1", 9);
        e.tecla("Enter");
        assert_eq!(e.texto(), "    a = 1\n    ");
    }

    #[test]
    fn enter_entre_chaves_desce_a_que_fecha() {
        let mut e = ed("x = {}", 5);
        e.tecla("Enter");
        assert_eq!(e.texto(), "x = {\n    \n}");
        assert_eq!(e.linha_coluna(), (1, 4));
    }

    #[test]
    fn tab_shift_tab_e_backspace_andam_de_nivel_em_nivel() {
        let mut e = ed("ab", 0);
        e.tecla("Tab");
        assert_eq!(e.texto(), "    ab");
        e.campo.cursor = 6;
        e.tecla("Tab");
        assert_eq!(e.texto(), "    ab  ", "Tab vai até o próximo múltiplo de 4");
        e.tecla("Shift+Tab");
        assert_eq!(e.texto(), "ab  ");
        // Backspace no recuo apaga o nível inteiro.
        let mut e = ed("        x", 8);
        e.tecla("Backspace");
        assert_eq!(e.texto(), "    x");
        // Fora do recuo, um caractere.
        let mut e = ed("    xy", 6);
        e.tecla("Backspace");
        assert_eq!(e.texto(), "    x");
    }

    #[test]
    fn home_vai_ao_primeiro_caractere_e_depois_a_coluna_zero() {
        let mut e = ed("a\n    foo", 9);
        e.tecla("Home");
        assert_eq!(e.linha_coluna(), (1, 4));
        e.tecla("Home");
        assert_eq!(e.linha_coluna(), (1, 0));
        e.tecla("Ctrl+Home");
        assert_eq!(e.campo.cursor, 0);
        e.tecla("Ctrl+End");
        assert_eq!(e.campo.cursor, 9);
    }

    #[test]
    fn desfazer_junta_as_letras_e_refazer_volta() {
        let mut e = ed("", 0);
        digitar(&mut e, "let x");
        e.tecla("Enter");
        digitar(&mut e, "y");
        assert_eq!(e.texto(), "let x\ny");
        e.tecla("Ctrl+z");
        assert_eq!(e.texto(), "let x\n");
        e.tecla("Ctrl+z");
        assert_eq!(e.texto(), "let x");
        e.tecla("Ctrl+z");
        assert_eq!(e.texto(), "let ", "o x depois do espaço é passo novo");
        e.tecla("Ctrl+z");
        assert_eq!(e.texto(), "let", "o espaço também");
        e.tecla("Ctrl+y");
        e.tecla("Ctrl+y");
        assert_eq!(e.texto(), "let x");
    }

    #[test]
    fn ctrl_f_procura_e_ctrl_g_vai_ao_proximo() {
        let mut e = ed("um x\ndois x\ntrês X", 0);
        e.tecla("Ctrl+f");
        digitar(&mut e, "x");
        assert!(e.prende("Escape"));
        e.tecla("Enter");
        assert_eq!(e.linha_coluna(), (0, 3));
        e.tecla("Ctrl+g");
        assert_eq!(e.linha_coluna(), (1, 5));
        e.tecla("Ctrl+g");
        assert_eq!(e.linha_coluna(), (2, 5), "sem diferença de maiúscula");
        e.tecla("Ctrl+g");
        assert_eq!(e.linha_coluna(), (0, 3), "dá a volta");
        // Digitar continua editando; o primeiro Esc só limpa a busca.
        digitar(&mut e, "y");
        assert_eq!(e.texto(), "um yx\ndois x\ntrês X");
        assert!(e.prende("Escape") && !e.prende("Ctrl+s"));
        e.tecla("Escape");
        assert!(e.busca.is_none() && !e.prende("Escape"));
    }

    #[test]
    fn colar_entra_como_veio_sem_somar_recuo() {
        let mut e = ed("fn f() {\n    ", 13);
        e.colar("if x {\r\n    y();\r\n}");
        assert_eq!(e.texto(), "fn f() {\n    if x {\n    y();\n}");
        // Um passo só no desfazer.
        e.tecla("Ctrl+z");
        assert_eq!(e.texto(), "fn f() {\n    ");
    }

    #[test]
    fn ctrl_k_apaga_a_linha() {
        let mut e = ed("a\nb\nc", 2);
        e.tecla("Ctrl+k");
        assert_eq!(e.texto(), "a\nc");
        let mut e = ed("a\nb", 3);
        e.tecla("Ctrl+k");
        assert_eq!(e.texto(), "a");
    }

    #[test]
    fn seletor_troca_a_linguagem_e_a_cerca_guarda_a_marca() {
        let mut e = ed("x", 0);
        e.tecla("Ctrl+l");
        assert_eq!(e.seletor, Some(2), "abre na linguagem atual (rust)");
        e.tecla("p"); // Python
        assert_eq!(e.tecla("Enter"), Resposta::Linguagem);
        assert_eq!(e.lingua.as_deref(), Some("python"));
        assert_eq!(cerca_com("````rust", e.lingua.as_deref()), "````python");
        assert_eq!(cerca_com("```", None), "```");
        assert_eq!(lingua_da_cerca("```rust {.x}"), Some("rust".into()));
        assert_eq!(lingua_da_cerca("```"), None);
        // "sem linguagem" tira.
        e.tecla("Ctrl+l");
        e.tecla("g");
        e.tecla("Enter");
        assert_eq!(e.lingua, None);
    }

    #[test]
    fn a_vista_segue_o_cursor_nas_duas_direcoes() {
        use ratatui::backend::TestBackend;
        use ratatui::Terminal;
        let longa = format!("{}FIM", "a".repeat(100));
        let texto: String = (0..30).map(|i| format!("linha {i}\n")).collect::<String>() + &longa;
        let mut e = EditorDeCodigo::novo(texto.clone(), None);
        e.campo.cursor = texto.chars().count();
        let mut term = Terminal::new(TestBackend::new(40, 10)).unwrap();
        term.draw(|f| e.desenhar(f, f.area(), &Tema::novo("mocha"))).unwrap();
        let buf = term.backend().buffer().clone();
        let tela: Vec<String> = (0..10).map(|y| (0..40).map(|x| buf[(x, y)].symbol().to_string()).collect()).collect();
        let tudo = tela.join("\n");
        assert!(tudo.contains("FIM"), "o cursor no fim da linha longa não ficou à vista:\n{tudo}");
        assert!(tudo.contains(" 31 "), "a última linha não ficou à vista:\n{tudo}");
        assert!(!tudo.contains("linha 0"), "{tudo}");
    }
}
