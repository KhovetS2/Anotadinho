//! Componentes da TUI (ciclo 339): as peças de que as telas complexas são
//! feitas, no mesmo espírito dos componentes da janela.
//!
//! - [`Campo`]: um campo de texto de uma linha, com cursor — o
//!   `<input>` da janela.
//! - [`Lista`]: uma lista com filtro digitado e seleção — o miolo da paleta
//!   de comandos, dos seletores e dos menus.
//! - [`desenhar_modal`]: a caixa por cima da tela, com título e fundo, como
//!   o `.modal`/`.command-palette` da janela.
//! - [`desenhar_lista`]: a lista dentro de um modal.
//!
//! Nada aqui conhece página, embed ou vault: é só teclado e desenho. Quem
//! monta uma paleta decide o que os itens significam pela `chave`.

use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::tema::Tema;

/// Um campo de texto de uma linha, com cursor em caracteres.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Campo {
    /// O que está escrito.
    pub texto: String,
    /// Onde está o cursor, em caracteres.
    pub cursor: usize,
}

impl Campo {
    /// Um campo com texto e o cursor no fim.
    pub fn com(texto: impl Into<String>) -> Self {
        let texto = texto.into();
        let cursor = texto.chars().count();
        Self { texto, cursor }
    }

    fn byte(&self, c: usize) -> usize {
        self.texto.char_indices().nth(c).map_or(self.texto.len(), |(i, _)| i)
    }

    /// Aplica uma tecla de edição. Devolve se ela foi usada — `Enter`,
    /// `Escape` e setas verticais são de quem contém o campo.
    pub fn tecla(&mut self, tecla: &str) -> bool {
        let n = self.texto.chars().count();
        self.cursor = self.cursor.min(n);
        match tecla {
            "Backspace" => {
                if self.cursor > 0 {
                    let i = self.byte(self.cursor - 1);
                    self.texto.remove(i);
                    self.cursor -= 1;
                }
            }
            "Delete" => {
                if self.cursor < n {
                    let i = self.byte(self.cursor);
                    self.texto.remove(i);
                }
            }
            "ArrowLeft" => self.cursor = self.cursor.saturating_sub(1),
            "ArrowRight" => self.cursor = (self.cursor + 1).min(n),
            "Home" | "Ctrl+a" => self.cursor = 0,
            "End" | "Ctrl+e" => self.cursor = n,
            "Ctrl+w" => {
                let antes: Vec<char> = self.texto.chars().take(self.cursor).collect();
                let mut k = antes.len();
                while k > 0 && antes[k - 1] == ' ' {
                    k -= 1;
                }
                while k > 0 && antes[k - 1] != ' ' {
                    k -= 1;
                }
                let resto: String = self.texto.chars().skip(self.cursor).collect();
                self.texto = format!("{}{resto}", antes[..k].iter().collect::<String>());
                self.cursor = k;
            }
            "Ctrl+u" => {
                self.texto = self.texto.chars().skip(self.cursor).collect();
                self.cursor = 0;
            }
            t if t.chars().count() == 1 => {
                let i = self.byte(self.cursor);
                self.texto.insert_str(i, t);
                self.cursor += 1;
            }
            _ => return false,
        }
        true
    }

    /// O texto com o cursor em vídeo inverso; vazio, o `placeholder`
    /// apagado.
    pub fn spans(&self, estilo: Style, placeholder: &str, tema: &Tema) -> Vec<Span<'static>> {
        let invertido = Style::default().fg(tema.var("bg-base")).bg(tema.var("text-primary"));
        if self.texto.is_empty() {
            let mut v = vec![Span::styled(" ", invertido)];
            if !placeholder.is_empty() {
                v.push(Span::styled(placeholder.to_string(), Style::default().fg(tema.var("text-muted"))));
            }
            return v;
        }
        let chars: Vec<char> = self.texto.chars().collect();
        let c = self.cursor.min(chars.len());
        let antes: String = chars[..c].iter().collect();
        let sob: String = chars.get(c).map(|x| x.to_string()).unwrap_or_else(|| " ".into());
        let depois: String = chars.get(c + 1..).map(|r| r.iter().collect()).unwrap_or_default();
        vec![Span::styled(antes, estilo), Span::styled(sob, invertido), Span::styled(depois, estilo)]
    }
}

