//! O estado da tela e o que cada tecla faz com ele.
//!
//! Separado do laço de eventos de propósito: laço não se testa, transição
//! de estado se testa. `main.rs` só lê tecla, chama `tecla()` e desenha.

use anotadinho_core::inline::Marca;
use anotadinho_core::navegacao::Passo;
use anotadinho_core::vim::{self, Comando, Movimento};
use anotadinho_core::unidade::Tipo;
use anotadinho_core::unidade::{Caminho, Unidade};
use anotadinho_ipc::PageMeta;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::tela::{self, Linha};
use crate::tema::{Realce, Tema};

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
    /// A paleta, lida do CSS da janela (ciclo 288).
    pub tema: Tema,
    /// Os níveis dobrados, pelo caminho (ciclo 289).
    pub dobrados: std::collections::HashSet<anotadinho_core::unidade::Caminho>,
    /// O comando de vim digitado pela metade (ciclo 291).
    pub vim: vim::Pendente,
    /// O termo do filtro. Vazio é "sem filtro" (ciclo 292).
    pub busca: String,
    /// A barra está capturando tecla?
    ///
    /// Separado do termo de propósito. Na primeira versão os dois eram o
    /// mesmo `Option`, e Enter — que fecha a barra MANTENDO o filtro —
    /// não tinha como se exprimir: o `j` seguinte virava texto do termo
    /// em vez de andar pelo resultado. O teste pegou.
    pub barra_aberta: bool,
}

impl Estado {
    /// Estado inicial, com a árvore da primeira página já aberta.
    pub fn novo(paginas: Vec<PageMeta>, arvore: Unidade) -> Self {
        let linhas = tela::linhas(&arvore);
        let cursor = tela::primeiro(&arvore).unwrap_or_default();
        let dobrados = tela::dobras_iniciais(&arvore);
        Self {
            dobrados,
            vim: vim::Pendente::default(),
            busca: String::new(),
            barra_aberta: false,
            paginas,
            pagina: 0,
            arvore,
            linhas,
            cursor,
            topo: 0,
            altura: 20,
            foco: Foco::Paginas,
            sair: false,
            tema: Tema::novo("escuro"),
        }
    }

    /// Troca a paleta.
    pub fn com_tema(mut self, nome: &str) -> Self {
        self.tema = Tema::novo(nome);
        self
    }

    /// Troca a página aberta, recomeçando o cursor.
    pub fn abrir(&mut self, arvore: Unidade) {
        self.linhas = tela::linhas(&arvore);
        self.cursor = tela::primeiro(&arvore).unwrap_or_default();
        self.dobrados = tela::dobras_iniciais(&arvore);
        self.arvore = arvore;
        self.topo = 0;
    }

    /// As linhas que aparecem agora: sem o que está dentro de dobra e,
    /// se houver busca, só o que casa.
    pub fn visiveis(&self) -> Vec<&Linha> {
        let dobradas = tela::visiveis(&self.linhas, &self.dobrados);
        match Some(self.busca.as_str()).filter(|b| !b.is_empty()) {
            None => dobradas,
            Some(termo) => {
                let alvo = termo.to_lowercase();
                dobradas
                    .into_iter()
                    .filter(|l| {
                        l.texto.to_lowercase().contains(&alvo)
                            || l.marca.to_lowercase().contains(&alvo)
                    })
                    .collect()
            }
        }
    }

    /// As páginas que aparecem — filtradas pela busca, quando é o painel
    /// delas que está com foco.
    pub fn paginas_visiveis(&self) -> Vec<(usize, &PageMeta)> {
        let termo = Some(self.busca.as_str())
            .filter(|b| !b.is_empty() && self.foco == Foco::Paginas)
            .map(|b| b.to_lowercase());
        self.paginas
            .iter()
            .enumerate()
            .filter(|(_, p)| match &termo {
                None => true,
                Some(t) => p.title.to_lowercase().contains(t),
            })
            .collect()
    }

    /// Traz o cursor de volta pra uma linha que EXISTE.
    ///
    /// Filtrar pode esconder a linha onde o cursor estava, e um cursor
    /// apontando pro que não aparece é o "fiquei preso" de novo: as
    /// setas andariam sem nada mudar na tela.
    pub fn corrigir_cursor(&mut self) {
        let visiveis = self.visiveis();
        if visiveis
            .iter()
            .any(|l| !l.enfeite && l.caminho == self.cursor)
        {
            return;
        }
        if let Some(primeira) = visiveis.iter().find(|l| !l.enfeite) {
            self.cursor = primeira.caminho.clone();
        }
        self.topo = 0;
        self.seguir_cursor();
    }

    /// A unidade sob o cursor comporta filhos?
    fn cursor_e_nivel(&self) -> bool {
        self.arvore
            .em(&self.cursor)
            .is_some_and(|u| u.politica().aceita_filhos && !u.filhos.is_empty())
    }

