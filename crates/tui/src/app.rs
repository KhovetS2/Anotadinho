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
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

pub mod conversa;
pub mod especiais;
pub mod edicao;
pub mod markdown;
mod wikilink;
mod formatar;
pub mod teclas;
pub mod modais;
pub use edicao::{AcaoDaPergunta, Pergunta, Registro};
pub use modais::{Modal, Pedido, Preferencias};
pub use markdown::{EdicaoDeBloco, Hospedeiro};
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
    /// As pastas do disco, inclusive vazias (`pages/…`), pro `main` dizer
    /// (ciclo 345).
    pub pastas_do_vault: Vec<String>,
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
    /// O índice do vault, pras consultas (ciclo 331). Quem varre é o
    /// `main`; vazio é "ninguém varreu".
    pub indice_do_vault: Vec<anotadinho_core::index::PageIndexEntry>,
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
    /// A tela de conversa, quando a página aberta é `type: conversa`
    /// (ciclo 340).
    pub conversa: Option<conversa::TelaDeConversa>,
    /// A tela de tags, assets ou propostas (ciclo 346).
    pub especial: Option<especiais::TelaEspecial>,
    /// O bloco novo à espera do que o menu `/` vai pôr nele (ciclo 347).
    pub bloco_a_inserir: Option<markdown::EdicaoDeBloco>,
    /// A sugestão escolhida no autocompletar de wikilink (ciclo 352).
    pub wikilink_sel: usize,
    /// O `[[` cuja lista a pessoa fechou com `Esc`.
    pub wikilink_dispensado: Option<usize>,
    /// As abas abertas, pelo caminho (ciclo 354), na ordem em que abriram.
    pub abas: Vec<String>,
    /// O que a busca da sidebar achou no CONTEÚDO (ciclo 370): o termo e
    /// os trechos, como a seção "Resultados" da janela.
    pub resultados_da_busca: Option<(String, Vec<anotadinho_core::embed::SearchHit>)>,
    /// O termo buscado no conteúdo: a página aberta a seguir leva o
    /// cursor até ele (ciclo 371), como a janela revela o trecho.
    pub alvo_de_busca: Option<String>,
    /// A página de início deste vault (ciclo 362): abre primeiro, e a aba
    /// dela fica fixa na frente.
    pub inicio: Option<String>,
    /// A imagem nova da galeria esperando o arquivo escolhido em
    /// `assets/` (ciclo 356).
    pub imagem_pendente: Option<edicao::AcaoDaPergunta>,
    /// A célula de coluna "página" esperando a escolha (ciclo 367):
    /// tabela, linha e coluna.
    pub celula_pendente: Option<(Caminho, usize, usize)>,
    /// A inserção guardada enquanto o menu Formatar está aberto (ciclo 357).
    pub pergunta_suspensa: Option<edicao::Pergunta>,
    /// Propostas do agente esperando revisão (ciclo 355), como o botão do
    /// cabeçalho da janela.
    pub propostas_pendentes: usize,
    /// Arquivos mudados no git; `None` fora de repositório.
    pub mudancas_no_git: Option<usize>,
    /// O modal aberto — barra de comandos, escolha, confirmação (ciclo 339).
    pub modal: Option<Modal>,
    /// O que só o `main` pode fazer (abrir, criar, apagar, gravar
    /// preferências), na ordem em que foi pedido.
    pub pedidos: Vec<Pedido>,
    /// As preferências da TUI (tema, sidebar, agente).
    pub preferencias: Preferencias,
    /// Agora, `AAAA-MM-DD HH:MM`, quando o `main` diz.
    pub agora: Option<String>,
    /// O que `yy`/`dd` guardaram pra `p` colar (ciclo 322).
    pub registro: Option<Registro>,
    /// Texto e cursor de antes de cada edição, pro `u` (ciclo 322).
    pub desfazer: Vec<(String, Caminho)>,
    /// O que o `u` desfez, pro `Ctrl+R`.
    pub refazer: Vec<(String, Caminho)>,
    /// A visão de cada calendário (ciclo 316). Ausente é Mês.
    pub visoes: std::collections::HashMap<Caminho, anotadinho_core::analise::Visao>,
    /// O primeiro `d` do `dd` na sidebar.
    pub d_na_sidebar: bool,
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
            pastas_do_vault: Vec::new(),
            pastas_fechadas,
            linha_sidebar: 0,
            vim: vim::Pendente::default(),
            busca: String::new(),
            busca_em: Foco::Paginas,
            barra_aberta: false,
            d_na_sidebar: false,
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
            indice_do_vault: Vec::new(),
            texto_da_pagina: None,
            versao: None,
            gravacao: None,
            aviso: None,
            pergunta: None,
            registro: None,
            modal: None,
            conversa: None,
            especial: None,
            bloco_a_inserir: None,
            wikilink_sel: 0,
            wikilink_dispensado: None,
            abas: Vec::new(),
            resultados_da_busca: None,
            alvo_de_busca: None,
            inicio: None,
            imagem_pendente: None,
            celula_pendente: None,
            pergunta_suspensa: None,
            propostas_pendentes: 0,
            mudancas_no_git: None,
            pedidos: Vec::new(),
            preferencias: Preferencias::default(),
            agora: None,
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
        if let Some(p) = self.paginas.get(self.pagina).map(|p| p.path.clone()) {
            if !self.abas.contains(&p) {
                self.abas.push(p);
            }
        }
        self
    }

    /// Troca a página aberta pelo arquivo `texto` (ciclo 318) — o caminho
    /// que o `main` usa, pra a edição saber o que gravar.
    pub fn abrir_texto(&mut self, texto: &str, versao: Option<String>) {
        // Conversa abre como conversa (ciclo 340); o que é da tela (o
        // rascunho, o agente rodando) atravessa a releitura.
        let (path, titulo) = self.paginas.get(self.pagina).map(|p| (p.path.clone(), p.title.clone())).unwrap_or_default();
        // Toda página aberta ganha aba (ciclo 354), como na janela.
        if !path.is_empty() && !self.abas.contains(&path) {
            self.abas.push(path.clone());
        }
        self.organizar_abas();
        let velha = self.conversa.take();
        self.conversa = conversa::TelaDeConversa::do_arquivo(&path, &titulo, texto).map(|mut nova| {
            if let Some(v) = &velha {
                nova.herdar(v);
            }
            nova
        });
        let (frontmatter, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(texto);
        // Tags, assets e propostas mostram o vault (ciclo 346): a tela
        // abre carregando e pede os dados. Reabrir a mesma mantém o lugar.
        let velha = self.especial.take();
        self.especial = especiais::TipoEspecial::do_frontmatter(frontmatter).map(|tipo| {
            let mut tela = match velha {
                Some(v) if v.tipo == tipo => v,
                _ => especiais::TelaEspecial::nova(tipo),
            };
            // O kanban de página lê a própria página: não precisa do `main`.
            if tipo == especiais::TipoEspecial::Kanban {
                tela.dados = Some(especiais::kanban_da_pagina(texto));
            } else {
                self.pedidos.push(Pedido::CarregarEspecial(tipo));
            }
            tela
        });
        // `type: calendar` de página inteira é o calendário do vault
        // (ciclo 353), como o `Calendar` da janela: somente leitura.
        let calendario = pagina_de_calendario(frontmatter);
        let corpo = if calendario { CALENDARIO_DA_PAGINA } else { corpo };
        self.abrir(anotadinho_core::analise::analisar(corpo));
        self.desfazer.clear();
        self.refazer.clear();
        self.texto_da_pagina = (!calendario).then(|| texto.to_string());
        self.versao = versao;
        if let Some(termo) = self.alvo_de_busca.take() {
            self.revelar(&termo);
        }
    }

    /// Leva o cursor ao primeiro bloco que contém `termo`, abrindo as
    /// dobras no caminho (ciclo 371).
    pub fn revelar(&mut self, termo: &str) {
        let alvo = termo.to_lowercase();
        let achado = self
            .arvore
            .percorrer()
            .into_iter()
            .find(|(_, u)| !u.texto.is_empty() && u.texto.to_lowercase().contains(&alvo))
            .map(|(c, _)| c);
        if let Some(c) = achado {
            for n in 1..c.len() {
                self.dobrados.remove(&c[..n].to_vec());
            }
            self.cursor = c;
            self.foco = Foco::Conteudo;
            self.seguir_cursor();
        }
    }

    /// A aba do início vai pra frente, sem mexer na ordem das outras —
    /// o `organize_tabs` da janela.
    pub fn organizar_abas(&mut self) {
        if let Some(i) = self.inicio.as_ref().and_then(|c| self.abas.iter().position(|a| a == c)) {
            let a = self.abas.remove(i);
            self.abas.insert(0, a);
        }
    }

    /// Seleciona a página pelo caminho (ciclo 372).
    pub fn com_pagina(mut self, path: &str) -> Self {
        if let Some(i) = self.paginas.iter().position(|p| p.path == path) {
            self.pagina = i;
        }
        self
    }

    /// Diz qual é a página de início e a seleciona (ciclo 362).
    pub fn com_inicio(mut self, inicio: Option<String>) -> Self {
        if let Some(i) = inicio.as_ref().and_then(|c| self.paginas.iter().position(|p| p.path == *c)) {
            self.pagina = i;
        }
        self.inicio = inicio;
        self
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

    /// Entrega o índice do vault e roda as consultas da página.
    pub fn com_indice_do_vault(mut self, indice: Vec<anotadinho_core::index::PageIndexEntry>) -> Self {
        self.indice_do_vault = indice;
        self.ancorar_calendarios();
        self
    }

    /// Troca os filhos de cada consulta da página pelo resultado dela
    /// sobre o índice (ciclo 331). Sem índice, fica o que a análise pôs.
    fn montar_consultas(&mut self) {
        if self.indice_do_vault.is_empty() {
            return;
        }
        for i in 0..self.arvore.filhos.len() {
            let u = &self.arvore.filhos[i];
            if !matches!(&u.tipo, Tipo::Embed(n) if n == "query") {
                continue;
            }
            let Some(anotadinho_core::embed::EmbedData::Query(q)) = u.fonte.as_deref().and_then(|f| {
                anotadinho_core::embed::segment(f).into_iter().find_map(|s| match s {
                    anotadinho_core::embed::DocSegment::Embed(d) => Some(d),
                    _ => None,
                })
            }) else {
                continue;
            };
            self.arvore.filhos[i].filhos = anotadinho_core::analise::partes_da_consulta(&q, &self.indice_do_vault);
        }
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
        self.montar_consultas();
        let Some(hoje) = self.hoje.clone() else {
            self.linhas = tela::linhas(&self.arvore);
            return;
        };
        self.ancorar_cronogramas(&hoje);
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

    /// Os cronogramas da página na janela de tempo da tela (ciclo 332):
    /// `escala` dias a partir da âncora — ou, sem âncora, começando um
    /// quarto da janela antes de hoje, como a janela gráfica. No modo
    /// vault as barras são as páginas com data.
    fn ancorar_cronogramas(&mut self, hoje: &str) {
        for i in 0..self.arvore.filhos.len() {
            let u = &self.arvore.filhos[i];
            if !matches!(&u.tipo, Tipo::Embed(n) if n == "timeline") {
                continue;
            }
            let Some(anotadinho_core::embed::EmbedData::Timeline(mut d)) = u.fonte.as_deref().and_then(|f| {
                anotadinho_core::embed::segment(f).into_iter().find_map(|s| match s {
                    anotadinho_core::embed::DocSegment::Embed(d) => Some(d),
                    _ => None,
                })
            }) else {
                continue;
            };
            if d.source == anotadinho_core::embed::TimelineSource::Vault {
                d.items = anotadinho_core::calendario::itens_do_vault(&self.indice_do_vault);
            }
            let dias = d.scale.days();
            let inicio = self
                .ancoras
                .get(&vec![i])
                .cloned()
                .or_else(|| anotadinho_core::date_util::add_days(hoje, -(dias / 4)))
                .unwrap_or_else(|| hoje.to_string());
            self.arvore.filhos[i].filhos =
                anotadinho_core::analise::partes_do_cronograma(&d, Some((&inicio, dias)), Some(hoje));
        }
    }

    /// Muda a âncora de um calendário e refaz a árvore dele.
    fn ancorar(&mut self, embed: &[usize], data: String) {
        self.ancoras.insert(embed.to_vec(), data);
        self.ancorar_calendarios();
    }

    /// Troca a paleta.
    pub fn com_tema(mut self, nome: &str) -> Self {
        self.tema = Tema::novo(nome).com_aparencia(&self.preferencias.destaque, &self.preferencias.botoes);
        self.preferencias.tema = nome.to_string();
        self
    }

    /// As pastas do disco, pra a árvore mostrar as vazias também.
    pub fn com_pastas(mut self, pastas: Vec<String>) -> Self {
        self.pastas_do_vault = pastas;
        let paginas = self.paginas.clone();
        self.atualizar_paginas(paginas);
        self
    }

    /// A pasta sob o cursor da sidebar (`pages/…`): a própria pasta, ou a
    /// da página.
    pub fn pasta_da_sidebar(&self) -> String {
        match self.sidebar_visivel().get(self.linha_sidebar).map(|l| l.item.clone()) {
            Some(Item::Pasta { caminho, .. }) => {
                if caminho.starts_with("journals") { caminho } else { format!("pages/{caminho}") }
            }
            _ => self
                .paginas
                .get(self.pagina)
                .and_then(|p| std::path::Path::new(&p.path).parent().map(|x| x.to_string_lossy().to_string()))
                .unwrap_or_else(|| "pages".into()),
        }
    }

    /// Aplica as preferências gravadas (ciclo 339).
    pub fn com_preferencias(mut self, p: Preferencias) -> Self {
        self.tema = Tema::novo(&p.tema).com_aparencia(&p.destaque, &p.botoes);
        if !p.sidebar {
            self.foco = Foco::Conteudo;
        }
        self.preferencias = p;
        self
    }

    /// Troca a lista de páginas (depois de criar ou apagar uma).
    pub fn atualizar_paginas(&mut self, paginas: Vec<PageMeta>) {
        // A selecionada segue pelo CAMINHO: uma página nova antes dela na
        // lista não pode trocar a aberta de lugar (ciclo 351).
        let atual = self.paginas.get(self.pagina).map(|p| p.path.clone());
        self.arvore_sidebar = sidebar::com_pastas(sidebar::arvore(&paginas), &self.pastas_do_vault);
        // Página que sumiu (apagada, movida) perde a aba.
        self.abas.retain(|a| paginas.iter().any(|p| p.path == *a));
        self.paginas = paginas;
        self.pagina = atual
            .and_then(|a| self.paginas.iter().position(|p| p.path == a))
            .unwrap_or(self.pagina)
            .min(self.paginas.len().saturating_sub(1));
    }

    /// O arquivo aberto mudou no disco por outro programa (ciclo 351),
    /// como o watcher da janela: relê sem perder o lugar — cursor, dobras,
    /// rolagem — e zera o desfazer, que regravaria o conteúdo velho.
    ///
    /// Com uma edição aberta (inserção, formulário) não mexe: devolve
    /// `false` e quem vigia tenta de novo depois.
    pub fn recarregar_do_disco(&mut self, texto: &str, versao: Option<String>) -> bool {
        if self.pergunta.is_some() || matches!(self.modal, Some(Modal::Detalhe { .. } | Modal::Opcoes(_) | Modal::Conflito(_))) {
            return false;
        }
        let (frontmatter, _) = anotadinho_core::MarkdownCodec::split_frontmatter_text(texto);
        if self.conversa.is_some() || self.especial.is_some() || pagina_de_calendario(frontmatter) {
            self.abrir_texto(texto, versao);
            return true;
        }
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(texto);
        self.arvore = anotadinho_core::analise::analisar(corpo);
        self.ancorar_calendarios();
        self.linhas = tela::linhas(&self.arvore);
        while !self.cursor.is_empty() && self.arvore.em(&self.cursor).is_none() {
            self.cursor.pop();
        }
        if self.cursor.is_empty() {
            self.cursor = tela::primeiro(&self.arvore).unwrap_or_default();
        }
        self.texto_da_pagina = Some(texto.to_string());
        self.versao = versao;
        self.desfazer.clear();
        self.refazer.clear();
        self.aviso = Some("a página mudou por fora e foi relida".into());
        true
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
                        Item::Pasta { .. } | Item::Resultado { .. } => false,
                    })
                    .map(|mut l| {
                        l.nivel = 0;
                        l
                    })
                    .chain(
                        self.resultados_da_busca
                            .iter()
                            .filter(|(t, _)| *t == termo)
                            .flat_map(|(_, hits)| hits.iter())
                            .filter_map(|h| {
                                let indice = self.paginas.iter().position(|p| p.path == h.path)?;
                                let trecho = h.snippet.replace("**", "").split_whitespace().collect::<Vec<_>>().join(" ");
                                Some(sidebar::Linha {
                                    nivel: 0,
                                    item: Item::Resultado { indice, titulo: self.paginas[indice].title.clone(), trecho },
                                })
                            }),
                    )
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
        if let Some(Item::Pagina { indice, .. } | Item::Resultado { indice, .. }) = visiveis.get(nova).map(|l| &l.item) {
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

/// O corpo que a página `type: calendar` mostra: o calendário do vault.
const CALENDARIO_DA_PAGINA: &str = "{{ type: \"calendar\" }}\nmode: vault\n{{ /calendar }}\n";

fn pagina_de_calendario(frontmatter: &str) -> bool {
    frontmatter
        .lines()
        .any(|l| l.strip_prefix("type:").map(|v| v.trim().trim_matches('"').trim_matches('\'')) == Some("calendar"))
}

/// O tempo passou sem tecla: quem acompanha coisa que roda por fora
/// (o agente de uma conversa) atualiza aqui.
pub fn tique(_e: &mut Estado) {}

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
    // Um modal recebe tudo (ciclo 339).
    if e.modal.is_some() {
        modais::tecla(e, tecla);
        return None;
    }
    // Escrevendo numa conversa, toda tecla é texto (ciclo 340).
    if e.foco == Foco::Conteudo && e.conversa.as_ref().is_some_and(|c| c.escrevendo) {
        conversa::tecla(e, tecla);
        return None;
    }
    // As teclas remapeadas (ciclo 359) valem daqui pra baixo: o que vem
    // antes é onde se digita texto.
    let traduzida;
    let tecla = match teclas::traduzir(e, tecla) {
        teclas::Remapeada::Traduzida(t) => {
            traduzida = t;
            traduzida.as_str()
        }
        teclas::Remapeada::Comando("paleta") => {
            modais::abrir_paleta(e);
            return None;
        }
        teclas::Remapeada::Comando(c) => {
            modais::executar(e, c);
            return None;
        }
        teclas::Remapeada::Nada => return None,
    };
    // As abas (ciclo 354): `Alt+1`…`Alt+9` vão direto, `Ctrl+W` (ou
    // `Alt+L`) passa pra próxima, `Alt+H` volta, `Alt+Q` fecha — as teclas
    // da janela, com `Alt` porque o terminal come `Ctrl+número`.
    if let Some(destino) = tecla_das_abas(e, tecla) {
        return destino;
    }
    // A barra de comandos: `:` (o modo de comando do vim) ou `Ctrl+K` (o
    // atalho da janela); `?` mostra os atalhos.
    if tecla == "Ctrl+k" || (matches!(tecla, ":" | "?") && !e.vim.em_curso()) {
        if tecla == "?" {
            e.modal = Some(Modal::Atalhos(0));
        } else {
            modais::abrir_paleta(e);
        }
        return None;
    }
    match tecla {
        "q" => {
            e.sair = true;
            None
        }
        "Tab" if !e.preferencias.sidebar => {
            e.foco = Foco::Conteudo;
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
        _ if e.conversa.is_some() => {
            conversa::tecla(e, tecla);
            None
        }
        _ if e.especial.is_some() => {
            especiais::tecla(e, tecla);
            None
        }
        _ => {
            // Enter num evento do vault abre a página dele (ciclo 317), o
            // que o clique faz na janela.
            // Enter numa transição do fluxo move a etapa (ciclo 335).
            // `=` configura o item (ciclo 344).
            if tecla == "=" && !e.vim.em_curso() && edicao::configurar(e) {
                return None;
            }
            if tecla == "Enter"
                && (edicao::transicao_do_cursor(e)
                    || edicao::acao_do_fluxo(e)
                    || edicao::agendar_sem_data(e)
                    || edicao::abrir_pagina_da_celula(e)
                    || edicao::cartao_na_coluna_vazia(e)
                    || edicao::busca_da_consulta(e)
                    || edicao::abrir_opcoes(e)
                    || edicao::acionar_botao(e)
                    || edicao::abrir_detalhe_do_cartao(e)
                    || edicao::abrir_detalhe_do_evento(e)
                    || edicao::seguir_wikilink(e))
            {
                return None;
            }
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

/// Uma tecla de aba. `None` é "não é de aba"; `Some(destino)` é o que
/// `tecla` devolve (a página a carregar, se mudou).
fn tecla_das_abas(e: &mut Estado, tecla: &str) -> Option<Option<String>> {
    if e.vim.em_curso() {
        return None;
    }
    let atual = e.paginas.get(e.pagina).map(|p| p.path.clone()).unwrap_or_default();
    let pos = e.abas.iter().position(|a| *a == atual);
    let n = e.abas.len();
    let alvo = match tecla {
        "Ctrl+w" | "Alt+l" if n > 0 => pos.map_or(0, |p| (p + 1) % n),
        "Alt+h" if n > 0 => pos.map_or(0, |p| (p + n - 1) % n),
        "Alt+q" => return Some(fechar_aba(e)),
        _ => match tecla.strip_prefix("Alt+").and_then(|d| d.parse::<usize>().ok()) {
            Some(d) if (1..=9).contains(&d) => {
                if d > n {
                    e.aviso = Some(format!("não há aba {d}"));
                    return Some(None);
                }
                d - 1
            }
            _ => return None,
        },
    };
    Some(ir_pra_aba(e, alvo))
}

/// Os comandos de aba da barra (ciclo 354): pedem pra abrir a página.
pub(super) fn comando_de_aba(e: &mut Estado, tecla: &str) {
    if let Some(Some(path)) = tecla_das_abas(e, tecla) {
        e.pedidos.push(Pedido::AbrirPagina(path));
    }
}

fn ir_pra_aba(e: &mut Estado, i: usize) -> Option<String> {
    let path = e.abas.get(i)?.clone();
    let atual = e.paginas.get(e.pagina).map(|p| p.path.clone());
    if atual.as_deref() == Some(path.as_str()) {
        return None;
    }
    e.pagina = e.paginas.iter().position(|p| p.path == path)?;
    Some(path)
}

/// Fecha a aba da página aberta e abre a vizinha (a da direita, ou a da
/// esquerda se era a última). A última aba não fecha: sempre há uma
/// página na tela.
fn fechar_aba(e: &mut Estado) -> Option<String> {
    let atual = e.paginas.get(e.pagina).map(|p| p.path.clone())?;
    let pos = e.abas.iter().position(|a| *a == atual)?;
    if e.abas.len() <= 1 {
        e.aviso = Some("é a única aba".into());
        return None;
    }
    if e.inicio.as_deref() == Some(atual.as_str()) {
        e.aviso = Some("a aba de início fica fixa".into());
        return None;
    }
    e.abas.remove(pos);
    ir_pra_aba(e, pos.min(e.abas.len() - 1))
}

/// A barra de abas, na borda de cima do conteúdo (ciclo 354): o número,
/// o título e a aberta acesa, como a `TabBar` da janela. Com uma aba só,
/// a borda mostra o título como sempre.
fn desenhar_abas(f: &mut Frame, e: &Estado, area: ratatui::layout::Rect) {
    if e.abas.len() < 2 || area.width < 20 {
        return;
    }
    let atual = e.paginas.get(e.pagina).map(|p| p.path.as_str()).unwrap_or("");
    let mut spans = Vec::new();
    for (i, path) in e.abas.iter().enumerate() {
        let titulo = e.paginas.iter().find(|p| p.path == *path).map(|p| p.title.clone()).unwrap_or_else(|| path.clone());
        let curto: String = if titulo.chars().count() > 22 { format!("{}…", titulo.chars().take(21).collect::<String>()) } else { titulo };
        let (num, nome) = if path == atual {
            (e.tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD), e.tema.estilo(Realce::Cursor))
        } else {
            (Style::default().fg(e.tema.var("text-muted")).bg(e.tema.var("bg-surface")), Style::default().fg(e.tema.var("text-primary")).bg(e.tema.var("bg-surface")))
        };
        if e.inicio.as_deref() == Some(path.as_str()) {
            spans.push(Span::styled(" ⌂", num));
        } else if i < 9 {
            spans.push(Span::styled(format!(" {}", i + 1), num));
        }
        spans.push(Span::styled(format!(" {curto} "), nome));
        spans.push(Span::raw(" "));
    }
    let barra = ratatui::layout::Rect { x: area.x + 1, y: area.y, width: area.width.saturating_sub(2), height: 1 };
    f.render_widget(ratatui::widgets::Clear, barra);
    f.render_widget(Paragraph::new(Line::from(spans)).style(e.tema.estilo(Realce::Fundo)), barra);
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
            buscar_conteudo_na_sidebar(e);
            None
        }
        // Uma tecla de um caractere é texto; o resto (setas, F1) não.
        t if t.chars().count() == 1 => {
            e.busca.push_str(t);
            e.corrigir_cursor();
            e.corrigir_pagina();
            buscar_conteudo_na_sidebar(e);
            None
        }
        _ => None,
    }
}

/// Com 3 letras ou mais na busca da sidebar, busca também no conteúdo
/// (ciclo 370), como a sidebar da janela.
fn buscar_conteudo_na_sidebar(e: &mut Estado) {
    if e.busca_em != Foco::Paginas {
        return;
    }
    if e.busca.chars().count() >= 3 {
        e.pedidos.push(Pedido::BuscarNaSidebar(e.busca.clone()));
    } else {
        e.resultados_da_busca = None;
    }
}

fn tecla_nas_paginas(e: &mut Estado, tecla: &str) -> Option<String> {
    if tecla != "d" {
        e.d_na_sidebar = false;
    }
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
        // Na sidebar, os comandos de vim são de PÁGINA e PASTA (ciclo 345):
        // `o` página nova na pasta do cursor, `O` pasta nova, `m` mover a
        // página, `dd` excluir.
        "o" => {
            modais::abrir_entrada_na_pasta(e, false);
            None
        }
        "O" => {
            modais::abrir_entrada_na_pasta(e, true);
            None
        }
        "m" => {
            modais::abrir_mover_pagina(e);
            None
        }
        "d" if e.d_na_sidebar => {
            e.d_na_sidebar = false;
            // Só página se exclui por aqui; numa pasta, nada.
            let na_pagina = matches!(e.sidebar_visivel().get(e.linha_sidebar).map(|l| &l.item), Some(Item::Pagina { .. }));
            if na_pagina {
                modais::confirmar_exclusao(e);
            } else {
                e.aviso = Some("dd exclui página; pasta ainda não".into());
            }
            None
        }
        "d" => {
            e.d_na_sidebar = true;
            None
        }
        "Enter" => {
            if matches!(e.sidebar_visivel().get(e.linha_sidebar).map(|l| &l.item), Some(Item::Resultado { .. })) {
                e.alvo_de_busca = Some(e.busca.clone());
            }
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
    // Sem sidebar (preferência, ciclo 339), o conteúdo toma a tela.
    let divisao = if e.preferencias.sidebar {
        [Constraint::Percentage(30), Constraint::Percentage(70)]
    } else {
        [Constraint::Length(0), Constraint::Percentage(100)]
    };
    let colunas = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(divisao)
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
                // Resultado no conteúdo: a página e o trecho apagado.
                Item::Resultado { titulo, trecho, .. } => Line::from(vec![
                    Span::styled("⌕ ", Style::default().fg(e.tema.var("text-muted"))),
                    Span::styled(titulo.clone(), estilo),
                    Span::styled(format!(" · {trecho}"), Style::default().fg(e.tema.var("text-muted"))),
                ]),
            }
        })
        .collect();
    let mut bloco_paginas = borda("páginas", e.foco == Foco::Paginas, &e.tema);
    if let Some(rodape) = rodape_de_busca(e, Foco::Paginas) {
        bloco_paginas = bloco_paginas.title_bottom(rodape);
    }
    // O que a janela mostra no cabeçalho (ciclo 355): propostas esperando
    // revisão e o que mudou no git — a barra de comandos abre os dois.
    let mut indicadores = Vec::new();
    if e.propostas_pendentes > 0 {
        indicadores.push(Span::styled(format!(" ✓ {} proposta(s) ", e.propostas_pendentes), e.tema.pilula(Realce::BadgeAtencao)));
    }
    if let Some(n) = e.mudancas_no_git.filter(|n| *n > 0) {
        indicadores.push(Span::raw(" "));
        indicadores.push(Span::styled(format!(" ⑂ {n} "), Style::default().fg(e.tema.var("text-muted"))));
    }
    if !indicadores.is_empty() {
        bloco_paginas = bloco_paginas.title_bottom(Line::from(indicadores).right_aligned());
    }
    f.render_widget(Paragraph::new(paginas).block(bloco_paginas), colunas[0]);

    // Uma conversa tem tela própria (ciclo 340).
    if e.conversa.is_some() {
        conversa::desenhar(f, e, colunas[1]);
        desenhar_abas(f, e, colunas[1]);
        modais::desenhar(f, e);
        return;
    }
    if e.especial.is_some() {
        especiais::desenhar(f, e, colunas[1]);
        desenhar_abas(f, e, colunas[1]);
        modais::desenhar(f, e);
        return;
    }
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
        // Os embeds de colunas já desenhados: o bloco inteiro sai na
        // primeira linha dele, lado a lado (ciclo 326).
        let mut colunas_feitas: Vec<Vec<usize>> = Vec::new();
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
            // Embeds desenhados INTEIROS, de uma vez, na primeira linha
            // deles: o que a janela põe lado a lado (colunas, 326; kanban,
            // 328) não cabe no desenho de uma linha por unidade.
            let nas_colunas = matches!(l.embed_dono.as_deref(), Some("columns" | "kanban" | "query"))
                && !matches!(l.tipo, Tipo::Embed(_));
            if nas_colunas && l.dono_embed.as_ref().is_some_and(|d| colunas_feitas.contains(d)) {
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
            let na_galeria = l.embed_dono.as_deref() == Some("gallery");
            let mut desenhadas: Vec<Vec<Line>> = if nas_colunas {
                let dono = l.dono_embed.clone().unwrap_or_default();
                if linhas_visiveis.iter().any(|f| f.dono_embed.as_ref() == Some(&dono) && f.mostra(&e.cursor)) {
                    achou = true;
                }
                let bloco = if l.embed_dono.as_deref() == Some("kanban") {
                    linhas_do_kanban(e, &dono, largura_conteudo)
                } else if l.embed_dono.as_deref() == Some("query") {
                    linhas_da_consulta(e, &dono, largura_conteudo)
                } else {
                    linhas_das_colunas(e, &dono, &linhas_visiveis, largura_conteudo)
                };
                colunas_feitas.push(dono);
                bloco.into_iter().map(|linha| vec![linha]).collect()
            } else if na_galeria && nome_da_parte == "cabecalho" {
                vec![vec![linha_do_cabecalho_da_galeria(l, &e.arvore, &e.tema, largura_conteudo)]]
            } else if na_galeria && !l.segmentos.is_empty() {
                vec![linhas_da_galeria(l, &e.arvore, &e.tema, largura_conteudo, no_foco.then_some(e.cursor.as_slice()))]
            } else if l.embed_dono.as_deref() == Some("timeline") && nome_da_parte == "eixo" {
                let hoje = desloc_de_hoje(&e.arvore, l.dono_embed.as_deref().unwrap_or(&[]));
                vec![linhas_do_eixo(l, &e.tema, largura_conteudo, hoje)]
            } else if l.embed_dono.as_deref() == Some("timeline") && nome_da_parte == "cabecalho" {
                vec![vec![linha_do_cabecalho_do_cronograma(l, &e.arvore, &e.tema, largura_conteudo)]]
            } else if l.embed_dono.as_deref() == Some("timeline") && nome_da_parte == "barra" {
                let hoje = desloc_de_hoje(&e.arvore, l.dono_embed.as_deref().unwrap_or(&[]));
                vec![vec![linha_da_barra(l, &e.arvore, &e.tema, largura_conteudo, no_foco.then_some(e.cursor.as_slice()), hoje)]]
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
            } else if l.embed_dono.as_deref() == Some("fluxo") && matches!(nome_da_parte, "titulo" | "dica" | "nota") {
                vec![vec![linha_do_fluxo(l, nome_da_parte, &e.tema)]]
            } else if no_callout && nome_da_parte == "titulo" {
                let dono = l.dono_embed.as_deref().unwrap_or(&[]);
                vec![vec![linha_do_titulo_do_callout(
                    l,
                    &variante_do_callout(&e.arvore, dono),
                    papel_do_callout(&e.arvore, dono),
                    &e.tema,
                    largura_conteudo,
                )]]
            } else if no_calendario && nome_da_parte == "cabecalho" {
                let visao = e
                    .visoes
                    .get(l.dono_embed.as_deref().unwrap_or(&[]))
                    .copied()
                    .unwrap_or_default();
                let dono = l.dono_embed.as_deref().unwrap_or(&[]);
                let periodo = l
                    .caminho
                    .split_last()
                    .and_then(|(i, pai)| e.arvore.em(pai)?.filhos.get(i + 1))
                    .filter(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "mes" || nome == "agenda"))
                    .map(|u| u.texto.clone());
                let vault = e
                    .arvore
                    .em(dono)
                    .and_then(|u| u.fonte.as_deref())
                    .and_then(dados_do_calendario)
                    .is_some_and(|d| d.mode == anotadinho_core::embed::CalendarSource::Vault);
                vec![vec![linha_do_cabecalho_do_calendario(l, visao, periodo.as_deref(), vault, &e.tema, largura_conteudo)]]
            } else if no_calendario && nome_da_parte == "agenda" && !e.dobrados.contains(&l.caminho) {
                // O dia já está no cabeçalho (ciclo 336).
                Vec::new()
            } else if no_calendario && nome_da_parte.starts_with("compromisso") {
                vec![vec![linha_do_compromisso(l, &e.arvore, &e.tema, largura_conteudo, Some(&e.cursor))]]
            } else if matches!(l.embed_dono.as_deref(), Some("calendar" | "timeline")) && nome_da_parte == "sem-data" {
                // A gaveta da janela: "▸ Sem data (N)" e o "+ evento sem
                // data" tracejado ao lado (ciclo 336).
                let aberta = !e.dobrados.contains(&l.caminho);
                let apagado = Style::default().fg(e.tema.var("text-muted"));
                let estilo = if no_foco && l.mostra(&e.cursor) { e.tema.estilo(Realce::Cursor) } else { apagado };
                let mut spans = vec![
                    Span::raw("  ".repeat(l.nivel)),
                    Span::styled(format!("{} {}", if aberta { "▾" } else { "▸" }, l.texto), estilo),
                ];
                if no_calendario {
                    spans.push(Span::styled("   ╎ + evento sem data ╎", apagado));
                }
                vec![vec![Line::from(spans)]]
            } else if l.embed_dono.is_some() && nome_da_parte == "nada" {
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
                // Com cabeçalho, o nome do mês já está nele (ciclo 336).
                let com_cabecalho = l
                    .caminho
                    .split_last()
                    .and_then(|(i, pai)| i.checked_sub(1).and_then(|k| e.arvore.em(pai)?.filhos.get(k)))
                    .is_some_and(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "cabecalho"));
                vec![linhas_do_mes(l, &e.tema, largura_conteudo, com_cabecalho)]
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
                    None => linhas_de_fileira(l, &e.arvore, &e.tema, largura_conteudo, Some(&e.cursor)),
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
                linhas_de_fileira(l, &e.arvore, &e.tema, largura_conteudo, Some(&e.cursor))
            } else if let Some(modo) = e.arvore.em(&l.caminho).and_then(caixa_avulsa) {
                vec![linha_de_caixa(l, &e.tema, largura_conteudo, Some(&e.cursor), modo)]
            } else {
                let linha = linha_estilizada(
                    // Sem fundo no texto (ciclo 295): quem diz onde se
                    // está é a moldura. Reintroduzi isto sem querer ao
                    // reescrever o laço, e o teste do 295 pegou.
                    l,
                    false,
                    e.dobrados.contains(&l.caminho),
                    &e.tema,
                    largura_conteudo,
                );
                // O texto QUEBRA na largura (ciclo 338), em vez de ser
                // cortado na borda — e as quebras do parágrafo viram
                // linhas. A continuação alinha depois da marca (`- `).
                let marca = marca_com_dobra(l, e.dobrados.contains(&l.caminho));
                let recuo = 2 * l.nivel + if marca.trim().is_empty() { 0 } else { marca.chars().count() + 1 };
                if matches!(l.tipo, Tipo::Titulo(1)) {
                    vec![vec![linha]]
                } else {
                    quebrar_texto(linha, largura_conteudo, recuo).into_iter().map(|x| vec![x]).collect()
                }
            };
            // O bloco em inserção aparece NO LUGAR (ciclo 334): o texto
            // sendo digitado substitui a linha, ou entra antes/depois dela
            // quando o bloco é novo.
            if let Some((b, p)) = insercao_no_lugar(e) {
                if l.caminho == b.alvo {
                    let edicao = linhas_em_insercao(l.nivel, &b.prefixo, &p.texto, p.cursor, &e.tema, largura_conteudo);
                    if !b.novo {
                        desenhadas = vec![edicao];
                    } else if b.antes {
                        desenhadas.insert(0, edicao);
                    } else {
                        desenhadas.push(edicao);
                    }
                }
            }
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
        // O texto só vai pro rodapé quando não está sendo editado no
        // lugar (ciclo 334): o bloco de markdown mostra a inserção na
        // própria linha, e o rodapé só diz o modo, como o vim.
        let mut spans = vec![Span::styled(format!(" {}", p.rotulo), e.tema.estilo(Realce::Cursor))];
        if insercao_no_lugar(e).is_none() {
            spans.push(Span::styled(": ", e.tema.estilo(Realce::Cursor)));
            spans.extend(texto_com_cursor(&p.texto, p.cursor, e.tema.estilo(Realce::Cursor), &e.tema));
        }
        spans.push(Span::styled(" ", e.tema.estilo(Realce::Cursor)));
        bloco = bloco.title_bottom(Line::from(spans));
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
    desenhar_abas(f, e, colunas[1]);
    wikilink::desenhar(f, e, colunas[1]);
    // Os modais por cima de tudo (ciclo 339).
    modais::desenhar(f, e);
    // Guardado só agora: `linhas_visiveis` empresta `e` até aqui, e o
    // topo corrigido precisa sobreviver pro próximo quadro — senão a
    // tela "conserta" e desconserta a cada tecla.
    if let Some(i) = novo_topo {
        e.topo = i;
    }
}