/// Um item de [`Lista`].
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// O que se lê.
    pub rotulo: String,
    /// Texto apagado à direita (o caminho, o valor atual).
    pub detalhe: String,
    /// Um glifo de uma célula na frente.
    pub icone: &'static str,
    /// O que o item significa pra quem montou a lista.
    pub chave: String,
}

impl Item {
    /// Um item com rótulo e chave.
    pub fn novo(icone: &'static str, rotulo: impl Into<String>, chave: impl Into<String>) -> Self {
        Self { rotulo: rotulo.into(), detalhe: String::new(), icone, chave: chave.into() }
    }

    /// Com o detalhe à direita.
    pub fn com_detalhe(mut self, detalhe: impl Into<String>) -> Self {
        self.detalhe = detalhe.into();
        self
    }
}

/// O que a lista fez com a tecla.
#[derive(Debug, Clone, PartialEq)]
pub enum Resposta {
    /// Nada que interesse a quem contém a lista.
    Nada,
    /// `Enter` num item: a chave dele.
    Escolhido(String),
    /// `Escape`: fechar.
    Fechar,
}

/// Uma lista com filtro e seleção.
#[derive(Debug, Clone, PartialEq)]
pub struct Lista {
    /// Todos os itens, na ordem de quem montou.
    pub itens: Vec<Item>,
    /// O filtro digitado (sem filtro, a lista é um menu: `j`/`k` andam).
    pub filtro: Option<Campo>,
    /// A posição selecionada, entre os VISÍVEIS.
    pub selecionado: usize,
}

impl Lista {
    /// Uma lista com campo de filtro — a paleta.
    pub fn filtravel(itens: Vec<Item>) -> Self {
        Self { itens, filtro: Some(Campo::default()), selecionado: 0 }
    }

    /// Uma lista sem filtro — o menu.
    pub fn menu(itens: Vec<Item>) -> Self {
        Self { itens, filtro: None, selecionado: 0 }
    }

    /// Os itens que passam no filtro, com o índice original, do mais
    /// parecido pro menos: começo do rótulo, depois palavra que começa com
    /// o termo, depois contém, depois as letras em ordem.
    pub fn visiveis(&self) -> Vec<(usize, &Item)> {
        let termo = self.filtro.as_ref().map(|c| c.texto.trim().to_lowercase()).unwrap_or_default();
        if termo.is_empty() {
            return self.itens.iter().enumerate().collect();
        }
        let mut pontuados: Vec<(u8, usize, &Item)> = self
            .itens
            .iter()
            .enumerate()
            .filter_map(|(i, it)| pontuar(&termo, &it.rotulo, &it.detalhe).map(|p| (p, i, it)))
            .collect();
        pontuados.sort_by_key(|(p, i, _)| (*p, *i));
        pontuados.into_iter().map(|(_, i, it)| (i, it)).collect()
    }

    /// O item selecionado, se há algum visível.
    pub fn atual(&self) -> Option<&Item> {
        self.visiveis().get(self.selecionado).map(|(_, it)| *it)
    }

    /// Aplica uma tecla.
    pub fn tecla(&mut self, tecla: &str) -> Resposta {
        let total = self.visiveis().len();
        let com_filtro = self.filtro.is_some();
        match tecla {
            "Escape" => return Resposta::Fechar,
            "Enter" => {
                return self.atual().map(|it| Resposta::Escolhido(it.chave.clone())).unwrap_or(Resposta::Nada);
            }
            "ArrowDown" | "Ctrl+n" | "Tab" => self.selecionado = (self.selecionado + 1).min(total.saturating_sub(1)),
            "ArrowUp" | "Ctrl+p" => self.selecionado = self.selecionado.saturating_sub(1),
            "j" if !com_filtro => self.selecionado = (self.selecionado + 1).min(total.saturating_sub(1)),
            "k" if !com_filtro => self.selecionado = self.selecionado.saturating_sub(1),
            "PageDown" => self.selecionado = (self.selecionado + 10).min(total.saturating_sub(1)),
            "PageUp" => self.selecionado = self.selecionado.saturating_sub(10),
            outra => {
                if let Some(c) = self.filtro.as_mut() {
                    if c.tecla(outra) {
                        self.selecionado = 0;
                    }
                }
            }
        }
        Resposta::Nada
    }
}

