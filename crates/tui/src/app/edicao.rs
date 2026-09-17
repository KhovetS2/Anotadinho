//! Editar embeds pela TUI (ciclos 318 a 324).
//!
//! Toda edição aqui é um COMANDO DE VIM sobre o item sob o cursor
//! (ciclo 322). A gramática é a do núcleo (`vim::tecla_normal`), e a
//! tradução pra "o que fazer com um item" também (`vim::edicao_de`): `o`
//! cria, `i`/`a` reescrevem, `cc` reescreve do zero, `dd` apaga, `x` apaga
//! o conteúdo, `yy`/`p` copiam e colam, `>>`/`<<` andam com o item,
//! `Ctrl+A`/`Ctrl+X` esticam ou trocam a opção, `~` alterna, `u`/`Ctrl+R`
//! desfazem e refazem. Cada embed só diz o que isso significa pra ele.
//!
//! A mudança em si é sempre a mesma (`editar_embed`): os dados do embed
//! saem da fonte dele, mudam, voltam pelo `to_fence_text` — o que a
//! janela grava — e o resto do arquivo fica byte a byte.


use anotadinho_core::date_util::{add_days, days_between};
use anotadinho_core::embed::{self as em, DocSegment, EmbedData};
use anotadinho_core::unidade::{Caminho, Tipo, Unidade};
use anotadinho_core::vim::Edicao;

use super::Estado;
use crate::tela;

/// Uma pergunta no rodapé: o texto que a edição precisa (ciclo 318).
#[derive(Debug, Clone, PartialEq)]
pub struct Pergunta {
    /// O que se pergunta ("Novo evento em 12/08/2026").
    pub rotulo: String,
    /// O que já foi digitado.
    pub texto: String,
    /// Onde está o cursor de texto, em caracteres (ciclo 333).
    pub cursor: usize,
    /// O que fazer com a resposta.
    pub acao: AcaoDaPergunta,
}

/// O que a resposta de uma pergunta faz.
#[derive(Debug, Clone, PartialEq)]
pub enum AcaoDaPergunta {
    /// Cria um evento de dia inteiro nessa data.
    NovoEvento {
        /// O calendário.
        embed: Caminho,
        /// `AAAA-MM-DD`.
        data: String,
    },
    /// Troca o título do evento.
    RenomearEvento {
        /// O calendário.
        embed: Caminho,
        /// A entrada, no arquivo.
        indice: usize,
    },
    /// Cria uma barra no cronograma, de `inicio` a `fim` (ciclo 320).
    NovaBarra {
        /// O cronograma.
        embed: Caminho,
        /// `AAAA-MM-DD`.
        inicio: String,
        /// `AAAA-MM-DD`.
        fim: String,
        /// Onde ela entra na lista.
        posicao: usize,
    },
    /// Troca o título da barra.
    RenomearBarra {
        /// O cronograma.
        embed: Caminho,
        /// O item, no arquivo.
        indice: usize,
    },
    /// Cria um cartão na coluna do kanban (ciclo 319).
    NovoCartao {
        /// O kanban.
        embed: Caminho,
        /// O nome da coluna.
        coluna: String,
        /// Antes de qual item do arquivo; `None` é no fim.
        antes_de: Option<usize>,
    },
    /// Troca o título do cartão.
    RenomearCartao {
        /// O kanban.
        embed: Caminho,
        /// O item, no arquivo.
        indice: usize,
    },
    /// Cria uma coluna no kanban nesta posição (ciclo 337).
    NovaColunaDoKanban {
        /// O kanban.
        embed: Caminho,
        /// Onde ela entra.
        posicao: usize,
    },
    /// Cria uma coluna de texto na tabela nesta posição (ciclo 337).
    NovaColunaDaTabela {
        /// A tabela.
        embed: Caminho,
        /// Onde ela entra.
        posicao: usize,
    },
    /// Troca o nome da coluna (e dos cartões que apontam pra ela).
    RenomearColuna {
        /// O kanban.
        embed: Caminho,
        /// A posição da coluna.
        coluna: usize,
    },
    /// Troca o valor de uma célula da tabela (ciclo 321).
    EditarCelula {
        /// A tabela.
        embed: Caminho,
        /// A linha de dados.
        linha: usize,
        /// A coluna.
        coluna: usize,
    },
    /// Troca o nome de uma coluna da tabela.
    RenomearColunaDaTabela {
        /// A tabela.
        embed: Caminho,
        /// A coluna.
        coluna: usize,
    },
    /// Troca o título do callout (ciclo 323).
    TituloDoCallout {
        /// O callout.
        embed: Caminho,
    },
    /// Reescreve ou cria um bloco de markdown — da página, do corpo de um
    /// callout ou de um painel (ciclo 334).
    Bloco(super::markdown::EdicaoDeBloco),
    /// Troca a legenda de uma imagem da galeria (ciclo 335).
    LegendaDaImagem {
        /// A galeria.
        embed: Caminho,
        /// O item.
        indice: usize,
    },
    /// Põe uma imagem na galeria, pelo caminho.
    NovaImagem {
        /// A galeria.
        embed: Caminho,
        /// Onde ela entra.
        posicao: usize,
    },
    /// Troca a nota do fluxo.
    NotaDoFluxo {
        /// O fluxo.
        embed: Caminho,
    },
    /// Troca o recorte da consulta pela linha de filtro.
    FiltroDaConsulta {
        /// A consulta.
        embed: Caminho,
    },
    /// Cria um botão no embed de ações (ciclo 324).
    NovoBotao {
        /// As ações.
        embed: Caminho,
        /// Onde ele entra.
        posicao: usize,
    },
    /// Troca o rótulo do botão.
    RenomearBotao {
        /// As ações.
        embed: Caminho,
        /// O botão.
        indice: usize,
    },
}

/// O que `yy` (ou `dd`) guardou pra `p` colar (ciclo 322). Cada embed
/// só cola o que é dele.
#[derive(Debug, Clone, PartialEq)]
pub enum Registro {
    /// Um evento do calendário.
    Evento(em::CalendarEntry),
    /// Um cartão do kanban.
    Cartao(em::KanbanCard),
    /// Uma barra do cronograma.
    Barra(em::TimelineItem),
    /// Uma linha da tabela.
    Linha(Vec<String>),
    /// Um botão de ações.
    Botao(em::ActionButton),
    /// Um bloco de markdown do corpo de um callout.
    Bloco(String),
    /// Uma imagem da galeria.
    Imagem(em::GalleryItem),
    /// Uma coluna do kanban, com os cartões dela.
    ColunaDoKanban(String, Vec<em::KanbanCard>),
    /// Uma coluna da tabela, com as células dela.
    ColunaDaTabela(em::TableColumn, Vec<String>),
    /// Um painel de colunas.
    Painel(em::ColumnPane),
}

/// Troca o trecho de um embed no texto da página pelo resultado de
/// `mudar` sobre os dados dele (ciclo 318).
pub(super) fn editar_embed(
    e: &mut Estado,
    embed: &[usize],
    mudar: impl FnOnce(&mut EmbedData) -> Result<(), String>,
) -> bool {
    let Some(texto) = e.texto_da_pagina.clone() else {
        e.aviso = Some("esta página não pode ser editada daqui".into());
        return false;
    };
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto);
    let base = texto.len() - corpo.len();
    let Some(u) = e.arvore.em(embed) else { return false };
    let (Some(faixa), Some(fonte)) = (u.intervalo.clone(), u.fonte.clone()) else { return false };
    let Some(mut dados) = dados_da_fonte(&fonte) else { return false };
    if let Err(motivo) = mudar(&mut dados) {
        e.aviso = Some(motivo);
        return false;
    }
    let mut novo = dados.to_fence_text();
    if fonte.ends_with('\n') && !novo.ends_with('\n') {
        novo.push('\n');
    }
    let novo_texto = format!("{}{}{}", &texto[..base + faixa.start], novo, &texto[base + faixa.end..]);
    e.aplicar_edicao(novo_texto);
    true
}

fn dados_da_fonte(fonte: &str) -> Option<EmbedData> {
    em::segment(fonte).into_iter().find_map(|s| match s {
        DocSegment::Embed(d) => Some(d),
        _ => None,
    })
}

/// Os dados do embed em `embed`, lidos da fonte dele.
pub(super) fn dados(e: &Estado, embed: &[usize]) -> Option<EmbedData> {
    dados_da_fonte(e.arvore.em(embed)?.fonte.as_deref()?)
}

/// Um par ler/editar pra cada tipo de embed. A edição recusa o que não
/// for do tipo — e, no calendário e no cronograma, a fonte vault, que é
/// só leitura como na janela.
macro_rules! tipado {
    ($ler:ident, $editar:ident, $variante:ident, $tipo:ty, $erro:literal $(, $vault:expr)?) => {
        #[allow(dead_code)]
        fn $ler(e: &Estado, embed: &[usize]) -> Option<$tipo> {
            match dados(e, embed)? {
                EmbedData::$variante(d) => Some(d),
                _ => None,
            }
        }
        fn $editar(e: &mut Estado, embed: &[usize], mudar: impl FnOnce(&mut $tipo) -> Result<(), String>) -> bool {
            editar_embed(e, embed, |dados| match dados {
                $(EmbedData::$variante(d) if ($vault)(&*d) => {
                    Err("vem do vault e é só leitura: edite a página".into())
                })?
                EmbedData::$variante(d) => mudar(d),
                _ => Err($erro.into()),
            })
        }
    };
}

tipado!(ler_calendario, editar_calendario, Calendar, em::CalendarEmbedData, "isto não é um calendário",
    |d: &em::CalendarEmbedData| d.mode == em::CalendarSource::Vault);
tipado!(ler_cronograma, editar_cronograma, Timeline, em::TimelineEmbedData, "isto não é um cronograma",
    |d: &em::TimelineEmbedData| d.source == em::TimelineSource::Vault);
tipado!(ler_kanban, editar_kanban, Kanban, em::KanbanEmbedData, "isto não é um kanban");
tipado!(ler_tabela, editar_tabela, Table, em::TableEmbedData, "isto não é uma tabela");
tipado!(ler_callout, editar_callout, Callout, em::CalloutEmbedData, "isto não é um callout");
tipado!(ler_acoes, editar_acoes, Actions, em::ActionsEmbedData, "isto não é um embed de ações");
tipado!(ler_galeria, editar_galeria, Gallery, em::GalleryEmbedData, "isto não é uma galeria");
tipado!(ler_colunas, editar_colunas, Columns, em::ColumnsEmbedData, "isto não é um embed de colunas");
tipado!(ler_fluxo, editar_fluxo, Fluxo, em::FluxoEmbedData, "isto não é um fluxo");
tipado!(ler_consulta, editar_consulta, Query, anotadinho_core::query::Query, "isto não é uma consulta");

/// O item do arquivo que a parte sob o cursor representa.
pub(super) fn indice_do_cursor(e: &Estado) -> Option<usize> {
    e.arvore
        .em(&e.cursor)?
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "indice"))
        .and_then(|f| f.texto.parse().ok())
}

/// O embed do tipo `tipo` que contém o cursor (ou é ele).
pub(super) fn embed_do_cursor(e: &Estado, tipo: &str) -> Option<Caminho> {
    (1..=e.cursor.len())
        .map(|n| &e.cursor[..n])
        .find(|c| matches!(e.arvore.em(c).map(|u| &u.tipo), Some(Tipo::Embed(n)) if n == tipo))
        .map(|c| c.to_vec())
}

/// Onde está, na árvore de um calendário, o evento da entrada `indice`
/// — de preferência o COMEÇO da barra, e na data `data` quando dada.
fn achar_evento(arvore: &Unidade, embed: &[usize], indice: usize, data: Option<&str>) -> Option<Caminho> {
    let mut achados: Vec<(Caminho, bool)> = Vec::new();
    for (c, u) in arvore.em(embed)?.percorrer() {
        let Tipo::Parte { nome, .. } = &u.tipo else { continue };
        let e_evento = tela::e_evento(nome) || nome.starts_with("compromisso");
        let mesmo = u.filhos.iter().any(|f| {
            matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "indice") && f.texto == indice.to_string()
        });
        if e_evento && mesmo {
            let mut caminho = embed.to_vec();
            caminho.extend(c);
            achados.push((caminho, nome != "evento-continua"));
        }
    }
    if let Some(data) = data {
        if let Some((c, _)) = achados.iter().find(|(c, _)| {
            let mut dia = c.clone();
            dia.pop();
            tela::data_do_cursor(arvore, &dia).as_deref() == Some(data)
        }) {
            return Some(c.clone());
        }
    }
    achados.iter().find(|(_, comeco)| *comeco).or(achados.first()).map(|(c, _)| c.clone())
}

/// Onde está a parte `nome` que carrega o `indice` dado, dentro do embed.
pub(super) fn achar_com_indice(arvore: &Unidade, embed: &[usize], nome_da_parte: &str, indice: usize) -> Option<Caminho> {
    arvore.em(embed)?.percorrer().into_iter().find_map(|(c, u)| {
        let e_parte = matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == nome_da_parte);
        let mesmo = u.filhos.iter().any(|f| {
            matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "indice") && f.texto == indice.to_string()
        });
        (e_parte && mesmo).then(|| {
            let mut caminho = embed.to_vec();
            caminho.extend(c);
            caminho
        })
    })
}

fn ir(e: &mut Estado, destino: Option<Caminho>) {
    if let Some(c) = destino {
        e.cursor = c;
    }
    e.seguir_cursor();
}

pub(super) fn perguntar(e: &mut Estado, rotulo: impl Into<String>, texto: String, acao: AcaoDaPergunta) {
    let cursor = texto.chars().count();
    e.pergunta = Some(Pergunta { rotulo: rotulo.into(), texto, cursor, acao });
}

/// `AAAA-MM-DD` como `DD/MM/AAAA`.
fn data_legivel(iso: &str) -> String {
    match anotadinho_core::date_util::parse_date(iso) {
        Some((y, m, d)) => format!("{d:02}/{m:02}/{y}"),
        None => iso.to_string(),
    }
}