/// O valor da parte escondida `campo` entre os filhos de `u`.
fn valor_escondido<'a>(u: &'a Unidade, campo: &str) -> Option<&'a str> {
    u.filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == campo))
        .map(|f| f.texto.as_str())
}

/// `texto` cortado ou completado com espaço até `largura` colunas.
fn na_largura(texto: &str, largura: usize) -> String {
    let mut s: String = texto.chars().take(largura).collect();
    let n = s.chars().count();
    s.push_str(&" ".repeat(largura - n));
    s
}

/// `texto` centralizado em `largura` colunas.
fn centralizado(texto: &str, largura: usize) -> String {
    let texto: String = texto.chars().take(largura).collect();
    let sobra = largura - texto.chars().count();
    format!("{}{}{}", " ".repeat(sobra / 2), texto, " ".repeat(sobra - sobra / 2))
}

/// O cabeçalho da galeria (ciclo 325), como a barra da janela: a
/// contagem à esquerda; à direita o seletor de tamanho `P M G`, com o
/// ativo cheio na cor de destaque, e quantas colunas.
fn linha_do_cabecalho_da_galeria(l: &crate::tela::Linha, arvore: &Unidade, tema: &Tema, largura: usize) -> Line<'static> {
    let recuo = "  ".repeat(l.nivel);
    let cab = arvore.em(&l.caminho);
    let tamanho = cab.and_then(|u| valor_escondido(u, "tamanho")).unwrap_or("md").to_string();
    let colunas = cab.and_then(|u| valor_escondido(u, "colunas")).unwrap_or("3").to_string();
    let mut direita: Vec<Span<'static>> = Vec::new();
    for (slug, rotulo) in [("sm", "P"), ("md", "M"), ("lg", "G")] {
        let estilo = if slug == tamanho {
            tema.miolo_do_botao(Realce::Cartao)
        } else {
            tema.pilula(Realce::Marca)
        };
        direita.push(Span::styled(format!(" {rotulo} "), estilo));
    }
    direita.push(Span::styled(format!("  {colunas} col"), tema.estilo(Realce::Marca)));
    let usado = recuo.chars().count()
        + l.texto.chars().count()
        + direita.iter().map(|s| s.content.chars().count()).sum::<usize>();
    let mut spans = vec![
        Span::styled(recuo, Style::default()),
        Span::styled(l.texto.clone(), tema.estilo(Realce::Marca)),
        Span::styled(" ".repeat(largura.saturating_sub(usado + 1).max(2)), Style::default()),
    ];
    spans.extend(direita);
    Line::from(spans)
}

/// Uma linha da grade da galeria (ciclo 325): as miniaturas lado a lado,
/// cada uma do tamanho de uma coluna da grade.
///
/// A janela mostra a imagem; o terminal não tem como. O que ele mostra é
/// o QUADRO dela — o `.gallery__thumb` com o fundo da página e a borda —,
/// da altura do tamanho escolhido, com o nome do arquivo no meio (é o que
/// a janela faz quando a imagem falta) e a legenda embaixo. Sob o cursor,
/// a borda e a legenda acendem.
fn linhas_da_galeria(
    l: &crate::tela::Linha,
    arvore: &Unidade,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
) -> Vec<Line<'static>> {
    let dono = l.dono_embed.as_deref().unwrap_or(&[]);
    let cab = arvore
        .em(dono)
        .and_then(|u| u.filhos.iter().find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "cabecalho")));
    let colunas: usize = cab.and_then(|u| valor_escondido(u, "colunas")).and_then(|c| c.parse().ok()).unwrap_or(3).max(1);
    let miolo = match cab.and_then(|u| valor_escondido(u, "tamanho")) {
        Some("sm") => 2,
        Some("lg") => 7,
        _ => 4,
    };
    let recuo = "  ".repeat(l.nivel);
    let vao = 2;
    let disponivel = largura.saturating_sub(recuo.len());
    let w = (disponivel.saturating_sub(vao * (colunas - 1)) / colunas).max(6);
    let nome_estilo = Style::default().bg(tema.fundo_da_pagina()).fg(tema.estilo(Realce::Marca).fg.unwrap_or(Color::Gray));
    let borda_comum = Style::default().fg(tema.estilo(Realce::Grade).fg.unwrap_or(Color::DarkGray));

    let mut linhas: Vec<Vec<Span<'static>>> = (0..miolo + 3).map(|_| vec![Span::raw(recuo.clone())]).collect();
    for (k, seg) in l.segmentos.iter().enumerate() {
        if k > 0 {
            for linha in &mut linhas {
                linha.push(Span::raw(" ".repeat(vao)));
            }
        }
        let aceso = cursor.is_some_and(|c| c == seg.caminho.as_slice());
        let borda = if aceso { tema.contorno_do_botao(Realce::Cursor) } else { borda_comum };
        let caminho = arvore.em(&seg.caminho).and_then(|u| valor_escondido(u, "caminho")).unwrap_or("").to_string();
        let arquivo = caminho.rsplit('/').next().unwrap_or("").to_string();
        let dentro = w - 2;
        linhas[0].push(Span::styled(format!("▗{}▖", "▄".repeat(dentro)), borda));
        for r in 0..miolo {
            let meio = if r == miolo / 2 { centralizado(&arquivo, dentro) } else { " ".repeat(dentro) };
            linhas[r + 1].push(Span::styled("▐", borda));
            linhas[r + 1].push(Span::styled(meio, nome_estilo));
            linhas[r + 1].push(Span::styled("▌", borda));
        }
        linhas[miolo + 1].push(Span::styled(format!("▝{}▘", "▀".repeat(dentro)), borda));
        let (legenda, estilo) = if seg.texto != caminho && !seg.texto.is_empty() {
            (seg.texto.clone(), tema.estilo(Realce::Texto))
        } else {
            ("Legenda".to_string(), tema.estilo(Realce::Dica))
        };
        let estilo = if aceso { tema.estilo(Realce::Cursor) } else { estilo };
        linhas[miolo + 2].push(Span::styled(na_largura(&format!(" {legenda}"), w), estilo));
    }
    linhas.into_iter().map(Line::from).collect()
}

/// Quebra uma linha já estilizada em pedaços de `largura` colunas, na
/// última palavra que cabe; a continuação herda o recuo. Cada pedaço sai
/// completo até a largura, com o fundo `fundo` onde o trecho não tem um.
fn quebrar_linha(linha: Line<'static>, largura: usize, fundo: Color) -> Vec<Line<'static>> {
    let celulas: Vec<(char, Style)> =
        linha.spans.iter().flat_map(|s| s.content.chars().map(move |c| (c, s.style))).collect();
    let recuo = celulas.iter().take_while(|(c, _)| *c == ' ').count().min(largura / 2);
    let largura = largura.max(1);
    let mut pedacos: Vec<Vec<(char, Style)>> = Vec::new();
    let mut resto: &[(char, Style)] = &celulas;
    let mut primeiro = true;
    loop {
        let prefixo = if primeiro { 0 } else { recuo };
        let cabe = largura.saturating_sub(prefixo).max(1);
        if resto.len() <= cabe {
            let mut p: Vec<(char, Style)> = vec![(' ', Style::default()); prefixo];
            p.extend_from_slice(resto);
            pedacos.push(p);
            break;
        }
        let corte = resto[..=cabe.min(resto.len() - 1)]
            .iter()
            .rposition(|(c, _)| *c == ' ')
            .filter(|i| *i > 0)
            .unwrap_or(cabe);
        let mut p: Vec<(char, Style)> = vec![(' ', Style::default()); prefixo];
        p.extend_from_slice(&resto[..corte]);
        pedacos.push(p);
        resto = &resto[corte..];
        while resto.first().is_some_and(|(c, _)| *c == ' ') {
            resto = &resto[1..];
        }
        primeiro = false;
        if resto.is_empty() {
            break;
        }
    }
    pedacos
        .into_iter()
        .map(|p| {
            // Sob o cursor a faixa vai até a borda, na cor dele.
            let estilo_final = p.last().map(|(_, s)| *s).filter(|s| s.bg.is_some());
            let mut spans: Vec<Span<'static>> = p
                .iter()
                .map(|(c, s)| Span::styled(c.to_string(), if s.bg.is_some() { *s } else { s.bg(fundo) }))
                .collect();
            let n = p.len();
            if n < largura {
                spans.push(Span::styled(
                    " ".repeat(largura - n),
                    estilo_final.unwrap_or_else(|| Style::default().bg(fundo)),
                ));
            }
            Line::from(spans)
        })
        .collect()
}

/// Um pedaço de linha de largura conhecida: os spans e quantas colunas
/// eles ocupam. É a peça com que o kanban monta colunas lado a lado.
#[derive(Clone, Default)]
struct Faixa {
    spans: Vec<Span<'static>>,
    largura: usize,
}

impl Faixa {
    fn mais(mut self, texto: impl Into<String>, estilo: Style) -> Self {
        let texto = texto.into();
        self.largura += texto.chars().count();
        self.spans.push(Span::styled(texto, estilo));
        self
    }
    /// Acrescenta outra faixa no fim.
    fn juntar(mut self, outra: Faixa) -> Self {
        self.largura += outra.largura;
        self.spans.extend(outra.spans);
        self
    }
    /// Completa com espaço até `largura`.
    fn ate(self, largura: usize, estilo: Style) -> Self {
        let falta = largura.saturating_sub(self.largura);
        self.mais(" ".repeat(falta), estilo)
    }
}

/// `texto` em até `largura` colunas, com reticências se não couber.
fn cortado(texto: &str, largura: usize) -> String {
    if texto.chars().count() <= largura {
        return texto.to_string();
    }
    let mut s: String = texto.chars().take(largura.saturating_sub(1)).collect();
    s.push('…');
    s
}

/// Pílulas lado a lado que QUEBRAM de linha quando não cabem (o
/// `flex-wrap` da janela). Cada linha sai com a largura `largura`.
fn pilulas_quebrando(pilulas: &[(String, Style)], largura: usize, fundo: Style) -> Vec<Faixa> {
    let mut linhas = vec![Faixa::default()];
    for (texto, estilo) in pilulas {
        let texto = cortado(texto, largura);
        let n = texto.chars().count();
        let atual = linhas.last_mut().expect("há sempre uma");
        if atual.largura > 0 && atual.largura + 1 + n > largura {
            linhas.push(Faixa::default());
        }
        let atual = linhas.last_mut().expect("há sempre uma");
        if atual.largura > 0 {
            *atual = std::mem::take(atual).mais(" ", fundo);
        }
        *atual = std::mem::take(atual).mais(texto, *estilo);
    }
    linhas.into_iter().map(|f| f.ate(largura, fundo)).collect()
}

/// O kanban inteiro, com as colunas LADO A LADO (ciclo 328) — o
/// `.kanban__board` da janela.
///
/// Cada coluna é o quadro `--bg-surface` de 30 colunas (os 240px da
/// janela): o nome em caixa alta apagado com a contagem numa pílula, os
/// cartões `--bg-elevated` — título, as insígnias de checklist, prazo,
/// comentários e anexos no fundo da página, as tags em pílula —, e o
/// "+ card" no pé. Depois da última, o "+ coluna" tracejado. Coluna que
/// não cabe na largura desce pra uma nova fileira, em vez de sumir pela
/// borda como a rolagem lateral da janela esconderia.
///
/// Sob o cursor, o cartão fica cheio na cor de destaque; a coluna acende
/// o nome.
fn linhas_do_kanban(e: &Estado, dono: &[usize], largura: usize) -> Vec<Line<'static>> {
    const VAO: usize = 2;
    let Some(embed) = e.arvore.em(dono) else { return Vec::new() };
    // 30 colunas é a largura da janela; encolhe até 24 pra caber mais
    // colunas por fileira antes de descer.
    let disponivel = largura.saturating_sub(2 * dono.len());
    let quantas = embed.filhos.len().max(1);
    let cabem = ((disponivel + VAO) / (24 + VAO)).clamp(1, quantas);
    let w: usize = ((disponivel + VAO) / cabem).saturating_sub(VAO).clamp(24, 30);
    let Some(dados) = embed.fonte.as_deref().and_then(|f| {
        anotadinho_core::embed::segment(f).into_iter().find_map(|s| match s {
            anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Kanban(d)) => Some(d),
            _ => None,
        })
    }) else {
        return Vec::new();
    };
    let t = &e.tema;
    let no_foco = e.foco == Foco::Conteudo;
    let superficie = t.var("bg-surface");
    let elevado = t.var("bg-elevated");
    let base = t.var("bg-base");
    let apagado = t.var("text-muted");
    let texto = t.var("text-primary");
    let borda = t.var("border");
    let nivel = 2 * dono.len();
    let recuo = " ".repeat(nivel);

    let em_superficie = Style::default().bg(superficie);
    let borda_da_coluna = Style::default().fg(superficie);
    let dentro = w - 2;
    let quadros: Vec<Vec<Faixa>> = embed
        .filhos
        .iter()
        .enumerate()
        .map(|(c, coluna)| {
            let caminho_coluna: Vec<usize> = [dono, &[c]].concat();
            let na_coluna = no_foco && e.cursor == caminho_coluna;
            let mut q: Vec<Faixa> = Vec::new();
            q.push(Faixa::default().mais(format!("▗{}▖", "▄".repeat(dentro)), borda_da_coluna));
            // Cabeçalho: NOME ········ ( n ) ×
            let contagem = format!(" {} ", coluna.filhos.len());
            let resto = dentro - 2 - contagem.chars().count() - 2;
            let nome = cortado(&coluna.texto.to_uppercase(), resto);
            let estilo_nome = if na_coluna {
                t.estilo(Realce::Cursor).add_modifier(Modifier::BOLD)
            } else {
                em_superficie.fg(apagado).add_modifier(Modifier::BOLD)
            };
            q.push(
                Faixa::default()
                    .mais("▐", borda_da_coluna)
                    .mais(" ", em_superficie)
                    .mais(nome.clone(), estilo_nome)
                    .mais(" ".repeat(resto - nome.chars().count()), em_superficie)
                    .mais(contagem, Style::default().bg(elevado).fg(apagado))
                    .mais(" ×", em_superficie.fg(apagado))
                    .mais(" ", em_superficie)
                    .mais("▌", borda_da_coluna),
            );
            let cartao_w = dentro - 2;
            let miolo_w = cartao_w - 4;
            for (k, cartao) in coluna.filhos.iter().enumerate() {
                let caminho: Vec<usize> = [dono, &[c, k]].concat();
                let aceso = no_foco && e.cursor == caminho;
                let item = cartao
                    .filhos
                    .iter()
                    .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "indice"))
                    .and_then(|f| f.texto.parse::<usize>().ok())
                    .and_then(|i| dados.items.get(i));
                let fundo_cartao = if aceso { t.estilo(Realce::Cursor).bg.unwrap_or(elevado) } else { elevado };
                let no_cartao = Style::default().bg(fundo_cartao);
                let contorno = Style::default().fg(fundo_cartao).bg(superficie);
                let linha_do_cartao = |miolo: Faixa| -> Faixa {
                    Faixa::default()
                        .mais("▐", borda_da_coluna)
                        .mais(" ", em_superficie)
                        .mais("▐", contorno)
                        .mais(" ", no_cartao)
                        .juntar(miolo.ate(miolo_w, no_cartao))
                        .mais(" ", no_cartao)
                        .mais("▌", contorno)
                        .mais(" ", em_superficie)
                        .mais("▌", borda_da_coluna)
                };
                q.push(
                    Faixa::default()
                        .mais("▐", borda_da_coluna)
                        .mais(" ", em_superficie)
                        .mais(format!("▗{}▖", "▄".repeat(cartao_w - 2)), contorno)
                        .mais(" ", em_superficie)
                        .mais("▌", borda_da_coluna),
                );
                let cor_titulo = if aceso { t.estilo(Realce::Cursor).fg.unwrap_or(base) } else { texto };
                q.push(linha_do_cartao(Faixa::default().mais(cortado(&cartao.texto, miolo_w), no_cartao.fg(cor_titulo))));
                if let Some(item) = item {
                    let mut insignias: Vec<(String, Style)> = Vec::new();
                    let insignia = Style::default().bg(base).fg(apagado);
                    if !item.checklist.is_empty() {
                        let feitos = item.checklist.iter().filter(|c| c.done).count();
                        insignias.push((format!(" ✓ {feitos}/{} ", item.checklist.len()), insignia));
                    }
                    if let Some(prazo) = &item.due {
                        insignias.push((format!(" {prazo} "), insignia));
                    }
                    if !item.comments.is_empty() {
                        insignias.push((format!(" ✎ {} ", item.comments.len()), insignia));
                    }
                    if !item.attachments.is_empty() {
                        insignias.push((format!(" ⌁ {} ", item.attachments.len()), insignia));
                    }
                    for f in pilulas_quebrando(&insignias, miolo_w, no_cartao) {
                        if !insignias.is_empty() {
                            q.push(linha_do_cartao(f));
                        }
                    }
                    let tags: Vec<(String, Style)> =
                        item.tags.iter().map(|tag| (format!(" {tag} "), t.pilula(Realce::BadgeInfo))).collect();
                    if !tags.is_empty() {
                        for f in pilulas_quebrando(&tags, miolo_w, no_cartao) {
                            q.push(linha_do_cartao(f));
                        }
                    }
                }
                q.push(
                    Faixa::default()
                        .mais("▐", borda_da_coluna)
                        .mais(" ", em_superficie)
                        .mais(format!("▝{}▘", "▀".repeat(cartao_w - 2)), contorno)
                        .mais(" ", em_superficie)
                        .mais("▌", borda_da_coluna),
                );
            }
            q.push(
                Faixa::default()
                    .mais("▐", borda_da_coluna)
                    .mais(" ", em_superficie)
                    // A tecla que cria: `Enter` na coluna vazia, `o` no
                    // último cartão (ciclo 337).
                    .mais(
                        na_largura(if coluna.filhos.is_empty() { "↵ + card" } else { "o + card" }, dentro - 2),
                        em_superficie.fg(apagado),
                    )
                    .mais(" ", em_superficie)
                    .mais("▌", borda_da_coluna),
            );
            q.push(Faixa::default().mais(format!("▝{}▘", "▀".repeat(dentro)), borda_da_coluna));
            q
        })
        .collect();

    // O "+ coluna" tracejado, com a largura dos 200px da janela.
    const W_NOVA: usize = 25;
    let tracejado = Style::default().fg(borda);
    let nova_coluna = vec![
        Faixa::default().mais(format!("┌{}┐", "╌".repeat(W_NOVA - 2)), tracejado),
        Faixa::default()
            .mais("╎", tracejado)
            .mais(centralizado("o + coluna", W_NOVA - 2), Style::default().fg(apagado))
            .mais("╎", tracejado),
        Faixa::default().mais(format!("└{}┘", "╌".repeat(W_NOVA - 2)), tracejado),
    ];

    let por_fileira = ((disponivel + VAO) / (w + VAO)).max(1);
    let mut fora: Vec<Line<'static>> = Vec::new();
    let mut fileiras: Vec<Vec<(Vec<Faixa>, usize)>> =
        quadros.chunks(por_fileira).map(|f| f.iter().map(|q| (q.clone(), w)).collect()).collect();
    let cabe_na_ultima = fileiras
        .last()
        .map(|f| f.len() * (w + VAO) + W_NOVA <= disponivel)
        .unwrap_or(true);
    if cabe_na_ultima && !fileiras.is_empty() {
        fileiras.last_mut().expect("não vazia").push((nova_coluna, W_NOVA));
    } else {
        fileiras.push(vec![(nova_coluna, W_NOVA)]);
    }
    for (n, fileira) in fileiras.iter().enumerate() {
        if n > 0 {
            fora.push(Line::from(""));
        }
        let altura = fileira.iter().map(|(q, _)| q.len()).max().unwrap_or(0);
        for r in 0..altura {
            let mut spans = vec![Span::raw(recuo.clone())];
            for (i, (q, w)) in fileira.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" ".repeat(VAO)));
                }
                match q.get(r) {
                    Some(f) => spans.extend(f.spans.iter().cloned()),
                    None => spans.push(Span::raw(" ".repeat(*w))),
                }
            }
            fora.push(Line::from(spans));
        }
    }
    fora
}