/// Quão bem `termo` casa com o item (menor é melhor); `None` não casa.
fn pontuar(termo: &str, rotulo: &str, detalhe: &str) -> Option<u8> {
    // "pagina" acha "página": acento não é o que a pessoa digita.
    let termo = &sem_acento(termo);
    let r = sem_acento(&rotulo.to_lowercase());
    let detalhe = &sem_acento(detalhe);
    if r.starts_with(termo) {
        return Some(0);
    }
    if r.split(|c: char| !c.is_alphanumeric()).any(|p| p.starts_with(termo)) {
        return Some(1);
    }
    if r.contains(termo) {
        return Some(2);
    }
    if detalhe.to_lowercase().contains(termo) {
        return Some(3);
    }
    let mut letras = r.chars();
    termo.chars().filter(|c| !c.is_whitespace()).all(|c| letras.any(|x| x == c)).then_some(4)
}

fn sem_acento(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
            'é' | 'è' | 'ê' | 'ë' => 'e',
            'í' | 'ì' | 'î' | 'ï' => 'i',
            'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
            'ú' | 'ù' | 'û' | 'ü' => 'u',
            'ç' => 'c',
            outro => outro,
        })
        .collect()
}

/// A área de um modal centrado: `largura` colunas (no máximo a tela
/// menos margem) e `altura` linhas, no terço de cima como a paleta da
/// janela.
pub fn area_do_modal(tela: Rect, largura: u16, altura: u16) -> Rect {
    let w = largura.min(tela.width.saturating_sub(4)).max(20.min(tela.width));
    let h = altura.min(tela.height.saturating_sub(2)).max(3.min(tela.height));
    let x = tela.x + (tela.width - w) / 2;
    let topo_livre = tela.height.saturating_sub(h);
    let y = tela.y + (topo_livre / 5).min(topo_livre);
    Rect::new(x, y, w, h)
}

/// A caixa do modal: limpa o que está embaixo, pinta o fundo elevado e a
/// borda arredondada, põe o título. Devolve a área de dentro.
pub fn desenhar_modal(f: &mut Frame, area: Rect, titulo: &str, rodape: &str, tema: &Tema) -> Rect {
    f.render_widget(Clear, area);
    let mut bloco = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(tema.var("border")).bg(tema.var("bg-surface")))
        .style(Style::default().bg(tema.var("bg-surface")).fg(tema.var("text-primary")));
    if !titulo.is_empty() {
        bloco = bloco.title(Span::styled(
            format!(" {titulo} "),
            Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD),
        ));
    }
    if !rodape.is_empty() {
        bloco = bloco.title_bottom(Line::from(Span::styled(
            format!(" {rodape} "),
            Style::default().fg(tema.var("text-muted")),
        )));
    }
    let dentro = bloco.inner(area);
    f.render_widget(bloco, area);
    dentro
}