    /// Dobra ou desdobra o nível sob o cursor.
    pub fn dobrar(&mut self) {
        if !self.cursor_e_nivel() {
            return;
        }
        if !self.dobrados.remove(&self.cursor) {
            self.dobrados.insert(self.cursor.clone());
        }
    }

    /// Rola pra deixar o cursor visível — o mínimo, nunca centralizando.
    fn seguir_cursor(&mut self) {
        // Conta nas VISÍVEIS: com uma dobra fechada, a posição na lista
        // completa não é a posição na tela, e a janela rolaria pro lugar
        // errado.
        let visiveis = self.visiveis();
        if let Some(l) = visiveis
            .iter()
            .position(|l| !l.enfeite && l.caminho == self.cursor)
        {
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
    // A busca vem ANTES de tudo (ciclo 292): enquanto a barra está
    // aberta, cada tecla é texto. Sem isto, digitar "java" numa busca
    // executaria o `a` de inserção e o `j` de descer.
    if e.barra_aberta {
        return tecla_na_busca(e, tecla);
    }
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

/// O que uma tecla faz com a barra de busca aberta.
fn tecla_na_busca(e: &mut Estado, tecla: &str) -> Option<String> {
    match tecla {
        // Escape LIMPA e fecha: sair deixando o filtro aplicado
        // esconderia metade da página sem nada na tela dizendo por quê.
        "Escape" => {
            e.busca.clear();
            e.barra_aberta = false;
            e.corrigir_cursor();
            None
        }
        // Enter fecha a barra e MANTÉM o filtro — é o que deixa navegar
        // pelo resultado.
        "Enter" => {
            e.barra_aberta = false;
            e.corrigir_cursor();
            if e.foco == Foco::Paginas {
                return e.paginas.get(e.pagina).map(|p| p.path.clone());
            }
            None
        }
        "Backspace" => {
            e.busca.pop();
            e.corrigir_cursor();
            None
        }
        // Uma tecla de um caractere é texto; o resto (setas, F1) não.
        t if t.chars().count() == 1 => {
            e.busca.push_str(t);
            e.corrigir_cursor();
            None
        }
        _ => None,
    }
}

fn tecla_nas_paginas(e: &mut Estado, tecla: &str) -> Option<String> {
    if tecla == "/" {
        e.busca.clear();
        e.barra_aberta = true;
        return None;
    }
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
    // A gramática do vim primeiro (ciclo 291).
    //
    // Ela mora no núcleo desde o ciclo 285 — contagem, operador,
    // movimento, `gg`/`G` — e ninguém a consultava. É ela que dá `10j` e
    // `G` sem uma linha de lógica nova aqui: a TUI só traduz o comando
    // fechado em passos de navegação.
    match vim::tecla_normal(&mut e.vim, tecla, false) {
        vim::Passo::Aguardando => return,
        // A gramática já mapeia `/` pra busca desde o ciclo 254 — mais
        // uma coisa que estava escrita e não era consultada.
        vim::Passo::Pronto(vim::Comando::Busca) => {
            e.busca.clear();
            e.barra_aberta = true;
            return;
        }
        vim::Passo::Pronto(c) => {
            comando_de_vim(e, c);
            return;
        }
        // Seta, Escape, `z`: não são da gramática e seguem o caminho de
        // sempre, logo abaixo.
        vim::Passo::Ignorada => {}
    }

    // Quem decide o destino é o núcleo. Este `match` só diz qual PASSO a
    // tecla pede; a régua de "dá ou não dá" é da árvore (ciclo 281).
    // Dobrar é do painel, não da árvore: o modelo não sabe o que está
    // escondido.
    if tecla == "z" || tecla == " " {
        e.dobrar();
        e.seguir_cursor();
        return;
    }
    let passo = match tecla {
        "j" | "ArrowDown" => Passo::Proximo,
        "k" | "ArrowUp" => Passo::Anterior,
        "Enter" | "l" | "ArrowRight" => Passo::Entrar,
        "Escape" | "h" | "ArrowLeft" => Passo::Sair,
        _ => return,
    };
    // Entrar num nível dobrado ABRE ele: pedir pra descer e não descer
    // seria o "fiquei preso" de novo, agora por outra porta.
    if passo == Passo::Entrar {
        e.dobrados.remove(&e.cursor);
    }
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
        .paginas_visiveis()
        .into_iter()
        .map(|(i, p)| {
            let estilo = if i == e.pagina {
                realce(e.foco == Foco::Paginas, &e.tema)
            } else {
                e.tema.estilo(Realce::Texto)
            };
            Line::from(Span::styled(p.title.clone(), estilo))
        })
        .collect();
    let mut bloco_paginas = borda("páginas", e.foco == Foco::Paginas, &e.tema);
    if let Some(rodape) = rodape_de_busca(e, Foco::Paginas) {
        bloco_paginas = bloco_paginas.title_bottom(rodape);
    }
    f.render_widget(Paragraph::new(paginas).block(bloco_paginas), colunas[0]);

    // Largura de dentro da borda: a faixa do h1 precisa chegar até a
    // ponta pra parecer faixa.
    let largura_util = colunas[1].width.saturating_sub(2) as usize;
    let linhas_visiveis = e.visiveis();
    let visiveis: Vec<Line> = linhas_visiveis
        .iter()
        .skip(e.topo)
        .take(e.altura)
        .map(|l| {
            linha_estilizada(
                l,
                !l.enfeite && l.caminho == e.cursor && e.foco == Foco::Conteudo,
                e.dobrados.contains(&l.caminho),
                &e.tema,
                largura_util,
            )
        })
        .collect();
    let titulo = e
        .paginas
        .get(e.pagina)
        .map(|p| p.title.clone())
        .unwrap_or_default();
    // O que está digitado pela metade aparece no rodapé, como o "2d3"
    // do canto do vim. Sem isso, teclar `1` `0` e não ver nada faz a
    // pessoa achar que a tecla não pegou (ciclo 291).
    let mut bloco = borda(&titulo, e.foco == Foco::Conteudo, &e.tema);
    if let Some(rodape) = rodape_de_busca(e, Foco::Conteudo) {
        bloco = bloco.title_bottom(rodape);
    }
    if e.vim.em_curso() {
        bloco = bloco.title_bottom(
            Line::from(Span::styled(
                format!(" {} ", e.vim.rotulo()),
                e.tema.estilo(Realce::Cursor),
            ))
            .right_aligned(),
        );
    }
    f.render_widget(Paragraph::new(visiveis).block(bloco), colunas[1]);
}

/// Uma linha do conteúdo, com o estilo do BLOCO e o dos trechos.
///
/// Dois níveis, e eles se somam: o bloco dá a cor de fundo do papel
/// (título forte e colorido, citação apagada, código em outra cor), e o
/// trecho dá o negrito/itálico/código de dentro (ciclo 287).
///
/// Sob o cursor, tudo cede: a linha inteira recebe o realce, porque
/// duas ênfases competindo fazem a pessoa não saber onde está.
fn linha_estilizada<'a>(
    l: &'a crate::tela::Linha,
    sob_cursor: bool,
    dobrada: bool,
    tema: &Tema,
    largura: usize,
) -> Line<'a> {
    let recuo = "  ".repeat(l.nivel);
    // O fecho da caixa vai até a borda, e vem ANTES de qualquer outra
    // decisão: ele não tem texto nem trechos, então sairia pelos
    // atalhos abaixo sem nunca chegar no preenchimento.
    if l.enfeite && l.marca == "└" {
        let traco = largura.saturating_sub(l.nivel * 2 + 1);
        return Line::from(vec![
            Span::styled(recuo, Style::default()),
            Span::styled(
                format!("└{}", "─".repeat(traco)),
                tema.estilo(Realce::Embed),
            ),
        ]);
    }
    let marca = marca_com_dobra(l, dobrada);
    // O resumo só entra quando o nível está FECHADO — aberto, os filhos
    // já dizem o que ele tem, e `· 1 item` em cima do único item é
    // ruído (ciclo 293).
    let texto = if dobrada && !l.resumo.is_empty() {
        l.resumo.clone()
    } else {
        l.texto.clone()
    };
    if sob_cursor {
        let plano = format!("{recuo}{marca} {texto}");
        return Line::from(Span::styled(
            plano.trim_end().to_string(),
            tema.estilo(Realce::Cursor),
        ));
    }

    // Enfeite de embed veste a cor do embed: ele é parte do cartão.
    let base = if l.enfeite {
        tema.estilo(Realce::Embed)
    } else {
        tema.estilo(papel_do_bloco(&l.tipo))
    };
    let mut spans = vec![Span::styled(recuo, Style::default())];
    if !marca.trim().is_empty() {
        // A marca costuma ser estrutura e fica apagada pra não competir
        // com o conteúdo — o `##` de um título, o `-` de um item são
        // sintaxe de markdown, não o que se lê.
        //
        // Menos quando ela É a identidade: `[kanban]`, `·` de uma lista.
        // Ali apagar a marca apaga o que a linha tem de mais importante.
        //
        // Quem responde é o MODELO, não o texto: bloco que aceita texto
        // tem a marca como sintaxe; quem não aceita (embed, grupo) tem a
        // marca como nome. A primeira versão perguntava "o texto está
        // vazio?", e quebrou no mesmo dia em que os níveis passaram a
        // dizer o que contêm (ciclo 289).
        let estilo_marca = if l.tipo.politica().aceita_texto {
            tema.estilo(Realce::Marca)
        } else {
            base
        };
        spans.push(Span::styled(format!("{marca} "), estilo_marca));
    }
    if l.trechos.is_empty() || texto != l.texto {
        spans.push(Span::styled(texto, base));
        return Line::from(spans);
    }
    for t in &l.trechos {
        spans.push(Span::styled(
            t.texto.clone(),
            estilo_do_trecho(&t.marcas, base, tema),
        ));
    }
    preencher(spans, l, base, largura)
}

/// Estica a faixa do h1 até a borda.
///
/// Sem isto o fundo do título para onde o texto acaba, e o que devia
/// parecer uma faixa parece um realce mal-acabado. Só o h1: é o nome da
/// página, e faixa em tudo vira listra.
fn preencher<'a>(
    spans: Vec<Span<'a>>,
    l: &crate::tela::Linha,
    base: Style,
    largura: usize,
) -> Line<'a> {
    if !matches!(l.tipo, Tipo::Titulo(1)) {
        return Line::from(spans);
    }
    let usado: usize = spans.iter().map(|s| s.content.chars().count()).sum();
    let mut spans = spans;
    if usado < largura {
        spans.push(Span::styled(" ".repeat(largura - usado), base));
    }
    Line::from(spans)
}

