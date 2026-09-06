//! O estado da tela e o que cada tecla faz com ele.
//!
//! Separado do laço de eventos de propósito: laço não se testa, transição
//! de estado se testa. `main.rs` só lê tecla, chama `tecla()` e desenha.

use anotadinho_core::inline::Marca;
use anotadinho_core::navegacao::Passo;
use anotadinho_core::unidade::Tipo;
use anotadinho_core::unidade::{Caminho, Unidade};
use anotadinho_ipc::PageMeta;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tela::{self, Linha};

/// Qual painel recebe as teclas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foco {
    /// A lista de páginas.
    Paginas,
    /// O conteúdo da página aberta.
    Conteudo,
}

/// Tudo que a tela precisa saber.
pub struct Estado {
    /// As páginas do vault.
    pub paginas: Vec<PageMeta>,
    /// Qual delas está selecionada na lista.
    pub pagina: usize,
    /// A árvore da página aberta.
    pub arvore: Unidade,
    /// As linhas dela, já desenhadas.
    pub linhas: Vec<Linha>,
    /// Onde o cursor está, na árvore.
    pub cursor: Caminho,
    /// Primeira linha visível.
    pub topo: usize,
    /// Quantas linhas de conteúdo cabem — o laço atualiza a cada quadro.
    pub altura: usize,
    /// Qual painel manda.
    pub foco: Foco,
    /// Fim do programa.
    pub sair: bool,
}

impl Estado {
    /// Estado inicial, com a árvore da primeira página já aberta.
    pub fn novo(paginas: Vec<PageMeta>, arvore: Unidade) -> Self {
        let linhas = tela::linhas(&arvore);
        let cursor = tela::primeiro(&arvore).unwrap_or_default();
        Self {
            paginas,
            pagina: 0,
            arvore,
            linhas,
            cursor,
            topo: 0,
            altura: 20,
            foco: Foco::Paginas,
            sair: false,
        }
    }

    /// Troca a página aberta, recomeçando o cursor.
    pub fn abrir(&mut self, arvore: Unidade) {
        self.linhas = tela::linhas(&arvore);
        self.cursor = tela::primeiro(&arvore).unwrap_or_default();
        self.arvore = arvore;
        self.topo = 0;
    }

    /// Rola pra deixar o cursor visível — o mínimo, nunca centralizando.
    fn seguir_cursor(&mut self) {
        if let Some(l) = tela::linha_de(&self.linhas, &self.cursor) {
            self.topo = tela::rolar(self.topo, self.altura, l);
        }
    }
}

/// O que uma tecla pedida faz.
///
/// Devolve `Some(caminho)` quando a página selecionada mudou e o laço
/// precisa carregar outra árvore — carregar arquivo é I/O, e I/O não
/// entra aqui.
pub fn tecla(e: &mut Estado, tecla: &str) -> Option<String> {
    match tecla {
        "q" => {
            e.sair = true;
            None
        }
        "Tab" => {
            e.foco = match e.foco {
                Foco::Paginas => Foco::Conteudo,
                Foco::Conteudo => Foco::Paginas,
            };
            None
        }
        _ if e.foco == Foco::Paginas => tecla_nas_paginas(e, tecla),
        _ => {
            tecla_no_conteudo(e, tecla);
            None
        }
    }
}

fn tecla_nas_paginas(e: &mut Estado, tecla: &str) -> Option<String> {
    match tecla {
        // A lista de páginas NÃO circula, pelo mesmo motivo do documento
        // (ciclo 279): numa lista longa, um `j` a mais que teleporta pro
        // topo faz perder o lugar sem aviso.
        "j" | "ArrowDown" => {
            if e.pagina + 1 < e.paginas.len() {
                e.pagina += 1;
            }
            None
        }
        "k" | "ArrowUp" => {
            e.pagina = e.pagina.saturating_sub(1);
            None
        }
        "Enter" => {
            e.foco = Foco::Conteudo;
            e.paginas.get(e.pagina).map(|p| p.path.clone())
        }
        _ => None,
    }
}

fn tecla_no_conteudo(e: &mut Estado, tecla: &str) {
    // Quem decide o destino é o núcleo. Este `match` só diz qual PASSO a
    // tecla pede; a régua de "dá ou não dá" é da árvore (ciclo 281).
    let passo = match tecla {
        "j" | "ArrowDown" => Passo::Proximo,
        "k" | "ArrowUp" => Passo::Anterior,
        "Enter" | "l" | "ArrowRight" => Passo::Entrar,
        "Escape" | "h" | "ArrowLeft" => Passo::Sair,
        _ => return,
    };
    e.cursor = tela::andar(&e.arvore, &e.cursor, passo);
    e.seguir_cursor();
}

