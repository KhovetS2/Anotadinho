//! As páginas de tipo que não são documento (ciclo 346): `type: tags`,
//! `type: assets` e `type: propostas`, como a `TagsPage`, a `AssetsPage` e
//! a `PropostasView` da janela.
//!
//! Elas mostram o VAULT, não o arquivo: as tags dos embeds e as páginas
//! onde aparecem, os arquivos de `assets/` e se alguma página usa, as
//! propostas que o agente deixou pra revisar. Ler isso é I/O, então a tela
//! abre "Carregando..." e pede ao `main` os dados ([`Pedido::CarregarEspecial`]).
//!
//! Teclas: `j`/`k` passam pelos itens; nas tags, `h`/`l` passam pelas
//! páginas e `Enter` abre; nos assets, `x` (ou `dd`) exclui; nas propostas,
//! `a` aplica, `r` recusa, `v` troca Diff/Visualização e `Ctrl+D`/`Ctrl+U`
//! rolam uma proposta comprida. `R` recarrega.

use std::collections::BTreeSet;

use anotadinho_core::diff::LinhaDiff;
use anotadinho_core::proposta::{Operacao, Proposta};
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::modais::{Modal, Pedido};
use super::{Estado, Foco};
use crate::tema::{misturar, Realce, Tema};

/// Qual das telas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TipoEspecial {
    Tags,
    Assets,
    Propostas,
}

impl TipoEspecial {
    /// O tipo pelo `type:` do frontmatter.
    pub fn do_frontmatter(frontmatter: &str) -> Option<Self> {
        let tipo = frontmatter
            .lines()
            .find_map(|l| l.strip_prefix("type:"))
            .map(|v| v.trim().trim_matches('"').trim_matches('\''))?;
        match tipo {
            "tags" => Some(Self::Tags),
            "assets" => Some(Self::Assets),
            "propostas" => Some(Self::Propostas),
            _ => None,
        }
    }

    /// Onde a página fica quando a barra de comandos precisa criar.
    pub fn pagina(self) -> (&'static str, &'static str) {
        match self {
            Self::Tags => ("pages/tags.md", "Tags"),
            Self::Assets => ("pages/assets.md", "Assets"),
            Self::Propostas => ("pages/propostas.md", "Propostas"),
        }
    }
}

/// Um arquivo de `assets/`.
#[derive(Debug, Clone, PartialEq)]
pub struct Asset {
    pub path: String,
    pub tamanho: u64,
    /// Alguma página cita o caminho ou o nome do arquivo.
    pub usado: bool,
}

/// Uma proposta e o conteúdo atual da página alvo.
#[derive(Debug, Clone, PartialEq)]
pub struct PropostaNaTela {
    pub proposta: Proposta,
    pub atual: String,
}

/// O que o `main` leu do vault.
#[derive(Debug, Clone, PartialEq)]
pub enum Dados {
    /// Tag → (caminho, título) das páginas.
    Tags(Vec<(String, Vec<(String, String)>)>),
    Assets(Vec<Asset>),
    Propostas(Vec<PropostaNaTela>),
}

/// O estado da tela.
#[derive(Debug, Clone, PartialEq)]
pub struct TelaEspecial {
    pub tipo: TipoEspecial,
    /// `None` enquanto carrega.
    pub dados: Option<Dados>,
    pub selecionado: usize,
    /// A página escolhida dentro da tag.
    pub chip: usize,
    /// As propostas em modo Visualização.
    pub visualizando: BTreeSet<String>,
    /// Rolagem dentro do item selecionado.
    pub deslocamento: usize,
    /// O erro da última ação.
    pub erro: Option<String>,
    /// Um `d` esperando o segundo.
    d_pendente: bool,
}

impl TelaEspecial {
    pub fn nova(tipo: TipoEspecial) -> Self {
        Self {
            tipo,
            dados: None,
            selecionado: 0,
            chip: 0,
            visualizando: BTreeSet::new(),
            deslocamento: 0,
            erro: None,
            d_pendente: false,
        }
    }

    fn total(&self) -> usize {
        match &self.dados {
            Some(Dados::Tags(t)) => t.len(),
            Some(Dados::Assets(a)) => a.len(),
            Some(Dados::Propostas(p)) => p.len(),
            None => 0,
        }
    }
}