/// A barra de busca no rodapé do painel, quando é dele a busca.
///
/// Aparece como `/termo`, que é onde o vim a põe. Enquanto está aberta
/// ela é a única coisa que recebe tecla, então precisa estar VISÍVEL —
/// uma busca invisível filtrando a tela seria indistinguível de um
/// defeito.
fn rodape_de_busca<'a>(e: &Estado, painel: Foco) -> Option<Line<'a>> {
    if e.foco != painel || (!e.barra_aberta && e.busca.is_empty()) {
        return None;
    }
    let termo = &e.busca;
    Some(Line::from(Span::styled(
        format!(" /{termo} "),
        e.tema.estilo(Realce::Cursor),
    )))
}

/// Executa um comando fechado da gramática do vim.
///
/// Só MOVIMENTO por enquanto: a TUI é de leitura, e apagar/copiar/entrar
/// em inserção precisam da edição ligada — que existe no núcleo desde o
/// ciclo 285 e ainda não tem caminho até aqui. O comando é consumido e
/// não faz nada, em vez de vazar pro tratamento de tecla e disparar
/// outra coisa por engano.
fn comando_de_vim(e: &mut Estado, c: Comando) {
    let Comando::Mover(mov, vezes) = c else { return };
    match mov {
        Movimento::Baixo => repetir(e, Passo::Proximo, vezes),
        Movimento::Cima => repetir(e, Passo::Anterior, vezes),
        // Numa árvore, "pra dentro" é o que direita significa — e é o
        // que o Enter já faz.
        Movimento::Direita => repetir(e, Passo::Entrar, 1),
        Movimento::Esquerda => repetir(e, Passo::Sair, 1),
        Movimento::InicioDoDocumento => ir_para_linha(e, 0),
        // `G` sozinho é o fim; `10G` é a décima linha, como no vim.
        Movimento::FimDoDocumento => {
            let ultima = e.visiveis().iter().filter(|l| !l.enfeite).count().saturating_sub(1);
            let alvo = if vezes > 1 {
                (vezes as usize - 1).min(ultima)
            } else {
                ultima
            };
            ir_para_linha(e, alvo);
        }
        // Movimento DENTRO da linha não tem o que fazer numa tela de
        // blocos: aqui o cursor pousa em unidades, não em caracteres.
        // Volta quando a edição chegar.
        _ => {}
    }
}

