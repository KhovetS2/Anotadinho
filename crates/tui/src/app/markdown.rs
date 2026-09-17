//! Editar MARKDOWN pela TUI (ciclo 334): os blocos da página, do corpo de
//! um callout e dos painéis de colunas, com a mesma gramática do vim dos
//! embeds (ciclo 322).
//!
//! Um bloco é a unidade sob o cursor: parágrafo, título, item de lista,
//! citação — e, na página, também um embed inteiro (pra criar texto em
//! volta dele, apagá-lo, copiá-lo ou movê-lo). O que se edita é o TEXTO do
//! bloco, com as marcas de dentro da linha (`**`, `[[ ]]`) à vista; a
//! marca do bloco (`## `, `- [ ] `, `> `) fica de fora e volta sozinha.
//!
//! - `i`/`a`/`I`/`A` editam o texto (cursor no começo ou no fim); `cc`
//!   começa vazio. `Esc` confirma, como no vim; `Enter` confirma e abre o
//!   bloco seguinte (item depois de item).
//! - `o`/`O` criam um bloco depois/antes (num item, outro item).
//! - `dd` apaga o bloco; `yy`/`p`/`P` copiam e colam.
//! - `>>`/`<<` trocam o bloco de lugar com o vizinho.
//! - `Ctrl+A`/`Ctrl+X` sobem e descem o nível do título.
//! - `~` marca e desmarca a caixa do item (`- [ ]` ↔ `- [x]`).
//! - `x` não apaga caractere (não há cursor de caractere no modo normal):
//!   avisa em vez de destruir.

use std::ops::Range;

use anotadinho_core::embed::EmbedData;
use anotadinho_core::unidade::{Caminho, Tipo, Unidade};
use anotadinho_core::vim::Edicao;

use super::edicao::{editar_embed, perguntar, AcaoDaPergunta, Registro};
use super::Estado;

/// Onde mora o markdown que se edita.
#[derive(Debug, Clone, PartialEq)]
pub enum Hospedeiro {
    /// O corpo da página (sem o frontmatter).
    Pagina,
    /// O `body` de um callout.
    Callout(Caminho),
    /// O `body` do painel `n` de um embed de colunas.
    Painel(Caminho, usize),
}

/// Uma edição de bloco aberta no modo de inserção.
#[derive(Debug, Clone, PartialEq)]
pub struct EdicaoDeBloco {
    /// Onde mora o markdown.
    pub hospedeiro: Hospedeiro,
    /// O trecho do corpo que a resposta substitui — vazio é inserir.
    pub faixa: Range<usize>,
    /// A marca do bloco, que volta na frente do texto (`## `, `- `).
    pub prefixo: String,
    /// Item de lista: vizinho se separa por uma quebra, não linha em branco.
    pub de_lista: bool,
    /// O bloco que se edita — ou, num bloco novo, o vizinho dele.
    pub alvo: Caminho,
    /// Bloco novo (a `faixa` é vazia).
    pub novo: bool,
    /// Bloco novo ANTES do alvo.
    pub antes: bool,
}

/// O corpo de markdown do hospedeiro.
fn corpo(e: &Estado, h: &Hospedeiro) -> Option<String> {
    match h {
        Hospedeiro::Pagina => {
            let texto = e.texto_da_pagina.as_deref()?;
            Some(anotadinho_core::MarkdownCodec::split_frontmatter_text(texto).1.to_string())
        }
        Hospedeiro::Callout(embed) => match super::edicao::dados(e, embed)? {
            EmbedData::Callout(d) => Some(d.body),
            _ => None,
        },
        Hospedeiro::Painel(embed, i) => match super::edicao::dados(e, embed)? {
            EmbedData::Columns(d) => d.columns.get(*i).map(|p| p.body.clone()),
            _ => None,
        },
    }
}

