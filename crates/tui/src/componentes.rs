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
    // Seleção fora da lista (`usize::MAX`) é "nenhum aceso": começa do topo.
    let inicio = if lista.selecionado < visiveis.len() { lista.selecionado.saturating_sub(cabe.saturating_sub(1)) } else { 0 };
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

// ---------------------------------------------------------------------
// Formulário (ciclo 343)
// ---------------------------------------------------------------------

/// O valor de um campo de [`Formulario`].
#[derive(Debug, Clone, PartialEq)]
pub enum Valor {
    /// Uma linha de texto (título, data `AAAA-MM-DD`, hora `HH:MM`).
    Texto(String),
    /// Liga/desliga ("Vários dias", "Horário específico").
    Booleano(bool),
    /// Uma lista de textos (tags, comentários, anexos).
    Lista(Vec<String>),
    /// Itens com caixa (a checklist).
    Checklist(Vec<(bool, String)>),
}

/// Um campo de [`Formulario`].
#[derive(Debug, Clone, PartialEq)]
pub struct CampoDoFormulario {
    /// Como quem montou reconhece o campo.
    pub chave: &'static str,
    /// O que se lê.
    pub rotulo: String,
    /// O valor.
    pub valor: Valor,
    /// Dica apagada quando vazio ("AAAA-MM-DD", "Nova tag").
    pub dica: String,
    /// Some da tela (ex.: "Fim" com "Vários dias" desligado).
    pub escondido: bool,
}

impl CampoDoFormulario {
    /// Um campo.
    pub fn novo(chave: &'static str, rotulo: impl Into<String>, valor: Valor) -> Self {
        Self { chave, rotulo: rotulo.into(), valor, dica: String::new(), escondido: false }
    }
    /// Com a dica de quando está vazio.
    pub fn com_dica(mut self, dica: impl Into<String>) -> Self {
        self.dica = dica.into();
        self
    }
}

/// Onde o cursor está num formulário: o campo e, numa lista, o item
/// (`itens.len()` é o "+ item").
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Posicao {
    /// O campo.
    pub campo: usize,
    /// O item, em campos de lista.
    pub item: usize,
}

/// O que o formulário fez com a tecla.
#[derive(Debug, Clone, PartialEq)]
pub enum RespostaDoFormulario {
    /// Nada mudou nos valores.
    Nada,
    /// Um valor mudou: quem montou grava.
    Mudou,
    /// `Esc` fora de edição: fechar.
    Fechar,
    /// Um botão do rodapé (a chave dele), ex. `"excluir"`.
    Botao(&'static str),
}

/// Um formulário de campos, navegado pelo teclado como o resto da TUI:
/// `j`/`k` andam (item por item dentro das listas), `Enter`/`i`/`a` editam,
/// `o` acrescenta item, `dd` apaga item, `~`/espaço alternam
/// (liga/desliga e a caixa da checklist), `Esc` fecha. Editando, `Enter` ou
/// `Esc` confirmam.
#[derive(Debug, Clone, PartialEq)]
pub struct Formulario {
    /// Os campos.
    pub campos: Vec<CampoDoFormulario>,
    /// Botões no fim (`(chave, rótulo)`), como "Excluir evento".
    pub botoes: Vec<(&'static str, String)>,
    /// O cursor.
    pub posicao: Posicao,
    /// O texto sendo editado.
    pub editando: Option<Campo>,
    /// O primeiro `d` do `dd`.
    pub d_pendente: bool,
}

impl Formulario {
    /// Um formulário.
    pub fn novo(campos: Vec<CampoDoFormulario>) -> Self {
        Self { campos, botoes: Vec::new(), posicao: Posicao::default(), editando: None, d_pendente: false }
    }

    /// O valor de um campo pela chave.
    pub fn valor(&self, chave: &str) -> Option<&Valor> {
        self.campos.iter().find(|c| c.chave == chave).map(|c| &c.valor)
    }

    /// O texto de um campo de texto.
    pub fn texto(&self, chave: &str) -> String {
        match self.valor(chave) {
            Some(Valor::Texto(t)) => t.clone(),
            _ => String::new(),
        }
    }

    /// O liga/desliga de um campo.
    pub fn booleano(&self, chave: &str) -> bool {
        matches!(self.valor(chave), Some(Valor::Booleano(true)))
    }

    /// Esconde ou mostra um campo.
    pub fn esconder(&mut self, chave: &str, escondido: bool) {
        if let Some(c) = self.campos.iter_mut().find(|c| c.chave == chave) {
            c.escondido = escondido;
        }
    }

