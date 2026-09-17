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

mod edicao;
pub use edicao::{AcaoDaPergunta, Pergunta, Registro};
use edicao::tecla_na_pergunta;

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
    /// O dia de hoje (`AAAA-MM-DD`), quando alguém o disse (ciclo 315).
    ///
    /// Sem ele o calendário é o que o arquivo diz — todo mês com evento,
    /// sem cabeçalho. Com ele, cada calendário vira a janela: um mês por
    /// vez, ancorado em hoje, e o dia de hoje marcado. Quem lê o relógio é
    /// o `main`; aqui só se guarda, pra teste nenhum depender da data.
    pub hoje: Option<String>,
    /// A âncora de cada calendário da página aberta, pelo caminho do
    /// embed — o mês que ele mostra. Ausente é "o mês de hoje".
    pub ancoras: std::collections::HashMap<Caminho, String>,
    /// Os eventos que as páginas do vault declaram (`date::`), pros
    /// calendários em modo vault (ciclo 317). Quem varre o vault é o
    /// `main`; vazio é "ninguém varreu".
    pub eventos_do_vault: Vec<anotadinho_core::embed::CalendarEntry>,
    /// O texto inteiro do arquivo da página aberta (ciclo 318).
    ///
    /// Sem ele a TUI é só leitura: editar um embed é trocar o trecho dele
    /// NESTE texto e gravar o resultado. `None` em teste que não liga a
    /// edição.
    pub texto_da_pagina: Option<String>,
    /// A versão do arquivo quando foi lido — a gravação só acontece se o
    /// arquivo ainda estiver nela (a mesma trava da janela).
    pub versao: Option<String>,
    /// O texto novo que o `main` deve gravar, depois de uma edição.
    pub gravacao: Option<String>,
    /// Um aviso pro rodapé: gravação recusada, embed só leitura.
    pub aviso: Option<String>,
    /// A pergunta aberta no rodapé, quando uma edição precisa de texto.
    pub pergunta: Option<Pergunta>,
    /// O que `yy`/`dd` guardaram pra `p` colar (ciclo 322).
    pub registro: Option<Registro>,
    /// Texto e cursor de antes de cada edição, pro `u` (ciclo 322).
    pub desfazer: Vec<(String, Caminho)>,
    /// O que o `u` desfez, pro `Ctrl+R`.
    pub refazer: Vec<(String, Caminho)>,
    /// A visão de cada calendário (ciclo 316). Ausente é Mês.
    pub visoes: std::collections::HashMap<Caminho, anotadinho_core::analise::Visao>,
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
            hoje: None,
            ancoras: std::collections::HashMap::new(),
            visoes: std::collections::HashMap::new(),
            eventos_do_vault: Vec::new(),
            texto_da_pagina: None,
            versao: None,
            gravacao: None,
            aviso: None,
            pergunta: None,
            registro: None,
            desfazer: Vec::new(),
            refazer: Vec::new(),
        }
    }

    /// Diz que dia é hoje, e ancora os calendários nele (ciclo 315).
    pub fn com_hoje(mut self, hoje: &str) -> Self {
        self.hoje = Some(hoje.to_string());
        self.ancorar_calendarios();
        self.cursor = tela::primeiro(&self.arvore).unwrap_or_default();
        self
    }

    /// Liga a edição: o texto do arquivo aberto e a versão dele.
    pub fn com_texto(mut self, texto: &str, versao: Option<String>) -> Self {
        self.texto_da_pagina = Some(texto.to_string());
        self.versao = versao;
        self
    }

    /// Troca a página aberta pelo arquivo `texto` (ciclo 318) — o caminho
    /// que o `main` usa, pra a edição saber o que gravar.
    pub fn abrir_texto(&mut self, texto: &str, versao: Option<String>) {
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(texto);
        self.abrir(anotadinho_core::analise::analisar(corpo));
        self.desfazer.clear();
        self.refazer.clear();
        self.texto_da_pagina = Some(texto.to_string());
        self.versao = versao;
    }

    /// Aplica um texto novo da MESMA página depois de uma edição, sem
    /// perder o lugar: cursor, âncoras, visões, dobras e rolagem ficam. A
    /// gravação fica pendente pro `main`.
    fn aplicar_edicao(&mut self, texto: String) {
        if let Some(antes) = self.texto_da_pagina.clone() {
            self.desfazer.push((antes, self.cursor.clone()));
            if self.desfazer.len() > 200 {
                self.desfazer.remove(0);
            }
        }
        self.refazer.clear();
        self.trocar_texto(texto);
    }

    /// Troca o texto da página sem mexer no histórico — o que o `u` usa.
    fn trocar_texto(&mut self, texto: String) {
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
        self.arvore = anotadinho_core::analise::analisar(corpo);
        self.ancorar_calendarios();
        self.linhas = tela::linhas(&self.arvore);
        while !self.cursor.is_empty() && self.arvore.em(&self.cursor).is_none() {
            self.cursor.pop();
        }
        if self.cursor.is_empty() {
            self.cursor = tela::primeiro(&self.arvore).unwrap_or_default();
        }
        self.texto_da_pagina = Some(texto.clone());
        self.gravacao = Some(texto);
    }

    /// Entrega os eventos do vault e remonta os calendários em modo vault.
    pub fn com_eventos_do_vault(mut self, eventos: Vec<anotadinho_core::embed::CalendarEntry>) -> Self {
        self.eventos_do_vault = eventos;
        self.ancorar_calendarios();
        self
    }

    /// Refaz a árvore de cada calendário da página na âncora dele.
    ///
    /// O conteúdo vem da FONTE do embed (o markdown original, que
    /// `analisar` guarda), passado pela mesma `partes_do_calendario` que
    /// monta a árvore sem âncora — então a grade ancorada e a do CLI são o
    /// mesmo código, com um mês a menos.
    fn ancorar_calendarios(&mut self) {
        let Some(hoje) = self.hoje.clone() else { return };
        for i in 0..self.arvore.filhos.len() {
            let u = &self.arvore.filhos[i];
            if !matches!(&u.tipo, Tipo::Embed(n) if n == "calendar") {
                continue;
            }
            let Some(mut dados) = u.fonte.as_deref().and_then(dados_do_calendario) else { continue };
            // Modo vault: os eventos são as páginas com data, não o que
            // está escrito no embed — como a janela faz.
            if dados.mode == anotadinho_core::embed::CalendarSource::Vault {
                dados.entries = self.eventos_do_vault.clone();
            }
            let ancora = self.ancoras.get(&vec![i]).cloned().unwrap_or_else(|| hoje.clone());
            let visao = self.visoes.get(&vec![i]).copied().unwrap_or_default();
            self.arvore.filhos[i].filhos = anotadinho_core::analise::partes_do_calendario_na_visao(
                &dados,
                visao,
                Some(&ancora),
                Some(&hoje),
            );
        }
        self.linhas = tela::linhas(&self.arvore);
        if self.arvore.em(&self.cursor).is_none() {
            self.cursor = tela::primeiro(&self.arvore).unwrap_or_default();
        }
    }

    /// Muda a âncora de um calendário e refaz a árvore dele.
    fn ancorar(&mut self, embed: &[usize], data: String) {
        self.ancoras.insert(embed.to_vec(), data);
        self.ancorar_calendarios();
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
        // Outra página, outros calendários: âncoras voltam pra hoje.
        self.ancoras.clear();
        self.visoes.clear();
        self.ancorar_calendarios();
        self.cursor = tela::primeiro(&self.arvore).unwrap_or_default();
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
    // O aviso vale até a próxima tecla.
    e.aviso = None;
    // A pergunta de uma edição, como a busca, recebe tudo enquanto está
    // aberta (ciclo 318).
    if e.pergunta.is_some() {
        tecla_na_pergunta(e, tecla);
        return None;
    }
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
            // Enter num evento do vault abre a página dele (ciclo 317), o
            // que o clique faz na janela.
            if tecla == "Enter" {
                if let Some(pagina) = pagina_do_cursor(e) {
                    if let Some(i) = e.paginas.iter().position(|p| p.path == pagina) {
                        e.pagina = i;
                    }
                    return Some(pagina);
                }
            }
            tecla_no_conteudo(e, tecla);
            None
        }
    }
}

/// A página pra onde a parte sob o cursor aponta, se aponta.
fn pagina_do_cursor(e: &Estado) -> Option<String> {
    e.arvore
        .em(&e.cursor)?
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "pagina"))
        .map(|f| f.texto.clone())
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
    if tecla_do_calendario(e, tecla) {
        return;
    }
    // `Ctrl+R`, `Ctrl+A`, `Ctrl+X`: o `main` manda com o prefixo.
    let (tecla, ctrl) = match tecla.strip_prefix("Ctrl+") {
        Some(t) => (t, true),
        None => (tecla, false),
    };
    // A gramática do vim primeiro (ciclo 291).
    //
    // Ela mora no núcleo desde o ciclo 285 — contagem, operador,
    // movimento, `gg`/`G` — e ninguém a consultava. É ela que dá `10j` e
    // `G` sem uma linha de lógica nova aqui: a TUI só traduz o comando
    // fechado em passos de navegação.
    match vim::tecla_normal(&mut e.vim, tecla, ctrl) {
        vim::Passo::Aguardando => return,
        // A gramática já mapeia `/` pra busca desde o ciclo 254 — mais
        // uma coisa que estava escrita e não era consultada.
        vim::Passo::Pronto(vim::Comando::Busca) => {
            e.busca.clear();
            e.busca_em = Foco::Conteudo;
            e.barra_aberta = true;
            return;
        }
        // Editar é comando de vim sobre o item sob o cursor (ciclo 322):
        // `o`, `cc`, `dd`, `yy`, `p`, `>>`, `Ctrl+A`, `u`…
        vim::Passo::Pronto(c) => {
            if !vim::edicao_de(&c).is_some_and(|ed| edicao::editar_item(e, ed)) {
                comando_de_vim(e, c);
            }
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
            // Ciclo 304: a moldura pinta pelo TIPO do embed, não mais
            // uma cor só pra todos — `embed_dono` é o nome (`"kanban"`,
            // `"callout"`…) que `encaixotar_embeds` gravou junto.
            let papel = match l.embed_dono.as_deref() {
                // O callout pinta pela VARIANTE, não pelo tipo (ciclo
                // 307): a caixa de um `warning` e a de um `info` são
                // cores diferentes, como na janela.
                Some("callout") => papel_do_callout(&e.arvore, &d),
                outro => papel_do_embed(outro.unwrap_or("")),
            };
            return Some((d, papel));
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
            // Embed ABERTO não desenha o próprio rótulo (ciclo 309).
            //
            // O ciclo 302 já tirava `[fluxo]` quando havia título. Agora
            // que todo embed é caixa preenchida na cor do tipo (308), o
            // `[kanban]` em cima do conteúdo diz o que a caixa já diz — e
            // na janela não há rótulo nenhum. Fechado ele volta, porque aí
            // o rótulo é a única identidade que sobra; e um embed sem
            // nenhuma linha de conteúdo também o mantém, senão não haveria
            // caixa pra ver.
            if matches!(l.tipo, Tipo::Embed(_))
                && !e.dobrados.contains(&l.caminho)
                && linhas_visiveis
                    .iter()
                    .any(|f| f.caminho != l.caminho && f.dono_embed.as_ref() == Some(&l.caminho))
            {
                // O cursor NO embed continua visível: a caixa inteira é
                // quem o mostra (a lateral acende, logo abaixo).
                if l.mostra(&e.cursor) {
                    achou = true;
                }
                continue;
            }
            let quer = regiao(l);
            if quer != atual {
                if let Some((dono, velha)) = &atual {
                    if let Some(d) = linha_de_detalhe(e, dono) {
                        if fora.len() < e.altura {
                            fora.push(emoldurar(d, largura_util, *velha, *velha, &e.tema));
                        }
                    }
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
            // botões, porque botão tem borda fechada. Um cartão ou uma
            // miniatura são a MESMA faixa de três linhas, sozinhos — não
            // ficam lado a lado, cada um é dono da própria linha (ciclo
            // 304). A fileira de uma TABELA foge dessa regra: célula
            // não é botão, é grade — uma linha só, alinhada em coluna
            // (ciclo 305). Todo o resto rende um grupo de uma linha só.
            let nome_da_parte = match &l.tipo {
                Tipo::Parte { nome, .. } => nome.as_str(),
                _ => "",
            };
            let no_calendario = l.embed_dono.as_deref() == Some("calendar");
            let no_callout = l.embed_dono.as_deref() == Some("callout");
            let desenhadas = if l.embed_dono.as_deref() == Some("timeline") && nome_da_parte == "eixo" {
                vec![linhas_do_eixo(l, &e.tema, largura_conteudo)]
            } else if matches!(&l.tipo, Tipo::Embed(n) if n == "callout") {
                // O rótulo `[callout]` (callout sem título, ou dobrado)
                // veste a cor da VARIANTE, não a genérica do tipo — senão
                // um `error` fechado se anunciaria em âmbar (ciclo 307).
                let generica = e.tema.estilo(Realce::EmbedCallout).fg;
                let da_variante = e.tema.estilo(papel_do_callout(&e.arvore, &l.caminho)).fg;
                let linha = linha_estilizada(
                    l,
                    false,
                    e.dobrados.contains(&l.caminho),
                    &e.tema,
                    largura_conteudo,
                );
                let spans = linha
                    .spans
                    .into_iter()
                    .map(|sp| {
                        let estilo = if sp.style.fg == generica && generica.is_some() {
                            Style { fg: da_variante, ..sp.style }
                        } else {
                            sp.style
                        };
                        Span::styled(sp.content, estilo)
                    })
                    .collect::<Vec<_>>();
                vec![vec![Line::from(spans)]]
            } else if no_callout && nome_da_parte == "titulo" {
                let dono = l.dono_embed.as_deref().unwrap_or(&[]);
                vec![vec![linha_do_titulo_do_callout(
                    l,
                    &variante_do_callout(&e.arvore, dono),
                    papel_do_callout(&e.arvore, dono),
                    &e.tema,
                )]]
            } else if no_calendario && nome_da_parte == "cabecalho" {
                let visao = e
                    .visoes
                    .get(l.dono_embed.as_deref().unwrap_or(&[]))
                    .copied()
                    .unwrap_or_default();
                vec![vec![linha_do_cabecalho_do_calendario(l, visao, &e.tema, largura_conteudo)]]
            } else if no_calendario && nome_da_parte == "agenda" && !e.dobrados.contains(&l.caminho) {
                vec![vec![Line::from(vec![
                    Span::styled("  ".repeat(l.nivel), Style::default()),
                    Span::styled(l.texto.clone(), e.tema.estilo(Realce::TituloCartao)),
                ])]]
            } else if no_calendario && nome_da_parte.starts_with("compromisso") {
                vec![vec![linha_do_compromisso(l, &e.arvore, &e.tema, largura_conteudo, Some(&e.cursor))]]
            } else if no_calendario && nome_da_parte == "nada" {
                vec![vec![Line::from(vec![
                    Span::styled("  ".repeat(l.nivel), Style::default()),
                    Span::styled(l.texto.clone(), e.tema.estilo(Realce::Dica)),
                ])]]
            } else if no_calendario
                && nome_da_parte == "mes"
                && !e.dobrados.contains(&l.caminho)
            {
                // A grade do mês (ciclo 306). Dobrado, o mês volta a ser
                // uma linha comum com o resumo — sem semanas embaixo,
                // uma borda de cima sem a de baixo leria como desenho
                // quebrado.
                vec![linhas_do_mes(l, &e.tema, largura_conteudo)]
            } else if no_calendario && nome_da_parte == "semana" && !l.segmentos.is_empty() {
                let (ultimo, pai) = l.caminho.split_last().unwrap_or((&0, &[]));
                let ultima = e.arvore.em(pai).is_some_and(|m| m.filhos.len() == ultimo + 1);
                match e.arvore.em(&l.caminho) {
                    Some(semana) => vec![linhas_da_semana(
                        l,
                        semana,
                        &e.tema,
                        largura_conteudo,
                        Some(&e.cursor),
                        ultima,
                    )],
                    None => linhas_de_fileira(l, &e.tema, largura_conteudo, Some(&e.cursor)),
                }
            } else if !l.segmentos.is_empty() && l.embed_dono.as_deref() == Some("table")
            {
                let embed = e.arvore.em(l.dono_embed.as_deref().unwrap_or(&[]));
                let recuo = 2 * l.nivel;
                let larguras = larguras_que_cabem(
                    embed.map(larguras_de_tabela).unwrap_or_default(),
                    largura_conteudo.saturating_sub(recuo),
                );
                let cabecalho = matches!(&l.tipo, Tipo::Parte { nome, .. } if nome == "header");
                let ultima = embed.is_some_and(|t| l.caminho.last() == Some(&(t.filhos.len() - 1)));
                vec![linhas_de_tabela(
                    l,
                    &e.arvore,
                    &e.tema,
                    &larguras,
                    cabecalho,
                    ultima,
                    Some(&e.cursor),
                )]
            } else if !l.segmentos.is_empty() {
                linhas_de_fileira(l, &e.tema, largura_conteudo, Some(&e.cursor))
            } else if let Some(modo) = e.arvore.em(&l.caminho).and_then(caixa_avulsa) {
                vec![linha_de_caixa(l, &e.tema, largura_conteudo, Some(&e.cursor), modo)]
            } else {
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
            };
            // A lateral acende na linha do cursor: é como o foco se
            // marca dentro de um cartão que não é dele.
            let cor_lateral = match &atual {
                // Com o cursor no próprio embed, é a caixa INTEIRA que
                // acende: o rótulo, que era a linha do cursor, não é mais
                // desenhado (ciclo 309).
                Some((dono, r))
                    if no_foco && !l.enfeite && (l.mostra(&e.cursor) || *dono == e.cursor) =>
                {
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
                        Some(r) => emoldurar(
                            linha,
                            largura_util,
                            r,
                            atual.as_ref().map(|(_, regiao)| *regiao).unwrap_or(r),
                            &e.tema,
                        ),
                        None => linha,
                    });
                }
            }
        }
        if let Some((dono, velha)) = atual {
            if let Some(d) = linha_de_detalhe(e, &dono) {
                if fora.len() < e.altura {
                    fora.push(emoldurar(d, largura_util, velha, velha, &e.tema));
                }
            }
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
    if let Some(p) = &e.pergunta {
        bloco = bloco.title_bottom(Line::from(Span::styled(
            format!(" {}: {}▏ ", p.rotulo, p.texto),
            e.tema.estilo(Realce::Cursor),
        )));
    } else if let Some(aviso) = &e.aviso {
        bloco = bloco.title_bottom(Line::from(Span::styled(
            format!(" {aviso} "),
            e.tema.estilo(Realce::BadgeAtencao),
        )));
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

/// O papel de um badge pelo sufixo do nome (`--info`, `--success`…),
/// o mesmo de `badge_class` no núcleo.
fn papel_do_badge(sufixo: &str) -> Option<Realce> {
    match sufixo {
        "--info" => Some(Realce::BadgeInfo),
        "--success" => Some(Realce::BadgeSucesso),
        "--warning" => Some(Realce::BadgeAtencao),
        "--error" => Some(Realce::BadgeErro),
        _ => None,
    }
}

/// Quanto uma célula ocupa na tela (ciclo 308).
///
/// Badge é PÍLULA: um espaço de respiro de cada lado, o `padding: 1px
/// 8px` do `.badge` da janela. Uma célula de tags soma as pílulas e um
/// espaço entre elas (o `gap: 4px` de `.task-table__tags`).
fn largura_da_celula(celula: &Unidade) -> usize {
    let nome = match &celula.tipo {
        Tipo::Parte { nome, .. } => nome.as_str(),
        _ => "",
    };
    if nome == "tags" {
        let n = celula.filhos.len();
        return celula.filhos.iter().map(|t| t.texto.chars().count() + 2).sum::<usize>()
            + n.saturating_sub(1);
    }
    let texto = celula.texto.chars().count();
    if nome.starts_with("badge") && !celula.texto.is_empty() {
        texto + 2
    } else {
        texto
    }
}

/// A largura de cada COLUNA de uma tabela — o máximo entre cabeçalho e
/// linhas, pra célula ficar alinhada em grade (ciclo 305).
///
/// Recebe o embed inteiro (não uma linha): a largura de uma coluna só
/// existe olhando TODAS as linhas juntas, e é por isso que a tabela não
/// podia reusar o botão — um botão só olha o próprio texto.
fn larguras_de_tabela(embed: &Unidade) -> Vec<usize> {
    let mut larguras: Vec<usize> = Vec::new();
    for linha in &embed.filhos {
        for (i, celula) in linha.filhos.iter().enumerate() {
            let n = largura_da_celula(celula);
            match larguras.get_mut(i) {
                Some(atual) => *atual = (*atual).max(n),
                None => larguras.push(n),
            }
        }
    }
    larguras
}

/// As larguras cabendo no painel: enquanto a grade passa da largura,
/// a coluna mais larga cede uma célula (nunca abaixo de 3). O texto que
/// não couber sai cortado com `…`, em vez de a borda direita sumir.
fn larguras_que_cabem(mut larguras: Vec<usize>, disponivel: usize) -> Vec<usize> {
    let total = |ls: &[usize]| ls.iter().map(|w| w + 3).sum::<usize>() + 1;
    while total(&larguras) > disponivel {
        let Some((i, maior)) = larguras.iter().copied().enumerate().max_by_key(|(_, w)| *w) else { break };
        if maior <= 3 {
            break;
        }
        larguras[i] -= 1;
    }
    larguras
}

/// Uma régua da grade da tabela: `┌──┬──┐`, `╞══╪══╡`, `├──┼──┤`,
/// `└──┴──┘`. Cada coluna ocupa a largura dela e mais os dois espaços de
/// respiro da célula.
fn regua_da_tabela<'a>(recuo: &str, larguras: &[usize], pecas: [&str; 4], tema: &Tema) -> Line<'a> {
    let miolo = larguras
        .iter()
        .map(|w| pecas[3].repeat(w + 2))
        .collect::<Vec<_>>()
        .join(pecas[1]);
    Line::from(vec![
        Span::styled(recuo.to_string(), Style::default()),
        Span::styled(format!("{}{miolo}{}", pecas[0], pecas[2]), tema.estilo(Realce::Grade)),
    ])
}

/// Uma linha de TABELA desenhada como GRADE (ciclo 309): borda dos
/// quatro lados, `│` entre as colunas, uma régua depois de cada linha — o
/// `border-bottom` de cada `<tr>` da janela — e o cabeçalho separado por
/// régua dupla. O cabeçalho abre a grade; a última linha a fecha.
///
/// Select e multiselect saem em PÍLULA (ciclo 308): texto na cor do
/// badge sobre o fundo tingido dele, e o multiselect com uma pílula
/// por tag, cada uma na sua cor.
#[allow(clippy::too_many_arguments)]
fn linhas_de_tabela<'a>(
    l: &crate::tela::Linha,
    arvore: &Unidade,
    tema: &Tema,
    larguras: &[usize],
    cabecalho: bool,
    ultima: bool,
    cursor: Option<&[usize]>,
) -> Vec<Line<'a>> {
    let recuo = "  ".repeat(l.nivel);
    let grade = tema.estilo(Realce::Grade);
    let mut spans = vec![Span::styled(recuo.clone(), Style::default()), Span::styled("│", grade)];
    for (i, seg) in l.segmentos.iter().enumerate() {
        let largura = larguras.get(i).copied().unwrap_or(0);
        let aceso = cursor.is_some_and(|c| c == seg.caminho.as_slice());
        if aceso || cabecalho {
            let papel = if aceso { Realce::Cursor } else { Realce::CabecalhoDeTabela };
            spans.push(Span::styled(format!(" {} ", caber(&seg.texto, largura)), tema.estilo(papel)));
            spans.push(Span::styled("│", grade));
            continue;
        }
        // As pílulas desta célula, como (texto, papel).
        let pilulas: Vec<(String, Realce)> = if seg.nome == "tags" {
            arvore
                .em(&seg.caminho)
                .map(|c| {
                    c.filhos
                        .iter()
                        .filter_map(|t| match &t.tipo {
                            Tipo::Parte { nome, .. } => nome
                                .strip_prefix("tag")
                                .map(|suf| (t.texto.clone(), papel_do_badge(suf).unwrap_or(Realce::Celula))),
                            _ => None,
                        })
                        .collect()
                })
                .unwrap_or_default()
        } else if let Some(papel) = seg
            .nome
            .strip_prefix("badge")
            .and_then(papel_do_badge)
            .filter(|_| !seg.texto.is_empty())
        {
            vec![(seg.texto.clone(), papel)]
        } else {
            spans.push(Span::styled(
                format!(" {} ", caber(&seg.texto, largura)),
                tema.estilo(papel_da_parte(&seg.nome)),
            ));
            spans.push(Span::styled("│", grade));
            continue;
        };
        spans.push(Span::styled(" ", Style::default()));
        let mut resta = largura;
        for (k, (texto, papel)) in pilulas.iter().enumerate() {
            if k > 0 {
                if resta == 0 {
                    break;
                }
                spans.push(Span::styled(" ", Style::default()));
                resta -= 1;
            }
            let precisa = texto.chars().count() + 2;
            if resta >= precisa {
                spans.push(Span::styled(format!(" {texto} "), tema.pilula(*papel)));
                resta -= precisa;
            } else {
                // A pílula que não cabe inteira sai cortada, e as
                // seguintes ficam de fora — a borda da coluna é fixa.
                if resta >= 3 {
                    spans.push(Span::styled(format!(" {} ", caber(texto, resta - 2)), tema.pilula(*papel)));
                    resta = 0;
                }
                break;
            }
        }
        spans.push(Span::styled(" ".repeat(resta + 1), Style::default()));
        spans.push(Span::styled("│", grade));
    }

    let mut fora = Vec::new();
    if cabecalho {
        fora.push(regua_da_tabela(&recuo, larguras, ["┌", "┬", "┐", "─"], tema));
    }
    fora.push(Line::from(spans));
    fora.push(match (cabecalho, ultima) {
        (_, true) => regua_da_tabela(&recuo, larguras, ["└", "┴", "┘", "─"], tema),
        (true, false) => regua_da_tabela(&recuo, larguras, ["╞", "╪", "╡", "═"], tema),
        (false, false) => regua_da_tabela(&recuo, larguras, ["├", "┼", "┤", "─"], tema),
    });
    fora
}

/// A largura de uma célula (dia) da grade do mês: sete dias e oito
/// traços de borda cabem na largura disponível (ciclo 306).
fn largura_do_dia(disponivel: usize) -> usize {
    (disponivel.saturating_sub(8) / 7).max(3)
}

/// `texto` exatamente em `largura` colunas: cortado com `…` se passa,
/// completado com espaço se falta.
fn caber(texto: &str, largura: usize) -> String {
    let n = texto.chars().count();
    if n > largura {
        let mut t: String = texto.chars().take(largura.saturating_sub(1)).collect();
        t.push('…');
        t
    } else {
        format!("{texto}{}", " ".repeat(largura - n))
    }
}

/// Uma régua horizontal da grade: `┌───┬───┐`, `├───┼───┤` ou
/// `└───┴───┘`.
fn regua_do_mes<'a>(recuo: &str, cel: usize, pontas: [&str; 3], tema: &Tema) -> Line<'a> {
    let miolo = vec!["─".repeat(cel); 7].join(pontas[1]);
    Line::from(vec![
        Span::styled(recuo.to_string(), Style::default()),
        Span::styled(format!("{}{miolo}{}", pontas[0], pontas[2]), tema.estilo(Realce::Grade)),
    ])
}

/// O cabeçalho de um mês: o nome, os dias da semana e a borda de cima
/// da grade (ciclo 306) — o `Agosto 2026` e a fileira `D S T Q Q S S`
/// que a janela desenha em cima das semanas.
fn linhas_do_mes(l: &crate::tela::Linha, tema: &Tema, largura: usize) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(l.nivel);
    let cel = largura_do_dia(largura.saturating_sub(recuo.len()));
    let dias: Vec<String> = anotadinho_core::calendario::WEEKDAY_LABELS
        .iter()
        .map(|d| format!("{d:^cel$}"))
        .collect();
    vec![
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(l.texto.clone(), tema.estilo(Realce::TituloCartao)),
        ]),
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(format!(" {} ", dias.join(" ")), tema.estilo(Realce::CabecalhoDeTabela)),
        ]),
        regua_do_mes(&recuo, cel, ["┌", "┬", "┐"], tema),
    ]
}