/// A lista dentro de um modal: o campo de filtro em cima (quando há), a
/// régua, e os itens com o selecionado aceso, rolando pra mantê-lo à
/// vista.
pub fn desenhar_lista(f: &mut Frame, area: Rect, lista: &Lista, placeholder: &str, tema: &Tema) {
    let mut linhas: Vec<Line<'static>> = Vec::new();
    let largura = area.width as usize;
    if let Some(c) = &lista.filtro {
        let mut spans = vec![Span::raw(" ")];
        spans.extend(c.spans(Style::default().fg(tema.var("text-primary")), placeholder, tema));
        linhas.push(Line::from(spans));
        linhas.push(Line::from(Span::styled("─".repeat(largura), Style::default().fg(tema.var("border")))));
    }
    let visiveis = lista.visiveis();
    let cabe = (area.height as usize).saturating_sub(linhas.len()).max(1);
    let inicio = lista.selecionado.saturating_sub(cabe.saturating_sub(1));
    if visiveis.is_empty() {
        linhas.push(Line::from(Span::styled(" nada encontrado", Style::default().fg(tema.var("text-muted")))));
    }
    for (pos, (_, it)) in visiveis.iter().enumerate().skip(inicio).take(cabe) {
        let aceso = pos == lista.selecionado;
        let fundo = if aceso { Style::default().bg(tema.var("bg-elevated")) } else { Style::default() };
        let rotulo_estilo = fundo.fg(tema.var("text-primary"));
        let detalhe_estilo = fundo.fg(tema.var("text-muted"));
        let icone = format!(" {} ", it.icone);
        let usado = icone.chars().count() + it.rotulo.chars().count() + 1;
        let espaco = largura.saturating_sub(usado + it.detalhe.chars().count() + 1);
        let detalhe: String = if espaco == 0 {
            String::new()
        } else {
            it.detalhe.clone()
        };
        let mut spans = vec![
            Span::styled(icone, fundo.fg(if aceso { tema.var("accent-blue") } else { tema.var("text-muted") })),
            Span::styled(format!("{} ", cortar(&it.rotulo, largura.saturating_sub(4))), rotulo_estilo),
        ];
        let ocupado: usize = spans.iter().map(|s| s.content.chars().count()).sum();
        let resto = largura.saturating_sub(ocupado + detalhe.chars().count() + 1);
        spans.push(Span::styled(" ".repeat(resto), fundo));
        spans.push(Span::styled(detalhe, detalhe_estilo));
        spans.push(Span::styled(" ", fundo));
        linhas.push(Line::from(spans));
    }
    f.render_widget(Paragraph::new(linhas), area);
}

fn cortar(texto: &str, largura: usize) -> String {
    if texto.chars().count() <= largura {
        return texto.to_string();
    }
    let mut s: String = texto.chars().take(largura.saturating_sub(1)).collect();
    s.push('…');
    s
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_campo_edita_no_cursor() {
        let mut c = Campo::com("abc");
        c.tecla("ArrowLeft");
        c.tecla("X");
        assert_eq!((c.texto.as_str(), c.cursor), ("abXc", 3));
        c.tecla("Home");
        c.tecla("Delete");
        assert_eq!(c.texto, "bXc");
        c.tecla("End");
        c.tecla(" ");
        c.tecla("d");
        c.tecla("Ctrl+w");
        assert_eq!(c.texto, "bXc ");
    }

    #[test]
    fn a_lista_filtra_pelo_mais_parecido_e_escolhe() {
        let mut l = Lista::filtravel(vec![
            Item::novo("ϟ", "Alternar tema", "tema"),
            Item::novo("ϟ", "Nova página", "nova"),
            Item::novo("≡", "tarefas", "p1").com_detalhe("pages/tarefas.md"),
        ]);
        for t in ["t", "e", "m"] {
            l.tecla(t);
        }
        assert_eq!(l.atual().unwrap().chave, "tema");
        l.tecla("Ctrl+u");
        for t in ["p", "a", "g"] {
            l.tecla(t);
        }
        assert_eq!(l.visiveis().iter().map(|(_, i)| i.chave.as_str()).collect::<Vec<_>>(), ["nova", "p1"]);
        l.tecla("ArrowDown");
        assert_eq!(l.tecla("Enter"), Resposta::Escolhido("p1".into()));
        assert_eq!(l.tecla("Escape"), Resposta::Fechar);
    }

    #[test]
    fn o_menu_anda_com_j_e_k() {
        let mut l = Lista::menu(vec![Item::novo("·", "a", "a"), Item::novo("·", "b", "b")]);
        l.tecla("j");
        assert_eq!(l.atual().unwrap().chave, "b");
        l.tecla("k");
        assert_eq!(l.atual().unwrap().chave, "a");
    }
}