/// Grava um corpo novo no hospedeiro.
fn gravar(e: &mut Estado, h: &Hospedeiro, novo: String) -> bool {
    match h {
        Hospedeiro::Pagina => {
            let Some(texto) = e.texto_da_pagina.clone() else {
                e.aviso = Some("esta página não pode ser editada daqui".into());
                return false;
            };
            let base = texto.len() - anotadinho_core::MarkdownCodec::split_frontmatter_text(&texto).1.len();
            e.aplicar_edicao(format!("{}{}", &texto[..base], novo));
            true
        }
        Hospedeiro::Callout(embed) => editar_embed(e, embed, |dados| match dados {
            EmbedData::Callout(d) => {
                d.body = novo;
                Ok(())
            }
            _ => Err("isto não é um callout".into()),
        }),
        Hospedeiro::Painel(embed, i) => editar_embed(e, embed, |dados| match dados {
            EmbedData::Columns(d) => {
                let p = d.columns.get_mut(*i).ok_or("o painel sumiu do arquivo")?;
                p.body = novo;
                Ok(())
            }
            _ => Err("isto não é um embed de colunas".into()),
        }),
    }
}

/// O caminho de onde os blocos do hospedeiro começam.
fn raiz(h: &Hospedeiro) -> Caminho {
    match h {
        Hospedeiro::Pagina => Vec::new(),
        Hospedeiro::Callout(embed) => embed.clone(),
        Hospedeiro::Painel(embed, i) => [embed.as_slice(), &[*i]].concat(),
    }
}

/// O hospedeiro do markdown sob o cursor.
///
/// Fora de embed é a página. No PRÓPRIO embed (o cursor nele, não dentro)
/// também é a página: o embed é um bloco dela. Dentro do corpo de um
/// callout ou de um painel, é ele.
pub(super) fn hospedeiro_do_cursor(e: &Estado) -> Option<Hospedeiro> {
    let embed = (1..=e.cursor.len())
        .rev()
        .map(|n| &e.cursor[..n])
        .find(|c| matches!(e.arvore.em(c).map(|u| &u.tipo), Some(Tipo::Embed(_))));
    let Some(embed) = embed else { return Some(Hospedeiro::Pagina) };
    if embed.len() == e.cursor.len() {
        return (embed.len() == 1).then_some(Hospedeiro::Pagina);
    }
    let u = e.arvore.em(&e.cursor)?;
    if matches!(u.tipo, Tipo::Parte { .. }) {
        return None;
    }
    match e.arvore.em(embed).map(|u| &u.tipo) {
        Some(Tipo::Embed(n)) if n == "callout" => Some(Hospedeiro::Callout(embed.to_vec())),
        Some(Tipo::Embed(n)) if n == "columns" && e.cursor.len() >= embed.len() + 2 => {
            Some(Hospedeiro::Painel(embed.to_vec(), e.cursor[embed.len()]))
        }
        _ => None,
    }
}

/// A marca de bloco e o texto editável de uma unidade, pela fonte dela.
/// `None` é bloco que não se edita pelo rodapé (código, régua, lista
/// inteira, embed).
fn decompor(tipo: &Tipo, fonte: &str) -> Option<(String, String)> {
    let juntar = |linhas: &mut dyn Iterator<Item = &str>| -> String {
        linhas.map(str::trim).filter(|l| !l.is_empty()).collect::<Vec<_>>().join(" ")
    };
    match tipo {
        Tipo::Titulo(_) => {
            let recuo = fonte.len() - fonte.trim_start().len();
            let resto = &fonte[recuo..];
            let cerquilhas = resto.chars().take_while(|c| *c == '#').count();
            let depois = resto[cerquilhas..].len() - resto[cerquilhas..].trim_start().len();
            let corte = recuo + cerquilhas + depois;
            Some((fonte[..corte].to_string(), fonte[corte..].trim_end().to_string()))
        }
        Tipo::Item => {
            let primeira = fonte.lines().next().unwrap_or("");
            let prefixo = marca_do_item(primeira);
            let mut resto = std::iter::once(primeira.get(prefixo.len()..).unwrap_or("")).chain(fonte.lines().skip(1));
            Some((prefixo, juntar(&mut resto)))
        }
        Tipo::Citacao => {
            let mut linhas = fonte.lines().map(|l| {
                let sem = l.trim_start().strip_prefix('>').unwrap_or(l);
                sem.strip_prefix(' ').unwrap_or(sem)
            });
            Some(("> ".to_string(), juntar(&mut linhas)))
        }
        Tipo::Paragrafo => Some((String::new(), juntar(&mut fonte.lines()))),
        _ => None,
    }
}