/// Desenha o quadro inteiro.
pub fn desenhar(f: &mut Frame, e: &mut Estado) {
    let colunas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    // A altura útil exclui a borda de cima e a de baixo.
    e.altura = colunas[1].height.saturating_sub(2) as usize;
    e.seguir_cursor();

    let paginas: Vec<Line> = e
        .paginas
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let estilo = if i == e.pagina {
                realce(e.foco == Foco::Paginas)
            } else {
                Style::default()
            };
            Line::from(Span::styled(p.title.clone(), estilo))
        })
        .collect();
    f.render_widget(
        Paragraph::new(paginas).block(borda("páginas", e.foco == Foco::Paginas)),
        colunas[0],
    );

    let visiveis: Vec<Line> = e
        .linhas
        .iter()
        .skip(e.topo)
        .take(e.altura)
        .map(|l| linha_estilizada(l, l.caminho == e.cursor && e.foco == Foco::Conteudo))
        .collect();
    let titulo = e
        .paginas
        .get(e.pagina)
        .map(|p| p.title.clone())
        .unwrap_or_default();
    f.render_widget(
        Paragraph::new(visiveis).block(borda(&titulo, e.foco == Foco::Conteudo)),
        colunas[1],
    );
}

/// Uma linha do conteúdo, com o estilo do BLOCO e o dos trechos.
///
/// Dois níveis, e eles se somam: o bloco dá a cor de fundo do papel
/// (título forte e colorido, citação apagada, código em outra cor), e o
/// trecho dá o negrito/itálico/código de dentro (ciclo 287).
///
/// Sob o cursor, tudo cede: a linha inteira recebe o realce, porque
/// duas ênfases competindo fazem a pessoa não saber onde está.
fn linha_estilizada<'a>(l: &'a crate::tela::Linha, sob_cursor: bool) -> Line<'a> {
    let recuo = "  ".repeat(l.nivel);
    if sob_cursor {
        let plano = format!("{recuo}{} {}", l.marca, l.texto);
        return Line::from(Span::styled(plano.trim_end().to_string(), realce(true)));
    }

    let base = estilo_do_bloco(&l.tipo);
    let mut spans = vec![Span::styled(recuo, Style::default())];
    if !l.marca.trim().is_empty() {
        // A marca costuma ser estrutura e fica apagada pra não competir
        // com o conteúdo — o `##` de um título, o `-` de um item.
        //
        // Menos quando ela É o conteúdo: um embed, uma régua, um grupo
        // de lista não têm texto próprio, e apagar a marca deles apaga a
        // linha inteira. Visto no painel de tmux: `[fluxo]` saía em
        // cinza-escuro, como se fosse enfeite.
        let estilo_marca = if l.texto.trim().is_empty() {
            base
        } else {
            Style::default().fg(Color::DarkGray)
        };
        spans.push(Span::styled(format!("{} ", l.marca), estilo_marca));
    }
    if l.trechos.is_empty() {
        spans.push(Span::styled(l.texto.clone(), base));
        return Line::from(spans);
    }
    for t in &l.trechos {
        spans.push(Span::styled(t.texto.clone(), estilo_do_trecho(&t.marcas, base)));
    }
    Line::from(spans)
}

/// O estilo que o TIPO do bloco dá à linha inteira.
fn estilo_do_bloco(tipo: &Tipo) -> Style {
    match tipo {
        // Título: quanto mais alto, mais forte. O h1 é o nome da página.
        Tipo::Titulo(1) => Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        Tipo::Titulo(_) => Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD),
        Tipo::Citacao => Style::default().fg(Color::Gray).add_modifier(Modifier::ITALIC),
        Tipo::Codigo(_) => Style::default().fg(Color::LightGreen),
        Tipo::Embed(_) => Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD),
        Tipo::Parte { .. } => Style::default().fg(Color::Yellow),
        _ => Style::default(),
    }
}

/// O estilo que as marcas de um trecho somam ao do bloco.
fn estilo_do_trecho(marcas: &[Marca], base: Style) -> Style {
    marcas.iter().fold(base, |e, m| match m {
        Marca::Negrito => e.add_modifier(Modifier::BOLD),
        Marca::Italico => e.add_modifier(Modifier::ITALIC),
        Marca::Tachado => e.add_modifier(Modifier::CROSSED_OUT),
        Marca::Codigo => e.fg(Color::LightGreen),
        Marca::Link => e.fg(Color::Blue).add_modifier(Modifier::UNDERLINED),
        Marca::Wikilink => e.fg(Color::Cyan).add_modifier(Modifier::UNDERLINED),
    })
}