    /// Os destinos do cursor, na ordem da tela: `(campo, item)`; o índice
    /// `campos.len()` com `item = b` é o botão `b`.
    fn paradas(&self) -> Vec<Posicao> {
        let mut v = Vec::new();
        for (i, c) in self.campos.iter().enumerate() {
            if c.escondido {
                continue;
            }
            match &c.valor {
                Valor::Lista(l) => (0..=l.len()).for_each(|k| v.push(Posicao { campo: i, item: k })),
                Valor::Checklist(l) => (0..=l.len()).for_each(|k| v.push(Posicao { campo: i, item: k })),
                _ => v.push(Posicao { campo: i, item: 0 }),
            }
        }
        for b in 0..self.botoes.len() {
            v.push(Posicao { campo: self.campos.len(), item: b });
        }
        v
    }

    fn andar(&mut self, passo: i64) {
        let paradas = self.paradas();
        let agora = paradas.iter().position(|p| *p == self.posicao).unwrap_or(0) as i64;
        let novo = (agora + passo).clamp(0, paradas.len() as i64 - 1) as usize;
        if let Some(p) = paradas.get(novo) {
            self.posicao = *p;
        }
    }

    /// Aplica uma tecla.
    pub fn tecla(&mut self, tecla: &str) -> RespostaDoFormulario {
        let p = self.posicao;
        // Editando um texto.
        if let Some(mut campo) = self.editando.take() {
            if matches!(tecla, "Enter" | "Escape") {
                let texto = campo.texto.trim().to_string();
                let Some(c) = self.campos.get_mut(p.campo) else { return RespostaDoFormulario::Nada };
                match &mut c.valor {
                    Valor::Texto(t) => *t = texto,
                    Valor::Lista(l) if texto.is_empty() => {
                        if p.item < l.len() {
                            l.remove(p.item);
                        }
                    }
                    Valor::Lista(l) => {
                        if p.item < l.len() {
                            l[p.item] = texto;
                        } else {
                            // O cursor fica no "+", pronto pro próximo.
                            l.push(texto);
                            self.posicao.item = l.len();
                        }
                    }
                    Valor::Checklist(l) if texto.is_empty() => {
                        if p.item < l.len() {
                            l.remove(p.item);
                        }
                    }
                    Valor::Checklist(l) => {
                        if p.item < l.len() {
                            l[p.item].1 = texto;
                        } else {
                            l.push((false, texto));
                            self.posicao.item = l.len();
                        }
                    }
                    Valor::Booleano(_) => {}
                }
                return RespostaDoFormulario::Mudou;
            }
            campo.tecla(tecla);
            self.editando = Some(campo);
            return RespostaDoFormulario::Nada;
        }
        let d_antes = std::mem::take(&mut self.d_pendente);
        if p.campo >= self.campos.len() {
            return match tecla {
                "j" | "ArrowDown" | "Tab" => {
                    self.andar(1);
                    RespostaDoFormulario::Nada
                }
                "k" | "ArrowUp" => {
                    self.andar(-1);
                    RespostaDoFormulario::Nada
                }
                "Enter" => self.botoes.get(p.item).map(|(c, _)| RespostaDoFormulario::Botao(c)).unwrap_or(RespostaDoFormulario::Nada),
                "Escape" | "q" => RespostaDoFormulario::Fechar,
                _ => RespostaDoFormulario::Nada,
            };
        }
        let valor = self.campos[p.campo].valor.clone();
        match (tecla, valor) {
            ("Escape" | "q", _) => return RespostaDoFormulario::Fechar,
            ("j" | "ArrowDown" | "Tab", _) => self.andar(1),
            ("k" | "ArrowUp", _) => self.andar(-1),
            ("Enter" | "i" | "a" | " " | "~", Valor::Booleano(b)) => {
                self.campos[p.campo].valor = Valor::Booleano(!b);
                return RespostaDoFormulario::Mudou;
            }
            ("Enter" | "i" | "a" | "A", Valor::Texto(t)) => self.editando = Some(Campo::com(t)),
            ("c", Valor::Texto(_)) => self.editando = Some(Campo::default()),
            ("Enter" | "i" | "a" | "A", Valor::Lista(l)) => {
                self.editando = Some(Campo::com(l.get(p.item).cloned().unwrap_or_default()));
            }
            ("Enter" | "i" | "a" | "A", Valor::Checklist(l)) => {
                self.editando = Some(Campo::com(l.get(p.item).map(|x| x.1.clone()).unwrap_or_default()));
            }
            ("o", Valor::Lista(l)) => {
                self.posicao.item = l.len();
                self.editando = Some(Campo::default());
            }
            ("o", Valor::Checklist(l)) => {
                self.posicao.item = l.len();
                self.editando = Some(Campo::default());
            }
            (" " | "~" | "x", Valor::Checklist(mut l)) if p.item < l.len() => {
                l[p.item].0 = !l[p.item].0;
                self.campos[p.campo].valor = Valor::Checklist(l);
                return RespostaDoFormulario::Mudou;
            }
            ("d", Valor::Lista(_) | Valor::Checklist(_)) if !d_antes => self.d_pendente = true,
            ("d", Valor::Lista(mut l)) if p.item < l.len() => {
                l.remove(p.item);
                self.campos[p.campo].valor = Valor::Lista(l);
                return RespostaDoFormulario::Mudou;
            }
            ("d", Valor::Checklist(mut l)) if p.item < l.len() => {
                l.remove(p.item);
                self.campos[p.campo].valor = Valor::Checklist(l);
                return RespostaDoFormulario::Mudou;
            }
            _ => {}
        }
        RespostaDoFormulario::Nada
    }

