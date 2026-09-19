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
    /// `type: kanban` de página inteira (ciclo 353): os itens `- ` da
    /// própria página com `column::`.
    Kanban,
    /// `type: table` de página inteira: as páginas com `status::` ou
    /// `priority::`.
    Tarefas,
    /// `type: graph` (ciclo 361): as conexões por wikilink, em lista — o
    /// grafo 3D da janela, lido no terminal.
    Grafo,
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
            "kanban" => Some(Self::Kanban),
            "table" => Some(Self::Tarefas),
            "graph" => Some(Self::Grafo),
            _ => None,
        }
    }

    /// Onde a página fica quando a barra de comandos precisa criar.
    pub fn pagina(self) -> (&'static str, &'static str) {
        match self {
            Self::Tags => ("pages/tags.md", "Tags"),
            Self::Assets => ("pages/assets.md", "Assets"),
            Self::Propostas => ("pages/propostas.md", "Propostas"),
            Self::Kanban => ("pages/kanban.md", "Kanban"),
            Self::Tarefas => ("pages/tarefas.md", "Tarefas"),
            Self::Grafo => ("pages/grafo.md", "Grafo"),
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
    /// As colunas (chave, rótulo) e os títulos dos cartões de cada uma.
    Kanban(Vec<(String, String, Vec<String>)>),
    Tarefas(Vec<Tarefa>),
    /// Cada página (caminho, título) e as vizinhas, a mais ligada primeiro.
    Grafo(Vec<(String, String, Vec<(String, String)>)>),
}

/// As conexões do vault, como o `montar` do grafo da janela: um
/// `[[wikilink]]` pro título de outra página liga as duas, nos dois
/// sentidos, uma vez só.
pub fn grafo_do_indice(indice: &[anotadinho_core::index::PageIndexEntry]) -> Dados {
    use std::collections::{BTreeSet, HashMap};
    let por_titulo: HashMap<String, usize> = indice.iter().enumerate().map(|(i, p)| (p.title.to_lowercase(), i)).collect();
    let mut vizinhos: Vec<BTreeSet<usize>> = vec![BTreeSet::new(); indice.len()];
    for (i, p) in indice.iter().enumerate() {
        for alvo in &p.wikilinks {
            if let Some(&j) = por_titulo.get(&alvo.to_lowercase()) {
                if j != i {
                    vizinhos[i].insert(j);
                    vizinhos[j].insert(i);
                }
            }
        }
    }
    let mut nos: Vec<(String, String, Vec<(String, String)>)> = indice
        .iter()
        .enumerate()
        .map(|(i, p)| {
            let v = vizinhos[i].iter().map(|&j| (indice[j].path.clone(), indice[j].title.clone())).collect();
            (p.path.clone(), p.title.clone(), v)
        })
        .collect();
    nos.sort_by(|a, b| b.2.len().cmp(&a.2.len()).then_with(|| a.1.to_lowercase().cmp(&b.1.to_lowercase())));
    Dados::Grafo(nos)
}

/// Uma linha da tabela de tarefas.
#[derive(Debug, Clone, PartialEq)]
pub struct Tarefa {
    pub path: String,
    pub titulo: String,
    pub status: String,
    pub prioridade: String,
}

/// O quadro da página `type: kanban`, como o `Kanban` da janela: cada
/// `- ` é um cartão, `column:: x` diz a coluna (backlog se não diz) e
/// `title:: y` o título; as partes se separam por dois espaços.
pub fn kanban_da_pagina(texto: &str) -> Dados {
    let mut colunas: Vec<(String, String, Vec<String>)> = [
        ("backlog", "Backlog"),
        ("todo", "A Fazer"),
        ("doing", "Fazendo"),
        ("done", "Concluído"),
    ]
    .iter()
    .map(|(c, r)| (c.to_string(), r.to_string(), Vec::new()))
    .collect();
    for linha in texto.lines() {
        let Some(corpo) = linha.trim().strip_prefix("- ") else { continue };
        let (mut coluna, mut titulo) = ("backlog".to_string(), String::new());
        for parte in corpo.split("  ") {
            if let Some(v) = parte.strip_prefix("column:: ") {
                coluna = v.trim().to_string();
            } else if let Some(v) = parte.strip_prefix("title:: ") {
                titulo = v.trim().to_string();
            } else if parte.starts_with("id:: ") {
            } else if titulo.is_empty() {
                titulo = parte.to_string();
            }
        }
        if titulo.is_empty() {
            titulo = corpo.to_string();
        }
        // Coluna fora das quatro não aparece, como na janela.
        if let Some(c) = colunas.iter_mut().find(|c| c.0 == coluna) {
            c.2.push(titulo);
        }
    }
    Dados::Kanban(colunas)
}

/// A tabela de tarefas a partir de `(caminho, título, conteúdo)` das
/// páginas, como o `TaskTable` da janela.
pub fn tarefas_das_paginas(paginas: Vec<(String, String, String)>) -> Dados {
    Dados::Tarefas(
        paginas
            .into_iter()
            .filter_map(|(path, titulo, conteudo)| {
                let (mut status, mut prioridade) = ("-".to_string(), "-".to_string());
                for l in conteudo.lines() {
                    if let Some(v) = l.trim().strip_prefix("status:: ") {
                        status = v.trim().to_string();
                    }
                    if let Some(v) = l.trim().strip_prefix("priority:: ") {
                        prioridade = v.trim().to_string();
                    }
                }
                (status != "-" || prioridade != "-").then_some(Tarefa { path, titulo, status, prioridade })
            })
            .collect(),
    )
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
    /// O trecho do diff sob o cursor, na proposta selecionada (ciclo 404).
    pub trecho: usize,
    /// Os trechos TIRADOS da aplicação, por `id#índice`.
    pub fora: BTreeSet<String>,
    /// As propostas marcadas pra ação em lote (ciclo 409).
    pub marcadas: BTreeSet<String>,
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
            trecho: 0,
            fora: BTreeSet::new(),
            marcadas: BTreeSet::new(),
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
            Some(Dados::Kanban(c)) => c.get(self.chip).map_or(0, |c| c.2.len()),
            Some(Dados::Tarefas(t)) => t.len(),
            Some(Dados::Grafo(g)) => g.len(),
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
    if t.tipo != TipoEspecial::Kanban {
        t.chip = 0;
    }
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
            t.trecho = 0;
            if t.tipo != TipoEspecial::Kanban {
                t.chip = 0;
            }
            t.deslocamento = 0;
        }
        "k" | "ArrowUp" => {
            t.selecionado = t.selecionado.saturating_sub(1);
            t.trecho = 0;
            if t.tipo != TipoEspecial::Kanban {
                t.chip = 0;
            }
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
        "=" if t.tipo != TipoEspecial::Propostas => {
            super::edicao::abrir_propriedades(e);
        }
        _ => {
            let tipo = t.tipo;
            return match tipo {
                TipoEspecial::Tags => tecla_nas_tags(e, tecla),
                TipoEspecial::Assets => tecla_nos_assets(e, tecla, d_antes),
                TipoEspecial::Propostas => tecla_nas_propostas(e, tecla),
                TipoEspecial::Kanban => tecla_no_kanban(e, tecla),
                TipoEspecial::Tarefas => tecla_nas_tarefas(e, tecla),
                TipoEspecial::Grafo => tecla_no_grafo(e, tecla),
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

/// No grafo, `chip` 0 é a própria página e `k` a vizinha `k-1`.
fn tecla_no_grafo(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Grafo(nos)) = &t.dados else { return false };
    let Some((path, _, vizinhos)) = nos.get(t.selecionado) else { return false };
    match tecla {
        "l" | "ArrowRight" | "w" => t.chip = (t.chip + 1).min(vizinhos.len()),
        "h" | "ArrowLeft" | "b" => t.chip = t.chip.saturating_sub(1),
        "Enter" => {
            let alvo = if t.chip == 0 { Some(path) } else { vizinhos.get(t.chip - 1).map(|v| &v.0) };
            if let Some(p) = alvo {
                e.pedidos.push(Pedido::AbrirPagina(p.clone()));
            }
        }
        _ => return false,
    }
    true
}

fn tecla_no_kanban(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Kanban(colunas)) = &t.dados else { return false };
    let n = colunas.len();
    match tecla {
        "l" | "ArrowRight" => t.chip = (t.chip + 1).min(n.saturating_sub(1)),
        "h" | "ArrowLeft" => t.chip = t.chip.saturating_sub(1),
        _ => return false,
    }
    let cartoes = colunas.get(t.chip).map_or(0, |c| c.2.len());
    t.selecionado = t.selecionado.min(cartoes.saturating_sub(1));
    true
}

fn tecla_nas_tarefas(e: &mut Estado, tecla: &str) -> bool {
    let Some(t) = e.especial.as_mut() else { return false };
    let Some(Dados::Tarefas(lista)) = &t.dados else { return false };
    match tecla {
        // `s` gira a ordem, como clicar no cabeçalho da coluna.
        "s" => {
            t.chip = (t.chip + 1) % 3;
            t.selecionado = 0;
        }
        "Enter" => {
            if let Some(path) = tarefas_ordenadas(lista, t.chip).get(t.selecionado).map(|x| x.path.clone()) {
                e.pedidos.push(Pedido::AbrirPagina(path));
            }
        }
        _ => return false,
    }
    true
}

/// As tarefas na ordem escolhida: título, status ou prioridade.
fn tarefas_ordenadas(lista: &[Tarefa], ordem: usize) -> Vec<&Tarefa> {
    let mut v: Vec<&Tarefa> = lista.iter().collect();
    match ordem {
        1 => v.sort_by(|a, b| a.status.cmp(&b.status)),
        2 => v.sort_by(|a, b| a.prioridade.cmp(&b.prioridade)),
        _ => v.sort_by(|a, b| a.titulo.cmp(&b.titulo)),
    }
    v
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
    let Some(na_tela) = lista.get(t.selecionado).cloned() else { return false };
    let p = na_tela.proposta.clone();
    let diff = p.diff(&na_tela.atual);
    let trechos = anotadinho_core::diff::trechos(&diff);
    let escolhidos: Vec<bool> = (0..trechos.len()).map(|i| !t.fora.contains(&format!("{}#{i}", p.id))).collect();
    let aceitos = escolhidos.iter().filter(|x| **x).count();
    match tecla {
        // Andar pelos trechos e tirar/pôr um deles (ciclo 404).
        "l" | "ArrowRight" | "Tab" => t.trecho = (t.trecho + 1).min(trechos.len().saturating_sub(1)),
        "h" | "ArrowLeft" => t.trecho = t.trecho.saturating_sub(1),
        " " | "x" if !trechos.is_empty() => {
            let chave = format!("{}#{}", p.id, t.trecho.min(trechos.len() - 1));
            if !t.fora.remove(&chave) {
                t.fora.insert(chave);
            }
        }
        // Marcar pra agir em lote (ciclo 409): revisar uma por uma e
        // decidir todas de uma vez é o caminho de quem acumulou fila.
        "m" => {
            if !t.marcadas.remove(&p.id) {
                t.marcadas.insert(p.id.clone());
            }
            let n = t.marcadas.len();
            e.aviso = Some(if n == 0 { "nenhuma marcada".into() } else { format!("{n} marcada(s)") });
        }
        "M" => {
            if t.marcadas.len() == lista.len() {
                t.marcadas.clear();
                e.aviso = Some("desmarquei todas".into());
            } else {
                t.marcadas = lista.iter().map(|x| x.proposta.id.clone()).collect();
                e.aviso = Some(format!("{} marcada(s)", t.marcadas.len()));
            }
        }
        // Com marcadas, `a` e `r` valem pra elas — e só inteiras: um
        // lote não é lugar de escolher trecho.
        // Proposta de lote (ciclo 420): a decisão é do lote inteiro —
        // aplicar três de quatro deixa o vault num estado que ninguém
        // propôs.
        "a" | "Enter" if p.lote.is_some() && t.marcadas.is_empty() => {
            let lote = p.lote.clone().unwrap_or_default();
            let alvos: Vec<String> = lista
                .iter()
                .filter(|x| x.proposta.lote.as_deref() == Some(lote.as_str()))
                .map(|x| x.proposta.alvo.clone())
                .collect();
            e.modal = Some(Modal::Confirmar {
                titulo: format!("Aplicar o lote {lote}"),
                mensagem: format!("Gravar as {} página(s) juntas: {}?", alvos.len(), alvos.join(", ")),
                acao: Pedido::DecidirLote { lote, aplicar: true, motivo: String::new() },
            });
        }
        "r" if p.lote.is_some() && t.marcadas.is_empty() => {
            let lote = p.lote.clone().unwrap_or_default();
            e.modal = Some(Modal::Entrada {
                titulo: format!("Recusar o lote {lote} — por quê?"),
                campo: crate::componentes::Campo::default(),
                acao: super::modais::AcaoDaEntrada::MotivoDaRecusaDoLote(lote),
            });
        }
        "a" | "Enter" if !t.marcadas.is_empty() => {
            let ids: Vec<String> = lista
                .iter()
                .map(|x| x.proposta.id.clone())
                .filter(|id| t.marcadas.contains(id))
                .collect();
            let alvos: Vec<String> = lista
                .iter()
                .filter(|x| t.marcadas.contains(&x.proposta.id))
                .map(|x| x.proposta.alvo.clone())
                .collect();
            e.modal = Some(Modal::Confirmar {
                titulo: format!("Aplicar {} proposta(s)", ids.len()),
                mensagem: format!("Gravar inteiras: {}?", alvos.join(", ")),
                acao: Pedido::DecidirVarias { ids, aplicar: true, motivo: String::new() },
            });
        }
        "r" if !t.marcadas.is_empty() => {
            let ids: Vec<String> = lista
                .iter()
                .map(|x| x.proposta.id.clone())
                .filter(|id| t.marcadas.contains(id))
                .collect();
            e.modal = Some(Modal::Entrada {
                titulo: format!("Recusar {} proposta(s) — por quê?", ids.len()),
                campo: crate::componentes::Campo::default(),
                acao: super::modais::AcaoDaEntrada::MotivoDaRecusaVarias(ids),
            });
        }
        "A" if !trechos.is_empty() => {
            let ids: Vec<String> = lista.iter().map(|x| x.proposta.id.clone()).collect();
            t.fora.retain(|c| !ids.iter().any(|id| c.starts_with(&format!("{id}#"))));
            e.aviso = Some("todos os trechos de volta".into());
        }
        "v" => {
            if !t.visualizando.remove(&p.id) {
                t.visualizando.insert(p.id.clone());
            }
            t.deslocamento = 0;
        }
        "a" | "Enter" if aceitos == 0 && !trechos.is_empty() => {
            e.aviso = Some("nenhum trecho escolhido: espaço põe de volta".into());
        }
        "a" | "Enter" => {
            let parcial = aceitos < trechos.len();
            let acao = if parcial {
                Pedido::AplicarPropostaParcial {
                    id: p.id.clone(),
                    conteudo: anotadinho_core::diff::aplicar_trechos(&diff, &trechos, &escolhidos),
                    aceitos,
                    de: trechos.len(),
                }
            } else {
                Pedido::DecidirProposta { id: p.id.clone(), aplicar: true, motivo: String::new() }
            };
            e.modal = Some(Modal::Confirmar {
                titulo: if parcial { "Aplicar os trechos escolhidos".into() } else { "Aplicar proposta".to_string() },
                mensagem: if parcial {
                    format!("Gravar {} com {aceitos} de {} trecho(s) da proposta de {}?", p.alvo, trechos.len(), p.autor)
                } else {
                    format!(
                        "{} {} com o conteúdo proposto por {}?",
                        if p.operacao == Operacao::Criar { "Criar" } else { "Substituir" },
                        p.alvo,
                        p.autor
                    )
                },
                acao,
            });
        }
        // Editar antes de aplicar (ciclo 411): a revisão que quase
        // aceita, mas quer trocar uma frase, não precisa mais recusar e
        // pedir de novo.
        "e" => {
            super::modais::editar_proposta(e, &p.id, &p.alvo, &p.conteudo);
        }
        // Recusar pede o motivo (ciclo 404): ele entra no registro.
        "r" => {
            e.modal = Some(Modal::Entrada {
                titulo: format!("Recusar {} — por quê? (Enter sem texto recusa sem motivo)", p.alvo),
                campo: crate::componentes::Campo::default(),
                acao: super::modais::AcaoDaEntrada::MotivoDaRecusa(p.id.clone()),
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
        TipoEspecial::Propostas => " j k proposta · h l trecho · espaço tira · a aplica · r recusa · v visualização ",
        TipoEspecial::Kanban => " h l coluna · j k cartão ",
        TipoEspecial::Tarefas => " j k · Enter abre · s ordena ",
        TipoEspecial::Grafo => " j k página · h l conexão · Enter abre ",
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
    let titulo = e.paginas.get(e.pagina).map(|p| p.title.clone()).unwrap_or_default();
    let (linhas, faixas) = linhas_da_tela(t, &titulo, tema, w, no_foco);
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
pub fn linhas_da_tela(t: &TelaEspecial, pagina: &str, tema: &Tema, w: usize, no_foco: bool) -> (Vec<Line<'static>>, Vec<(usize, usize)>) {
    let texto = Style::default().fg(tema.var("text-primary"));
    let apagado = Style::default().fg(tema.var("text-muted"));
    let mut fora: Vec<Line<'static>> = Vec::new();
    // O cabeçalho da página tipada (ciclo 360), como o `TypedPageHeader`
    // da janela: o título dela e "Propriedades" (`=`). A revisão de
    // propostas não tem, lá também não.
    if t.tipo != TipoEspecial::Propostas && !pagina.is_empty() {
        let esquerda = format!(" {pagina}");
        let direita = "✲ Propriedades  = ";
        fora.push(Line::from(vec![
            Span::styled(esquerda.clone(), Style::default().fg(tema.var("accent-blue")).add_modifier(Modifier::BOLD)),
            Span::raw(" ".repeat(w.saturating_sub(esquerda.chars().count() + direita.chars().count()))),
            Span::styled(direita, apagado),
        ]));
        fora.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(tema.var("border")))));
    }
    let mut faixas = Vec::new();
    let (titulo, direita) = match (&t.tipo, &t.dados) {
        (TipoEspecial::Tags, _) => ("Tags", String::new()),
        (TipoEspecial::Assets, _) => ("Assets", String::new()),
        (TipoEspecial::Propostas, Some(Dados::Propostas(p))) => ("Propostas do agente", format!("{} pendente(s) ", p.len())),
        (TipoEspecial::Propostas, _) => ("Propostas do agente", String::new()),
        (TipoEspecial::Kanban, _) => ("Kanban", String::new()),
        (TipoEspecial::Tarefas, Some(Dados::Tarefas(l))) => ("Tarefas", format!("{} ", l.len())),
        (TipoEspecial::Tarefas, _) => ("Tarefas", String::new()),
        (TipoEspecial::Grafo, Some(Dados::Grafo(g))) => (
            "Grafo de conexões",
            format!("{} páginas · {} ligações ", g.len(), g.iter().map(|n| n.2.len()).sum::<usize>() / 2),
        ),
        (TipoEspecial::Grafo, _) => ("Grafo de conexões", String::new()),
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
                let antes = fora.len();
                fora.extend(tabela_de_assets(t, tema, assets, w, no_foco, antes, &mut faixas));
            }
        }
        Dados::Grafo(nos) => {
            fora.push(Line::default());
            if nos.iter().all(|n| n.2.is_empty()) {
                fora.extend(paragrafo("Nenhuma conexão ainda. Ligue páginas com [[Título]] e elas aparecem aqui.", apagado, w));
            }
            for (i, (_, titulo, vizinhos)) in nos.iter().enumerate() {
                let sel = no_foco && i == t.selecionado;
                let inicio = fora.len();
                fora.extend(no_do_grafo(tema, titulo, vizinhos, sel.then_some(t.chip), i == t.selecionado, w));
                faixas.push((inicio, fora.len()));
            }
        }
        Dados::Kanban(colunas) => {
            fora.push(Line::default());
            fora.extend(quadro_kanban(t, tema, colunas, w, no_foco));
        }
        Dados::Tarefas(lista) => {
            fora.push(Line::default());
            if lista.is_empty() {
                fora.extend(paragrafo("Nenhuma tarefa encontrada. Use 'status::' e 'priority::' nas páginas.", apagado, w));
            } else {
                let antes = fora.len();
                fora.extend(tabela_de_tarefas(t, tema, lista, w, no_foco, antes, &mut faixas));
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
                let cursor_do_trecho = (i == t.selecionado && no_foco).then_some(t.trecho);
                fora.extend(cartao_da_proposta(tema, p, visualizar, i == t.selecionado, no_foco, w, cursor_do_trecho, &t.fora, t.marcadas.contains(&p.proposta.id), t.marcadas.len()));
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
    antes: usize,
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
    // As faixas contam do começo da tela.
    let base = antes + fora.len();
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
    cursor_do_trecho: Option<usize>,
    fora_da_aplicacao: &BTreeSet<String>,
    // `marcada`: esta proposta entra no lote (ciclo 409); `em_lote`:
    // quantas marcadas na tela — com marcadas, os botões mudam.
    marcada: bool,
    em_lote: usize,
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
        Span::styled(if marcada { "◉ " } else { "  " }.to_string(), Style::default().fg(tema.var("accent-blue"))),
        Span::styled(format!(" {op} "), tema.pilula(papel)),
        // O lote na frente do alvo (ciclo 420): quem decide precisa ver
        // que aquela página não vem sozinha.
        Span::styled(
            proposta.lote.as_ref().map(|l| format!(" ⛓ {l} ")).unwrap_or_default(),
            Style::default().fg(tema.var("accent-purple")),
        ),
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
    // O veredito do revisor (ciclo 436), antes do diff: é o que muda a
    // atenção com que se lê o resto.
    if let Some(r) = &proposta.revisao {
        use anotadinho_core::proposta::Veredito;
        let cor = match r.veredito {
            Veredito::Aprova => tema.var("success"),
            Veredito::Ressalva => tema.var("warning"),
            Veredito::Recusa => tema.var("error"),
        };
        miolo.push(Line::from(vec![
            Span::styled(format!(" revisor: {} ", r.veredito.rotulo()), Style::default().fg(cor).add_modifier(Modifier::BOLD)),
            Span::styled(format!("· {} · {}", r.agente, r.quando), apagado),
        ]));
        for l in super::quebrar_texto(Line::from(Span::styled(r.notas.clone(), apagado)), dentro, 1) {
            miolo.push(l);
        }
    }
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
        miolo.extend(pagina_renderizada(corpo, tema, dentro));
    } else {
        let sai = Style::default().fg(tema.var("text-primary")).bg(misturar(tema.var("error"), tema.var("bg-surface"), 0.16));
        let entra = Style::default().fg(tema.var("text-primary")).bg(misturar(tema.var("success"), tema.var("bg-surface"), 0.16));
        // Cada trecho ganha cabeçalho: dá pra tirar um e aplicar o resto
        // (ciclo 404).
        let trechos = anotadinho_core::diff::trechos(&linhas);
        let esta_fora = |k: usize| fora_da_aplicacao.contains(&format!("{}#{k}", proposta.id));
        // O realce fino (ciclo 410): numa troca de linha, a PALAVRA que
        // mudou fica mais forte que o resto. Sem isto, uma linha longa
        // em que mudou uma data pinta inteira, e a pessoa relê tudo pra
        // achar o quê.
        let mut par_de: std::collections::HashMap<usize, usize> = std::collections::HashMap::new();
        for t in &trechos {
            for (i, j) in anotadinho_core::diff::pares_do_trecho(&linhas, t) {
                par_de.insert(i, j);
                par_de.insert(j, i);
            }
        }
        let forte = |base: Style, cor: &str| base.bg(misturar(tema.var(cor), tema.var("bg-surface"), 0.42)).add_modifier(Modifier::BOLD);
        for (i, l) in linhas.iter().enumerate() {
            let k = trechos.iter().position(|t| i >= t.inicio && i < t.fim);
            if let Some(k) = k.filter(|k| trechos[*k].inicio == i) {
                let t = &trechos[k];
                let rotulo = format!(
                    " trecho {}/{} · −{} +{} {} ",
                    k + 1,
                    trechos.len(),
                    t.removidas,
                    t.adicionadas,
                    if esta_fora(k) { "· FORA (espaço põe)" } else { "· espaço tira" }
                );
                let estilo = match (cursor_do_trecho == Some(k), esta_fora(k)) {
                    (true, _) => tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD),
                    (false, true) => apagado.add_modifier(Modifier::DIM),
                    (false, false) => apagado,
                };
                miolo.push(Line::from(Span::styled(rotulo, estilo)));
            }
            let dentro_de_fora = k.is_some_and(esta_fora);
            let (marca, estilo) = match l {
                LinhaDiff::Igual { .. } => (" ", apagado),
                _ if dentro_de_fora => (if matches!(l, LinhaDiff::Removida { .. }) { "-" } else { "+" }, apagado.add_modifier(Modifier::DIM)),
                LinhaDiff::Removida { .. } => ("-", sai),
                LinhaDiff::Adicionada { .. } => ("+", entra),
            };
            // Os pedaços da linha, quando ela tem par parecido.
            let pedacos = par_de.get(&i).filter(|_| l.mudou() && !dentro_de_fora).and_then(|&outra| {
                let (a, b) = (linhas[i.min(outra)].texto(), linhas[i.max(outra)].texto());
                anotadinho_core::diff::diff_palavras(a, b)
                    .map(|(lado_a, lado_b)| if i < outra { lado_a } else { lado_b })
            });
            let mut spans = vec![Span::styled(marca.to_string(), estilo)];
            let mut usado = 1;
            match pedacos {
                Some(pedacos) => {
                    let realcado = forte(estilo, if matches!(l, LinhaDiff::Removida { .. }) { "error" } else { "success" });
                    for pedaco in pedacos {
                        let cabe: String = pedaco.texto.chars().take(dentro.saturating_sub(usado)).collect();
                        if cabe.is_empty() {
                            break;
                        }
                        usado += cabe.chars().count();
                        spans.push(Span::styled(cabe, if pedaco.mudou { realcado } else { estilo }));
                    }
                }
                None => {
                    let cabe: String = l.texto().chars().take(dentro.saturating_sub(usado)).collect();
                    usado += cabe.chars().count();
                    spans.push(Span::styled(cabe, estilo));
                }
            }
            if l.mudou() && !dentro_de_fora {
                spans.push(Span::styled(" ".repeat(dentro.saturating_sub(usado)), estilo));
            }
            miolo.push(Line::from(spans));
        }
    }
    miolo.push(Line::default());
    let primario = tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD);
    let fantasma = Style::default().fg(tema.var("text-primary")).bg(tema.var("bg-elevated"));
    let n_trechos = anotadinho_core::diff::trechos(&linhas).len();
    let de_fora = (0..n_trechos).filter(|k| fora_da_aplicacao.contains(&format!("{}#{k}", proposta.id))).count();
    let rotulo_aplicar = if de_fora > 0 {
        format!(" Aplicar {} de {n_trechos} a ", n_trechos - de_fora)
    } else {
        " Aplicar a ".to_string()
    };
    // Com propostas marcadas, `a`/`r` valem pro lote — o botão diz isso
    // em vez de mentir que age só nesta (ciclo 409).
    let (rotulo_aplicar, rotulo_recusar, dica) = if em_lote > 0 {
        (
            format!(" Aplicar {em_lote} marcada(s) a "),
            format!(" Recusar {em_lote} r "),
            " m marca/desmarca · M todas ".to_string(),
        )
    } else {
        match &proposta.lote {
            // No lote não há trecho a escolher: a decisão é do conjunto.
            Some(l) => (
                format!(" Aplicar o lote {l} a "),
                format!(" Recusar o lote r "),
                " a decisão vale pras páginas do lote ".to_string(),
            ),
            None => (
                rotulo_aplicar,
                " Recusar r ".to_string(),
                " Editar e · h l trecho · espaço tira/põe · m marca ".to_string(),
            ),
        }
    };
    miolo.push(Line::from(vec![
        Span::styled(rotulo_aplicar, primario),
        Span::raw(" "),
        Span::styled(rotulo_recusar, fantasma),
        Span::raw(" "),
        Span::styled(dica, apagado),
    ]));
    let mut fora = caixa(tema, miolo, w, cor_da_borda(tema, selecionado, no_foco));
    fora.push(Line::default());
    fora
}

/// As quatro colunas lado a lado, como `.kanban__board`.
fn quadro_kanban(t: &TelaEspecial, tema: &Tema, colunas: &[(String, String, Vec<String>)], w: usize, no_foco: bool) -> Vec<Line<'static>> {
    let n = colunas.len().max(1);
    let largura = (w.saturating_sub(n + 1) / n).max(8);
    let altura = colunas.iter().map(|c| c.2.len()).max().unwrap_or(0).max(1);
    let superficie = tema.var("bg-surface");
    let elevado = tema.var("bg-elevated");
    let texto = Style::default().fg(tema.var("text-primary"));
    let apagado = Style::default().fg(tema.var("text-muted"));
    let celula = |s: &str, n: usize| -> String {
        let mut c: String = s.chars().take(n).collect();
        c.push_str(&" ".repeat(n.saturating_sub(c.chars().count())));
        c
    };
    let mut linhas: Vec<Vec<Span<'static>>> = vec![vec![Span::raw(" ")]; altura * 2 + 1];
    for (ci, (_, rotulo, cartoes)) in colunas.iter().enumerate() {
        let cab = format!(" {rotulo} ");
        let conta = format!("{} ", cartoes.len());
        linhas[0].push(Span::styled(cab.clone(), texto.bg(superficie).add_modifier(Modifier::BOLD)));
        linhas[0].push(Span::styled(
            format!("{}{conta}", " ".repeat(largura.saturating_sub(cab.chars().count() + conta.chars().count()))),
            apagado.bg(superficie),
        ));
        for k in 0..altura {
            let (l1, l2) = (1 + 2 * k, 2 + 2 * k);
            match cartoes.get(k) {
                Some(titulo) => {
                    let aceso = no_foco && ci == t.chip && k == t.selecionado;
                    let estilo = if aceso { tema.estilo(Realce::Cursor) } else { texto.bg(elevado) };
                    linhas[l1].push(Span::styled(" ", Style::default().bg(superficie)));
                    linhas[l1].push(Span::styled(celula(&format!(" {titulo}"), largura.saturating_sub(2)), estilo));
                    linhas[l1].push(Span::styled(" ", Style::default().bg(superficie)));
                }
                None => linhas[l1].push(Span::styled(" ".repeat(largura), Style::default().bg(superficie))),
            }
            linhas[l2].push(Span::styled(" ".repeat(largura), Style::default().bg(superficie)));
        }
        for l in linhas.iter_mut() {
            l.push(Span::raw(" "));
        }
    }
    linhas.into_iter().map(Line::from).collect()
}

/// A tabela de tarefas, com o status em badge como `.task-table`.
fn tabela_de_tarefas(t: &TelaEspecial, tema: &Tema, lista: &[Tarefa], w: usize, no_foco: bool, antes: usize, faixas: &mut Vec<(usize, usize)>) -> Vec<Line<'static>> {
    let apagado = Style::default().fg(tema.var("text-muted"));
    let texto = Style::default().fg(tema.var("text-primary"));
    let (col_status, col_prio) = (16, 12);
    let col_titulo = w.saturating_sub(col_status + col_prio + 2).max(8);
    let celula = |s: &str, n: usize| -> String {
        let mut c: String = s.chars().take(n.saturating_sub(1)).collect();
        c.push_str(&" ".repeat(n.saturating_sub(c.chars().count())));
        c
    };
    let seta = |i: usize| if t.chip == i { " ↓" } else { "" };
    let cab = apagado.add_modifier(Modifier::BOLD);
    let mut fora = vec![
        Line::from(vec![
            Span::styled(format!(" {}", celula(&format!("Título{}", seta(0)), col_titulo)), cab),
            Span::styled(celula(&format!("Status{}", seta(1)), col_status), cab),
            Span::styled(celula(&format!("Prioridade{}", seta(2)), col_prio), cab),
        ]),
        Line::from(Span::styled("─".repeat(w), Style::default().fg(tema.var("border")))),
    ];
    let base = antes + fora.len();
    for (i, x) in tarefas_ordenadas(lista, t.chip).into_iter().enumerate() {
        let sel = no_foco && i == t.selecionado;
        let fundo = |s: Style| if sel { s.bg(tema.var("bg-elevated")) } else { s };
        let papel = match x.status.as_str() {
            "done" | "concluido" => Realce::BadgeSucesso,
            "doing" | "em-andamento" => Realce::BadgeInfo,
            _ => Realce::BadgeAtencao,
        };
        fora.push(Line::from(vec![
            Span::styled(if sel { "▌" } else { " " }, fundo(Style::default().fg(tema.var("accent-blue")))),
            Span::styled(celula(&x.titulo, col_titulo), fundo(texto)),
            Span::styled(format!(" {} ", x.status), tema.pilula(papel)),
            Span::styled(" ".repeat(col_status.saturating_sub(x.status.chars().count() + 2)), fundo(Style::default())),
            Span::styled(celula(&x.prioridade, col_prio), fundo(texto)),
        ]));
        faixas.push((base + i, base + i + 1));
    }
    fora
}