/// A pílula de um valor de consulta: a cor sai do VALOR
/// (`indice_cor_consulta`), como `.query-embed__chip--cor-N`.
fn chip_da_consulta(valor: &str, tema: &Tema) -> Style {
    let elevado = tema.var("bg-elevated");
    let (cor, pct) = match anotadinho_core::query::indice_cor_consulta(valor) {
        0 => (tema.var("accent-blue"), 0.24),
        1 => (tema.var("accent-purple"), 0.25),
        2 => (tema.var("success"), 0.30),
        3 => (tema.var("warning"), 0.32),
        4 => (tema.var("error"), 0.24),
        _ => (tema.var("text-muted"), 0.28),
    };
    Style::default().fg(tema.var("text-primary")).bg(crate::tema::misturar(cor, elevado, pct))
}

/// A consulta inteira (ciclo 331), como `.query-embed`: a barra com a
/// lupa, o recorte e a contagem; embaixo os resultados na visão do
/// arquivo — lista (título e pílulas à direita), tabela (cabeçalho em
/// caixa alta, uma coluna por campo) ou cartões (grade de quadros com
/// título, caminho e pílulas) —, cada linha com o traço de baixo da
/// janela. Com grupos, um cabeçalho por grupo com a seta, o total e os
/// agregados. Sob o cursor, a linha acende inteira.
fn linhas_da_consulta(e: &Estado, dono: &[usize], largura: usize) -> Vec<Line<'static>> {
    use anotadinho_core::query::QueryView;
    let Some(embed) = e.arvore.em(dono) else { return Vec::new() };
    let t = &e.tema;
    let consulta = embed.fonte.as_deref().and_then(|f| {
        anotadinho_core::embed::segment(f).into_iter().find_map(|s| match s {
            anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Query(q)) => Some(q),
            _ => None,
        })
    });
    let visao = consulta.as_ref().map(|q| q.view).unwrap_or_default();
    let colunas: Vec<String> = consulta.as_ref().map(|q| q.columns.clone()).unwrap_or_default();
    let recuo = " ".repeat(2 * dono.len());
    let w = largura.saturating_sub(recuo.len()).max(20);
    let apagado = Style::default().fg(t.var("text-muted"));
    let traco = Style::default().fg(t.var("border"));
    let no_foco = e.foco == Foco::Conteudo;
    let aceso = |c: &[usize]| no_foco && e.cursor == c;
    let fundo_aceso = Style::default().bg(crate::tema::misturar(t.var("accent-blue"), t.var("bg-surface"), 0.25));
    let mut fora: Vec<Faixa> = Vec::new();
    let regua = || Faixa::default().mais("─".repeat(w), traco);

    // A barra: ⌕ recorte ····· N páginas ✲
    let posicao_da_busca = embed.filhos.iter().position(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "busca"));
    let cab = posicao_da_busca.map(|i| &embed.filhos[i]);
    let descricao = cab.map(|c| c.texto.clone()).unwrap_or_else(|| {
        consulta.as_ref().map(|q| q.descrever()).unwrap_or_default()
    });
    let contagem = cab.and_then(|c| valor_escondido(c, "contagem")).unwrap_or("").to_string();
    // A barra é o campo de busca (ciclo 338): o cursor pousa nela, e
    // `Enter` (ou `a`) edita o filtro.
    let busca_acesa = posicao_da_busca.is_some_and(|i| aceso(&[dono, &[i]].concat()));
    let fundo_da_busca = if busca_acesa { fundo_aceso } else { Style::default() };
    let direita = format!("{contagem}  ↵ filtrar ");
    let cabe = w.saturating_sub(direita.chars().count() + 5);
    fora.push(
        Faixa::default()
            .mais(" ⌕ ", fundo_da_busca.fg(t.var("text-muted")))
            .mais(cortado(&descricao, cabe), if busca_acesa { fundo_da_busca.fg(t.var("text-primary")) } else { apagado })
            .ate(w - direita.chars().count() - 1, fundo_da_busca)
            .mais(direita, fundo_da_busca.fg(t.var("text-muted"))),
    );
    fora.push(regua());

    // Os campos de um resultado, na ordem das colunas.
    let campos = |u: &Unidade| -> Vec<String> {
        u.filhos
            .iter()
            .filter(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "campo"))
            .map(|f| f.texto.split_once('=').map(|(_, v)| v.to_string()).unwrap_or_default())
            .collect()
    };
    let e_parte = |u: &Unidade, n: &str| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == n);

    if let Some(nada) = embed.filhos.iter().find(|f| e_parte(f, "nada")) {
        fora.push(Faixa::default().mais(format!(" {}", nada.texto), apagado));
    }

    // Todos os resultados (com o caminho), pra medir colunas.
    let mut todos: Vec<(Vec<usize>, &Unidade)> = Vec::new();
    for (i, f) in embed.filhos.iter().enumerate() {
        if e_parte(f, "resultado") {
            todos.push(([dono, &[i]].concat(), f));
        } else if e_parte(f, "grupo") {
            for (k, r) in f.filhos.iter().enumerate() {
                if e_parte(r, "resultado") {
                    todos.push(([dono, &[i, k]].concat(), r));
                }
            }
        }
    }

    let linha_de_lista = |caminho: &[usize], r: &Unidade| -> Faixa {
        let fundo = if aceso(caminho) { fundo_aceso } else { Style::default() };
        let chips: Vec<String> = campos(r).into_iter().filter(|v| !v.is_empty()).collect();
        let largura_chips: usize = chips.iter().map(|c| c.chars().count() + 3).sum();
        let titulo = cortado(&r.texto, w.saturating_sub(largura_chips + 3));
        let mut f = Faixa::default()
            .mais(" ", fundo)
            .mais(titulo, fundo.fg(t.var("text-primary")).add_modifier(Modifier::BOLD))
            .ate(w.saturating_sub(largura_chips + 1), fundo);
        for c in chips {
            f = f.mais(" ", fundo).mais(format!(" {c} "), chip_da_consulta(&c, t));
        }
        f.ate(w, fundo)
    };

    match visao {
        QueryView::Table if !todos.is_empty() => {
            // Largura de cada coluna pelo maior conteúdo, com o título
            // levando o que sobrar.
            let mut larguras: Vec<usize> = colunas
                .iter()
                .enumerate()
                .map(|(i, c)| {
                    todos
                        .iter()
                        .map(|(_, r)| campos(r).get(i).map_or(0, |v| v.chars().count() + 2))
                        .max()
                        .unwrap_or(0)
                        .max(c.chars().count())
                        + 3
                })
                .collect();
            let resto: usize = larguras.iter().sum();
            let titulo_w = w.saturating_sub(resto + 1).max(12);
            if titulo_w + resto + 1 > w {
                for l in &mut larguras {
                    *l = (*l).min(14);
                }
            }
            let mut cabecalho = Faixa::default().mais(" ", Style::default()).mais(
                na_largura("PÁGINA", titulo_w),
                apagado.add_modifier(Modifier::BOLD),
            );
            for (c, lw) in colunas.iter().zip(&larguras) {
                cabecalho = cabecalho.mais(na_largura(&c.to_uppercase(), *lw), apagado.add_modifier(Modifier::BOLD));
            }
            fora.push(cabecalho.ate(w, Style::default()));
            fora.push(regua());
            for (caminho, r) in &todos {
                let fundo = if aceso(caminho) { fundo_aceso } else { Style::default() };
                let mut f = Faixa::default()
                    .mais(" ", fundo)
                    .mais(na_largura(&cortado(&r.texto, titulo_w - 1), titulo_w), fundo.fg(t.var("text-primary")));
                for (v, lw) in campos(r).iter().zip(&larguras) {
                    if v.is_empty() {
                        f = f.mais(" ".repeat(*lw), fundo);
                    } else {
                        let chip = cortado(v, lw.saturating_sub(3));
                        let n = chip.chars().count() + 2;
                        f = f.mais(format!(" {chip} "), chip_da_consulta(v, t)).mais(" ".repeat(lw.saturating_sub(n)), fundo);
                    }
                }
                fora.push(f.ate(w, fundo));
                fora.push(regua());
            }
            fora.pop();
        }
        QueryView::Cards if !todos.is_empty() => {
            const MIN: usize = 26;
            let por_linha = ((w + 2) / (MIN + 2)).max(1);
            let cw = (w + 2) / por_linha - 2;
            let base = t.var("bg-base");
            for fileira in todos.chunks(por_linha) {
                let quadros: Vec<Vec<Faixa>> = fileira
                    .iter()
                    .map(|(caminho, r)| {
                        let borda = if aceso(caminho) { t.var("accent-blue") } else { t.var("border") };
                        let no_cartao = Style::default().bg(base);
                        let lado = |miolo: Faixa| {
                            Faixa::default()
                                .mais("▐", Style::default().fg(borda))
                                .mais(" ", no_cartao)
                                .juntar(miolo.ate(cw - 4, no_cartao))
                                .mais(" ", no_cartao)
                                .mais("▌", Style::default().fg(borda))
                        };
                        let mut q = vec![Faixa::default().mais(format!("▗{}▖", "▄".repeat(cw - 2)), Style::default().fg(borda))];
                        q.push(lado(Faixa::default().mais(cortado(&r.texto, cw - 4), no_cartao.fg(t.var("text-primary")).add_modifier(Modifier::BOLD))));
                        let pagina = valor_escondido(r, "pagina").unwrap_or("");
                        q.push(lado(Faixa::default().mais(cortado(pagina, cw - 4), no_cartao.fg(t.var("text-muted")))));
                        let chips: Vec<(String, Style)> = campos(r)
                            .into_iter()
                            .filter(|v| !v.is_empty())
                            .map(|v| (format!(" {v} "), chip_da_consulta(&v, t)))
                            .collect();
                        for f in pilulas_quebrando(&chips, cw - 4, no_cartao) {
                            if !chips.is_empty() {
                                q.push(lado(f));
                            }
                        }
                        q.push(Faixa::default().mais(format!("▝{}▘", "▀".repeat(cw - 2)), Style::default().fg(borda)));
                        q
                    })
                    .collect();
                let altura = quadros.iter().map(Vec::len).max().unwrap_or(0);
                for y in 0..altura {
                    let mut f = Faixa::default();
                    for (i, q) in quadros.iter().enumerate() {
                        if i > 0 {
                            f = f.mais("  ", Style::default());
                        }
                        f = match q.get(y) {
                            Some(p) => f.juntar(p.clone()),
                            None => f.mais(" ".repeat(cw), Style::default()),
                        };
                    }
                    fora.push(f);
                }
            }
        }
        _ => {
            for (i, f) in embed.filhos.iter().enumerate() {
                let caminho: Vec<usize> = [dono, &[i]].concat();
                if e_parte(f, "resultado") {
                    fora.push(linha_de_lista(&caminho, f));
                    fora.push(regua());
                } else if e_parte(f, "grupo") {
                    let fundo = if aceso(&caminho) { fundo_aceso } else { Style::default() };
                    let aberto = f.filhos.iter().any(|r| e_parte(r, "resultado"));
                    let mut cab = Faixa::default()
                        .mais(if aberto { " ▾ " } else { " ▸ " }, fundo.fg(t.var("text-muted")))
                        .mais(f.texto.clone(), fundo.fg(t.var("text-primary")).add_modifier(Modifier::BOLD))
                        .mais(format!("  {}", valor_escondido(f, "total").unwrap_or("0")), fundo.fg(t.var("text-muted")));
                    for a in f.filhos.iter().filter(|a| e_parte(a, "agregado")) {
                        cab = cab.mais(" ", fundo).mais(format!(" {} ", a.texto), Style::default().bg(t.var("bg-elevated")).fg(t.var("text-muted")));
                    }
                    fora.push(cab.ate(w, fundo));
                    fora.push(regua());
                    for (k, r) in f.filhos.iter().enumerate() {
                        if e_parte(r, "resultado") {
                            fora.push(linha_de_lista(&[caminho.as_slice(), &[k]].concat(), r));
                            fora.push(regua());
                        }
                    }
                }
            }
            if fora.last().is_some_and(|f| f.spans.first().is_some_and(|s| s.content.starts_with('─'))) && fora.len() > 2 {
                fora.pop();
            }
        }
    }
    fora.into_iter()
        .map(|f| Line::from([vec![Span::raw(recuo.clone())], f.spans].concat()))
        .collect()
}

/// Quebra uma linha estilizada na largura, na última palavra que cabe
/// (ciclo 338). `\n` dentro do texto força a quebra — é o parágrafo de
/// várias linhas do arquivo. A continuação começa em `recuo` colunas.
/// Nada de fundo é pintado: o fundo é de quem desenha em volta.
fn quebrar_texto(linha: Line<'_>, largura: usize, recuo: usize) -> Vec<Line<'static>> {
    let largura = largura.max(8);
    let recuo = recuo.min(largura / 2);
    let celulas: Vec<(char, Style)> =
        linha.spans.iter().flat_map(|s| s.content.chars().map(move |c| (c, s.style))).collect();
    let mut fora: Vec<Vec<(char, Style)>> = Vec::new();
    let mut atual: Vec<(char, Style)> = Vec::new();
    let quebrar = |atual: &mut Vec<(char, Style)>, fora: &mut Vec<Vec<(char, Style)>>| {
        fora.push(std::mem::take(atual));
        atual.extend(std::iter::repeat_n((' ', Style::default()), recuo));
    };
    for (c, st) in celulas {
        if c == '\n' {
            quebrar(&mut atual, &mut fora);
            continue;
        }
        if atual.len() >= largura {
            // Volta até o último espaço depois do recuo, se houver.
            let corte = atual.iter().rposition(|(x, _)| *x == ' ').filter(|i| *i > recuo);
            match corte {
                Some(i) => {
                    let resto: Vec<(char, Style)> = atual.split_off(i + 1);
                    atual.pop();
                    fora.push(std::mem::take(&mut atual));
                    atual.extend(std::iter::repeat_n((' ', Style::default()), recuo));
                    atual.extend(resto);
                }
                None => quebrar(&mut atual, &mut fora),
            }
        }
        if !(c == ' ' && atual.len() == recuo && !fora.is_empty()) {
            atual.push((c, st));
        }
    }
    fora.push(atual);
    fora.into_iter()
        .map(|cs| {
            // Junta células vizinhas de mesmo estilo num span só.
            let mut spans: Vec<Span<'static>> = Vec::new();
            let mut texto = String::new();
            let mut estilo: Option<Style> = None;
            for (c, st) in cs {
                if estilo != Some(st) {
                    if let Some(e) = estilo {
                        spans.push(Span::styled(std::mem::take(&mut texto), e));
                    }
                    estilo = Some(st);
                }
                texto.push(c);
            }
            if let Some(e) = estilo {
                spans.push(Span::styled(texto, e));
            }
            Line::from(spans)
        })
        .collect()
}

/// O embed de colunas inteiro, com os painéis LADO A LADO (ciclo 326).
///
/// Na janela é uma grade de painéis de markdown, cada um com a sua
/// fração da largura (`2fr 1fr`). Aqui cada painel é um quadro com o
/// fundo da página por dentro — é o que a janela mostra, o embed não tem
/// caixa própria —, a largura `Nfr` em cima (a barra que a janela mostra
/// no foco) e o markdown dele com o mesmo desenho do resto da página,
/// quebrado na largura do painel. O painel com o cursor acende a borda.
fn linhas_das_colunas(
    e: &Estado,
    dono: &[usize],
    linhas_visiveis: &[&crate::tela::Linha],
    largura: usize,
) -> Vec<Line<'static>> {
    let Some(embed) = e.arvore.em(dono) else { return Vec::new() };
    let paineis = &embed.filhos;
    if paineis.is_empty() {
        return Vec::new();
    }
    let nivel = linhas_visiveis
        .iter()
        .find(|f| f.caminho.len() == dono.len() + 1 && f.caminho.starts_with(dono))
        .map_or(1, |f| f.nivel);
    let recuo = "  ".repeat(nivel);
    let vao = 2;
    let fracoes: Vec<usize> =
        paineis.iter().map(|p| valor_escondido(p, "largura").and_then(|w| w.parse().ok()).unwrap_or(1).max(1)).collect();
    let total: usize = fracoes.iter().sum();
    let disponivel = largura.saturating_sub(recuo.len() + vao * (paineis.len() - 1));
    let mut larguras: Vec<usize> = fracoes.iter().map(|f| (disponivel * f / total).max(8)).collect();
    let usado: usize = larguras.iter().sum();
    if let Some(ultima) = larguras.last_mut() {
        *ultima = (*ultima + disponivel.saturating_sub(usado)).max(8);
    }
    let fundo = e.tema.fundo_da_pagina();
    let no_foco = e.foco == Foco::Conteudo;
    let borda_comum = Style::default().fg(e.tema.estilo(Realce::Grade).fg.unwrap_or(Color::DarkGray));

    let colunas: Vec<Vec<Line<'static>>> = paineis
        .iter()
        .enumerate()
        .map(|(i, _)| {
            let caminho: Vec<usize> = [dono, &[i]].concat();
            let w = larguras[i];
            let dentro = w.saturating_sub(4).max(1);
            let aceso = no_foco && e.cursor.starts_with(&caminho);
            let borda = if aceso { e.tema.contorno_do_botao(Realce::Cursor) } else { borda_comum };
            let mut miolo: Vec<Line<'static>> = vec![Line::from(Span::styled(
                format!("{:>dentro$}", format!("{}fr", fracoes[i])),
                Style::default().bg(fundo).fg(e.tema.estilo(Realce::Marca).fg.unwrap_or(Color::Gray)),
            ))];
            let conteudo: Vec<&crate::tela::Linha> = linhas_visiveis
                .iter()
                .copied()
                .filter(|f| f.caminho.len() > caminho.len() && f.caminho.starts_with(&caminho))
                .filter(|f| !matches!(f.tipo, Tipo::Lista | Tipo::ListaOrdenada) || e.dobrados.contains(&f.caminho))
                .collect();
            let base = conteudo.iter().map(|f| f.nivel).min().unwrap_or(0);
            if conteudo.is_empty() {
                miolo.push(Line::from(Span::styled(
                    na_largura("Painel vazio", dentro),
                    e.tema.estilo(Realce::Dica).bg(fundo),
                )));
            }
            for f in conteudo {
                let mut f = f.clone();
                f.nivel -= base;
                let sob_cursor = no_foco && f.mostra(&e.cursor);
                let linha = linha_estilizada(&f, sob_cursor, e.dobrados.contains(&f.caminho), &e.tema, dentro);
                let linha = Line::from(
                    linha.spans.into_iter().map(|s| Span::styled(s.content.into_owned(), s.style)).collect::<Vec<_>>(),
                );
                miolo.extend(quebrar_linha(linha, dentro, fundo));
            }
            let mut quadro = vec![Line::from(Span::styled(format!("▗{}▖", "▄".repeat(w - 2)), borda))];
            for m in miolo {
                let mut spans = vec![Span::styled("▐", borda), Span::styled(" ", Style::default().bg(fundo))];
                spans.extend(m.spans);
                spans.push(Span::styled(" ", Style::default().bg(fundo)));
                spans.push(Span::styled("▌", borda));
                quadro.push(Line::from(spans));
            }
            quadro.push(Line::from(Span::styled(format!("▝{}▘", "▀".repeat(w - 2)), borda)));
            quadro
        })
        .collect();

    // Os painéis têm a altura do mais alto: o fundo de dentro continua
    // até a borda de baixo, como a grade da janela alinha pelo topo mas
    // a caixa aqui fecha junta.
    let altura = colunas.iter().map(|c| c.len()).max().unwrap_or(0);
    (0..altura)
        .map(|r| {
            let mut spans = vec![Span::raw(recuo.clone())];
            for (i, quadro) in colunas.iter().enumerate() {
                if i > 0 {
                    spans.push(Span::raw(" ".repeat(vao)));
                }
                let w = larguras[i];
                let aceso = no_foco && e.cursor.starts_with(&[dono, &[i]].concat());
                let borda = if aceso { e.tema.contorno_do_botao(Realce::Cursor) } else { borda_comum };
                let n = quadro.len();
                if r < n - 1 {
                    spans.extend(quadro[r].spans.iter().cloned());
                } else if r == altura - 1 {
                    spans.extend(quadro[n - 1].spans.iter().cloned());
                } else {
                    spans.push(Span::styled("▐", borda));
                    spans.push(Span::styled(" ".repeat(w - 2), Style::default().bg(fundo)));
                    spans.push(Span::styled("▌", borda));
                }
            }
            Line::from(spans)
        })
        .collect()
}