    /// As linhas do formulário, com o cursor aceso.
    pub fn linhas(&self, largura: usize, tema: &Tema) -> Vec<Line<'static>> {
        let apagado = Style::default().fg(tema.var("text-muted"));
        let texto = Style::default().fg(tema.var("text-primary"));
        let aceso = Style::default().bg(tema.var("bg-elevated"));
        let destaque = tema.var("accent-blue");
        let mut fora: Vec<Line<'static>> = Vec::new();
        let linha = |conteudo: Vec<Span<'static>>, esta: bool| -> Line<'static> {
            if !esta {
                return Line::from(conteudo);
            }
            let usado: usize = conteudo.iter().map(|s| s.content.chars().count()).sum();
            let mut spans: Vec<Span<'static>> = conteudo
                .into_iter()
                .map(|s| {
                    let st = if s.style.bg.is_none() { s.style.bg(tema.var("bg-elevated")) } else { s.style };
                    Span::styled(s.content, st)
                })
                .collect();
            spans.push(Span::styled(" ".repeat(largura.saturating_sub(usado)), aceso));
            Line::from(spans)
        };
        let valor_editado = |p: Posicao| -> Option<Vec<Span<'static>>> {
            (self.posicao == p).then_some(())?;
            self.editando.as_ref().map(|c| c.spans(texto, "", tema))
        };
        for (i, c) in self.campos.iter().enumerate() {
            if c.escondido {
                continue;
            }
            match &c.valor {
                Valor::Texto(t) => {
                    let p = Posicao { campo: i, item: 0 };
                    let esta = self.posicao == p;
                    let mut spans = vec![Span::styled(format!(" {:<14}", c.rotulo), if esta { apagado.fg(destaque) } else { apagado })];
                    match valor_editado(p) {
                        Some(v) => spans.extend(v),
                        None if t.is_empty() => spans.push(Span::styled(c.dica.clone(), apagado)),
                        None => spans.push(Span::styled(t.clone(), texto)),
                    }
                    fora.push(linha(spans, esta));
                }
                Valor::Booleano(b) => {
                    let p = Posicao { campo: i, item: 0 };
                    let esta = self.posicao == p;
                    fora.push(linha(
                        vec![
                            Span::styled(format!(" {} ", if *b { "▣" } else { "□" }), if *b { texto.fg(destaque) } else { apagado }),
                            Span::styled(c.rotulo.clone(), texto),
                        ],
                        esta,
                    ));
                }
                Valor::Lista(l) => {
                    fora.push(Line::from(Span::styled(format!(" {}", c.rotulo), apagado)));
                    for (k, item) in l.iter().enumerate() {
                        let p = Posicao { campo: i, item: k };
                        let esta = self.posicao == p;
                        let mut spans = vec![Span::styled("   • ", apagado)];
                        spans.extend(valor_editado(p).unwrap_or_else(|| vec![Span::styled(item.clone(), texto)]));
                        fora.push(linha(spans, esta));
                    }
                    let p = Posicao { campo: i, item: l.len() };
                    let esta = self.posicao == p;
                    let mut spans = vec![Span::styled("   + ", apagado)];
                    spans.extend(valor_editado(p).unwrap_or_else(|| vec![Span::styled(c.dica.clone(), apagado)]));
                    fora.push(linha(spans, esta));
                }
                Valor::Checklist(l) => {
                    let feitos = l.iter().filter(|x| x.0).count();
                    fora.push(Line::from(Span::styled(format!(" {} {feitos}/{}", c.rotulo, l.len()), apagado)));
                    for (k, (feito, item)) in l.iter().enumerate() {
                        let p = Posicao { campo: i, item: k };
                        let esta = self.posicao == p;
                        let mut spans = vec![Span::styled(
                            format!("   {} ", if *feito { "☑" } else { "☐" }),
                            if *feito { texto.fg(tema.var("success")) } else { apagado },
                        )];
                        spans.extend(valor_editado(p).unwrap_or_else(|| {
                            vec![Span::styled(
                                item.clone(),
                                if *feito { apagado.add_modifier(Modifier::CROSSED_OUT) } else { texto },
                            )]
                        }));
                        fora.push(linha(spans, esta));
                    }
                    let p = Posicao { campo: i, item: l.len() };
                    let esta = self.posicao == p;
                    let mut spans = vec![Span::styled("   + ", apagado)];
                    spans.extend(valor_editado(p).unwrap_or_else(|| vec![Span::styled(c.dica.clone(), apagado)]));
                    fora.push(linha(spans, esta));
                }
            }
        }
        if !self.botoes.is_empty() {
            fora.push(Line::from(""));
            let mut spans = vec![Span::raw(" ")];
            for (b, (_, rotulo)) in self.botoes.iter().enumerate() {
                let esta = self.posicao == Posicao { campo: self.campos.len(), item: b };
                let estilo = if esta {
                    Style::default().bg(tema.var("error")).fg(tema.var("bg-base")).add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(tema.var("error"))
                };
                spans.push(Span::styled(format!(" {rotulo} "), estilo));
                spans.push(Span::raw("  "));
            }
            fora.push(Line::from(spans));
        }
        fora
    }