/// Aplica o mesmo passo `vezes` vezes, parando na borda.
///
/// **Com busca ativa, anda pela LISTA e não pela árvore.** Uma vista
/// filtrada não é uma árvore: os irmãos que não casaram sumiram, e
/// andar pelo modelo pousaria numa linha que não está na tela — o
/// "fiquei preso" do ciclo 280 por outra porta.
///
/// Sem busca, quem decide continua sendo `navegacao::mover`, e o nível
/// nunca muda (ciclo 281).
fn repetir(e: &mut Estado, passo: Passo, vezes: u32) {
    let filtrando = !e.busca.is_empty();
    if filtrando && matches!(passo, Passo::Proximo | Passo::Anterior) {
        andar_na_lista(e, passo == Passo::Proximo, vezes);
        return;
    }
    for _ in 0..vezes.max(1) {
        let antes = e.cursor.clone();
        if passo == Passo::Entrar {
            e.dobrados.remove(&e.cursor);
        }
        e.cursor = tela::andar(&e.arvore, &e.cursor, passo);
        // Bateu na borda: repetir não leva a lugar nenhum, e `1000j` não
        // pode custar mil travessias da árvore.
        if e.cursor == antes {
            break;
        }
    }
    e.seguir_cursor();
}

/// Anda pelas linhas que estão na tela, parando nas pontas.
fn andar_na_lista(e: &mut Estado, adiante: bool, vezes: u32) {
    // Enfeite não é destino: a borda de uma caixa ocupa linha e não
    // recebe cursor.
    let visiveis: Vec<_> = e.visiveis().into_iter().filter(|l| !l.enfeite).collect();
    let Some(atual) = visiveis.iter().position(|l| l.caminho == e.cursor) else {
        return;
    };
    let passos = vezes.max(1) as usize;
    let alvo = if adiante {
        (atual + passos).min(visiveis.len().saturating_sub(1))
    } else {
        atual.saturating_sub(passos)
    };
    let destino = visiveis[alvo].caminho.clone();
    e.cursor = destino;
    e.seguir_cursor();
}