/// Uma página do grafo: o título (que abre) e as vizinhas em pílulas.
fn no_do_grafo(tema: &Tema, titulo: &str, vizinhos: &[(String, String)], chip: Option<usize>, selecionado: bool, w: usize) -> Vec<Line<'static>> {
    let apagado = Style::default().fg(tema.var("text-muted"));
    let proprio = if chip == Some(0) {
        tema.estilo(Realce::Cursor).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(tema.var("text-primary")).add_modifier(Modifier::BOLD)
    };
    let conta = match vizinhos.len() {
        0 => "sem conexões".to_string(),
        1 => "1 conexão".to_string(),
        n => format!("{n} conexões"),
    };
    let mut miolo = vec![Line::from(vec![Span::styled(format!(" {titulo} "), proprio), Span::raw(" "), Span::styled(conta, apagado)])];
    let chip_normal = Style::default().bg(tema.var("bg-elevated")).fg(tema.var("text-primary"));
    miolo.extend(pilulas(
        vizinhos
            .iter()
            .enumerate()
            .map(|(k, (_, t))| (format!("↔ {t}"), if chip == Some(k + 1) { tema.estilo(Realce::Cursor) } else { chip_normal }))
            .collect(),
        w.saturating_sub(4),
    ));
    caixa(tema, miolo, w, cor_da_borda(tema, selecionado, chip.is_some()))
}