    /// Em que linha da tela está o cursor (pra rolar).
    pub fn linha_do_cursor(&self) -> usize {
        let mut n = 0;
        for (i, c) in self.campos.iter().enumerate() {
            if c.escondido {
                continue;
            }
            match &c.valor {
                Valor::Lista(l) => {
                    n += 1;
                    if self.posicao.campo == i {
                        return n + self.posicao.item;
                    }
                    n += l.len() + 1;
                }
                Valor::Checklist(l) => {
                    n += 1;
                    if self.posicao.campo == i {
                        return n + self.posicao.item;
                    }
                    n += l.len() + 1;
                }
                _ => {
                    if self.posicao.campo == i {
                        return n;
                    }
                    n += 1;
                }
            }
        }
        n + 1
    }
}

#[cfg(test)]
mod testes_do_formulario {
    use super::*;

    #[test]
    fn o_formulario_edita_texto_lista_e_checklist() {
        let mut f = Formulario::novo(vec![
            CampoDoFormulario::novo("titulo", "Título", Valor::Texto("A".into())),
            CampoDoFormulario::novo("varios", "Vários dias", Valor::Booleano(false)),
            CampoDoFormulario::novo("tags", "Tags", Valor::Lista(vec!["x".into()])).com_dica("nova tag"),
            CampoDoFormulario::novo("check", "Checklist", Valor::Checklist(vec![(false, "um".into())])),
        ]);
        f.botoes.push(("excluir", "Excluir".into()));
        f.tecla("a");
        f.tecla("B");
        assert_eq!(f.tecla("Enter"), RespostaDoFormulario::Mudou);
        assert_eq!(f.texto("titulo"), "AB");
        f.tecla("j");
        assert_eq!(f.tecla(" "), RespostaDoFormulario::Mudou);
        assert!(f.booleano("varios"));
        f.tecla("j"); // tag x
        f.tecla("j"); // + tag
        f.tecla("Enter");
        f.tecla("y");
        f.tecla("Escape");
        assert_eq!(f.valor("tags"), Some(&Valor::Lista(vec!["x".into(), "y".into()])));
        f.tecla("k");
        f.tecla("k");
        f.tecla("d");
        assert_eq!(f.tecla("d"), RespostaDoFormulario::Mudou);
        assert_eq!(f.valor("tags"), Some(&Valor::Lista(vec!["y".into()])));
        f.tecla("j");
        f.tecla("j"); // checklist "um"
        assert_eq!(f.tecla("~"), RespostaDoFormulario::Mudou);
        assert_eq!(f.valor("check"), Some(&Valor::Checklist(vec![(true, "um".into())])));
        f.tecla("o");
        f.tecla("d");
        f.tecla("o");
        f.tecla("s");
        f.tecla("Enter");
        assert_eq!(f.valor("check"), Some(&Valor::Checklist(vec![(true, "um".into()), (false, "dos".into())])));
        f.tecla("j");
        f.tecla("j");
        assert_eq!(f.tecla("Enter"), RespostaDoFormulario::Botao("excluir"));
        assert_eq!(f.tecla("Escape"), RespostaDoFormulario::Fechar);
    }
}
