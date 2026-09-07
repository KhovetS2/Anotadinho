//! O estado da tela e o que cada tecla faz com ele.
//!
//! Separado do laço de eventos de propósito: laço não se testa, transição
//! de estado se testa. `main.rs` só lê tecla, chama `tecla()` e desenha.

use anotadinho_core::inline::Marca;
use anotadinho_core::navegacao::Passo;
use anotadinho_core::vim::{self, Comando, Movimento};
use anotadinho_core::unidade::{Arranjo, Tipo};
use anotadinho_core::unidade::{Caminho, Unidade};
use anotadinho_ipc::PageMeta;
use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::sidebar::{self, Item};
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
    /// A árvore de pastas da sidebar (ciclo 299).
    pub arvore_sidebar: sidebar::No,
    /// As pastas fechadas, pelo caminho.
    pub pastas_fechadas: std::collections::BTreeSet<String>,
    /// Que linha da sidebar está selecionada.
    pub linha_sidebar: usize,
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
    /// Em qual painel a busca foi digitada (ciclo 297).
    ///
    /// Achado usando: busquei uma página, abri, e o conteúdo veio vazio
    /// — o termo da busca de PÁGINAS estava filtrando as linhas também.
    /// Filtro é do painel onde foi digitado; aplicar nos dois faz o
    /// segundo esconder tudo.
    pub busca_em: Foco,
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
        let arvore_sidebar = sidebar::arvore(&paginas);
        let pastas_fechadas = sidebar::fechadas_iniciais(&arvore_sidebar);
        Self {
            dobrados,
            arvore_sidebar,
            pastas_fechadas,
            linha_sidebar: 0,
            vim: vim::Pendente::default(),
            busca: String::new(),
            busca_em: Foco::Paginas,
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
        // Abrir uma página descarta a busca: o termo era pra achar a
        // página, e mantê-lo filtraria o conteúdo dela por acidente.
        self.busca.clear();
        self.barra_aberta = false;
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
        match Some(self.busca.as_str())
            .filter(|b| !b.is_empty() && self.busca_em == Foco::Conteudo)
        {
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
            .filter(|b| !b.is_empty() && self.busca_em == Foco::Paginas)
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
        if visiveis.iter().any(|l| !l.enfeite && l.mostra(&self.cursor)) {
            return;
        }
        if let Some(primeira) = visiveis.iter().find(|l| !l.enfeite) {
            self.cursor = primeira.caminho.clone();
        }
        self.topo = 0;
        self.seguir_cursor();
    }

    /// As linhas da sidebar que aparecem agora.
    pub fn sidebar_visivel(&self) -> Vec<sidebar::Linha> {
        let todas = sidebar::visiveis(&self.arvore_sidebar, &self.paginas, &self.pastas_fechadas);
        match Some(self.busca.as_str())
            .filter(|b| !b.is_empty() && self.busca_em == Foco::Paginas)
        {
            None => todas,
            // Com busca, a hierarquia sai da frente: o que interessa é
            // o que casou, e mostrar as pastas vazias em volta seria
            // ruído. É o que a janela faz desde o ciclo 106.
            Some(termo) => {
                let alvo = termo.to_lowercase();
                todas
                    .into_iter()
                    .filter(|l| match &l.item {
                        Item::Pagina { titulo, .. } => titulo.to_lowercase().contains(&alvo),
                        Item::Pasta { .. } => false,
                    })
                    .map(|mut l| {
                        l.nivel = 0;
                        l
                    })
                    .collect()
            }
        }
    }

    /// Abre ou fecha a pasta selecionada.
    fn dobrar_pasta(&mut self) -> bool {
        let visiveis = self.sidebar_visivel();
        let Some(l) = visiveis.get(self.linha_sidebar) else {
            return false;
        };
        let Item::Pasta { caminho, .. } = &l.item else {
            return false;
        };
        if !self.pastas_fechadas.remove(caminho) {
            self.pastas_fechadas.insert(caminho.clone());
        }
        true
    }

    /// Anda na sidebar VISÍVEL — pastas e páginas, na ordem da tela.
    fn andar_nas_paginas(&mut self, adiante: bool) {
        let visiveis = self.sidebar_visivel();
        if visiveis.is_empty() {
            return;
        }
        let nova = if adiante {
            (self.linha_sidebar + 1).min(visiveis.len() - 1)
        } else {
            self.linha_sidebar.saturating_sub(1)
        };
        self.linha_sidebar = nova;
        // A página selecionada acompanha, pra o Enter abrir a certa.
        if let Some(Item::Pagina { indice, .. }) = visiveis.get(nova).map(|l| &l.item) {
            self.pagina = *indice;
        }
    }

    /// Traz a seleção de página pra dentro do que o filtro deixou.
    fn corrigir_pagina(&mut self) {
        let visiveis: Vec<usize> = self.paginas_visiveis().into_iter().map(|(i, _)| i).collect();
        if !visiveis.contains(&self.pagina) {
            self.pagina = visiveis.first().copied().unwrap_or(0);
        }
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
            .position(|l| !l.enfeite && l.mostra(&self.cursor))
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
            e.corrigir_pagina();
            None
        }
        // Uma tecla de um caractere é texto; o resto (setas, F1) não.
        t if t.chars().count() == 1 => {
            e.busca.push_str(t);
            e.corrigir_cursor();
            e.corrigir_pagina();
            None
        }
        _ => None,
    }
}