/// Realce do item sob o cursor. Fora do painel com foco ele fica
/// apagado: sem isso a tela mostra dois cursores e nenhum dos dois
/// parece o de verdade.
fn realce(com_foco: bool) -> Style {
    if com_foco {
        Style::default().fg(Color::Black).bg(Color::Cyan)
    } else {
        Style::default().add_modifier(Modifier::DIM)
    }
}

fn borda(titulo: &str, com_foco: bool) -> Block<'_> {
    let b = Block::default().borders(Borders::ALL).title(titulo.to_string());
    if com_foco {
        b.border_style(Style::default().fg(Color::Cyan))
    } else {
        b
    }
}

#[cfg(test)]
mod testes {
    use super::*;
    use anotadinho_core::analise::analisar;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    const PAGINA: &str = "# Título\n\nUm parágrafo.\n\n- um\n- dois\n\nFim.\n";

    fn paginas() -> Vec<PageMeta> {
        ["alfa", "beta", "gama"]
            .iter()
            .map(|t| PageMeta {
                path: format!("pages/{t}.md"),
                title: t.to_string(),
                section: "pages".into(),
            })
            .collect()
    }

    fn estado() -> Estado {
        Estado::novo(paginas(), analisar(PAGINA))
    }

    #[test]
    fn tab_alterna_o_painel_e_q_encerra() {
        let mut e = estado();
        assert_eq!(e.foco, Foco::Paginas);
        tecla(&mut e, "Tab");
        assert_eq!(e.foco, Foco::Conteudo);
        tecla(&mut e, "Tab");
        assert_eq!(e.foco, Foco::Paginas);
        assert!(!e.sair);
        tecla(&mut e, "q");
        assert!(e.sair);
    }

    #[test]
    fn a_lista_de_paginas_nao_circula() {
        // Mesma regra do documento (ciclo 279): numa lista longa, um `j`
        // a mais que teleporta pro topo faz perder o lugar sem aviso.
        let mut e = estado();
        for _ in 0..10 {
            tecla(&mut e, "j");
        }
        assert_eq!(e.pagina, 2, "passou do fim");
        for _ in 0..10 {
            tecla(&mut e, "k");
        }
        assert_eq!(e.pagina, 0, "passou do começo");
    }

    #[test]
    fn enter_numa_pagina_pede_o_arquivo_e_muda_o_foco() {
        // Ler arquivo é I/O e não acontece aqui: a transição devolve o
        // caminho e o laço resolve. É o que deixa este teste existir.
        let mut e = estado();
        tecla(&mut e, "j");
        let pedido = tecla(&mut e, "Enter");
        assert_eq!(pedido.as_deref(), Some("pages/beta.md"));
        assert_eq!(e.foco, Foco::Conteudo);
    }

    #[test]
    fn no_conteudo_o_cursor_anda_pela_arvore() {
        let mut e = estado();
        e.foco = Foco::Conteudo;
        assert_eq!(e.cursor, vec![0]);
        tecla(&mut e, "j");
        assert_eq!(e.cursor, vec![1]);
        // Na borda de cima ele fica.
        tecla(&mut e, "k");
        tecla(&mut e, "k");
        assert_eq!(e.cursor, vec![0]);
    }

    #[test]
    fn a_janela_segue_o_cursor_pra_baixo() {
        let corpo: String = (0..60).map(|i| format!("Linha {i}.\n\n")).collect();
        let mut e = Estado::novo(paginas(), analisar(&corpo));
        e.foco = Foco::Conteudo;
        e.altura = 10;
        for _ in 0..20 {
            tecla(&mut e, "j");
        }
        let linha = crate::tela::linha_de(&e.linhas, &e.cursor).unwrap();
        assert!(
            linha >= e.topo && linha < e.topo + e.altura,
            "o cursor saiu da janela: linha {linha}, topo {}, altura {}",
            e.topo,
            e.altura
        );
        assert!(e.topo > 0, "a janela não rolou");
    }