/// A página desenhada pelo MESMO desenho da TUI (ciclo 399), como o
/// `PaginaPreview` da janela mostra a proposta renderizada — embeds
/// inclusive. Desenha num terminal de mentira da largura pedida e devolve
/// as linhas de dentro da borda, sem as vazias do fim.
pub fn pagina_renderizada(corpo: &str, tema: &Tema, largura: usize) -> Vec<Line<'static>> {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let paginas = vec![anotadinho_ipc::PageMeta { path: "pages/visualizacao.md".into(), title: String::new(), section: "pages".into() }];
    let mut e = Estado::novo(paginas, anotadinho_core::analise::analisar(corpo));
    e.tema = tema.clone();
    e.preferencias.sidebar = false;
    e.foco = Foco::Paginas;
    // A trilha da sidebar recolhida ocupa 3 colunas (ciclo 400).
    const TRILHA: u16 = 3;
    let (w, h) = (largura as u16 + 2 + TRILHA, 300u16);
    let Ok(mut term) = Terminal::new(TestBackend::new(w, h)) else { return Vec::new() };
    if term.draw(|f| super::desenhar(f, &mut e)).is_err() {
        return Vec::new();
    }
    let buf = term.backend().buffer().clone();
    let mut linhas: Vec<Line<'static>> = Vec::new();
    for y in 1..h - 1 {
        let mut spans: Vec<Span<'static>> = Vec::new();
        let mut texto = String::new();
        let mut estilo: Option<Style> = None;
        for x in TRILHA + 1..w - 1 {
            let cel = &buf[(x, y)];
            let st = cel.style();
            if estilo != Some(st) {
                if let Some(s) = estilo {
                    spans.push(Span::styled(std::mem::take(&mut texto), s));
                }
                estilo = Some(st);
            }
            texto.push_str(cel.symbol());
        }
        if let Some(s) = estilo {
            spans.push(Span::styled(texto, s));
        }
        linhas.push(Line::from(spans));
    }
    while linhas.last().is_some_and(|l| l.spans.iter().all(|s| s.content.trim().is_empty())) {
        linhas.pop();
    }
    linhas
}