/// Monta a lista de tags a partir do índice do vault, como
/// `scan_vault_tags` da janela.
pub fn tags_do_indice(indice: &[anotadinho_core::index::PageIndexEntry]) -> Dados {
    let mut mapa: std::collections::BTreeMap<String, Vec<(String, String)>> = Default::default();
    for p in indice {
        for t in &p.embed_tags {
            mapa.entry(t.clone()).or_default().push((p.path.clone(), p.title.clone()));
        }
    }
    Dados::Tags(mapa.into_iter().collect())
}

/// Decide "usado" pra cada asset olhando o texto de todas as páginas.
pub fn assets_com_uso(lista: Vec<(String, u64)>, paginas: &str) -> Dados {
    Dados::Assets(
        lista
            .into_iter()
            .map(|(path, tamanho)| {
                let nome = nome_do_arquivo(&path);
                let usado = paginas.contains(&path) || (!nome.is_empty() && paginas.contains(&nome));
                Asset { path, tamanho, usado }
            })
            .collect(),
    )
}

/// Os dados chegaram.
pub fn carregar(e: &mut Estado, dados: Dados) {
    let Some(t) = e.especial.as_mut() else { return };
    t.dados = Some(dados);
    let total = t.total();
    t.selecionado = t.selecionado.min(total.saturating_sub(1));
    t.chip = 0;
    t.deslocamento = 0;
}

fn nome_do_arquivo(path: &str) -> String {
    std::path::Path::new(path).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_default()
}

/// `1.2 KB`, como `format_size` da janela.
pub fn tamanho_legivel(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

/// Uma tecla na tela. Devolve se foi usada.
pub fn tecla(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let total = t.total();
    let d_antes = std::mem::take(&mut t.d_pendente);
    match tecla {
        "j" | "ArrowDown" => {
            t.selecionado = (t.selecionado + 1).min(total.saturating_sub(1));
            t.chip = 0;
            t.deslocamento = 0;
        }
        "k" | "ArrowUp" => {
            t.selecionado = t.selecionado.saturating_sub(1);
            t.chip = 0;
            t.deslocamento = 0;
        }
        "g" | "Home" => {
            t.selecionado = 0;
            t.deslocamento = 0;
        }
        "G" | "End" => {
            t.selecionado = total.saturating_sub(1);
            t.deslocamento = 0;
        }
        "Ctrl+d" | "PageDown" => t.deslocamento += 10,
        "Ctrl+u" | "PageUp" => t.deslocamento = t.deslocamento.saturating_sub(10),
        "R" => {
            let tipo = t.tipo;
            e.pedidos.push(Pedido::CarregarEspecial(tipo));
        }
        _ => {
            let tipo = t.tipo;
            return match tipo {
                TipoEspecial::Tags => tecla_nas_tags(e, tecla),
                TipoEspecial::Assets => tecla_nos_assets(e, tecla, d_antes),
                TipoEspecial::Propostas => tecla_nas_propostas(e, tecla),
            };
        }
    }
    true
}

fn tecla_nas_tags(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Tags(tags)) = &t.dados else { return false };
    let Some((_, paginas)) = tags.get(t.selecionado) else { return false };
    match tecla {
        "l" | "ArrowRight" | "w" => t.chip = (t.chip + 1).min(paginas.len().saturating_sub(1)),
        "h" | "ArrowLeft" | "b" => t.chip = t.chip.saturating_sub(1),
        "Enter" => {
            if let Some((path, _)) = paginas.get(t.chip) {
                e.pedidos.push(Pedido::AbrirPagina(path.clone()));
            }
        }
        _ => return false,
    }
    true
}

fn tecla_nos_assets(e: &mut Estado, tecla: &str, d_antes: bool) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Assets(assets)) = &t.dados else { return false };
    let Some(a) = assets.get(t.selecionado) else { return false };
    match tecla {
        "d" if !d_antes => t.d_pendente = true,
        "x" | "Delete" | "d" => {
            e.modal = Some(Modal::Confirmar {
                titulo: "Excluir asset".into(),
                mensagem: format!("Excluir \"{}\"? Isso não pode ser desfeito.", nome_do_arquivo(&a.path)),
                acao: Pedido::ExcluirAsset(a.path.clone()),
            });
        }
        _ => return false,
    }
    true
}