/// O cabeçalho do calendário ancorado (ciclo 315): as teclas que fazem
/// o papel dos controles da janela (‹ › e "Hoje") e a contagem de
/// eventos, à direita como o `.calendar-grid__count`.
fn linha_do_cabecalho_do_calendario(
    l: &crate::tela::Linha,
    visao: anotadinho_core::analise::Visao,
    tema: &Tema,
    largura: usize,
) -> Line<'static> {
    let recuo = "  ".repeat(l.nivel);
    let teclas = format!("[ ‹   ] ›   t hoje   m {}", visao.rotulo());
    let usado = recuo.chars().count() + teclas.chars().count() + l.texto.chars().count();
    Line::from(vec![
        Span::styled(recuo, Style::default()),
        Span::styled(teclas.clone(), tema.estilo(Realce::Marca)),
        Span::styled(" ".repeat(largura.saturating_sub(usado + 1).max(2)), Style::default()),
        Span::styled(l.texto.clone(), tema.estilo(Realce::Marca)),
    ])
}

/// Um compromisso da agenda do Dia (ciclo 316): a hora (ou "dia
/// inteiro") numa coluna fixa e o título numa pílula na cor do evento,
/// até a borda — a lista que a janela desenha na visão Dia.
fn linha_do_compromisso(
    l: &crate::tela::Linha,
    arvore: &Unidade,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
) -> Line<'static> {
    let recuo = "  ".repeat(l.nivel);
    let hora = arvore
        .em(&l.caminho)
        .and_then(|u| {
            u.filhos
                .iter()
                .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "hora"))
                .map(|f| f.texto.clone())
        })
        .unwrap_or_default();
    let nome = match &l.tipo {
        Tipo::Parte { nome, .. } => nome.clone(),
        _ => String::new(),
    };
    let aceso = cursor.is_some_and(|c| l.mostra(c));
    let estilo = if aceso {
        tema.estilo(Realce::Cursor)
    } else {
        tema.pilula(papel_do_evento(&nome.replacen("compromisso", "evento", 1)))
    };
    let coluna_hora = 13;
    let resto = largura.saturating_sub(recuo.len() + coluna_hora + 1).max(4);
    Line::from(vec![
        Span::styled(recuo, Style::default()),
        Span::styled(format!("{hora:<coluna_hora$}"), tema.estilo(Realce::Marca)),
        Span::styled(" ", Style::default()),
        Span::styled(caber(&format!(" {}", l.texto), resto), estilo),
    ])
}

/// O papel de um evento na grade, pelo sufixo que o núcleo pôs no nome
/// (`evento--info` → a cor de `badge--info`).
fn papel_do_evento(nome: &str) -> Realce {
    match nome.strip_prefix("evento") {
        Some("--info") => Realce::BadgeInfo,
        Some("--success") => Realce::BadgeSucesso,
        Some("--warning") => Realce::BadgeAtencao,
        Some("--error") => Realce::BadgeErro,
        _ => Realce::Evento,
    }
}

/// Uma SEMANA da grade do mês (ciclo 306): a linha dos números, uma
/// linha por faixa de evento, o "+N mais" quando sobra, e a régua que
/// fecha — `├┼┤` entre semanas, `└┴┘` na última.
///
/// Um evento de vários dias é UMA barra: o texto começa no dia em que
/// ele entra na semana e corre por cima das bordas dos dias seguintes,
/// como a barra da janela atravessa as células com `grid-column`.
fn linhas_da_semana(
    l: &crate::tela::Linha,
    semana: &Unidade,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
    ultima: bool,
) -> Vec<Line<'static>> {
    // O recuo é o do MÊS, não o da semana: a semana é filha dele na
    // árvore, mas na tela as duas são a mesma grade — um nível a mais
    // empurraria as bordas pra fora do alinhamento com o cabeçalho.
    let recuo = "  ".repeat(l.nivel.saturating_sub(1));
    let cel = largura_do_dia(largura.saturating_sub(recuo.len()));
    let borda = tema.estilo(Realce::Grade);
    let nome = |u: &Unidade| match &u.tipo {
        Tipo::Parte { nome, .. } => nome.clone(),
        _ => String::new(),
    };
    let faixas_de = |dia: &Unidade| -> Vec<Unidade> {
        dia.filhos.iter().filter(|f| !matches!(nome(f).as_str(), "mais" | "detalhe" | "data")).cloned().collect()
    };
    let dias: Vec<&Unidade> = semana.filhos.iter().collect();
    let mut fora = Vec::new();

    // Os números, à direita da célula como na janela.
    let mut spans = vec![Span::styled(recuo.clone(), Style::default()), Span::styled("│", borda)];
    for (col, dia) in dias.iter().enumerate() {
        let aceso = cursor.is_some_and(|c| {
            l.segmentos.get(col).is_some_and(|s| s.caminho.as_slice() == c)
        });
        // Hoje é PÍLULA no tom de destaque (ciclo 315): a janela pinta o
        // número num círculo de `--accent-blue`. O cursor continua fundo
        // cheio — os dois precisam se distinguir quando coincidem.
        let estilo = if aceso {
            tema.estilo(Realce::Cursor)
        } else if nome(dia) == "dia-hoje" {
            tema.pilula(Realce::BadgeInfo).add_modifier(Modifier::BOLD)
        } else if nome(dia) == "dia-fora" {
            tema.estilo(Realce::Marca)
        } else {
            tema.estilo(Realce::Celula)
        };
        let numero = format!("{:>2}", dia.texto);
        spans.push(Span::styled(" ".repeat(cel.saturating_sub(numero.chars().count() + 1)), Style::default()));
        spans.push(Span::styled(numero, estilo));
        spans.push(Span::styled(" ", Style::default()));
        spans.push(Span::styled("│", borda));
    }
    fora.push(Line::from(spans));

    // As faixas. Uma semana sem evento ainda ganha uma linha em branco:
    // a célula da janela tem altura mínima, e um dia de uma linha só
    // não lê como dia.
    let faixas = dias.iter().map(|d| faixas_de(d).len()).max().unwrap_or(0);
    for faixa in 0..faixas.max(1) {
        let mut spans = vec![Span::styled(recuo.clone(), Style::default()), Span::styled("│", borda)];
        let mut col = 0;
        while col < dias.len() {
            let slot = faixas_de(dias[col]).get(faixa).cloned();
            let nome_slot = slot.as_ref().map(&nome).unwrap_or_default();
            // O cursor num slot desta faixa, no dia `c`? A faixa é a
            // POSIÇÃO dela entre os filhos do dia, que vêm primeiro.
            let no_slot = |c_dia: usize| {
                cursor.is_some_and(|c| {
                    l.segmentos.get(c_dia).is_some_and(|s| {
                        c.len() == s.caminho.len() + 1
                            && c.starts_with(&s.caminho)
                            && c[s.caminho.len()] == faixa
                    })
                })
            };
            if nome_slot.starts_with("evento") {
                let mut k = 1;
                while col + k < dias.len()
                    && faixas_de(dias[col + k])
                        .get(faixa)
                        .is_some_and(|s| nome(s) == "evento-continua")
                {
                    k += 1;
                }
                let w = k * cel + (k - 1);
                let texto = slot.map(|s| s.texto).unwrap_or_default();
                // A barra INTEIRA acende com o cursor em qualquer dia
                // dela (ciclo 310): no começo ou no meio, é o mesmo
                // evento.
                let aceso = (col..col + k).any(no_slot);
                let estilo = if aceso {
                    tema.estilo(Realce::Cursor)
                } else {
                    tema.pilula(papel_do_evento(&nome_slot))
                };
                spans.push(Span::styled(caber(&format!(" {texto}"), w), estilo));
                col += k;
            } else {
                spans.push(Span::styled(" ".repeat(cel), Style::default()));
                col += 1;
            }
            spans.push(Span::styled("│", borda));
        }
        fora.push(Line::from(spans));
    }

    // "+N mais", na célula de cada dia que transbordou.
    let mais: Vec<Option<String>> = dias
        .iter()
        .map(|d| d.filhos.iter().find(|f| nome(f) == "mais").map(|f| f.texto.clone()))
        .collect();
    if mais.iter().any(Option::is_some) {
        let mut spans = vec![Span::styled(recuo.clone(), Style::default()), Span::styled("│", borda)];
        for m in &mais {
            spans.push(Span::styled(caber(m.as_deref().unwrap_or(""), cel), tema.estilo(Realce::Marca)));
            spans.push(Span::styled("│", borda));
        }
        fora.push(Line::from(spans));
    }

    fora.push(if ultima {
        regua_do_mes(&recuo, cel, ["└", "┴", "┘"], tema)
    } else {
        regua_do_mes(&recuo, cel, ["├", "┼", "┤"], tema)
    });
    fora
}