/// Uma edição de item, vinda da gramática do vim (ciclo 322).
///
/// Devolve `false` quando o cursor não está num embed editável — aí o
/// comando segue pro caminho de sempre. Num embed, a tecla é sempre
/// consumida: o que não se aplica ali vira aviso no rodapé, em vez de
/// cair calado na navegação.
pub(super) fn editar_item(e: &mut Estado, ed: Edicao) -> bool {
    match ed {
        Edicao::Desfazer => {
            desfazer(e, true);
            return true;
        }
        Edicao::Refazer => {
            desfazer(e, false);
            return true;
        }
        _ => {}
    }
    // No PRÓPRIO embed (não dentro dele), criar, apagar, copiar, colar e
    // mover são do embed como BLOCO da página (ciclo 334): `o` num kanban
    // abre um parágrafo depois dele, `dd` apaga o kanban inteiro.
    let no_embed = e.cursor.len() == 1 && matches!(e.arvore.em(&e.cursor).map(|u| &u.tipo), Some(Tipo::Embed(_)));
    let de_bloco = matches!(
        ed,
        Edicao::Criar { .. } | Edicao::Apagar | Edicao::Copiar | Edicao::Colar { .. } | Edicao::Deslocar(_) | Edicao::Reordenar(_)
    );
    let feito = if no_embed && de_bloco {
        super::markdown::no_markdown(e, ed, super::markdown::Hospedeiro::Pagina)
    } else if tela::calendario_do_cursor(&e.arvore, &e.cursor).is_some() {
        no_calendario(e, ed)
    } else if embed_do_cursor(e, "kanban").is_some() {
        no_kanban(e, ed)
    } else if embed_do_cursor(e, "timeline").is_some() {
        no_cronograma(e, ed)
    } else if embed_do_cursor(e, "table").is_some() {
        na_tabela(e, ed)
    } else if embed_do_cursor(e, "callout").is_some() {
        no_callout(e, ed)
    } else if embed_do_cursor(e, "actions").is_some() {
        nas_acoes(e, ed)
    } else if embed_do_cursor(e, "gallery").is_some() {
        na_galeria(e, ed)
    } else if embed_do_cursor(e, "fluxo").is_some() {
        no_fluxo(e, ed)
    } else if embed_do_cursor(e, "query").is_some() {
        na_consulta(e, ed)
    } else if embed_do_cursor(e, "columns").is_some_and(|c| e.cursor.len() <= c.len() + 1) {
        nos_paineis(e, ed)
    } else if let Some(h) = super::markdown::hospedeiro_do_cursor(e) {
        // O markdown: a página, ou o corpo de um painel de colunas.
        if !super::markdown::no_markdown(e, ed, h) {
            return false;
        }
        true
    } else {
        return false;
    };
    // `i`/`I` começam com o cursor de texto no começo.
    if let (Edicao::Reescrever { no_fim: false, .. }, Some(p)) = (ed, e.pergunta.as_mut()) {
        p.cursor = 0;
    }
    if !feito && e.aviso.is_none() && e.pergunta.is_none() {
        e.aviso = Some("esse comando não vale aqui".into());
    }
    true
}

/// `u` e `Ctrl+R`: volta (ou avança) o texto da página e o cursor de
/// antes da edição. A gravação vai junto — desfazer é editar de volta.
fn desfazer(e: &mut Estado, desfazendo: bool) {
    let origem = if desfazendo { &mut e.desfazer } else { &mut e.refazer };
    let Some((texto, cursor)) = origem.pop() else {
        e.aviso = Some(if desfazendo { "nada pra desfazer" } else { "nada pra refazer" }.into());
        return;
    };
    let atual = (e.texto_da_pagina.clone().unwrap_or_default(), e.cursor.clone());
    if desfazendo {
        e.refazer.push(atual);
    } else {
        e.desfazer.push(atual);
    }
    e.cursor = cursor;
    e.trocar_texto(texto);
    e.aviso = Some(if desfazendo { "desfeito" } else { "refeito" }.into());
    e.seguir_cursor();
}

/// Responde a pergunta aberta (ciclo 318).
pub(super) fn tecla_na_pergunta(e: &mut Estado, tecla: &str) {
    let Some(p) = e.pergunta.as_mut() else { return };
    let n = p.texto.chars().count();
    p.cursor = p.cursor.min(n);
    let byte = |t: &str, c: usize| t.char_indices().nth(c).map_or(t.len(), |(i, _)| i);
    // `/` num bloco novo ainda vazio abre o menu de inserir (ciclo 347),
    // como digitar `/` numa linha vazia da janela.
    if tecla == "/" && p.texto.is_empty() {
        if let AcaoDaPergunta::Bloco(b) = &p.acao {
            if b.novo {
                let b = b.clone();
                e.pergunta = None;
                super::markdown::abrir_menu(e, b);
                return;
            }
        }
    }
    match tecla {
        // O modo de inserção do vim (ciclo 333): `Esc` CONFIRMA — sai da
        // inserção com o que foi digitado. Desistir é `u` depois.
        "Escape" | "Enter" => {
            let Some(p) = e.pergunta.take() else { return };
            let titulo = p.texto.trim().to_string();
            if titulo.is_empty() {
                return;
            }
            let continua = tecla == "Enter";
            if let AcaoDaPergunta::Bloco(b) = &p.acao {
                if let Some(gravado) = super::markdown::responder(e, b, &titulo) {
                    if continua {
                        super::markdown::continuar(e, b, gravado);
                    }
                }
            } else {
                responder(e, p.acao, titulo);
            }
            e.seguir_cursor();
        }
        "Backspace" if p.cursor > 0 => {
            let i = byte(&p.texto, p.cursor - 1);
            p.texto.remove(i);
            p.cursor -= 1;
        }
        "Delete" if p.cursor < n => {
            let i = byte(&p.texto, p.cursor);
            p.texto.remove(i);
        }
        "Backspace" | "Delete" => {}
        "ArrowLeft" => p.cursor = p.cursor.saturating_sub(1),
        "ArrowRight" => p.cursor = (p.cursor + 1).min(n),
        "Home" | "Ctrl+a" => p.cursor = 0,
        "End" | "Ctrl+e" => p.cursor = n,
        // `Ctrl+W` apaga a palavra antes do cursor; `Ctrl+U`, tudo antes.
        "Ctrl+w" => {
            let antes: Vec<char> = p.texto.chars().take(p.cursor).collect();
            let mut k = antes.len();
            while k > 0 && antes[k - 1] == ' ' {
                k -= 1;
            }
            while k > 0 && antes[k - 1] != ' ' {
                k -= 1;
            }
            let resto: String = p.texto.chars().skip(p.cursor).collect();
            p.texto = format!("{}{resto}", antes[..k].iter().collect::<String>());
            p.cursor = k;
        }
        "Ctrl+u" => {
            p.texto = p.texto.chars().skip(p.cursor).collect();
            p.cursor = 0;
        }
        t if t.chars().count() == 1 => {
            let i = byte(&p.texto, p.cursor);
            p.texto.insert_str(i, t);
            p.cursor += 1;
        }
        _ => {}
    }
}

fn responder(e: &mut Estado, acao: AcaoDaPergunta, titulo: String) {
    match acao {
        AcaoDaPergunta::NovoEvento { embed, data } => {
            let mut novo = 0;
            if editar_calendario(e, &embed, |d| {
                d.add_entry(data.clone(), titulo.clone());
                novo = d.entries.len() - 1;
                Ok(())
            }) {
                let destino = achar_evento(&e.arvore, &embed, novo, Some(&data));
                ir(e, destino);
            }
        }
        AcaoDaPergunta::RenomearEvento { embed, indice } => {
            let data_antes = tela::data_do_cursor(&e.arvore, &e.cursor);
            if editar_calendario(e, &embed, |d| {
                let mut entrada = d.entries.get(indice).cloned().ok_or("o evento sumiu do arquivo")?;
                entrada.title = titulo.clone();
                d.update_entry(indice, entrada);
                Ok(())
            }) {
                let destino = achar_evento(&e.arvore, &embed, indice, data_antes.as_deref());
                ir(e, destino);
            }
        }
        AcaoDaPergunta::NovaBarra { embed, inicio, fim, posicao } => {
            let mut novo = 0;
            if editar_cronograma(e, &embed, |d| {
                d.add_item(titulo.clone(), inicio.clone(), fim.clone());
                let item = d.items.pop().ok_or("a barra não entrou")?;
                novo = posicao.min(d.items.len());
                d.items.insert(novo, item);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "barra", novo);
                ir(e, destino);
            }
        }
        AcaoDaPergunta::RenomearBarra { embed, indice } => {
            if editar_cronograma(e, &embed, |d| {
                let mut item = d.items.get(indice).cloned().ok_or("a barra sumiu do arquivo")?;
                item.title = titulo.clone();
                d.update_item(indice, item);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "barra", indice)
                    .or_else(|| achar_com_indice(&e.arvore, &embed, "item", indice));
                ir(e, destino);
            }
        }
        AcaoDaPergunta::NovoCartao { embed, coluna, antes_de } => {
            let mut novo = 0;
            if editar_kanban(e, &embed, |d| {
                d.add_card(coluna.clone(), titulo.clone());
                novo = d.items.len() - 1;
                if let Some(b) = antes_de.filter(|b| *b < novo) {
                    d.move_card(novo, coluna.clone(), Some(b));
                    novo = b;
                }
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "card", novo);
                ir(e, destino);
            }
        }
        AcaoDaPergunta::RenomearCartao { embed, indice } => {
            if editar_kanban(e, &embed, |d| {
                if indice >= d.items.len() {
                    return Err("o cartão sumiu do arquivo".into());
                }
                d.edit_card(indice, titulo.clone());
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "card", indice);
                ir(e, destino);
            }
        }
        AcaoDaPergunta::NovaColunaDoKanban { embed, posicao } => {
            let mut onde = 0;
            if editar_kanban(e, &embed, |d| {
                if d.columns.contains(&titulo) {
                    return Err(format!("já existe a coluna {titulo}"));
                }
                onde = posicao.min(d.columns.len());
                d.columns.insert(onde, titulo.clone());
                Ok(())
            }) {
                ir(e, Some([embed.as_slice(), &[onde]].concat()));
            }
        }
        AcaoDaPergunta::NovaColunaDaTabela { embed, posicao } => {
            let mut onde = 0;
            if editar_tabela(e, &embed, |d| {
                d.add_column(titulo.clone());
                let fim = d.columns.len() - 1;
                onde = posicao.min(fim);
                mover_coluna_da_tabela(d, fim, onde);
                Ok(())
            }) {
                ir(e, Some([embed.as_slice(), &[0, onde]].concat()));
            }
        }
        AcaoDaPergunta::RenomearColuna { embed, coluna } => {
            editar_kanban(e, &embed, |d| {
                if coluna >= d.columns.len() {
                    return Err("a coluna sumiu do arquivo".into());
                }
                d.rename_column(coluna, titulo.clone());
                Ok(())
            });
        }
        AcaoDaPergunta::EditarCelula { embed, linha, coluna } => {
            editar_tabela(e, &embed, |d| {
                let tipo = d.columns.get(coluna).map(|c| c.kind.clone()).ok_or("a coluna sumiu do arquivo")?;
                if linha >= d.rows.len() {
                    return Err("a linha sumiu do arquivo".into());
                }
                // Valor novo num select vira opção — a criação inline de
                // tag da janela.
                let valor = match tipo {
                    em::ColumnKind::MultiSelect { .. } => {
                        let tags: Vec<String> =
                            titulo.split(',').map(|t| t.trim().to_string()).filter(|t| !t.is_empty()).collect();
                        for t in &tags {
                            d.add_column_option(coluna, t.clone());
                        }
                        tags.join(", ")
                    }
                    _ => {
                        d.add_column_option(coluna, titulo.clone());
                        titulo.clone()
                    }
                };
                d.set_cell(linha, coluna, valor);
                Ok(())
            });
        }
        AcaoDaPergunta::RenomearColunaDaTabela { embed, coluna } => {
            editar_tabela(e, &embed, |d| {
                if coluna >= d.columns.len() {
                    return Err("a coluna sumiu do arquivo".into());
                }
                d.set_column_name(coluna, titulo.clone());
                Ok(())
            });
        }
        AcaoDaPergunta::TituloDoCallout { embed } => {
            if editar_callout(e, &embed, |d| {
                d.title = titulo.clone();
                Ok(())
            }) {
                // O título é a primeira parte visível depois da variante.
                let mut c = embed.clone();
                c.push(1);
                ir(e, Some(c));
            }
        }
        // Respondido por `markdown::responder`, em `tecla_na_pergunta`.
        AcaoDaPergunta::Bloco(_) => {}
        AcaoDaPergunta::LegendaDaImagem { embed, indice } => {
            editar_galeria(e, &embed, |d| {
                if indice >= d.items.len() {
                    return Err("a imagem sumiu do arquivo".into());
                }
                d.set_caption(indice, titulo.clone());
                Ok(())
            });
        }
        AcaoDaPergunta::NovaImagem { embed, posicao } => {
            let mut onde = 0;
            if editar_galeria(e, &embed, |d| {
                d.add_item(titulo.clone());
                let item = d.items.pop().ok_or("a imagem não entrou")?;
                onde = posicao.min(d.items.len());
                d.items.insert(onde, item);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "miniatura", onde);
                ir(e, destino);
            }
        }
        AcaoDaPergunta::NotaDoFluxo { embed } => {
            editar_fluxo(e, &embed, |d| {
                d.nota = Some(titulo.clone());
                Ok(())
            });
        }
        AcaoDaPergunta::FiltroDaConsulta { embed } => {
            if editar_consulta(e, &embed, |q| q.aplicar_linha_de_filtro(&titulo)) {
                e.aviso = Some("consulta atualizada".into());
            }
        }
        AcaoDaPergunta::NovoBotao { embed, posicao } => {
            let mut novo = 0;
            if editar_acoes(e, &embed, |d| {
                novo = posicao.min(d.buttons.len());
                d.buttons.insert(
                    novo,
                    em::ActionButton { label: titulo.clone(), action: "open-page".into(), ..Default::default() },
                );
                Ok(())
            }) {
                ir(e, Some([embed.as_slice(), &[0, novo]].concat()));
                e.aviso = Some("botão novo: o destino se configura na janela".into());
            }
        }
        AcaoDaPergunta::RenomearBotao { embed, indice } => {
            editar_acoes(e, &embed, |d| {
                let mut b = d.buttons.get(indice).cloned().ok_or("o botão sumiu do arquivo")?;
                b.label = titulo.clone();
                d.update_button(indice, b);
                Ok(())
            });
        }
    }
}

// ---------------------------------------------------------------------
// Calendário (ciclo 318)
// ---------------------------------------------------------------------