fn tecla_nas_propostas(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Propostas(lista)) = &t.dados else { return false };
    let Some(p) = lista.get(t.selecionado).map(|p| p.proposta.clone()) else { return false };
    match tecla {
        "v" => {
            if !t.visualizando.remove(&p.id) {
                t.visualizando.insert(p.id.clone());
            }
            t.deslocamento = 0;
        }
        "a" | "Enter" => {
            e.modal = Some(Modal::Confirmar {
                titulo: "Aplicar proposta".into(),
                mensagem: format!(
                    "{} {} com o conteúdo proposto por {}?",
                    if p.operacao == Operacao::Criar { "Criar" } else { "Substituir" },
                    p.alvo,
                    p.autor
                ),
                acao: Pedido::DecidirProposta { id: p.id.clone(), aplicar: true },
            });
        }
        "r" | "x" => {
            e.modal = Some(Modal::Confirmar {
                titulo: "Recusar proposta".into(),
                mensagem: format!("Descartar a proposta pra {}? Ela sai da fila.", p.alvo),
                acao: Pedido::DecidirProposta { id: p.id.clone(), aplicar: false },
            });
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Desenho
// ---------------------------------------------------------------------

/// Desenha a tela no painel de conteúdo.
pub fn desenhar(f: &mut Frame, e: &Estado, area: Rect) {
    let Some(t) = &e.especial else { return };
    let tema = &e.tema;
    let no_foco = e.foco == Foco::Conteudo;
    let dica = match t.tipo {
        TipoEspecial::Tags => " j k tag · h l página · Enter abre · R recarrega ",
        TipoEspecial::Assets => " j k · x exclui · R recarrega ",
        TipoEspecial::Propostas => " j k · a aplica · r recusa · v diff/visualização · Ctrl+D rola ",
    };
    let mut borda = Block::default()
        .borders(Borders::ALL)
        .border_style(if no_foco { tema.estilo(Realce::BordaFoco) } else { tema.estilo(Realce::Borda) })
        .style(tema.estilo(Realce::Fundo))
        .title_bottom(Line::from(Span::styled(dica, tema.estilo(Realce::Dica))));
    if let Some(aviso) = e.aviso.as_ref().or(t.erro.as_ref()) {
        borda = borda.title_bottom(Line::from(Span::styled(format!(" {aviso} "), tema.estilo(Realce::BadgeAtencao))).right_aligned());
    }
    let dentro = borda.inner(area);
    f.render_widget(borda, area);
    let w = dentro.width as usize;
    if w < 20 || dentro.height < 4 {
        return;
    }
    let (linhas, faixas) = linhas_da_tela(t, tema, w, no_foco);
    let altura = dentro.height as usize;
    let topo = match faixas.get(t.selecionado) {
        Some(&(_, fim)) if fim <= altura => 0,
        Some(&(inicio, fim)) => (inicio.saturating_sub(1) + t.deslocamento).min(fim.saturating_sub(1)),
        None => 0,
    }
    .min(linhas.len().saturating_sub(altura));
    let visiveis: Vec<Line<'static>> = linhas.into_iter().skip(topo).take(altura).collect();
    f.render_widget(Paragraph::new(visiveis), dentro);
}

/// As linhas da tela e, pra cada item, a faixa de linhas `[inicio, fim)`
/// que ele ocupa — a rolagem segue o item selecionado.
pub fn linhas_da_tela(t: &TelaEspecial, tema: &Tema, w: usize, no_foco: bool) -> (Vec<Line<'static>>, Vec<(usize, usize)>) {
    let texto = Style::default().fg(tema.var("text-primary"));
    let apagado = Style::default().fg(tema.var("text-muted"));
    let mut fora: Vec<Line<'static>> = Vec::new();
    let mut faixas = Vec::new();
    let (titulo, direita) = match (&t.tipo, &t.dados) {
        (TipoEspecial::Tags, _) => ("Tags", String::new()),
        (TipoEspecial::Assets, _) => ("Assets", String::new()),
        (TipoEspecial::Propostas, Some(Dados::Propostas(p))) => ("Propostas do agente", format!("{} pendente(s) ", p.len())),
        (TipoEspecial::Propostas, _) => ("Propostas do agente", String::new()),
    };
    let esquerda = format!(" {titulo}");
    fora.push(Line::from(vec![
        Span::styled(esquerda.clone(), texto.add_modifier(Modifier::BOLD)),
        Span::raw(" ".repeat(w.saturating_sub(esquerda.chars().count() + direita.chars().count()))),
        Span::styled(direita, apagado),
    ]));
    let Some(dados) = &t.dados else {
        fora.push(Line::default());
        fora.push(Line::from(Span::styled(" Carregando...", apagado)));
        return (fora, faixas);
    };
    match dados {
        Dados::Tags(tags) => {
            fora.push(Line::default());
            if tags.is_empty() {
                fora.extend(paragrafo(
                    "Nenhuma tag encontrada. Adicione tags em cards de kanban ou eventos de calendário.",
                    apagado,
                    w,
                ));
            }
            let nomes: Vec<String> = tags.iter().map(|(n, _)| n.clone()).collect();
            for (i, (tag, paginas)) in tags.iter().enumerate() {
                let sel = no_foco && i == t.selecionado;
                let inicio = fora.len();
                fora.extend(grupo_de_tag(tema, &nomes, tag, paginas, sel.then_some(t.chip), i == t.selecionado, w));
                faixas.push((inicio, fora.len()));
            }
        }
        Dados::Assets(assets) => {
            let total: u64 = assets.iter().map(|a| a.tamanho).sum();
            let nao_usados = assets.iter().filter(|a| !a.usado).count();
            fora.push(Line::from(Span::styled(
                format!(" {} arquivos · {} · {} não referenciados", assets.len(), tamanho_legivel(total), nao_usados),
                apagado,
            )));
            fora.push(Line::default());
            if assets.is_empty() {
                fora.push(Line::from(Span::styled(" Nenhum arquivo em assets/ ainda.", apagado)));
            } else {
                fora.extend(tabela_de_assets(t, tema, assets, w, no_foco, &mut faixas));
            }
        }
        Dados::Propostas(lista) => {
            fora.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(tema.var("border")))));
            if lista.is_empty() {
                fora.extend(paragrafo(
                    "Nada pendente. O agente grava aqui em vez de escrever no vault; o que ele propuser aparece nesta tela pra você aprovar.",
                    apagado,
                    w,
                ));
            }
            for (i, p) in lista.iter().enumerate() {
                let inicio = fora.len();
                let visualizar = t.visualizando.contains(&p.proposta.id);
                fora.extend(cartao_da_proposta(tema, p, visualizar, i == t.selecionado, no_foco, w));
                faixas.push((inicio, fora.len()));
            }
        }
    }
    (fora, faixas)
}