/// Põe o cursor na n-ésima linha VISÍVEL.
fn ir_para_linha(e: &mut Estado, indice: usize) {
    let reais: Vec<_> = e.visiveis().into_iter().filter(|l| !l.enfeite).collect();
    if let Some(l) = reais.get(indice) {
        e.cursor = l.caminho.clone();
    }
    e.seguir_cursor();
}

/// A marca da linha, com a seta de dobra quando o nível pode dobrar.
///
/// `▾`/`▸` são a afordância universal de outline, e resolvem uma coisa
/// que faltava: nada na tela dizia que um nível PODE ser fechado.
///
/// A lista troca o `·` pela seta; o embed mantém o nome dele e ganha a
/// seta na frente, porque `[kanban]` é identidade e não se troca.
fn marca_com_dobra(l: &crate::tela::Linha, dobrada: bool) -> String {
    if l.resumo.is_empty() {
        return l.marca.clone();
    }
    let seta = if dobrada { "▸" } else { "▾" };
    match l.tipo {
        Tipo::Lista | Tipo::ListaOrdenada => seta.to_string(),
        _ => format!("{seta} {}", l.marca),
    }
}

/// O papel de um bloco, pelo tipo dele.
fn papel_do_bloco(tipo: &Tipo) -> Realce {
    match tipo {
        Tipo::Titulo(n) => Realce::Titulo(*n),
        Tipo::Citacao => Realce::Citacao,
        Tipo::Codigo(_) => Realce::Codigo,
        Tipo::Embed(_) => Realce::Embed,
        Tipo::Parte { .. } => Realce::Parte,
        _ => Realce::Texto,
    }
}

/// As marcas de um trecho somadas ao estilo do bloco.
///
/// Negrito e itálico são MODIFICADOR, não cor: valem em qualquer tema e
/// não passam pela paleta. Código e link têm cor própria, e essa vem do
/// tema como todo o resto.
fn estilo_do_trecho(marcas: &[Marca], base: Style, tema: &Tema) -> Style {
    marcas.iter().fold(base, |e, m| match m {
        Marca::Negrito => e.add_modifier(Modifier::BOLD),
        Marca::Italico => e.add_modifier(Modifier::ITALIC),
        Marca::Tachado => e.add_modifier(Modifier::CROSSED_OUT),
        Marca::Codigo => e.patch(tema.estilo(Realce::Codigo)),
        Marca::Link => e.patch(tema.estilo(Realce::Link)),
        Marca::Wikilink => e.patch(tema.estilo(Realce::Wikilink)),
    })
}

/// Realce do item sob o cursor.
///
/// Fora do painel com foco ele fica apagado: sem isso a tela mostra dois
/// cursores e nenhum dos dois parece o de verdade.
fn realce(com_foco: bool, tema: &Tema) -> Style {
    tema.estilo(if com_foco {
        Realce::Cursor
    } else {
        Realce::CursorApagado
    })
}

fn borda<'a>(titulo: &str, com_foco: bool, tema: &Tema) -> Block<'a> {
    Block::default()
        .borders(Borders::ALL)
        .title(titulo.to_string())
        .border_style(tema.estilo(if com_foco { Realce::BordaFoco } else { Realce::Borda }))
}