/// A marca inteira de um item na linha: recuo, marcador e caixa.
fn marca_do_item(linha: &str) -> String {
    let recuo: String = linha.chars().take_while(|c| c.is_whitespace()).collect();
    let resto = &linha[recuo.len()..];
    let marcador = ["- ", "* ", "+ "].into_iter().find(|m| resto.starts_with(m)).map(str::to_string).or_else(|| {
        let digitos = resto.chars().take_while(char::is_ascii_digit).count();
        [". ", ") "]
            .into_iter()
            .find(|m| digitos > 0 && resto[digitos..].starts_with(m))
            .map(|m| format!("{}{m}", &resto[..digitos]))
    });
    let Some(marcador) = marcador else {
        // O item vazio (`-` sozinho).
        return format!("{recuo}{} ", resto.trim());
    };
    let depois = &resto[marcador.len()..];
    let caixa = ["[ ] ", "[x] ", "[X] "].into_iter().find(|c| depois.starts_with(c)).unwrap_or("");
    format!("{recuo}{marcador}{caixa}")
}

/// A marca de um item NOVO ao lado deste: mesmo marcador, caixa vazia,
/// o número seguinte numa lista numerada.
fn marca_vizinha(marca: &str) -> String {
    let recuo: String = marca.chars().take_while(|c| c.is_whitespace()).collect();
    let resto = &marca[recuo.len()..];
    let digitos = resto.chars().take_while(char::is_ascii_digit).count();
    let mut nova = if digitos > 0 {
        let n: usize = resto[..digitos].parse().unwrap_or(1);
        format!("{}{}", n + 1, &resto[digitos..digitos + 2])
    } else {
        resto.chars().take(2).collect()
    };
    if resto.contains("[ ] ") || resto.contains("[x] ") || resto.contains("[X] ") {
        nova.push_str("[ ] ");
    }
    format!("{recuo}{nova}")
}

/// Insere `texto` como bloco novo em `pos` do corpo e devolve o corpo e
/// onde o bloco começou.
///
/// `pos` é o FIM de um bloco (`antes` falso: o novo vem depois dele) ou o
/// COMEÇO de um (`antes` verdadeiro). A separação do lado de lá fica como
/// estava — um item novo no fim da lista não gruda na citação que vinha
/// depois dela —, e a do lado de cá é uma quebra (item) ou linha em
/// branco.
pub(super) fn inserir_bloco(corpo: &str, pos: usize, texto: &str, de_lista: bool, antes: bool) -> (String, usize) {
    let sep = if de_lista { "\n" } else { "\n\n" };
    if corpo.trim().is_empty() {
        return (format!("{texto}\n"), 0);
    }
    let (a, d) = corpo.split_at(pos.min(corpo.len()));
    if antes {
        (format!("{a}{texto}{sep}{d}"), a.len())
    } else if d.trim().is_empty() {
        let a = a.trim_end_matches('\n');
        (format!("{a}{sep}{texto}\n"), a.len() + sep.len())
    } else {
        (format!("{a}{sep}{texto}{d}"), a.len() + sep.len())
    }
}

/// Tira o bloco `faixa` do corpo, colando os vizinhos.
fn remover_bloco(corpo: &str, faixa: Range<usize>, de_lista: bool) -> String {
    let antes = corpo[..faixa.start].trim_end_matches('\n');
    let depois = corpo[faixa.end..].trim_start_matches('\n');
    let mut s = String::from(antes);
    if !antes.is_empty() && !depois.is_empty() {
        s.push_str(if de_lista { "\n" } else { "\n\n" });
    }
    s.push_str(depois);
    if corpo.ends_with('\n') && !s.ends_with('\n') && !s.is_empty() {
        s.push('\n');
    }
    s
}