/// A linha de DETALHE de um embed: com o cursor numa parte que tem
/// "detalhe" (a barra do cronograma, o evento do calendário), ele aparece
/// no pé da caixa (ciclo 313) — o que na janela é o `title` ao passar o
/// mouse ou o modal ao clicar.
fn linha_de_detalhe<'a>(e: &Estado, dono: &[usize]) -> Option<Line<'a>> {
    if e.foco != Foco::Conteudo || e.cursor.len() <= dono.len() || !e.cursor.starts_with(dono) {
        return None;
    }
    let u = e.arvore.em(&e.cursor)?;
    let detalhe = u
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "detalhe"))?;
    // O dia do calendário já diz a data por extenso no detalhe; o
    // número dele na frente seria a mesma coisa duas vezes.
    let e_dia = matches!(&u.tipo, Tipo::Parte { nome, .. } if nome.starts_with("dia"));
    let mut spans = vec![Span::styled("  ", Style::default())];
    if !e_dia {
        spans.push(Span::styled(u.texto.clone(), e.tema.estilo(Realce::TituloCartao)));
        spans.push(Span::styled(" · ", e.tema.estilo(Realce::Marca)));
    }
    spans.push(Span::styled(detalhe.texto.clone(), e.tema.estilo(Realce::Dica)));
    Some(Line::from(spans))
}

/// Abreviação do mês pro eixo do cronograma — a da janela (`03 ago`).
fn mes_abreviado(m: u32) -> &'static str {
    const M: [&str; 12] = ["jan", "fev", "mar", "abr", "mai", "jun", "jul", "ago", "set", "out", "nov", "dez"];
    M.get(m.saturating_sub(1) as usize).copied().unwrap_or("")
}

/// O EIXO de datas do cronograma (ciclo 313): os rótulos e uma régua com
/// `┬` em cada marca, na MESMA conta de coluna das barras — a marca de
/// 10 de agosto cai em cima do começo de uma barra que começa no dia 10.
///
/// Marca por semana quando a janela é curta (até 10 semanas, como a
/// escala `week` da janela); por mês quando é longa; por ano quando
/// passa de dois. Rótulo que encostaria no anterior é pulado.
fn linhas_do_eixo(l: &crate::tela::Linha, tema: &Tema, largura: usize) -> Vec<Line<'static>> {
    use anotadinho_core::date_util::{add_days, days_between, parse_date};
    let recuo = "  ".repeat(l.nivel);
    let disponivel = largura.saturating_sub(recuo.len()).max(4);
    let Some((inicio, fim)) = l.texto.split_once(' ') else { return Vec::new() };
    let dias = days_between(inicio, fim).unwrap_or(0) + 1;
    if dias <= 0 {
        return Vec::new();
    }
    let coluna = |data: &str| -> usize {
        let off = days_between(inicio, data).unwrap_or(0).max(0);
        ((disponivel as f64 * off as f64 / dias as f64).round() as usize).min(disponivel - 1)
    };
    // As datas das marcas.
    let mut marcas: Vec<(String, String)> = Vec::new();
    if dias <= 70 {
        let mut k = 0;
        while k < dias {
            if let Some(d) = add_days(inicio, k) {
                if let Some((_, m, dd)) = parse_date(&d) {
                    marcas.push((d.clone(), format!("{dd:02} {}", mes_abreviado(m))));
                }
            }
            k += 7;
        }
    } else {
        let Some((mut y, mut m, _)) = parse_date(inicio) else { return Vec::new() };
        let anual = dias > 730;
        loop {
            let d = anotadinho_core::date_util::format_date(y, m, 1);
            if d.as_str() > fim {
                break;
            }
            if d.as_str() >= inicio && (!anual || m == 1) {
                let rotulo = if anual { y.to_string() } else { format!("{} {y}", mes_abreviado(m)) };
                marcas.push((d, rotulo));
            }
            (y, m) = anotadinho_core::date_util::next_month(y, m);
        }
    }
    let mut rotulos = vec![' '; disponivel];
    let mut regua: Vec<char> = vec!['─'; disponivel];
    let mut livre_a_partir = 0usize;
    for (data, rotulo) in marcas {
        let c = coluna(&data);
        regua[c] = '┬';
        let n = rotulo.chars().count();
        if c >= livre_a_partir && c + n <= disponivel {
            for (k, ch) in rotulo.chars().enumerate() {
                rotulos[c + k] = ch;
            }
            livre_a_partir = c + n + 1;
        }
    }
    let marca = tema.estilo(Realce::Marca);
    vec![
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(rotulos.into_iter().collect::<String>(), marca),
        ]),
        Line::from(vec![
            Span::styled(recuo, Style::default()),
            Span::styled(regua.into_iter().collect::<String>(), tema.estilo(Realce::Grade)),
        ]),
    ]
}

/// Como um retângulo/quadrado preenchido SOZINHO ocupa a largura do
/// painel (ciclo 304, estendido no 305 pra caber a barra do
/// cronograma).
#[derive(Debug, Clone, Copy, PartialEq)]
enum ModoCaixa {
    /// Estica até a borda — o cartão do kanban, a entrada do
    /// calendário: ocupam a largura da coluna/painel, como na janela.
    Cheia,
    /// Cresce só até caber o texto — a miniatura da galeria, um selo.
    Conteudo,
    /// Retângulo PROPORCIONAL dentro da largura disponível — a barra
    /// do cronograma. Os dois números são porcentagem (0-100): onde
    /// começa, quanto ocupa.
    Proporcional { inicio_pct: u8, largura_pct: u8 },
}

/// A unidade, quando ela quer ser desenhada como retângulo/quadrado
/// preenchido sozinha na própria linha — não numa fileira (ciclo 304).
///
/// `None` pra qualquer outra parte — aí quem desenha usa a linha comum.
fn caixa_avulsa(u: &Unidade) -> Option<ModoCaixa> {
    match &u.tipo {
        Tipo::Parte { nome, arranjo: Arranjo::Folha } if nome == "card" => Some(ModoCaixa::Cheia),
        // Miniatura e evento crescem só até caber o conteúdo — nem uma
        // é "coisa inteira" como o cartão: uma é o TAMANHO de uma
        // legenda, a outra é um PONTO no calendário, não uma faixa que
        // preencha a coluna (isso é o cronograma, que tem largura
        // própria — `Proporcional`, abaixo).
        Tipo::Parte { nome, arranjo: Arranjo::Folha } if nome == "miniatura" || nome == "entry" => {
            Some(ModoCaixa::Conteudo)
        }
        // A barra é GRUPO, não folha: início e duração viajam como
        // duas partes-filhas em porcentagem (`crates/core/src/analise.rs`,
        // ciclo 305) — dado de desenho, não conteúdo pra navegar ou
        // mostrar (`tela::fica_fora_da_tela` as esconde da tela e do Enter).
        Tipo::Parte { nome, .. } if nome == "barra" => {
            let pct = |campo: &str| -> Option<u8> {
                u.filhos
                    .iter()
                    .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == campo))
                    .and_then(|f| f.texto.parse().ok())
            };
            Some(ModoCaixa::Proporcional {
                inicio_pct: pct("inicio").unwrap_or(0),
                largura_pct: pct("duracao").unwrap_or(100),
            })
        }
        _ => None,
    }
}

/// Um retângulo preenchido SOZINHO — mesmo contorno de meio-bloco e
/// miolo do botão (ciclo 302), só que empilhado como o resto de uma
/// coluna em vez de lado a lado numa fileira (ciclo 304).
///
/// Existe porque nem todo conteúdo clicável fica em fileira: um cartão
/// de kanban é filho de uma COLUNA — `j`/`k` andam entre cartões, não
/// `h`/`l` — e mesmo assim quer a caixa fechada que só botão tinha.
fn linha_de_caixa(
    l: &crate::tela::Linha,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
    modo: ModoCaixa,
) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(l.nivel);
    let disponivel = largura.saturating_sub(recuo.len()).max(4);
    let nome = match &l.tipo {
        Tipo::Parte { nome, .. } => nome.as_str(),
        _ => "",
    };
    let aceso = cursor.is_some_and(|c| l.mostra(c));
    let papel = if aceso { Realce::Cursor } else { papel_da_parte(nome) };
    let contorno = tema.contorno_do_botao(papel);
    let miolo_estilo = tema.miolo_do_botao(papel);

    // Cheia e Proporcional preenchem o miolo com espaço até a largura
    // alvo — é o que faz a caixa parecer um retângulo sólido em vez de
    // encolher pro tamanho do texto. Conteúdo não: um selo cresce só
    // até caber a legenda.
    let (largura_caixa, deslocamento, preencher) = match modo {
        ModoCaixa::Cheia => (disponivel, 0, true),
        ModoCaixa::Conteudo => (disponivel, 0, false),
        ModoCaixa::Proporcional { inicio_pct, largura_pct } => {
            let desloc = (disponivel as f64 * inicio_pct as f64 / 100.0).round() as usize;
            let larg = ((disponivel as f64 * largura_pct as f64 / 100.0).round() as usize).max(4);
            // A barra não pode vazar a borda: sobra da largura entra
            // no deslocamento antes, não depois.
            (larg, desloc.min(disponivel.saturating_sub(larg)), true)
        }
    };

    // O miolo cabe na largura menos as duas meias-célula do contorno e
    // o espaço de respiro de cada lado — a mesma conta do botão
    // (`largura_do_botao`), só que aqui a largura é a da CAIXA, não a
    // do texto: um cartão vazio ainda ocupa a coluna inteira.
    let cabe = largura_caixa.saturating_sub(4).max(1);
    let mut texto: String = l.texto.chars().take(cabe).collect();
    if preencher {
        let usado = texto.chars().count();
        if usado < cabe {
            texto.push_str(&" ".repeat(cabe - usado));
        }
    }
    let largura_miolo = texto.chars().count() + 2;
    let prefixo = " ".repeat(deslocamento);

    vec![
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(prefixo.clone(), Style::default()),
            Span::styled(format!("▗{}▖", "▄".repeat(largura_miolo)), contorno),
        ]),
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(prefixo.clone(), Style::default()),
            Span::styled("▐", contorno),
            Span::styled(format!(" {texto} "), miolo_estilo),
            Span::styled("▌", contorno),
        ]),
        Line::from(vec![
            Span::styled(recuo, Style::default()),
            Span::styled(prefixo, Style::default()),
            Span::styled(format!("▝{}▘", "▀".repeat(largura_miolo)), contorno),
        ]),
    ]
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

/// Os dados de um calendário, lidos da fonte do embed.
fn dados_do_calendario(fonte: &str) -> Option<anotadinho_core::embed::CalendarEmbedData> {
    anotadinho_core::embed::segment(fonte).into_iter().find_map(|seg| match seg {
        anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Calendar(d)) => Some(d),
        _ => None,
    })
}

/// A data com `delta` meses a mais, no dia 1 — a âncora de "mês que vem".
fn outro_mes(data: &str, delta: i32) -> Option<String> {
    use anotadinho_core::date_util::{format_date, next_month, parse_date, prev_month};
    let (mut y, mut m, _) = parse_date(data)?;
    for _ in 0..delta.unsigned_abs() {
        (y, m) = if delta > 0 { next_month(y, m) } else { prev_month(y, m) };
    }
    Some(format_date(y, m, 1))
}

/// As teclas do calendário ancorado (ciclos 315 e 316): `[` e `]` andam
/// pra trás e pra frente (os botões ‹ › da janela — um mês, uma semana ou
/// um dia, conforme a visão), `t` volta pra hoje, `m` troca a visão
/// (Mês → Semana → Dia, o seletor da janela).
///
/// Só com o cursor dentro de um calendário e com o dia de hoje conhecido;
/// fora disso a tecla segue pro resto. O cursor vai pro mês (ou pra
/// agenda), pra grade nova estar à vista.
fn tecla_do_calendario(e: &mut Estado, tecla: &str) -> bool {
    use anotadinho_core::analise::Visao;
    if !matches!(tecla, "[" | "]" | "t" | "m") {
        return false;
    }
    let (Some(hoje), Some(embed)) = (e.hoje.clone(), tela::calendario_do_cursor(&e.arvore, &e.cursor)) else {
        return false;
    };
    let atual = e.ancoras.get(&embed).cloned().unwrap_or_else(|| hoje.clone());
    let visao = e.visoes.get(&embed).copied().unwrap_or_default();
    let passo = |delta: i32| match visao {
        Visao::Mes => outro_mes(&atual, delta),
        Visao::Semana => anotadinho_core::date_util::add_days(&atual, 7 * delta as i64),
        Visao::Dia => anotadinho_core::date_util::add_days(&atual, delta as i64),
    };
    let nova = match tecla {
        "[" => passo(-1),
        "]" => passo(1),
        "m" => {
            e.visoes.insert(embed.clone(), visao.seguinte());
            // Trocar de visão não troca de dia: com o cursor num dia ou
            // evento, a semana ou a agenda são as DELE.
            Some(tela::data_do_cursor(&e.arvore, &e.cursor).unwrap_or(atual.clone()))
        }
        _ => Some(hoje),
    };
    let Some(nova) = nova else { return true };
    e.ancorar(&embed, nova);
    let mut no_mes = embed.clone();
    no_mes.push(1);
    e.cursor = if e.arvore.em(&no_mes).is_some() { no_mes } else { embed };
    e.seguir_cursor();
    true
}

/// Com o calendário ancorado, `h`/`l` num evento sem vizinho no que está
/// na tela pulam pro próximo dia com evento FORA dela (ciclos 315 e 316):
/// o próximo mês na visão Mês, a próxima semana na Semana, o próximo dia
/// na agenda do Dia. A âncora vai pra esse dia, e o cursor pousa no
/// primeiro evento dele.
fn pular_pro_mes_com_evento(e: &mut Estado, adiante: bool) -> bool {
    let Some(embed) = tela::calendario_do_cursor(&e.arvore, &e.cursor) else { return false };
    let Some(dados) = e.arvore.em(&embed).and_then(|u| u.fonte.as_deref()).and_then(dados_do_calendario) else {
        return false;
    };
    // Os dias que estão na tela: os do mês (ou da semana), ou o da agenda.
    let visiveis = tela::datas_visiveis(&e.arvore, &embed);
    let (Some(primeiro), Some(ultimo)) = (visiveis.first().cloned(), visiveis.last().cloned()) else {
        return false;
    };
    // Todo dia tocado por algum evento, em ordem.
    let mut datas = std::collections::BTreeSet::new();
    for ev in &dados.entries {
        let Some(inicio) = ev.date.as_deref() else { continue };
        let fim = ev.end_date.as_deref().filter(|f| *f > inicio).unwrap_or(inicio);
        let dias = anotadinho_core::date_util::days_between(inicio, fim).unwrap_or(0).clamp(0, 366);
        for k in 0..=dias {
            if let Some(dt) = anotadinho_core::date_util::add_days(inicio, k) {
                datas.insert(dt);
            }
        }
    }
    let alvo = if adiante {
        datas.iter().find(|d| **d > ultimo).cloned()
    } else {
        datas.iter().rev().find(|d| **d < primeiro).cloned()
    };
    let Some(alvo) = alvo else { return false };
    e.ancorar(&embed, alvo.clone());
    let mut agenda = embed.clone();
    agenda.push(1);
    let e_agenda = matches!(
        e.arvore.em(&agenda).map(|u| &u.tipo),
        Some(Tipo::Parte { nome, .. }) if nome == "agenda"
    );
    if e_agenda {
        agenda.push(0);
        e.cursor = agenda;
        return true;
    }
    let Some(dia) = tela::dia_com_data(&e.arvore, &embed, &alvo) else { return false };
    let Some(u) = e.arvore.em(&dia) else { return false };
    let evento = u
        .filhos
        .iter()
        .position(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if tela::e_evento(nome)));
    let mut c = dia;
    if let Some(f) = evento {
        c.push(f);
    }
    e.cursor = c;
    true
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
    // Numa barra do cronograma, `h`/`l` andam pela ordem no TEMPO (ciclo
    // 313) — ver `tela::barra_ao_lado`.
    if matches!(mov, Movimento::Direita | Movimento::Esquerda)
        && e.cursor.len() >= 2
        && matches!(
            e.arvore.em(&e.cursor[..e.cursor.len() - 1]).map(|p| &p.tipo),
            Some(Tipo::Embed(n)) if n == "timeline"
        )
    {
        let adiante = matches!(mov, Movimento::Direita);
        for _ in 0..vezes.max(1) {
            match tela::barra_ao_lado(&e.arvore, &e.cursor, adiante) {
                Some(c) => e.cursor = c,
                None => break,
            }
        }
        e.seguir_cursor();
        return;
    }
    // Num evento do calendário, `h`/`l` andam de DIA com o evento
    // selecionado (ciclo 311) — ver `tela::evento_ao_lado`.
    let num_compromisso = e
        .arvore
        .em(&e.cursor)
        .is_some_and(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome.starts_with("compromisso")));
    let num_evento = e
        .arvore
        .em(&e.cursor)
        .is_some_and(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if tela::e_evento(nome)))
        && tela::calendario_do_cursor(&e.arvore, &e.cursor).is_some();
    // Na agenda do Dia, `h`/`l` vão pro dia anterior/seguinte com evento.
    if matches!(mov, Movimento::Direita | Movimento::Esquerda) && num_compromisso {
        let adiante = matches!(mov, Movimento::Direita);
        for _ in 0..vezes.max(1) {
            if !pular_pro_mes_com_evento(e, adiante) {
                break;
            }
        }
        e.seguir_cursor();
        return;
    }
    if matches!(mov, Movimento::Direita | Movimento::Esquerda) && num_evento {
        let adiante = matches!(mov, Movimento::Direita);
        for _ in 0..vezes.max(1) {
            match tela::evento_ao_lado(&e.arvore, &e.cursor, adiante) {
                Some(c) => e.cursor = c,
                // Acabou o mês visível: com o calendário ancorado, o
                // próximo mês com evento vem pra tela (ciclo 315).
                None if e.hoje.is_some() => {
                    if !pular_pro_mes_com_evento(e, adiante) {
                        break;
                    }
                }
                None => break,
            }
        }
        e.seguir_cursor();
        return;
    }
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
fn emoldurar<'a>(
    linha: Line<'a>,
    largura: usize,
    cor: Realce,
    regiao: Realce,
    tema: &Tema,
) -> Line<'a> {
    let util = largura.saturating_sub(2);
    // Caixa PREENCHIDA (ciclo 307): todo trecho sem fundo próprio ganha o
    // fundo da região, e o que sobra até a borda também — senão o fundo
    // pararia onde o texto acaba, e a caixa leria como faixas soltas.
    let fundo = tema.fundo_da_regiao(regiao);
    let com_fundo = |estilo: Style| match fundo {
        Some(f) if estilo.bg.is_none() => estilo.bg(f),
        _ => estilo,
    };
    let (esquerda, direita) = match fundo {
        // A lateral esquerda é o `border-left: 3px` da janela: meia
        // célula na cor da variante (ou do foco, quando o cursor está na
        // linha) sobre o fundo. A direita é só meia célula do fundo,
        // como a borda de um botão.
        Some(f) => (
            Span::styled("▌", Style::default().fg(tema.estilo(cor).fg.unwrap_or(f)).bg(f)),
            Span::styled("▌", Style::default().fg(f)),
        ),
        None => (
            Span::styled("│", tema.estilo(cor)),
            Span::styled("│", tema.estilo(cor)),
        ),
    };
    let mut spans: Vec<Span<'a>> = vec![esquerda];
    let mut usado = 0usize;
    for s in linha.spans {
        let n = s.content.chars().count();
        if usado + n <= util {
            usado += n;
            spans.push(Span::styled(s.content, com_fundo(s.style)));
            continue;
        }
        // Este trecho não cabe inteiro: entra o que couber, menos um
        // caractere pro `…`.
        let cabe = util.saturating_sub(usado + 1);
        if cabe > 0 {
            let corte: String = s.content.chars().take(cabe).collect();
            spans.push(Span::styled(corte, com_fundo(s.style)));
            usado += cabe;
        }
        if usado < util {
            spans.push(Span::styled("…", com_fundo(s.style)));
            usado += 1;
        }
        break;
    }
    if usado < util {
        spans.push(Span::styled(" ".repeat(util - usado), com_fundo(Style::default())));
    }
    spans.push(direita);
    Line::from(spans)
}