fn no_calendario(e: &mut Estado, ed: Edicao) -> bool {
    let Some(embed) = tela::calendario_do_cursor(&e.arvore, &e.cursor) else { return false };
    let data = tela::data_do_cursor(&e.arvore, &e.cursor);
    let indice = indice_do_cursor(e);
    let entrada = indice.and_then(|i| ler_calendario(e, &embed)?.entries.get(i).cloned());
    match (ed, indice, entrada) {
        (Edicao::Criar { .. }, _, _) => {
            let Some(data) = data else {
                e.aviso = Some("escolha um dia pra criar o evento".into());
                return true;
            };
            perguntar(e, format!("Novo evento em {}", data_legivel(&data)), String::new(), AcaoDaPergunta::NovoEvento {
                embed,
                data,
            });
        }
        (Edicao::Reescrever { limpar, .. }, Some(indice), Some(entrada)) => {
            // Na semana o título vem com a hora na frente; o arquivo não.
            let texto = if limpar { String::new() } else { entrada.title };
            perguntar(e, "Renomear evento", texto, AcaoDaPergunta::RenomearEvento { embed, indice });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some(indice), Some(entrada)) => {
            let mut dia = e.cursor.clone();
            dia.pop();
            if editar_calendario(e, &embed, |d| {
                d.remove_entry(indice);
                Ok(())
            }) {
                e.registro = Some(Registro::Evento(entrada));
                if e.arvore.em(&dia).is_some() {
                    e.cursor = dia;
                }
                e.aviso = Some("evento apagado".into());
                e.seguir_cursor();
            }
        }
        (Edicao::Copiar, Some(_), Some(entrada)) => {
            e.registro = Some(Registro::Evento(entrada));
            e.aviso = Some("evento copiado".into());
        }
        (Edicao::Colar { .. }, _, _) => {
            let Some(Registro::Evento(mut evento)) = e.registro.clone() else {
                e.aviso = Some("não há evento copiado".into());
                return true;
            };
            let Some(data) = data else {
                e.aviso = Some("escolha um dia pra colar o evento".into());
                return true;
            };
            if let (Some(antes), Some(fim)) = (&evento.date, &evento.end_date) {
                let dias = days_between(antes, fim).unwrap_or(0);
                evento.end_date = add_days(&data, dias);
            }
            evento.date = Some(data.clone());
            let mut novo = 0;
            if editar_calendario(e, &embed, |d| {
                d.entries.push(evento);
                novo = d.entries.len() - 1;
                Ok(())
            }) {
                let destino = achar_evento(&e.arvore, &embed, novo, Some(&data));
                ir(e, destino);
            }
        }
        (Edicao::Deslocar(n), Some(indice), Some(entrada)) => {
            let Some(inicio) = entrada.date.clone() else {
                e.aviso = Some("evento sem data não se move por dia".into());
                return true;
            };
            let Some(novo) = add_days(&inicio, n) else { return true };
            if editar_calendario(e, &embed, |d| {
                d.move_entry(indice, novo.clone());
                Ok(())
            }) {
                // O cursor acompanha o evento; se ele saiu do que está na
                // tela, a âncora vai atrás.
                let alvo = data.and_then(|d| add_days(&d, n)).unwrap_or(novo);
                let mut achou = achar_evento(&e.arvore, &embed, indice, Some(&alvo));
                if achou.is_none() {
                    e.ancorar(&embed, alvo.clone());
                    achou = achar_evento(&e.arvore, &embed, indice, Some(&alvo));
                }
                ir(e, achou);
            }
        }
        (Edicao::Somar(n), Some(indice), Some(mut entrada)) => {
            let Some(inicio) = entrada.date.clone() else {
                e.aviso = Some("evento sem data não tem duração".into());
                return true;
            };
            let fim = entrada.end_date.clone().unwrap_or_else(|| inicio.clone());
            let novo = add_days(&fim, n).filter(|f| days_between(&inicio, f).unwrap_or(0) > 0);
            entrada.end_date = novo;
            if editar_calendario(e, &embed, |d| {
                d.update_entry(indice, entrada);
                Ok(())
            }) {
                let destino = achar_evento(&e.arvore, &embed, indice, data.as_deref());
                ir(e, destino);
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Kanban (ciclo 319)
// ---------------------------------------------------------------------

fn no_kanban(e: &mut Estado, ed: Edicao) -> bool {
    let Some(embed) = embed_do_cursor(e, "kanban") else { return false };
    let Some(&coluna) = e.cursor.get(embed.len()) else {
        e.aviso = Some("entre numa coluna pra editar o kanban".into());
        return true;
    };
    let Some(dados) = ler_kanban(e, &embed) else { return false };
    let nome = dados.columns.get(coluna).cloned().unwrap_or_default();
    let indice = indice_do_cursor(e);
    let cartao = indice.and_then(|i| dados.items.get(i).cloned());
    // Os itens do arquivo desta coluna, na ordem da tela.
    let da_coluna: Vec<usize> =
        dados.items.iter().enumerate().filter(|(_, c)| c.column == nome).map(|(i, _)| i).collect();
    // Antes de qual item do arquivo entra um cartão novo: ao lado do
    // cartão do cursor, ou numa ponta da coluna.
    let antes_de = |antes: bool| -> Option<usize> {
        match indice {
            Some(i) if antes => Some(i),
            Some(i) => da_coluna.iter().copied().find(|j| *j > i),
            None if antes => da_coluna.first().copied(),
            None => None,
        }
    };
    // Na COLUNA (não num cartão), os comandos são da coluna (ciclo 337):
    // ela é o item, os cartões são o que ela contém — o mesmo desenho dos
    // painéis de colunas.
    if indice.is_none() {
        return na_coluna_do_kanban(e, ed, embed, coluna, dados);
    }
    match (ed, indice, cartao) {
        (Edicao::Criar { antes }, _, _) => {
            let antes_de = antes_de(antes);
            perguntar(e, format!("Novo cartão em {nome}"), String::new(), AcaoDaPergunta::NovoCartao {
                embed,
                coluna: nome,
                antes_de,
            });
        }
        (Edicao::Reescrever { limpar, .. }, Some(indice), Some(cartao)) => {
            let texto = if limpar { String::new() } else { cartao.title };
            perguntar(e, "Renomear cartão", texto, AcaoDaPergunta::RenomearCartao { embed, indice });
        }
        (Edicao::Reescrever { limpar, .. }, None, _) => {
            let texto = if limpar { String::new() } else { nome };
            perguntar(e, "Renomear coluna", texto, AcaoDaPergunta::RenomearColuna { embed, coluna });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some(indice), Some(cartao)) => {
            if editar_kanban(e, &embed, |d| {
                d.remove_card(indice);
                Ok(())
            }) {
                e.registro = Some(Registro::Cartao(cartao));
                e.cursor = [embed.as_slice(), &[coluna]].concat();
                e.aviso = Some("cartão apagado".into());
                e.seguir_cursor();
            }
        }
        (Edicao::Copiar, Some(_), Some(cartao)) => {
            e.registro = Some(Registro::Cartao(cartao));
            e.aviso = Some("cartão copiado".into());
        }
        (Edicao::Colar { antes }, _, _) => {
            let Some(Registro::Cartao(mut novo)) = e.registro.clone() else {
                e.aviso = Some("não há cartão copiado".into());
                return true;
            };
            novo.column = nome.clone();
            let antes_de = antes_de(antes);
            let mut onde = 0;
            if editar_kanban(e, &embed, |d| {
                d.items.push(novo);
                onde = d.items.len() - 1;
                if let Some(b) = antes_de {
                    d.move_card(onde, nome.clone(), Some(b));
                    onde = b;
                }
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "card", onde);
                ir(e, destino);
            }
        }
        // `J`/`K`: o cartão desce/sobe DENTRO da coluna (ciclo 338).
        (Edicao::Reordenar(n), Some(indice), Some(_)) => {
            let Some(pos) = da_coluna.iter().position(|i| *i == indice) else { return false };
            let destino = (pos as i64 + n).clamp(0, da_coluna.len() as i64 - 1) as usize;
            if destino == pos {
                e.aviso = Some("não há cartão desse lado".into());
                return true;
            }
            let antes_de = if destino > pos { da_coluna.get(destino + 1).copied() } else { Some(da_coluna[destino]) };
            if editar_kanban(e, &embed, |d| {
                d.move_card(indice, nome.clone(), antes_de);
                Ok(())
            }) {
                ir(e, Some([embed.as_slice(), &[coluna, destino]].concat()));
            }
        }
        (Edicao::Deslocar(n), Some(indice), Some(mut cartao)) => {
            let destino = (coluna as i64 + n).clamp(0, dados.columns.len() as i64 - 1) as usize;
            if destino == coluna {
                e.aviso = Some("não há coluna desse lado".into());
                return true;
            }
            cartao.column = dados.columns[destino].clone();
            if editar_kanban(e, &embed, |d| {
                d.update_card(indice, cartao);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "card", indice);
                ir(e, destino);
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Cronograma (ciclo 320)
// ---------------------------------------------------------------------

/// Troca a escala do cronograma no arquivo (ciclo 332). Vale também no
/// modo vault: a escala é da tela, não das barras.
pub(super) fn trocar_escala(e: &mut Estado, embed: &[usize], escala: em::TimelineScale) {
    if editar_embed(e, embed, |dados| match dados {
        EmbedData::Timeline(d) => {
            d.scale = escala;
            Ok(())
        }
        _ => Err("isto não é um cronograma".into()),
    }) {
        e.aviso = Some(format!("escala: {}", escala.label()));
    }
}

fn no_cronograma(e: &mut Estado, ed: Edicao) -> bool {
    let Some(embed) = embed_do_cursor(e, "timeline") else { return false };
    // `~` alterna entre os itens do embed e as páginas do vault — o botão
    // "Manual/Vault" da janela.
    if ed == Edicao::Alternar {
        let mut vault = false;
        if editar_embed(e, &embed, |dados| match dados {
            EmbedData::Timeline(d) => {
                d.source = if d.source == em::TimelineSource::Vault {
                    em::TimelineSource::Manual
                } else {
                    em::TimelineSource::Vault
                };
                vault = d.source == em::TimelineSource::Vault;
                Ok(())
            }
            _ => Err("isto não é um cronograma".into()),
        }) {
            e.aviso = Some(if vault { "fonte: páginas do vault (só leitura)" } else { "fonte: itens do embed" }.into());
            if e.arvore.em(&e.cursor).is_none() {
                e.cursor = embed;
            }
            e.seguir_cursor();
        }
        return true;
    }
    let Some(dados) = ler_cronograma(e, &embed) else { return false };
    let indice = indice_do_cursor(e);
    let barra = indice.and_then(|i| dados.items.get(i).cloned());
    // Onde cabe uma barra de `dias` dias ao lado da do cursor: depois do
    // fim dela, ou terminando na véspera do começo. Fora de uma barra,
    // hoje — ou o começo da janela.
    let lugar = |antes: bool, dias: i64| -> Option<(String, String)> {
        let ao_lado = barra.as_ref().and_then(|b| {
            let inicio = b.start.clone()?;
            let fim = b.end.clone().unwrap_or_else(|| inicio.clone());
            if antes {
                let f = add_days(&inicio, -1)?;
                Some((add_days(&f, -dias)?, f))
            } else {
                let i = add_days(&fim, 1)?;
                Some((i.clone(), add_days(&i, dias)?))
            }
        });
        ao_lado.or_else(|| {
            let i = e.hoje.clone().or_else(|| dados.items.iter().filter_map(|i| i.start.clone()).min())?;
            Some((i.clone(), add_days(&i, dias)?))
        })
    };
    let posicao = |antes: bool| indice.map(|i| if antes { i } else { i + 1 }).unwrap_or(dados.items.len());
    match (ed, indice, barra.clone()) {
        (Edicao::Criar { antes }, _, _) => {
            let Some((inicio, fim)) = lugar(antes, 6) else {
                e.aviso = Some("sem data de referência pra criar a barra".into());
                return true;
            };
            perguntar(
                e,
                format!("Nova barra de {} a {}", data_legivel(&inicio), data_legivel(&fim)),
                String::new(),
                AcaoDaPergunta::NovaBarra { embed, inicio, fim, posicao: posicao(antes) },
            );
        }
        (Edicao::Reescrever { limpar, .. }, Some(indice), Some(barra)) => {
            let texto = if limpar { String::new() } else { barra.title };
            perguntar(e, "Renomear barra", texto, AcaoDaPergunta::RenomearBarra { embed, indice });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some(indice), Some(barra)) => {
            if editar_cronograma(e, &embed, |d| {
                d.remove_item(indice);
                Ok(())
            }) {
                e.registro = Some(Registro::Barra(barra));
                e.aviso = Some("barra apagada".into());
                e.seguir_cursor();
            }
        }
        (Edicao::Copiar, Some(_), Some(barra)) => {
            e.registro = Some(Registro::Barra(barra));
            e.aviso = Some("barra copiada".into());
        }
        (Edicao::Colar { antes }, _, _) => {
            let Some(Registro::Barra(mut nova)) = e.registro.clone() else {
                e.aviso = Some("não há barra copiada".into());
                return true;
            };
            if let Some(inicio) = nova.start.clone() {
                let dias = nova.end.as_deref().and_then(|f| days_between(&inicio, f)).unwrap_or(0);
                if let Some((i, f)) = lugar(antes, dias) {
                    nova.start = Some(i);
                    nova.end = Some(f);
                }
            }
            let onde = posicao(antes);
            if editar_cronograma(e, &embed, |d| {
                d.items.insert(onde.min(d.items.len()), nova);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "barra", onde)
                    .or_else(|| achar_com_indice(&e.arvore, &embed, "item", onde));
                ir(e, destino);
            }
        }
        // `J`/`K`: a barra desce/sobe na pilha (a ordem do arquivo).
        (Edicao::Reordenar(n), Some(indice), Some(barra)) => {
            let destino = (indice as i64 + n).clamp(0, dados.items.len() as i64 - 1) as usize;
            if destino == indice {
                e.aviso = Some("não há barra desse lado".into());
                return true;
            }
            if editar_cronograma(e, &embed, |d| {
                d.items.remove(indice);
                d.items.insert(destino, barra);
                Ok(())
            }) {
                let alvo = achar_com_indice(&e.arvore, &embed, "barra", destino);
                ir(e, alvo);
            }
        }
        (Edicao::Deslocar(n) | Edicao::Somar(n), Some(indice), Some(barra)) => {
            let mover = matches!(ed, Edicao::Deslocar(_));
            let Some(inicio) = barra.start.clone() else {
                e.aviso = Some("item sem data não se move por dia".into());
                return true;
            };
            if editar_cronograma(e, &embed, |d| {
                if mover {
                    d.move_item(indice, add_days(&inicio, n).ok_or("data inválida")?);
                } else {
                    let fim = barra.end.clone().unwrap_or(inicio);
                    d.resize_item(indice, false, add_days(&fim, n).ok_or("data inválida")?);
                }
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "barra", indice);
                ir(e, destino);
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Tabela (ciclo 321)
// ---------------------------------------------------------------------

/// A tabela sob o cursor e a célula dele: (tabela, fileira, coluna) — a
/// fileira 0 é o cabeçalho, a 1 é a primeira linha de dados.
pub(super) fn celula_do_cursor(e: &Estado) -> Option<(Caminho, usize, usize)> {
    let embed = embed_do_cursor(e, "table")?;
    let fileira = *e.cursor.get(embed.len())?;
    // Na fileira inteira (sem célula), vale a primeira coluna.
    let coluna = e.cursor.get(embed.len() + 1).copied().unwrap_or(0);
    Some((embed, fileira, coluna))
}

fn na_tabela(e: &mut Estado, ed: Edicao) -> bool {
    let Some(embed) = embed_do_cursor(e, "table") else { return false };
    let Some(dados) = ler_tabela(e, &embed) else { return false };
    let celula = celula_do_cursor(e).map(|(_, f, c)| (f, c));
    // Onde entra uma linha nova, em índice de DADOS.
    let posicao = |antes: bool| match celula {
        Some((0, _)) => 0,
        Some((f, _)) if antes => f - 1,
        Some((f, _)) => f,
        None if antes => 0,
        None => dados.rows.len(),
    };
    let valor = |f: usize, c: usize| dados.rows.get(f.wrapping_sub(1)).and_then(|r| r.get(c)).cloned().unwrap_or_default();
    // No cabeçalho, os comandos são da COLUNA (ciclo 337).
    if let Some((0, coluna)) = celula {
        if !matches!(ed, Edicao::Desfazer | Edicao::Refazer) {
            return no_cabecalho_da_tabela(e, ed, embed, coluna);
        }
    }
    match (ed, celula) {
        (Edicao::Criar { antes }, _) | (Edicao::Colar { antes }, _) => {
            let colando = matches!(ed, Edicao::Colar { .. });
            let linha = match (&e.registro, colando) {
                (_, false) => vec![String::new(); dados.columns.len()],
                (Some(Registro::Linha(l)), true) => {
                    let mut l = l.clone();
                    l.resize(dados.columns.len(), String::new());
                    l
                }
                _ => {
                    e.aviso = Some("não há linha copiada".into());
                    return true;
                }
            };
            let onde = posicao(antes);
            if editar_tabela(e, &embed, |d| {
                d.rows.insert(onde.min(d.rows.len()), linha);
                Ok(())
            }) {
                let coluna = if colando { celula.map_or(0, |(_, c)| c) } else { 0 };
                ir(e, Some([embed.as_slice(), &[onde + 1, coluna]].concat()));
                if !colando {
                    e.aviso = Some("linha nova: i edita a célula".into());
                }
            }
        }
        (Edicao::Reescrever { limpar, .. }, Some((0, coluna))) => {
            let texto = if limpar { String::new() } else { dados.columns.get(coluna).map(|c| c.name.clone()).unwrap_or_default() };
            perguntar(e, "Renomear coluna", texto, AcaoDaPergunta::RenomearColunaDaTabela { embed, coluna });
        }
        (Edicao::Reescrever { limpar, .. }, Some((f, coluna))) => {
            let texto = if limpar { String::new() } else { valor(f, coluna) };
            let rotulo = dados.columns.get(coluna).map(|c| c.name.clone()).unwrap_or_else(|| "Célula".into());
            perguntar(e, rotulo, texto, AcaoDaPergunta::EditarCelula { embed, linha: f - 1, coluna });
        }
        (Edicao::ApagarConteudo, Some((f, coluna))) if f > 0 => {
            editar_tabela(e, &embed, |d| {
                d.set_cell(f - 1, coluna, String::new());
                Ok(())
            });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some((0, _))) => {
            e.aviso = Some("o cabeçalho não se apaga".into());
        }
        (Edicao::Apagar, Some((f, coluna))) => {
            let linha = dados.rows.get(f - 1).cloned().unwrap_or_default();
            let mut sobram = 0;
            if editar_tabela(e, &embed, |d| {
                d.remove_row(f - 1);
                sobram = d.rows.len();
                Ok(())
            }) {
                e.registro = Some(Registro::Linha(linha));
                e.aviso = Some("linha apagada".into());
                ir(e, Some([embed.as_slice(), &[f.min(sobram), coluna]].concat()));
            }
        }
        // `J`/`K`: a linha desce/sobe (ciclo 338).
        (Edicao::Reordenar(n), Some((f, coluna))) if f > 0 => {
            let destino = (f as i64 - 1 + n).clamp(0, dados.rows.len() as i64 - 1) as usize;
            if destino == f - 1 {
                e.aviso = Some("não há linha desse lado".into());
                return true;
            }
            if editar_tabela(e, &embed, |d| {
                let linha = d.rows.remove(f - 1);
                d.rows.insert(destino, linha);
                Ok(())
            }) {
                ir(e, Some([embed.as_slice(), &[destino + 1, coluna]].concat()));
            }
        }
        (Edicao::Copiar, Some((f, _))) if f > 0 => {
            e.registro = dados.rows.get(f - 1).cloned().map(Registro::Linha);
            e.aviso = Some("linha copiada".into());
        }
        (Edicao::Somar(n), Some((f, coluna))) if f > 0 => {
            let atual = valor(f, coluna);
            let novo = match dados.columns.get(coluna).map(|c| &c.kind) {
                Some(em::ColumnKind::Select { options }) if !options.is_empty() => {
                    let agora = options.iter().position(|o| *o == atual).map_or(-1, |p| p as i64);
                    let base = if agora < 0 && n < 0 { 0 } else { agora };
                    options[(base + n).rem_euclid(options.len() as i64) as usize].clone()
                }
                Some(em::ColumnKind::Checkbox) => (atual != "true").to_string(),
                _ => {
                    e.aviso = Some("Ctrl+A/Ctrl+X trocam a opção de um select".into());
                    return true;
                }
            };
            editar_tabela(e, &embed, |d| {
                d.set_cell(f - 1, coluna, novo);
                Ok(())
            });
        }
        (Edicao::Alternar, Some((f, coluna)))
            if f > 0 && matches!(dados.columns.get(coluna).map(|c| &c.kind), Some(em::ColumnKind::Checkbox)) =>
        {
            let novo = (valor(f, coluna) != "true").to_string();
            editar_tabela(e, &embed, |d| {
                d.set_cell(f - 1, coluna, novo);
                Ok(())
            });
        }
        (_, None) => e.aviso = Some("entre numa célula pra editar a tabela".into()),
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Callout (ciclo 323)
// ---------------------------------------------------------------------

fn no_callout(e: &mut Estado, ed: Edicao) -> bool {
    use super::markdown::{EdicaoDeBloco, Hospedeiro};
    let Some(embed) = embed_do_cursor(e, "callout") else { return false };
    let Some(dados) = ler_callout(e, &embed) else { return false };
    let tipo = e.arvore.em(&e.cursor).map(|u| u.tipo.clone());
    let no_corpo = e.cursor.len() > embed.len() && !matches!(tipo, Some(Tipo::Parte { .. }));
    // Um bloco do corpo é markdown como o da página (ciclo 334) — menos
    // `Ctrl+A` fora de título e `~` fora de item, que continuam sendo do
    // callout: a variante e o "nasce recolhido".
    if no_corpo {
        let do_bloco = match ed {
            Edicao::Somar(_) => matches!(tipo, Some(Tipo::Titulo(_))),
            Edicao::Alternar => matches!(tipo, Some(Tipo::Item)),
            _ => true,
        };
        if do_bloco {
            return super::markdown::no_markdown(e, ed, Hospedeiro::Callout(embed));
        }
    }
    // O primeiro bloco do corpo, pra criar/colar no começo dele.
    let primeiro = e
        .arvore
        .em(&embed)
        .and_then(|u| u.filhos.iter().position(|f| !matches!(f.tipo, Tipo::Parte { .. })))
        .map(|i| [embed.as_slice(), &[i]].concat());
    match ed {
        // Ctrl+A/Ctrl+X giram a variante — é a "opção" do callout.
        Edicao::Somar(n) => {
            let todas = em::CalloutVariant::all();
            let agora = todas.iter().position(|v| *v == dados.variant).unwrap_or(0) as i64;
            let nova = todas[(agora + n).rem_euclid(todas.len() as i64) as usize];
            if editar_callout(e, &embed, |d| {
                d.variant = nova;
                Ok(())
            }) {
                e.aviso = Some(format!("variante: {}", nova.label()));
                e.seguir_cursor();
            }
        }
        Edicao::Alternar => {
            let recolher = !dados.collapsed;
            if editar_callout(e, &embed, |d| {
                d.collapsed = recolher;
                Ok(())
            }) {
                e.aviso = Some(if recolher { "nasce recolhido na janela" } else { "nasce aberto na janela" }.into());
            }
        }
        Edicao::Reescrever { limpar, .. } => {
            let texto = if limpar { String::new() } else { dados.title };
            perguntar(e, "Título do callout", texto, AcaoDaPergunta::TituloDoCallout { embed });
        }
        // No título: o bloco novo é o primeiro do corpo.
        Edicao::Criar { .. } => {
            let alvo = primeiro.clone().unwrap_or_else(|| e.cursor.clone());
            perguntar(
                e,
                "-- INSERÇÃO --",
                String::new(),
                AcaoDaPergunta::Bloco(EdicaoDeBloco {
                    hospedeiro: Hospedeiro::Callout(embed),
                    faixa: 0..0,
                    prefixo: String::new(),
                    de_lista: false,
                    alvo,
                    novo: true,
                    antes: primeiro.is_some(),
                }),
            );
        }
        Edicao::Colar { .. } => {
            let Some(Registro::Bloco(texto)) = e.registro.clone() else {
                e.aviso = Some("não há bloco copiado".into());
                return true;
            };
            editar_callout(e, &embed, |d| {
                d.body = super::markdown::inserir_bloco(&d.body, 0, &texto, false, true).0;
                Ok(())
            });
        }
        Edicao::Apagar | Edicao::ApagarConteudo => {
            if dados.title.trim().is_empty() {
                return false;
            }
            if editar_callout(e, &embed, |d| {
                d.title.clear();
                Ok(())
            }) {
                e.registro = Some(Registro::Bloco(dados.title));
                e.aviso = Some("título apagado".into());
                e.seguir_cursor();
            }
        }
        Edicao::Copiar => {
            e.registro = Some(Registro::Bloco(dados.title));
            e.aviso = Some("copiado".into());
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Ações (ciclo 324)
// ---------------------------------------------------------------------

fn nas_acoes(e: &mut Estado, ed: Edicao) -> bool {
    // Numa lista na horizontal, `J`/`K` são o mesmo que `>>`/`<<`.
    let ed = if let Edicao::Reordenar(n) = ed { Edicao::Deslocar(n) } else { ed };
    let Some(embed) = embed_do_cursor(e, "actions") else { return false };
    let Some(dados) = ler_acoes(e, &embed) else { return false };
    let total = dados.buttons.len();
    // O botão é a célula da fileira única: `embed / 0 / i`.
    let indice = (e.cursor.len() == embed.len() + 2).then(|| e.cursor[embed.len() + 1]).filter(|i| *i < total);
    let botao = indice.and_then(|i| dados.buttons.get(i).cloned());
    let posicao = |antes: bool| indice.map(|i| if antes { i } else { i + 1 }).unwrap_or(if antes { 0 } else { total });
    let no_botao = |i: usize| [embed.as_slice(), &[0, i]].concat();
    match (ed, indice, botao) {
        (Edicao::Criar { antes }, _, _) => {
            perguntar(e, "Novo botão", String::new(), AcaoDaPergunta::NovoBotao { embed, posicao: posicao(antes) });
        }
        (Edicao::Reescrever { limpar, .. }, Some(indice), Some(botao)) => {
            let texto = if limpar { String::new() } else { botao.label };
            perguntar(e, "Rótulo do botão", texto, AcaoDaPergunta::RenomearBotao { embed, indice });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some(indice), Some(botao)) => {
            if editar_acoes(e, &embed, |d| {
                d.remove_button(indice);
                Ok(())
            }) {
                e.registro = Some(Registro::Botao(botao));
                e.aviso = Some("botão apagado".into());
                let destino = if total > 1 { no_botao(indice.min(total - 2)) } else { embed.clone() };
                ir(e, Some(destino));
            }
        }
        (Edicao::Copiar, Some(_), Some(botao)) => {
            e.registro = Some(Registro::Botao(botao));
            e.aviso = Some("botão copiado".into());
        }
        (Edicao::Colar { antes }, _, _) => {
            let Some(Registro::Botao(novo)) = e.registro.clone() else {
                e.aviso = Some("não há botão copiado".into());
                return true;
            };
            let onde = posicao(antes);
            if editar_acoes(e, &embed, |d| {
                d.buttons.insert(onde.min(d.buttons.len()), novo);
                Ok(())
            }) {
                ir(e, Some(no_botao(onde)));
            }
        }
        (Edicao::Deslocar(n), Some(indice), Some(botao)) => {
            let destino = (indice as i64 + n).clamp(0, total as i64 - 1) as usize;
            if destino == indice {
                e.aviso = Some("não há botão desse lado".into());
                return true;
            }
            if editar_acoes(e, &embed, |d| {
                d.buttons.remove(indice);
                d.buttons.insert(destino, botao);
                Ok(())
            }) {
                ir(e, Some(no_botao(destino)));
            }
        }
        (Edicao::Alternar, Some(indice), Some(mut botao)) => {
            let destacar = botao.variant.as_deref() != Some("primary");
            botao.variant = destacar.then(|| "primary".to_string());
            if editar_acoes(e, &embed, |d| {
                d.update_button(indice, botao);
                Ok(())
            }) {
                e.aviso = Some(if destacar { "botão em destaque" } else { "botão comum" }.into());
                e.seguir_cursor();
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Galeria (ciclo 335)
// ---------------------------------------------------------------------

/// A galeria: `a`/`cc` a legenda, `o`/`O` uma imagem nova (pelo caminho,
/// como o "+ imagem" da janela), `dd` tira, `yy`/`p` duplicam, `>>`/`<<`
/// reordenam, `Ctrl+A`/`Ctrl+X` mudam as colunas da grade e `~` gira o
/// tamanho das miniaturas (P → M → G).
fn na_galeria(e: &mut Estado, ed: Edicao) -> bool {
    // Numa lista na horizontal, `J`/`K` são o mesmo que `>>`/`<<`.
    let ed = if let Edicao::Reordenar(n) = ed { Edicao::Deslocar(n) } else { ed };
    let Some(embed) = embed_do_cursor(e, "gallery") else { return false };
    let Some(dados) = ler_galeria(e, &embed) else { return false };
    let indice = indice_do_cursor(e).filter(|i| *i < dados.items.len());
    let item = indice.and_then(|i| dados.items.get(i).cloned());
    let posicao = |antes: bool| indice.map(|i| if antes { i } else { i + 1 }).unwrap_or(dados.items.len());
    match (ed, indice, item) {
        (Edicao::Somar(n), _, _) => {
            let nova = (dados.columns as i64 + n).clamp(1, 6) as u8;
            if editar_galeria(e, &embed, |d| {
                d.columns = nova;
                Ok(())
            }) {
                e.aviso = Some(format!("{nova} colunas"));
                e.seguir_cursor();
            }
        }
        (Edicao::Alternar, _, _) => {
            let todos = em::GallerySize::all();
            let agora = todos.iter().position(|t| *t == dados.size).unwrap_or(0);
            let novo = todos[(agora + 1) % todos.len()];
            if editar_galeria(e, &embed, |d| {
                d.set_size(novo);
                Ok(())
            }) {
                e.aviso = Some(format!("miniaturas {}", novo.label()));
            }
        }
        (Edicao::Criar { antes }, _, _) => {
            perguntar(e, "Imagem (caminho)", "assets/".into(), AcaoDaPergunta::NovaImagem { embed, posicao: posicao(antes) });
        }
        (Edicao::Reescrever { limpar, .. }, Some(indice), Some(item)) => {
            let texto = if limpar { String::new() } else { item.caption };
            perguntar(e, "Legenda", texto, AcaoDaPergunta::LegendaDaImagem { embed, indice });
        }
        (Edicao::Apagar | Edicao::ApagarConteudo, Some(indice), Some(item)) => {
            if editar_galeria(e, &embed, |d| {
                d.remove_item(indice);
                Ok(())
            }) {
                e.registro = Some(Registro::Imagem(item));
                e.aviso = Some("imagem tirada da galeria".into());
                e.seguir_cursor();
            }
        }
        (Edicao::Copiar, Some(_), Some(item)) => {
            e.registro = Some(Registro::Imagem(item));
            e.aviso = Some("imagem copiada".into());
        }
        (Edicao::Colar { antes }, _, _) => {
            let Some(Registro::Imagem(nova)) = e.registro.clone() else {
                e.aviso = Some("não há imagem copiada".into());
                return true;
            };
            let onde = posicao(antes);
            if editar_galeria(e, &embed, |d| {
                d.items.insert(onde.min(d.items.len()), nova);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "miniatura", onde);
                ir(e, destino);
            }
        }
        (Edicao::Deslocar(n), Some(indice), Some(item)) => {
            let destino = (indice as i64 + n).clamp(0, dados.items.len() as i64 - 1) as usize;
            if destino == indice {
                e.aviso = Some("não há imagem desse lado".into());
                return true;
            }
            if editar_galeria(e, &embed, |d| {
                d.items.remove(indice);
                d.items.insert(destino, item);
                Ok(())
            }) {
                let destino = achar_com_indice(&e.arvore, &embed, "miniatura", destino);
                ir(e, destino);
            }
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Painéis de colunas (ciclo 335)
// ---------------------------------------------------------------------

/// Os painéis: `o`/`O` criam um vazio ao lado (até 4), `dd` tira (fica
/// pelo menos um), `yy`/`p` duplicam, `>>`/`<<` reordenam, `Ctrl+A`/
/// `Ctrl+X` alargam e estreitam, e `i`/`a` começam a escrever no começo
/// do painel. Dentro dele, o markdown é o da página (`markdown.rs`).
fn nos_paineis(e: &mut Estado, ed: Edicao) -> bool {
    // Numa lista na horizontal, `J`/`K` são o mesmo que `>>`/`<<`.
    let ed = if let Edicao::Reordenar(n) = ed { Edicao::Deslocar(n) } else { ed };
    let Some(embed) = embed_do_cursor(e, "columns") else { return false };
    let Some(dados) = ler_colunas(e, &embed) else { return false };
    let Some(&painel) = e.cursor.get(embed.len()) else {
        e.aviso = Some("entre num painel pra editar as colunas".into());
        return true;
    };
    let total = dados.columns.len();
    let no_painel = |i: usize| [embed.as_slice(), &[i]].concat();
    match ed {
        Edicao::Criar { antes } | Edicao::Colar { antes } => {
            let colando = matches!(ed, Edicao::Colar { .. });
            let novo = match (&e.registro, colando) {
                (_, false) => em::ColumnPane { width: 1, body: String::new() },
                (Some(Registro::Painel(p)), true) => p.clone(),
                _ => {
                    e.aviso = Some("não há painel copiado".into());
                    return true;
                }
            };
            if total >= em::ColumnsEmbedData::MAX_COLUMNS {
                e.aviso = Some(format!("no máximo {} colunas", em::ColumnsEmbedData::MAX_COLUMNS));
                return true;
            }
            let onde = if antes { painel } else { painel + 1 };
            if editar_colunas(e, &embed, |d| {
                d.columns.insert(onde.min(d.columns.len()), novo);
                Ok(())
            }) {
                ir(e, Some(no_painel(onde)));
            }
        }
        Edicao::Apagar | Edicao::ApagarConteudo => {
            if total <= 1 {
                e.aviso = Some("fica pelo menos um painel".into());
                return true;
            }
            let tirado = dados.columns[painel].clone();
            if editar_colunas(e, &embed, |d| {
                d.remove_column(painel);
                Ok(())
            }) {
                e.registro = Some(Registro::Painel(tirado));
                e.aviso = Some("painel apagado".into());
                ir(e, Some(no_painel(painel.min(total - 2))));
            }
        }
        Edicao::Copiar => {
            e.registro = Some(Registro::Painel(dados.columns[painel].clone()));
            e.aviso = Some("painel copiado".into());
        }
        Edicao::Deslocar(n) => {
            let destino = (painel as i64 + n).clamp(0, total as i64 - 1) as usize;
            if destino == painel {
                e.aviso = Some("não há painel desse lado".into());
                return true;
            }
            if editar_colunas(e, &embed, |d| {
                let p = d.columns.remove(painel);
                d.columns.insert(destino, p);
                Ok(())
            }) {
                ir(e, Some(no_painel(destino)));
            }
        }
        Edicao::Somar(n) => {
            let nova = (dados.columns[painel].width as i64 + n).clamp(1, 6) as u8;
            if editar_colunas(e, &embed, |d| {
                d.columns[painel].width = nova;
                Ok(())
            }) {
                e.aviso = Some(format!("largura {nova}fr"));
                e.seguir_cursor();
            }
        }
        Edicao::Reescrever { .. } => {
            let alvo = e
                .arvore
                .em(&no_painel(painel))
                .and_then(|u| u.filhos.iter().position(|f| !matches!(f.tipo, Tipo::Parte { .. })))
                .map(|k| [embed.as_slice(), &[painel, k]].concat())
                .unwrap_or_else(|| no_painel(painel));
            perguntar(
                e,
                "-- INSERÇÃO --",
                String::new(),
                AcaoDaPergunta::Bloco(super::markdown::EdicaoDeBloco {
                    hospedeiro: super::markdown::Hospedeiro::Painel(embed, painel),
                    faixa: 0..0,
                    prefixo: String::new(),
                    de_lista: false,
                    alvo,
                    novo: true,
                    antes: true,
                }),
            );
        }
        _ => return false,
    }
    true
}

// ---------------------------------------------------------------------
// Fluxo (ciclo 335)
// ---------------------------------------------------------------------

/// O fluxo: `Enter` numa transição move pra ela (`fluxo_do_cursor`),
/// `>>` dá o avanço natural, `a`/`cc` editam a nota.
fn no_fluxo(e: &mut Estado, ed: Edicao) -> bool {
    let Some(embed) = embed_do_cursor(e, "fluxo") else { return false };
    let Some(dados) = ler_fluxo(e, &embed) else { return false };
    match ed {
        Edicao::Deslocar(n) if n > 0 => {
            let Some(destino) = dados.etapa.avanco_natural() else {
                e.aviso = Some("esta etapa não tem avanço natural".into());
                return true;
            };
            mover_fluxo(e, &embed, destino);
        }
        Edicao::Reescrever { limpar, .. } => {
            let texto = if limpar { String::new() } else { dados.nota.unwrap_or_default() };
            perguntar(e, "Nota", texto, AcaoDaPergunta::NotaDoFluxo { embed });
        }
        Edicao::Apagar | Edicao::ApagarConteudo if dados.nota.is_some() => {
            if editar_fluxo(e, &embed, |d| {
                d.nota = None;
                Ok(())
            }) {
                e.aviso = Some("nota apagada".into());
            }
        }
        _ => return false,
    }
    true
}

fn mover_fluxo(e: &mut Estado, embed: &[usize], destino: anotadinho_core::fluxo::Etapa) {
    if editar_fluxo(e, embed, |d| {
        if d.ir_para(destino, d.nota.clone()) {
            Ok(())
        } else {
            Err(format!("não dá pra ir pra {}", destino.label()))
        }
    }) {
        e.aviso = Some(format!("etapa: {}", destino.label()));
        if e.arvore.em(&e.cursor).is_none() {
            e.cursor = embed.to_vec();
        }
        e.seguir_cursor();
    }
}

/// `Enter` num botão de transição do fluxo: move pra etapa dele. Devolve
/// se a tecla foi usada.
pub(super) fn transicao_do_cursor(e: &mut Estado) -> bool {
    let Some(embed) = embed_do_cursor(e, "fluxo") else { return false };
    let Some(u) = e.arvore.em(&e.cursor) else { return false };
    if !matches!(&u.tipo, Tipo::Parte { nome, .. } if nome.starts_with("transicao")) {
        return false;
    }
    let Some(destino) = anotadinho_core::fluxo::Etapa::all().iter().copied().find(|x| x.label() == u.texto) else {
        return false;
    };
    mover_fluxo(e, &embed, destino);
    true
}

/// Enter na ação do fluxo (ciclo 348): "Planejar implementação",
/// "Executar" ou "Pedir alteração" abrem a conversa com a página anexada,
/// como na janela.
pub(super) fn acao_do_fluxo(e: &mut Estado) -> bool {
    if embed_do_cursor(e, "fluxo").is_none() {
        return false;
    }
    let Some(u) = e.arvore.em(&e.cursor) else { return false };
    if !matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "acao") || u.texto.is_empty() {
        return false;
    }
    let Some(pagina) = e.paginas.get(e.pagina).map(|p| p.path.clone()) else { return false };
    let alterar = u.texto == "Pedir alteração";
    e.pedidos.push(super::modais::Pedido::ConversaDoFluxo { pagina, alterar });
    true
}

// ---------------------------------------------------------------------
// Consulta (ciclo 335)
// ---------------------------------------------------------------------

/// A consulta: `a`/`cc` editam o recorte como linha de filtro
/// (`from:pages status=done sort:-date limit:10`), `~` gira a visão
/// (lista → tabela → cartões) e `Ctrl+A`/`Ctrl+X` mudam o limite.
fn na_consulta(e: &mut Estado, ed: Edicao) -> bool {
    use anotadinho_core::query::QueryView;
    let Some(embed) = embed_do_cursor(e, "query") else { return false };
    let Some(q) = ler_consulta(e, &embed) else { return false };
    match ed {
        Edicao::Reescrever { limpar, .. } => {
            let texto = if limpar { String::new() } else { q.linha_de_filtro() };
            perguntar(e, "Filtro", texto, AcaoDaPergunta::FiltroDaConsulta { embed: embed.clone() });
        }
        Edicao::Alternar => {
            let nova = match q.view {
                QueryView::List => QueryView::Table,
                QueryView::Table => QueryView::Cards,
                QueryView::Cards => QueryView::List,
            };
            if editar_consulta(e, &embed, |q| {
                q.view = nova;
                Ok(())
            }) {
                e.aviso = Some(format!("visão: {}", nova.label()));
            }
        }
        Edicao::Somar(n) => {
            let novo = (q.limit.unwrap_or(10) as i64 + n).max(1) as usize;
            if editar_consulta(e, &embed, |q| {
                q.limit = Some(novo);
                Ok(())
            }) {
                e.aviso = Some(format!("limite {novo}"));
            }
        }
        _ => return false,
    }
    if e.arvore.em(&e.cursor).is_none() {
        e.cursor = embed;
    }
    e.seguir_cursor();
    true
}

// ---------------------------------------------------------------------
// Colunas do kanban e da tabela (ciclo 337)
// ---------------------------------------------------------------------

/// Numa coluna do kanban: `o`/`O` criam uma coluna ao lado, `a`/`cc`
/// renomeiam (os cartões seguem), `dd` apaga com os cartões (fica pelo
/// menos uma), `yy`/`p` duplicam com os cartões, `>>`/`<<` mudam a coluna
/// de lugar. Colar um CARTÃO copiado numa coluna põe ele no fim dela.
fn na_coluna_do_kanban(e: &mut Estado, ed: Edicao, embed: Caminho, coluna: usize, dados: em::KanbanEmbedData) -> bool {
    // Numa lista na horizontal, `J`/`K` são o mesmo que `>>`/`<<`.
    let ed = if let Edicao::Reordenar(n) = ed { Edicao::Deslocar(n) } else { ed };
    let total = dados.columns.len();
    let Some(nome) = dados.columns.get(coluna).cloned() else { return false };
    let cartoes: Vec<em::KanbanCard> = dados.items.iter().filter(|c| c.column == nome).cloned().collect();
    let na = |i: usize| [embed.as_slice(), &[i]].concat();
    match ed {
        Edicao::Criar { antes } => {
            let posicao = if antes { coluna } else { coluna + 1 };
            perguntar(e, "Nova coluna", String::new(), AcaoDaPergunta::NovaColunaDoKanban { embed, posicao });
        }
        Edicao::Reescrever { limpar, .. } => {
            let texto = if limpar { String::new() } else { nome };
            perguntar(e, "Renomear coluna", texto, AcaoDaPergunta::RenomearColuna { embed, coluna });
        }
        Edicao::Apagar | Edicao::ApagarConteudo => {
            if total <= 1 {
                e.aviso = Some("fica pelo menos uma coluna".into());
                return true;
            }
            if editar_kanban(e, &embed, |d| {
                d.remove_column(coluna);
                Ok(())
            }) {
                e.registro = Some(Registro::ColunaDoKanban(nome.clone(), cartoes));
                e.aviso = Some(format!("coluna {nome} apagada"));
                ir(e, Some(na(coluna.min(total - 2))));
            }
        }
        Edicao::Copiar => {
            e.registro = Some(Registro::ColunaDoKanban(nome, cartoes));
            e.aviso = Some("coluna copiada".into());
        }
        Edicao::Colar { antes } => match e.registro.clone() {
            Some(Registro::ColunaDoKanban(copiada, cartoes)) => {
                let onde = if antes { coluna } else { coluna + 1 };
                // Nome repetido separaria mal os cartões: a cópia ganha
                // um sufixo.
                let mut novo_nome = copiada.clone();
                while dados.columns.contains(&novo_nome) {
                    novo_nome.push_str(" (cópia)");
                }
                if editar_kanban(e, &embed, |d| {
                    d.columns.insert(onde.min(d.columns.len()), novo_nome.clone());
                    for mut c in cartoes {
                        c.column = novo_nome.clone();
                        d.items.push(c);
                    }
                    Ok(())
                }) {
                    ir(e, Some(na(onde)));
                }
            }
            Some(Registro::Cartao(mut cartao)) => {
                cartao.column = nome.clone();
                let mut onde = 0;
                if editar_kanban(e, &embed, |d| {
                    d.items.push(cartao);
                    onde = d.items.len() - 1;
                    Ok(())
                }) {
                    let destino = achar_com_indice(&e.arvore, &embed, "card", onde);
                    ir(e, destino);
                }
            }
            _ => e.aviso = Some("não há coluna nem cartão copiados".into()),
        },
        Edicao::Deslocar(n) => {
            let destino = (coluna as i64 + n).clamp(0, total as i64 - 1) as usize;
            if destino == coluna {
                e.aviso = Some("não há coluna desse lado".into());
                return true;
            }
            if editar_kanban(e, &embed, |d| {
                let c = d.columns.remove(coluna);
                d.columns.insert(destino, c);
                Ok(())
            }) {
                ir(e, Some(na(destino)));
            }
        }
        _ => return false,
    }
    true
}

/// `Enter` numa coluna VAZIA do kanban abre o primeiro cartão dela — o
/// "+ card" da janela. Devolve se a tecla foi usada.
pub(super) fn cartao_na_coluna_vazia(e: &mut Estado) -> bool {
    let Some(embed) = embed_do_cursor(e, "kanban") else { return false };
    if e.cursor.len() != embed.len() + 1 {
        return false;
    }
    let Some(u) = e.arvore.em(&e.cursor) else { return false };
    if !u.filhos.is_empty() {
        return false;
    }
    let coluna = u.texto.clone();
    perguntar(e, format!("Novo cartão em {coluna}"), String::new(), AcaoDaPergunta::NovoCartao { embed, coluna, antes_de: None });
    true
}

/// Leva a coluna `de` da tabela pra posição `para`, com as células.
fn mover_coluna_da_tabela(d: &mut em::TableEmbedData, de: usize, para: usize) {
    if de == para || de >= d.columns.len() {
        return;
    }
    let c = d.columns.remove(de);
    d.columns.insert(para.min(d.columns.len()), c);
    for linha in &mut d.rows {
        if de < linha.len() {
            let v = linha.remove(de);
            linha.insert(para.min(linha.len()), v);
        }
    }
}

/// Os tipos de coluna na ordem em que `~` gira (a do seletor da janela).
fn tipos_de_coluna() -> [&'static str; 8] {
    ["texto", "número", "data", "caixa", "seleção", "tags", "url", "página"]
}

fn nome_do_tipo(k: &em::ColumnKind) -> &'static str {
    match k {
        em::ColumnKind::Text => "texto",
        em::ColumnKind::Number => "número",
        em::ColumnKind::Date => "data",
        em::ColumnKind::Checkbox => "caixa",
        em::ColumnKind::Select { .. } => "seleção",
        em::ColumnKind::MultiSelect { .. } => "tags",
        em::ColumnKind::Url => "url",
        em::ColumnKind::PageLink => "página",
    }
}

/// O tipo pelo nome, com as opções tiradas dos valores que a coluna já
/// tem — trocar pra seleção não pode perder o que estava escrito.
fn tipo_pelo_nome(nome: &str, valores: &[String]) -> em::ColumnKind {
    let opcoes = |separar: bool| -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for x in valores {
            let partes: Vec<&str> = if separar { x.split(',').collect() } else { vec![x.as_str()] };
            for p in partes.into_iter().map(str::trim).filter(|p| !p.is_empty()) {
                if !v.iter().any(|o| o == p) {
                    v.push(p.to_string());
                }
            }
        }
        v
    };
    match nome {
        "número" => em::ColumnKind::Number,
        "data" => em::ColumnKind::Date,
        "caixa" => em::ColumnKind::Checkbox,
        "seleção" => em::ColumnKind::Select { options: opcoes(false) },
        "tags" => em::ColumnKind::MultiSelect { options: opcoes(true) },
        "url" => em::ColumnKind::Url,
        "página" => em::ColumnKind::PageLink,
        _ => em::ColumnKind::Text,
    }
}

/// No cabeçalho da tabela: `o`/`O` criam uma coluna de texto ao lado,
/// `a`/`cc` renomeiam, `dd`/`x` apagam (fica pelo menos uma), `yy`/`p`
/// duplicam com as células, `>>`/`<<` mudam de lugar, e `~`
/// (`Ctrl+A`/`Ctrl+X` pros dois lados) gira o tipo — texto, número, data,
/// caixa, seleção, tags, url, página.
pub(super) fn no_cabecalho_da_tabela(e: &mut Estado, ed: Edicao, embed: Caminho, coluna: usize) -> bool {
    // Numa lista na horizontal, `J`/`K` são o mesmo que `>>`/`<<`.
    let ed = if let Edicao::Reordenar(n) = ed { Edicao::Deslocar(n) } else { ed };
    let Some(dados) = ler_tabela(e, &embed) else { return false };
    let total = dados.columns.len();
    let Some(col) = dados.columns.get(coluna).cloned() else { return false };
    let celulas: Vec<String> = dados.rows.iter().map(|r| r.get(coluna).cloned().unwrap_or_default()).collect();
    let no_cabecalho = |i: usize| [embed.as_slice(), &[0, i]].concat();
    match ed {
        Edicao::Criar { antes } => {
            let posicao = if antes { coluna } else { coluna + 1 };
            perguntar(e, "Nova coluna", String::new(), AcaoDaPergunta::NovaColunaDaTabela { embed, posicao });
        }
        Edicao::Reescrever { limpar, .. } => {
            let texto = if limpar { String::new() } else { col.name };
            perguntar(e, "Renomear coluna", texto, AcaoDaPergunta::RenomearColunaDaTabela { embed, coluna });
        }
        Edicao::Apagar | Edicao::ApagarConteudo => {
            if total <= 1 {
                e.aviso = Some("fica pelo menos uma coluna".into());
                return true;
            }
            if editar_tabela(e, &embed, |d| {
                d.remove_column(coluna);
                Ok(())
            }) {
                e.registro = Some(Registro::ColunaDaTabela(col.clone(), celulas));
                e.aviso = Some(format!("coluna {} apagada", col.name));
                ir(e, Some(no_cabecalho(coluna.min(total - 2))));
            }
        }
        Edicao::Copiar => {
            e.registro = Some(Registro::ColunaDaTabela(col, celulas));
            e.aviso = Some("coluna copiada".into());
        }
        Edicao::Colar { antes } => {
            let Some(Registro::ColunaDaTabela(copiada, valores)) = e.registro.clone() else {
                e.aviso = Some("não há coluna copiada".into());
                return true;
            };
            let onde = if antes { coluna } else { coluna + 1 };
            if editar_tabela(e, &embed, |d| {
                d.columns.push(copiada);
                for (k, linha) in d.rows.iter_mut().enumerate() {
                    linha.push(valores.get(k).cloned().unwrap_or_default());
                }
                let fim = d.columns.len() - 1;
                mover_coluna_da_tabela(d, fim, onde);
                Ok(())
            }) {
                ir(e, Some(no_cabecalho(onde)));
            }
        }
        Edicao::Deslocar(n) => {
            let destino = (coluna as i64 + n).clamp(0, total as i64 - 1) as usize;
            if destino == coluna {
                e.aviso = Some("não há coluna desse lado".into());
                return true;
            }
            if editar_tabela(e, &embed, |d| {
                mover_coluna_da_tabela(d, coluna, destino);
                Ok(())
            }) {
                ir(e, Some(no_cabecalho(destino)));
            }
        }
        Edicao::Alternar | Edicao::Somar(_) => {
            let passo = if let Edicao::Somar(n) = ed { n } else { 1 };
            let tipos = tipos_de_coluna();
            let agora = tipos.iter().position(|t| *t == nome_do_tipo(&col.kind)).unwrap_or(0) as i64;
            let novo = tipos[(agora + passo).rem_euclid(tipos.len() as i64) as usize];
            if editar_tabela(e, &embed, |d| {
                d.set_column_kind(coluna, tipo_pelo_nome(novo, &celulas));
                Ok(())
            }) {
                e.aviso = Some(format!("{}: {novo}", col.name));
                ir(e, None);
            }
        }
        Edicao::Desfazer | Edicao::Refazer | Edicao::Reordenar(_) => return false,
    }
    true
}

/// `Enter` na barra de busca de uma consulta: edita o filtro (ciclo 338).
pub(super) fn busca_da_consulta(e: &mut Estado) -> bool {
    if embed_do_cursor(e, "query").is_none() {
        return false;
    }
    if !e.arvore.em(&e.cursor).is_some_and(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "busca")) {
        return false;
    }
    na_consulta(e, Edicao::Reescrever { limpar: false, no_fim: true })
}

// ---------------------------------------------------------------------
// Opções de uma coluna de seleção (ciclo 339)
// ---------------------------------------------------------------------

fn opcoes_da_coluna(e: &Estado, embed: &[usize], coluna: usize) -> Option<(String, Vec<String>, bool)> {
    let d = ler_tabela(e, embed)?;
    let c = d.columns.get(coluna)?;
    match &c.kind {
        em::ColumnKind::Select { options } => Some((c.name.clone(), options.clone(), false)),
        em::ColumnKind::MultiSelect { options } => Some((c.name.clone(), options.clone(), true)),
        _ => None,
    }
}

fn lista_de_opcoes(opcoes: &[String], multi: bool) -> crate::componentes::Lista {
    crate::componentes::Lista::menu(
        opcoes
            .iter()
            .enumerate()
            .map(|(i, o)| crate::componentes::Item::novo(if multi { "◇" } else { "○" }, o.clone(), i.to_string()))
            .collect(),
    )
}

/// `Enter` no cabeçalho de uma coluna de seleção (ou tags) abre o editor
/// das opções dela — o que o modal de configuração de coluna da janela
/// faz.
pub(super) fn abrir_opcoes(e: &mut Estado) -> bool {
    let Some((embed, 0, coluna)) = celula_do_cursor(e) else { return false };
    if e.cursor.len() != embed.len() + 2 {
        return false;
    }
    let Some((nome, opcoes, multi)) = opcoes_da_coluna(e, &embed, coluna) else { return false };
    e.modal = Some(super::modais::Modal::Opcoes(super::modais::EditorDeOpcoes {
        embed,
        coluna,
        nome,
        lista: lista_de_opcoes(&opcoes, multi),
        editando: None,
        d_pendente: false,
    }));
    true
}

/// Uma tecla no editor de opções: `j`/`k` andam, `o` cria, `a`/`i`/`cc`
/// renomeiam (as células com o nome velho acompanham), `dd` apaga (e tira
/// das células), `J`/`K` reordenam, `Esc` fecha. Escrevendo, `Enter` ou
/// `Esc` confirmam.
pub(super) fn tecla_nas_opcoes(e: &mut Estado, mut ed: super::modais::EditorDeOpcoes, tecla: &str) {
    use super::modais::Modal;
    let recarregar = |e: &Estado, ed: &mut super::modais::EditorDeOpcoes| {
        if let Some((_, opcoes, multi)) = opcoes_da_coluna(e, &ed.embed, ed.coluna) {
            let sel = ed.lista.selecionado;
            ed.lista = lista_de_opcoes(&opcoes, multi);
            ed.lista.selecionado = sel.min(opcoes.len().saturating_sub(1));
        }
    };
    if let Some((alvo, mut campo)) = ed.editando.take() {
        match tecla {
            "Enter" | "Escape" => {
                let nome = campo.texto.trim().to_string();
                let (embed, coluna) = (ed.embed.clone(), ed.coluna);
                if !nome.is_empty() {
                    editar_tabela(e, &embed, |d| {
                        let (opcoes, multi) = match &mut d.columns.get_mut(coluna).ok_or("a coluna sumiu")?.kind {
                            em::ColumnKind::Select { options } => (options, false),
                            em::ColumnKind::MultiSelect { options } => (options, true),
                            _ => return Err("a coluna não é mais de seleção".into()),
                        };
                        if opcoes.contains(&nome) {
                            return Err(format!("já existe a opção {nome}"));
                        }
                        match alvo {
                            None => opcoes.push(nome.clone()),
                            Some(i) => {
                                let velho = opcoes.get(i).cloned().ok_or("a opção sumiu")?;
                                opcoes[i] = nome.clone();
                                for linha in &mut d.rows {
                                    let Some(cel) = linha.get_mut(coluna) else { continue };
                                    if multi {
                                        let partes: Vec<String> = cel
                                            .split(',')
                                            .map(|p| if p.trim() == velho { nome.clone() } else { p.trim().to_string() })
                                            .filter(|p| !p.is_empty())
                                            .collect();
                                        *cel = partes.join(", ");
                                    } else if *cel == velho {
                                        *cel = nome.clone();
                                    }
                                }
                            }
                        }
                        Ok(())
                    });
                    recarregar(e, &mut ed);
                    if alvo.is_none() {
                        ed.lista.selecionado = ed.lista.itens.len().saturating_sub(1);
                    }
                }
            }
            outra => {
                campo.tecla(outra);
                ed.editando = Some((alvo, campo));
            }
        }
        e.modal = Some(Modal::Opcoes(ed));
        return;
    }
    let atual = ed.lista.selecionado;
    let total = ed.lista.itens.len();
    let d_antes = std::mem::take(&mut ed.d_pendente);
    match tecla {
        "Escape" | "q" => return,
        "o" | "O" => ed.editando = Some((None, crate::componentes::Campo::default())),
        "a" | "i" | "A" | "Enter" if total > 0 => {
            ed.editando = Some((Some(atual), crate::componentes::Campo::com(ed.lista.itens[atual].rotulo.clone())));
        }
        "c" if total > 0 => ed.editando = Some((Some(atual), crate::componentes::Campo::default())),
        "d" if !d_antes => ed.d_pendente = true,
        "d" | "x" if total > 0 => {
            let opcao = ed.lista.itens[atual].rotulo.clone();
            let (embed, coluna) = (ed.embed.clone(), ed.coluna);
            if editar_tabela(e, &embed, |d| {
                d.remove_column_option(coluna, &opcao);
                Ok(())
            }) {
                e.aviso = Some(format!("opção {opcao} apagada"));
            }
            recarregar(e, &mut ed);
        }
        "J" | "K" if total > 1 => {
            let destino = if tecla == "J" { (atual + 1).min(total - 1) } else { atual.saturating_sub(1) };
            if destino != atual {
                let (embed, coluna) = (ed.embed.clone(), ed.coluna);
                editar_tabela(e, &embed, |d| {
                    if let Some(em::ColumnKind::Select { options } | em::ColumnKind::MultiSelect { options }) =
                        d.columns.get_mut(coluna).map(|c| &mut c.kind)
                    {
                        options.swap(atual, destino);
                    }
                    Ok(())
                });
                recarregar(e, &mut ed);
                ed.lista.selecionado = destino;
            }
        }
        "u" => {
            desfazer(e, true);
            recarregar(e, &mut ed);
        }
        outra => {
            ed.lista.tecla(outra);
        }
    }
    e.modal = Some(Modal::Opcoes(ed));
}

// ---------------------------------------------------------------------
// Executar botões e seguir wikilinks (ciclo 342)
// ---------------------------------------------------------------------

/// `Enter` num botão de ações: faz o que ele declara, como na janela —
/// abrir página, criar a partir de template (pede o título), gravar
/// propriedade ou abrir a busca com o termo.
pub(super) fn acionar_botao(e: &mut Estado) -> bool {
    use super::modais::{AcaoDaEntrada, Modal, Pedido};
    let Some(embed) = embed_do_cursor(e, "actions") else { return false };
    if e.cursor.len() != embed.len() + 2 {
        return false;
    }
    let indice = e.cursor[embed.len() + 1];
    let Some(botao) = ler_acoes(e, &embed).and_then(|d| d.buttons.get(indice).cloned()) else { return false };
    if !botao.is_runnable() {
        e.aviso = Some(format!("\"{}\" não tem destino configurado", botao.label));
        return true;
    }
    match botao.spec() {
        em::ActionSpec::OpenPage { path } => e.pedidos.push(Pedido::AbrirPagina(path)),
        em::ActionSpec::NewFromTemplate { template, folder } => {
            e.modal = Some(Modal::Entrada {
                titulo: "Título da nova página".into(),
                campo: crate::componentes::Campo::default(),
                acao: AcaoDaEntrada::PaginaDeTemplate { template, pasta: folder },
            });
        }
        em::ActionSpec::SetProperty { path, field, value } => {
            e.pedidos.push(Pedido::DefinirPropriedade { path, campo: field, valor: value });
        }
        em::ActionSpec::RunSearch { query } => {
            super::modais::abrir_paleta(e);
            if let Some(Modal::Paleta(l)) = e.modal.as_mut() {
                l.filtro = Some(crate::componentes::Campo::com(query));
            }
        }
        em::ActionSpec::Unknown(a) => e.aviso = Some(format!("ação desconhecida: {a}")),
    }
    true
}

/// A página de um `[[alvo]]`: pelo título (do frontmatter, sem caixa) ou
/// pelo nome do arquivo.
pub(super) fn resolver_wikilink(e: &Estado, alvo: &str) -> Option<String> {
    let alvo = alvo.trim().to_lowercase();
    let pelo_indice = e.indice_do_vault.iter().find(|p| p.title.to_lowercase() == alvo).map(|p| p.path.clone());
    pelo_indice.or_else(|| {
        e.paginas
            .iter()
            .find(|p| {
                p.title.to_lowercase() == alvo
                    || std::path::Path::new(&p.path).file_stem().is_some_and(|s| s.to_string_lossy().to_lowercase() == alvo)
            })
            .map(|p| p.path.clone())
    })
}

/// `Enter` num bloco com `[[wikilink]]`: abre a página (com vários, pergunta
/// qual; o que não existe se oferece pra criar).
pub(super) fn seguir_wikilink(e: &mut Estado) -> bool {
    use super::modais::{AcaoDaEscolha, Modal, Pedido};
    let Some(u) = e.arvore.em(&e.cursor) else { return false };
    if matches!(u.tipo, Tipo::Embed(_) | Tipo::Parte { .. }) {
        return false;
    }
    let alvos = anotadinho_core::links::extract_wikilink_targets(&u.texto);
    if alvos.is_empty() {
        return false;
    }
    if alvos.len() == 1 {
        match resolver_wikilink(e, &alvos[0]) {
            Some(path) => e.pedidos.push(Pedido::AbrirPagina(path)),
            None => {
                e.modal = Some(Modal::Confirmar {
                    titulo: "Página não existe".into(),
                    mensagem: format!("Criar a página \"{}\"?", alvos[0]),
                    acao: Pedido::CriarPaginaComTitulo { titulo: alvos[0].clone(), tipo: None },
                })
            }
        }
        return true;
    }
    let itens = alvos
        .iter()
        .filter_map(|a| resolver_wikilink(e, a).map(|p| crate::componentes::Item::novo("↗", a.clone(), p.clone()).com_detalhe(p)))
        .collect::<Vec<_>>();
    if itens.is_empty() {
        e.aviso = Some("nenhum dos links aponta pra página existente".into());
        return true;
    }
    e.modal = Some(Modal::Escolha { titulo: "Abrir link".into(), lista: crate::componentes::Lista::menu(itens), acao: AcaoDaEscolha::AbrirPagina });
    true
}

// ---------------------------------------------------------------------
// Detalhes do cartão e do evento (ciclo 343)
// ---------------------------------------------------------------------

/// `Enter` num cartão do kanban abre os detalhes dele — o
/// `CardDetailModal` da janela: título, descrição, tags, vencimento,
/// checklist, comentários e anexos.
pub(super) fn abrir_detalhe_do_cartao(e: &mut Estado) -> bool {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let Some(embed) = embed_do_cursor(e, "kanban") else { return false };
    if e.cursor.len() != embed.len() + 2 {
        return false;
    }
    let Some(indice) = indice_do_cursor(e) else { return false };
    let Some(c) = ler_kanban(e, &embed).and_then(|d| d.items.get(indice).cloned()) else { return false };
    let mut form = Formulario::novo(vec![
        C::novo("titulo", "Título", Valor::Texto(c.title.clone())),
        C::novo("descricao", "Descrição", Valor::Texto(c.description.clone().unwrap_or_default())).com_dica("sem descrição"),
        C::novo("tags", "Tags", Valor::Lista(c.tags.clone())).com_dica("tag"),
        C::novo("vencimento", "Vencimento", Valor::Texto(c.due.clone().unwrap_or_default())).com_dica("AAAA-MM-DD"),
        C::novo("checklist", "Checklist", Valor::Checklist(c.checklist.iter().map(|i| (i.done, i.text.clone())).collect()))
            .com_dica("item"),
        C::novo("comentarios", "Comentários", Valor::Lista(c.comments.iter().map(|x| x.text.clone()).collect())).com_dica("comentário"),
        C::novo("anexos", "Anexos", Valor::Lista(c.attachments.iter().map(|x| x.path.clone()).collect())).com_dica("caminho do anexo"),
    ]);
    form.botoes.push(("excluir", "Excluir cartão".into()));
    e.modal = Some(super::modais::Modal::Detalhe {
        titulo: format!("Cartão · {}", c.column),
        form,
        alvo: super::modais::AlvoDoDetalhe::Cartao { embed, indice },
    });
    true
}

/// `Enter` num evento do calendário (fora do modo vault) abre os detalhes
/// — o `EventDetailModal`: título, início, vários dias e fim, horário
/// específico com início e fim, tags.
pub(super) fn abrir_detalhe_do_evento(e: &mut Estado) -> bool {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let Some(embed) = tela::calendario_do_cursor(&e.arvore, &e.cursor) else { return false };
    let Some(indice) = indice_do_cursor(e) else { return false };
    let Some(d) = ler_calendario(e, &embed) else { return false };
    if d.mode == em::CalendarSource::Vault {
        return false;
    }
    let Some(ev) = d.entries.get(indice).cloned() else { return false };
    let varios = ev.end_date.as_ref().is_some_and(|f| Some(f) != ev.date.as_ref());
    let horario = ev.start_time.is_some();
    let mut form = Formulario::novo(vec![
        C::novo("titulo", "Título", Valor::Texto(ev.title.clone())),
        C::novo("inicio", "Início", Valor::Texto(ev.date.clone().unwrap_or_default())).com_dica("AAAA-MM-DD (vazio: sem data)"),
        C::novo("varios", "Vários dias", Valor::Booleano(varios)),
        C::novo("fim", "Fim", Valor::Texto(ev.end_date.clone().unwrap_or_default())).com_dica("AAAA-MM-DD"),
        C::novo("horario", "Horário específico", Valor::Booleano(horario)),
        C::novo("hora_inicio", "Das", Valor::Texto(ev.start_time.clone().unwrap_or_default())).com_dica("HH:MM"),
        C::novo("hora_fim", "Até", Valor::Texto(ev.end_time.clone().unwrap_or_default())).com_dica("HH:MM"),
        C::novo("tags", "Tags", Valor::Lista(ev.all_tags())).com_dica("tag"),
    ]);
    form.esconder("fim", !varios);
    form.esconder("hora_inicio", !horario);
    form.esconder("hora_fim", !horario);
    form.botoes.push(("excluir", "Excluir evento".into()));
    e.modal = Some(super::modais::Modal::Detalhe {
        titulo: "Evento".into(),
        form,
        alvo: super::modais::AlvoDoDetalhe::Evento { embed, indice },
    });
    true
}

fn hora_valida(h: &str) -> bool {
    let Some((a, b)) = h.split_once(':') else { return false };
    a.len() == 2 && b.len() == 2 && a.parse::<u8>().is_ok_and(|x| x < 24) && b.parse::<u8>().is_ok_and(|x| x < 60)
}

/// Grava o que o formulário tem (a cada mudança, como a janela).
pub(super) fn aplicar_detalhe(e: &mut Estado, alvo: &super::modais::AlvoDoDetalhe, form: &mut crate::componentes::Formulario) {
    use crate::componentes::Valor;
    use super::modais::AlvoDoDetalhe;
    let lista = |form: &crate::componentes::Formulario, chave: &str| match form.valor(chave) {
        Some(Valor::Lista(l)) => l.clone(),
        _ => Vec::new(),
    };
    let opcional = |t: String| (!t.trim().is_empty()).then_some(t);
    match alvo {
        AlvoDoDetalhe::Cartao { embed, indice } => {
            let vencimento = form.texto("vencimento");
            if !vencimento.is_empty() && anotadinho_core::date_util::parse_date(&vencimento).is_none() {
                e.aviso = Some("vencimento: use AAAA-MM-DD".into());
                return;
            }
            let agora = e.agora.clone().unwrap_or_default();
            let indice = *indice;
            editar_kanban(e, embed, |d| {
                let mut c = d.items.get(indice).cloned().ok_or("o cartão sumiu do arquivo")?;
                let titulo = form.texto("titulo");
                if !titulo.is_empty() {
                    c.title = titulo;
                }
                c.description = opcional(form.texto("descricao"));
                c.tags = lista(form, "tags");
                c.due = opcional(vencimento);
                c.checklist = match form.valor("checklist") {
                    Some(Valor::Checklist(l)) => l.iter().map(|(done, text)| em::ChecklistItem { text: text.clone(), done: *done }).collect(),
                    _ => Vec::new(),
                };
                let velhos = c.comments.clone();
                c.comments = lista(form, "comentarios")
                    .into_iter()
                    .map(|text| {
                        let created = velhos.iter().find(|v| v.text == text).map(|v| v.created.clone()).unwrap_or_else(|| agora.clone());
                        em::Comment { text, created }
                    })
                    .collect();
                let velhos = c.attachments.clone();
                c.attachments = lista(form, "anexos")
                    .into_iter()
                    .map(|path| {
                        let name = velhos.iter().find(|v| v.path == path).map(|v| v.name.clone()).unwrap_or_else(|| {
                            std::path::Path::new(&path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_else(|| path.clone())
                        });
                        em::Attachment { name, path }
                    })
                    .collect();
                d.update_card(indice, c);
                Ok(())
            });
        }
        AlvoDoDetalhe::Propriedades | AlvoDoDetalhe::Botao { .. } | AlvoDoDetalhe::Consulta { .. } => {
            aplicar_configuracao(e, alvo, form);
        }
        AlvoDoDetalhe::Evento { embed, indice } => {
            form.esconder("fim", !form.booleano("varios"));
            let horario = form.booleano("horario");
            form.esconder("hora_inicio", !horario);
            form.esconder("hora_fim", !horario);
            let inicio = form.texto("inicio");
            let fim = form.texto("fim");
            let (hi, hf) = (form.texto("hora_inicio"), form.texto("hora_fim"));
            for (nome, data) in [("início", &inicio), ("fim", &fim)] {
                if !data.is_empty() && anotadinho_core::date_util::parse_date(data).is_none() {
                    e.aviso = Some(format!("{nome}: use AAAA-MM-DD"));
                    return;
                }
            }
            if horario {
                for (nome, h) in [("das", &hi), ("até", &hf)] {
                    if !h.is_empty() && !hora_valida(h) {
                        e.aviso = Some(format!("{nome}: use HH:MM"));
                        return;
                    }
                }
            }
            let varios = form.booleano("varios");
            let indice = *indice;
            let data_antes = tela::data_do_cursor(&e.arvore, &e.cursor);
            if editar_calendario(e, embed, |d| {
                let mut ev = d.entries.get(indice).cloned().ok_or("o evento sumiu do arquivo")?;
                let titulo = form.texto("titulo");
                if !titulo.is_empty() {
                    ev.title = titulo;
                }
                ev.date = opcional(inicio.clone());
                ev.end_date = if varios { opcional(fim.clone()).filter(|f| Some(f) != ev.date.as_ref()) } else { None };
                ev.start_time = if horario { opcional(hi.clone()) } else { None };
                ev.end_time = if horario { opcional(hf.clone()) } else { None };
                ev.tags = lista(form, "tags");
                d.update_entry(indice, ev);
                Ok(())
            }) {
                let destino = achar_evento(&e.arvore, embed, indice, data_antes.as_deref());
                ir(e, destino);
            }
        }
    }
}

/// O botão "Excluir" do formulário.
pub(super) fn excluir_do_detalhe(e: &mut Estado, alvo: &super::modais::AlvoDoDetalhe) {
    use super::modais::AlvoDoDetalhe;
    match alvo {
        AlvoDoDetalhe::Cartao { embed, indice } => {
            let i = *indice;
            if editar_kanban(e, embed, |d| {
                d.remove_card(i);
                Ok(())
            }) {
                e.aviso = Some("cartão apagado".into());
                e.seguir_cursor();
            }
        }
        AlvoDoDetalhe::Botao { embed, indice } => {
            let i = *indice;
            if editar_acoes(e, embed, |d| {
                d.remove_button(i);
                Ok(())
            }) {
                e.aviso = Some("botão apagado".into());
                if e.arvore.em(&e.cursor).is_none() {
                    e.cursor = embed.clone();
                }
                e.seguir_cursor();
            }
        }
        AlvoDoDetalhe::Propriedades | AlvoDoDetalhe::Consulta { .. } => {}
        AlvoDoDetalhe::Evento { embed, indice } => {
            let i = *indice;
            if editar_calendario(e, embed, |d| {
                d.remove_entry(i);
                Ok(())
            }) {
                e.aviso = Some("evento apagado".into());
                e.seguir_cursor();
            }
        }
    }
}

// ---------------------------------------------------------------------
// Propriedades da página, configurar botão e consulta (ciclo 344)
// ---------------------------------------------------------------------

/// Os tipos de página que a janela oferece no painel de propriedades.
const TIPOS_DE_PAGINA: &[(&str, &str)] = &[
    ("", "Página normal"),
    ("landing", "Landing"),
    ("kanban", "Kanban"),
    ("calendar", "Calendário"),
    ("table", "Tabela"),
    ("tags", "Tags"),
    ("assets", "Assets"),
    ("graph", "Grafo"),
    ("conversa", "Conversa"),
    ("prompt", "Prompt"),
];

fn yaml_como_texto(v: &serde_yaml::Value) -> String {
    match v {
        serde_yaml::Value::String(s) => s.clone(),
        serde_yaml::Value::Null => String::new(),
        outro => serde_yaml::to_string(outro).unwrap_or_default().trim().to_string(),
    }
}

/// O painel de propriedades (frontmatter) da página aberta — o
/// `PropertiesPanel` da janela: título, tipo, tags, criado, atualizado e
/// as propriedades livres como `chave: valor`.
pub(super) fn abrir_propriedades(e: &mut Estado) -> bool {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let Some(texto) = e.texto_da_pagina.clone() else {
        e.aviso = Some("esta página não pode ser editada daqui".into());
        return true;
    };
    let fm = anotadinho_core::MarkdownCodec::split_frontmatter(&texto).map(|(fm, _)| fm).unwrap_or_default();
    let mut tipos: Vec<(String, String)> = TIPOS_DE_PAGINA.iter().map(|(k, r)| (k.to_string(), r.to_string())).collect();
    let atual = fm.page_type.clone().unwrap_or_default();
    if !tipos.iter().any(|(k, _)| *k == atual) {
        tipos.push((atual.clone(), atual.clone()));
    }
    let idx = tipos.iter().position(|(k, _)| *k == atual).unwrap_or(0);
    let form = Formulario::novo(vec![
        C::novo("titulo", "Título", Valor::Texto(fm.title.clone().unwrap_or_default())).com_dica("(nome do arquivo)"),
        C::novo("tipo", "Tipo", Valor::Opcoes(tipos, idx)),
        C::novo("tags", "Tags", Valor::Lista(fm.tags.clone())).com_dica("tag"),
        C::novo("criado", "Criado", Valor::Texto(fm.created.clone().unwrap_or_default())).com_dica("AAAA-MM-DD"),
        C::novo("atualizado", "Atualizado", Valor::Texto(fm.updated.clone().unwrap_or_default())).com_dica("AAAA-MM-DD"),
        C::novo(
            "extra",
            "Propriedades",
            Valor::Lista(fm.extra.iter().map(|(k, v)| format!("{k}: {}", yaml_como_texto(v))).collect()),
        )
        .com_dica("chave: valor"),
    ]);
    let titulo = e.paginas.get(e.pagina).map(|p| format!("Propriedades · {}", p.title)).unwrap_or_else(|| "Propriedades".into());
    e.modal = Some(super::modais::Modal::Detalhe { titulo, form, alvo: super::modais::AlvoDoDetalhe::Propriedades });
    true
}

const ICONES: &[&str] = &["", "search", "home", "file-text", "folder", "calendar", "check", "edit", "link", "clock", "image", "table", "settings", "zap"];
const ACOES: &[(&str, &str)] = &[
    ("open-page", "Abrir página"),
    ("new-from-template", "Nova de template"),
    ("set-property", "Gravar propriedade"),
    ("run-search", "Buscar"),
];

fn esconder_campos_do_botao(form: &mut crate::componentes::Formulario) {
    let acao = form.escolha("acao");
    form.esconder("path", !matches!(acao.as_str(), "open-page" | "set-property"));
    form.esconder("template", acao != "new-from-template");
    form.esconder("folder", acao != "new-from-template");
    form.esconder("field", acao != "set-property");
    form.esconder("value", acao != "set-property");
    form.esconder("query", acao != "run-search");
}

/// `=`: configurar o item sob o cursor (ciclo 344) — o botão de ações
/// (`ActionButtonModal`) ou a consulta (`QuerySettingsModal`).
pub(super) fn configurar(e: &mut Estado) -> bool {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    use super::modais::{AlvoDoDetalhe, Modal};
    if let Some(embed) = embed_do_cursor(e, "actions") {
        let Some(indice) = (e.cursor.len() == embed.len() + 2).then(|| e.cursor[embed.len() + 1]) else {
            e.aviso = Some("entre num botão pra configurar".into());
            return true;
        };
        let Some(b) = ler_acoes(e, &embed).and_then(|d| d.buttons.get(indice).cloned()) else { return false };
        let icones: Vec<(String, String)> =
            ICONES.iter().map(|i| (i.to_string(), if i.is_empty() { "—".into() } else { super::glifo_do_icone(i).to_string() })).collect();
        let icone = icones.iter().position(|(k, _)| Some(k.as_str()) == b.icon.as_deref()).unwrap_or(0);
        let acoes: Vec<(String, String)> = ACOES.iter().map(|(k, r)| (k.to_string(), r.to_string())).collect();
        let acao = acoes.iter().position(|(k, _)| *k == b.action).unwrap_or(0);
        let mut form = Formulario::novo(vec![
            C::novo("label", "Rótulo", Valor::Texto(b.label.clone())),
            C::novo("icon", "Ícone", Valor::Opcoes(icones, icone)),
            C::novo("primary", "Destaque", Valor::Booleano(b.variant.as_deref() == Some("primary"))),
            C::novo("acao", "Ação", Valor::Opcoes(acoes, acao)),
            C::novo("path", "Página", Valor::Texto(b.path.clone().unwrap_or_default())).com_dica("pages/…md"),
            C::novo("template", "Template", Valor::Texto(b.template.clone().unwrap_or_default())).com_dica("templates/…md"),
            C::novo("folder", "Pasta", Valor::Texto(b.folder.clone().unwrap_or_default())).com_dica("pages/…"),
            C::novo("field", "Campo", Valor::Texto(b.field.clone().unwrap_or_default())).com_dica("status"),
            C::novo("value", "Valor", Valor::Texto(b.value.clone().unwrap_or_default())),
            C::novo("query", "Busca", Valor::Texto(b.query.clone().unwrap_or_default())),
        ]);
        esconder_campos_do_botao(&mut form);
        form.botoes.push(("excluir", "Excluir botão".into()));
        e.modal = Some(Modal::Detalhe { titulo: "Botão".into(), form, alvo: AlvoDoDetalhe::Botao { embed, indice } });
        return true;
    }
    if let Some(embed) = embed_do_cursor(e, "query") {
        let Some(q) = ler_consulta(e, &embed) else { return false };
        use anotadinho_core::query::QueryView;
        let visoes: Vec<(String, String)> = QueryView::all().iter().map(|v| (v.slug().to_string(), v.label().to_string())).collect();
        let visao = QueryView::all().iter().position(|v| *v == q.view).unwrap_or(0);
        let form = Formulario::novo(vec![
            C::novo("from", "Em", Valor::Texto(q.from.clone().unwrap_or_default())).com_dica("vault inteiro"),
            C::novo("tags", "Com tags", Valor::Lista(q.tags.clone())).com_dica("tag"),
            C::novo("where", "Condições", Valor::Lista(q.conditions.iter().map(|c| c.como_texto()).collect()))
                .com_dica("campo=valor · campo!=valor · campo~texto · campo? · campo>valor"),
            C::novo("sort", "Ordenar por", Valor::Texto(q.sort.as_ref().map(|s| s.field.clone()).unwrap_or_default())).com_dica("campo"),
            C::novo("desc", "Decrescente", Valor::Booleano(q.sort.as_ref().is_some_and(|s| s.desc))),
            C::novo("limit", "Limite", Valor::Texto(q.limit.map(|l| l.to_string()).unwrap_or_default())).com_dica("sem limite"),
            C::novo("view", "Visão", Valor::Opcoes(visoes, visao)),
            C::novo("columns", "Colunas", Valor::Lista(q.columns.clone())).com_dica("campo"),
            C::novo("group", "Agrupar por", Valor::Texto(q.group_by.clone().unwrap_or_default())).com_dica("campo"),
            C::novo("aggregate", "Agregados", Valor::Lista(q.aggregate.iter().map(|a| a.como_texto()).collect()))
                .com_dica("count · sum:campo · avg:campo"),
        ]);
        e.modal = Some(Modal::Detalhe { titulo: "Consulta".into(), form, alvo: AlvoDoDetalhe::Consulta { embed } });
        return true;
    }
    false
}

/// A parte do `aplicar_detalhe` dos formulários do ciclo 344.
pub(super) fn aplicar_configuracao(e: &mut Estado, alvo: &super::modais::AlvoDoDetalhe, form: &mut crate::componentes::Formulario) {
    use super::modais::AlvoDoDetalhe;
    let opcional = |t: String| (!t.trim().is_empty()).then_some(t);
    match alvo {
        AlvoDoDetalhe::Propriedades => {
            let Some(texto) = e.texto_da_pagina.clone() else { return };
            let velho = anotadinho_core::MarkdownCodec::split_frontmatter(&texto).map(|(fm, _)| fm).unwrap_or_default();
            let mut fm = velho.clone();
            fm.title = opcional(form.texto("titulo"));
            fm.page_type = opcional(form.escolha("tipo"));
            fm.tags = form.lista("tags");
            fm.created = opcional(form.texto("criado"));
            fm.updated = opcional(form.texto("atualizado"));
            let mut extra = std::collections::BTreeMap::new();
            for linha in form.lista("extra") {
                let Some((k, v)) = linha.split_once(':') else {
                    e.aviso = Some(format!("\"{linha}\": use chave: valor"));
                    return;
                };
                let (k, v) = (k.trim().to_string(), v.trim().to_string());
                // Valor que não mudou mantém o tipo YAML de antes (lista,
                // número); o que mudou vira texto.
                let valor = match velho.extra.get(&k) {
                    Some(antigo) if yaml_como_texto(antigo) == v => antigo.clone(),
                    _ => serde_yaml::Value::String(v),
                };
                extra.insert(k, valor);
            }
            fm.extra = extra;
            match anotadinho_core::MarkdownCodec::substituir_frontmatter(&texto, &fm) {
                Ok(novo) if novo != texto => e.aplicar_edicao(novo),
                Ok(_) => {}
                Err(err) => e.aviso = Some(format!("não gravou: {err}")),
            }
        }
        AlvoDoDetalhe::Botao { embed, indice } => {
            esconder_campos_do_botao(form);
            let i = *indice;
            editar_acoes(e, embed, |d| {
                let mut b = d.buttons.get(i).cloned().ok_or("o botão sumiu do arquivo")?;
                let rotulo = form.texto("label");
                if !rotulo.is_empty() {
                    b.label = rotulo;
                }
                b.icon = opcional(form.escolha("icon"));
                b.variant = form.booleano("primary").then(|| "primary".to_string());
                b.action = form.escolha("acao");
                let usa = |campo: &str| match b.action.as_str() {
                    "open-page" => campo == "path",
                    "set-property" => matches!(campo, "path" | "field" | "value"),
                    "new-from-template" => matches!(campo, "template" | "folder"),
                    "run-search" => campo == "query",
                    _ => false,
                };
                b.path = if usa("path") { opcional(form.texto("path")) } else { None };
                b.template = if usa("template") { opcional(form.texto("template")) } else { None };
                b.folder = if usa("folder") { opcional(form.texto("folder")) } else { None };
                b.field = if usa("field") { opcional(form.texto("field")) } else { None };
                b.value = if usa("value") { opcional(form.texto("value")) } else { None };
                b.query = if usa("query") { opcional(form.texto("query")) } else { None };
                d.update_button(i, b);
                Ok(())
            });
        }
        AlvoDoDetalhe::Consulta { embed } => {
            use anotadinho_core::query::{Aggregate, Condition, QueryView, Sort};
            let mut condicoes = Vec::new();
            for c in form.lista("where") {
                match Condition::parse(&c) {
                    Ok(x) => condicoes.push(x),
                    Err(err) => {
                        e.aviso = Some(err);
                        return;
                    }
                }
            }
            let mut agregados = Vec::new();
            for a in form.lista("aggregate") {
                match Aggregate::parse(&a) {
                    Ok(x) => agregados.push(x),
                    Err(err) => {
                        e.aviso = Some(err);
                        return;
                    }
                }
            }
            let limite = form.texto("limit");
            let limite = if limite.is_empty() {
                None
            } else {
                match limite.parse::<usize>() {
                    Ok(n) => Some(n),
                    Err(_) => {
                        e.aviso = Some("limite: um número".into());
                        return;
                    }
                }
            };
            let visao = QueryView::all().iter().copied().find(|v| v.slug() == form.escolha("view")).unwrap_or_default();
            let ordenar = opcional(form.texto("sort")).map(|field| Sort { field, desc: form.booleano("desc") });
            editar_consulta(e, embed, |q| {
                q.from = opcional(form.texto("from"));
                q.tags = form.lista("tags");
                q.conditions = condicoes;
                q.sort = ordenar;
                q.limit = limite;
                q.view = visao;
                q.columns = form.lista("columns");
                q.group_by = opcional(form.texto("group"));
                q.aggregate = agregados;
                Ok(())
            });
        }
        _ => {}
    }
}