/// Os blocos editáveis do hospedeiro, com o caminho: os de primeiro nível
/// e os itens das listas. Na página, o conteúdo dos embeds fica de fora
/// — as faixas de lá são de OUTRO corpo.
fn blocos<'a>(e: &'a Estado, h: &Hospedeiro) -> Vec<(Caminho, &'a Unidade)> {
    let base = raiz(h);
    let Some(r) = e.arvore.em(&base) else { return Vec::new() };
    let mut v = Vec::new();
    for (i, u) in r.filhos.iter().enumerate() {
        if matches!(u.tipo, Tipo::Parte { .. }) || u.intervalo.is_none() {
            continue;
        }
        let c = [base.as_slice(), &[i]].concat();
        if matches!(u.tipo, Tipo::Lista | Tipo::ListaOrdenada) {
            for (k, item) in u.filhos.iter().enumerate() {
                if item.intervalo.is_some() {
                    v.push(([c.as_slice(), &[k]].concat(), item));
                }
            }
        }
        v.push((c, u));
    }
    v
}

/// O bloco que começa em `inicio` — o mais fundo: uma lista e o primeiro
/// item dela começam no mesmo byte, e quem foi editado é o item.
fn bloco_em(e: &Estado, h: &Hospedeiro, inicio: usize) -> Option<Caminho> {
    blocos(e, h)
        .into_iter()
        .filter(|(_, u)| u.intervalo.as_ref().is_some_and(|r| r.start == inicio))
        .max_by_key(|(c, _)| c.len())
        .map(|(c, _)| c)
}

fn ir(e: &mut Estado, destino: Option<Caminho>) {
    if let Some(c) = destino {
        e.cursor = c;
    }
    e.seguir_cursor();
}