fn tecla_nas_paginas(e: &mut Estado, tecla: &str) -> Option<String> {
    if tecla == "/" {
        e.busca.clear();
        e.busca_em = Foco::Paginas;
        e.barra_aberta = true;
        return None;
    }
    match tecla {
        // Anda na lista VISÍVEL, não na completa (ciclo 297).
        //
        // Achado usando: busquei "incio", a lista filtrou pra uma
        // página, e o Enter abriu a PRIMEIRA da lista inteira. O índice
        // andava no vetor original, que com filtro não é o que está na
        // tela.
        //
        // E não circula, pelo mesmo motivo do documento (ciclo 279):
        // numa lista longa, um `j` a mais que teleporta pro topo faz
        // perder o lugar sem aviso.
        "j" | "ArrowDown" => {
            e.andar_nas_paginas(true);
            None
        }
        "k" | "ArrowUp" => {
            e.andar_nas_paginas(false);
            None
        }
        // Numa árvore, direita/esquerda abrem e fecham — é o que se
        // espera de pasta em qualquer lugar (ciclo 299).
        "l" | "ArrowRight" | "h" | "ArrowLeft" => {
            e.dobrar_pasta();
            None
        }
        "Enter" => {
            // Enter numa PASTA abre ou fecha; numa página, abre a
            // página. A mesma tecla, o que faz sentido pro que está sob
            // o cursor.
            if e.dobrar_pasta() {
                return None;
            }
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
            e.busca_em = Foco::Conteudo;
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
        // Mudar de NÍVEL é só destas teclas (ciclo 302).
        //
        // `h`/`l` e as setas laterais saíram daqui: elas andam entre
        // irmãos num galho em linha, e não fazem nada num galho em
        // coluna. Misturar "andar" com "descer" na mesma tecla é a
        // ambiguidade que os ciclos 279 a 281 tiraram do caminho.
        "Enter" => Passo::Entrar,
        "Escape" | "Backspace" => Passo::Sair,
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
    // O fundo da TELA INTEIRA, da cor da janela (ciclo 302).
    //
    // Sem isto o terminal mostra o fundo dele por baixo — que pode ser
    // qualquer coisa, inclusive uma imagem. As cores do app são
    // escolhidas contra o `--bg-base`, e sobre outro fundo o contraste
    // que a janela garante deixa de valer.
    f.render_widget(
        ratatui::widgets::Block::default().style(e.tema.estilo(Realce::Fundo)),
        f.area(),
    );
    let colunas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(f.area());

    // A altura útil exclui a borda de cima e a de baixo.
    e.altura = colunas[1].height.saturating_sub(2) as usize;
    e.seguir_cursor();

    let paginas: Vec<Line> = e
        .sidebar_visivel()
        .into_iter()
        .enumerate()
        .map(|(i, l)| {
            let selecionada = i == e.linha_sidebar;
            let estilo = if selecionada {
                realce(e.foco == Foco::Paginas, &e.tema)
            } else {
                e.tema.estilo(Realce::Texto)
            };
            let recuo = "  ".repeat(l.nivel);
            match &l.item {
                // A pasta leva a seta do estado, como no outline: `▸`
                // fechada, `▾` aberta. Aqui a seta é informação — quem
                // olha precisa saber que tem coisa dentro.
                Item::Pasta { caminho, nome } => {
                    let seta = if e.pastas_fechadas.contains(caminho) {
                        "▸"
                    } else {
                        "▾"
                    };
                    Line::from(vec![
                        Span::styled(recuo, Style::default()),
                        Span::styled(
                            format!("{seta} {nome}/"),
                            if selecionada {
                                estilo
                            } else {
                                e.tema.estilo(Realce::Parte)
                            },
                        ),
                    ])
                }
                Item::Pagina { titulo, .. } => Line::from(vec![
                    Span::styled(recuo, Style::default()),
                    Span::styled(titulo.clone(), estilo),
                ]),
            }
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
    // A unidade em foco é desenhada como CARTÃO: caixa em volta dela e
    // dos filhos (ciclo 294).
    //
    // Uma régua à esquerda não bastava — a pessoa mostrou o desenho que
    // queria, e o que ele tem é MOLDURA. Faz sentido: numa árvore, a
    // pergunta não é só "que linha", é "até onde vai o que estou
    // olhando". Um `##` com nove itens embaixo é uma coisa só, e a
    // moldura é quem diz isso.
    let no_foco = e.foco == Foco::Conteudo;
    // A moldura é desenhada por REGIÃO (ciclo 296).
    //
    // Uma região é um trecho contíguo de linhas que pertencem à mesma
    // caixa: a subárvore em foco, ou um embed. O embed desenha a sua
    // SEMPRE, com cor própria; o foco desenha por cima, porque quando
    // as duas coincidem quem importa é onde a pessoa está.
    //
    // Antes eu desenhava a caixa do foco aqui e a do embed na mão, lá no
    // `tela` — dois vocabulários de caixa na mesma tela, e o embed
    // ficava com meia moldura.
    // A região é identificada pelo DONO, não só pela cor (ciclo 298).
    //
    // Agrupar por cor fundia embeds vizinhos: seis embeds seguidos
    // viravam uma caixa magenta só, porque todos pediam a mesma cor. O
    // dono distingue — dois embeds lado a lado são duas caixas.
    let regiao = |l: &crate::tela::Linha| -> Option<(Vec<usize>, Realce)> {
        // Dentro de um embed, a caixa é DELE — mesmo com o cursor lá
        // dentro (ciclo 298).
        //
        // Deixar o foco abrir caixa própria partia o cartão em dois: a
        // moldura do fluxo fechava antes da linha do cursor e outra
        // abria depois. Um cartão partido no meio não é cartão.
        //
        // Quem marca o foco lá dentro é a LATERAL da linha, que acende.
        if let Some(d) = l.dono_embed.clone() {
            return Some((d, Realce::Embed));
        }
        if no_foco
            && (l.mostra(&e.cursor)
                || (l.caminho.len() > e.cursor.len() && l.caminho.starts_with(&e.cursor)))
        {
            return Some((e.cursor.clone(), Realce::BordaUnidade));
        }
        None
    };

    // Monta as linhas a partir de um topo, e diz se o cursor coube.
    //
    // As molduras ocupam linha e a janela é calculada em linhas REAIS —
    // então caber "na conta" não garante caber na tela. Em vez de
    // estimar, monta e confere: se o cursor não coube, monta de novo a
    // partir dele (ciclo 298).
    //
    // Achado usando: `G` ia pro fim da página e a tela ficava parada no
    // meio.
    let montar = |topo: usize| -> (Vec<Line>, bool) {
        let mut fora: Vec<Line> = Vec::with_capacity(e.altura + 2);
        let mut atual: Option<(Vec<usize>, Realce)> = None;
        let mut achou = false;
        'linhas: for l in linhas_visiveis.iter().skip(topo) {
            if fora.len() >= e.altura {
                break;
            }
            if matches!(l.tipo, Tipo::Lista | Tipo::ListaOrdenada)
                && !e.dobrados.contains(&l.caminho)
            {
                continue;
            }
            // Embed ABERTO que tem título não desenha o próprio rótulo
            // (ciclo 302): `[fluxo]` em cima de `PROPOSTA: Em revisão`
            // diz duas vezes, e o título diz melhor. Fechado ele volta,
            // porque aí o rótulo é a única identidade que sobra.
            if matches!(l.tipo, Tipo::Embed(_))
                && !l.resumo.is_empty()
                && l.resumo != l.texto
                && !e.dobrados.contains(&l.caminho)
                && linhas_visiveis
                    .iter()
                    .any(|f| f.texto == l.resumo && f.caminho.starts_with(&l.caminho))
            {
                continue;
            }
            let quer = regiao(l);
            if quer != atual {
                if let Some((_, velha)) = &atual {
                    fora.push(moldura(false, largura_util, *velha, &e.tema));
                }
                if let Some((_, nova)) = &quer {
                    if fora.len() < e.altura {
                        fora.push(moldura(true, largura_util, *nova, &e.tema));
                    }
                }
                atual = quer;
            }
            if fora.len() >= e.altura {
                break;
            }
            if !l.enfeite && l.mostra(&e.cursor) {
                achou = true;
            }
            let largura_conteudo = if atual.is_some() {
                largura_util - 2
            } else {
                largura_util
            };
            // Uma fileira rende uma FAIXA de três linhas por grupo de
            // botões, porque botão tem borda fechada. Todo o resto
            // rende um grupo de uma linha só.
            let desenhadas = if l.segmentos.is_empty() {
                vec![vec![linha_estilizada(
                    // Sem fundo no texto (ciclo 295): quem diz onde se
                    // está é a moldura. Reintroduzi isto sem querer ao
                    // reescrever o laço, e o teste do 295 pegou.
                    l,
                    false,
                    e.dobrados.contains(&l.caminho),
                    &e.tema,
                    largura_conteudo,
                )]]
            } else {
                linhas_de_fileira(l, &e.tema, largura_conteudo, Some(&e.cursor))
            };
            // A lateral acende na linha do cursor: é como o foco se
            // marca dentro de um cartão que não é dele.
            let cor_lateral = match &atual {
                Some((_, r)) if no_foco && !l.enfeite && l.mostra(&e.cursor) => {
                    let _ = r;
                    Some(Realce::BordaUnidade)
                }
                Some((_, r)) => Some(*r),
                None => None,
            };
            // O grupo é INDIVISÍVEL: ou cabem as três linhas da faixa,
            // ou ela fica pro próximo quadro. Meia borda de botão no
            // rodapé lê como erro de desenho.
            //
            // Descartar a fileira INTEIRA quando uma faixa não cabe é o
            // que eu tinha feito antes, e num painel estreito a trilha
            // do fluxo sumia por completo — inclusive a etapa atual,
            // que é a única que precisa estar na tela.
            for grupo in desenhadas {
                if fora.len() + grupo.len() > e.altura {
                    break 'linhas;
                }
                for linha in grupo {
                    fora.push(match cor_lateral {
                        Some(r) => emoldurar(linha, largura_util, r, &e.tema),
                        None => linha,
                    });
                }
            }
        }
        if let Some((_, velha)) = atual {
            if fora.len() < e.altura {
                fora.push(moldura(false, largura_util, velha, &e.tema));
            }
        }
        (fora, achou)
    };

    let (mut visiveis, coube) = montar(e.topo);
    // O cursor ficou de fora da tela desenhada.
    //
    // Começar a janela NELE resolve e estraga: ele aparece na primeira
    // linha e o resto fica em branco — foi o que a tela mostrou depois
    // de um `G`. O certo é recuar o máximo que ainda o mantenha visível,
    // que é o que enche a tela acima dele.
    //
    // O recuo é medido montando, não estimando: cada moldura ocupa linha
    // e não dá pra saber quantas cabem sem desenhar. Custa até `altura`
    // montagens, e só quando há correção a fazer.
    let novo_topo = (!coube && no_foco)
        .then(|| linhas_visiveis.iter().position(|l| l.mostra(&e.cursor)))
        .flatten()
        .map(|i| {
            let mut melhor = i;
            for t in (0..i).rev() {
                if montar(t).1 {
                    melhor = t;
                } else {
                    break;
                }
            }
            melhor
        });
    if let Some(i) = novo_topo {
        visiveis = montar(i).0;
    }

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
    // Guardado só agora: `linhas_visiveis` empresta `e` até aqui, e o
    // topo corrigido precisa sobreviver pro próximo quadro — senão a
    // tela "conserta" e desconserta a cada tecla.
    if let Some(i) = novo_topo {
        e.topo = i;
    }
}

/// O respiro entre dois botões vizinhos.
///
/// Um só: o contorno de meio-bloco já deixa meia célula de fundo de
/// cada lado, então dois espaços viravam três.
const VAO: usize = 1;

/// A largura que um botão ocupa: texto, um espaço de cada lado e as
/// duas meias-células do contorno.
fn largura_do_botao(texto: &str) -> usize {
    texto.chars().count() + 4
}

/// Uma fileira desenhada como GRADE DE BOTÕES (ciclo 302).
///
/// Três linhas por faixa de botões, porque botão aqui tem a borda
/// fechada dos quatro lados e superfície preenchida — é o padrão do
/// botão na TUI, pedido depois de ver `[ texto ]` na tela. Colchete é
/// convenção de terminal; retângulo é o que a pessoa desenhou, e é o
/// que a janela mostra.
///
/// Quando a fileira não cabe na largura, ela QUEBRA em faixas em vez
/// de ser cortada na borda. Seis etapas dão 89 colunas, e o painel de
/// conteúdo raramente tem isso: antes o fim da trilha simplesmente
/// sumia — e a etapa que sumia podia ser a atual.
fn linhas_de_fileira(
    l: &crate::tela::Linha,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
) -> Vec<Vec<Line<'static>>> {
    let recuo = "  ".repeat(l.nivel);
    let disponivel = largura.saturating_sub(recuo.len()).max(4);

    // Empacota em faixas que cabem. Um botão sozinho maior que a
    // largura fica na sua própria faixa e é cortado — não há o que
    // fazer, e cortar um é melhor que cortar todos depois dele.
    let mut faixas: Vec<Vec<&crate::tela::Segmento>> = Vec::new();
    let mut faixa: Vec<&crate::tela::Segmento> = Vec::new();
    let mut usado = 0usize;
    for seg in &l.segmentos {
        let largura_seg = largura_do_botao(&seg.texto);
        let custo = if faixa.is_empty() { largura_seg } else { largura_seg + VAO };
        if !faixa.is_empty() && usado + custo > disponivel {
            faixas.push(std::mem::take(&mut faixa));
            usado = largura_seg;
        } else {
            usado += custo;
        }
        faixa.push(seg);
    }
    if !faixa.is_empty() {
        faixas.push(faixa);
    }

    let mut fora = Vec::with_capacity(faixas.len());
    for faixa in faixas {
        let mut topo = vec![Span::styled(recuo.clone(), Style::default())];
        let mut meio = vec![Span::styled(recuo.clone(), Style::default())];
        let mut base = vec![Span::styled(recuo.clone(), Style::default())];
        for (i, seg) in faixa.iter().enumerate() {
            if i > 0 {
                let vao = " ".repeat(VAO);
                topo.push(Span::styled(vao.clone(), Style::default()));
                meio.push(Span::styled(vao.clone(), Style::default()));
                base.push(Span::styled(vao, Style::default()));
            }
            // O que está sob o cursor acende (ciclo 298): é o "cursor
            // como coluna dentro da linha" — sem ele, entrar num botão
            // move o cursor pra um lugar sem linha e o Enter parece
            // não fazer nada.
            //
            // Fora do cursor, cada botão pela SUA cor: a etapa atual
            // de um fluxo não pode sair igual às outras cinco.
            let aceso = cursor.is_some_and(|c| c == seg.caminho.as_slice());
            let papel = if aceso {
                Realce::Cursor
            } else {
                papel_da_parte(&seg.nome)
            };
            let contorno = tema.contorno_do_botao(papel);
            let largura_miolo = seg.texto.chars().count() + 2;
            // Contorno de MEIO-BLOCO: cada célula pinta a metade que
            // olha pra dentro, e deixa a de fora com o fundo da tela.
            // `▗▄▖` em cima, `▝▀▘` embaixo, `▐` e `▌` nos lados — os
            // cantos são quadrantes, senão a quina fica quadrada e o
            // botão volta a parecer uma caixa cheia.
            topo.push(Span::styled(
                format!("▗{}▖", "▄".repeat(largura_miolo)),
                contorno,
            ));
            meio.push(Span::styled("▐", contorno));
            meio.push(Span::styled(
                format!(" {} ", seg.texto),
                tema.miolo_do_botao(papel),
            ));
            meio.push(Span::styled("▌", contorno));
            base.push(Span::styled(
                format!("▝{}▘", "▀".repeat(largura_miolo)),
                contorno,
            ));
        }
        fora.push(vec![Line::from(topo), Line::from(meio), Line::from(base)]);
    }
    fora
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
    if e.busca_em != painel || (!e.barra_aberta && e.busca.is_empty()) {
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
    // Onde o cursor está, o PAI arruma os filhos como? (ciclo 297)
    //
    // Num galho em linha os irmãos estão lado a lado, e aí quem anda
    // entre eles é `h`/`l`; `j`/`k` saem do galho. Num galho em coluna é
    // o contrário, que é o caso de sempre.
    //
    // Quem responde é o modelo. Antes isso teria que ser geometria —
    // adivinhar pela tela quem está ao lado de quem — e geometria não
    // existe num renderizador que ainda não desenhou.
    let em_fileira = e
        .cursor
        .len()
        .checked_sub(1)
        .and_then(|n| e.arvore.em(&e.cursor[..n]))
        .is_some_and(|pai| pai.tipo.arranjo() == Arranjo::Linha);
    match mov {
        // Num galho em linha, `j`/`k` não têm pra onde ir: os irmãos
        // estão lado a lado. Ficam parados.
        //
        // Na primeira versão (ciclo 297) eles SAÍAM do galho, e isso
        // estava errado: mudar de nível é do Enter, do Backspace e do
        // Escape — só. Misturar "andar" com "mudar de nível" na mesma
        // tecla é a ambiguidade que os ciclos 279 a 281 tiraram do
        // caminho, e reintroduzi-la num caso especial é como ela
        // voltaria (ciclo 302).
        Movimento::Baixo | Movimento::Cima if em_fileira => {}
        Movimento::Direita if em_fileira => repetir(e, Passo::Proximo, vezes),
        Movimento::Esquerda if em_fileira => repetir(e, Passo::Anterior, vezes),
        Movimento::Baixo => repetir(e, Passo::Proximo, vezes),
        Movimento::Cima => repetir(e, Passo::Anterior, vezes),
        // `h`/`l` FORA de fileira também não mudam de nível, pelo mesmo
        // motivo. Num galho em coluna eles não têm o que fazer.
        Movimento::Direita | Movimento::Esquerda => {}
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
    let Some(atual) = visiveis.iter().position(|l| l.mostra(&e.cursor)) else {
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

/// Põe as laterais da moldura numa linha de conteúdo.
///
/// O retângulo precisa das QUATRO bordas pra ler como retângulo: sem as
/// laterais ele é um par de traços soltos, e o texto parece escapar por
/// eles.
///
/// O conteúdo é cortado no que cabe, com `…`. Deixar transbordar seria
/// pior do que cortar: o `│` da direita sumiria e a moldura quebraria
/// justo na linha mais longa.
fn emoldurar<'a>(linha: Line<'a>, largura: usize, cor: Realce, tema: &Tema) -> Line<'a> {
    let estilo = tema.estilo(cor);
    let util = largura.saturating_sub(2);
    let mut spans: Vec<Span<'a>> = vec![Span::styled("│", estilo)];
    let mut usado = 0usize;
    for s in linha.spans {
        let n = s.content.chars().count();
        if usado + n <= util {
            usado += n;
            spans.push(s);
            continue;
        }
        // Este trecho não cabe inteiro: entra o que couber, menos um
        // caractere pro `…`.
        let cabe = util.saturating_sub(usado + 1);
        if cabe > 0 {
            let corte: String = s.content.chars().take(cabe).collect();
            spans.push(Span::styled(corte, s.style));
            usado += cabe;
        }
        if usado < util {
            spans.push(Span::styled("…", s.style));
            usado += 1;
        }
        break;
    }
    if usado < util {
        spans.push(Span::styled(" ".repeat(util - usado), Style::default()));
    }
    spans.push(Span::styled("│", estilo));
    Line::from(spans)
}

/// O topo ou o fundo da moldura da unidade em foco.
fn moldura<'a>(topo: bool, largura: usize, cor: Realce, tema: &Tema) -> Line<'a> {
    let (canto, fim) = if topo { ("┌", "┐") } else { ("└", "┘") };
    let meio = "─".repeat(largura.saturating_sub(2));
    Line::from(Span::styled(
        format!("{canto}{meio}{fim}"),
        tema.estilo(cor),
    ))
}

/// A marca da linha, com a seta de dobra quando o nível pode dobrar.
///
/// `▾`/`▸` são a afordância universal de outline, e resolvem uma coisa
/// que faltava: nada na tela dizia que um nível PODE ser fechado.
///
/// A lista troca o `·` pela seta; o embed mantém o nome dele e ganha a
/// seta na frente, porque `[kanban]` é identidade e não se troca.
fn marca_com_dobra(l: &crate::tela::Linha, dobrada: bool) -> String {
    // Unidade comum NÃO anuncia dobra (ciclo 294).
    //
    // Eu tinha posto `▾`/`▸` em todo nível, e a pessoa apontou o erro:
    // não é assim que a janela funciona. Lá uma lista de markdown é uma
    // lista — quem tem botão de recolher é o callout, porque ele É um
    // objeto. Anunciar dobra em tudo ensina uma coisa que a janela não
    // faz.
    //
    // Fechado, a seta aparece: aí ela não é anúncio, é ESTADO — a
    // pessoa precisa saber que tem coisa escondida ali.
    if !dobrada {
        return l.marca.clone();
    }
    match l.tipo {
        Tipo::Lista | Tipo::ListaOrdenada => "▸".to_string(),
        _ => format!("▸ {}", l.marca),
    }
}

/// O papel de um bloco, pelo tipo dele.
fn papel_do_bloco(tipo: &Tipo) -> Realce {
    match tipo {
        Tipo::Titulo(n) => Realce::Titulo(*n),
        Tipo::Citacao => Realce::Citacao,
        Tipo::Codigo(_) => Realce::Codigo,
        Tipo::Embed(_) => Realce::Embed,
        // A parte se pinta pelo NOME (ciclo 302). Tudo roxo dizia só
        // "isto é embed"; um cartão de fluxo tem coisas de naturezas
        // diferentes, e é o embed que sabe o nome de cada uma.
        Tipo::Parte { nome, .. } => papel_da_parte(nome),
        _ => Realce::Texto,
    }
}

/// O papel de uma parte, pelo nome que o embed deu a ela.
fn papel_da_parte(nome: &str) -> Realce {
    match nome {
        "titulo" => Realce::TituloCartao,
        "etapa" => Realce::Etapa,
        "etapa-atual" => Realce::EtapaAtual,
        "acao" => Realce::Acao,
        "dica" => Realce::Dica,
        "transicao" => Realce::Transicao,
        "transicao-principal" => Realce::TransicaoPrincipal,
        _ => Realce::Parte,
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

    /// As células de uma linha da tela que têm um modificador.
    fn com_modificador(e: &mut Estado, m: Modifier) -> Vec<String> {
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
    fn o_conteudo_em_foco_nao_ganha_fundo(){
        // Quem diz onde se está é a MOLDURA (ciclo 295). Fundo mais
        // moldura são dois destaques competindo, e o texto colorido por
        // cima do fundo perde a cor que o tipo dele tinha.
        let mut e = Estado::novo(paginas(), analisar("# Título\n\ntexto\n"));
        e.foco = Foco::Conteudo;
        let cursor = e.tema.estilo(Realce::Cursor).bg.unwrap();
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        // A ÚNICA coisa com fundo de cursor é a página selecionada na
        // lista, que não tem moldura pra marcar.
        let com_fundo: Vec<String> = (0..buf.area.height)
            .filter_map(|y| {
                let t: String = (0..buf.area.width)
                    .filter(|x| buf[(*x, y)].style().bg == Some(cursor))
                    .map(|x| buf[(x, y)].symbol().to_string())
                    .collect();
                (!t.trim().is_empty()).then(|| t.trim().to_string())
            })
            .collect();
        assert!(
            !com_fundo.iter().any(|l| l.contains("Título")),
            "o conteúdo em foco ganhou fundo: {com_fundo:?}"
        );
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
        // E a seta NÃO aparece: no GUI uma lista de markdown não é
        // colapsável, e anunciar dobra em tudo ensina uma coisa que a
        // janela não faz (ciclo 294).
        assert!(
            !tudo.contains('▾'),
            "unidade comum anunciando dobra:\n{tudo}"
        );
    }

    /// Uma página com um embed de ações de dois botões.
    fn com_acoes() -> Unidade {
        analisar("{{ type: \"actions\" }}\nbuttons:\n- label: Abrir\n  action: open-page\n- label: Buscar\n  action: run-search\n{{ /actions }}\n")
    }

    #[test]
    fn a_busca_de_pagina_nao_filtra_o_conteudo() {
        // Achado usando: busquei uma página, abri, e o conteúdo veio
        // vazio — o termo da busca de PÁGINAS estava filtrando as linhas
        // também. Filtro é do painel onde foi digitado.
        let mut e = Estado::novo(paginas(), analisar("alfa\n\nbeta\n"));
        assert_eq!(e.foco, Foco::Paginas);
        tecla(&mut e, "/");
        tecla(&mut e, "g"); // casa com "gama", não com o conteúdo
        assert_eq!(e.paginas_visiveis().len(), 1);
        assert_eq!(e.visiveis().len(), 2, "a busca de página escondeu o conteúdo");
    }

    #[test]
    fn buscar_uma_pagina_seleciona_a_que_sobrou() {
        // Achado usando: busquei "incio", a lista filtrou pra uma página,
        // e o Enter abriu a PRIMEIRA da lista inteira — o índice andava
        // no vetor original, que com filtro não é o que está na tela.
        let mut e = estado();
        tecla(&mut e, "/");
        tecla(&mut e, "g"); // só "gama"
        assert_eq!(e.paginas_visiveis().len(), 1);
        let pedido = tecla(&mut e, "Enter");
        assert_eq!(pedido.as_deref(), Some("pages/gama.md"));
    }

    #[test]
    fn andar_na_lista_filtrada_nao_sai_do_filtro() {
        let mut e = estado();
        tecla(&mut e, "/");
        tecla(&mut e, "a"); // alfa, beta, gama — todos têm "a"
        tecla(&mut e, "Enter");
        tecla(&mut e, "j");
        let visiveis: Vec<usize> = e.paginas_visiveis().into_iter().map(|(i, _)| i).collect();
        assert!(visiveis.contains(&e.pagina), "a seleção saiu do filtro");
    }

    #[test]
    fn abrir_uma_pagina_descarta_a_busca() {
        // O termo era pra achar a página; mantê-lo filtraria o conteúdo
        // dela por acidente.
        let mut e = Estado::novo(paginas(), analisar("alfa\n"));
        tecla(&mut e, "/");
        tecla(&mut e, "b");
        e.abrir(analisar("outra coisa\n\nmais\n"));
        assert!(e.busca.is_empty());
        assert_eq!(e.visiveis().len(), 2);
    }

    #[test]
    fn g_maiusculo_mostra_o_fim_da_pagina() {
        // Achado usando: `G` ia pro fim e a tela ficava vazia. A janela
        // é contada em linhas REAIS e o desenho insere molduras, então
        // caber na conta não é caber na tela.
        // Com EMBEDS: cada um insere duas molduras, e é isso que faz a
        // conta em linhas reais divergir do que cabe na tela.
        let md: String = (0..8)
            .map(|i| {
                format!("linha {i}\n\n{{{{ type: \"callout\" }}}}\nvariant: info\nbody: |\n  corpo {i}\n{{{{ /callout }}}}\n\n")
            })
            .chain(std::iter::once("linha 39\n".to_string()))
            .collect();
        let mut e = Estado::novo(paginas(), analisar(&md));
        e.foco = Foco::Conteudo;
        let linhas = desenho(&mut e, 60, 12);
        tecla(&mut e, "G");
        let depois = desenho(&mut e, 60, 12);
        let tudo = depois.join("\n");
        assert!(
            tudo.contains("linha 39"),
            "o fim não apareceu:\n{tudo}\n(antes era:\n{})",
            linhas.join("\n")
        );
        // E a tela fica CHEIA: começar a janela no cursor o põe na
        // primeira linha e deixa o resto em branco, que foi o que
        // apareceu na tela depois de um `G`.
        let cheias = depois.iter().filter(|l| !l.trim().is_empty()).count();
        assert!(
            cheias >= 10,
            "a tela ficou vazia abaixo do cursor ({cheias} linhas):\n{tudo}"
        );
    }

    #[test]
    fn o_botao_sob_o_cursor_acende_na_fileira() {
        // O defeito que a pessoa mostrou: entrar num botão movia o
        // cursor pra um lugar SEM linha na tela, e o Enter parecia não
        // fazer nada. A linha é uma, os destinos são vários — o que
        // muda é qual segmento acende.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 0, 1]; // o segundo botão
        let cursor = e.tema.estilo(Realce::Cursor).bg.unwrap();
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        let aceso: String = (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .filter(|(x, y)| buf[(*x, *y)].style().bg == Some(cursor))
            .map(|(x, y)| buf[(x, y)].symbol().to_string())
            .collect();
        assert!(aceso.contains("Buscar"), "o botão do cursor não acendeu: {aceso:?}");
        assert!(!aceso.contains("Abrir"), "acendeu o botão errado também: {aceso:?}");
    }

    #[test]
    fn dois_embeds_vizinhos_sao_duas_caixas() {
        // O defeito da captura: seis embeds seguidos viravam UMA caixa
        // magenta, porque a região era identificada pela cor e todos
        // pediam a mesma. O dono distingue.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"callout\" }}\nvariant: info\nbody: |\n  um\n{{ /callout }}\n\n{{ type: \"callout\" }}\nvariant: info\nbody: |\n  dois\n{{ /callout }}\n"),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 14).join("\n");
        // Duas caixas de embed, mais a borda dos dois painéis.
        assert_eq!(
            tudo.matches('┌').count(),
            4,
            "os embeds vizinhos viraram uma caixa só:\n{tudo}"
        );
    }

    #[test]
    fn a_etapa_atual_sai_de_uma_cor_diferente_das_outras() {
        // Sem o nome da parte no segmento, todas as seis etapas saíam da
        // mesma cor — e a única que importa é a atual.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n"),
        );
        e.foco = Foco::Paginas;
        let atual = e.tema.estilo(Realce::EtapaAtual).fg.unwrap();
        let outra = e.tema.estilo(Realce::Etapa).fg.unwrap();
        assert_ne!(atual, outra, "as duas cores são iguais no tema");

        let mut term = Terminal::new(TestBackend::new(70, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();

        // Botão preenchido: a cor do papel é o FUNDO do miolo, e o
        // texto por cima é da cor do fundo da tela (ciclo 302).
        let pintado = |cor: Color| -> String {
            (0..buf.area.height)
                .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
                .filter(|(x, y)| {
                    let e = buf[(*x, *y)].style();
                    e.fg == Some(cor) || e.bg == Some(cor)
                })
                .map(|(x, y)| buf[(x, y)].symbol().to_string())
                .collect()
        };
        assert!(
            pintado(atual).contains("Aprovada"),
            "a etapa atual não saiu na cor dela: {:?}",
            pintado(atual)
        );
        assert!(
            pintado(outra).contains("Rascunho"),
            "as outras etapas não saíram na cor delas"
        );
    }

    #[test]
    fn o_fundo_da_tela_e_o_da_janela() {
        // Sem pintar o fundo, o terminal mostra o dele por baixo — que
        // pode ser qualquer coisa, inclusive uma imagem. As cores do app
        // são escolhidas contra o `--bg-base`.
        let mut e = estado();
        let fundo = e.tema.estilo(Realce::Fundo).bg.unwrap();
        let mut term = Terminal::new(TestBackend::new(40, 8)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();
        let com_fundo = (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .filter(|(x, y)| buf[(*x, *y)].style().bg == Some(fundo))
            .count();
        assert!(
            com_fundo > (buf.area.width as usize * buf.area.height as usize) / 2,
            "menos da metade da tela ficou com o fundo do app ({com_fundo} células)"
        );
    }

    #[test]
    fn numa_fileira_o_cursor_anda_com_h_e_l() {
        // O arranjo vem do MODELO (ciclo 297): num galho em linha os
        // irmãos estão lado a lado, e quem anda entre eles é h/l.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Conteudo;
        tecla(&mut e, "Enter"); // entra no embed → a fileira
        assert_eq!(e.cursor, vec![0, 0]);
        tecla(&mut e, "Enter"); // entra na fileira → o primeiro botão
        assert_eq!(e.cursor, vec![0, 0, 0]);

        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![0, 0, 1], "l não andou pro botão seguinte");
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0, 0], "h não voltou");
        // Na borda ele fica, como em qualquer nível.
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0, 0]);
    }

    #[test]
    fn numa_fileira_j_e_k_ficam_parados() {
        // Empilhar não existe ali, então `j` não tem pra onde ir.
        //
        // A primeira versão fazia ele SAIR do galho, e isso estava
        // errado: mudar de nível é do Enter, do Backspace e do Escape.
        // Misturar "andar" com "mudar de nível" na mesma tecla é a
        // ambiguidade que os ciclos 279 a 281 tiraram do caminho.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 0, 1];
        tecla(&mut e, "j");
        assert_eq!(e.cursor, vec![0, 0, 1], "j mudou de nível");
        tecla(&mut e, "k");
        assert_eq!(e.cursor, vec![0, 0, 1], "k mudou de nível");
    }

    #[test]
    fn so_enter_escape_e_backspace_mudam_de_nivel() {
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1];

        // `l` não desce mais.
        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![1], "`l` desceu de nível");

        tecla(&mut e, "Enter");
        assert_eq!(e.cursor, vec![1, 0], "Enter não desceu");

        // `h` não sobe.
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![1, 0], "`h` subiu de nível");

        tecla(&mut e, "Backspace");
        assert_eq!(e.cursor, vec![1], "Backspace não subiu");
        tecla(&mut e, "Enter");
        tecla(&mut e, "Escape");
        assert_eq!(e.cursor, vec![1], "Escape não subiu");
    }

    #[test]
    fn os_botoes_de_uma_fileira_ficam_lado_a_lado_em_caixas() {
        // Botão é retângulo de borda fechada e fundo preenchido — foi o
        // pedido depois de ver `[ Abrir ]` na tela. Colchete é convenção
        // de terminal; num cartão cheio de texto ele não se distingue de
        // uma citação.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 12).join("\n");
        assert!(
            tudo.contains("▗▄▄▄▄▄▄▄▖ ▗▄▄▄▄▄▄▄▄▖"),
            "os botões não ficaram lado a lado em caixas:\n{tudo}"
        );
        assert!(
            tudo.contains("▐ Abrir ▌ ▐ Buscar ▌"),
            "os rótulos não ficaram dentro das caixas:\n{tudo}"
        );
        assert!(
            tudo.contains("▝▀▀▀▀▀▀▀▘ ▝▀▀▀▀▀▀▀▀▘"),
            "as caixas dos botões não fecharam embaixo:\n{tudo}"
        );
    }

    #[test]
    fn o_contorno_do_botao_encosta_no_preenchimento() {
        // Três voltas até aqui, e as duas primeiras foram por tratar
        // célula como pixel: ou a cor tomava a célula inteira do traço
        // (botão gordo, contorno engolido), ou não tomava nada (faixa
        // escura entre o preenchimento e o traço).
        //
        // Meio-bloco é a terceira opção que eu tinha decidido que não
        // existia: a célula pinta a metade que olha PRA DENTRO. O vão
        // some e o botão encolhe meia célula de cada lado.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Paginas;
        let cor = e.tema.miolo_do_botao(Realce::Parte).bg.unwrap();
        assert_eq!(
            e.tema.contorno_do_botao(Realce::Parte).fg,
            Some(cor),
            "o traço do contorno não é da cor do preenchimento"
        );
        assert_eq!(
            e.tema.contorno_do_botao(Realce::Parte).bg,
            None,
            "a metade de fora do contorno ficou pintada — o botão incha"
        );

        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();
        // Onde a cor aparece, seja como traço ou como preenchimento.
        let com_cor = |y: u16| -> Vec<u16> {
            (0..buf.area.width)
                .filter(|x| {
                    let s = buf[(*x, y)].style();
                    s.fg == Some(cor) || s.bg == Some(cor)
                })
                .collect::<Vec<u16>>()
        };
        let faixa: Vec<Vec<u16>> = (0..buf.area.height)
            .map(com_cor)
            .filter(|xs| !xs.is_empty())
            .collect();
        assert_eq!(faixa.len(), 3, "esperava as três linhas de uma faixa");
        assert!(
            faixa[0] == faixa[1] && faixa[1] == faixa[2],
            "a cor não alcança as mesmas colunas nas três linhas — é vão: {faixa:?}"
        );

        // E o contorno é meio-bloco de verdade: com `─` e `│` a cor
        // voltaria a tomar a célula inteira, que foi a volta anterior.
        let simbolo = |x: u16, y: u16| buf[(x, y)].symbol().to_string();
        let (x0, y0) = (faixa[0][0], (0..buf.area.height).find(|y| !com_cor(*y).is_empty()).unwrap());
        assert_eq!(simbolo(x0, y0), "▗", "a quina do botão não é quadrante");
        assert_eq!(simbolo(x0 + 1, y0), "▄", "o topo do botão não é meio-bloco");
        assert_eq!(simbolo(x0, y0 + 1), "▐", "a lateral do botão não é meio-bloco");
    }

    #[test]
    fn o_texto_do_botao_contrasta_com_o_preenchimento() {
        // Texto da mesma cor do preenchimento é um botão sem rótulo.
        let e = Estado::novo(paginas(), com_acoes());
        let miolo = e.tema.miolo_do_botao(Realce::Parte);
        assert_ne!(
            miolo.fg, miolo.bg,
            "o texto do botão sumiu dentro do preenchimento"
        );
    }

    #[test]
    fn a_fileira_larga_quebra_em_grade_em_vez_de_sumir() {
        // Seis etapas dão 89 colunas, e o painel de conteúdo raramente
        // tem isso. Cortar na borda esconderia o fim da trilha — e o
        // que sumia podia ser a etapa atual.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n"),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 100, 24).join("\n");
        for etapa in ["Rascunho", "Em revisão", "Aprovada", "Bloqueada"] {
            assert!(tudo.contains(etapa), "`{etapa}` sumiu da trilha:\n{tudo}");
        }
    }

    #[test]
    fn a_unidade_em_foco_ganha_moldura() {
        // O desenho que a pessoa pediu: caixa em volta da unidade e dos
        // filhos dela. Numa árvore, "até onde vai o que estou olhando" é
        // parte da pergunta.
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n\ndepois\n"));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1]; // a lista
        let linhas = desenho(&mut e, 50, 12);
        // A moldura da unidade é a que não está na primeira linha (essa
        // é a borda dos painéis).
        let topo = linhas
            .iter()
            .enumerate()
            .skip(1)
            .find(|(_, l)| l.contains('┌'))
            .map(|(i, _)| i)
            .expect("faltou o topo");
        let fundo = linhas
            .iter()
            .enumerate()
            .skip(topo + 1)
            .find(|(_, l)| l.contains('┘'))
            .map(|(i, _)| i)
            .expect("faltou o fundo");
        assert!(topo < fundo, "a moldura saiu invertida");
        // Os itens ficam DENTRO.
        let dentro = &linhas[topo + 1..fundo];
        assert!(dentro.iter().any(|l| l.contains("um")), "{dentro:?}");
        assert!(dentro.iter().any(|l| l.contains("dois")), "{dentro:?}");
        // E o bloco de fora, não.
        assert!(!dentro.iter().any(|l| l.contains("depois")), "{dentro:?}");
    }

    #[test]
    fn sem_foco_no_conteudo_nao_ha_moldura() {
        // Os dois painéis já têm `┌` no cabeçalho — contar é o jeito de
        // distinguir a moldura da unidade das bordas de sempre.
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n"));
        e.foco = Foco::Paginas;
        let sem = desenho(&mut e, 50, 12).join("\n").matches('┌').count();
        e.foco = Foco::Conteudo;
        e.cursor = vec![1];
        let com = desenho(&mut e, 50, 12).join("\n").matches('┌').count();
        assert_eq!(sem, 2, "esperava só as bordas dos painéis");
        assert_eq!(com, 3, "a moldura da unidade não apareceu");
    }

    #[test]
    fn lista_aberta_nao_desenha_a_propria_linha() {
        // O `·` sozinho dentro da própria moldura era ruído: a caixa já
        // diz onde a lista começa e acaba. Na janela também não existe
        // "linha da lista".
        let mut e = Estado::novo(paginas(), analisar("antes\n\n- um\n- dois\n"));
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 50, 12).join("\n");
        assert!(!tudo.contains('·'), "a linha da lista apareceu:\n{tudo}");
        assert!(tudo.contains("- um"), "os itens sumiram:\n{tudo}");
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