/// A edição de bloco aberta, quando ela aparece no lugar do bloco: um
/// bloco da página ou do corpo de um callout que tem linha própria na
/// tela. Num painel de colunas, ou ao lado de um embed, o texto vai pro
/// rodapé.
fn insercao_no_lugar(e: &Estado) -> Option<(&EdicaoDeBloco, &Pergunta)> {
    let p = e.pergunta.as_ref()?;
    let AcaoDaPergunta::Bloco(b) = &p.acao else { return None };
    if matches!(b.hospedeiro, Hospedeiro::Painel(..)) {
        return None;
    }
    let u = e.arvore.em(&b.alvo)?;
    if matches!(u.tipo, Tipo::Embed(_) | Tipo::Parte { .. }) || b.alvo.is_empty() {
        return None;
    }
    Some((b, p))
}

/// O texto com o cursor de inserção: a célula do cursor em vídeo inverso
/// (um espaço, no fim).
fn texto_com_cursor(texto: &str, cursor: usize, estilo: Style, tema: &Tema) -> Vec<Span<'static>> {
    let chars: Vec<char> = texto.chars().collect();
    let c = cursor.min(chars.len());
    let antes: String = chars[..c].iter().collect();
    let sob: String = chars.get(c).map(|x| x.to_string()).unwrap_or_else(|| " ".into());
    let depois: String = chars.get(c + 1..).map(|r| r.iter().collect()).unwrap_or_default();
    let invertido = Style::default().fg(tema.var("bg-base")).bg(tema.var("text-primary"));
    vec![Span::styled(antes, estilo), Span::styled(sob, invertido), Span::styled(depois, estilo)]
}

/// As linhas de um bloco em inserção: o recuo dele, a marca apagada e o
/// texto com o cursor, quebrado na largura.
fn linhas_em_insercao(nivel: usize, prefixo: &str, texto: &str, cursor: usize, tema: &Tema, largura: usize) -> Vec<Line<'static>> {
    let mut spans = vec![Span::raw("  ".repeat(nivel))];
    if !prefixo.is_empty() {
        spans.push(Span::styled(prefixo.to_string(), tema.estilo(Realce::Marca)));
    }
    spans.extend(texto_com_cursor(texto, cursor, tema.estilo(Realce::Texto), tema));
    // A quebra pinta de fundo o que não tem; aqui o fundo é o da região
    // (página ou caixa do callout), então esse fundo sai de volta.
    let fundo = tema.var("bg-base");
    quebrar_linha(Line::from(spans), largura.max(8), fundo)
        .into_iter()
        .map(|l| {
            Line::from(
                l.spans
                    .into_iter()
                    .map(|s| {
                        let estilo = if s.style.bg == Some(fundo) { Style { bg: None, ..s.style } } else { s.style };
                        Span::styled(s.content, estilo)
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect()
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
    arvore: &Unidade,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
) -> Vec<Vec<Line<'static>>> {
    let recuo = "  ".repeat(l.nivel);
    let disponivel = largura.saturating_sub(recuo.len()).max(4);
    let dono = l.dono_embed.as_deref().unwrap_or(&[]);

    // A trilha do fluxo é de PÍLULAS, não de botões (ciclo 330): uma
    // linha só, cada etapa na cor do seu estado — feita, atual, futura —
    // como `.fluxo__passo`. "Bloqueada" é exceção e não entra na trilha,
    // como na janela, a não ser que seja onde o fluxo está.
    if l.segmentos.iter().any(|s| s.nome.starts_with("etapa")) {
        let atual = l.segmentos.iter().position(|s| s.nome == "etapa-atual");
        let superficie = tema.var("bg-surface");
        let pilulas: Vec<(String, Style)> = l
            .segmentos
            .iter()
            .enumerate()
            .filter(|(i, s)| s.texto != "Bloqueada" || Some(*i) == atual)
            .map(|(i, s)| {
                let aceso = cursor.is_some_and(|c| c == s.caminho.as_slice());
                let estilo = if aceso {
                    tema.estilo(Realce::Cursor)
                } else {
                    match atual {
                        Some(a) if i < a => {
                            let cor = tema.var("success");
                            Style::default().fg(cor).bg(crate::tema::misturar(cor, superficie, 0.14))
                        }
                        Some(a) if i == a => Style::default()
                            .fg(tema.var("text-primary"))
                            .bg(crate::tema::misturar(tema.var("accent-blue"), superficie, 0.22)),
                        _ => Style::default().fg(tema.var("text-muted")).bg(tema.var("bg-elevated")),
                    }
                };
                (format!(" {} ", s.texto), estilo)
            })
            .collect();
        return pilulas_quebrando(&pilulas, disponivel, Style::default())
            .into_iter()
            .map(|f| vec![Line::from([vec![Span::raw(recuo.clone())], f.spans].concat())])
            .collect();
    }

    // Os ícones dos botões de ações, lidos do arquivo pela posição.
    let icones: Vec<Option<String>> = arvore
        .em(dono)
        .filter(|u| matches!(&u.tipo, Tipo::Embed(n) if n == "actions"))
        .and_then(|u| u.fonte.as_deref())
        .and_then(|f| {
            anotadinho_core::embed::segment(f).into_iter().find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(anotadinho_core::embed::EmbedData::Actions(d)) => Some(d),
                _ => None,
            })
        })
        .map(|d| d.buttons.iter().map(|b| b.icon.clone()).collect())
        .unwrap_or_default();
    let rotulo = |seg: &crate::tela::Segmento| -> String {
        let icone = seg.caminho.last().and_then(|i| icones.get(*i)).and_then(|i| i.as_deref());
        match icone {
            Some(nome) => format!("{} {}", glifo_do_icone(nome), seg.texto),
            None => seg.texto.clone(),
        }
    };
    let e_acoes = !icones.is_empty() || arvore.em(dono).is_some_and(|u| matches!(&u.tipo, Tipo::Embed(n) if n == "actions"));

    // Empacota em faixas que cabem. Um botão sozinho maior que a
    // largura fica na sua própria faixa e é cortado — não há o que
    // fazer, e cortar um é melhor que cortar todos depois dele.
    let mut itens: Vec<(String, Option<&crate::tela::Segmento>)> =
        l.segmentos.iter().map(|s| (rotulo(s), Some(s))).collect();
    // O "+ ação" da janela, depois do último botão: não é destino do
    // cursor, só o lembrete de onde se cria (com `o`).
    if e_acoes {
        itens.push(("+ ação".to_string(), None));
    }
    let mut faixas: Vec<Vec<&(String, Option<&crate::tela::Segmento>)>> = Vec::new();
    let mut faixa = Vec::new();
    let mut usado = 0usize;
    for item in &itens {
        let largura_seg = largura_do_botao(&item.0);
        let custo = if faixa.is_empty() { largura_seg } else { largura_seg + VAO };
        if !faixa.is_empty() && usado + custo > disponivel {
            faixas.push(std::mem::take(&mut faixa));
            usado = largura_seg;
        } else {
            usado += custo;
        }
        faixa.push(item);
    }
    if !faixa.is_empty() {
        faixas.push(faixa);
    }

    let mut fora = Vec::with_capacity(faixas.len());
    for faixa in faixas {
        let mut topo = vec![Span::styled(recuo.clone(), Style::default())];
        let mut meio = vec![Span::styled(recuo.clone(), Style::default())];
        let mut base = vec![Span::styled(recuo.clone(), Style::default())];
        for (i, (texto, seg)) in faixa.iter().enumerate() {
            if i > 0 {
                let vao = " ".repeat(VAO);
                topo.push(Span::styled(vao.clone(), Style::default()));
                meio.push(Span::styled(vao.clone(), Style::default()));
                base.push(Span::styled(vao, Style::default()));
            }
            let nome = seg.map(|s| s.nome.as_str()).unwrap_or("adicionar");
            let aceso = seg.is_some_and(|s| cursor.is_some_and(|c| c == s.caminho.as_slice()));
            let (fundo, estilo_texto) = aparencia_do_botao(nome, tema);
            // Sob o cursor, o botão ganha o CONTORNO na cor de destaque
            // (o `:focus-visible` da janela) e mantém o preenchimento — um
            // botão primário aceso continua dizendo que é o primário.
            let destaque = tema.var("accent-blue");
            let cor_contorno = match (aceso, fundo) {
                (true, Some(f)) if f == destaque => Some(tema.var("text-primary")),
                (true, _) => Some(destaque),
                (false, f) => f,
            };
            let largura_miolo = texto.chars().count() + 2;
            let (cima, lado_e, lado_d, baixo) = match cor_contorno {
                Some(_) => tema.contorno_em_volta(largura_miolo),
                None => (" ".repeat(largura_miolo + 2), " ", " ", " ".repeat(largura_miolo + 2)),
            };
            let contorno = match cor_contorno {
                Some(c) => Style::default().fg(c),
                None => Style::default(),
            };
            let miolo = match fundo {
                Some(f) => estilo_texto.bg(f),
                None => estilo_texto,
            };
            // Contorno de MEIO-BLOCO: cada célula pinta a metade que
            // olha pra dentro, e deixa a de fora com o fundo da tela.
            // Com contorno aceso sobre fundo próprio, a metade de dentro
            // fica com o fundo do botão.
            let contorno_lateral = match (aceso, fundo) {
                (true, Some(f)) => contorno.bg(f),
                _ => contorno,
            };
            topo.push(Span::styled(cima, contorno));
            meio.push(Span::styled(lado_e, contorno_lateral));
            meio.push(Span::styled(format!(" {texto} "), miolo));
            meio.push(Span::styled(lado_d, contorno_lateral));
            base.push(Span::styled(baixo, contorno));
        }
        fora.push(vec![Line::from(topo), Line::from(meio), Line::from(base)]);
    }
    fora
}

/// As linhas de texto do fluxo (ciclo 330): o topo é o `.fluxo__topo`
/// da janela — o artefato em caixa alta apagado e a etapa numa pílula na
/// cor dela (azul; verde concluída; âmbar bloqueada) —, e a dica e a
/// nota saem apagadas, sem itálico.
fn linha_do_fluxo(l: &crate::tela::Linha, nome: &str, tema: &Tema) -> Line<'static> {
    let recuo = Span::raw("  ".repeat(l.nivel));
    let apagado = Style::default().fg(tema.var("text-muted"));
    if nome != "titulo" {
        return Line::from(vec![recuo, Span::styled(l.texto.clone(), apagado)]);
    }
    let (artefato, etapa) = l.texto.split_once(": ").unwrap_or((l.texto.as_str(), ""));
    let cor = match etapa {
        "Concluída" => tema.var("success"),
        "Bloqueada" => tema.var("warning"),
        _ => tema.var("accent-blue"),
    };
    let pilula = Style::default()
        .fg(cor)
        .bg(crate::tema::misturar(cor, tema.var("bg-surface"), 0.16))
        .add_modifier(Modifier::BOLD);
    Line::from(vec![
        recuo,
        Span::styled(artefato.to_string(), apagado),
        Span::raw("  "),
        Span::styled(format!(" {etapa} "), pilula),
    ])
}

/// O fundo e o texto de um botão, pelo papel — as classes da janela:
///
/// - `button` é `.actions-embed__btn`: superfície, texto normal;
/// - `button-primary` e `acao` de aprovada: `--accent-blue` com texto no
///   fundo da página;
/// - `acao` (o "Pedir alteração"): `.btn` elevado;
/// - `transicao-principal`: o `.btn--primary` do fluxo, o gradiente azul
///   → roxo, aqui no meio dele;
/// - `transicao`: `.btn--ghost`, sem fundo, texto apagado;
/// - `adicionar` ("+ ação"): elevado e apagado.
///
/// `None` no fundo é botão sem caixa.
fn aparencia_do_botao(nome: &str, tema: &Tema) -> (Option<Color>, Style) {
    let base = tema.var("bg-base");
    match nome {
        "button" => (Some(tema.var("bg-surface")), Style::default().fg(tema.var("text-primary"))),
        "button-primary" => (Some(tema.var("accent-blue")), Style::default().fg(base)),
        "acao" => (Some(tema.var("bg-elevated")), Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD)),
        "transicao-principal" => (
            Some(crate::tema::misturar(tema.var("accent-blue"), tema.var("accent-purple"), 0.5)),
            Style::default().fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD),
        ),
        "transicao" | "origem" => (None, Style::default().fg(tema.var("text-muted"))),
        "adicionar" => (Some(tema.var("bg-elevated")), Style::default().fg(tema.var("text-muted"))),
        outro => {
            let papel = papel_da_parte(outro);
            (Some(tema.contorno_do_botao(papel).fg.unwrap_or(Color::Gray)), tema.miolo_do_botao(papel))
        }
    }
}

/// Um glifo de uma célula pro ícone de um botão de ações (os nomes de
/// `components/icon.rs`). Nada de emoji: ele ocupa duas células em
/// metade dos terminais e desalinha a caixa.
pub(super) fn glifo_do_icone(nome: &str) -> &'static str {
    match nome {
        "search" => "⌕",
        "home" => "⌂",
        "file-text" => "≡",
        "folder" => "▭",
        "calendar" | "table" => "▦",
        "check" => "✓",
        "edit" => "✎",
        "link" | "external-link" => "↗",
        "clock" => "◷",
        "network" => "⋈",
        "image" => "▨",
        "columns" => "▥",
        "layout" => "▤",
        "settings" => "✲",
        "download" => "↓",
        "git-branch" => "⑂",
        "message-circle" => "○",
        "paperclip" => "⌁",
        "info" => "ℹ",
        "lightbulb" => "✱",
        _ => "ϟ",
    }
}

/// O papel de um badge pelo sufixo do nome (`--info`, `--success`…),
/// o mesmo de `badge_class` no núcleo.
pub(super) fn papel_do_badge(sufixo: &str) -> Option<Realce> {
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
    // O "+ linha" da janela embaixo da última (ciclo 336) — com a tecla.
    if ultima {
        fora.push(Line::from(vec![
            Span::raw(recuo.clone()),
            Span::styled(" o ", tema.estilo(Realce::Marca)),
            Span::styled("+ linha", Style::default().fg(tema.var("text-muted"))),
        ]));
    }
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
fn linhas_do_mes(l: &crate::tela::Linha, tema: &Tema, largura: usize, sem_rotulo: bool) -> Vec<Line<'static>> {
    let recuo = "  ".repeat(l.nivel);
    let cel = largura_do_dia(largura.saturating_sub(recuo.len()));
    let dias: Vec<String> = anotadinho_core::calendario::WEEKDAY_LABELS
        .iter()
        .map(|d| format!("{d:^cel$}"))
        .collect();
    let mut fora = Vec::new();
    if !sem_rotulo {
        fora.push(Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(l.texto.clone(), tema.estilo(Realce::TituloCartao)),
        ]));
    }
    fora.extend([
        Line::from(vec![
            Span::styled(recuo.clone(), Style::default()),
            Span::styled(format!(" {} ", dias.join(" ")), tema.estilo(Realce::CabecalhoDeTabela)),
        ]),
        regua_do_mes(&recuo, cel, ["┌", "┬", "┐"], tema),
    ]);
    fora
}

/// O cabeçalho do calendário ancorado (ciclo 315): as teclas que fazem
/// o papel dos controles da janela (‹ › e "Hoje") e a contagem de
/// eventos, à direita como o `.calendar-grid__count`.
fn linha_do_cabecalho_do_calendario(
    l: &crate::tela::Linha,
    visao: anotadinho_core::analise::Visao,
    periodo: Option<&str>,
    vault: bool,
    tema: &Tema,
    largura: usize,
) -> Line<'static> {
    // Como `.calendar-grid__header` (ciclo 336): ‹ o período em negrito ›,
    // Hoje, a visão e a fonte; à direita a contagem e o "+ evento". A
    // tecla de cada controle vem apagada na frente dele.
    let tecla = tema.estilo(Realce::Marca);
    let apagado = Style::default().fg(tema.var("text-muted"));
    let caixa = Style::default().bg(tema.var("bg-elevated")).fg(tema.var("text-primary"));
    let mut esquerda = Faixa::default().mais("  ".repeat(l.nivel), Style::default()).mais("[ ‹  ", tecla);
    if let Some(p) = periodo {
        esquerda = esquerda.mais(p.to_string(), Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD));
    }
    esquerda = esquerda
        .mais("  ] ›", tecla)
        .mais("   t ", tecla)
        .mais("Hoje", apagado)
        .mais("   m ", tecla)
        .mais(format!(" {} ", visao.rotulo()), caixa)
        .mais("  ", Style::default())
        .mais(format!(" {} ", if vault { "Vault" } else { "Manual" }), caixa);
    let mut direita = Faixa::default().mais(l.texto.clone(), apagado);
    if !vault {
        direita = direita.mais("   o ", tecla).mais("+ evento", apagado);
    }
    let sobra = largura.saturating_sub(esquerda.largura + direita.largura).max(2);
    Line::from(esquerda.mais(" ".repeat(sobra), Style::default()).juntar(direita).spans)
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
fn linhas_do_eixo(l: &crate::tela::Linha, tema: &Tema, largura: usize, hoje: Option<i64>) -> Vec<Line<'static>> {
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
    if dias <= 91 {
        // Uma marca por semana (por dia na escala Semana), `dd/mm` — as
        // marcas do `.timeline__axis` da janela.
        let passo = if dias <= 7 { 1 } else { 7 };
        let mut k = 0;
        while k < dias {
            if let Some(d) = add_days(inicio, k) {
                if let Some((_, m, dd)) = parse_date(&d) {
                    marcas.push((d.clone(), format!("{dd:02}/{m:02}")));
                }
            }
            k += passo;
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
    let mut linha_da_regua = vec![Span::styled(recuo.clone(), Style::default())];
    match hoje.map(|h| ((disponivel as f64 * h as f64 / dias as f64).round() as usize).min(disponivel - 1)) {
        // Hoje cruza a régua na cor de destaque, como `.timeline__today`.
        Some(c) => {
            let texto: String = regua.iter().collect();
            let antes: String = texto.chars().take(c).collect();
            let depois: String = texto.chars().skip(c + 1).collect();
            linha_da_regua.push(Span::styled(antes, tema.estilo(Realce::Grade)));
            linha_da_regua.push(Span::styled("┼", Style::default().fg(tema.var("accent-blue"))));
            linha_da_regua.push(Span::styled(depois, tema.estilo(Realce::Grade)));
        }
        None => linha_da_regua.push(Span::styled(regua.into_iter().collect::<String>(), tema.estilo(Realce::Grade))),
    }
    vec![
        Line::from(vec![
            Span::styled(recuo, Style::default()),
            Span::styled(rotulos.into_iter().collect::<String>(), marca),
        ]),
        Line::from(linha_da_regua),
    ]
}

/// Quantos dias depois do início da janela é hoje, se hoje cai nela.
fn desloc_de_hoje(arvore: &Unidade, embed: &[usize]) -> Option<i64> {
    arvore
        .em(embed)?
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "cabecalho"))
        .and_then(|c| valor_escondido(c, "hoje"))
        .and_then(|h| h.parse().ok())
}

/// A barra de cima do cronograma (ciclo 332), como `.timeline__bar`: a
/// janela de tempo com as setas, "Hoje", o seletor de escala com a ativa
/// cheia, e a fonte (Manual/Vault) — com a tecla de cada coisa na frente,
/// como o cabeçalho do calendário.
fn linha_do_cabecalho_do_cronograma(l: &crate::tela::Linha, arvore: &Unidade, tema: &Tema, largura: usize) -> Line<'static> {
    let recuo = "  ".repeat(l.nivel);
    let u = arvore.em(&l.caminho);
    let escala = u.and_then(|u| valor_escondido(u, "escala")).unwrap_or("month").to_string();
    let fonte = u.and_then(|u| valor_escondido(u, "fonte")).unwrap_or("Manual").to_string();
    let apagado = Style::default().fg(tema.var("text-muted"));
    let tecla = tema.estilo(Realce::Marca);
    let mut esquerda = Faixa::default()
        .mais(recuo, Style::default())
        .mais("[ ‹  ", tecla)
        .mais(l.texto.clone(), apagado)
        .mais("  ] ›", tecla)
        .mais("   t ", tecla)
        .mais("Hoje", apagado);
    let mut direita = Faixa::default().mais("m ", tecla);
    for (slug, rotulo) in [("week", "Semana"), ("month", "Mês"), ("quarter", "Trimestre")] {
        let estilo = if slug == escala {
            Style::default().bg(tema.var("accent-blue")).fg(tema.var("bg-base"))
        } else {
            Style::default().bg(tema.var("bg-elevated")).fg(tema.var("text-muted"))
        };
        direita = direita.mais(format!(" {rotulo} "), estilo);
    }
    direita = direita.mais("   ~ ", tecla).mais(fonte.clone(), apagado);
    if fonte == "Manual" {
        direita = direita.mais("   o ", tecla).mais("+ etapa", apagado);
    }
    let sobra = largura.saturating_sub(esquerda.largura + direita.largura).max(2);
    esquerda = esquerda.mais(" ".repeat(sobra), Style::default()).juntar(direita);
    Line::from(esquerda.spans)
}

/// Uma barra do cronograma (ciclo 332): uma linha só, como
/// `.timeline__bar-item` — a pílula na cor do badge da primeira tag,
/// posicionada e dimensionada pela janela de tempo, com o título cortado
/// por dentro. Hoje atravessa a linha na cor de destaque onde a barra
/// não está. Sob o cursor, a pílula fica cheia na cor de destaque.
fn linha_da_barra(
    l: &crate::tela::Linha,
    arvore: &Unidade,
    tema: &Tema,
    largura: usize,
    cursor: Option<&[usize]>,
    hoje: Option<i64>,
) -> Line<'static> {
    let recuo = "  ".repeat(l.nivel);
    let disponivel = largura.saturating_sub(recuo.len()).max(4);
    let u = arvore.em(&l.caminho);
    let pct = |campo: &str| -> f64 { u.and_then(|u| valor_escondido(u, campo)).and_then(|v| v.parse().ok()).unwrap_or(0.0) };
    let inicio = ((disponivel as f64 * pct("inicio") / 100.0).round() as usize).min(disponivel - 1);
    let tamanho = ((disponivel as f64 * pct("duracao") / 100.0).round() as usize).max(3).min(disponivel - inicio);
    let dias: i64 = u
        .and_then(|_| {
            let dono = l.dono_embed.as_deref()?;
            let cab = arvore.em(dono)?.filhos.iter().find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "cabecalho"))?;
            valor_escondido(cab, "dias")?.parse().ok()
        })
        .unwrap_or(0);
    let coluna_hoje = hoje.filter(|_| dias > 0).map(|h| ((disponivel as f64 * h as f64 / dias as f64).round() as usize).min(disponivel - 1));
    let aceso = cursor.is_some_and(|c| l.mostra(c));
    let estilo_barra = if aceso {
        tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD)
    } else {
        let papel = u
            .and_then(|u| valor_escondido(u, "cor"))
            .and_then(|c| papel_do_badge(c.strip_prefix("badge").unwrap_or("")))
            .unwrap_or(Realce::BadgeInfo);
        tema.pilula(papel)
    };
    let hoje_estilo = Style::default().fg(crate::tema::misturar(tema.var("accent-blue"), tema.var("bg-surface"), 0.6));
    let trilho = |de: usize, ate: usize| -> Vec<Span<'static>> {
        let mut v = Vec::new();
        match coluna_hoje.filter(|c| *c >= de && *c < ate) {
            Some(c) => {
                v.push(Span::raw(" ".repeat(c - de)));
                v.push(Span::styled("│", hoje_estilo));
                v.push(Span::raw(" ".repeat(ate - c - 1)));
            }
            None => v.push(Span::raw(" ".repeat(ate - de))),
        }
        v
    };
    let mut spans = vec![Span::raw(recuo)];
    spans.extend(trilho(0, inicio));
    let texto = cortado(&format!(" {}", l.texto), tamanho);
    spans.push(Span::styled(na_largura(&texto, tamanho), estilo_barra));
    spans.extend(trilho(inicio + tamanho, disponivel));
    Line::from(spans)
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
    if tecla_do_cronograma(e, tecla) {
        return true;
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

/// A navegação no TEMPO do cronograma (ciclo 332), com as mesmas teclas
/// do calendário: `[`/`]` andam uma janela inteira, `t` volta pra hoje e
/// `m` troca a escala (Semana → Mês → Trimestre) — que, como na janela,
/// fica gravada no arquivo.
fn tecla_do_cronograma(e: &mut Estado, tecla: &str) -> bool {
    use anotadinho_core::embed::TimelineScale;
    let Some(hoje) = e.hoje.clone() else { return false };
    let Some(embed) = (1..=e.cursor.len())
        .map(|n| e.cursor[..n].to_vec())
        .find(|c| matches!(e.arvore.em(c).map(|u| &u.tipo), Some(Tipo::Embed(n)) if n == "timeline"))
    else {
        return false;
    };
    let cab = e
        .arvore
        .em(&embed)
        .and_then(|u| u.filhos.iter().find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "cabecalho")));
    let dias: i64 = cab.and_then(|c| valor_escondido(c, "dias")).and_then(|d| d.parse().ok()).unwrap_or(35);
    let inicio = e
        .arvore
        .em(&embed)
        .and_then(|u| u.filhos.iter().find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "eixo")))
        .and_then(|x| x.texto.split_once(' ').map(|(a, _)| a.to_string()));
    match tecla {
        "[" | "]" => {
            let delta = if tecla == "]" { dias } else { -dias };
            if let Some(novo) = inicio.and_then(|i| anotadinho_core::date_util::add_days(&i, delta)) {
                e.ancorar(&embed, novo);
            }
        }
        "t" => {
            e.ancoras.remove(&embed);
            e.ancorar_calendarios();
        }
        _ => {
            let atual = cab.and_then(|c| valor_escondido(c, "escala")).unwrap_or("month").to_string();
            let seguinte = match atual.as_str() {
                "week" => TimelineScale::Month,
                "month" => TimelineScale::Quarter,
                _ => TimelineScale::Week,
            };
            edicao::trocar_escala(e, &embed, seguinte);
            let _ = hoje;
        }
    }
    if e.arvore.em(&e.cursor).is_none() {
        e.cursor = embed;
    }
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
/// As grades que o desenho põe lado a lado e o modelo guarda de outro
/// jeito (ciclos 325 e 326):
///
/// - Num painel de colunas, `h`/`l` vão pro painel vizinho — no modelo
///   os painéis são irmãos em coluna, mas na tela estão lado a lado.
/// - Numa miniatura da galeria, `j`/`k` vão pra miniatura de baixo/cima
///   na MESMA coluna da grade, em vez de sair da fileira.
fn andar_na_grade(e: &mut Estado, mov: Movimento, vezes: u32) -> bool {
    let Some((&ultimo, pai)) = e.cursor.split_last() else { return false };
    let Some(pai_u) = e.arvore.em(pai) else { return false };
    let lateral = matches!(mov, Movimento::Esquerda | Movimento::Direita);
    let vertical = matches!(mov, Movimento::Cima | Movimento::Baixo);
    let adiante = matches!(mov, Movimento::Direita | Movimento::Baixo);
    let passo = |i: usize, total: usize| -> usize {
        if adiante {
            (i + vezes.max(1) as usize).min(total.saturating_sub(1))
        } else {
            i.saturating_sub(vezes.max(1) as usize)
        }
    };
    // O kanban também põe as colunas lado a lado (ciclo 328): na coluna,
    // `h`/`l` vão pra vizinha; num cartão, pro cartão da mesma altura da
    // vizinha (ou pra ela mesma, se estiver vazia).
    if lateral && matches!(&pai_u.tipo, Tipo::Parte { nome, .. } if nome == "column") {
        let Some((&coluna, kanban)) = pai.split_last() else { return false };
        let Some(k) = e.arvore.em(kanban) else { return false };
        let destino = passo(coluna, k.filhos.len());
        if destino == coluna {
            return false;
        }
        let cartoes = k.filhos[destino].filhos.len();
        e.cursor = if cartoes == 0 {
            [kanban, &[destino]].concat()
        } else {
            [kanban, &[destino, ultimo.min(cartoes - 1)]].concat()
        };
        return true;
    }
    if lateral && matches!(&pai_u.tipo, Tipo::Embed(n) if n == "columns" || n == "kanban") {
        let destino = passo(ultimo, pai_u.filhos.len());
        e.cursor = [pai, &[destino]].concat();
        return true;
    }
    let e_foto = matches!(&pai_u.tipo, Tipo::Parte { nome, .. } if nome == "fotos");
    if vertical && e_foto {
        let Some((&fileira, galeria)) = pai.split_last() else { return false };
        let Some(g) = e.arvore.em(galeria) else { return false };
        let fotos: Vec<usize> = g
            .filhos
            .iter()
            .enumerate()
            .filter(|(_, f)| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "fotos"))
            .map(|(i, _)| i)
            .collect();
        let Some(k) = fotos.iter().position(|i| *i == fileira) else { return false };
        let alvo = fotos[passo(k, fotos.len())];
        // Na primeira ou na última linha da grade, sai dela como sempre.
        if alvo == fileira {
            return false;
        }
        let na_linha = g.filhos[alvo].filhos.len();
        e.cursor = [galeria, &[alvo, ultimo.min(na_linha.saturating_sub(1))]].concat();
        return true;
    }
    false
}