fn paragrafo(texto: &str, estilo: Style, w: usize) -> Vec<Line<'static>> {
    super::quebrar_texto(Line::from(Span::styled(format!(" {texto}"), estilo)), w.saturating_sub(1), 1)
}

/// Uma caixa com contorno arredondado em volta de `miolo`, com fundo de
/// superfície — o `.tags-page__group` e o `.propostas__item`.
fn caixa(tema: &Tema, miolo: Vec<Line<'static>>, w: usize, cor_da_borda: Color) -> Vec<Line<'static>> {
    let fundo = tema.var("bg-surface");
    let borda = Style::default().fg(cor_da_borda).bg(fundo);
    let dentro = w.saturating_sub(4);
    let mut fora = vec![Line::from(Span::styled(format!("╭{}╮", "─".repeat(w.saturating_sub(2))), borda))];
    for l in miolo {
        let mut usado = 0;
        let mut spans = vec![Span::styled("│ ", borda)];
        for s in l.spans {
            let resto = dentro.saturating_sub(usado);
            if resto == 0 {
                break;
            }
            let conteudo: String = s.content.chars().take(resto).collect();
            usado += conteudo.chars().count();
            let estilo = if s.style.bg.is_none() { s.style.bg(fundo) } else { s.style };
            spans.push(Span::styled(conteudo, estilo));
        }
        spans.push(Span::styled(" ".repeat(dentro.saturating_sub(usado)), Style::default().bg(fundo)));
        spans.push(Span::styled(" │", borda));
        fora.push(Line::from(spans));
    }
    fora.push(Line::from(Span::styled(format!("╰{}╯", "─".repeat(w.saturating_sub(2))), borda)));
    fora
}

fn cor_da_borda(tema: &Tema, selecionado: bool, no_foco: bool) -> Color {
    match (selecionado, no_foco) {
        (true, true) => tema.var("accent-blue"),
        (true, false) => misturar(tema.var("accent-blue"), tema.var("bg-base"), 0.45),
        _ => tema.var("border"),
    }
}

/// Pílulas quebrando na largura; `marcada` ganha o estilo de cursor.
fn pilulas(itens: Vec<(String, Style)>, largura: usize) -> Vec<Line<'static>> {
    let mut linhas: Vec<Line<'static>> = Vec::new();
    let mut atual: Vec<Span<'static>> = Vec::new();
    let mut usado = 0;
    for (rotulo, estilo) in itens {
        let n = rotulo.chars().count() + 2;
        if usado > 0 && usado + 1 + n > largura {
            linhas.push(Line::from(std::mem::take(&mut atual)));
            usado = 0;
        }
        if usado > 0 {
            atual.push(Span::raw(" "));
            usado += 1;
        }
        atual.push(Span::styled(format!(" {rotulo} "), estilo));
        usado += n;
    }
    if !atual.is_empty() {
        linhas.push(Line::from(atual));
    }
    linhas
}