#[cfg(test)]
mod testes {
    use super::*;
    use anotadinho_core::analise::analisar;
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
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
        let cursor = e.tema.estilo(Realce::Cursor).bg.unwrap();
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        let realcadas: Vec<String> = (0..buf.area.height)
            .filter_map(|y| {
                let mut texto = String::new();
                let mut tem_realce = false;
                for x in 0..buf.area.width {
                    let c = &buf[(x, y)];
                    if c.style().bg == Some(cursor) {
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
    fn a_marca_que_e_identidade_nao_fica_apagada() {
        // Achado olhando o painel de tmux: `[fluxo]` saía em
        // cinza-escuro, porque a marca é apagada por ser "estrutura" —
        // só que num embed a marca É a identidade. Quem separa os dois
        // casos é a política do modelo, não o texto estar vazio.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n"),
        );
        e.foco = Foco::Paginas;
        let marca = e.tema.estilo(Realce::Marca).fg.unwrap();
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        let apagado: Vec<String> = (0..buf.area.height)
            .filter_map(|y| {
                let t: String = (0..buf.area.width)
                    .filter(|x| buf[(*x, y)].style().fg == Some(marca))
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

    /// Uma página com `n` parágrafos numerados, pra contar saltos.
    fn pagina_numerada(n: usize) -> Unidade {
        analisar(
            &(0..n)
                .map(|i| format!("linha {i}\n\n"))
                .collect::<String>(),
        )
    }

    #[test]
    fn lista_aberta_nao_repete_a_contagem_na_tela() {
        // O ruído que a pessoa apontou: uma página com nove listas de um
        // item mostrava nove `· 1 item`, cada um em cima do seu único
        // item. Aberto, o resumo não é informação.
        let md = (0..5)
            .map(|i| format!("parágrafo {i}\n\n- item {i}\n\n"))
            .collect::<String>();
        let mut e = Estado::novo(paginas(), analisar(&md));
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 20).join("\n");
        assert!(
            !tudo.contains("1 item"),
            "a contagem apareceu com o nível aberto:\n{tudo}"
        );
        // E a seta diz que dá pra fechar.
        assert!(tudo.contains('▾'), "faltou a afordância de dobra:\n{tudo}");
    }

    #[test]
    fn lista_fechada_mostra_a_contagem_e_a_seta_vira() {
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n- tres\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1];
        tecla(&mut e, "z");
        e.foco = Foco::Paginas; // pra o realce do cursor não cobrir
        let tudo = desenho(&mut e, 60, 12).join("\n");
        assert!(tudo.contains("3 items"), "faltou a contagem:\n{tudo}");
        assert!(tudo.contains('▸'), "a seta não virou:\n{tudo}");
    }

    #[test]
    fn a_busca_filtra_as_linhas_do_conteudo() {
        let mut e = Estado::novo(
            paginas(),
            analisar("alfa\n\nbeta\n\nalfa de novo\n\ngama\n"),
        );
        e.foco = Foco::Conteudo;
        assert_eq!(e.visiveis().len(), 4);

        tecla(&mut e, "/");
        for t in ["a", "l", "f", "a"] {
            tecla(&mut e, t);
        }
        let achadas: Vec<&str> = e.visiveis().iter().map(|l| l.texto.as_str()).collect();
        assert_eq!(achadas, ["alfa", "alfa de novo"]);
    }

    #[test]
    fn a_busca_nao_liga_pra_caixa() {
        let mut e = Estado::novo(paginas(), analisar("Alfa\n\nbeto\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "/");
        tecla(&mut e, "A");
        assert_eq!(e.visiveis().len(), 1);
        tecla(&mut e, "Backspace");
        tecla(&mut e, "a");
        assert_eq!(e.visiveis().len(), 1, "minúscula não achou o que maiúscula achou");
    }

    #[test]
    fn digitar_na_busca_nao_dispara_comando_de_vim() {
        // Sem a busca vindo antes de tudo, digitar "java" executaria o
        // `a` de inserção e o `j` de descer. É o defeito mais provável
        // de todo o ciclo.
        let mut e = Estado::novo(paginas(), pagina_numerada(30));
        e.foco = Foco::Conteudo;
        let antes = e.cursor.clone();
        tecla(&mut e, "/");
        for t in ["j", "a", "v", "a"] {
            tecla(&mut e, t);
        }
        assert_eq!(e.busca, "java");
        assert_eq!(e.cursor, antes, "o `j` do termo moveu o cursor");
        assert!(!e.vim.em_curso());
    }

    #[test]
    fn escape_limpa_a_busca_e_devolve_a_pagina_inteira() {
        // Sair deixando o filtro aplicado esconderia metade da página
        // sem nada na tela dizendo por quê.
        let mut e = Estado::novo(paginas(), analisar("alfa\n\nbeta\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "/");
        tecla(&mut e, "a");
        tecla(&mut e, "l");
        assert_eq!(e.visiveis().len(), 1);
        tecla(&mut e, "Escape");
        assert!(e.busca.is_empty());
        assert_eq!(e.visiveis().len(), 2);
    }

    #[test]
    fn enter_fecha_a_barra_e_mantem_o_filtro() {
        // É o que deixa navegar pelo resultado com j/k.
        let mut e = Estado::novo(paginas(), analisar("alfa\n\nbeta\n\nalfa dois\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "/");
        tecla(&mut e, "a");
        tecla(&mut e, "l");
        tecla(&mut e, "Enter");
        assert_eq!(e.busca, "al", "o filtro caiu junto com a barra");
        assert_eq!(e.visiveis().len(), 2);
        // E as teclas voltam a ser comando.
        // E as teclas voltam a ser comando — andando pela LISTA, que é
        // o que a vista filtrada é.
        tecla(&mut e, "j");
        assert_eq!(e.cursor, vec![2], "j pousou fora do resultado");
    }

    #[test]
    fn o_cursor_nao_fica_apontando_pro_que_sumiu() {
        // Filtrar pode esconder a linha onde o cursor estava, e aí as
        // setas andariam sem nada mudar na tela — o "fiquei preso" de
        // novo, por outra porta.
        let mut e = Estado::novo(paginas(), analisar("alfa\n\nbeta\n\ngama\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![2]; // gama
        tecla(&mut e, "/");
        tecla(&mut e, "a");
        tecla(&mut e, "l");
        let visiveis = e.visiveis();
        assert!(
            visiveis.iter().any(|l| l.caminho == e.cursor),
            "o cursor ficou fora do que aparece"
        );
    }

    #[test]
    fn a_busca_filtra_as_paginas_quando_o_foco_e_delas() {
        let mut e = estado();
        assert_eq!(e.paginas_visiveis().len(), 3);
        tecla(&mut e, "/");
        tecla(&mut e, "b");
        let achadas: Vec<&str> = e
            .paginas_visiveis()
            .into_iter()
            .map(|(_, p)| p.title.as_str())
            .collect();
        assert_eq!(achadas, ["beta"]);
    }

    #[test]
    fn a_barra_de_busca_aparece_na_tela() {
        // Busca invisível filtrando a tela é indistinguível de defeito.
        let mut e = Estado::novo(paginas(), analisar("alfa\n\nbeta\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "/");
        tecla(&mut e, "a");
        let tudo = desenho(&mut e, 60, 12).join("\n");
        assert!(tudo.contains("/a"), "a barra não apareceu:\n{tudo}");
    }

    #[test]
    fn contagem_anda_varios_blocos_de_uma_vez() {
        // `10j` — a gramática vem do núcleo (ciclo 285) e nunca tinha
        // sido consultada por ninguém.
        let mut e = Estado::novo(paginas(), pagina_numerada(30));
        e.foco = Foco::Conteudo;
        for t in ["1", "0", "j"] {
            tecla(&mut e, t);
        }
        assert_eq!(e.cursor, vec![10]);
        // E pra trás.
        for t in ["3", "k"] {
            tecla(&mut e, t);
        }
        assert_eq!(e.cursor, vec![7]);
    }

    #[test]
    fn a_contagem_fica_pendente_ate_o_movimento_chegar() {
        // Teclar `1` e `0` não move nada: o comando não fechou. Sem isso
        // o `1` viraria "ir pra linha 1" e a contagem nunca existiria.
        let mut e = Estado::novo(paginas(), pagina_numerada(30));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "1");
        tecla(&mut e, "0");
        assert_eq!(e.cursor, vec![0], "moveu antes do comando fechar");
        assert!(e.vim.em_curso());
        assert_eq!(e.vim.rotulo(), "10");
    }

    #[test]
    fn gg_e_g_maiusculo_vao_pras_pontas() {
        let mut e = Estado::novo(paginas(), pagina_numerada(30));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "G");
        assert_eq!(e.cursor, vec![29]);
        tecla(&mut e, "g");
        assert!(e.vim.em_curso(), "um `g` só já fechou comando");
        tecla(&mut e, "g");
        assert_eq!(e.cursor, vec![0]);
    }

    #[test]
    fn numero_antes_do_g_maiusculo_e_a_linha() {
        // `10G` é a décima linha, como no vim.
        let mut e = Estado::novo(paginas(), pagina_numerada(30));
        e.foco = Foco::Conteudo;
        for t in ["1", "0", "G"] {
            tecla(&mut e, t);
        }
        assert_eq!(e.cursor, vec![9]);
    }

    #[test]
    fn a_contagem_para_na_borda_e_nao_custa_mil_travessias() {
        // `1000j` numa página de 5 blocos para no último. Sem a saída
        // antecipada, seriam mil travessias da árvore por tecla.
        let mut e = Estado::novo(paginas(), pagina_numerada(5));
        e.foco = Foco::Conteudo;
        for t in ["1", "0", "0", "0", "j"] {
            tecla(&mut e, t);
        }
        assert_eq!(e.cursor, vec![4]);
    }

    #[test]
    fn a_seta_continua_valendo_junto_da_gramatica() {
        // A gramática ignora seta, e o caminho de sempre trata. As duas
        // coisas convivem — quebrar a seta pra ganhar `10j` seria troca
        // ruim.
        let mut e = Estado::novo(paginas(), pagina_numerada(5));
        e.foco = Foco::Conteudo;
        tecla(&mut e, "ArrowDown");
        assert_eq!(e.cursor, vec![1]);
        tecla(&mut e, "ArrowUp");
        assert_eq!(e.cursor, vec![0]);
    }

    #[test]
    fn dobrar_continua_funcionando_com_a_gramatica_ligada() {
        // `z` não é comando de vim, então a gramática o ignora e ele
        // chega no tratamento do painel.
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1];
        tecla(&mut e, "z");
        assert!(e.dobrados.contains(&vec![1]), "z deixou de dobrar");
    }

    #[test]
    fn operador_sem_edicao_e_consumido_sem_fazer_nada() {
        // `dd` fecha um comando de apagar, e a TUI é de leitura. O certo
        // é consumir e não fazer nada — deixar vazar faria o segundo `d`
        // cair no tratamento de tecla e disparar outra coisa.
        let mut e = Estado::novo(paginas(), pagina_numerada(5));
        e.foco = Foco::Conteudo;
        let antes = e.cursor.clone();
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(e.cursor, antes);
        assert!(!e.vim.em_curso(), "o comando ficou pendente pra sempre");
    }

    #[test]
    fn dobrar_esconde_o_conteudo_e_mantem_a_linha() {
        // Quem dobrou precisa continuar vendo ONDE dobrou — é essa linha
        // que ele vai reabrir.
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n\ndepois\n"));
        e.foco = Foco::Conteudo;
        let antes = e.visiveis().len();

        e.cursor = vec![1]; // a lista
        tecla(&mut e, "z");
        let depois = e.visiveis();
        assert_eq!(depois.len(), antes - 2, "os itens não sumiram");
        assert!(
            depois.iter().any(|l| l.caminho == vec![1]),
            "a linha da lista sumiu junto"
        );

        tecla(&mut e, "z");
        assert_eq!(e.visiveis().len(), antes, "desdobrar não trouxe de volta");
    }

    #[test]
    fn entrar_num_nivel_dobrado_abre_ele() {
        // Pedir pra descer e não descer seria o "fiquei preso" do ciclo
        // 280 de novo, por outra porta.
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1];
        tecla(&mut e, "z");
        assert!(e.dobrados.contains(&vec![1]));

        tecla(&mut e, "Enter");
        assert!(!e.dobrados.contains(&vec![1]), "continuou dobrado");
        assert_eq!(e.cursor, vec![1, 0], "não desceu pro primeiro item");
    }

    #[test]
    fn dobrar_num_bloco_de_texto_nao_faz_nada() {
        let mut e = Estado::novo(paginas(), analisar("um parágrafo\n"));
        e.foco = Foco::Conteudo;
        let antes = e.visiveis().len();
        tecla(&mut e, "z");
        assert!(e.dobrados.is_empty());
        assert_eq!(e.visiveis().len(), antes);
    }

    #[test]
    fn uma_lista_enorme_nasce_dobrada() {
        // O caso que a pessoa levantou: mil itens não se percorre de `j`
        // em `j`. Dobrada, ela ocupa uma linha até alguém abrir.
        let md: String = std::iter::once("antes\n\n".to_string())
            .chain((0..1000).map(|i| format!("- item {i}\n")))
            .collect();
        let e = Estado::novo(paginas(), analisar(&md));
        assert_eq!(
            e.visiveis().len(),
            2,
            "a lista de mil itens não nasceu dobrada"
        );
        // E a linha dela diz o tamanho — no resumo, que é o que se
        // mostra com o nível fechado (ciclo 293).
        assert_eq!(e.visiveis()[1].resumo, "1000 items");
    }

    #[test]
    fn a_janela_conta_as_linhas_visiveis_e_nao_todas() {
        // Com dobra fechada, a posição na lista completa não é a posição
        // na tela — e a janela rolaria pro lugar errado.
        let md: String = std::iter::once("topo\n\n".to_string())
            .chain((0..40).map(|i| format!("- item {i}\n")))
            .chain(std::iter::once("\nfim\n".to_string()))
            .collect();
        let mut e = Estado::novo(paginas(), analisar(&md));
        e.foco = Foco::Conteudo;
        e.altura = 10;
        // A lista nasce dobrada, então só três linhas existem.
        assert_eq!(e.visiveis().len(), 3);
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        assert_eq!(e.topo, 0, "rolou numa página que cabe inteira");
    }

    #[test]
    fn trocar_o_tema_muda_as_cores_desenhadas() {
        // O ponto do ciclo 288: a paleta vem do CSS da janela, e trocar
        // de tema troca o que aparece na tela. Sem esta asserção, os
        // quatro temas poderiam estar todos caindo no escuro em
        // silêncio — que é o defeito mais fácil de não perceber.
        fn cores(nome: &str) -> Vec<Color> {
            let mut e = Estado::novo(paginas(), analisar("# Título\n\ntexto\n")).com_tema(nome);
            e.foco = Foco::Paginas;
            let mut term = Terminal::new(TestBackend::new(40, 8)).unwrap();
            term.draw(|f| desenhar(f, &mut e)).unwrap();
            let buf = term.backend().buffer().clone();
            let mut vistas: Vec<Color> = (0..buf.area.height)
                .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
                .filter_map(|(x, y)| buf[(x, y)].style().fg)
                .collect();
            vistas.sort_by_key(|c| format!("{c:?}"));
            vistas.dedup();
            vistas
        }
        let escuro = cores("escuro");
        let papel = cores("papel");
        assert_ne!(escuro, papel, "os dois temas desenharam as mesmas cores");
        // E toda cor PINTADA é do CSS, não nome de terminal. O `Reset`
        // fica de fora: são as células que ninguém tocou.
        assert!(
            escuro
                .iter()
                .all(|c| matches!(c, Color::Rgb(..) | Color::Reset)),
            "veio cor de terminal em vez da paleta: {escuro:?}"
        );
        assert!(
            escuro.iter().filter(|c| matches!(c, Color::Rgb(..))).count() >= 4,
            "quase nada foi pintado pela paleta: {escuro:?}"
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