    /// O que a tela realmente mostra, linha a linha.
    fn desenho(e: &mut Estado, largura: u16, altura: u16) -> Vec<String> {
        let mut term = Terminal::new(TestBackend::new(largura, altura)).unwrap();
        term.draw(|f| desenhar(f, e)).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area.height)
            .map(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect::<String>()
                    .trim_end()
                    .to_string()
            })
            .collect()
    }

    #[test]
    fn a_tela_mostra_as_paginas_e_o_conteudo() {
        let mut e = estado();
        let linhas = desenho(&mut e, 60, 12);
        let tudo = linhas.join("\n");
        assert!(tudo.contains("páginas"), "{tudo}");
        assert!(tudo.contains("alfa"), "{tudo}");
        assert!(tudo.contains("# Título"), "{tudo}");
        assert!(tudo.contains("- um"), "{tudo}");
    }

    #[test]
    fn o_item_sob_o_cursor_fica_realcado() {
        // Sem realce, a pessoa não sabe onde está — e num terminal não há
        // mouse pra descobrir. É o mesmo achado do ciclo 267: "a lógica
        // está certa" e "a pessoa vê acontecer" são coisas diferentes.
        let mut e = estado();
        e.foco = Foco::Conteudo;
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        let realcadas: Vec<String> = (0..buf.area.height)
            .filter_map(|y| {
                let mut texto = String::new();
                let mut tem_realce = false;
                for x in 0..buf.area.width {
                    let c = &buf[(x, y)];
                    if c.style().bg == Some(Color::Cyan) {
                        tem_realce = true;
                        texto.push_str(c.symbol());
                    }
                }
                tem_realce.then(|| texto.trim().to_string())
            })
            .collect();
        assert_eq!(realcadas.len(), 1, "esperava UMA linha realçada: {realcadas:?}");
        assert!(realcadas[0].contains("Título"), "{realcadas:?}");
    }

    /// As células de uma linha da tela que têm um modificador.
    fn com_modificador(
        e: &mut Estado,
        m: Modifier,
    ) -> Vec<String> {
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, e)).unwrap();
        let buf = term.backend().buffer().clone();
        (0..buf.area.height)
            .filter_map(|y| {
                let texto: String = (0..buf.area.width)
                    .filter(|x| buf[(*x, y)].style().add_modifier.contains(m))
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect();
                (!texto.trim().is_empty()).then(|| texto.trim().to_string())
            })
            .collect()
    }

    #[test]
    fn o_titulo_sai_em_negrito_e_o_marcador_nao_aparece() {
        // É o que o ciclo 287 entrega: o `#` vira estilo, não texto.
        let mut e = Estado::novo(paginas(), analisar("# Um título\n\nplano\n"));
        e.foco = Foco::Paginas; // o cursor não pode roubar o realce
        let negrito = com_modificador(&mut e, Modifier::BOLD);
        assert!(
            negrito.iter().any(|l| l.contains("Um título")),
            "o título não saiu em negrito: {negrito:?}"
        );
        // O parágrafo comum não.
        assert!(
            !negrito.iter().any(|l| l.contains("plano")),
            "o parágrafo veio em negrito: {negrito:?}"
        );
    }

    #[test]
    fn o_negrito_de_dentro_da_linha_pega_so_a_palavra() {
        let mut e = Estado::novo(paginas(), analisar("um **forte** e nada\n"));
        e.foco = Foco::Paginas;
        let negrito = com_modificador(&mut e, Modifier::BOLD);
        assert_eq!(negrito, ["forte"], "o negrito pegou o que não devia");
    }

    #[test]
    fn a_marca_de_quem_nao_tem_texto_nao_fica_apagada() {
        // Achado olhando o painel de tmux: `[fluxo]` saía em
        // cinza-escuro, porque a marca é apagada por ser "estrutura" —
        // só que num embed a marca É o conteúdo.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n"),
        );
        e.foco = Foco::Paginas;
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        let apagado: Vec<String> = (0..buf.area.height)
            .filter_map(|y| {
                let t: String = (0..buf.area.width)
                    .filter(|x| buf[(*x, y)].style().fg == Some(Color::DarkGray))
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect();
                (!t.trim().is_empty()).then(|| t.trim().to_string())
            })
            .collect();
        assert!(
            !apagado.iter().any(|l| l.contains("callout")),
            "o rótulo do embed saiu apagado: {apagado:?}"
        );
    }

    #[test]
    fn a_tela_nao_quebra_num_terminal_minusculo() {
        // Terminal menor que as bordas: `altura` vira 0 e tudo que
        // divide por ela precisa aguentar.
        let mut e = estado();
        let linhas = desenho(&mut e, 10, 2);
        assert_eq!(linhas.len(), 2);
    }
}