/// As edições de markdown (ciclo 334). `false` é "o cursor não está num
/// bloco de markdown".
pub(super) fn no_markdown(e: &mut Estado, ed: Edicao, h: Hospedeiro) -> bool {
    let Some(corpo_atual) = corpo(e, &h) else { return false };
    let alvo = e.cursor.clone();
    let Some(u) = e.arvore.em(&alvo) else { return false };
    // O cursor tem que estar num bloco DESTE hospedeiro.
    if !blocos(e, &h).iter().any(|(c, _)| *c == alvo) {
        // Página vazia, ou o cursor fora de bloco: `o` cria no fim.
        if matches!(ed, Edicao::Criar { .. }) && h == Hospedeiro::Pagina {
            let fim = corpo_atual.len();
            perguntar(
                e,
                "-- INSERÇÃO --",
                String::new(),
                AcaoDaPergunta::Bloco(EdicaoDeBloco {
                    hospedeiro: h,
                    faixa: fim..fim,
                    prefixo: String::new(),
                    de_lista: false,
                    alvo,
                    novo: true,
                    antes: false,
                }),
            );
            return true;
        }
        return false;
    }
    let Some(faixa) = u.intervalo.clone() else { return false };
    // A faixa de um embed inclui a quebra do fim da cerca; o bloco é só o
    // conteúdo.
    let aparar = |r: Range<usize>| -> Range<usize> {
        let t = corpo_atual.get(r.clone()).unwrap_or("");
        r.start..r.start + t.trim_end().len()
    };
    let faixa = aparar(faixa);
    let tipo = u.tipo.clone();
    let fonte = corpo_atual.get(faixa.clone()).unwrap_or("").to_string();
    let de_lista = matches!(tipo, Tipo::Item);
    let decomposto = decompor(&tipo, &fonte);
    let e_embed = matches!(tipo, Tipo::Embed(_));

    match ed {
        Edicao::Reescrever { limpar, no_fim } => {
            let Some((prefixo, texto)) = decomposto else {
                e.aviso = Some(match tipo {
                    Tipo::Codigo(_) => "bloco de código: edite no arquivo",
                    Tipo::Lista | Tipo::ListaOrdenada => "entre num item pra editar a lista",
                    _ => "este bloco não tem texto pra editar",
                }
                .into());
                return true;
            };
            let texto = if limpar { String::new() } else { texto };
            perguntar(
                e,
                "-- INSERÇÃO --",
                texto,
                AcaoDaPergunta::Bloco(EdicaoDeBloco {
                    hospedeiro: h,
                    faixa,
                    prefixo,
                    de_lista,
                    alvo,
                    novo: false,
                    antes: false,
                }),
            );
            if !no_fim {
                if let Some(p) = e.pergunta.as_mut() {
                    p.cursor = 0;
                }
            }
        }
        Edicao::Criar { antes } => {
            let prefixo = match &decomposto {
                Some((p, _)) if de_lista => marca_vizinha(p),
                _ => String::new(),
            };
            let pos = if antes { faixa.start } else { faixa.end };
            perguntar(
                e,
                "-- INSERÇÃO --",
                String::new(),
                AcaoDaPergunta::Bloco(EdicaoDeBloco {
                    hospedeiro: h,
                    faixa: pos..pos,
                    prefixo,
                    de_lista,
                    alvo,
                    novo: true,
                    antes,
                }),
            );
        }
        Edicao::Apagar => {
            let novo = remover_bloco(&corpo_atual, faixa, de_lista);
            if gravar(e, &h, novo) {
                e.registro = Some(Registro::Bloco(fonte));
                e.aviso = Some(if e_embed { "embed apagado" } else { "bloco apagado" }.into());
                e.seguir_cursor();
            }
        }
        Edicao::ApagarConteudo => {
            e.aviso = Some("x não apaga bloco: dd apaga, i edita o texto".into());
        }
        Edicao::Copiar => {
            e.registro = Some(Registro::Bloco(fonte));
            e.aviso = Some(if e_embed { "embed copiado" } else { "bloco copiado" }.into());
        }
        Edicao::Colar { antes } => {
            let Some(Registro::Bloco(texto)) = e.registro.clone() else {
                e.aviso = Some("não há bloco copiado".into());
                return true;
            };
            // Item colado ao lado de item continua na lista.
            let e_item = decompor(&Tipo::Item, &texto).is_some_and(|(p, _)| !p.trim().is_empty() && p.trim() != texto.trim());
            let junto = de_lista && e_item;
            let pos = if antes { faixa.start } else { faixa.end };
            let (novo, onde) = inserir_bloco(&corpo_atual, pos, &texto, junto, antes);
            if gravar(e, &h, novo) {
                let destino = bloco_em(e, &h, onde);
                ir(e, destino);
            }
        }
        Edicao::Deslocar(n) | Edicao::Reordenar(n) => {
            // Os vizinhos do MESMO pai: itens da mesma lista, ou blocos do
            // mesmo nível.
            let (pai, idx) = match alvo.split_last() {
                Some((i, p)) => (p.to_vec(), *i),
                None => return false,
            };
            let irmaos: Vec<(usize, Range<usize>)> = e
                .arvore
                .em(&pai)
                .map(|p| {
                    p.filhos
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| !matches!(f.tipo, Tipo::Parte { .. }))
                        .filter_map(|(i, f)| f.intervalo.clone().map(|r| (i, aparar(r))))
                        .collect()
                })
                .unwrap_or_default();
            let Some(pos) = irmaos.iter().position(|(i, _)| *i == idx) else { return false };
            let destino = (pos as i64 + n).clamp(0, irmaos.len() as i64 - 1) as usize;
            if destino == pos {
                e.aviso = Some("não há bloco desse lado".into());
                return true;
            }
            // Troca com o vizinho, um passo por vez, na própria string: o
            // que fica ENTRE os dois (a separação) não muda.
            let mut texto = corpo_atual.clone();
            let mut faixas: Vec<Range<usize>> = irmaos.iter().map(|(_, r)| r.clone()).collect();
            let mut p = pos;
            while p != destino {
                let q = if destino > p { p + 1 } else { p - 1 };
                let (ia, ib) = if q > p { (p, q) } else { (q, p) };
                let (a, b) = (faixas[ia].clone(), faixas[ib].clone());
                let (ta, tb, meio) = (texto[a.clone()].to_string(), texto[b.clone()].to_string(), texto[a.end..b.start].to_string());
                texto = format!("{}{tb}{meio}{ta}{}", &texto[..a.start], &texto[b.end..]);
                let nb = a.start..a.start + tb.len();
                let na = nb.end + meio.len()..nb.end + meio.len() + ta.len();
                faixas[ia] = nb;
                faixas[ib] = na;
                p = q;
            }
            let onde = faixas[destino].start;
            if gravar(e, &h, texto) {
                let destino = bloco_em(e, &h, onde);
                ir(e, destino);
            }
        }
        Edicao::Somar(n) => {
            let Tipo::Titulo(nivel) = tipo else {
                e.aviso = Some("Ctrl+A/Ctrl+X mudam o nível de um título".into());
                return true;
            };
            let Some((_, texto)) = decomposto else { return true };
            let novo_nivel = (nivel as i64 - n).clamp(1, 6) as usize;
            let novo_bloco = format!("{} {texto}", "#".repeat(novo_nivel));
            let mut novo = corpo_atual.clone();
            novo.replace_range(faixa.clone(), &novo_bloco);
            if gravar(e, &h, novo) {
                e.aviso = Some(format!("título nível {novo_nivel}"));
                ir(e, None);
            }
        }
        Edicao::Alternar => {
            let Some((prefixo, texto)) = decomposto.filter(|_| de_lista) else {
                e.aviso = Some("~ marca a caixa de um item de lista".into());
                return true;
            };
            let novo_prefixo = if prefixo.contains("[ ] ") {
                prefixo.replace("[ ] ", "[x] ")
            } else if prefixo.contains("[x] ") || prefixo.contains("[X] ") {
                prefixo.replace("[x] ", "[ ] ").replace("[X] ", "[ ] ")
            } else {
                format!("{prefixo}[ ] ")
            };
            let mut novo = corpo_atual.clone();
            novo.replace_range(faixa.clone(), &format!("{novo_prefixo}{texto}"));
            if gravar(e, &h, novo) {
                ir(e, None);
            }
        }
        _ => return false,
    }
    true
}