fn grupo_de_tag(
    tema: &Tema,
    nomes: &[String],
    tag: &str,
    paginas: &[(String, String)],
    chip: Option<usize>,
    selecionado: bool,
    w: usize,
) -> Vec<Line<'static>> {
    let classe = anotadinho_core::embed::badge_class(nomes, tag);
    let papel = super::papel_do_badge(classe.trim_start_matches("badge")).unwrap_or(Realce::BadgeInfo);
    let mut miolo = vec![Line::from(vec![
        Span::styled(format!(" {tag} "), tema.pilula(papel)),
        Span::raw(" "),
        Span::styled(paginas.len().to_string(), Style::default().fg(tema.var("text-muted"))),
    ])];
    let chip_normal = Style::default().bg(tema.var("bg-elevated")).fg(tema.var("text-primary"));
    miolo.extend(pilulas(
        paginas
            .iter()
            .enumerate()
            .map(|(i, (_, titulo))| {
                (titulo.clone(), if chip == Some(i) { tema.estilo(Realce::Cursor) } else { chip_normal })
            })
            .collect(),
        w.saturating_sub(4),
    ));
    caixa(tema, miolo, w, cor_da_borda(tema, selecionado, chip.is_some()))
}

fn tabela_de_assets(
    t: &TelaEspecial,
    tema: &Tema,
    assets: &[Asset],
    w: usize,
    no_foco: bool,
    faixas: &mut Vec<(usize, usize)>,
) -> Vec<Line<'static>> {
    let apagado = Style::default().fg(tema.var("text-muted"));
    let texto = Style::default().fg(tema.var("text-primary"));
    // Tamanho, uso e o botão têm largura fixa; o arquivo leva o resto.
    let (col_tam, col_uso, col_botao) = (10, 12, 9);
    let col_arq = w.saturating_sub(col_tam + col_uso + col_botao + 1).max(8);
    let celula = |s: &str, n: usize| -> String {
        let mut c: String = s.chars().take(n.saturating_sub(1)).collect();
        let falta = n.saturating_sub(c.chars().count());
        c.push_str(&" ".repeat(falta));
        c
    };
    let mut fora = vec![
        Line::from(vec![
            Span::styled(format!(" {}", celula("Arquivo", col_arq)), apagado.add_modifier(Modifier::BOLD)),
            Span::styled(celula("Tamanho", col_tam), apagado.add_modifier(Modifier::BOLD)),
            Span::styled(celula("Uso", col_uso), apagado.add_modifier(Modifier::BOLD)),
        ]),
        Line::from(Span::styled("─".repeat(w), Style::default().fg(tema.var("border")))),
    ];
    // As faixas contam do começo da tela: título, resumo e vão vêm antes.
    let base = 3 + fora.len();
    for (i, a) in assets.iter().enumerate() {
        let sel = i == t.selecionado;
        let fundo = if sel && no_foco { Some(tema.var("bg-elevated")) } else { None };
        let com = |s: Style| if let Some(f) = fundo { s.bg(f) } else { s };
        let (rotulo, papel) = if a.usado { ("usado", Realce::BadgeSucesso) } else { ("não usado", Realce::BadgeAtencao) };
        let marca = if sel && no_foco { "▌" } else { " " };
        let mut spans = vec![
            Span::styled(marca, com(Style::default().fg(tema.var("accent-blue")))),
            Span::styled(celula(&a.path, col_arq), com(texto)),
            Span::styled(celula(&tamanho_legivel(a.tamanho), col_tam), com(apagado)),
            Span::styled(format!(" {rotulo} "), tema.pilula(papel)),
            Span::styled(" ".repeat(col_uso.saturating_sub(rotulo.chars().count() + 2)), com(Style::default())),
        ];
        let botao = Style::default().fg(tema.var("error"));
        spans.push(Span::styled(
            if sel && no_foco { " Excluir x" } else { " Excluir  " },
            com(if sel && no_foco { botao.add_modifier(Modifier::BOLD) } else { botao }),
        ));
        faixas.push((base + i, base + i + 1));
        fora.push(Line::from(spans));
    }
    fora
}