/// O topo ou o fundo da moldura da unidade em foco.
///
/// Numa região com fundo (o callout, ciclo 307) a moldura é de
/// MEIO-BLOCO, como o botão do ciclo 302: `▗▄▄▖` em cima, `▝▀▀▘` embaixo,
/// na cor do fundo — a caixa inteira vira um retângulo preenchido, sem
/// traço em volta.
fn moldura<'a>(topo: bool, largura: usize, cor: Realce, tema: &Tema) -> Line<'a> {
    let meio = largura.saturating_sub(2);
    if let Some(f) = tema.fundo_da_regiao(cor) {
        let (canto, traco, fim) = if topo { ("▗", "▄", "▖") } else { ("▝", "▀", "▘") };
        return Line::from(Span::styled(
            format!("{canto}{}{fim}", traco.repeat(meio)),
            Style::default().fg(f),
        ));
    }
    let (canto, fim) = if topo { ("┌", "┐") } else { ("└", "┘") };
    Line::from(Span::styled(
        format!("{canto}{}{fim}", "─".repeat(meio)),
        tema.estilo(cor),
    ))
}

/// A variante de um callout, lida da parte "variante" que o núcleo põe
/// na frente do corpo (ciclo 307).
fn variante_do_callout(arvore: &Unidade, embed: &[usize]) -> String {
    arvore
        .em(embed)
        .and_then(|u| {
            u.filhos
                .iter()
                .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "variante"))
        })
        .map(|v| v.texto.clone())
        .unwrap_or_default()
}

/// O papel da caixa de um callout, pela variante.
fn papel_do_callout(arvore: &Unidade, embed: &[usize]) -> Realce {
    match variante_do_callout(arvore, embed).as_str() {
        "success" => Realce::CalloutSucesso,
        "warning" => Realce::CalloutAtencao,
        "error" => Realce::CalloutErro,
        "tip" => Realce::CalloutDica,
        _ => Realce::CalloutInfo,
    }
}

/// O ícone da variante, no lugar do ícone SVG da janela (`info`,
/// `check`, `alert-triangle`, `alert-circle`, `lightbulb`). Todos de
/// uma célula — emoji ocupa duas em quase todo terminal e desalinharia a
/// caixa.
fn icone_do_callout(variante: &str) -> &'static str {
    match variante {
        "success" => "✔",
        "warning" => "⚠",
        "error" => "✖",
        "tip" => "✦",
        _ => "ℹ",
    }
}

/// O cabeçalho do callout: ícone na cor da variante e o título em
/// negrito — o `.callout__header` da janela.
fn linha_do_titulo_do_callout<'a>(
    l: &crate::tela::Linha,
    variante: &str,
    papel: Realce,
    tema: &Tema,
) -> Line<'a> {
    Line::from(vec![
        Span::styled("  ".repeat(l.nivel), Style::default()),
        Span::styled(icone_do_callout(variante), tema.estilo(papel)),
        Span::styled(" ", Style::default()),
        Span::styled(l.texto.clone(), tema.estilo(Realce::TituloCartao)),
    ])
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
        // Ciclo 304: o rótulo fechado (`[kanban]`) usa a MESMA cor da
        // moldura aberta — é `papel_do_embed` quem resolve as duas, pra
        // abrir e fechar não trocar de cor.
        Tipo::Embed(nome) => papel_do_embed(nome),
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
        // Ciclo 304: o cartão do kanban e a miniatura da galeria — cada
        // um puxa a cor do embed dono (ver `tema::estilo`). O botão de
        // ações se separa em dois pra `variant: primary` deixar de
        // desaparecer no meio dos outros.
        "card" => Realce::Cartao,
        "miniatura" => Realce::Miniatura,
        // "button" fica de fora de propósito: cai no `Parte` genérico
        // de sempre, a mesma cor que já tinha antes deste ciclo — só o
        // `primary` precisava se destacar dos outros.
        "button-primary" => Realce::BotaoPrimario,
        // Ciclo 305: o evento do calendário e a barra do cronograma.
        "entry" => Realce::Entrada,
        "barra" => Realce::Barra,
        // Célula de tabela: "cell" é dado comum, "badge--X" é a MESMA
        // cor que `badge_class` (núcleo) já escolheu pra coluna
        // select/multiselect — o desenho só traduz o nome pro papel do
        // tema (ciclo 305).
        // A gaveta de eventos sem data: rótulo apagado, como o botão que
        // a abre na janela (ciclo 306).
        "sem-data" => Realce::Marca,
        "cell" => Realce::Celula,
        "badge--info" => Realce::BadgeInfo,
        "badge--success" => Realce::BadgeSucesso,
        "badge--warning" => Realce::BadgeAtencao,
        "badge--error" => Realce::BadgeErro,
        _ => Realce::Parte,
    }
}