/// Confirma uma edição de bloco. Devolve o caminho do bloco gravado.
pub(super) fn responder(e: &mut Estado, b: &EdicaoDeBloco, texto: &str) -> Option<Caminho> {
    let corpo_atual = corpo(e, &b.hospedeiro)?;
    if b.faixa.end > corpo_atual.len() {
        e.aviso = Some("o texto mudou por fora".into());
        return None;
    }
    let bloco = format!("{}{}", b.prefixo, texto);
    let (novo, onde) = if b.novo {
        inserir_bloco(&corpo_atual, b.faixa.start, &bloco, b.de_lista, b.antes)
    } else {
        let mut s = corpo_atual.clone();
        s.replace_range(b.faixa.clone(), &bloco);
        (s, b.faixa.start)
    };
    if !gravar(e, &b.hospedeiro, novo) {
        return None;
    }
    let destino = bloco_em(e, &b.hospedeiro, onde);
    ir(e, destino.clone());
    destino
}

/// Depois do `Enter` num bloco: abre o seguinte, do mesmo jeito que um
/// editor faz — item depois de item, parágrafo depois do resto.
pub(super) fn continuar(e: &mut Estado, b: &EdicaoDeBloco, gravado: Caminho) {
    let Some(u) = e.arvore.em(&gravado) else { return };
    let Some(faixa) = u.intervalo.clone() else { return };
    let prefixo = if b.de_lista { marca_vizinha(&b.prefixo) } else { String::new() };
    perguntar(
        e,
        "-- INSERÇÃO --",
        String::new(),
        AcaoDaPergunta::Bloco(EdicaoDeBloco {
            hospedeiro: b.hospedeiro.clone(),
            faixa: faixa.end..faixa.end,
            prefixo,
            de_lista: b.de_lista,
            alvo: gravado,
            novo: true,
            antes: false,
        }),
    );
}