fn comando_de_vim(e: &mut Estado, c: Comando) {
    let Comando::Mover(mov, vezes) = c else { return };
    if andar_na_grade(e, mov, vezes) {
        e.seguir_cursor();
        return;
    }
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
    largura: usize,
) -> Line<'a> {
    // A seta de recolher à direita, como o `chevron-down` da janela
    // (ciclo 336) — `z` dobra.
    let esquerda = 2 * l.nivel + 2 + l.texto.chars().count();
    let sobra = largura.saturating_sub(esquerda + 4).max(1);
    Line::from(vec![
        Span::styled("  ".repeat(l.nivel), Style::default()),
        Span::styled(icone_do_callout(variante), tema.estilo(papel)),
        Span::styled(" ", Style::default()),
        Span::styled(l.texto.clone(), tema.estilo(Realce::TituloCartao)),
        Span::raw(" ".repeat(sobra)),
        Span::styled("z ", tema.estilo(Realce::Marca)),
        Span::styled("⌄", Style::default().fg(tema.var("text-muted"))),
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
        Marca::Cor(nome) => e.fg(tema.var(&format!("cor-{nome}"))),
        Marca::Fundo(nome) => e.bg(tema.var(&format!("fundo-{nome}"))),
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
        // muda é qual segmento acende: o contorno dele vai pra cor de
        // destaque (o `:focus-visible` da janela, ciclo 329).
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 0, 1]; // o segundo botão
        let destaque = e.tema.var("accent-blue");
        let buf = quadro(&mut e, 60, 12);
        let y = (0..buf.area.height)
            .find(|y| (0..buf.area.width).map(|x| buf[(x, *y)].symbol().to_string()).collect::<String>().contains("Buscar"))
            .expect("o botão sumiu");
        let linha: Vec<String> = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
        let inicio = linha.join("").find("Buscar").map(|b| linha.join("")[..b].chars().count()).unwrap();
        let fim_abrir = linha.join("").find("Abrir").map(|b| linha.join("")[..b].chars().count()).unwrap();
        // Só o contorno de meio-bloco dos botões, não a moldura da região.
        let acesos: Vec<usize> = (0..buf.area.width)
            .filter(|x| buf[(*x, y)].style().fg == Some(destaque) && linha[*x as usize] == "▐" || linha[*x as usize] == "▌" && buf[(*x, y)].style().fg == Some(destaque))
            .map(|x| x as usize)
            .filter(|x| *x > fim_abrir && *x < inicio + 12)
            .collect();
        assert_eq!(acesos.len(), 2, "o contorno do botão do cursor não acendeu: {acesos:?}");
        assert!(acesos[0] < inicio && acesos[1] > inicio, "acendeu fora do Buscar: {acesos:?}");
        assert!(acesos[0] > fim_abrir, "acendeu o botão errado: {acesos:?}");
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
        // A trilha é de pílulas como `.fluxo__passo` (ciclo 330): feita,
        // atual e futura, cada uma de um jeito.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n"),
        );
        e.foco = Foco::Paginas;
        let buf = quadro(&mut e, 130, 14);
        let fundo_de = |palavra: &str| -> Option<Color> {
            (0..buf.area.height).find_map(|y| {
                let linha: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
                if !linha.contains("Rascunho") || !linha.contains("Concluída") {
                    return None;
                }
                let x = linha.chars().collect::<Vec<_>>().windows(palavra.chars().count()).position(|w| w.iter().collect::<String>() == palavra)?;
                buf[(x as u16, y)].style().bg
            })
        };
        let (feita, atual, futura) = (fundo_de("Rascunho"), fundo_de("Aprovada"), fundo_de("Concluída"));
        assert!(feita.is_some() && atual.is_some() && futura.is_some(), "{feita:?} {atual:?} {futura:?}\n{}", desenho(&mut e, 130, 14).join("\n"));
        assert_ne!(atual, feita, "a atual saiu igual à feita");
        assert_ne!(atual, futura, "a atual saiu igual à futura");
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
    fn a_galeria_e_uma_grade_de_quadros_com_legenda() {
        // A barra e a grade da janela (ciclo 325): contagem, tamanho e
        // colunas em cima; as miniaturas LADO A LADO, cada uma um quadro
        // com o nome do arquivo no meio e a legenda embaixo.
        let mut e = Estado::novo(paginas(), com_galeria());
        e.foco = Foco::Paginas;
        let tela = desenho(&mut e, 90, 16);
        let tudo = tela.join("\n");
        assert!(tela.iter().any(|l| l.contains("2 imagens") && l.contains(" P  M  G ") && l.contains("3 col")), "{tudo}");
        assert!(tela.iter().any(|l| l.contains("▖  ▗")), "os quadros não ficaram lado a lado:\n{tudo}");
        assert!(tela.iter().any(|l| l.contains("a.png") && l.contains("b.png")), "{tudo}");
        assert!(tela.iter().any(|l| l.contains("Primeira") && l.contains("Segunda")), "{tudo}");
    }

    #[test]
    fn na_galeria_j_e_k_andam_na_mesma_coluna_da_grade() {
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"gallery\" }}\ncolumns: 2\nitems:\n- path: a.png\n- path: b.png\n- path: c.png\n{{ /gallery }}\n\nfim\n"),
        );
        e.foco = Foco::Conteudo;
        // embed → cabeçalho, fileira [a, b], fileira [c].
        e.cursor = vec![0, 1, 1];
        tecla(&mut e, "j");
        assert_eq!(e.cursor, vec![0, 2, 0], "a linha de baixo só tem uma");
        tecla(&mut e, "k");
        assert_eq!(e.cursor, vec![0, 1, 0]);
        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![0, 1, 1]);
    }

    #[test]
    fn as_colunas_ficam_lado_a_lado_na_proporcao_e_h_l_trocam_de_painel() {
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"columns\" }}\ncolumns:\n- width: 2\n  body: |\n    Esquerda larga com um texto comprido que precisa quebrar.\n- width: 1\n  body: Direita.\n{{ /columns }}\n"),
        );
        e.foco = Foco::Paginas;
        let tela = desenho(&mut e, 100, 16);
        let tudo = tela.join("\n");
        let lado_a_lado = tela.iter().find(|l| l.contains("Esquerda") && l.contains("Direita.")).expect(&tudo);
        let esq = lado_a_lado.find("2fr").or_else(|| lado_a_lado.find("Esquerda")).unwrap();
        assert!(esq > 0);
        let proporcao = tela.iter().find(|l| l.contains("2fr") && l.contains("1fr")).expect(&tudo);
        let (a, b) = (proporcao.chars().position(|c| c == '2').unwrap(), proporcao.chars().position(|c| c == '1').unwrap());
        assert!(b > a, "{tudo}");
        assert!(!tudo.contains("pane"), "o nome da parte vazou:\n{tudo}");
        // O texto longo quebra dentro do painel em vez de invadir o outro.
        assert!(tela.iter().any(|l| l.contains("quebrar")), "{tudo}");
        e.foco = Foco::Conteudo;
        e.cursor = vec![0, 0];
        tecla(&mut e, "l");
        assert_eq!(e.cursor, vec![0, 1]);
        tecla(&mut e, "h");
        assert_eq!(e.cursor, vec![0, 0]);
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
        // (mais a gaveta "Sem data", ciclo 368)
        assert_eq!(e.arvore.filhos[0].filhos.len(), 4);
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
        assert!(tudo.contains("3 eventos") && tudo.contains("t Hoje"), "sem cabeçalho:\n{tudo}");
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
        assert!(tudo.contains("m  Semana "), "{tudo}");
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
        assert!(tudo.contains("m  Dia "), "{tudo}");
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
    fn escape_confirma_como_no_vim_e_u_desfaz() {
        // No vim, `Esc` sai da inserção COM o que foi digitado (ciclo
        // 333). Desistir é `u`.
        let mut e = editavel();
        e.cursor = vec![1, 1, 3, 4];
        tecla(&mut e, "o");
        digitar(&mut e, "Nada");
        tecla(&mut e, "Escape");
        assert!(e.pergunta.is_none());
        assert_eq!(gravado(&e).entries.len(), 3);
        // E o `j` volta a ser movimento, não texto.
        tecla(&mut e, "j");
        assert!(e.pergunta.is_none());
        tecla(&mut e, "u");
        assert_eq!(e.gravacao.as_deref(), Some(PAGINA_COM_CALENDARIO));
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
        // Na coluna "Feito" (vazia), `Enter` abre o primeiro cartão — o
        // "+ card" da janela (ciclo 337; `o` na coluna cria coluna).
        e.cursor = vec![1, 2];
        tecla(&mut e, "Enter");
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
        tecla(&mut e, "Escape");
        assert_eq!(callout_gravado(&e).body, "Primeiro parágrafo.\n\nSegundo.\n\n- um\n- dois\n\nÚltimo.\n");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Segundo.");
        // `cc` num item mantém a marca (ela fica fora do texto); `o` num
        // item cria item vizinho.
        e.cursor = no_callout(&e, "um");
        tecla(&mut e, "c");
        tecla(&mut e, "c");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "");
        digitar(&mut e, "primeiro");
        tecla(&mut e, "Escape");
        tecla(&mut e, "o");
        digitar(&mut e, "meio");
        tecla(&mut e, "Escape");
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

    const PAGINA_MARKDOWN: &str = "---\ntitle: Notas\n---\n# Título\n\nUm parágrafo com **negrito**.\n\n- [ ] tarefa um\n- [ ] tarefa dois\n\n> citação\n";

    fn markdown_editavel() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(PAGINA_MARKDOWN, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    /// O corpo gravado, conferindo que o frontmatter ficou.
    fn corpo_gravado(e: &Estado) -> String {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        assert!(texto.starts_with("---\ntitle: Notas\n---\n"), "o frontmatter mudou:\n{texto}");
        texto["---\ntitle: Notas\n---\n".len()..].to_string()
    }

    #[test]
    fn a_e_i_editam_o_texto_do_paragrafo_com_as_marcas_a_vista() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "A");
        let p = e.pergunta.as_ref().unwrap();
        assert_eq!((p.texto.as_str(), p.cursor), ("Um parágrafo com **negrito**.", 29));
        // O texto aparece NO LUGAR, e o rodapé só diz o modo.
        let tela = desenho(&mut e, 90, 16).join("\n");
        assert!(tela.contains("Um parágrafo com **negrito**."), "{tela}");
        assert!(tela.contains("-- INSERÇÃO --") && !tela.contains("INSERÇÃO --: "), "{tela}");
        digitar(&mut e, " Fim.");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("\n\nUm parágrafo com **negrito**. Fim.\n\n"));
        tecla(&mut e, "I");
        assert_eq!(e.pergunta.as_ref().unwrap().cursor, 0);
        digitar(&mut e, "Ah. ");
        tecla(&mut e, "ArrowRight");
        tecla(&mut e, "Backspace");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("\n\nAh. m parágrafo"), "{}", corpo_gravado(&e));
    }

    #[test]
    fn enter_confirma_e_abre_o_bloco_seguinte() {
        let mut e = markdown_editavel();
        e.cursor = vec![2, 1];
        tecla(&mut e, "o");
        digitar(&mut e, "tarefa três");
        tecla(&mut e, "Enter");
        // Item depois de item, com a caixa vazia.
        assert!(e.pergunta.is_some());
        digitar(&mut e, "tarefa quatro");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("- [ ] tarefa dois\n- [ ] tarefa três\n- [ ] tarefa quatro\n\n> citação"), "{}", corpo_gravado(&e));
        // Num parágrafo, o seguinte é parágrafo.
        e.cursor = vec![1];
        tecla(&mut e, "O");
        digitar(&mut e, "Antes do parágrafo.");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).starts_with("# Título\n\nAntes do parágrafo.\n\nUm parágrafo"), "{}", corpo_gravado(&e));
    }

    #[test]
    fn til_marca_a_caixa_e_ctrl_a_muda_o_nivel_do_titulo() {
        let mut e = markdown_editavel();
        e.cursor = vec![2, 0];
        tecla(&mut e, "~");
        assert!(corpo_gravado(&e).contains("- [x] tarefa um\n- [ ] tarefa dois"));
        tecla(&mut e, "~");
        assert!(corpo_gravado(&e).contains("- [ ] tarefa um\n"));
        e.cursor = vec![0];
        tecla(&mut e, "2");
        tecla(&mut e, "Ctrl+x");
        assert!(corpo_gravado(&e).starts_with("### Título\n"));
        tecla(&mut e, "Ctrl+a");
        assert!(corpo_gravado(&e).starts_with("## Título\n"));
    }

    #[test]
    fn maior_maior_troca_o_bloco_de_lugar_e_dd_p_move() {
        let mut e = markdown_editavel();
        e.cursor = vec![2, 0];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert!(corpo_gravado(&e).contains("- [ ] tarefa dois\n- [ ] tarefa um\n\n> citação"));
        assert!(e.arvore.em(&e.cursor).unwrap().texto.ends_with("tarefa um"));
        // O parágrafo desce pra depois da lista.
        e.cursor = vec![1];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert!(corpo_gravado(&e).starts_with("# Título\n\n- [ ] tarefa dois\n- [ ] tarefa um\n\nUm parágrafo"), "{}", corpo_gravado(&e));
        // `dd` e `P` no começo: o título vai pro fim do bloco de cima.
        e.cursor = vec![3];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        e.cursor = vec![0];
        tecla(&mut e, "P");
        assert!(corpo_gravado(&e).starts_with("> citação\n\n# Título"), "{}", corpo_gravado(&e));
        assert!(e.aviso.is_none() || !e.aviso.as_deref().unwrap().contains("não"));
    }

    #[test]
    fn x_no_markdown_avisa_em_vez_de_apagar() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "x");
        assert!(e.gravacao.is_none());
        assert!(e.aviso.as_deref().unwrap_or("").contains("dd"));
    }

    #[test]
    fn no_proprio_embed_o_cria_texto_depois_e_dd_apaga_o_embed() {
        let mut e = kanban_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "Depois do kanban.");
        tecla(&mut e, "Escape");
        let texto = e.gravacao.clone().unwrap();
        assert!(texto.contains("{{ /kanban }}\n\nDepois do kanban.\n\nDepois."), "{texto}");
        e.cursor = vec![1];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(e.gravacao.as_deref(), Some("Antes.\n\nDepois do kanban.\n\nDepois.\n"));
        tecla(&mut e, "u");
        assert!(e.gravacao.as_deref().unwrap().contains("{{ type: \"kanban\" }}"));
    }

    #[test]
    fn no_painel_de_colunas_o_markdown_e_o_do_painel() {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(
            "{{ type: \"columns\" }}\ncolumns:\n- width: 1\n  body: |\n    Esquerda.\n- width: 1\n  body: |\n    Direita.\n{{ /columns }}\n",
            Some("v1".into()),
        );
        e.foco = Foco::Conteudo;
        // embed → painel 1 → largura (escondida), parágrafo.
        e.cursor = vec![0, 1, 1];
        tecla(&mut e, "A");
        // Painel é desenhado inteiro: o texto vai pro rodapé.
        let tela = desenho(&mut e, 100, 16).join("\n");
        assert!(tela.contains("-- INSERÇÃO --: Direita."), "{tela}");
        digitar(&mut e, " Mais.");
        tecla(&mut e, "Escape");
        let texto = e.gravacao.clone().unwrap();
        assert!(texto.contains("Direita. Mais."), "{texto}");
        assert!(texto.contains("Esquerda."), "{texto}");
    }

    /// Os dados do primeiro embed no texto gravado.
    fn embed_gravado(e: &Estado) -> anotadinho_core::embed::EmbedData {
        let texto = e.gravacao.clone().expect("nada pra gravar");
        anotadinho_core::embed::segment(&texto)
            .into_iter()
            .find_map(|s| match s {
                anotadinho_core::embed::DocSegment::Embed(d) => Some(d),
                _ => None,
            })
            .expect("o embed sumiu")
    }

    fn pagina_com(fonte: &str) -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(fonte, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e
    }

    #[test]
    fn na_galeria_legenda_imagem_nova_ordem_colunas_e_tamanho() {
        use anotadinho_core::embed::{EmbedData, GallerySize};
        let mut e = pagina_com("{{ type: \"gallery\" }}\nitems:\n- path: assets/a.png\n  caption: A\n- path: assets/b.png\n{{ /gallery }}\n");
        // embed → cabeçalho, fileira [a, b].
        e.cursor = vec![0, 1, 1];
        tecla(&mut e, "a");
        digitar(&mut e, "Bê");
        tecla(&mut e, "Escape");
        let EmbedData::Gallery(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.items[1].caption, "Bê");
        tecla(&mut e, "O");
        // A lista de `assets/` (ciclo 356): só imagens, e digitar o caminho.
        assert_eq!(e.pedidos, [Pedido::AssetsParaInserir]);
        e.pedidos.clear();
        markdown::escolher_asset(&mut e, vec!["assets/c.png".into(), "assets/doc.pdf".into()]);
        let tela = desenho(&mut e, 100, 30).join("\n");
        assert!(tela.contains("Imagem do vault") && tela.contains("Digitar o caminho") && !tela.contains("doc.pdf"), "{tela}");
        digitar(&mut e, "c.png");
        tecla(&mut e, "Enter");
        let EmbedData::Gallery(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.items.iter().map(|i| i.path.as_str()).collect::<Vec<_>>(), ["assets/a.png", "assets/c.png", "assets/b.png"]);
        e.cursor = vec![0, 1, 0];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let EmbedData::Gallery(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.items[1].path, "assets/a.png");
        tecla(&mut e, "2");
        tecla(&mut e, "Ctrl+x");
        tecla(&mut e, "~");
        let EmbedData::Gallery(d) = embed_gravado(&e) else { panic!() };
        assert_eq!((d.columns, d.size), (1, GallerySize::Lg));
        // Com uma coluna a grade virou três fileiras de uma.
        e.cursor = vec![0, 2, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let EmbedData::Gallery(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.items.len(), 2);
    }

    #[test]
    fn nos_paineis_cria_alarga_reordena_e_escreve_no_vazio() {
        use anotadinho_core::embed::EmbedData;
        let mut e = pagina_com("{{ type: \"columns\" }}\ncolumns:\n- width: 1\n  body: |\n    Esquerda.\n- width: 1\n  body: ''\n{{ /columns }}\n");
        e.cursor = vec![0, 1];
        tecla(&mut e, "Ctrl+a");
        let EmbedData::Columns(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.columns[1].width, 2);
        // `a` num painel vazio começa o primeiro parágrafo dele.
        tecla(&mut e, "a");
        digitar(&mut e, "Direita.");
        tecla(&mut e, "Escape");
        let EmbedData::Columns(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.columns[1].body.trim(), "Direita.");
        e.cursor = vec![0, 0];
        tecla(&mut e, "o");
        let EmbedData::Columns(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.columns.len(), 3);
        assert_eq!(e.cursor, vec![0, 1]);
        tecla(&mut e, "<");
        tecla(&mut e, "<");
        let EmbedData::Columns(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.columns[1].body.trim(), "Esquerda.");
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let EmbedData::Columns(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.columns.len(), 2);
    }

    #[test]
    fn no_fluxo_enter_na_transicao_move_e_a_edita_a_nota() {
        use anotadinho_core::embed::EmbedData;
        use anotadinho_core::fluxo::Etapa;
        let mut e = pagina_com("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: em-revisao\n{{ /fluxo }}\n");
        let transicao = e
            .arvore
            .percorrer()
            .into_iter()
            .find(|(_, u)| u.texto == "Rascunho" && matches!(&u.tipo, Tipo::Parte { nome, .. } if nome.starts_with("transicao")))
            .map(|(c, _)| c)
            .unwrap();
        e.cursor = transicao;
        tecla(&mut e, "Enter");
        let EmbedData::Fluxo(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.etapa, Etapa::Rascunho);
        // Dentro do fluxo (no próprio embed, `>>` move o bloco da página).
        e.cursor = vec![0, 1];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let EmbedData::Fluxo(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.etapa, Etapa::EmRevisao);
        tecla(&mut e, "a");
        digitar(&mut e, "falta o teste");
        tecla(&mut e, "Escape");
        let EmbedData::Fluxo(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.nota.as_deref(), Some("falta o teste"));
    }

    #[test]
    fn na_consulta_a_edita_o_filtro_til_gira_a_visao_e_ctrl_a_o_limite() {
        use anotadinho_core::embed::EmbedData;
        use anotadinho_core::query::QueryView;
        let mut e = pagina_com("{{ type: \"query\" }}\nfrom: pages\nlimit: 5\n{{ /query }}\n");
        e.cursor = vec![0];
        tecla(&mut e, "A");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "from:pages limit:5");
        digitar(&mut e, " status=done");
        tecla(&mut e, "Escape");
        let EmbedData::Query(q) = embed_gravado(&e) else { panic!() };
        assert_eq!(q.conditions.len(), 1);
        assert_eq!(q.limit, Some(5));
        tecla(&mut e, "~");
        tecla(&mut e, "3");
        tecla(&mut e, "Ctrl+a");
        let EmbedData::Query(q) = embed_gravado(&e) else { panic!() };
        assert_eq!((q.view, q.limit), (QueryView::Table, Some(8)));
    }

    #[test]
    fn na_coluna_do_kanban_cria_move_duplica_e_apaga_coluna() {
        let mut e = kanban_editavel();
        // Backlog, Fazendo, Feito; cursor em Fazendo.
        e.cursor = vec![1, 1];
        tecla(&mut e, "o");
        digitar(&mut e, "Revisão");
        tecla(&mut e, "Escape");
        assert_eq!(kanban_gravado(&e).columns, ["Backlog", "Fazendo", "Revisão", "Feito"]);
        assert_eq!(e.cursor, vec![1, 2]);
        // Nome repetido é recusado.
        tecla(&mut e, "O");
        digitar(&mut e, "Feito");
        tecla(&mut e, "Escape");
        assert!(e.aviso.as_deref().unwrap_or("").contains("já existe"));
        e.cursor = vec![1, 1];
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        assert_eq!(kanban_gravado(&e).columns, ["Backlog", "Revisão", "Fazendo", "Feito"]);
        assert_eq!(e.cursor, vec![1, 2]);
        // yy + p duplica com os cartões, com nome novo.
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        tecla(&mut e, "p");
        let d = kanban_gravado(&e);
        assert_eq!(d.columns[3], "Fazendo (cópia)");
        assert_eq!(d.items.iter().filter(|c| c.column == "Fazendo (cópia)").count(), 1);
        // dd leva a coluna e os cartões.
        e.cursor = vec![1, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let d = kanban_gravado(&e);
        assert!(!d.columns.contains(&"Backlog".to_string()));
        assert!(d.items.iter().all(|c| c.title != "Escrever"));
        tecla(&mut e, "u");
        assert!(kanban_gravado(&e).columns.contains(&"Backlog".to_string()));
    }

    #[test]
    fn no_cabecalho_da_tabela_cria_move_duplica_e_troca_o_tipo() {
        use anotadinho_core::embed::ColumnKind;
        let mut e = tabela_editavel();
        // Tarefa, Status (select), Tags (multiselect); cursor em Tarefa.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "o");
        digitar(&mut e, "Prazo");
        tecla(&mut e, "Escape");
        let d = tabela_gravada(&e);
        assert_eq!(d.columns.iter().map(|c| c.name.as_str()).collect::<Vec<_>>(), ["Tarefa", "Prazo", "Status", "Tags"]);
        assert_eq!(d.rows[0], ["API", "", "done", "api"]);
        assert_eq!(e.cursor, vec![1, 0, 1]);
        // `~` gira o tipo: texto → número → data.
        tecla(&mut e, "~");
        tecla(&mut e, "~");
        assert_eq!(tabela_gravada(&e).columns[1].kind, ColumnKind::Date);
        // Tarefa vira seleção com os valores que já tinha como opções.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "4");
        tecla(&mut e, "Ctrl+a");
        let d = tabela_gravada(&e);
        assert_eq!(d.columns[0].kind, ColumnKind::Select { options: vec!["API".into(), "Docs".into()] });
        // >> leva a coluna com as células.
        tecla(&mut e, "2");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
        let d = tabela_gravada(&e);
        assert_eq!(d.columns[2].name, "Tarefa");
        assert_eq!(d.rows[1][2], "Docs");
        // yy + P duplica antes.
        tecla(&mut e, "y");
        tecla(&mut e, "y");
        tecla(&mut e, "P");
        let d = tabela_gravada(&e);
        assert_eq!((d.columns[2].name.as_str(), d.columns[3].name.as_str()), ("Tarefa", "Tarefa"));
        assert_eq!(d.rows[0].len(), 5);
    }

    #[test]
    fn j_e_k_reordenam_o_cartao_na_coluna_e_a_linha_na_tabela() {
        let mut e = pagina_com("{{ type: \"kanban\" }}\ncolumns:\n- A\n- B\nitems:\n- title: um\n  column: A\n- title: x\n  column: B\n- title: dois\n  column: A\n- title: três\n  column: A\n{{ /kanban }}\n");
        e.cursor = vec![0, 0, 0];
        tecla(&mut e, "J");
        let titulos = |e: &Estado| -> Vec<String> {
            let anotadinho_core::embed::EmbedData::Kanban(d) = embed_gravado(e) else { panic!() };
            d.items.iter().filter(|c| c.column == "A").map(|c| c.title.clone()).collect()
        };
        assert_eq!(titulos(&e), ["dois", "um", "três"]);
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "um");
        tecla(&mut e, "J");
        assert_eq!(titulos(&e), ["dois", "três", "um"]);
        tecla(&mut e, "2");
        tecla(&mut e, "K");
        assert_eq!(titulos(&e), ["um", "dois", "três"]);
        assert_eq!(e.cursor, vec![0, 0, 0]);

        let mut e = tabela_editavel();
        e.cursor = vec![1, 1, 1];
        tecla(&mut e, "J");
        let d = tabela_gravada(&e);
        assert_eq!((d.rows[0][0].as_str(), d.rows[1][0].as_str()), ("Docs", "API"));
        assert_eq!(e.cursor, vec![1, 2, 1]);
    }

    #[test]
    fn paragrafo_longo_quebra_na_largura_em_vez_de_cortar() {
        let texto = "palavra ".repeat(30);
        let mut e = Estado::novo(paginas(), analisar(&format!("{}\nsegunda linha do arquivo\n\n- item {}\n", texto.trim(), texto.trim())));
        e.foco = Foco::Paginas;
        let tela = desenho(&mut e, 80, 30);
        let tudo = tela.join("\n");
        assert!(tela.iter().filter(|l| l.contains("palavra")).count() >= 6, "{tudo}");
        assert!(tudo.contains("segunda linha do arquivo"), "a quebra do parágrafo virou linha:\n{tudo}");
        // A continuação do item alinha depois da marca.
        let item = tela.iter().position(|l| l.contains("- item")).unwrap();
        let col_item = tela[item].find("item").unwrap();
        let continua = &tela[item + 1];
        assert_eq!(continua.find("palavra"), Some(col_item), "\n{}\n{continua}", tela[item]);
    }

    #[test]
    fn a_busca_da_consulta_e_destino_do_cursor_e_enter_edita_o_filtro() {
        let mut e = pagina_com("{{ type: \"query\" }}\nfrom: pages\n{{ /query }}\n")
            .com_indice_do_vault(vec![anotadinho_core::index::PageIndexEntry { path: "pages/a.md".into(), title: "A".into(), ..Default::default() }]);
        e.cursor = vec![0];
        tecla(&mut e, "Enter");
        assert!(matches!(&e.arvore.em(&e.cursor).unwrap().tipo, Tipo::Parte { nome, .. } if nome == "busca"));
        tecla(&mut e, "Enter");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "from:pages");
        tecla(&mut e, "Escape");
        tecla(&mut e, "j");
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "A");
    }

    #[test]
    fn a_barra_de_comandos_filtra_executa_e_abre_pagina() {
        let mut e = Estado::novo(paginas(), analisar("texto"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, ":");
        assert!(matches!(e.modal, Some(Modal::Paleta(_))));
        let tela = desenho(&mut e, 120, 30).join("\n");
        assert!(tela.contains("Buscar página ou comando") && tela.contains("Nova conversa com o agente"), "{tela}");
        digitar(&mut e, "alternar tema");
        tecla(&mut e, "Enter");
        assert!(e.modal.is_none());
        assert_eq!(e.preferencias.tema, "papel");
        assert_eq!(e.pedidos, vec![Pedido::GravarPreferencias]);
        e.pedidos.clear();
        // Página pelo nome.
        tecla(&mut e, "Ctrl+k");
        digitar(&mut e, "beta");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::AbrirPagina("pages/beta.md".into())]);
        // Escape fecha sem fazer nada.
        e.pedidos.clear();
        tecla(&mut e, ":");
        tecla(&mut e, "Escape");
        assert!(e.modal.is_none() && e.pedidos.is_empty());
    }

    #[test]
    fn personalizar_troca_sidebar_e_nova_pagina_pede_titulo() {
        let mut e = Estado::novo(paginas(), analisar("texto"));
        tecla(&mut e, ":");
        digitar(&mut e, "personalizar");
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Escolha { .. })));
        for _ in 0..3 {
            tecla(&mut e, "j");
        }
        tecla(&mut e, "Enter");
        assert!(!e.preferencias.sidebar);
        assert_eq!(e.foco, Foco::Conteudo);
        let tela = desenho(&mut e, 100, 20);
        assert!(!tela[0].contains("páginas"), "a sidebar continuou: {}", tela[0]);
        tecla(&mut e, "Escape");
        tecla(&mut e, ":");
        digitar(&mut e, "nova página: kanban");
        tecla(&mut e, "Enter");
        digitar(&mut e, "Quadro");
        tecla(&mut e, "Enter");
        assert!(e.pedidos.contains(&Pedido::CriarPaginaComTitulo { titulo: "Quadro".into(), tipo: Some("kanban".into()) }));
        // Nova conversa usa o relógio e anexa a página aberta.
        e.agora = Some("2026-09-17 10:30".into());
        tecla(&mut e, ":");
        digitar(&mut e, "nova conversa");
        tecla(&mut e, "Enter");
        let Some(Pedido::CriarPagina { path, conteudo }) = e.pedidos.last() else { panic!("{:?}", e.pedidos) };
        assert!(path.starts_with("pages/conversas/"), "{path}");
        assert!(conteudo.contains("type: conversa") && conteudo.contains("pages/alfa.md"), "{conteudo}");
    }

    #[test]
    fn o_editor_de_opcoes_cria_renomeia_reordena_e_apaga() {
        use anotadinho_core::embed::ColumnKind;
        let mut e = tabela_editavel();
        // Status: select [todo, done]; API=done, Docs=todo.
        e.cursor = vec![1, 0, 1];
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Opcoes(_))));
        tecla(&mut e, "o");
        digitar(&mut e, "doing");
        tecla(&mut e, "Enter");
        let opcoes = |e: &Estado| match &tabela_gravada(e).columns[1].kind {
            ColumnKind::Select { options } => options.clone(),
            _ => panic!(),
        };
        assert_eq!(opcoes(&e), ["todo", "done", "doing"]);
        // Renomear "done" leva as células junto.
        tecla(&mut e, "k");
        tecla(&mut e, "a");
        for _ in 0..4 {
            tecla(&mut e, "Backspace");
        }
        digitar(&mut e, "feito");
        tecla(&mut e, "Escape");
        assert_eq!(opcoes(&e), ["todo", "feito", "doing"]);
        assert_eq!(tabela_gravada(&e).rows[0][1], "feito");
        tecla(&mut e, "K");
        assert_eq!(opcoes(&e), ["feito", "todo", "doing"]);
        // dd apaga a opção e limpa as células.
        tecla(&mut e, "j");
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert_eq!(opcoes(&e), ["feito", "doing"]);
        assert_eq!(tabela_gravada(&e).rows[1][1], "");
        tecla(&mut e, "Escape");
        assert!(e.modal.is_none());
        // Coluna de texto não abre o editor.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "Enter");
        assert!(e.modal.is_none());
    }

    const CONVERSA: &str = "---\ntitle: \"Planejar: imagens\"\ntype: conversa\ncontexto:\n- pages/specs/imagens.md\n---\n## você · 2026-08-24 19:41\n\nEscreva uma **proposta** com etapas.\n\n## agente · 2026-08-24 19:42\n\n# Proposta\n\n1. primeiro passo\n2. segundo passo\n";

    fn conversa_aberta() -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(CONVERSA, Some("v1".into()));
        e.foco = Foco::Conteudo;
        e.agora = Some("2026-09-17 10:00".into());
        e
    }

    #[test]
    fn pagina_de_conversa_abre_a_tela_de_conversa() {
        let mut e = conversa_aberta();
        let c = e.conversa.as_ref().expect("não virou conversa");
        assert_eq!(c.mensagens.len(), 2);
        assert_eq!(c.anexos, ["pages/specs/imagens.md"]);
        let tela = desenho(&mut e, 120, 40);
        let tudo = tela.join("\n");
        for esperado in ["Planejar: imagens", "1 anexo(s)", "imagens ×", "VOCÊ", "AGENTE", "Proposta", "primeiro passo", "Enviar"] {
            assert!(tudo.contains(esperado), "faltou {esperado}:\n{tudo}");
        }
        // As suas à direita, as do agente à esquerda.
        let col = |t: &str| tela.iter().find_map(|l| l.find(t)).unwrap();
        assert!(col("VOCÊ") > col("AGENTE"), "\n{tudo}");
        // Página comum não é conversa.
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto("# nota\n", None);
        assert!(e.conversa.is_none());
    }

    #[test]
    fn escrever_e_enviar_pede_a_execucao_do_agente() {
        let mut e = conversa_aberta();
        tecla(&mut e, "i");
        // Escrevendo, `q` e `:` são texto, não sair nem barra de comandos.
        digitar(&mut e, "q: e o teste?");
        tecla(&mut e, "Ctrl+j");
        digitar(&mut e, "linha 2");
        assert!(!e.sair && e.modal.is_none());
        tecla(&mut e, "Enter");
        assert_eq!(
            e.pedidos,
            vec![Pedido::EnviarNaConversa {
                path: "pages/alfa.md".into(),
                pergunta: "q: e o teste?\nlinha 2".into(),
                anexos: vec!["pages/specs/imagens.md".into()],
            }]
        );
        let c = e.conversa.as_ref().unwrap();
        assert!(!c.escrevendo && c.rascunho.texto.is_empty());
        // Com o agente rodando, aparece o progresso e Ctrl+X interrompe.
        e.pedidos.clear();
        e.conversa.as_mut().unwrap().trabalho = Some((12, "lendo arquivos".into()));
        let tudo = desenho(&mut e, 120, 40).join("\n");
        assert!(tudo.contains("pensando há 12s") && tudo.contains("lendo arquivos") && tudo.contains("Parar"), "{tudo}");
        tecla(&mut e, "Ctrl+x");
        assert_eq!(e.pedidos, vec![Pedido::InterromperAgente("pages/alfa.md".into())]);
    }

    #[test]
    fn enter_na_resposta_vira_spec_e_a_barra_anexa_pagina() {
        let mut e = conversa_aberta();
        tecla(&mut e, "k");
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Escolha { .. })));
        tecla(&mut e, "Enter");
        let Some(Pedido::CriarPagina { path, conteudo }) = e.pedidos.first() else { panic!("{:?}", e.pedidos) };
        assert!(path.ends_with(".md") && conteudo.contains("Proposta"), "{path}\n{conteudo}");
        e.pedidos.clear();
        // Execução passa por confirmação.
        tecla(&mut e, "Enter");
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Confirmar { .. })));
        tecla(&mut e, "y");
        assert!(matches!(e.pedidos.first(), Some(Pedido::ExecutarDaConversa { .. })));
        e.pedidos.clear();
        // Anexar pela barra de comandos.
        tecla(&mut e, ":");
        digitar(&mut e, "anexar");
        tecla(&mut e, "Enter");
        digitar(&mut e, "beta");
        tecla(&mut e, "Enter");
        assert_eq!(
            e.pedidos,
            vec![Pedido::AnexosDaConversa {
                conversa: "pages/alfa.md".into(),
                lista: vec!["pages/specs/imagens.md".into(), "pages/beta.md".into()],
            }]
        );
    }

    #[test]
    fn prompt_padrao_escolhe_preenche_bloqueia_e_devolve_o_rascunho() {
        let mut e = conversa_aberta().com_indice_do_vault(vec![anotadinho_core::index::PageIndexEntry {
            path: "pages/prompts-default/entender.md".into(),
            title: "Entender um trecho".into(),
            page_type: "prompt".into(),
            ..Default::default()
        }]);
        // Rascunho antes do prompt.
        tecla(&mut e, "i");
        digitar(&mut e, "o parser");
        tecla(&mut e, "Escape");
        tecla(&mut e, "p");
        let Some(Modal::Prompt(sel)) = &e.modal else { panic!("{:?}", e.modal) };
        assert_eq!(sel.lista.itens.len(), 2);
        let tela = desenho(&mut e, 120, 40).join("\n");
        assert!(tela.contains("Prompt padrão") && tela.contains("Nenhum — escrever do zero") && tela.contains("Entender um trecho"), "{tela}");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::CarregarPrompt("pages/prompts-default/entender.md".into())]);
        e.pedidos.clear();
        // O `main` lê a página e aplica.
        conversa::aplicar_prompt(&mut e, "pages/prompts-default/entender.md", "---\ntype: prompt\n---\nExplique {{alvo}} e {{nivel}}.\n");
        let c = e.conversa.as_ref().unwrap();
        // O rascunho vira o valor da primeira variável — blindado como
        // dado, como na janela.
        assert!(c.rascunho.texto.starts_with("Explique ") && c.rascunho.texto.contains("o parser"), "{}", c.rascunho.texto);
        assert!(c.rascunho.texto.ends_with(" e {{nivel}}."), "{}", c.rascunho.texto);
        let Some(Modal::Prompt(sel)) = &e.modal else { panic!("os campos deviam ficar abertos") };
        assert_eq!(sel.campos.len(), 2);
        assert_eq!(sel.foco, Some(1), "o foco vai pro que falta");
        // Enviar com marcador pendente é recusado.
        tecla(&mut e, "Escape");
        tecla(&mut e, "i");
        tecla(&mut e, "Enter");
        assert!(e.pedidos.is_empty());
        assert!(e.aviso.as_deref().unwrap_or("").contains("marcadores"));
        digitar(&mut e, "básico");
        tecla(&mut e, "Enter");
        let texto = e.conversa.as_ref().unwrap().rascunho.texto.clone();
        assert!(texto.contains("básico") && !texto.contains("{{nivel}}"), "{texto}");
        // Continua em inserção depois do seletor; sai pra usar `p`.
        assert!(e.conversa.as_ref().unwrap().escrevendo);
        tecla(&mut e, "Escape");
        // "Nenhum" devolve o rascunho de antes.
        tecla(&mut e, "p");
        tecla(&mut e, "Escape");
        tecla(&mut e, "p");
        tecla(&mut e, "Tab");
        tecla(&mut e, "Tab");
        tecla(&mut e, "Tab");
        tecla(&mut e, "k");
        tecla(&mut e, "Enter");
        let c = e.conversa.as_ref().unwrap();
        assert!(c.prompt.is_none());
        assert_eq!(c.rascunho.texto, "o parser");
    }

    #[test]
    fn escrevendo_o_campo_acende_e_o_rodape_diz_insercao() {
        let mut e = conversa_aberta();
        let destaque = e.tema.var("accent-blue");
        let conta_destaque = |e: &mut Estado| {
            let buf = quadro(e, 120, 40);
            (0..buf.area.height).rev().take(8).map(|y| (0..buf.area.width).filter(|x| buf[(*x, y)].style().fg == Some(destaque)).count()).sum::<usize>()
        };
        let antes = conta_destaque(&mut e);
        tecla(&mut e, "i");
        let tela = desenho(&mut e, 120, 40).join("\n");
        assert!(tela.contains("-- INSERÇÃO --"), "{tela}");
        assert!(conta_destaque(&mut e) > antes + 100, "o contorno do campo não acendeu");
        tecla(&mut e, "Escape");
        assert!(!desenho(&mut e, 120, 40).join("\n").contains("-- INSERÇÃO --"));
    }

    #[test]
    fn enter_no_botao_executa_a_acao() {
        let mut e = pagina_com("{{ type: \"actions\" }}\nbuttons:\n- label: Abrir\n  action: open-page\n  path: pages/beta.md\n- label: Marcar\n  action: set-property\n  path: pages/beta.md\n  field: status\n  value: feito\n- label: Nota\n  action: new-from-template\n  template: templates/nota.md\n  folder: pages/notas\n- label: Buscar\n  action: run-search\n  query: kanban\n- label: Vazio\n  action: open-page\n{{ /actions }}\n");
        e.cursor = vec![0, 0, 0];
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::AbrirPagina("pages/beta.md".into())]);
        e.pedidos.clear();
        e.cursor = vec![0, 0, 1];
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::DefinirPropriedade { path: "pages/beta.md".into(), campo: "status".into(), valor: "feito".into() }]);
        e.pedidos.clear();
        e.cursor = vec![0, 0, 2];
        tecla(&mut e, "Enter");
        digitar(&mut e, "Reunião");
        tecla(&mut e, "Enter");
        assert_eq!(
            e.pedidos,
            vec![Pedido::CriarDeTemplate { template: "templates/nota.md".into(), titulo: "Reunião".into(), pasta: Some("pages/notas".into()) }]
        );
        e.pedidos.clear();
        e.cursor = vec![0, 0, 3];
        tecla(&mut e, "Enter");
        let Some(Modal::Paleta(l)) = &e.modal else { panic!() };
        assert_eq!(l.filtro.as_ref().unwrap().texto, "kanban");
        tecla(&mut e, "Escape");
        e.cursor = vec![0, 0, 4];
        tecla(&mut e, "Enter");
        assert!(e.pedidos.is_empty() && e.aviso.as_deref().unwrap_or("").contains("sem destino") || e.aviso.as_deref().unwrap_or("").contains("não tem destino"));
    }

    #[test]
    fn enter_no_wikilink_abre_pergunta_ou_oferece_criar() {
        let mut e = pagina_com("Veja [[beta]].\n\nVeja [[Inexistente]].\n\n[[alfa]] e [[gama]]\n");
        e.cursor = vec![0];
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::AbrirPagina("pages/beta.md".into())]);
        e.pedidos.clear();
        e.cursor = vec![1];
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Confirmar { .. })));
        tecla(&mut e, "y");
        assert_eq!(e.pedidos, vec![Pedido::CriarPaginaComTitulo { titulo: "Inexistente".into(), tipo: None }]);
        e.pedidos.clear();
        e.cursor = vec![2];
        tecla(&mut e, "Enter");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::AbrirPagina("pages/gama.md".into())]);
    }

    #[test]
    fn ctrl_f_na_barra_busca_no_conteudo_e_mostra_os_trechos() {
        let mut e = pagina_com("texto");
        tecla(&mut e, ":");
        digitar(&mut e, "sprint");
        tecla(&mut e, "Ctrl+f");
        assert_eq!(e.pedidos, vec![Pedido::BuscarConteudo("sprint".into())]);
        e.pedidos.clear();
        modais::mostrar_resultados_da_busca(
            &mut e,
            "sprint",
            &[anotadinho_core::embed::SearchHit { path: "pages/beta.md".into(), snippet: "a **sprint** de agosto".into(), origem: Some("card em Backlog".into()), ancora: None }],
        );
        let tela = desenho(&mut e, 120, 30).join("\n");
        assert!(tela.contains("beta") && tela.contains("a sprint de agosto"), "{tela}");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::AbrirPagina("pages/beta.md".into())]);
    }

    #[test]
    fn enter_no_cartao_abre_os_detalhes_e_grava_cada_campo() {
        let mut e = kanban_editavel();
        e.agora = Some("2026-09-17 11:00".into());
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Detalhe { .. })));
        let tela = desenho(&mut e, 120, 40).join("\n");
        for t in ["Cartão · Backlog", "Título", "Escrever", "Descrição", "Tags", "doc", "Vencimento", "Checklist", "Comentários", "Anexos", "Excluir cartão"] {
            assert!(tela.contains(t), "faltou {t}:\n{tela}");
        }
        let cartao = |e: &Estado| kanban_gravado(e).items[0].clone();
        // descrição
        tecla(&mut e, "j");
        tecla(&mut e, "a");
        digitar(&mut e, "com exemplos");
        tecla(&mut e, "Enter");
        assert_eq!(cartao(&e).description.as_deref(), Some("com exemplos"));
        // tags: + tag
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        digitar(&mut e, "urgente");
        tecla(&mut e, "Enter");
        assert_eq!(cartao(&e).tags, ["doc", "urgente"]);
        // vencimento inválido não grava (o cursor ficou no "+ tag")
        tecla(&mut e, "j");
        tecla(&mut e, "c");
        digitar(&mut e, "amanhã");
        tecla(&mut e, "Enter");
        assert!(e.aviso.as_deref().unwrap_or("").contains("AAAA-MM-DD"));
        assert_eq!(cartao(&e).due, None);
        tecla(&mut e, "c");
        digitar(&mut e, "2026-09-30");
        tecla(&mut e, "Enter");
        assert_eq!(cartao(&e).due.as_deref(), Some("2026-09-30"));
        // checklist: novo item e marcar
        tecla(&mut e, "j");
        tecla(&mut e, "o");
        digitar(&mut e, "revisar");
        tecla(&mut e, "Enter");
        tecla(&mut e, "k");
        tecla(&mut e, "~");
        assert_eq!(cartao(&e).checklist.iter().map(|c| (c.done, c.text.as_str())).collect::<Vec<_>>(), [(true, "revisar")]);
        // comentário com a hora de agora
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        digitar(&mut e, "ok");
        tecla(&mut e, "Enter");
        let c = cartao(&e);
        assert_eq!((c.comments[0].text.as_str(), c.comments[0].created.as_str()), ("ok", "2026-09-17 11:00"));
        tecla(&mut e, "Escape");
        assert!(e.modal.is_none());
    }

    #[test]
    fn enter_no_evento_abre_os_detalhes_com_varios_dias_e_horario() {
        let mut e = editavel();
        e.cursor = achar_evento_na_tela(&e, "Reunião");
        tecla(&mut e, "Enter");
        let Some(Modal::Detalhe { form, .. }) = &e.modal else { panic!("{:?}", e.modal) };
        assert!(!form.booleano("varios"));
        let tela = desenho(&mut e, 120, 40).join("\n");
        assert!(tela.contains("Evento") && tela.contains("Vários dias") && !tela.contains("Fim "), "{tela}");
        // título, início
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, " ");
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Fim"));
        tecla(&mut e, "j");
        tecla(&mut e, "c");
        digitar(&mut e, "2026-08-13");
        tecla(&mut e, "Enter");
        assert_eq!(gravado(&e).entries[1].end_date.as_deref(), Some("2026-08-13"));
        tecla(&mut e, "j");
        tecla(&mut e, " ");
        tecla(&mut e, "j");
        tecla(&mut e, "c");
        digitar(&mut e, "9h");
        tecla(&mut e, "Enter");
        assert!(e.aviso.as_deref().unwrap_or("").contains("HH:MM"));
        tecla(&mut e, "c");
        digitar(&mut e, "09:30");
        tecla(&mut e, "Enter");
        assert_eq!(gravado(&e).entries[1].start_time.as_deref(), Some("09:30"));
        // Excluir pelo botão.
        for _ in 0..6 {
            tecla(&mut e, "j");
        }
        tecla(&mut e, "Enter");
        assert!(e.modal.is_none());
        assert_eq!(gravado(&e).entries.len(), 1);
    }

    #[test]
    fn propriedades_da_pagina_editam_o_frontmatter_e_mantem_o_corpo() {
        let mut e = markdown_editavel();
        tecla(&mut e, ":");
        digitar(&mut e, "propriedades");
        tecla(&mut e, "Enter");
        let tela = desenho(&mut e, 120, 40).join("\n");
        assert!(tela.contains("Propriedades") && tela.contains("Notas") && tela.contains("Página normal"), "{tela}");
        // tipo: gira pra landing
        tecla(&mut e, "j");
        tecla(&mut e, "~");
        let texto = e.gravacao.clone().unwrap();
        assert!(texto.starts_with("---\ntitle: Notas\ntype: landing\n---\n# Título"), "{texto}");
        // propriedade livre
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        digitar(&mut e, "status: doing");
        tecla(&mut e, "Enter");
        let texto = e.gravacao.clone().unwrap();
        assert!(texto.contains("status: doing") && texto.contains("> citação"), "{texto}");
        tecla(&mut e, "Enter");
        digitar(&mut e, "sem dois pontos");
        tecla(&mut e, "Enter");
        assert!(e.aviso.as_deref().unwrap_or("").contains("chave: valor"));
    }

    #[test]
    fn igual_configura_o_botao_e_a_consulta() {
        use anotadinho_core::embed::EmbedData;
        let mut e = pagina_com("{{ type: \"actions\" }}\nbuttons:\n- label: Ir\n  action: open-page\n{{ /actions }}\n\n{{ type: \"query\" }}\nfrom: pages\n{{ /query }}\n");
        e.cursor = vec![0, 0, 0];
        tecla(&mut e, "=");
        let Some(Modal::Detalhe { form, .. }) = &e.modal else { panic!() };
        assert!(form.campos.iter().find(|c| c.chave == "query").unwrap().escondido);
        // ação: open-page → new-from-template mostra template e pasta
        for _ in 0..3 {
            tecla(&mut e, "j");
        }
        tecla(&mut e, "~");
        let Some(Modal::Detalhe { form, .. }) = &e.modal else { panic!() };
        assert!(!form.campos.iter().find(|c| c.chave == "template").unwrap().escondido);
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        digitar(&mut e, "templates/nota.md");
        tecla(&mut e, "Enter");
        let EmbedData::Actions(d) = embed_gravado(&e) else { panic!() };
        assert_eq!((d.buttons[0].action.as_str(), d.buttons[0].template.as_deref()), ("new-from-template", Some("templates/nota.md")));
        tecla(&mut e, "Escape");
        // A consulta.
        e.cursor = vec![1];
        tecla(&mut e, "Enter");
        tecla(&mut e, "=");
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        digitar(&mut e, "status!=done");
        tecla(&mut e, "Enter");
        tecla(&mut e, "Enter");
        digitar(&mut e, "=ruim");
        tecla(&mut e, "Enter");
        assert!(e.aviso.as_deref().unwrap_or("").contains("inválida"));
        let texto = e.gravacao.clone().unwrap();
        let q = anotadinho_core::embed::segment(&texto).into_iter().find_map(|s| match s {
            anotadinho_core::embed::DocSegment::Embed(EmbedData::Query(q)) => Some(q),
            _ => None,
        });
        assert_eq!(q.unwrap().conditions.len(), 1);
    }

    #[test]
    fn na_sidebar_o_cria_pagina_na_pasta_o_maiusculo_pasta_m_move_dd_exclui() {
        let paginas = vec![
            PageMeta { path: "pages/raiz.md".into(), title: "raiz".into(), section: "pages".into() },
            PageMeta { path: "pages/specs/editor.md".into(), title: "editor".into(), section: "pages".into() },
        ];
        let mut e = Estado::novo(paginas, analisar("")).com_pastas(vec!["pages/specs".into(), "pages/vazia".into()]);
        e.foco = Foco::Paginas;
        // A pasta vazia aparece.
        let tela = desenho(&mut e, 100, 20).join("\n");
        assert!(tela.contains("vazia/"), "{tela}");
        // Cursor na pasta specs (primeira linha).
        e.linha_sidebar = 0;
        tecla(&mut e, "o");
        digitar(&mut e, "Nova spec");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::CriarPaginaNaPasta { pasta: "pages/specs".into(), titulo: "Nova spec".into() }]);
        e.pedidos.clear();
        tecla(&mut e, "O");
        digitar(&mut e, "antigas");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::CriarPasta("pages/specs/antigas".into())]);
        e.pedidos.clear();
        // Numa página: m move, dd exclui (com confirmação).
        let linha_raiz = e.sidebar_visivel().iter().position(|l| matches!(&l.item, Item::Pagina { titulo, .. } if titulo == "raiz")).unwrap();
        e.linha_sidebar = linha_raiz;
        e.pagina = 0;
        tecla(&mut e, "m");
        digitar(&mut e, "specs");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, vec![Pedido::MoverPagina { de: "pages/raiz.md".into(), para: "pages/specs/raiz.md".into() }]);
        e.pedidos.clear();
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert!(matches!(e.modal, Some(Modal::Confirmar { .. })));
        tecla(&mut e, "y");
        assert_eq!(e.pedidos, vec![Pedido::ExcluirPagina("pages/raiz.md".into())]);
        // dd numa pasta não exclui nada.
        e.pedidos.clear();
        e.linha_sidebar = 0;
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        assert!(e.modal.is_none() && e.pedidos.is_empty());
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
        e.cursor = achar_com_indice(&e.arvore, &[1], "barra", 0).expect("sem hoje, as barras do arquivo aparecem");
        tecla(&mut e, ">");
        tecla(&mut e, ">");
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
        // No cabeçalho, `dd` é da COLUNA (ciclo 337): ela sai com as células.
        e.cursor = vec![1, 0, 0];
        tecla(&mut e, "d");
        tecla(&mut e, "d");
        let d = tabela_gravada(&e);
        assert_eq!(d.columns.len(), 2);
        assert_eq!(d.rows[0].len(), 2);
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
        // exatamente onde a primeira parou. A barra é uma linha só, como
        // `.timeline__bar-item` (ciclo 332): o fundo pintado é a barra.
        let mut e = Estado::novo(paginas(), com_cronograma());
        e.foco = Foco::Paginas;
        let buf = quadro(&mut e, 80, 16);
        let fundo_da_barra = e.tema.pilula(Realce::BadgeInfo).bg;
        let trecho = |texto: &str| -> (usize, usize) {
            let y = (0..buf.area.height)
                .find(|y| (0..buf.area.width).map(|x| buf[(x, *y)].symbol().to_string()).collect::<String>().contains(texto))
                .unwrap_or_else(|| panic!("{texto} sumiu"));
            let pintadas: Vec<usize> =
                (0..buf.area.width).filter(|x| buf[(*x, y)].style().bg == fundo_da_barra).map(|x| x as usize).collect();
            (pintadas[0], *pintadas.last().unwrap())
        };
        let (i1, f1) = trecho("Primeira etapa");
        let (i2, f2) = trecho("Segunda etapa");
        assert!(i2 > i1 && i2 >= f1, "a segunda barra devia começar onde a primeira acaba: {i1}-{f1} {i2}-{f2}");
        let (l1, l2) = (f1 - i1, f2 - i2);
        assert!(l1.abs_diff(l2) <= 1, "as duas duram o mesmo e saíram de tamanhos diferentes: {l1} {l2}");
        assert!(f1 < 60, "a primeira barra ocupou o painel inteiro, não a metade");
    }

    #[test]
    fn o_cronograma_tem_eixo_de_datas_alinhado_com_as_barras() {
        // Janela de 1 a 20 de agosto: marcas semanais em 01/08, 08/08 e
        // 15/08, como `.timeline__tick`.
        let mut e = Estado::novo(paginas(), com_cronograma());
        e.foco = Foco::Paginas;
        let linhas = desenho(&mut e, 90, 20);
        let tudo = linhas.join("\n");
        let rotulos = linhas.iter().find(|l| l.contains("01/08")).unwrap_or_else(|| panic!("sem eixo:\n{tudo}"));
        assert!(rotulos.contains("08/08") && rotulos.contains("15/08"), "{rotulos}");
        let regua = linhas.iter().find(|l| l.contains('┬')).unwrap_or_else(|| panic!("sem régua:\n{tudo}"));
        assert_eq!(colunas_de(regua, '┬').len(), 3, "{regua}");
        // A primeira marca (01/08) cai na coluna onde a primeira barra
        // começa: as duas usam a mesma conta.
        let barra = linhas.iter().find(|l| l.contains("Primeira etapa")).unwrap();
        let comeco = barra.chars().collect::<Vec<_>>().windows(2).position(|w| w[0] == ' ' && w[1] == 'P').unwrap();
        assert_eq!(colunas_de(regua, '┬')[0], comeco, "\n{regua}\n{barra}");
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
        // some e o botão encolhe meia célula de cada lado. O botão comum
        // de ações é `--bg-surface`, como `.actions-embed__btn`.
        let mut e = Estado::novo(paginas(), com_acoes());
        e.foco = Foco::Paginas;
        let cor = e.tema.var("bg-surface");

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
        // A trilha num painel estreito QUEBRA em vez de sumir pela borda
        // — o que sumia podia ser a etapa atual.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n"),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 70, 24).join("\n");
        for etapa in ["Rascunho", "Em revisão", "Aprovada", "Em execução", "Concluída"] {
            assert!(tudo.contains(etapa), "`{etapa}` sumiu da trilha:\n{tudo}");
        }
        // Bloqueada é exceção: fora da trilha, como na janela — a não ser
        // quando é onde o fluxo está.
        let mut e = Estado::novo(
            paginas(),
            analisar("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: bloqueada\n{{ /fluxo }}\n"),
        );
        e.foco = Foco::Paginas;
        let tudo = desenho(&mut e, 100, 24).join("\n");
        assert!(tudo.matches("Bloqueada").count() >= 2, "a etapa atual bloqueada sumiu da trilha:\n{tudo}");
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

    // --- Ciclo 346: tags, assets e propostas -------------------------

    fn especial_aberto(tipo: &str) -> Estado {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.abrir_texto(&format!("---\ntitle: X\ntype: {tipo}\n---\n"), None);
        e.foco = Foco::Conteudo;
        e
    }

    fn proposta(id: &str, op: anotadinho_core::proposta::Operacao, alvo: &str, conteudo: &str) -> anotadinho_core::proposta::Proposta {
        anotadinho_core::proposta::Proposta {
            id: id.into(),
            autor: "claude".into(),
            quando: "2026-09-17 10:00".into(),
            motivo: "arrumar a lista".into(),
            alvo: alvo.into(),
            operacao: op,
            conteudo: conteudo.into(),
        }
    }

    #[test]
    fn pagina_de_tags_abre_carregando_e_pede_os_dados() {
        let mut e = especial_aberto("tags");
        assert_eq!(e.pedidos, [Pedido::CarregarEspecial(especiais::TipoEspecial::Tags)]);
        assert!(desenho(&mut e, 100, 20).join("\n").contains("Carregando..."));
        let indice = vec![
            anotadinho_core::index::PageIndexEntry { path: "pages/a.md".into(), title: "Alfa".into(), embed_tags: vec!["infra".into(), "urgente".into()], ..Default::default() },
            anotadinho_core::index::PageIndexEntry { path: "pages/b.md".into(), title: "Beta".into(), embed_tags: vec!["infra".into()], ..Default::default() },
        ];
        especiais::carregar(&mut e, especiais::tags_do_indice(&indice));
        let tudo = desenho(&mut e, 100, 20).join("\n");
        for esperado in ["Tags", " infra ", " 2", "Alfa", "Beta", " urgente ", "Enter abre"] {
            assert!(tudo.contains(esperado), "faltou {esperado}:\n{tudo}");
        }
        // l anda pela página da tag; Enter abre.
        e.pedidos.clear();
        tecla(&mut e, "l");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/b.md".into())]);
        e.pedidos.clear();
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/a.md".into())]);
        // Sem tags, a frase da janela.
        let mut e = especial_aberto("tags");
        especiais::carregar(&mut e, especiais::tags_do_indice(&[]));
        assert!(desenho(&mut e, 100, 20).join("\n").contains("Nenhuma tag encontrada"));
    }

    #[test]
    fn assets_mostram_uso_e_excluir_pede_confirmacao() {
        let mut e = especial_aberto("assets");
        let dados = especiais::assets_com_uso(
            vec![("assets/foto.png".into(), 2048), ("assets/velho.pdf".into(), 10)],
            "![](../assets/foto.png)",
        );
        especiais::carregar(&mut e, dados);
        let tudo = desenho(&mut e, 100, 20).join("\n");
        for esperado in ["Assets", "2 arquivos · 2.0 KB · 1 não referenciados", "assets/foto.png", " usado ", " não usado ", "Excluir"] {
            assert!(tudo.contains(esperado), "faltou {esperado}:\n{tudo}");
        }
        tecla(&mut e, "j");
        tecla(&mut e, "d");
        assert!(e.modal.is_none(), "um d só espera o segundo");
        tecla(&mut e, "d");
        assert!(matches!(&e.modal, Some(Modal::Confirmar { mensagem, .. }) if mensagem.contains("velho.pdf")));
        e.pedidos.clear();
        tecla(&mut e, "y");
        assert_eq!(e.pedidos, [Pedido::ExcluirAsset("assets/velho.pdf".into())]);
    }

    #[test]
    fn propostas_mostram_diff_e_aplicam_ou_recusam() {
        use anotadinho_core::proposta::Operacao;
        let mut e = especial_aberto("propostas");
        especiais::carregar(
            &mut e,
            especiais::Dados::Propostas(vec![
                especiais::PropostaNaTela { proposta: proposta("p1", Operacao::Substituir, "pages/alfa.md", "# Alfa\n\nnovo\n"), atual: "# Alfa\n\nvelho\n".into() },
                especiais::PropostaNaTela { proposta: proposta("p2", Operacao::Criar, "pages/nova.md", "# Nova\n"), atual: String::new() },
            ]),
        );
        let tudo = desenho(&mut e, 100, 40).join("\n");
        for esperado in ["Propostas do agente", "2 pendente(s)", " SUBSTITUIR ", "pages/alfa.md", "ϟ claude", "arrumar a lista", "1 linha(s) removida(s) · 1 adicionada(s)", "-velho", "+novo", " CRIAR ", "Aplicar a", "Recusar r"] {
            assert!(tudo.contains(esperado), "faltou {esperado}:\n{tudo}");
        }
        // v troca pra Visualização: o markdown, sem os sinais do diff.
        tecla(&mut e, "v");
        let tudo = desenho(&mut e, 100, 40).join("\n");
        assert!(!tudo.contains("+novo") && tudo.contains("novo"), "{tudo}");
        // a aplica, com confirmação; r recusa a segunda.
        e.pedidos.clear();
        tecla(&mut e, "a");
        tecla(&mut e, "y");
        assert_eq!(e.pedidos, [Pedido::DecidirProposta { id: "p1".into(), aplicar: true }]);
        e.pedidos.clear();
        tecla(&mut e, "j");
        tecla(&mut e, "r");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::DecidirProposta { id: "p2".into(), aplicar: false }]);
        // Vazia, a frase da janela.
        let mut e = especial_aberto("propostas");
        especiais::carregar(&mut e, especiais::Dados::Propostas(vec![]));
        assert!(desenho(&mut e, 100, 20).join("\n").contains("Nada pendente."));
    }

    #[test]
    fn a_barra_de_comandos_abre_tags_assets_e_propostas() {
        let mut e = Estado::novo(paginas(), analisar("# x\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, ":");
        digitar(&mut e, "propostas do agente");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirEspecial(especiais::TipoEspecial::Propostas)]);
        assert_eq!(especiais::TipoEspecial::Propostas.pagina().0, "pages/propostas.md");
    }

    // --- Ciclo 347: o menu `/` ------------------------------------------

    #[test]
    fn barra_num_bloco_novo_vazio_abre_o_menu_de_inserir() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        tecla(&mut e, "/");
        assert!(e.pergunta.is_none());
        let tela = desenho(&mut e, 100, 40).join("\n");
        for esperado in ["Inserir", "Título 1", "Checklist", "Tabela 3×2", "Kanban"] {
            assert!(tela.contains(esperado), "faltou {esperado}:\n{tela}");
        }
        // Filtra e escolhe um título: a inserção continua com a marca.
        digitar(&mut e, "titulo 2");
        tecla(&mut e, "Enter");
        let p = e.pergunta.as_ref().expect("não voltou pra inserção");
        assert!(matches!(&p.acao, edicao::AcaoDaPergunta::Bloco(b) if b.prefixo == "## "));
        digitar(&mut e, "Seção");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("Um parágrafo com **negrito**.\n\n## Seção\n\n"), "{}", corpo_gravado(&e));
        // `/` com texto já digitado é só texto.
        tecla(&mut e, "o");
        digitar(&mut e, "a/b");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("## Seção\n\na/b\n"), "{}", corpo_gravado(&e));
    }

    #[test]
    fn o_menu_insere_embed_tabela_e_linha_inteiros() {
        let mut e = markdown_editavel().com_hoje("2026-08-12");
        e.cursor = vec![0];
        tecla(&mut e, "o");
        tecla(&mut e, "/");
        digitar(&mut e, "kanban");
        tecla(&mut e, "Enter");
        let corpo = corpo_gravado(&e);
        assert!(corpo.starts_with("# Título\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog"), "{corpo}");
        assert!(corpo.contains("column: Backlog\n{{ /kanban }}\n\nUm parágrafo"), "{corpo}");
        // O cursor vai pro embed novo.
        assert!(matches!(e.arvore.em(&e.cursor).map(|u| &u.tipo), Some(Tipo::Embed(n)) if n == "kanban"));
        // Pela barra de comandos: um bloco depois do cursor.
        e.cursor = vec![0];
        tecla(&mut e, ":");
        digitar(&mut e, "inserir bloco");
        tecla(&mut e, "Enter");
        digitar(&mut e, "linha");
        tecla(&mut e, "Enter");
        assert!(corpo_gravado(&e).starts_with("# Título\n\n---\n\n{{ type"), "{}", corpo_gravado(&e));
        // Diagrama pede o código; Esc desiste sem gravar.
        e.gravacao = None;
        tecla(&mut e, "o");
        tecla(&mut e, "/");
        digitar(&mut e, "diagrama");
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Entrada { .. })));
        digitar(&mut e, "graph TD; A-->B");
        tecla(&mut e, "Enter");
        assert!(corpo_gravado(&e).contains("```mermaid\ngraph TD; A-->B\n```"), "{}", corpo_gravado(&e));
        assert!(e.bloco_a_inserir.is_none());
    }

    #[test]
    fn assets_do_menu_viram_imagem_ou_link() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        tecla(&mut e, "/");
        digitar(&mut e, "assets");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AssetsParaInserir]);
        markdown::escolher_asset(&mut e, vec!["assets/foto.png".into(), "assets/doc.pdf".into()]);
        digitar(&mut e, "foto");
        tecla(&mut e, "Enter");
        assert!(corpo_gravado(&e).contains("**negrito**.\n\n![imagem](assets/foto.png)\n"), "{}", corpo_gravado(&e));
        assert_eq!(markdown::markdown_do_asset("assets/doc.pdf"), "[assets/doc.pdf](assets/doc.pdf)");
    }

    // --- Ciclo 348: a ação do fluxo abre a conversa ---------------------

    #[test]
    fn a_acao_do_fluxo_pede_a_conversa_e_a_origem_abre_a_pagina() {
        let parte = |e: &Estado, nome: &str| {
            e.arvore
                .percorrer()
                .into_iter()
                .find(|(_, u)| matches!(&u.tipo, Tipo::Parte { nome: n, .. } if n == nome) && !u.texto.is_empty())
                .map(|(c, _)| c)
                .unwrap_or_else(|| panic!("sem {nome}"))
        };
        let mut e = pagina_com("{{ type: \"fluxo\" }}\nartefato: spec\netapa: aprovada\norigem: pages/conversas/c1.md\n{{ /fluxo }}\n");
        let tela = desenho(&mut e, 100, 20).join("\n");
        assert!(tela.contains("↗ origem") && tela.contains("Planejar implementação"), "{tela}");
        assert!(!tela.contains("pages/conversas/c1.md"), "o caminho da origem é escondido:\n{tela}");
        e.cursor = parte(&e, "acao");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::ConversaDoFluxo { pagina: "pages/alfa.md".into(), alterar: false }]);
        e.pedidos.clear();
        e.cursor = parte(&e, "origem");
        assert_eq!(tecla(&mut e, "Enter"), Some("pages/conversas/c1.md".into()));
        // Em revisão, "Pedir alteração".
        let mut e = pagina_com("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: em-revisao\n{{ /fluxo }}\n");
        e.cursor = parte(&e, "acao");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::ConversaDoFluxo { pagina: "pages/alfa.md".into(), alterar: true }]);
    }

    #[test]
    fn a_pergunta_do_fluxo_vai_pro_campo_da_conversa_sem_enviar() {
        let mut e = conversa_aberta();
        conversa::escrever_no_campo(&mut e, "Leia a spec e planeje.");
        let c = e.conversa.as_ref().unwrap();
        assert_eq!(c.rascunho.texto, "Leia a spec e planeje.");
        assert!(!c.escrevendo && e.pedidos.is_empty());
        assert!(desenho(&mut e, 120, 40).join("\n").contains("Leia a spec e planeje."));
    }

    // --- Ciclo 349: git ----------------------------------------------------

    #[test]
    fn git_mostra_as_mudancas_e_pede_pull_ou_commit() {
        let mut e = Estado::novo(paginas(), analisar("# x\n"));
        e.foco = Foco::Conteudo;
        tecla(&mut e, ":");
        digitar(&mut e, "git status");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::StatusDoGit]);
        e.pedidos.clear();
        modais::mostrar_git(&mut e, Some(vec![("M".into(), "pages/beta.md".into()), ("??".into(), "assets/x.png".into())]));
        let tela = desenho(&mut e, 100, 30).join("\n");
        for esperado in ["Git · 2 mudança(s)", "Pull", "Commit + Push…", "pages/beta.md", "??"] {
            assert!(tela.contains(esperado), "faltou {esperado}:\n{tela}");
        }
        // Um arquivo que é página abre; o que não é, fica.
        tecla(&mut e, "j");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/beta.md".into())]);
        e.pedidos.clear();
        modais::mostrar_git(&mut e, Some(vec![]));
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert!(matches!(&e.modal, Some(Modal::Entrada { titulo, .. }) if titulo == "Mensagem do commit"));
        digitar(&mut e, "notas de hoje");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::GitCommit("notas de hoje".into())]);
        // Fora de repositório, o aviso da janela.
        modais::mostrar_git(&mut e, None);
        assert!(e.modal.is_none() && e.aviso.as_deref().unwrap().contains("não é um repositório git"));
    }

    #[test]
    fn historico_da_pagina_lista_os_commits() {
        let mut e = Estado::novo(paginas(), analisar("# x\n"));
        tecla(&mut e, ":");
        digitar(&mut e, "historico da pagina");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::HistoricoDoGit("pages/alfa.md".into())]);
        modais::mostrar_historico(&mut e, Some(vec![("abc123".into(), "2026-09-17".into(), "feat: notas".into())]));
        let tela = desenho(&mut e, 100, 30).join("\n");
        assert!(tela.contains("Histórico") && tela.contains("feat: notas") && tela.contains("abc123 · 2026-09-17"), "{tela}");
        modais::mostrar_historico(&mut e, Some(vec![]));
        assert!(e.aviso.as_deref().unwrap().contains("Nenhum commit"));
    }

    // --- Ciclo 350: templates e configuração do agente ---------------------

    #[test]
    fn nova_pagina_pergunta_o_template_quando_o_vault_tem() {
        let mut e = Estado::novo(paginas(), analisar("# x\n"));
        modais::executar(&mut e, "nova-pagina");
        assert_eq!(e.pedidos, [Pedido::ListarTemplates]);
        e.pedidos.clear();
        // Sem templates, direto o título.
        modais::escolher_template(&mut e, vec![]);
        assert!(matches!(&e.modal, Some(Modal::Entrada { acao: modais::AcaoDaEntrada::NovaPagina(None), .. })));
        e.modal = None;
        modais::escolher_template(&mut e, vec![("templates/reuniao.md".into(), "Reunião".into())]);
        let tela = desenho(&mut e, 100, 30).join("\n");
        assert!(tela.contains("Escolher template") && tela.contains("Página em branco") && tela.contains("Reunião"), "{tela}");
        tecla(&mut e, "ArrowDown");
        tecla(&mut e, "Enter");
        digitar(&mut e, "Daily");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::CriarDeTemplate { template: "templates/reuniao.md".into(), titulo: "Daily".into(), pasta: None }]);
    }

    #[test]
    fn configurar_agente_valida_e_grava_o_personalizado() {
        let mut e = Estado::novo(paginas(), analisar("# x\n"));
        modais::executar(&mut e, "configurar-agente");
        let tela = desenho(&mut e, 100, 40).join("\n");
        for esperado in ["Agente das conversas", "Executável", "Argumentos", "Formato da saída", "Tempo limite (minutos)", "Pastas extras", "Salvar e usar"] {
            assert!(tela.contains(esperado), "faltou {esperado}:\n{tela}");
        }
        // Executável vazio: o problema aparece e o formulário fica.
        let Some(Modal::Detalhe { form, .. }) = e.modal.as_mut() else { panic!() };
        form.campos[1].valor = crate::componentes::Valor::Texto(String::new());
        for _ in 0..40 {
            tecla(&mut e, "j");
        }
        tecla(&mut e, "Enter");
        assert!(e.modal.is_some(), "salvou sem executável");
        assert!(e.aviso.is_some());
        // Com executável e `{prompt}`, grava e usa.
        let Some(Modal::Detalhe { form, .. }) = e.modal.as_mut() else { panic!() };
        form.campos[0].valor = crate::componentes::Valor::Texto("meu".into());
        form.campos[1].valor = crate::componentes::Valor::Texto("/opt/agente".into());
        form.campos[2].valor = crate::componentes::Valor::Lista(vec!["-p".into(), "{prompt}".into()]);
        tecla(&mut e, "Enter");
        assert!(e.modal.is_none());
        let a = e.preferencias.agente.as_ref().unwrap();
        assert_eq!((a.nome.as_str(), a.binario.as_str(), a.args.len()), ("meu", "/opt/agente", 2));
        assert!(e.pedidos.contains(&Pedido::GravarPreferencias));
    }

    // --- Ciclo 351: arquivo mudado por fora --------------------------------

    #[test]
    fn mudanca_por_fora_rele_sem_perder_o_lugar() {
        let mut e = markdown_editavel();
        e.cursor = vec![2, 1];
        tecla(&mut e, "~");
        assert!(!e.desfazer.is_empty());
        e.gravacao = None;
        let novo = PAGINA_MARKDOWN.replace("tarefa dois", "tarefa dois (editada no outro editor)");
        assert!(e.recarregar_do_disco(&novo, Some("v2".into())));
        assert_eq!(e.cursor, vec![2, 1], "o cursor ficou onde estava");
        assert!(e.arvore.em(&e.cursor).unwrap().texto.contains("editada no outro editor"));
        assert!(e.desfazer.is_empty() && e.gravacao.is_none());
        assert_eq!(e.versao.as_deref(), Some("v2"));
        assert!(desenho(&mut e, 100, 20).join("\n").contains("mudou por fora"));
        // Inserindo, espera.
        tecla(&mut e, "A");
        assert!(!e.recarregar_do_disco(PAGINA_MARKDOWN, Some("v3".into())));
        assert_eq!(e.versao.as_deref(), Some("v2"));
    }

    #[test]
    fn a_lista_nova_mantem_a_pagina_aberta_pelo_caminho() {
        let mut e = Estado::novo(paginas(), analisar(""));
        e.pagina = 1; // beta
        let mut novas = paginas();
        novas.insert(0, PageMeta { path: "pages/aaa.md".into(), title: "aaa".into(), section: "pages".into() });
        e.atualizar_paginas(novas);
        assert_eq!(e.paginas[e.pagina].path, "pages/beta.md");
    }

    // --- Ciclo 352: autocompletar wikilink -----------------------------------

    #[test]
    fn colchetes_duplos_sugerem_paginas_e_enter_completa() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "ver [[ga");
        let tela = desenho(&mut e, 100, 30).join("\n");
        assert!(tela.contains("Wikilink") && tela.contains("gama"), "{tela}");
        tecla(&mut e, "Enter");
        let p = e.pergunta.as_ref().expect("Enter completou em vez de confirmar");
        assert_eq!((p.texto.as_str(), p.cursor), ("ver [[gama]]", 12));
        // Fechado o `]]`, Enter volta a confirmar.
        digitar(&mut e, " ok");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("ver [[gama]] ok"), "{}", corpo_gravado(&e));
    }

    #[test]
    fn setas_escolhem_e_esc_fecha_so_a_lista() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "[[a");
        // alfa começa com "a"; beta e gama só contêm.
        tecla(&mut e, "ArrowDown");
        tecla(&mut e, "Tab");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "[[beta]]");
        tecla(&mut e, "Backspace");
        tecla(&mut e, "Backspace");
        digitar(&mut e, " [[x");
        assert!(desenho(&mut e, 100, 30).join("\n").contains("Wikilink") == false, "nada bate com x");
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "[[al");
        tecla(&mut e, "Escape");
        assert!(e.pergunta.is_some(), "o primeiro Esc só fecha a lista");
        assert!(!desenho(&mut e, 100, 30).join("\n").contains("Wikilink"));
        tecla(&mut e, "Escape");
        assert!(e.pergunta.is_none());
    }

    // --- Ciclo 353: kanban, tabela e calendário de página inteira ------------

    #[test]
    fn pagina_kanban_le_os_itens_com_column_e_anda_pelas_colunas() {
        let mut e = pagina_com("---\ntitle: Q\ntype: kanban\n---\n\n- A  column:: todo\n- title:: B  column:: todo\n- C\n- D  column:: outra\n");
        assert!(e.pedidos.is_empty(), "o kanban de página não pede nada ao main");
        let Some(especiais::Dados::Kanban(c)) = e.especial.as_ref().and_then(|t| t.dados.clone()) else { panic!() };
        assert_eq!(c[0].2, ["C"]);
        assert_eq!(c[1].2, ["A", "B"]);
        tecla(&mut e, "l");
        tecla(&mut e, "j");
        let t = e.especial.as_ref().unwrap();
        assert_eq!((t.chip, t.selecionado), (1, 1));
        let tela = desenho(&mut e, 120, 20).join("\n");
        assert!(tela.contains("A Fazer") && tela.contains("Concluído") && !tela.contains(" D "), "{tela}");
    }

    #[test]
    fn pagina_de_tarefas_ordena_e_abre() {
        let mut e = pagina_com("---\ntitle: T\ntype: table\n---\n");
        assert_eq!(e.pedidos, [Pedido::CarregarEspecial(especiais::TipoEspecial::Tarefas)]);
        e.pedidos.clear();
        especiais::carregar(
            &mut e,
            especiais::tarefas_das_paginas(vec![
                ("pages/b.md".into(), "B".into(), "status:: todo".into()),
                ("pages/a.md".into(), "A".into(), "status:: done".into()),
                ("pages/c.md".into(), "C".into(), "nada".into()),
            ]),
        );
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/a.md".into())]);
        e.pedidos.clear();
        tecla(&mut e, "s"); // por status: done < todo
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/b.md".into())]);
        assert!(!desenho(&mut e, 100, 20).join("\n").contains("C "), "página sem status:: fica de fora");
    }

    #[test]
    fn pagina_de_calendario_e_o_calendario_do_vault_so_leitura() {
        let e = pagina_com("---\ntitle: Agenda\ntype: calendar\n---\n\n- \n");
        assert!(e.texto_da_pagina.is_none());
        assert!(matches!(e.arvore.filhos.first().map(|u| &u.tipo), Some(Tipo::Embed(n)) if n == "calendar"));
    }

    // --- Ciclo 354: abas ----------------------------------------------------

    #[test]
    fn abas_abrem_com_as_paginas_e_alt_numero_troca() {
        let mut e = Estado::novo(paginas(), analisar("# a\n")).com_texto("# a\n", None);
        assert_eq!(e.abas, ["pages/alfa.md"]);
        e.pagina = 2;
        e.abrir_texto("# gama\n", None);
        e.pagina = 1;
        e.abrir_texto("# beta\n", None);
        assert_eq!(e.abas, ["pages/alfa.md", "pages/gama.md", "pages/beta.md"]);
        let tela = desenho(&mut e, 120, 10).join("\n");
        assert!(tela.contains("1 alfa") && tela.contains("2 gama") && tela.contains("3 beta"), "{tela}");
        assert_eq!(tecla(&mut e, "Alt+1"), Some("pages/alfa.md".into()));
        assert_eq!(e.pagina, 0);
        assert_eq!(tecla(&mut e, "Ctrl+w"), Some("pages/gama.md".into()));
        assert_eq!(tecla(&mut e, "Alt+h"), Some("pages/alfa.md".into()));
        assert_eq!(tecla(&mut e, "Alt+h"), Some("pages/beta.md".into()), "volta dá a volta");
        assert_eq!(tecla(&mut e, "Alt+9"), None);
        assert!(e.aviso.as_deref().unwrap().contains("não há aba 9"));
        // Fechar a aba abre a vizinha; a última não fecha.
        assert_eq!(tecla(&mut e, "Alt+q"), Some("pages/gama.md".into()), "beta era a última: abre a da esquerda");
        assert_eq!(e.abas, ["pages/alfa.md", "pages/gama.md"]);
        tecla(&mut e, "Alt+q");
        assert_eq!(tecla(&mut e, "Alt+q"), None);
        assert_eq!(e.abas.len(), 1);
        // Com uma aba só, a borda volta ao título.
        assert!(!desenho(&mut e, 120, 10).join("\n").contains("1 alfa"));
    }

    #[test]
    fn pagina_apagada_perde_a_aba() {
        let mut e = Estado::novo(paginas(), analisar("# a\n")).com_texto("# a\n", None);
        e.pagina = 1;
        e.abrir_texto("# beta\n", None);
        let sem_beta: Vec<PageMeta> = paginas().into_iter().filter(|p| p.path != "pages/beta.md").collect();
        e.atualizar_paginas(sem_beta);
        assert_eq!(e.abas, ["pages/alfa.md"]);
    }

    #[test]
    fn a_sidebar_mostra_propostas_pendentes_e_mudancas_no_git() {
        let mut e = Estado::novo(paginas(), analisar("# a\n"));
        assert!(!desenho(&mut e, 120, 12).join("\n").contains("proposta(s)"));
        e.propostas_pendentes = 2;
        e.mudancas_no_git = Some(3);
        let tela = desenho(&mut e, 120, 12).join("\n");
        assert!(tela.contains("✓ 2 proposta(s)") && tela.contains("⑂ 3"), "{tela}");
    }

    // --- Ciclo 357: formatar na inserção --------------------------------------

    #[test]
    fn ctrl_b_poe_e_tira_negrito_na_palavra_do_cursor() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "muito importante");
        tecla(&mut e, "Ctrl+b");
        let p = e.pergunta.as_ref().unwrap();
        assert_eq!((p.texto.as_str(), p.cursor), ("muito **importante**", 20));
        tecla(&mut e, "Ctrl+b");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "muito importante");
        // Sem palavra: o par vazio, cursor no meio.
        digitar(&mut e, " ");
        tecla(&mut e, "Ctrl+b");
        digitar(&mut e, "x");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "muito importante **x**");
    }

    #[test]
    fn ctrl_t_abre_o_menu_formatar_com_link_e_cor() {
        let mut e = markdown_editavel();
        e.cursor = vec![1];
        tecla(&mut e, "o");
        digitar(&mut e, "veja docs");
        tecla(&mut e, "Ctrl+t");
        assert!(e.pergunta.is_none());
        let tela = desenho(&mut e, 100, 40).join("\n");
        assert!(tela.contains("Formatar") && tela.contains("Itálico") && tela.contains("Cor: Vermelho"), "{tela}");
        digitar(&mut e, "link");
        tecla(&mut e, "Enter");
        digitar(&mut e, "https://x.dev");
        tecla(&mut e, "Enter");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "veja [docs](https://x.dev)");
        // Cor pela paleta; Esc no menu devolve a inserção intacta.
        tecla(&mut e, "Ctrl+t");
        tecla(&mut e, "Escape");
        assert_eq!(e.pergunta.as_ref().unwrap().texto, "veja [docs](https://x.dev)");
        digitar(&mut e, " alerta");
        tecla(&mut e, "Ctrl+t");
        digitar(&mut e, "cor: verm");
        tecla(&mut e, "Enter");
        tecla(&mut e, "Escape");
        assert!(corpo_gravado(&e).contains("veja [docs](https://x.dev) <span class=\"cor--vermelho\">alerta</span>"), "{}", corpo_gravado(&e));
    }

    #[test]
    fn texto_com_cor_da_paleta_sai_colorido_e_riscado_sai_riscado() {
        let mut e = pagina_com("um <span class=\"cor--vermelho\">alerta</span> e ~~velho~~\n");
        let vermelho = e.tema.var("cor-vermelho");
        assert_ne!(vermelho, ratatui::style::Color::Gray, "o tema não tem --cor-vermelho");
        let buf = quadro(&mut e, 80, 6);
        let achar = |palavra: &str| {
            (0..buf.area.height).find_map(|y| {
                let linha: String = (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect();
                linha.find(palavra).map(|b| (linha[..b].chars().count() as u16, y))
            })
        };
        let (x, y) = achar("alerta").expect("sem alerta");
        assert_eq!(buf[(x, y)].style().fg, Some(vermelho));
        let (x, y) = achar("velho").expect("sem velho");
        assert!(buf[(x, y)].style().add_modifier.contains(Modifier::CROSSED_OUT));
        assert!(!desenho(&mut e, 80, 6).join("\n").contains("span"));
    }

    // --- Ciclo 358: aparência ------------------------------------------------

    #[test]
    fn cor_de_destaque_e_estilo_dos_botoes_valem_e_gravam() {
        let mut e = Estado::novo(paginas(), analisar("{{ type: \"actions\" }}\nbuttons:\n- label: Abrir\n  action: open-page\n  path: pages/beta.md\n{{ /actions }}\n"));
        e.foco = Foco::Conteudo;
        let azul_do_tema = e.tema.var("accent-blue");
        modais::executar(&mut e, "destaque");
        let tela = desenho(&mut e, 100, 20).join("\n");
        assert!(tela.contains("Cor de destaque") && tela.contains("Do tema") && tela.contains("Rosa"), "{tela}");
        for _ in 0..5 {
            tecla(&mut e, "j");
        }
        tecla(&mut e, "Enter");
        assert_eq!(e.preferencias.destaque, "rosa");
        assert_eq!(e.tema.var("accent-blue"), ratatui::style::Color::Rgb(0xE7, 0x41, 0x8C));
        assert_ne!(e.tema.var("accent-blue"), azul_do_tema);
        assert!(e.pedidos.contains(&Pedido::GravarPreferencias));
        // Trocar o tema mantém o destaque.
        modais::trocar_tema(&mut e, "papel");
        assert_eq!(e.tema.var("accent-blue"), ratatui::style::Color::Rgb(0xE7, 0x41, 0x8C));
        // Botões retos: o contorno troca de desenho.
        assert!(desenho(&mut e, 100, 20).join("\n").contains("▗"));
        modais::executar(&mut e, "botoes");
        tecla(&mut e, "j");
        tecla(&mut e, "Enter");
        assert_eq!(e.preferencias.botoes, "reto");
        let tela = desenho(&mut e, 100, 20).join("\n");
        assert!(tela.contains("┌") && tela.contains("Abrir"), "{tela}");
    }

    // --- Ciclo 359: remapear teclas -------------------------------------------

    #[test]
    fn tecla_remapeada_faz_a_acao_e_a_antiga_deixa_de_fazer() {
        let mut e = markdown_editavel();
        e.cursor = vec![0];
        e.preferencias.teclas_vim.insert("down".into(), "n".into());
        e.preferencias.teclas_globais.insert("alternar-tema".into(), "Ctrl+t".into());
        tecla(&mut e, "n");
        assert_eq!(e.cursor, vec![1], "n desce");
        tecla(&mut e, "j");
        assert_eq!(e.cursor, vec![1], "j não desce mais");
        let tema = e.preferencias.tema.clone();
        tecla(&mut e, "Ctrl+t");
        assert_ne!(e.preferencias.tema, tema, "Ctrl+T alternou o tema");
        // Inserindo, n é texto.
        tecla(&mut e, "A");
        tecla(&mut e, "n");
        assert!(e.pergunta.as_ref().unwrap().texto.ends_with('n'));
    }

    #[test]
    fn o_formulario_de_teclas_valida_repeticao_e_grava() {
        let mut e = Estado::novo(paginas(), analisar("# a\n"));
        modais::executar(&mut e, "remapear-teclas");
        let tela = desenho(&mut e, 100, 40).join("\n");
        assert!(tela.contains("Remapear teclas") && tela.contains("Vim · Descer"), "{tela}");
        let Some(Modal::Detalhe { form, .. }) = e.modal.as_mut() else { panic!() };
        let i = form.campos.iter().position(|c| c.chave == "down").unwrap();
        form.campos[i].valor = crate::componentes::Valor::Texto("k".into());
        let Some(Modal::Detalhe { form, .. }) = e.modal.clone() else { panic!() };
        let mut e2 = Estado::novo(paginas(), analisar("# a\n"));
        assert!(!teclas::salvar(&mut e2, &form));
        assert!(e2.aviso.as_deref().unwrap().contains("\"k\""), "{:?}", e2.aviso);
        let mut form = form;
        let i = form.campos.iter().position(|c| c.chave == "down").unwrap();
        form.campos[i].valor = crate::componentes::Valor::Texto("n".into());
        let g = form.campos.iter().position(|c| c.chave == "hoje").unwrap();
        form.campos[g].valor = crate::componentes::Valor::Texto("Ctrl+d".into());
        assert!(teclas::salvar(&mut e2, &form));
        assert_eq!(e2.preferencias.teclas_vim.get("down").map(String::as_str), Some("n"));
        assert_eq!(e2.preferencias.teclas_globais.get("hoje").map(String::as_str), Some("Ctrl+d"));
        assert_eq!(e2.preferencias.teclas_vim.len(), 1, "só o que mudou é gravado");
        tecla(&mut e2, "Ctrl+d");
        assert!(e2.pedidos.contains(&Pedido::AbrirHoje));
    }

    // --- Ciclo 360: cabeçalho da página tipada ---------------------------------

    #[test]
    fn pagina_tipada_mostra_o_titulo_e_igual_abre_as_propriedades() {
        let mut e = especial_aberto("tags");
        especiais::carregar(&mut e, especiais::tags_do_indice(&[]));
        let tela = desenho(&mut e, 100, 20).join("\n");
        assert!(tela.contains("alfa") && tela.contains("Propriedades"), "{tela}");
        tecla(&mut e, "=");
        assert!(matches!(e.modal, Some(Modal::Detalhe { alvo: modais::AlvoDoDetalhe::Propriedades, .. })));
        // Propostas não tem cabeçalho de página.
        let mut e = especial_aberto("propostas");
        especiais::carregar(&mut e, especiais::Dados::Propostas(vec![]));
        assert!(!desenho(&mut e, 100, 20).join("\n").contains("Propriedades"));
    }

    // --- Ciclo 361: grafo de conexões ------------------------------------------

    #[test]
    fn grafo_lista_as_conexoes_nos_dois_sentidos_e_abre() {
        let pagina = |path: &str, title: &str, links: &[&str]| anotadinho_core::index::PageIndexEntry {
            path: path.into(),
            title: title.into(),
            wikilinks: links.iter().map(|t| t.to_string()).collect(),
            ..Default::default()
        };
        let especiais::Dados::Grafo(nos) = especiais::grafo_do_indice(&[
            pagina("pages/a.md", "A", &["B", "b", "C"]),
            pagina("pages/b.md", "B", &["A"]),
            pagina("pages/c.md", "C", &[]),
            pagina("pages/d.md", "D", &["nada"]),
        ]) else {
            panic!()
        };
        let resumo: Vec<(String, usize)> = nos.iter().map(|n| (n.1.clone(), n.2.len())).collect();
        assert_eq!(resumo, [("A".into(), 2), ("B".into(), 1), ("C".into(), 1), ("D".into(), 0)]);
        let mut e = especial_aberto("graph");
        assert_eq!(e.pedidos, [Pedido::CarregarEspecial(especiais::TipoEspecial::Grafo)]);
        e.pedidos.clear();
        especiais::carregar(&mut e, especiais::Dados::Grafo(nos));
        let tela = desenho(&mut e, 100, 30).join("\n");
        assert!(tela.contains("Grafo de conexões") && tela.contains("4 páginas · 2 ligações") && tela.contains("↔ C"), "{tela}");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/a.md".into())]);
        e.pedidos.clear();
        tecla(&mut e, "l");
        tecla(&mut e, "l");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/c.md".into())]);
    }

    // --- Ciclo 362: início e exportar HTML -----------------------------------

    #[test]
    fn pagina_de_inicio_abre_primeiro_fica_na_frente_e_nao_fecha() {
        let mut e = Estado::novo(paginas(), analisar("# g\n")).com_inicio(Some("pages/gama.md".into())).com_texto("# g\n", None);
        assert_eq!(e.pagina, 2);
        e.pagina = 0;
        e.abrir_texto("# a\n", None);
        e.pagina = 1;
        e.abrir_texto("# b\n", None);
        assert_eq!(e.abas, ["pages/gama.md", "pages/alfa.md", "pages/beta.md"]);
        let tela = desenho(&mut e, 120, 10).join("\n");
        assert!(tela.contains("⌂ gama") && tela.contains("2 alfa"), "{tela}");
        tecla(&mut e, "Alt+1");
        assert_eq!(tecla(&mut e, "Alt+q"), None);
        assert!(e.aviso.as_deref().unwrap().contains("início"));
        // Definir outra como início a leva pra frente.
        e.inicio = Some("pages/beta.md".into());
        e.organizar_abas();
        assert_eq!(e.abas, ["pages/beta.md", "pages/gama.md", "pages/alfa.md"]);
        tecla(&mut e, ":");
        digitar(&mut e, "exportar html");
        tecla(&mut e, "Enter");
        assert!(matches!(e.pedidos.last(), Some(Pedido::ExportarHtml(_))));
        tecla(&mut e, ":");
        digitar(&mut e, "como inicio");
        tecla(&mut e, "Enter");
        assert!(matches!(e.pedidos.last(), Some(Pedido::AlternarInicio(_))));
    }

    // --- Ciclo 363: conflito com o disco --------------------------------------

    #[test]
    fn conflito_oferece_ver_diferenca_manter_ou_recarregar() {
        let mut e = markdown_editavel();
        e.modal = Some(Modal::Conflito(modais::Conflito {
            path: "pages/alfa.md".into(),
            meu: "# Título\n\nmeu texto\n".into(),
            disco: "# Título\n\ntexto de fora\n".into(),
            opcao: 0,
            diff: false,
            rolagem: 0,
        }));
        let tela = desenho(&mut e, 120, 30).join("\n");
        assert!(tela.contains("mudou no disco") && tela.contains("Manter o meu") && tela.contains("Recarregar"), "{tela}");
        tecla(&mut e, "Enter");
        let tela = desenho(&mut e, 120, 30).join("\n");
        assert!(tela.contains("-texto de fora") && tela.contains("+meu texto"), "{tela}");
        // Enquanto decide, o disco não relê por cima.
        assert!(!e.recarregar_do_disco("# outro\n", Some("v9".into())));
        tecla(&mut e, "l");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::GravarPorCima { path: "pages/alfa.md".into(), conteudo: "# Título\n\nmeu texto\n".into() }]);
        assert!(e.modal.is_none());
    }

    // --- Ciclo 366: agendar item sem data do cronograma --------------------------

    #[test]
    fn enter_no_item_sem_data_agenda_no_comeco_do_periodo() {
        use anotadinho_core::embed::EmbedData;
        let mut e = pagina_com("{{ type: \"timeline\" }}\nscale: month\nitems:\n- title: Com data\n  start: '2026-08-10'\n  end: '2026-08-12'\n- title: Solta\n{{ /timeline }}\n").com_hoje("2026-08-20");
        e.foco = Foco::Conteudo;
        let solta = e
            .arvore
            .percorrer()
            .into_iter()
            .find(|(_, u)| u.texto == "Solta" && matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "item"))
            .map(|(c, _)| c)
            .expect("sem a gaveta");
        e.cursor = solta;
        tecla(&mut e, "Enter");
        let EmbedData::Timeline(d) = embed_gravado(&e) else { panic!() };
        let esperado = anotadinho_core::date_util::add_days("2026-08-20", -(d.scale.days() / 4)).unwrap();
        assert_eq!(d.items[1].start.as_deref(), Some(esperado.as_str()));
        assert!(e.aviso.as_deref().unwrap().contains("agendado"));
    }

    // --- Ciclo 367: células de número, data e página ---------------------------

    #[test]
    fn celulas_de_numero_data_e_pagina_como_na_janela() {
        use anotadinho_core::embed::EmbedData;
        let mut e = pagina_com("{{ type: \"table\" }}\ncolumns:\n  - name: Horas\n    type: number\n  - name: Prazo\n    type: date\n  - name: Doc\n    type: page\n---\n| Horas | Prazo | Doc |\n| --- | --- | --- |\n| 3 | 2026-08-10 |  |\n{{ /table }}\n").com_hoje("2026-08-20");
        e.foco = Foco::Conteudo;
        let EmbedData::Table(d) = anotadinho_core::embed::segment(e.texto_da_pagina.as_deref().unwrap()).into_iter().find_map(|s| match s { anotadinho_core::embed::DocSegment::Embed(d) => Some(d), _ => None }).unwrap() else { panic!() };
        assert!(matches!(d.columns[2].kind, anotadinho_core::embed::ColumnKind::PageLink), "{:?}", d.columns);
        e.cursor = vec![0, 1, 0];
        tecla(&mut e, "2");
        tecla(&mut e, "Ctrl+a");
        let EmbedData::Table(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.rows[0][0], "5");
        e.cursor = vec![0, 1, 1];
        tecla(&mut e, "Ctrl+x");
        let EmbedData::Table(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.rows[0][1], "2026-08-09");
        // Página vazia: Enter escolhe; cheia: Enter abre.
        e.cursor = vec![0, 1, 2];
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Escolha { .. })));
        digitar(&mut e, "beta");
        tecla(&mut e, "Enter");
        let EmbedData::Table(d) = embed_gravado(&e) else { panic!() };
        assert_eq!(d.rows[0][2], "pages/beta.md");
        assert!(desenho(&mut e, 120, 12).join("\n").contains("↗ beta"));
        e.cursor = vec![0, 1, 2];
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos.last(), Some(&Pedido::AbrirPagina("pages/beta.md".into())));
    }

    // --- Ciclo 368: evento sem data ---------------------------------------------

    #[test]
    fn gaveta_sem_data_cria_e_abre_evento_sem_data() {
        use anotadinho_core::embed::EmbedData;
        let mut e = pagina_com("{{ type: \"calendar\" }}\nentries:\n- date: 2026-08-10\n  title: Reunião\n{{ /calendar }}\n").com_hoje("2026-08-12");
        e.foco = Foco::Conteudo;
        let t = desenho(&mut e, 120, 40).join("\n");
        assert!(t.contains("Sem data (0)") && t.contains("+ evento sem data"), "{t}");
        let vazio = e
            .arvore
            .percorrer()
            .into_iter()
            .find(|(_, u)| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "sem-data"))
            .map(|(c, _)| c)
            .expect("sem gaveta");
        e.cursor = vazio;
        tecla(&mut e, "o");
        assert!(e.pergunta.as_ref().unwrap().rotulo.contains("sem data"));
        digitar(&mut e, "Ideia");
        tecla(&mut e, "Escape");
        let EmbedData::Calendar(d) = embed_gravado(&e) else { panic!() };
        assert_eq!((d.entries[1].title.as_str(), d.entries[1].date.as_deref()), ("Ideia", None));
        assert_eq!(e.arvore.em(&e.cursor).unwrap().texto, "Ideia");
        tecla(&mut e, "Enter");
        assert!(matches!(e.modal, Some(Modal::Detalhe { alvo: modais::AlvoDoDetalhe::Evento { indice: 1, .. }, .. })));
    }

    // --- Ciclo 369: pastas do agente na conversa -------------------------------

    #[test]
    fn a_conversa_mostra_e_troca_as_pastas_do_agente() {
        let mut e = conversa_aberta();
        let mut a = anotadinho_core::agente::Adaptador::default();
        a.arg_pasta_extra = "--add-dir".into();
        a.pastas_extras = vec!["/home/x/outro-repo".into()];
        e.preferencias.agente = Some(a);
        let tela = desenho(&mut e, 160, 40).join("\n");
        assert!(tela.contains("▭ trabalha na raiz do projeto") && tela.contains("outro-repo ×") && tela.contains("+ pasta"), "{tela}");
        tecla(&mut e, ":");
        digitar(&mut e, "pasta de trabalho");
        tecla(&mut e, "Enter");
        digitar(&mut e, "/tmp");
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos.last(), Some(&Pedido::PastaDoAgente { pasta: "/tmp".into(), extra: false }));
        tecla(&mut e, ":");
        digitar(&mut e, "tirar pasta");
        tecla(&mut e, "Enter");
        tecla(&mut e, "Enter");
        assert!(e.preferencias.agente.as_ref().unwrap().pastas_extras.is_empty());
    }

    // --- Ciclo 370: busca no conteúdo pela sidebar ----------------------------

    #[test]
    fn a_busca_da_sidebar_mostra_resultados_do_conteudo() {
        let mut e = Estado::novo(paginas(), analisar("# a\n"));
        tecla(&mut e, "/");
        digitar(&mut e, "sp");
        assert!(e.pedidos.is_empty(), "duas letras ainda não buscam no conteúdo");
        digitar(&mut e, "r");
        assert_eq!(e.pedidos.last(), Some(&Pedido::BuscarNaSidebar("spr".into())));
        e.resultados_da_busca = Some((
            "spr".into(),
            vec![anotadinho_core::embed::SearchHit { path: "pages/gama.md".into(), snippet: "a **spr**int de agosto".into(), origem: None, ancora: None }],
        ));
        let tela = desenho(&mut e, 120, 12).join("\n");
        assert!(tela.contains("⌕ gama · a sprint de agosto"), "{tela}");
        tecla(&mut e, "Enter");
        tecla(&mut e, "j");
        assert_eq!(e.pagina, 2, "descer até o resultado seleciona a página dele");
        // Outro termo: o resultado velho some.
        tecla(&mut e, "/");
        digitar(&mut e, "xyz");
        assert!(!desenho(&mut e, 120, 12).join("\n").contains("⌕ gama"));
    }

    #[test]
    fn abrir_pelo_resultado_leva_o_cursor_ao_trecho() {
        let mut e = Estado::novo(paginas(), analisar("# a\n"));
        modais::mostrar_resultados_da_busca(
            &mut e,
            "sprint",
            &[anotadinho_core::embed::SearchHit { path: "pages/beta.md".into(), snippet: "a **sprint**".into(), origem: None, ancora: None }],
        );
        tecla(&mut e, "Enter");
        assert_eq!(e.pedidos, [Pedido::AbrirPagina("pages/beta.md".into())]);
        e.pagina = 1;
        e.abrir_texto("# Beta\n\nIntro.\n\n## Planos\n\n- revisar a Sprint de agosto\n", None);
        assert!(e.arvore.em(&e.cursor).unwrap().texto.contains("Sprint"), "{:?}", e.cursor);
        assert_eq!(e.foco, Foco::Conteudo);
        // A próxima página abre normal.
        e.abrir_texto("# Outra\n\nSprint aqui também.\n", None);
        assert_eq!(e.cursor, tela::primeiro(&e.arvore).unwrap_or_default());
    }
}