/// A cor que IDENTIFICA um TIPO de embed — moldura aberta e rótulo
/// fechado (`[kanban]`) usam a mesma (ciclo 304).
///
/// Nome desconhecido ou o próprio fluxo caem no `Embed` genérico: o
/// fluxo já tinha a dele desde o ciclo 293, e um embed novo que ainda
/// não ganhou cor própria não deve desenhar preto.
fn papel_do_embed(nome: &str) -> Realce {
    match nome {
        "kanban" => Realce::EmbedKanban,
        "calendar" => Realce::EmbedCalendar,
        "table" => Realce::EmbedTable,
        "callout" => Realce::EmbedCallout,
        "columns" => Realce::EmbedColumns,
        "gallery" => Realce::EmbedGallery,
        "query" => Realce::EmbedQuery,
        "timeline" => Realce::EmbedTimeline,
        "actions" => Realce::EmbedActions,
        _ => Realce::Embed,
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
    use edicao::achar_com_indice;
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

    /// Uma página com um kanban de uma coluna e um cartão.
    fn com_kanban() -> Unidade {
        analisar("{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: Card A\n  column: Backlog\n{{ /kanban }}\n")
    }

    /// Uma página com uma galeria de duas imagens.
    fn com_galeria() -> Unidade {
        analisar("{{ type: \"gallery\" }}\nitems:\n- path: a.png\n  caption: Primeira\n- path: b.png\n  caption: Segunda\n{{ /gallery }}\n")
    }

    /// Uma página com um calendário: um evento de um dia com tag, uma
    /// sprint de cinco dias com outra tag, e um evento sem data.
    fn com_calendario() -> Unidade {
        analisar(
            "{{ type: \"calendar\" }}\nentries:\n\
             - date: 2026-08-06\n  title: Revisão de código\n  tags:\n  - urgente\n\
             - date: 2026-08-10\n  title: Sprint de agosto\n  end_date: 2026-08-14\n  tags:\n  - infra\n\
             - title: Ligar pro fornecedor\n\
             {{ /calendar }}\n",
        )
    }

    /// Uma página com um cronograma de duas barras: uma que ocupa a
    /// janela inteira (10 dias) e outra que emenda logo depois.
    fn com_cronograma() -> Unidade {
        analisar(
            "{{ type: \"timeline\" }}\nitems:\n\
             - title: Primeira etapa\n  start: '2026-08-01'\n  end: '2026-08-10'\n\
             - title: Segunda etapa\n  start: '2026-08-11'\n  end: '2026-08-20'\n\
             {{ /timeline }}\n",
        )
    }

    /// Uma página com uma tabela de duas colunas, uma delas `select`.
    fn com_tabela() -> Unidade {
        analisar(
            "{{ type: \"table\" }}\ncolumns:\n\
             - name: Tarefa\n\
             - name: Status\n  type: select\n  options: [todo, doing, done]\n\
             ---\n\
             | Tarefa | Status |\n| --- | --- |\n\
             | API | done |\n\
             | Um nome de tarefa bem mais longo | doing |\n\
             {{ /table }}\n",
        )
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

    /// Um callout `warning` com título e um parágrafo.
    fn com_callout(variante: &str) -> Unidade {
        analisar(&format!(
            "{{{{ type: \"callout\" }}}}\nvariant: {variante}\ntitle: Cuidado com a gravação\nbody: |\n  Editar em duas janelas sobrescreve.\n{{{{ /callout }}}}\n"
        ))
    }

    #[test]
    fn o_callout_e_uma_caixa_preenchida_na_cor_da_variante() {
        // Até aqui só a BORDA do callout tinha cor (e sempre a mesma,
        // `warning`). Na janela a caixa inteira é tingida pela variante
        // (ciclo 307).
        let mut e = Estado::novo(paginas(), com_callout("warning"));
        e.foco = Foco::Paginas;
        let fundo = e.tema.fundo_da_regiao(Realce::CalloutAtencao).unwrap();
        let buf = quadro(&mut e, 80, 12);
        let linha = (0..buf.area.height)
            .find(|y| (0..buf.area.width).map(|x| buf[(x, *y)].symbol().to_string()).collect::<String>().contains("Editar"))
            .expect("o corpo sumiu");
        let simbolos: Vec<String> = (0..buf.area.width).map(|x| buf[(x, linha)].symbol().to_string()).collect();
        let esquerda = simbolos.iter().position(|c| c == "▌").expect("sem lateral esquerda") as u16;
        let direita = simbolos.iter().rposition(|c| c == "▌").expect("sem lateral direita") as u16;
        assert!(direita > esquerda + 10);
        // Da lateral esquerda até antes da direita, TODA célula tem o
        // fundo — inclusive o espaço depois do texto.
        for x in esquerda..direita {
            assert_eq!(buf[(x, linha)].style().bg, Some(fundo), "vão sem fundo na coluna {x}");
        }
        // A lateral esquerda é a faixa de 3px da janela: cor da variante.
        assert_eq!(
            buf[(esquerda, linha)].style().fg,
            e.tema.estilo(Realce::CalloutAtencao).fg
        );
        let tudo = desenho(&mut e, 80, 12).join("\n");
        assert!(tudo.contains("▗▄▄"), "a caixa não fechou em meio-bloco:\n{tudo}");
        assert!(tudo.contains("▝▀▀"), "{tudo}");
    }

    #[test]
    fn cada_variante_do_callout_pinta_sua_caixa() {
        let fundo_de = |variante: &str| {
            let mut e = Estado::novo(paginas(), com_callout(variante));
            e.foco = Foco::Paginas;
            let buf = quadro(&mut e, 80, 12);
            (0..buf.area.height)
                .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
                .find(|p| buf[*p].symbol() == "E")
                .and_then(|p| buf[p].style().bg)
        };
        let info = fundo_de("info");
        let erro = fundo_de("error");
        assert!(info.is_some() && erro.is_some());
        assert_ne!(info, erro, "info e error saíram com o mesmo fundo");
    }

    #[test]
    fn o_titulo_do_callout_vem_com_o_icone_da_variante() {
        let mut e = Estado::novo(paginas(), com_callout("warning"));
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 80, 12).join("\n");
        assert!(tudo.contains("⚠ Cuidado com a gravação"), "{tudo}");
        // Com título, o rótulo `[callout]` sai — o título diz melhor.
        assert!(!tudo.contains("[callout]"), "{tudo}");
        // E a variante é dado, não texto na tela.
        assert!(!tudo.contains("warning"), "{tudo}");
    }

    #[test]
    fn callout_fechado_anuncia_o_rotulo_na_cor_da_variante() {
        // Aberto, o rótulo não aparece mais (ciclo 309); fechado ele é a
        // única identidade da caixa, e veste a cor da variante.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"callout\" }}\nvariant: error\nbody: |\n  Quebrou.\n{{ /callout }}\n"),
        );
        e.foco = Foco::Paginas;
        e.dobrados.insert(vec![0]);
        let erro = e.tema.estilo(Realce::CalloutErro).fg;
        let buf = quadro(&mut e, 80, 12);
        let rotulo = (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .find(|p| buf[*p].symbol() == "[")
            .expect("sem rótulo [callout]");
        assert_eq!(buf[rotulo].style().fg, erro);
    }

    #[test]
    fn embed_aberto_nao_mostra_o_nome_do_tipo() {
        // `[kanban]`, `[table]`… em cima de uma caixa já pintada na cor do
        // tipo diziam a mesma coisa duas vezes; a janela não tem rótulo.
        for pagina in [com_kanban(), com_tabela(), com_galeria(), com_acoes(), com_cronograma()] {
            let mut e = Estado::novo(paginas(), pagina);
            e.foco = Foco::Paginas;
            let tudo = desenho(&mut e, 80, 20).join("\n");
            for rotulo in ["[kanban]", "[table]", "[gallery]", "[actions]", "[timeline]"] {
                assert!(!tudo.contains(rotulo), "{rotulo} ainda aparece:\n{tudo}");
            }
        }
    }

    #[test]
    fn embed_fechado_ou_vazio_mantem_o_rotulo() {
        let mut e = Estado::novo(paginas(), com_kanban());
        e.foco = Foco::Paginas;
        e.dobrados.insert(vec![0]);
        assert!(desenho(&mut e, 80, 12).join("\n").contains("[kanban]"));

        // Sem linha nenhuma de conteúdo, o rótulo é o que desenha a caixa.
        let mut vazio = Estado::novo(
            paginas(),
            analisar("{{ type: \"query\" }}\nview: list\n{{ /query }}\n"),
        );
        vazio.foco = Foco::Paginas;
        assert!(desenho(&mut vazio, 80, 12).join("\n").contains("[query]"));
    }

    #[test]
    fn com_o_cursor_no_embed_a_caixa_inteira_acende() {
        // O rótulo era a linha do cursor. Sem ele, quem diz "você está
        // neste embed" é a lateral de TODAS as linhas da caixa.
        let mut e = Estado::novo(paginas(), com_kanban());
        e.cursor = vec![0];
        e.foco = Foco::Conteudo;
        let foco = e.tema.estilo(Realce::BordaUnidade).fg;
        let buf = quadro(&mut e, 60, 14);
        let laterais: Vec<_> = (0..buf.area.height)
            .filter_map(|y| {
                (0..buf.area.width)
                    .find(|x| buf[(*x, y)].symbol() == "▌")
                    .map(|x| buf[(x, y)].style().fg)
            })
            .collect();
        assert!(laterais.len() >= 3, "{laterais:?}");
        assert!(laterais.iter().all(|c| *c == foco), "nem toda lateral acendeu: {laterais:?}");
    }

    #[test]
    fn o_cursor_passa_por_cima_da_variante() {
        // A variante é a primeira filha do callout e não tem linha: Enter
        // no callout pousa no TÍTULO, e `k` no título não volta pra ela.
        let arvore = com_callout("info");
        let no_embed: Caminho = vec![0];
        let dentro = tela::andar(&arvore, &no_embed, Passo::Entrar);
        assert_eq!(dentro, vec![0, 1]);
        assert_eq!(tela::andar(&arvore, &dentro, Passo::Anterior), dentro);
        assert_eq!(tela::andar(&arvore, &dentro, Passo::Proximo), vec![0, 2]);
    }

    #[test]
    fn dois_embeds_vizinhos_sao_duas_caixas() {
        // O defeito da captura: seis embeds seguidos viravam UMA caixa
        // magenta, porque a região era identificada pela cor e todos
        // pediam a mesma. O dono distingue.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"table\" }}\n| A |\n| --- |\n| um |\n{{ /table }}\n\n{{ type: \"table\" }}\n| B |\n| --- |\n| dois |\n{{ /table }}\n"),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 14).join("\n");
        // Duas caixas de embed: desde o ciclo 308 todo embed é caixa
        // preenchida, que abre com o canto de meio-bloco `▗`.
        assert_eq!(
            tudo.matches('▗').count(),
            2,
            "os embeds vizinhos viraram uma caixa só:\n{tudo}"
        );
    }

    #[test]
    fn o_kanban_e_uma_caixa_preenchida_sem_vao_em_volta_do_cartao() {
        // A caixa do callout (ciclo 307) em todo embed (ciclo 308). O
        // teste que importa é o do cartão: o contorno de meio-bloco dele
        // deixa meia célula "de fora", e ela tem que ser do fundo da
        // CAIXA — senão cada cartão fica cercado de uma faixa escura.
        let mut e = Estado::novo(paginas(), com_kanban());
        e.foco = Foco::Paginas;
        let fundo = e.tema.fundo_da_regiao(Realce::EmbedKanban).unwrap();
        let buf = quadro(&mut e, 60, 14);
        let linha = (0..buf.area.height)
            .find(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, *y)].symbol().to_string())
                    .collect::<String>()
                    .contains("Card A")
            })
            .expect("o cartão sumiu");
        let simbolos: Vec<String> = (0..buf.area.width).map(|x| buf[(x, linha)].symbol().to_string()).collect();
        let esquerda = simbolos.iter().position(|c| c == "▌").unwrap() as u16;
        let contorno = simbolos.iter().position(|c| c == "▐").unwrap() as u16;
        // Do lado de dentro da lateral da caixa até o contorno do cartão
        // (inclusive a meia célula de fora dele): fundo da caixa.
        for x in esquerda..=contorno {
            assert_eq!(buf[(x, linha)].style().bg, Some(fundo), "vão sem fundo na coluna {x}");
        }
    }

    #[test]
    fn a_moldura_do_kanban_pinta_com_a_cor_do_kanban() {
        // Até este ciclo TODO embed pedia `Realce::Embed` (roxo) pra
        // moldura — kanban, callout, tabela, todos a mesma cor. Agora
        // cada tipo puxa a dele (ciclo 304), e é a cor que precisa
        // aparecer na tela — não só existir no tema.
        let mut e = Estado::novo(paginas(), com_kanban());
        e.foco = Foco::Paginas;
        let mut term = Terminal::new(TestBackend::new(60, 12)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();
        let cor_kanban = e.tema.estilo(Realce::EmbedKanban).fg.unwrap();
        let aparece = (0..buf.area.height)
            .any(|y| (0..buf.area.width).any(|x| buf[(x, y)].style().fg == Some(cor_kanban)));
        assert!(aparece, "a cor do kanban não apareceu na tela");
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
    fn o_cartao_do_kanban_vira_retangulo_preenchido() {
        // Ciclo 304: o cartão não fica mais em texto puro — a mesma
        // caixa de contorno fechado do botão, só que sozinho na linha
        // (a coluna empilha, não enfileira) e esticado até a borda do
        // painel, como um cartão de verdade.
        let mut e = Estado::novo(paginas(), com_kanban());
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 14).join("\n");
        assert!(
            tudo.contains("▐ Card A"),
            "o cartão não abriu como caixa preenchida:\n{tudo}"
        );
        // Sem o prefixo "card": a caixa já diz o que ele é.
        assert!(!tudo.contains("card Card A"), "{tudo}");
    }

    #[test]
    fn a_miniatura_da_galeria_vira_selo_preenchido() {
        // O terminal não desenha a imagem — o selo colorido é o que
        // sobra pra dizer "aqui tinha uma" (ciclo 304).
        let mut e = Estado::novo(paginas(), com_galeria());
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 60, 14).join("\n");
        assert!(
            tudo.contains("▐ Primeira ▌"),
            "a miniatura não virou selo preenchido:\n{tudo}"
        );
        assert!(
            tudo.contains("▐ Segunda ▌"),
            "a segunda miniatura sumiu:\n{tudo}"
        );
    }

    /// O quadro inteiro, com o estilo de cada célula.
    fn quadro(e: &mut Estado, largura: u16, altura: u16) -> ratatui::buffer::Buffer {
        let mut term = Terminal::new(TestBackend::new(largura, altura)).unwrap();
        term.draw(|f| desenhar(f, e)).unwrap();
        term.backend().buffer().clone()
    }

    /// As colunas (em caracteres) onde `c` aparece na linha.
    fn colunas_de(linha: &str, c: char) -> Vec<usize> {
        linha.chars().enumerate().filter(|(_, x)| *x == c).map(|(i, _)| i).collect()
    }

    #[test]
    fn o_calendario_vira_grade_do_mes() {
        // Até aqui cada evento era um selo solto, um embaixo do outro. A
        // janela desenha um MÊS: nome, dias da semana e as semanas em
        // grade (ciclo 306).
        let mut e = Estado::novo(paginas(), com_calendario());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 100, 40);
        let tudo = linhas.join("\n");
        assert!(tudo.contains("Agosto 2026"), "{tudo}");
        let topo = linhas.iter().find(|l| l.contains('┌') && l.contains('┬')).expect("sem borda de cima");
        let meio = linhas.iter().find(|l| l.contains('┼')).expect("sem régua entre semanas");
        let fim = linhas.iter().find(|l| l.contains('┴')).expect("sem borda de baixo");
        // Sete dias: seis divisões internas, nas MESMAS colunas em cima,
        // no meio e embaixo — senão não é grade.
        assert_eq!(colunas_de(topo, '┬').len(), 6, "{topo}");
        assert_eq!(colunas_de(topo, '┬'), colunas_de(meio, '┼'), "\n{topo}\n{meio}");
        assert_eq!(colunas_de(topo, '┬'), colunas_de(fim, '┴'), "\n{topo}\n{fim}");
        // E a linha dos números da primeira semana: 26 de julho a 1º.
        let numeros = linhas.iter().find(|l| l.contains("26") && l.contains("31")).expect("sem a primeira semana");
        // As bordas dos dias batem com as da régua: da `┌` à `┐`.
        let esquerda = colunas_de(topo, '┌')[0];
        let direita = colunas_de(topo, '┐')[0];
        let esperado: Vec<usize> = std::iter::once(esquerda)
            .chain(colunas_de(topo, '┬'))
            .chain(std::iter::once(direita))
            .collect();
        let bordas: Vec<usize> = colunas_de(numeros, '│')
            .into_iter()
            .filter(|c| (esquerda..=direita).contains(c))
            .collect();
        assert_eq!(bordas, esperado, "\n{topo}\n{numeros}");
        // O evento sem data fica na gaveta, fora da grade.
        assert!(tudo.contains("Sem data (1)"), "{tudo}");
    }

    #[test]
    fn o_evento_de_varios_dias_e_uma_barra_so() {
        // A sprint vai de segunda a sexta: o título atravessa as bordas
        // dos dias, como a barra da janela atravessa as células.
        let mut e = Estado::novo(paginas(), com_calendario());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 100, 40);
        let barra = linhas
            .iter()
            .find(|l| l.contains("Sprint de agosto"))
            .unwrap_or_else(|| panic!("a sprint sumiu:\n{}", linhas.join("\n")));
        let inicio = barra.chars().collect::<Vec<_>>();
        let col = barra.find("Sprint").map(|b| barra[..b].chars().count()).unwrap();
        // Da coluna do título até o fim da barra não há borda de dia:
        // procura o próximo `│` depois do título e confere que ele está
        // bem depois do fim da primeira célula.
        let proxima_borda = inicio.iter().enumerate().skip(col).find(|(_, c)| **c == '│').unwrap().0;
        assert!(
            proxima_borda - col > "Sprint de agosto".len(),
            "a barra parou na borda do primeiro dia:\n{barra}"
        );
    }

    #[test]
    fn o_evento_com_tag_tem_a_cor_do_badge() {
        // tags do calendário: [infra, urgente] — a sprint (infra) é a
        // primeira, `badge--info`; a revisão (urgente), `badge--success`.
        let mut e = Estado::novo(paginas(), com_calendario());
        e.foco = Foco::Paginas;
        let info = e.tema.pilula(Realce::BadgeInfo);
        let sucesso = e.tema.pilula(Realce::BadgeSucesso);
        let buf = quadro(&mut e, 100, 40);
        let tem = |estilo: Style| {
            (0..buf.area.height).any(|y| {
                (0..buf.area.width).any(|x| {
                    let s = buf[(x, y)].style();
                    s.bg == estilo.bg && s.fg == estilo.fg
                })
            })
        };
        assert!(tem(info), "a sprint não saiu com a cor de badge--info");
        assert!(tem(sucesso), "a revisão não saiu com a cor de badge--success");
    }

    #[test]
    fn o_dia_sob_o_cursor_acende() {
        let mut e = Estado::novo(paginas(), com_calendario());
        // embed → mês → segunda semana → quinta-feira, dia 6.
        e.cursor = vec![0, 0, 1, 4];
        e.foco = Foco::Conteudo;
        let cursor = e.tema.estilo(Realce::Cursor);
        let buf = quadro(&mut e, 100, 40);
        let acesos: String = (0..buf.area.height)
            .flat_map(|y| (0..buf.area.width).map(move |x| (x, y)))
            .filter(|p| buf[*p].style().bg == cursor.bg && cursor.bg.is_some())
            .map(|p| buf[p].symbol().to_string())
            .collect();
        assert!(acesos.contains('6'), "o dia 6 não acendeu: {acesos:?}");
    }

    #[test]
    fn dia_com_mais_de_tres_eventos_mostra_quantos_sobraram() {
        // Três faixas, como na janela (`MAX_LANES`); o quarto evento vira
        // "+1 mais" na célula do dia.
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"calendar\" }}\nentries:\n\
                 - date: 2026-08-12\n  title: Um\n\
                 - date: 2026-08-12\n  title: Dois\n\
                 - date: 2026-08-12\n  title: Três\n\
                 - date: 2026-08-12\n  title: Quatro\n\
                 {{ /calendar }}\n",
            ),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 120, 60).join("\n");
        assert!(tudo.contains("+1 mais"), "{tudo}");
        assert!(!tudo.contains("Quatro"), "o quarto evento devia estar no +1:\n{tudo}");
    }

    #[test]
    fn a_grade_nao_quebra_num_terminal_estreito() {
        let mut e = Estado::novo(paginas(), com_calendario());
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 30, 40).join("\n");
        assert!(tudo.contains('┬'), "{tudo}");
    }

    #[test]
    fn entrar_num_dia_pousa_no_primeiro_evento_e_j_anda_entre_eles() {
        // Os eventos do dia são navegáveis (ciclo 309): Enter no dia vai
        // pro primeiro, `j`/`k` andam entre eles pulando faixa vazia,
        // Backspace volta pro dia.
        let arvore = analisar(
            "{{ type: \"calendar\" }}\nentries:\n\
             - date: 2026-08-10\n  title: Sprint\n  end_date: 2026-08-14\n\
             - date: 2026-08-12\n  title: Reunião\n\
             - date: 2026-08-12\n  title: Almoço\n\
             {{ /calendar }}\n",
        );
        // embed → mês → semana de 9 a 15 → quarta, dia 12. As faixas
        // dele: [continuação da sprint, Reunião, Almoço].
        let dia: Caminho = vec![0, 0, 2, 3];
        // Desde o ciclo 310 a continuação da sprint é destino: é ela o
        // primeiro evento do dia 12.
        let sprint = tela::andar(&arvore, &dia, Passo::Entrar);
        assert_eq!(sprint, vec![0, 0, 2, 3, 0]);
        assert_eq!(arvore.em(&sprint).unwrap().texto, "Sprint");
        let primeiro = tela::andar(&arvore, &sprint, Passo::Proximo);
        assert_eq!(primeiro, vec![0, 0, 2, 3, 1]);
        assert_eq!(arvore.em(&primeiro).unwrap().texto, "Reunião");
        let segundo = tela::andar(&arvore, &primeiro, Passo::Proximo);
        assert_eq!(arvore.em(&segundo).unwrap().texto, "Almoço");
        assert_eq!(tela::andar(&arvore, &segundo, Passo::Proximo), segundo);
        assert_eq!(tela::andar(&arvore, &sprint, Passo::Anterior), sprint);
        assert_eq!(tela::andar(&arvore, &segundo, Passo::Sair), dia);
        // Dia sem evento nenhum: Enter não leva a lugar sem linha.
        let domingo: Caminho = vec![0, 0, 2, 0];
        assert_eq!(tela::andar(&arvore, &domingo, Passo::Entrar), domingo);
    }

    #[test]
    fn no_meio_da_barra_o_cursor_acende_a_barra_inteira() {
        // Quarta, dia 12, no meio da sprint de 10 a 14: Enter no dia
        // pousa na continuação, e é a barra TODA que acende — o título
        // está no dia 10, e é ali que a cor do cursor tem que aparecer.
        let mut e = Estado::novo(paginas(), com_calendario());
        let dia: Caminho = vec![0, 0, 2, 3];
        e.cursor = tela::andar(&e.arvore, &dia, Passo::Entrar);
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Sprint de agosto");
        e.foco = Foco::Conteudo;
        let cursor = e.tema.estilo(Realce::Cursor);
        let linhas = desenho(&mut e, 140, 40);
        let buf = quadro(&mut e, 140, 40);
        let y = linhas
            .iter()
            .position(|l| l.contains("Sprint"))
            .unwrap_or_else(|| panic!("a sprint sumiu:\n{}", linhas.join("\n")));
        let linha = &linhas[y];
        let inicio = linha.find("Sprint").map(|b| linha[..b].chars().count()).unwrap() as u16;
        // O título (no dia 10) e o trecho que passa pelo dia 12.
        assert_eq!(buf[(inicio, y as u16)].style().bg, cursor.bg, "o começo da barra não acendeu");
        let fim_do_titulo = inicio + "Sprint de agosto".chars().count() as u16 + 12;
        assert_eq!(buf[(fim_do_titulo, y as u16)].style().bg, cursor.bg, "o meio da barra não acendeu");
    }

    #[test]
    fn h_e_l_andam_de_dia_com_o_evento_selecionado() {
        // Sprint de 10 a 14 na faixa 0; Reunião no dia 12 e Almoço no 15,
        // ambos na faixa 1 (a do 15 fica vaga na 0).
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"calendar\" }}\nentries:\n\
                 - date: 2026-08-06\n  title: Revisão\n\
                 - date: 2026-08-10\n  title: Sprint\n  end_date: 2026-08-14\n\
                 - date: 2026-08-12\n  title: Reunião\n\
                 - date: 2026-08-17\n  title: Retro\n\
                 {{ /calendar }}\n",
            ),
        );
        e.foco = Foco::Conteudo;
        let texto = |e: &Estado| e.arvore.em(&e.cursor).unwrap().texto.clone();
        // Começo da sprint: segunda, dia 10.
        e.cursor = vec![0, 0, 2, 1, 0];
        assert_eq!(texto(&e), "Sprint");
        // `l` percorre a barra dia a dia, mantendo a faixa.
        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![0, 0, 2, 2, 0]);
        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![0, 0, 2, 3, 0], "no dia 12 devia seguir na sprint, não na reunião");
        assert_eq!(texto(&e), "Sprint");
        // `3l`: 13, 14 e — acabou a barra, e o 15 e o 16 não têm evento —
        // a Retro no dia 17, já na semana seguinte.
        tecla(&mut e, "3");
        tecla(&mut e, "l");
        assert_eq!(texto(&e), "Retro");
        assert_eq!(e.cursor, vec![0, 0, 3, 1, 0]);
        // E de volta: `h` pula os dias vazios até o fim da sprint.
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0, 2, 5, 0]);
        // Da Reunião (faixa 1 do dia 12), `h` vai pro dia 11, onde a faixa
        // 1 está vazia: cai no primeiro evento de lá, a sprint.
        e.cursor = vec![0, 0, 2, 3, 1];
        assert_eq!(texto(&e), "Reunião");
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0, 2, 2, 0]);
        // Na ponta: nada antes da Revisão, o cursor fica.
        e.cursor = vec![0, 0, 1, 4, 0];
        assert_eq!(texto(&e), "Revisão");
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0, 1, 4, 0]);
    }

    #[test]
    fn h_e_l_atravessam_os_meses() {
        // Agosto termina com uma viagem que entra em setembro; outubro
        // tem um evento, e novembro nenhum (não tem grade).
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"calendar\" }}\nentries:\n\
                 - date: 2026-08-28\n  title: Revisão\n\
                 - date: 2026-08-31\n  title: Viagem\n  end_date: 2026-09-02\n\
                 - date: 2026-10-05\n  title: Entrega\n\
                 {{ /calendar }}\n",
            ),
        );
        e.foco = Foco::Conteudo;
        let onde = |e: &Estado| {
            let u = e.arvore.em(&e.cursor).unwrap();
            let mes = e.arvore.em(&e.cursor[..2]).unwrap().texto.clone();
            (mes, u.texto.clone())
        };
        // Três meses de grade: agosto, setembro, outubro.
        assert_eq!(e.arvore.filhos[0].filhos.len(), 3);
        // Revisão, sexta 28 de agosto: semana 5 (23–29), coluna 5.
        e.cursor = vec![0, 0, 4, 5, 0];
        assert_eq!(onde(&e), ("Agosto 2026".into(), "Revisão".into()));
        tecla(&mut e, "l");
        assert_eq!(onde(&e), ("Agosto 2026".into(), "Viagem".into()));
        // Do dia 31 de agosto (a última célula DO MÊS), `l` pula as
        // células de setembro no fim da grade de agosto e vai pro 1º de
        // setembro na grade de SETEMBRO.
        tecla(&mut e, "l");
        assert_eq!(onde(&e), ("Setembro 2026".into(), "Viagem".into()));
        tecla(&mut e, "l");
        assert_eq!(onde(&e), ("Setembro 2026".into(), "Viagem".into()));
        // Acabou a viagem (dia 2); o resto de setembro é vazio: outubro.
        tecla(&mut e, "l");
        assert_eq!(onde(&e), ("Outubro 2026".into(), "Entrega".into()));
        tecla(&mut e, "l");
        assert_eq!(onde(&e), ("Outubro 2026".into(), "Entrega".into()), "depois da entrega não há nada");
        // E de volta, sem nunca voltar no tempo.
        tecla(&mut e, "h");
        assert_eq!(onde(&e), ("Setembro 2026".into(), "Viagem".into()));
        let setembro_2 = e.cursor.clone();
        tecla(&mut e, "h");
        assert_ne!(e.cursor, setembro_2);
        assert_eq!(onde(&e), ("Setembro 2026".into(), "Viagem".into()));
        tecla(&mut e, "h");
        assert_eq!(onde(&e), ("Agosto 2026".into(), "Viagem".into()));
    }

    #[test]
    fn o_rotulo_do_mes_volta_a_ser_ano_e_mes() {
        for m in 1..=12 {
            let rotulo = format!("{} 2026", anotadinho_core::date_util::month_name(m));
            assert_eq!(tela::ano_e_mes(&rotulo), Some((2026, m)), "{rotulo}");
        }
        assert_eq!(tela::ano_e_mes("Sem data (1)"), None);
    }

    #[test]
    fn o_evento_e_o_dia_selecionados_mostram_o_detalhe() {
        let mut e = Estado::novo(paginas(), com_calendario());
        e.foco = Foco::Conteudo;
        // Revisão, quinta 6 de agosto.
        e.cursor = vec![0, 0, 1, 4, 0];
        let tudo = desenho(&mut e, 140, 45).join("\n");
        assert!(tudo.contains("Revisão de código · 06/08/2026 · #urgente"), "{tudo}");
        // O dia 9, sem evento: a data por extenso, sem o "9 ·" na frente.
        e.cursor = vec![0, 0, 2, 0];
        let tudo = desenho(&mut e, 140, 45).join("\n");
        assert!(tudo.contains("domingo, 9 de agosto de 2026 · sem eventos"), "{tudo}");
        assert!(!tudo.contains("9 · domingo"), "{tudo}");
    }

    #[test]
    fn com_hoje_o_calendario_mostra_o_mes_de_hoje_como_a_janela() {
        // Os eventos de `com_calendario` são de agosto; hoje é 17 de
        // setembro. A janela abre no mês de hoje, e a TUI também.
        let mut e = Estado::novo(paginas(), com_calendario()).com_hoje("2026-09-17");
        e.foco = Foco::Paginas;
        let dia_hoje = e.tema.pilula(Realce::BadgeInfo).bg;
        let linhas = desenho(&mut e, 120, 40);
        let tudo = linhas.join("\n");
        assert!(tudo.contains("Setembro 2026"), "{tudo}");
        assert!(!tudo.contains("Agosto 2026"), "{tudo}");
        assert!(!tudo.contains("Revisão"), "{tudo}");
        assert!(tudo.contains("3 eventos") && tudo.contains("t hoje"), "sem cabeçalho:\n{tudo}");
        // O 17 marcado como hoje.
        let buf = quadro(&mut e, 120, 40);
        let y = linhas.iter().position(|l| l.contains("17") && l.contains("13")).expect("sem a semana de hoje") as u16;
        let x = linhas[y as usize].find("17").map(|b| linhas[y as usize][..b].chars().count()).unwrap() as u16;
        assert_eq!(buf[(x, y)].style().bg, dia_hoje, "o dia de hoje não foi marcado");
    }

    #[test]
    fn colchetes_trocam_de_mes_e_t_volta_pra_hoje() {
        let mut e = Estado::novo(paginas(), com_calendario()).com_hoje("2026-09-17");
        e.foco = Foco::Conteudo;
        e.cursor = vec![0];
        let mes_na_tela = |e: &mut Estado| {
            desenho(e, 120, 40)
                .iter()
                .find_map(|l| ["Julho 2026", "Agosto 2026", "Setembro 2026", "Outubro 2026"].into_iter().find(|m| l.contains(m)))
                .map(str::to_string)
        };
        tecla(&mut e, "[");
        assert_eq!(mes_na_tela(&mut e).as_deref(), Some("Agosto 2026"));
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Revisão"));
        // O cursor vai pro mês, e a tecla seguinte ainda é do calendário.
        assert_eq!(e.cursor, vec![0, 1]);
        tecla(&mut e, "]");
        tecla(&mut e, "]");
        assert_eq!(mes_na_tela(&mut e).as_deref(), Some("Outubro 2026"));
        tecla(&mut e, "t");
        assert_eq!(mes_na_tela(&mut e).as_deref(), Some("Setembro 2026"));
        // Fora do calendário, `[` não faz nada disso.
        let mut fora = Estado::novo(paginas(), analisar("alfa\n")).com_hoje("2026-09-17");
        fora.foco = Foco::Conteudo;
        tecla(&mut fora, "[");
        assert_eq!(fora.cursor, vec![0]);
    }

    #[test]
    fn l_no_ultimo_evento_do_mes_traz_o_proximo_mes_com_evento() {
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"calendar\" }}\nentries:\n\
                 - date: 2026-08-28\n  title: Revisão\n\
                 - date: 2026-10-05\n  title: Entrega\n\
                 {{ /calendar }}\n",
            ),
        )
        .com_hoje("2026-08-20");
        e.foco = Foco::Conteudo;
        // embed → (cabeçalho, mês) → semana de 23 a 29 → sexta 28 → evento.
        e.cursor = vec![0, 1, 4, 5, 0];
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Revisão");
        tecla(&mut e, "l");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Entrega");
        let tudo = desenho(&mut e, 120, 40).join("\n");
        assert!(tudo.contains("Outubro 2026") && !tudo.contains("Agosto 2026"), "{tudo}");
        tecla(&mut e, "h");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Revisão");
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Agosto 2026"));
        // Nada antes de agosto: o cursor fica.
        tecla(&mut e, "h");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Revisão");
    }

    /// Um calendário com um dia cheio, pra as visões.
    fn com_agenda() -> Unidade {
        analisar(
            "{{ type: \"calendar\" }}\nentries:\n\
             - date: 2026-08-10\n  title: Sprint\n  end_date: 2026-08-14\n\
             - date: 2026-08-12\n  title: Almoço\n  start_time: '12:00'\n  end_time: '13:00'\n\
             - date: 2026-08-12\n  title: Reunião\n  start_time: '09:30'\n  tags:\n  - infra\n\
             - date: 2026-08-19\n  title: Retro\n\
             {{ /calendar }}\n",
        )
    }

    #[test]
    fn m_troca_a_visao_mes_semana_dia_na_data_do_cursor() {
        let mut e = Estado::novo(paginas(), com_agenda()).com_hoje("2026-08-20");
        e.foco = Foco::Conteudo;
        // Quarta, 12 de agosto, na grade do mês: embed → mês → semana 9–15.
        e.cursor = vec![0, 1, 2, 3];
        assert_eq!(tela::data_do_cursor(&e.arvore, &e.cursor).as_deref(), Some("2026-08-12"));
        tecla(&mut e, "m");
        let tudo = desenho(&mut e, 140, 40).join("\n");
        assert!(tudo.contains("9 – 15 de agosto de 2026"), "semana do dia 12:\n{tudo}");
        assert!(tudo.contains("m Semana"), "{tudo}");
        assert!(tudo.contains("09:30 Reun"), "o horário vem na frente na semana:\n{tudo}");
        let linhas = desenho(&mut e, 140, 40);
        let pos = |t: &str| linhas.iter().position(|l| l.contains(t)).unwrap();
        assert!(pos("09:30 Reun") < pos("12:00 Almo"), "na semana, pela hora:\n{tudo}");
        // A semana anda de 7 em 7.
        tecla(&mut e, "]");
        assert!(desenho(&mut e, 140, 40).join("\n").contains("16 – 22 de agosto de 2026"));
        tecla(&mut e, "[");
        // Dia: a agenda do dia em que o cursor está (o mês ancora no dia 12
        // de novo, e o cursor foi pro "mes" da semana — sem dia, fica a
        // âncora).
        e.cursor = vec![0, 1, 0, 3];
        tecla(&mut e, "m");
        let tudo = desenho(&mut e, 140, 40).join("\n");
        assert!(tudo.contains("quarta, 12 de agosto de 2026"), "{tudo}");
        assert!(tudo.contains("m Dia"), "{tudo}");
        tecla(&mut e, "m");
        assert!(desenho(&mut e, 140, 40).join("\n").contains("Agosto 2026"));
    }

    #[test]
    fn a_agenda_do_dia_lista_na_ordem_e_anda_entre_dias() {
        let mut e = Estado::novo(paginas(), com_agenda()).com_hoje("2026-08-12");
        e.foco = Foco::Conteudo;
        e.cursor = vec![0];
        tecla(&mut e, "m");
        tecla(&mut e, "m");
        let linhas = desenho(&mut e, 120, 30);
        let tudo = linhas.join("\n");
        // Dia inteiro primeiro, depois pela hora.
        let pos = |t: &str| linhas.iter().position(|l| l.contains(t)).unwrap_or_else(|| panic!("{t} sumiu:\n{tudo}"));
        assert!(pos("Sprint") < pos("Reunião") && pos("Reunião") < pos("Almoço"), "{tudo}");
        assert!(linhas[pos("Sprint")].contains("dia inteiro"));
        assert!(linhas[pos("Reunião")].contains("09:30"));
        assert!(linhas[pos("Almoço")].contains("12:00–13:00"));
        // Enter na agenda pousa no primeiro; j anda; o detalhe acompanha.
        tecla(&mut e, "Enter");
        let titulo = |e: &Estado| e.arvore.em(&e.cursor).unwrap().texto.clone();
        assert_eq!(titulo(&e), "Sprint");
        tecla(&mut e, "j");
        assert_eq!(titulo(&e), "Reunião");
        assert!(desenho(&mut e, 120, 30).join("\n").contains("Reunião · 12/08/2026 · 09:30 · #infra"));
        // `l`: próximo dia com evento é o 13 (a sprint continua).
        tecla(&mut e, "l");
        assert!(desenho(&mut e, 120, 30).join("\n").contains("quinta, 13 de agosto de 2026"));
        assert_eq!(titulo(&e), "Sprint");
        // Do 13, `l` pula 15–18 vazios... o 14 ainda tem a sprint.
        tecla(&mut e, "l");
        assert!(desenho(&mut e, 120, 30).join("\n").contains("sexta, 14 de agosto de 2026"));
        tecla(&mut e, "l");
        assert!(desenho(&mut e, 120, 30).join("\n").contains("quarta, 19 de agosto de 2026"));
        assert_eq!(titulo(&e), "Retro");
        // Dia sem nada: "sem eventos", e `]` anda um dia.
        tecla(&mut e, "]");
        let tudo = desenho(&mut e, 120, 30).join("\n");
        assert!(tudo.contains("quinta, 20 de agosto de 2026") && tudo.contains("sem eventos"), "{tudo}");
    }

    #[test]
    fn calendario_em_modo_vault_mostra_as_paginas_com_data_e_enter_abre() {
        let arvore = analisar("{{ type: \"calendar\" }}\nmode: vault\n{{ /calendar }}\n");
        let eventos = anotadinho_core::calendario::entradas_do_vault(&[anotadinho_core::PageIndexEntry {
            path: "journals/diario.md".into(),
            title: "Diário de 19".into(),
            properties: [("date".to_string(), "2026-08-19".to_string())].into_iter().collect(),
            ..Default::default()
        }]);
        let mut paginas = paginas();
        paginas.push(PageMeta {
            path: "journals/diario.md".into(),
            title: "Diário de 19".into(),
            section: "journals".into(),
        });
        let mut e = Estado::novo(paginas, arvore).com_hoje("2026-08-10").com_eventos_do_vault(eventos);
        e.foco = Foco::Conteudo;
        let tudo = desenho(&mut e, 140, 40).join("\n");
        assert!(tudo.contains("Diário d") && tudo.contains("1 evento"), "{tudo}");
        // Quarta, 19 de agosto: semana 16–22, coluna 3, a primeira faixa.
        e.cursor = vec![0, 1, 3, 3, 0];
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Diário de 19");
        assert!(desenho(&mut e, 140, 40).join("\n").contains("journals/diario.md"), "o detalhe diz a página");
        assert_eq!(tecla(&mut e, "Enter").as_deref(), Some("journals/diario.md"));
        assert_eq!(e.paginas[e.pagina].path, "journals/diario.md");
        // Evento escrito no embed (não do vault) não abre nada.
        let mut m = Estado::novo(super::testes::paginas(), com_calendario()).com_hoje("2026-08-10");
        m.foco = Foco::Conteudo;
        m.cursor = vec![0, 1, 1, 4, 0];
        assert_eq!(m.arvore.em(&m.cursor).unwrap().texto, "Revisão de código");
        assert_eq!(tecla(&mut m, "Enter"), None);
    }

    /// Uma página editável com texto antes e depois do calendário.
    const PAGINA_COM_CALENDARIO: &str = "---\ntitle: Agenda\n---\n\nAntes do calendário.\n\n{{ type: \"calendar\" }}\nentries:\n- date: 2026-08-10\n  title: Sprint\n  end_date: 2026-08-14\n- date: 2026-08-12\n  title: Reunião\n{{ /calendar }}\n\nDepois do calendário.\n";

    fn editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_CALENDARIO, Some("v1".into()));
        let mut e = e.com_hoje("2026-08-12");
        e.foco = Foco::Conteudo;
        e
    }

    fn digitar(e: &mut Estado, texto: &str) {
        for c in texto.chars() {
            tecla(e, &c.to_string());
        }
    }

    /// O começo do evento `titulo` na grade.
    fn achar_evento_na_tela(e: &Estado, titulo: &str) -> Caminho {
        e.arvore
            .percorrer()
            .into_iter()
            .find(|(_, u)| {
                u.texto == titulo
                    && matches!(&u.tipo, Tipo::Parte { nome, .. } if nome.starts_with("evento") && nome != "evento-continua")
            })
            .map(|(c, _)| c)
            .expect("evento fora da tela")
    }

    /// Os dados do calendário no texto que ficou pra gravar.
    fn gravado(e: &Estado) -> anotadinho_core::embed::CalendarEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("---\ntitle: Agenda\n---\n\nAntes do calendário.\n\n"), "o começo mudou:\n{texto}");
        assert!(texto.ends_with("\n\nDepois do calendário.\n"), "o fim mudou:\n{texto}");
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
        dados_do_calendario(corpo).expect("o calendário sumiu do texto")
    }

    #[test]
    fn o_cria_um_evento_no_dia_do_cursor() {
        let mut e = editavel();
        // O dia 20: semana de 16 a 22, quinta.
        e.cursor = vec![1, 1, 3, 4];
        assert_eq!(tela::data_do_cursor(&e.arvore, &e.cursor).as_deref(), Some("2026-08-20"));
        tecla(&mut e, "o");
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Novo evento em 20/08/2026"));
        digitar(&mut e, "Deploy");
        tecla(&mut e, "Enter");
        let d = gravado(&e);
        assert_eq!(d.entries.len(), 3);
        assert_eq!(d.entries[2].title, "Deploy");
        assert_eq!(d.entries[2].date.as_deref(), Some("2026-08-20"));
        // O cursor pousa no evento novo, e a tela já o mostra.
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Deploy");
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Deploy"));
    }

    #[test]
    fn escape_cancela_a_pergunta_sem_gravar() {
        let mut e = editavel();
        e.cursor = vec![1, 1, 3, 4];
        tecla(&mut e, "o");
        digitar(&mut e, "Nada");
        tecla(&mut e, "Escape");
        assert!(e.pergunta.is_none() && e.gravacao.is_none());
        // E o `j` volta a ser movimento, não texto.
        tecla(&mut e, "j");
        assert!(e.pergunta.is_none());
    }

    #[test]
    fn a_renomeia_e_x_apaga_o_evento() {
        let mut e = editavel();
        // A Reunião: semana 9–15, quarta 12, faixa 1.
        e.cursor = vec![1, 1, 2, 3, 1];
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Reunião");
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Reunião", "o título atual vem preenchido");
        for _ in 0.."Reunião".chars().count() {
            tecla(&mut e, "Backspace");
        }
        digitar(&mut e, "Daily");
        tecla(&mut e, "Enter");
        assert_eq!(gravado(&e).entries[1].title, "Daily");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Daily");
        e.gravacao = None;
        tecla(&mut e, "x");
        let d = gravado(&e);
        assert_eq!(d.entries.len(), 1);
        assert_eq!(d.entries[0].title, "Sprint");
        assert!(desenho(&mut e, 120, 40).join("\n").contains("evento apagado"));
    }

    #[test]
    fn dd_tambem_apaga_o_evento() {
        let mut e = editavel();
        e.cursor = vec![1, 1, 2, 1, 0];
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Sprint");
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(gravado(&e).entries.len(), 1);
        assert_eq!(gravado(&e).entries[0].title, "Reunião");
    }

    #[test]
    fn maior_maior_move_o_evento_preservando_a_duracao() {
        let mut e = editavel();
        // A Sprint vista do dia 12 (no meio da barra).
        e.cursor = vec![1, 1, 2, 3, 0];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let d = gravado(&e);
        assert_eq!(d.entries[0].date.as_deref(), Some("2026-08-11"));
        assert_eq!(d.entries[0].end_date.as_deref(), Some("2026-08-15"));
        // O cursor segue o evento, no dia seguinte.
        assert_eq!(tela::data_do_cursor(&e.arvore, &e.cursor).as_deref(), Some("2026-08-13"));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Sprint");
        tecla(&mut e, "<");
        tecla(&mut e, "<");
        tecla(&mut e, "<");
        tecla(&mut e, "<");
        assert_eq!(gravado(&e).entries[0].date.as_deref(), Some("2026-08-09"));
    }

    #[test]
    fn mover_pra_fora_do_mes_leva_a_ancora_junto() {
        let mut e = editavel();
        e.ancorar(&[1], "2026-08-01".into());
        // Reunião do dia 12 → move 20 vezes pra frente: 1º de setembro.
        e.cursor = vec![1, 1, 2, 3, 1];
        for _ in 0..20 {
            tecla(&mut e, ">");
            tecla(&mut e, ">");
        }
        assert_eq!(gravado(&e).entries[1].date.as_deref(), Some("2026-09-01"));
        // 1º de setembro ainda está na grade de agosto (as células do fim):
        // o cursor fica ali, como o arrastar da janela.
        assert_eq!(tela::data_do_cursor(&e.arvore, &e.cursor).as_deref(), Some("2026-09-01"));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Reunião");
        // Mais cinco: dia 6, fora da grade de agosto — a âncora vai junto.
        for _ in 0..5 {
            tecla(&mut e, ">");
            tecla(&mut e, ">");
        }
        assert_eq!(gravado(&e).entries[1].date.as_deref(), Some("2026-09-06"));
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Setembro 2026"));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Reunião");
    }

    #[test]
    fn calendario_do_vault_nao_se_edita() {
        let texto = "{{ type: \"calendar\" }}\nmode: vault\n{{ /calendar }}\n";
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(texto, None);
        let mut e = e.com_hoje("2026-08-12");
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 1, 2, 3];
        tecla(&mut e, "o");
        digitar(&mut e, "X");
        tecla(&mut e, "Enter");
        assert!(e.gravacao.is_none());
        assert!(desenho(&mut e, 120, 40).join("\n").contains("só leitura"));
    }

    #[test]
    fn ctrl_a_estica_o_evento_e_yy_p_cola_no_dia_do_cursor() {
        let mut e = editavel();
        let reuniao = achar_evento_na_tela(&e, "Reunião");
        e.cursor = reuniao;
        tecla(&mut e, "3");
        tecla(&mut e, "Ctrl+a");
        assert_eq!(gravado(&e).entries[1].end_date.as_deref(), Some("2026-08-15"));
        tecla(&mut e, "3");
        tecla(&mut e, "Ctrl+x");
        assert_eq!(gravado(&e).entries[1].end_date, None);
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        let mut dia = e.cursor.clone();
        dia.pop();
        *dia.last_mut().unwrap() += 1;
        e.cursor = dia;
        tecla(&mut e, "p");
        let d = gravado(&e);
        assert_eq!(d.entries.len(), 3);
        assert_eq!(d.entries[2].title, "Reunião");
        assert_eq!(d.entries[2].date.as_deref(), Some("2026-08-13"));
    }

    const PAGINA_COM_KANBAN: &str = "Antes.\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\n- Fazendo\n- Feito\nitems:\n- title: Escrever\n  column: Backlog\n  tags:\n  - doc\n- title: Revisar\n  column: Fazendo\n{{ /kanban }}\n\nDepois.\n";

    fn kanban_editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_KANBAN, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    fn kanban_gravado(e: &Estado) -> anotadinho_core::embed::KanbanEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("Antes.\n\n") && texto.ends_with("\n\nDepois.\n"), "{texto}");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Kanban(d)) => Some(d),
                _ => None,
            })
            .expect("o kanban sumiu")
    }

    #[test]
    fn o_cria_cartao_na_coluna_e_a_renomeia() {
        let mut e = kanban_editavel();
        // Na coluna "Feito" (vazia).
        e.cursor = vec![1, 2];
        tecla(&mut e, "o");
        assert!(desenho(&mut e, 100, 30).join("\n").contains("Novo cartão em Feito"));
        digitar(&mut e, "Publicar");
        tecla(&mut e, "Enter");
        let d = kanban_gravado(&e);
        assert_eq!(d.items.len(), 3);
        assert_eq!((d.items[2].title.as_str(), d.items[2].column.as_str()), ("Publicar", "Feito"));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Publicar");
        // `c` no cartão.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Escrever");
        tecla(&mut e, "Backspace");
        tecla(&mut e, "Backspace");
        digitar(&mut e, "am");
        tecla(&mut e, "Enter");
        let d = kanban_gravado(&e);
        assert_eq!(d.items[0].title, "Escrevam");
        // Os campos que a TUI não mexe continuam lá.
        assert_eq!(d.items[0].tags, vec!["doc".to_string()]);
    }

    #[test]
    fn a_na_coluna_renomeia_a_coluna_e_os_cartoes_seguem() {
        let mut e = kanban_editavel();
        e.cursor = vec![1, 1];
        tecla(&mut e, "a");
        for _ in 0.."Fazendo".len() {
            tecla(&mut e, "Backspace");
        }
        digitar(&mut e, "Em andamento");
        tecla(&mut e, "Enter");
        let d = kanban_gravado(&e);
        assert_eq!(d.columns[1], "Em andamento");
        assert_eq!(d.items[1].column, "Em andamento");
    }

    #[test]
    fn maior_maior_leva_o_cartao_de_coluna_e_x_apaga() {
        let mut e = kanban_editavel();
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert_eq!(kanban_gravado(&e).items[0].column, "Fazendo");
        // O cursor foi junto, pra coluna do meio.
        assert_eq!(e.cursor[..2], [1, 1]);
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Escrever");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert_eq!(kanban_gravado(&e).items[0].column, "Feito");
        assert!(desenho(&mut e, 100, 30).join("\n").contains("não há coluna desse lado"));
        tecla(&mut e, "x");
        let d = kanban_gravado(&e);
        assert_eq!(d.items.len(), 1);
        assert_eq!(d.items[0].title, "Revisar");
        assert_eq!(e.cursor, vec![1, 2]);
        // `dd` também.
        e.cursor = vec![1, 1, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert!(kanban_gravado(&e).items.is_empty());
    }

    #[test]
    fn o_maiusculo_cria_antes_e_cc_comeca_vazio() {
        let mut e = kanban_editavel();
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "O");
        digitar(&mut e, "Planejar");
        tecla(&mut e, "Enter");
        let d = kanban_gravado(&e);
        assert_eq!(d.items.iter().map(|c| c.title.as_str()).collect::<Vec<_>>(), vec!["Planejar", "Escrever", "Revisar"]);
        assert_eq!(e.cursor, vec![1, 0, 0]);
        tecla(&mut e, "c");
        tecla(&mut e, "c");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "");
        tecla(&mut e, "Escape");
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Planejar");
    }

    #[test]
    fn yy_p_duplica_e_dd_p_move_o_cartao() {
        let mut e = kanban_editavel();
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        assert!(e.gravacao.is_none());
        // Colar na coluna Feito, vazia.
        e.cursor = vec![1, 2];
        tecla(&mut e, "p");
        let d = kanban_gravado(&e);
        assert_eq!(d.items[2].title, "Escrever");
        assert_eq!(d.items[2].column, "Feito");
        assert_eq!(d.items[2].tags, vec!["doc".to_string()]);
        e.cursor = vec![1, 1, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "P");
        let d = kanban_gravado(&e);
        let backlog: Vec<_> = d.items.iter().filter(|c| c.column == "Backlog").map(|c| c.title.as_str()).collect();
        assert_eq!(backlog, vec!["Revisar", "Escrever"]);
    }

    #[test]
    fn contagem_no_maior_maior_e_u_desfaz_e_ctrl_r_refaz() {
        let mut e = kanban_editavel();
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "2");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert_eq!(kanban_gravado(&e).items[0].column, "Feito");
        assert_eq!(e.cursor, vec![1, 2, 0]);
        tecla(&mut e, "u");
        assert_eq!(e.gravacao.as_deref(), Some(PAGINA_COM_KANBAN));
        assert_eq!(e.cursor, vec![1, 0, 0]);
        tecla(&mut e, "u");
        assert_eq!(e.aviso.as_deref(), Some("nada pra desfazer"));
        tecla(&mut e, "Ctrl+r");
        assert_eq!(kanban_gravado(&e).items[0].column, "Feito");
        // Abrir outra página esquece o histórico.
        e.abrir_texto(PAGINA_COM_KANBAN, Some("v2".into()));
        tecla(&mut e, "u");
        assert_eq!(e.aviso.as_deref(), Some("nada pra desfazer"));
    }

    #[test]
    fn fora_de_embed_os_comandos_de_edicao_nao_fazem_nada() {
        let mut e = kanban_editavel();
        e.cursor = vec![0];
        for t in ["o", "d", "d", "x", ">", ">", "Ctrl+a"] {
            tecla(&mut e, t);
        }
        assert!(e.gravacao.is_none() && e.pergunta.is_none());
    }

    const PAGINA_COM_CALLOUT: &str = "Antes.\n\n{{ type: \"callout\" }}\nvariant: warning\ntitle: Cuidado\nbody: |\n  Primeiro parágrafo.\n\n  - um\n  - dois\n\n  Último.\n{{ /callout }}\n\nDepois.\n";

    fn callout_editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_CALLOUT, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    fn callout_gravado(e: &Estado) -> anotadinho_core::embed::CalloutEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("Antes.\n\n") && texto.ends_with("\n\nDepois.\n"), "{texto}");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Callout(d)) => Some(d),
                _ => None,
            })
            .expect("o callout sumiu")
    }

    /// O caminho da unidade com esse texto dentro do callout.
    fn no_callout(e: &Estado, texto: &str) -> Caminho {
        e.arvore.percorrer().into_iter().find(|(c, u)| c.first() == Some(&1) && u.texto == texto).map(|(c, _)| c).expect(texto)
    }

    #[test]
    fn no_callout_a_renomeia_o_titulo_e_ctrl_a_gira_a_variante() {
        let mut e = callout_editavel();
        e.cursor = no_callout(&e, "Cuidado");
        tecla(&mut e, "A");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Cuidado");
        digitar(&mut e, "!");
        tecla(&mut e, "Enter");
        let d = callout_gravado(&e);
        assert_eq!(d.title, "Cuidado!");
        assert_eq!(d.body, "Primeiro parágrafo.\n\n- um\n- dois\n\nÚltimo.\n");
        tecla(&mut e, "Ctrl+a");
        assert_eq!(callout_gravado(&e).variant, anotadinho_core::embed::CalloutVariant::Error);
        tecla(&mut e, "2");
        tecla(&mut e, "Ctrl+x");
        assert_eq!(callout_gravado(&e).variant, anotadinho_core::embed::CalloutVariant::Success);
        tecla(&mut e, "~");
        assert!(callout_gravado(&e).collapsed);
    }

    #[test]
    fn no_corpo_do_callout_cada_bloco_se_edita_cria_e_apaga() {
        let mut e = callout_editavel();
        e.cursor = no_callout(&e, "Primeiro parágrafo.");
        tecla(&mut e, "o");
        digitar(&mut e, "Segundo.");
        tecla(&mut e, "Enter");
        assert_eq!(callout_gravado(&e).body, "Primeiro parágrafo.\n\nSegundo.\n\n- um\n- dois\n\nÚltimo.\n");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Segundo.");
        // `cc` num item mantém a marca; `o` num item cria item vizinho.
        e.cursor = no_callout(&e, "um");
        tecla(&mut e, "c");
        tecla(&mut e, "c");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "- ");
        digitar(&mut e, "primeiro");
        tecla(&mut e, "Enter");
        tecla(&mut e, "o");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "- ");
        digitar(&mut e, "meio");
        tecla(&mut e, "Enter");
        assert_eq!(callout_gravado(&e).body, "Primeiro parágrafo.\n\nSegundo.\n\n- primeiro\n- meio\n- dois\n\nÚltimo.\n");
        e.cursor = no_callout(&e, "dois");
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        e.cursor = no_callout(&e, "Último.");
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(callout_gravado(&e).body, "Primeiro parágrafo.\n\nSegundo.\n\n- primeiro\n- meio\n");
        // O que o `dd` levou, o `P` devolve.
        e.cursor = no_callout(&e, "Primeiro parágrafo.");
        tecla(&mut e, "P");
        assert_eq!(callout_gravado(&e).body, "Último.\n\nPrimeiro parágrafo.\n\nSegundo.\n\n- primeiro\n- meio\n");
    }

    const PAGINA_COM_ACOES: &str = "Antes.\n\n{{ type: \"actions\" }}\nbuttons:\n- label: Nova página\n  variant: primary\n  action: new-page\n- label: Buscar\n  action: run-search\n  query: tag\n{{ /actions }}\n\nDepois.\n";

    fn acoes_gravadas(e: &Estado) -> anotadinho_core::embed::ActionsEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("Antes.\n\n") && texto.ends_with("\n\nDepois.\n"), "{texto}");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Actions(d)) => Some(d),
                _ => None,
            })
            .expect("as ações sumiram")
    }

    #[test]
    fn nas_acoes_o_cria_a_renomeia_maior_maior_reordena_e_til_destaca() {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_ACOES, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "o");
        digitar(&mut e, "Abrir");
        tecla(&mut e, "Enter");
        let d = acoes_gravadas(&e);
        assert_eq!(d.buttons.iter().map(|b| b.label.as_str()).collect::<Vec<_>>(), vec!["Nova página", "Abrir", "Buscar"]);
        assert_eq!(d.buttons[1].action, "open-page");
        assert_eq!(e.cursor, vec![1, 0, 1]);
        tecla(&mut e, "a");
        digitar(&mut e, " hoje");
        tecla(&mut e, "Enter");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let d = acoes_gravadas(&e);
        assert_eq!(d.buttons[2].label, "Abrir hoje");
        assert_eq!(e.cursor, vec![1, 0, 2]);
        tecla(&mut e, "~");
        assert_eq!(acoes_gravadas(&e).buttons[2].variant.as_deref(), Some("primary"));
        e.cursor = vec![1, 0, 1];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let d = acoes_gravadas(&e);
        assert_eq!(d.buttons.len(), 2);
        assert_eq!(e.cursor, vec![1, 0, 1]);
        tecla(&mut e, "P");
        let d = acoes_gravadas(&e);
        assert_eq!(d.buttons[1].label, "Buscar");
        assert_eq!(d.buttons[1].query.as_deref(), Some("tag"));
    }

    const PAGINA_COM_CRONOGRAMA: &str = "Antes.\n\n{{ type: \"timeline\" }}\nitems:\n- title: Levantar\n  start: 2026-08-03\n  end: 2026-08-10\n  tags:\n  - infra\n- title: Implementar\n  start: 2026-08-11\n  end: 2026-08-24\n{{ /timeline }}\n\nDepois.\n";

    fn cronograma_editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_CRONOGRAMA, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    fn cronograma_gravado(e: &Estado) -> anotadinho_core::embed::TimelineEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("Antes.\n\n") && texto.ends_with("\n\nDepois.\n"), "{texto}");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Timeline(d)) => Some(d),
                _ => None,
            })
            .expect("o cronograma sumiu")
    }

    #[test]
    fn o_cria_barra_depois_da_do_cursor_e_a_renomeia() {
        let mut e = cronograma_editavel();
        // embed → eixo, Levantar, Implementar.
        e.cursor = vec![1, 2];
        tecla(&mut e, "o");
        assert!(desenho(&mut e, 120, 30).join("\n").contains("Nova barra de 25/08/2026 a 31/08/2026"));
        digitar(&mut e, "Publicar");
        tecla(&mut e, "Enter");
        let d = cronograma_gravado(&e);
        assert_eq!(d.items[2].title, "Publicar");
        assert_eq!(d.items[2].start.as_deref(), Some("2026-08-25"));
        assert_eq!(d.items[2].end.as_deref(), Some("2026-08-31"));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Publicar");
        e.cursor = vec![1, 1];
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Levantar");
        digitar(&mut e, " requisitos");
        tecla(&mut e, "Enter");
        let d = cronograma_gravado(&e);
        assert_eq!(d.items[0].title, "Levantar requisitos");
        assert_eq!(d.items[0].tags, vec!["infra".to_string()]);
    }

    #[test]
    fn maior_maior_move_e_ctrl_a_ctrl_x_esticam_o_fim() {
        let mut e = cronograma_editavel();
        e.cursor = vec![1, 1];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let d = cronograma_gravado(&e);
        assert_eq!((d.items[0].start.as_deref(), d.items[0].end.as_deref()), (Some("2026-08-04"), Some("2026-08-11")));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Levantar");
        tecla(&mut e, "Ctrl+a");
        tecla(&mut e, "Ctrl+a");
        let d = cronograma_gravado(&e);
        assert_eq!(d.items[0].end.as_deref(), Some("2026-08-13"));
        // `-` não passa do começo.
        for _ in 0..20 {
            tecla(&mut e, "Ctrl+x");
        }
        let d = cronograma_gravado(&e);
        assert_eq!(d.items[0].end.as_deref(), Some("2026-08-04"));
        tecla(&mut e, "<");
        tecla(&mut e, "<");
        assert_eq!(cronograma_gravado(&e).items[0].start.as_deref(), Some("2026-08-03"));
    }

    #[test]
    fn o_maiusculo_poe_a_barra_antes_e_colar_mantem_a_duracao() {
        let mut e = cronograma_editavel();
        e.cursor = vec![1, 2];
        tecla(&mut e, "O");
        assert!(e.pergunta.as_ref().unwrap().rotulo.contains("de 04/08/2026 a 10/08/2026"));
        digitar(&mut e, "Desenhar");
        tecla(&mut e, "Enter");
        let d = cronograma_gravado(&e);
        assert_eq!(d.items[1].title, "Desenhar");
        assert_eq!(d.items[2].title, "Implementar");
        // Copiar Implementar (14 dias) e colar depois dele.
        e.cursor = achar_com_indice(&e.arvore, &[1], "barra", 2).unwrap();
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        tecla(&mut e, "p");
        let d = cronograma_gravado(&e);
        assert_eq!((d.items[3].start.as_deref(), d.items[3].end.as_deref()), (Some("2026-08-25"), Some("2026-09-07")));
    }

    #[test]
    fn cronograma_do_vault_nao_se_edita() {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(&PAGINA_COM_CRONOGRAMA.replace("\"timeline\" }}\n", "\"timeline\" }}\nsource: vault\n"), Some("v1".into()));
        e.foco = Foco::Conteudo;
        // No modo vault as barras vêm das páginas; a daqui não aparece,
        // então o cursor fica no próprio embed.
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "Nova");
        tecla(&mut e, "Enter");
        assert!(e.gravacao.is_none());
        assert!(e.aviso.as_deref().unwrap_or("").contains("só leitura"));
    }

    #[test]
    fn x_e_dd_apagam_a_barra() {
        let mut e = cronograma_editavel();
        e.cursor = vec![1, 2];
        tecla(&mut e, "x");
        assert_eq!(cronograma_gravado(&e).items.len(), 1);
        e.cursor = vec![1, 1];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert!(cronograma_gravado(&e).items.is_empty());
    }

    const PAGINA_COM_TABELA: &str = "Antes.\n\n{{ type: \"table\" }}\ncolumns:\n- name: Tarefa\n- name: Status\n  type: select\n  options: [todo, done]\n- name: Tags\n  type: multiselect\n  options: [api]\n---\n| Tarefa | Status | Tags |\n| --- | --- | --- |\n| API | done | api |\n| Docs | todo |  |\n{{ /table }}\n\nDepois.\n";

    fn tabela_editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_COM_TABELA, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    fn tabela_gravada(e: &Estado) -> anotadinho_core::embed::TableEmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("Antes.\n\n") && texto.ends_with("\n\nDepois.\n"), "{texto}");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Table(d)) => Some(d),
                _ => None,
            })
            .expect("a tabela sumiu")
    }

    fn opcoes(d: &anotadinho_core::embed::TableEmbedData, coluna: usize) -> Vec<String> {
        match &d.columns[coluna].kind {
            anotadinho_core::embed::ColumnKind::Select { options }
            | anotadinho_core::embed::ColumnKind::MultiSelect { options } => options.clone(),
            _ => vec![],
        }
    }

    #[test]
    fn a_edita_a_celula_e_valor_novo_vira_opcao() {
        let mut e = tabela_editavel();
        // tabela → cabeçalho, API, Docs.
        e.cursor = vec![1, 2, 0];
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "Docs");
        assert_eq!(e.pergunta.as_ref().unwrap().rotulo, "Tarefa");
        digitar(&mut e, " da API");
        tecla(&mut e, "Enter");
        assert_eq!(tabela_gravada(&e).rows[1][0], "Docs da API");
        e.cursor = vec![1, 2, 1];
        tecla(&mut e, "a");
        for _ in 0..4 {
            tecla(&mut e, "Backspace");
        }
        digitar(&mut e, "doing");
        tecla(&mut e, "Enter");
        let d = tabela_gravada(&e);
        assert_eq!(d.rows[1][1], "doing");
        assert_eq!(opcoes(&d, 1), vec!["todo", "done", "doing"]);
        e.cursor = vec![1, 2, 2];
        tecla(&mut e, "a");
        digitar(&mut e, "api ,web,");
        tecla(&mut e, "Enter");
        let d = tabela_gravada(&e);
        assert_eq!(d.rows[1][2], "api, web");
        assert_eq!(opcoes(&d, 2), vec!["api", "web"]);
        assert_eq!(d.rows[0], vec!["API", "done", "api"]);
    }

    #[test]
    fn ctrl_a_gira_o_select_e_yy_p_copia_a_linha() {
        let mut e = tabela_editavel();
        e.cursor = vec![1, 2, 1];
        tecla(&mut e, "Ctrl+a");
        assert_eq!(tabela_gravada(&e).rows[1][1], "done");
        tecla(&mut e, "Ctrl+a");
        assert_eq!(tabela_gravada(&e).rows[1][1], "todo");
        tecla(&mut e, "Ctrl+x");
        assert_eq!(tabela_gravada(&e).rows[1][1], "done");
        e.cursor = vec![1, 1, 0];
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        e.cursor = vec![1, 2, 0];
        tecla(&mut e, "p");
        let d = tabela_gravada(&e);
        assert_eq!(d.rows.len(), 3);
        assert_eq!(d.rows[2], vec!["API", "done", "api"]);
        assert_eq!(e.cursor, vec![1, 3, 0]);
        tecla(&mut e, "O");
        assert_eq!(tabela_gravada(&e).rows[2], vec!["", "", ""]);
    }

    #[test]
    fn a_no_cabecalho_renomeia_e_x_limpa_a_celula() {
        let mut e = tabela_editavel();
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "a");
        assert_eq!(e.pergunta.as_ref().unwrap().rotulo, "Renomear coluna");
        digitar(&mut e, "s");
        tecla(&mut e, "Enter");
        let d = tabela_gravada(&e);
        assert_eq!(d.columns[0].name, "Tarefas");
        assert_eq!(opcoes(&d, 1), vec!["todo", "done"]);
        e.cursor = vec![1, 1, 1];
        tecla(&mut e, "x");
        assert_eq!(tabela_gravada(&e).rows[0], vec!["API", "", "api"]);
    }

    #[test]
    fn o_cria_linha_embaixo_e_dd_apaga() {
        let mut e = tabela_editavel();
        e.cursor = vec![1, 1, 2];
        tecla(&mut e, "o");
        let d = tabela_gravada(&e);
        assert_eq!(d.rows.len(), 3);
        assert_eq!(d.rows[1], vec!["", "", ""]);
        assert_eq!(d.rows[2][0], "Docs");
        assert_eq!(e.cursor, vec![1, 2, 0]);
        tecla(&mut e, "a");
        digitar(&mut e, "Testes");
        tecla(&mut e, "Enter");
        assert_eq!(tabela_gravada(&e).rows[1][0], "Testes");
        e.cursor = vec![1, 1, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let d = tabela_gravada(&e);
        assert_eq!(d.rows.iter().map(|r| r[0].as_str()).collect::<Vec<_>>(), vec!["Testes", "Docs"]);
        // O cabeçalho não se apaga.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(tabela_gravada(&e).columns.len(), 3);
        assert!(e.aviso.is_some());
    }

    #[test]
    fn o_evento_sob_o_cursor_acende_na_grade() {
        let mut e = Estado::novo(paginas(), com_calendario());
        // embed → mês → segunda semana → quinta, dia 6 → Revisão.
        e.cursor = vec![0, 0, 1, 4, 0];
        e.foco = Foco::Conteudo;
        let cursor = e.tema.estilo(Realce::Cursor);
        let linhas = desenho(&mut e, 140, 40);
        let buf = quadro(&mut e, 140, 40);
        let y = linhas.iter().position(|l| l.contains("Revis")).unwrap_or_else(|| panic!("a revisão sumiu:\n{}", linhas.join("\n"))) as u16;
        let x = linhas[y as usize].find("Revis").map(|b| linhas[y as usize][..b].chars().count()).unwrap() as u16;
        assert_eq!(buf[(x, y)].style().bg, cursor.bg, "o evento sob o cursor não acendeu");
        // E a tela considera o cursor visível: a semana MOSTRA o evento.
        let l = tela::linhas(&e.arvore);
        assert!(tela::linha_de(&l, &e.cursor).is_some());
    }

    #[test]
    fn a_barra_do_cronograma_fica_proporcional_a_duracao() {
        // As duas etapas de `com_cronograma()` duram o mesmo tanto (10
        // dias cada, janela de 20) — a segunda barra tem que começar
        // exatamente onde a primeira parou, não randomicamente lado a
        // lado nem uma embaixo da outra na mesma coluna.
        let mut e = Estado::novo(paginas(), com_cronograma());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 80, 16);
        let linha_de = |texto: &str| {
            linhas
                .iter()
                .find(|l| l.contains(texto))
                .unwrap_or_else(|| panic!("\"{texto}\" sumiu da tela:\n{}", linhas.join("\n")))
        };
        let l1 = linha_de("Primeira etapa");
        let l2 = linha_de("Segunda etapa");
        // Coluna em CARACTERES, não bytes: `find`/`rfind` de `str`
        // devolvem posição de byte, e `▐`/`▌` ocupam 3 — junto dos
        // outros caracteres largos da tela (bordas dos painéis), o
        // índice de byte não bate com a coluna que se vê.
        let coluna = |l: &str, c: char| -> usize {
            l.chars().collect::<Vec<_>>().iter().position(|x| *x == c).expect("caractere sumiu")
        };
        let col1 = coluna(l1, '▐');
        let col2 = coluna(l2, '▐');
        assert!(
            col2 > col1,
            "a segunda barra devia começar mais à direita — col1={col1} col2={col2}\n{l1}\n{l2}"
        );
        // Cada uma é metade da janela: nenhuma toma o painel inteiro,
        // que é o que "retângulo proporcional" promete sobre "cartão".
        // O painel de conteúdo aqui tem uns 55 colunas úteis; metade
        // fica bem antes da borda direita da tela (80).
        let fim1 = coluna(l1, '▌');
        assert!(
            fim1 < 60,
            "a primeira barra ocupou o painel inteiro, não a metade:\n{l1}"
        );
    }

    #[test]
    fn o_cronograma_tem_eixo_de_datas_alinhado_com_as_barras() {
        // Janela de 1 a 20 de agosto: marcas semanais em 01, 08 e 15.
        let mut e = Estado::novo(paginas(), com_cronograma());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 90, 20);
        let tudo = linhas.join("\n");
        let rotulos = linhas.iter().find(|l| l.contains("01 ago")).unwrap_or_else(|| panic!("sem eixo:\n{tudo}"));
        assert!(rotulos.contains("08 ago") && rotulos.contains("15 ago"), "{rotulos}");
        let regua = linhas.iter().find(|l| l.contains('┬')).unwrap_or_else(|| panic!("sem régua:\n{tudo}"));
        assert_eq!(colunas_de(regua, '┬').len(), 3, "{regua}");
        // A primeira marca (01 ago) cai na coluna onde a primeira barra
        // começa: as duas usam a mesma conta.
        let barra = linhas.iter().find(|l| l.contains("Primeira etapa")).unwrap();
        let topo_da_barra = &linhas[linhas.iter().position(|l| l == barra).unwrap() - 1];
        assert_eq!(colunas_de(regua, '┬')[0], colunas_de(topo_da_barra, '▗')[0], "\n{regua}\n{topo_da_barra}");
    }

    #[test]
    fn h_e_l_andam_pelas_barras_na_ordem_do_tempo() {
        // No arquivo: C (15), A (1), B (8). No tempo: A, B, C.
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"timeline\" }}\nitems:\n\
                 - title: C\n  start: '2026-08-15'\n  end: '2026-08-20'\n\
                 - title: A\n  start: '2026-08-01'\n  end: '2026-08-05'\n\
                 - title: B\n  start: '2026-08-08'\n  end: '2026-08-10'\n\
                 {{ /timeline }}\n",
            ),
        );
        e.foco = Foco::Conteudo;
        let titulo = |e: &Estado| e.arvore.em(&e.cursor).unwrap().texto.clone();
        e.cursor = vec![0];
        tecla(&mut e, "Enter");
        // Enter passa por cima do eixo e pousa na primeira barra do ARQUIVO.
        assert_eq!(titulo(&e), "C");
        tecla(&mut e, "h");
        assert_eq!(titulo(&e), "B");
        tecla(&mut e, "h");
        assert_eq!(titulo(&e), "A");
        tecla(&mut e, "h");
        assert_eq!(titulo(&e), "A", "antes da primeira não há nada");
        tecla(&mut e, "2");
        tecla(&mut e, "l");
        assert_eq!(titulo(&e), "C");
        // `j`/`k` continuam na ordem do arquivo, e `k` não pousa no eixo.
        tecla(&mut e, "j");
        assert_eq!(titulo(&e), "A");
        e.cursor = vec![0, 1];
        tecla(&mut e, "k");
        assert_eq!(e.cursor, vec![0, 1]);
    }

    #[test]
    fn a_barra_selecionada_mostra_o_detalhe_no_pe_da_caixa() {
        let mut e = Estado::novo(paginas(), com_cronograma());
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 2];
        let tudo = desenho(&mut e, 100, 20).join("\n");
        assert!(
            tudo.contains("Segunda etapa · 11/08/2026 → 20/08/2026 · 10 dias"),
            "sem detalhe:\n{tudo}"
        );
        // Fora da barra, nada de detalhe.
        e.cursor = vec![0];
        let tudo = desenho(&mut e, 100, 20).join("\n");
        assert!(!tudo.contains("10 dias"), "{tudo}");
    }

    #[test]
    fn entrar_numa_barra_nao_desce_pra_geometria() {
        // Início/duração são filhas de verdade na árvore (é como
        // `linha_de_caixa` chega nelas) — sem a guarda, Enter numa
        // barra desceria pra "0", que não tem linha na tela: pareceria
        // que a tecla não fez nada, e só o Backspace devolveria.
        let arvore = com_cronograma();
        let caminho: Caminho = vec![0, 1]; // embed → eixo, primeira barra
        assert!(matches!(
            &arvore.em(&caminho).unwrap().tipo,
            Tipo::Parte { nome, .. } if nome == "barra"
        ));
        let depois = tela::andar(&arvore, &caminho, Passo::Entrar);
        assert_eq!(depois, caminho, "Entrar desceu pra dentro da geometria da barra");
    }

    #[test]
    fn a_tabela_vira_grade_alinhada_em_coluna() {
        // Célula não é botão: até este ciclo cada célula saía na
        // PRÓPRIA linha, uma embaixo da outra — uma tabela de duas
        // colunas e duas linhas virava quatro linhas de texto solto.
        // Agora cabeçalho e cada linha ocupam uma linha só, alinhados
        // (ciclo 305).
        let mut e = Estado::novo(paginas(), com_tabela());
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 80, 16);
        let linha_de = |texto: &str| {
            tudo.iter()
                .find(|l| l.contains(texto))
                .unwrap_or_else(|| panic!("\"{texto}\" sumiu da tela:\n{}", tudo.join("\n")))
        };
        let cabecalho = linha_de("Tarefa");
        assert!(cabecalho.contains("Status"), "{cabecalho}");
        let l_api = linha_de("API");
        assert!(l_api.contains("done"), "{l_api}");
        // As duas colunas têm a MESMA largura em toda linha — é o que
        // "alinhado" quer dizer. A segunda linha tem o nome mais
        // longo, então é ELA quem define a largura da primeira coluna;
        // a coluna de "API" cresce até lá.
        let l_longa = linha_de("Um nome de tarefa bem mais longo");
        let col_status_curta = l_api.find("done").unwrap();
        let col_status_longa = l_longa.find("doing").unwrap();
        assert_eq!(
            col_status_curta, col_status_longa,
            "a coluna Status não ficou alinhada entre as linhas:\n{l_api}\n{l_longa}"
        );
    }

    #[test]
    fn a_tabela_e_uma_grade_fechada() {
        // Até aqui a tabela era só células separadas por `│`, sem borda
        // nem régua entre as linhas. A janela desenha a borda de baixo
        // de cada `<tr>`; a grade fecha dos quatro lados (ciclo 309).
        let mut e = Estado::novo(paginas(), com_tabela());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 100, 20);
        let tudo = linhas.join("\n");
        let achar = |c: char| {
            linhas
                .iter()
                .find(|l| l.contains(c))
                .unwrap_or_else(|| panic!("sem `{c}`:\n{tudo}"))
                .clone()
        };
        let topo = achar('┬');
        let cabecalho = achar('╪');
        let meio = achar('┼');
        let fim = achar('┴');
        // Duas colunas: uma divisão interna, na MESMA coluna em toda régua.
        assert_eq!(colunas_de(&topo, '┬').len(), 1, "{topo}");
        assert_eq!(colunas_de(&topo, '┬'), colunas_de(&cabecalho, '╪'));
        assert_eq!(colunas_de(&topo, '┬'), colunas_de(&meio, '┼'));
        assert_eq!(colunas_de(&topo, '┬'), colunas_de(&fim, '┴'));
        // Duas linhas de dados: uma régua simples entre elas, e só uma.
        assert_eq!(linhas.iter().filter(|l| l.contains('┼')).count(), 1, "{tudo}");
        // A linha de "API" tem a borda interna na coluna do `┬`.
        let api = linhas.iter().find(|l| l.contains("API")).unwrap();
        let esquerda = colunas_de(&topo, '┌')[0];
        let direita = colunas_de(&topo, '┐')[0];
        let bordas: Vec<usize> = colunas_de(api, '│').into_iter().filter(|c| (esquerda..=direita).contains(c)).collect();
        assert_eq!(bordas, vec![esquerda, colunas_de(&topo, '┬')[0], direita], "\n{topo}\n{api}");
    }

    #[test]
    fn a_tabela_larga_cabe_no_painel_cortando_o_texto() {
        // A borda direita não pode sumir num painel estreito: a coluna
        // mais larga cede, e o texto sai cortado com `…`.
        let mut e = Estado::novo(paginas(), com_tabela());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 50, 20);
        let tudo = linhas.join("\n");
        let topo = linhas.iter().find(|l| l.contains('┬')).unwrap_or_else(|| panic!("{tudo}"));
        assert!(topo.contains('┐'), "a borda direita sumiu:\n{tudo}");
        assert!(tudo.contains('…'), "o nome longo devia ter sido cortado:\n{tudo}");
    }

    #[test]
    fn multiselect_tem_um_badge_por_tag() {
        // Antes as duas tags saíam como um texto só, "urgente, bug", na
        // cor da primeira. Na janela cada tag é um badge (ciclo 308).
        let mut e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"table\" }}\ncolumns:\n\
                 - name: Tarefa\n\
                 - name: Tags\n  type: multiselect\n  options: [urgente, bug, infra]\n\
                 ---\n| Tarefa | Tags |\n| --- | --- |\n\
                 | UI | urgente, bug |\n\
                 | API | infra |\n\
                 {{ /table }}\n",
            ),
        );
        e.foco = Foco::Paginas;
        let info = e.tema.pilula(Realce::BadgeInfo);
        let sucesso = e.tema.pilula(Realce::BadgeSucesso);
        let buf = quadro(&mut e, 80, 14);
        let linha = (0..buf.area.height)
            .find(|y| {
                (0..buf.area.width)
                    .map(|x| buf[(x, *y)].symbol().to_string())
                    .collect::<String>()
                    .contains("urgente")
            })
            .expect("a linha das tags sumiu");
        let texto: String = (0..buf.area.width).map(|x| buf[(x, linha)].symbol().to_string()).collect();
        assert!(texto.contains(" urgente   bug "), "as tags não viraram pílulas separadas:\n{texto}");
        assert!(!texto.contains("urgente,"), "{texto}");
        // A vírgula sumiu, e cada pílula tem o fundo da SUA cor.
        let coluna = |palavra: &str| texto.find(palavra).map(|b| texto[..b].chars().count() as u16).unwrap();
        let (u, b) = (coluna("urgente"), coluna("bug"));
        assert_eq!(buf[(u, linha)].style().bg, info.bg);
        assert_eq!(buf[(b, linha)].style().bg, sucesso.bg);
        // O espaço ENTRE as pílulas não é pílula.
        assert_ne!(buf[(b - 2, linha)].style().bg, info.bg);
        assert_ne!(buf[(b - 2, linha)].style().bg, sucesso.bg);
    }

    #[test]
    fn a_celula_de_select_ganha_a_cor_do_badge_na_tela() {
        // "done" é a terceira opção de [todo, doing, done] — mesma
        // conta do núcleo (`badge_class`): índice 2, `BadgeAtencao`
        // (o token `warning`). Uma célula comum ("Tarefa"/"API") não
        // pode sair dessa cor — só ela distingue "isto é um badge" de
        // "isto é texto".
        let mut e = Estado::novo(paginas(), com_tabela());
        e.foco = Foco::Paginas;
        let cor_badge = e.tema.estilo(Realce::BadgeAtencao).fg.unwrap();
        let mut term = Terminal::new(TestBackend::new(80, 16)).unwrap();
        term.draw(|f| desenhar(f, &mut e)).unwrap();
        let buf = term.backend().buffer().clone();
        let aparece = (0..buf.area.height)
            .any(|y| (0..buf.area.width).any(|x| buf[(x, y)].style().fg == Some(cor_badge)));
        assert!(aparece, "a cor do badge não apareceu na tela");
    }

    #[test]
    fn o_botao_primario_das_acoes_tem_cor_propria() {
        // `variant: primary` chegava até a árvore e morria aí — todo
        // botão saía na mesma cor, igual ao fantasma da janela quando o
        // destaque falha em aparecer (ciclo 304).
        let e = Estado::novo(
            paginas(),
            analisar(
                "{{ type: \"actions\" }}\nbuttons:\n- label: Cancelar\n  action: open-page\n  path: pages/a.md\n- label: Confirmar\n  action: open-page\n  path: pages/b.md\n  variant: primary\n{{ /actions }}\n",
            ),
        );
        assert_ne!(
            e.tema.contorno_do_botao(Realce::Parte).fg,
            e.tema.contorno_do_botao(Realce::BotaoPrimario).fg,
            "o botão primário saiu da mesma cor que o comum"
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