fn cartao_da_proposta(
    tema: &Tema,
    p: &PropostaNaTela,
    visualizar: bool,
    selecionado: bool,
    no_foco: bool,
    w: usize,
) -> Vec<Line<'static>> {
    let proposta = &p.proposta;
    let apagado = Style::default().fg(tema.var("text-muted"));
    let texto = Style::default().fg(tema.var("text-primary"));
    let dentro = w.saturating_sub(4);
    let (op, papel) = match proposta.operacao {
        Operacao::Criar => ("CRIAR", Realce::BadgeSucesso),
        Operacao::Substituir => ("SUBSTITUIR", Realce::BadgeAtencao),
    };
    let esquerda = vec![
        Span::styled(format!(" {op} "), tema.pilula(papel)),
        Span::raw(" "),
        Span::styled(proposta.alvo.clone(), Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!("ϟ {}", proposta.autor), Style::default().fg(tema.var("accent-purple"))),
    ];
    let usado: usize = esquerda.iter().map(|s| s.content.chars().count()).sum();
    let mut topo = esquerda;
    topo.push(Span::raw(" ".repeat(dentro.saturating_sub(usado + proposta.quando.chars().count()))));
    topo.push(Span::styled(proposta.quando.clone(), apagado));
    let mut miolo = vec![Line::from(topo)];
    if !proposta.motivo.trim().is_empty() {
        miolo.extend(super::quebrar_texto(Line::from(Span::styled(proposta.motivo.clone(), texto)), dentro, 0));
    }
    let linhas = proposta.diff(&p.atual);
    let (removidas, adicionadas) = anotadinho_core::diff::contar(&linhas);
    let resumo = format!("{removidas} linha(s) removida(s) · {adicionadas} adicionada(s)");
    let modo = |rotulo: &str, atual: bool| {
        if atual {
            Span::styled(format!(" {rotulo} "), Style::default().bg(tema.var("bg-elevated")).fg(tema.var("text-primary")).add_modifier(Modifier::BOLD))
        } else {
            Span::styled(format!(" {rotulo} "), apagado)
        }
    };
    let modos = vec![modo("Diff", !visualizar), modo("Visualização", visualizar)];
    let largura_modos: usize = modos.iter().map(|s| s.content.chars().count()).sum();
    let mut linha_modo = vec![Span::styled(resumo.clone(), apagado)];
    linha_modo.push(Span::raw(" ".repeat(dentro.saturating_sub(resumo.chars().count() + largura_modos))));
    linha_modo.extend(modos);
    miolo.push(Line::from(linha_modo));
    if visualizar {
        let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&proposta.conteudo);
        miolo.extend(super::conversa::corpo_da_mensagem(corpo, tema, dentro));
    } else {
        let sai = Style::default().fg(tema.var("text-primary")).bg(misturar(tema.var("error"), tema.var("bg-surface"), 0.16));
        let entra = Style::default().fg(tema.var("text-primary")).bg(misturar(tema.var("success"), tema.var("bg-surface"), 0.16));
        for l in &linhas {
            let (marca, estilo) = match l {
                LinhaDiff::Igual { .. } => (" ", apagado),
                LinhaDiff::Removida { .. } => ("-", sai),
                LinhaDiff::Adicionada { .. } => ("+", entra),
            };
            let conteudo: String = format!("{marca}{}", l.texto()).chars().take(dentro).collect();
            let falta = dentro.saturating_sub(conteudo.chars().count());
            let mut spans = vec![Span::styled(conteudo, estilo)];
            if l.mudou() {
                spans.push(Span::styled(" ".repeat(falta), estilo));
            }
            miolo.push(Line::from(spans));
        }
    }
    miolo.push(Line::default());
    let primario = tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD);
    let fantasma = Style::default().fg(tema.var("text-primary")).bg(tema.var("bg-elevated"));
    miolo.push(Line::from(vec![
        Span::styled(" Aplicar a ", primario),
        Span::raw(" "),
        Span::styled(" Recusar r ", fantasma),
    ]));
    let mut fora = caixa(tema, miolo, w, cor_da_borda(tema, selecionado, no_foco));
    fora.push(Line::default());
    fora
}
