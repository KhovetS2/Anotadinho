//! A tela de conversa com o agente (ciclo 340), como a `ConversaView` da
//! janela: uma página com `type: conversa` não abre como árvore de blocos,
//! abre como conversa.
//!
//! O que a janela tem e a TUI também:
//!
//! - o topo com o título, o agente configurado e quantas páginas estão
//!   anexadas, e as anexadas em pílulas embaixo;
//! - as mensagens — as suas à direita, as do agente à esquerda —, com o
//!   markdown desenhado como o resto da página;
//! - "pensando há 12s" com o que o agente já escreveu, enquanto ele roda;
//! - o campo de mensagem embaixo, com "Enviar" (ou "Parar");
//! - numa resposta do agente, virar spec, proposta ou execução.
//!
//! Teclas: `j`/`k` passam pelas mensagens, `i` (ou `Enter` no campo)
//! escreve, `Enter` envia, `Ctrl+J` quebra linha, `Esc` sai do campo,
//! `Enter` numa resposta abre as ações, `y` copia a mensagem, `Ctrl+X`
//! interrompe o agente. Anexar, tirar anexo e trocar de agente ficam na
//! barra de comandos (`:`).

use anotadinho_core::conversa::{Autor, Mensagem};
use anotadinho_core::fluxo::Artefato;
use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use super::modais::{AcaoDaEntrada, AcaoDaEscolha, Modal, Pedido};
use super::{Estado, Foco};
use crate::componentes::{Campo, Item, Lista};
use crate::tema::Tema;

/// Quantas mensagens do histórico vão no prompt, como na janela.
pub const HISTORICO_NO_PROMPT: usize = 12;

/// O estado da tela de conversa.
#[derive(Debug, Clone, PartialEq)]
pub struct TelaDeConversa {
    /// O arquivo da conversa.
    pub path: String,
    /// O título.
    pub titulo: String,
    /// As mensagens, lidas do arquivo.
    pub mensagens: Vec<Mensagem>,
    /// As páginas anexadas (o `contexto:` do frontmatter).
    pub anexos: Vec<String>,
    /// A mensagem selecionada; `mensagens.len()` é o campo.
    pub selecionada: usize,
    /// O rascunho.
    pub rascunho: Campo,
    /// O teclado está no campo.
    pub escrevendo: bool,
    /// O agente rodando: há quantos segundos, e o que já saiu.
    pub trabalho: Option<(u64, String)>,
    /// A posição na fila (ciclo 408), quando o envio espera vaga.
    pub na_fila: Option<usize>,
    /// O peso do contexto (ciclo 417): tokens estimados e se estourou o
    /// teto. `None` enquanto ninguém mediu.
    pub peso: Option<(String, bool)>,
    /// O erro da última execução.
    pub erro: Option<String>,
    /// O prompt padrão em uso (ciclo 341).
    pub prompt: Option<PromptAtivo>,
}

/// Um prompt padrão aplicado ao campo (ciclo 341), como na janela: o
/// molde, os valores das variáveis e o rascunho de antes — escolher
/// "Nenhum" devolve ele.
#[derive(Debug, Clone, PartialEq)]
pub struct PromptAtivo {
    /// A página do prompt.
    pub path: String,
    /// O título dela, que o botão mostra.
    pub titulo: String,
    /// O molde.
    pub molde: anotadinho_core::prompt_padrao::PromptPadrao,
    /// Os valores já preenchidos.
    pub valores: std::collections::BTreeMap<String, String>,
    /// O rascunho de antes do prompt.
    pub rascunho_antes: String,
}

impl PromptAtivo {
    /// As variáveis ainda sem valor.
    pub fn pendentes(&self) -> Vec<String> {
        self.molde
            .variaveis
            .iter()
            .filter(|v| self.valores.get(*v).is_none_or(|x| x.trim().is_empty()))
            .cloned()
            .collect()
    }
}

impl TelaDeConversa {
    /// Monta a tela a partir do arquivo, se ele é uma conversa.
    pub fn do_arquivo(path: &str, titulo: &str, texto: &str) -> Option<Self> {
        let (frontmatter, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(texto);
        let e_conversa = frontmatter.lines().any(|l| {
            l.strip_prefix("type:").map(|v| v.trim().trim_matches('"').trim_matches('\'')) == Some("conversa")
        });
        if !e_conversa {
            return None;
        }
        let titulo = frontmatter
            .lines()
            .find_map(|l| l.strip_prefix("title:"))
            .map(|v| v.trim().trim_matches('"').to_string())
            .filter(|v| !v.is_empty())
            .unwrap_or_else(|| titulo.to_string());
        let mensagens = anotadinho_core::conversa::parse(corpo);
        Some(Self {
            path: path.to_string(),
            titulo,
            selecionada: mensagens.len(),
            anexos: anotadinho_core::conversa::contexto_do_frontmatter(frontmatter),
            mensagens,
            rascunho: Campo::default(),
            escrevendo: false,
            trabalho: None,
            na_fila: None,
            peso: None,
            erro: None,
            prompt: None,
        })
    }

    /// Leva o que é da tela (não do arquivo) de uma montagem pra outra.
    pub fn herdar(&mut self, velha: &TelaDeConversa) {
        if velha.path == self.path {
            self.rascunho = velha.rascunho.clone();
            self.escrevendo = velha.escrevendo;
            self.trabalho = velha.trabalho.clone();
            self.na_fila = velha.na_fila;
            self.peso = velha.peso.clone();
            self.erro = velha.erro.clone();
            self.prompt = velha.prompt.clone();
            if velha.selecionada < velha.mensagens.len() {
                self.selecionada = velha.selecionada.min(self.mensagens.len());
            }
        }
    }
}

/// O último pedaço de uma pasta (`/home/x/projeto` → `projeto`).
fn nome_de_pasta(p: &str) -> String {
    std::path::Path::new(p.trim_end_matches('/')).file_name().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| p.to_string())
}

fn nome_curto(path: &str) -> String {
    std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string())
}

/// Uma tecla na tela de conversa. Devolve se foi usada.
pub fn tecla(e: &mut Estado, tecla: &str) -> bool {
    let rodando = e.conversa.as_ref().is_some_and(|c| c.trabalho.is_some() || c.na_fila.is_some());
    let Some(c) = e.conversa.as_mut() else { return false };
    let total = c.mensagens.len();
    if c.escrevendo {
        match tecla {
            "Escape" => c.escrevendo = false,
            "Ctrl+j" => {
                c.rascunho.tecla("\n");
            }
            "Enter" => enviar(e),
            "Ctrl+p" => abrir_seletor_de_prompt(e),
            outra => {
                let antes = c.rascunho.texto.clone();
                c.rascunho.tecla(outra);
                // Mexer no texto montado vira mensagem livre, como na
                // janela.
                if c.rascunho.texto != antes {
                    c.prompt = None;
                }
            }
        }
        return true;
    }
    match tecla {
        "j" | "ArrowDown" => c.selecionada = (c.selecionada + 1).min(total),
        "k" | "ArrowUp" => c.selecionada = c.selecionada.saturating_sub(1),
        "G" | "End" => c.selecionada = total,
        "g" | "Home" => c.selecionada = 0,
        "i" | "a" | "A" | "o" => {
            c.selecionada = total;
            c.escrevendo = !rodando;
            if rodando {
                e.aviso = Some("o agente está trabalhando — Ctrl+X interrompe".into());
            }
        }
        "Enter" if c.selecionada >= total => {
            c.escrevendo = !rodando;
        }
        "Enter" => {
            let m = &c.mensagens[c.selecionada];
            if m.autor == Autor::Agente {
                let i = c.selecionada;
                e.modal = Some(Modal::Escolha {
                    titulo: "Resposta do agente".into(),
                    lista: Lista::menu(vec![
                        Item::novo("▤", "virar spec", "spec").com_detalhe("cria e abre a spec"),
                        Item::novo("▤", "virar proposta", "proposta").com_detalhe("cria e abre a proposta"),
                        Item::novo("ϟ", "virar execução", "execucao").com_detalhe("cria e pede a implementação"),
                        Item::novo("⎘", "copiar o texto", "copiar"),
                    ]),
                    acao: AcaoDaEscolha::RespostaDoAgente(i),
                });
            }
        }
        "y" if c.selecionada < total => {
            let texto = c.mensagens[c.selecionada].texto.clone();
            e.registro = Some(super::Registro::Bloco(texto));
            e.aviso = Some("mensagem copiada".into());
        }
        "Ctrl+x" if rodando => {
            let path = c.path.clone();
            e.pedidos.push(Pedido::InterromperAgente(path));
        }
        "p" => abrir_seletor_de_prompt(e),
        "Escape" => c.erro = None,
        _ => return false,
    }
    true
}

fn enviar(e: &mut Estado) {
    let Some(c) = e.conversa.as_mut() else { return };
    let pergunta = c.rascunho.texto.trim().to_string();
    if pergunta.is_empty() {
        return;
    }
    if c.trabalho.is_some() {
        e.aviso = Some("já tem uma execução em andamento nesta conversa".into());
        return;
    }
    if c.na_fila.is_some() {
        e.aviso = Some("esta conversa já tem um envio esperando vaga".into());
        return;
    }
    if c.prompt.as_ref().is_some_and(|p| !p.pendentes().is_empty()) {
        e.aviso = Some("Preencha todos os marcadores antes de enviar.".into());
        abrir_seletor_de_prompt(e);
        return;
    }
    c.prompt = None;
    c.rascunho = Campo::default();
    c.escrevendo = false;
    c.erro = None;
    c.selecionada = c.mensagens.len() + 1;
    let (path, anexos) = (c.path.clone(), c.anexos.clone());
    e.pedidos.push(Pedido::EnviarNaConversa { path, pergunta, anexos });
}

/// O que fazer com uma resposta do agente (o menu do `Enter`).
pub fn acao_na_resposta(e: &mut Estado, indice: usize, chave: &str) {
    let Some(c) = e.conversa.as_ref() else { return };
    let Some(m) = c.mensagens.get(indice) else { return };
    let texto = m.texto.clone();
    let conversa = c.path.clone();
    let artefato = match chave {
        "spec" => Artefato::Spec,
        "proposta" => Artefato::Proposta,
        "execucao" => Artefato::Execucao,
        _ => {
            e.registro = Some(super::Registro::Bloco(texto));
            e.aviso = Some("resposta copiada".into());
            return;
        }
    };
    if artefato == Artefato::Execucao {
        e.modal = Some(Modal::Confirmar {
            titulo: "Pedir a implementação?".into(),
            mensagem: "Cria a página de execução, anexa nesta conversa e pede pro agente implementar agora.".into(),
            acao: Pedido::ExecutarDaConversa { conversa, texto },
        });
        return;
    }
    let hoje = e.agora.clone().unwrap_or_default().split(' ').next().unwrap_or("").to_string();
    let titulo = anotadinho_core::fluxo::titulo_sugerido(&texto, 60);
    let conteudo = anotadinho_core::fluxo::montar_pagina(artefato, &titulo, &texto, Some(&conversa), &hoje);
    let path = format!("{}/{}.md", artefato.pasta(), anotadinho_core::fluxo::slug_de_titulo(&titulo));
    e.pedidos.push(Pedido::CriarPagina { path, conteudo });
}

/// Põe `texto` no campo da conversa aberta, pronto pra mandar (ciclo
/// 348) — a "pergunta inicial" da janela.
pub fn escrever_no_campo(e: &mut Estado, texto: &str) {
    if let Some(c) = e.conversa.as_mut() {
        c.rascunho = Campo::com(texto);
        c.selecionada = c.mensagens.len();
        c.prompt = None;
        e.foco = Foco::Conteudo;
    }
}

/// Os comandos da barra que só existem numa conversa.
pub fn comandos(e: &Estado) -> Vec<Item> {
    let Some(c) = &e.conversa else { return Vec::new() };
    let mut v = vec![Item::novo("▤", "Usar prompt padrão…", "prompt"), Item::novo("⌁", "Anexar página…", "anexar")];
    v.push(
        Item::novo("▦", "Prévia do prompt", "previa")
            .com_detalhe("o que o agente vai receber, com o peso de cada parte"),
    );
    if !c.anexos.is_empty() {
        v.push(Item::novo("⌁", "Tirar anexo…", "desanexar"));
        v.push(
            Item::novo("▤", "Guardar anexos como página de contexto…", "guardar-contexto")
                .com_detalhe("vira um recorte reutilizável, por transclusão"),
        );
    }
    if c.trabalho.is_some() {
        v.push(Item::novo("■", "Interromper o agente", "interromper"));
    }
    // As pastas do agente (ciclo 369), os botões de pasta do topo da janela.
    let agente = e.preferencias.agente.clone().unwrap_or_default();
    v.push(Item::novo("▭", "Pasta de trabalho do agente…", "pasta-trabalho").com_detalhe(if agente.cwd.is_empty() { "raiz do projeto".to_string() } else { agente.cwd.clone() }));
    if !agente.arg_pasta_extra.trim().is_empty() {
        v.push(Item::novo("▭", "Dar alcance a outra pasta…", "pasta-extra"));
    }
    if !agente.pastas_extras.is_empty() {
        v.push(Item::novo("▭", "Tirar pasta do alcance…", "tirar-pasta"));
    }
    v
}

/// Executa um comando de conversa da barra.
pub fn executar(e: &mut Estado, chave: &str) -> bool {
    let Some(c) = &e.conversa else { return false };
    match chave {
        "anexar" => {
            let itens = e
                .paginas
                .iter()
                .filter(|p| p.path != c.path && !c.anexos.contains(&p.path))
                .map(|p| Item::novo("≡", p.title.clone(), p.path.clone()).com_detalhe(p.path.clone()))
                .collect();
            e.modal = Some(Modal::Escolha {
                titulo: "Anexar página".into(),
                lista: Lista::filtravel(itens),
                acao: AcaoDaEscolha::Anexar,
            });
        }
        "desanexar" => {
            let itens = c.anexos.iter().map(|a| Item::novo("⌁", nome_curto(a), a.clone()).com_detalhe(a.clone())).collect();
            e.modal = Some(Modal::Escolha { titulo: "Tirar anexo".into(), lista: Lista::menu(itens), acao: AcaoDaEscolha::Desanexar });
        }
        // Os anexos viram um recorte do vault (ciclo 416): uma página de
        // transclusões, que passa a ser O anexo. Reutilizável em outra
        // conversa, versionada, e editável como qualquer página.
        "previa" => {
            let pergunta = c.rascunho.texto.trim().to_string();
            e.pedidos.push(Pedido::PreviaDoPrompt { conversa: c.path.clone(), pergunta });
        }
        "guardar-contexto" => {
            e.modal = Some(Modal::Entrada {
                titulo: format!("Nome da página de contexto ({} anexo(s))", c.anexos.len()),
                campo: crate::componentes::Campo::default(),
                acao: super::modais::AcaoDaEntrada::PaginaDeContexto,
            });
        }
        "interromper" => e.pedidos.push(Pedido::InterromperAgente(c.path.clone())),
        "prompt" => abrir_seletor_de_prompt(e),
        "pasta-trabalho" | "pasta-extra" => {
            let extra = chave == "pasta-extra";
            let atual = e.preferencias.agente.as_ref().map(|a| a.cwd.clone()).unwrap_or_default();
            e.modal = Some(Modal::Entrada {
                titulo: if extra { "Outra pasta que o agente alcança".into() } else { "Onde o agente trabalha (vazio: raiz do projeto)".into() },
                campo: Campo::com(if extra { String::new() } else { atual }),
                acao: AcaoDaEntrada::PastaDoAgente(extra),
            });
        }
        "tirar-pasta" => {
            let itens = e
                .preferencias
                .agente
                .as_ref()
                .map(|a| a.pastas_extras.iter().map(|p| Item::novo("▭", nome_de_pasta(p), p.clone()).com_detalhe(p.clone())).collect())
                .unwrap_or_default();
            e.modal = Some(Modal::Escolha { titulo: "Tirar do alcance do agente".into(), lista: Lista::menu(itens), acao: AcaoDaEscolha::TirarPasta });
        }
        _ => return false,
    }
    true
}

/// Anexa ou tira uma página, pedindo pra gravar o frontmatter.
pub fn mudar_anexo(e: &mut Estado, path: &str, anexar: bool) {
    let Some(c) = e.conversa.as_mut() else { return };
    if anexar {
        if !c.anexos.iter().any(|a| a == path) {
            c.anexos.push(path.to_string());
        }
    } else {
        c.anexos.retain(|a| a != path);
    }
    let (conversa, lista) = (c.path.clone(), c.anexos.clone());
    e.pedidos.push(Pedido::AnexosDaConversa { conversa: conversa.clone(), lista });
    // Mudou o anexo, mudou o peso (ciclo 417).
    e.pedidos.push(Pedido::PesarContexto(conversa));
}

// ---------------------------------------------------------------------
// Prompt padrão (ciclo 341)
// ---------------------------------------------------------------------

/// O seletor de prompt: "Nenhum" e os prompts do vault
/// (`pages/prompts-default/`, `type: prompt`), com os campos das variáveis
/// do prompt em uso — o popover da janela.
pub fn abrir_seletor_de_prompt(e: &mut Estado) {
    let Some(c) = &e.conversa else { return };
    if c.trabalho.is_some() {
        e.aviso = Some("o agente está trabalhando — Ctrl+X interrompe".into());
        return;
    }
    let prompts = anotadinho_core::prompt_padrao::descobrir(e.indice_do_vault.clone());
    let mut itens = vec![Item::novo("○", "Nenhum — escrever do zero", "")];
    itens.extend(prompts.iter().map(|p| {
        let atual = c.prompt.as_ref().is_some_and(|a| a.path == p.path);
        Item::novo(if atual { "●" } else { "▤" }, p.title.clone(), p.path.clone())
    }));
    let mut lista = Lista::menu(itens);
    let campos: Vec<(String, Campo)> = c
        .prompt
        .as_ref()
        .map(|a| a.molde.variaveis.iter().map(|v| (v.clone(), Campo::com(a.valores.get(v).cloned().unwrap_or_default()))).collect())
        .unwrap_or_default();
    if let Some(a) = &c.prompt {
        lista.selecionado = prompts.iter().position(|p| p.path == a.path).map_or(0, |i| i + 1);
    }
    let foco = c.prompt.as_ref().and_then(|a| {
        let pend = a.pendentes();
        a.molde.variaveis.iter().position(|v| pend.contains(v))
    });
    e.modal = Some(Modal::Prompt(super::modais::SeletorDePrompt { lista, campos, foco }));
}

/// Aplica o prompt lido do vault ao campo (depois do `main` ler a página).
pub fn aplicar_prompt(e: &mut Estado, path: &str, conteudo: &str) {
    use anotadinho_core::prompt_padrao::PromptPadrao;
    let titulo = e
        .indice_do_vault
        .iter()
        .find(|p| p.path == path)
        .map(|p| p.title.clone())
        .unwrap_or_else(|| nome_curto(path));
    let molde = PromptPadrao::parse(conteudo);
    let ausentes: Vec<String> = molde.contexto.iter().filter(|p| !e.paginas.iter().any(|x| &x.path == *p)).cloned().collect();
    let Some(c) = e.conversa.as_mut() else { return };
    if !ausentes.is_empty() {
        c.erro = Some(format!("contexto do prompt não encontrado: {}", ausentes.join(", ")));
        e.modal = None;
        return;
    }
    let base = match &c.prompt {
        Some(a) => a.rascunho_antes.clone(),
        None => c.rascunho.texto.clone(),
    };
    let mut valores = std::collections::BTreeMap::new();
    if let Some(primeira) = molde.variaveis.first() {
        if !base.trim().is_empty() {
            valores.insert(primeira.clone(), base.clone());
        }
    }
    let exibido = if molde.variaveis.is_empty() { molde.com_rascunho_ao_final(&base) } else { molde.visualizar_parcial(&valores) };
    let mut anexos = c.anexos.clone();
    for ctx in &molde.contexto {
        if !anexos.contains(ctx) {
            anexos.push(ctx.clone());
        }
    }
    let mudou_anexos = anexos != c.anexos;
    c.anexos = anexos.clone();
    c.rascunho = Campo::com(exibido);
    c.erro = None;
    c.selecionada = c.mensagens.len();
    let sem_variaveis = molde.variaveis.is_empty();
    c.prompt = Some(PromptAtivo { path: path.to_string(), titulo, molde, valores, rascunho_antes: base });
    let conversa = c.path.clone();
    if mudou_anexos {
        e.pedidos.push(Pedido::AnexosDaConversa { conversa, lista: anexos });
    }
    // Um molde sem variáveis já está pronto: o seletor sai da frente. Com
    // variáveis, os campos ficam — é o que falta preencher.
    if sem_variaveis {
        e.modal = None;
    } else {
        abrir_seletor_de_prompt(e);
    }
}

/// "Nenhum": o campo volta ao rascunho de antes.
pub fn tirar_prompt(e: &mut Estado) {
    let Some(c) = e.conversa.as_mut() else { return };
    if let Some(a) = c.prompt.take() {
        c.rascunho = Campo::com(a.rascunho_antes);
    }
}

/// Um valor de variável mudou: o campo mostra o molde com o que já há.
pub fn preencher_variavel(e: &mut Estado, nome: &str, valor: &str) {
    let Some(c) = e.conversa.as_mut() else { return };
    let Some(a) = c.prompt.as_mut() else { return };
    a.valores.insert(nome.to_string(), valor.to_string());
    c.rascunho = Campo::com(a.molde.visualizar_parcial(&a.valores));
}

// ---------------------------------------------------------------------
// Desenho
// ---------------------------------------------------------------------

/// As linhas do corpo de uma mensagem: o markdown dela desenhado como a
/// página, quebrado na largura.
pub(super) fn corpo_da_mensagem(texto: &str, tema: &Tema, largura: usize) -> Vec<Line<'static>> {
    let arvore = anotadinho_core::analise::analisar(texto);
    let linhas = crate::tela::linhas(&arvore);
    let mut fora = Vec::new();
    for l in &linhas {
        if matches!(l.tipo, anotadinho_core::unidade::Tipo::Lista | anotadinho_core::unidade::Tipo::ListaOrdenada) {
            continue;
        }
        let linha = super::linha_estilizada(l, false, false, tema, largura);
        let marca = l.marca.clone();
        let recuo = 2 * l.nivel + if marca.trim().is_empty() { 0 } else { marca.chars().count() + 1 };
        fora.extend(super::quebrar_texto(linha, largura, recuo));
    }
    fora
}

/// Pinta de `fundo` o que não tem fundo e completa até `largura`.
fn com_fundo(linha: Line<'static>, largura: usize, fundo: Color) -> Vec<Span<'static>> {
    let mut usado = 0;
    let mut spans: Vec<Span<'static>> = linha
        .spans
        .into_iter()
        .map(|s| {
            usado += s.content.chars().count();
            let estilo = if s.style.bg.is_none() { s.style.bg(fundo) } else { s.style };
            Span::styled(s.content, estilo)
        })
        .collect();
    if usado < largura {
        spans.push(Span::styled(" ".repeat(largura - usado), Style::default().bg(fundo)));
    }
    spans
}

/// Desenha a conversa no painel de conteúdo.
pub fn desenhar(f: &mut Frame, e: &Estado, area: Rect) {
    let Some(c) = &e.conversa else { return };
    let t = &e.tema;
    let no_foco = e.foco == Foco::Conteudo;
    let borda = Block::default()
        .borders(Borders::ALL)
        .border_style(if no_foco { t.estilo(crate::tema::Realce::BordaFoco) } else { t.estilo(crate::tema::Realce::Borda) })
        .style(t.estilo(crate::tema::Realce::Fundo));
    // O título já está no topo da tela, como na janela: a borda fica sem.
    let mut borda = borda;
    if let Some(aviso) = &e.aviso {
        borda = borda.title_bottom(Line::from(Span::styled(format!(" {aviso} "), t.estilo(crate::tema::Realce::BadgeAtencao))));
    }
    let dentro = borda.inner(area);
    f.render_widget(borda, area);
    let w = dentro.width as usize;
    if w < 20 || dentro.height < 8 {
        return;
    }
    let apagado = Style::default().fg(t.var("text-muted"));
    let texto = Style::default().fg(t.var("text-primary"));
    let agente = e.preferencias.agente.clone().unwrap_or_default();

    // Topo: título · ⚡ agente ······ ⌁ N anexo(s)
    let mut topo: Vec<Line<'static>> = Vec::new();
    // Título comprido encurta com reticências: o agente e os anexos
    // ficam sempre à vista.
    // O peso do contexto ao lado da contagem (ciclo 417): anexar às
    // cegas era o padrão, e com transclusão um anexo traz outros.
    let direita = match &c.peso {
        Some((resumo, _)) => format!("⌁ {} anexo(s) · {resumo} ", c.anexos.len()),
        None => format!("⌁ {} anexo(s) ", c.anexos.len()),
    };
    let cabe = w.saturating_sub(direita.chars().count() + agente.nome.chars().count() + 8).max(8);
    let titulo = if c.titulo.chars().count() > cabe {
        format!("{}…", c.titulo.chars().take(cabe - 1).collect::<String>())
    } else {
        c.titulo.clone()
    };
    let esquerda = vec![
        Span::styled(format!(" {titulo}"), texto.add_modifier(Modifier::BOLD)),
        Span::raw("  "),
        Span::styled(format!(" ϟ {} ", agente.nome), Style::default().bg(t.var("bg-elevated")).fg(t.var("text-muted"))),
    ];
    let usado: usize = esquerda.iter().map(|s| s.content.chars().count()).sum();
    let mut linha = esquerda;
    linha.push(Span::raw(" ".repeat(w.saturating_sub(usado + direita.chars().count()))));
    linha.push(Span::styled(direita, apagado));
    topo.push(Line::from(linha));
    if !c.anexos.is_empty() {
        let pilula = t.pilula(crate::tema::Realce::BadgeInfo);
        let mut spans = vec![Span::raw(" ")];
        for a in &c.anexos {
            spans.push(Span::styled(format!(" {} × ", nome_curto(a)), pilula));
            spans.push(Span::raw(" "));
        }
        topo.push(Line::from(spans));
    }
    // As pastas do agente, como na janela: onde trabalha e as extras.
    {
        let pasta = Style::default().fg(t.var("text-muted")).bg(t.var("bg-elevated"));
        let mut spans = vec![
            Span::raw(" "),
            Span::styled(
                format!(" ▭ {} ", if agente.cwd.trim().is_empty() { "trabalha na raiz do projeto".to_string() } else { nome_de_pasta(&agente.cwd) }),
                pasta,
            ),
        ];
        for p in &agente.pastas_extras {
            spans.push(Span::raw(" "));
            spans.push(Span::styled(format!(" {} × ", nome_de_pasta(p)), pasta));
        }
        if !agente.arg_pasta_extra.trim().is_empty() {
            spans.push(Span::styled("  + pasta", Style::default().fg(t.var("text-muted"))));
        }
        topo.push(Line::from(spans));
    }
    topo.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(t.var("border")))));

    // O campo, embaixo. Escrevendo, ele ACENDE (ciclo 341): a caixa ganha
    // contorno inteiro na cor de destaque e fundo elevado, o rodapé vira
    // "-- INSERÇÃO --" e as mensagens ficam sem seleção — não pode haver
    // dúvida de onde as teclas estão indo.
    let caixa_w = w.saturating_sub(2);
    let no_campo = no_foco && c.selecionada >= c.mensagens.len();
    let fundo_campo = if c.escrevendo { t.var("bg-elevated") } else { t.var("bg-surface") };
    let cor_da_borda = if c.escrevendo {
        t.var("accent-blue")
    } else if no_campo {
        crate::tema::misturar(t.var("accent-blue"), t.var("bg-base"), 0.45)
    } else {
        t.var("border")
    };
    let borda_campo = Style::default().fg(cor_da_borda);
    let mut campo_linhas: Vec<Line<'static>> = Vec::new();
    let rascunho: Vec<Line<'static>> = if c.rascunho.texto.is_empty() && !c.escrevendo {
        vec![Line::from(Span::styled(" Escreva e mande. i escreve · p prompt padrão · Ctrl+J quebra linha.", apagado))]
    } else {
        let spans = if c.escrevendo {
            c.rascunho.spans(texto, "", t)
        } else {
            vec![Span::styled(c.rascunho.texto.clone(), texto)]
        };
        super::quebrar_texto(Line::from([vec![Span::raw(" ")], spans].concat()), caixa_w.saturating_sub(1), 1)
    };
    let altura_do_rascunho = rascunho.len().clamp(3, 8);
    let visiveis_do_rascunho = rascunho.len().saturating_sub(altura_do_rascunho);
    campo_linhas.push(Line::from(Span::styled(format!("▗{}▖", "▄".repeat(w.saturating_sub(2))), borda_campo)));
    for k in 0..altura_do_rascunho {
        let l = rascunho.get(visiveis_do_rascunho + k).cloned().unwrap_or_default();
        let mut spans = vec![Span::styled("▐", borda_campo.bg(fundo_campo))];
        spans.extend(com_fundo(l, caixa_w, fundo_campo));
        spans.push(Span::styled("▌", borda_campo.bg(fundo_campo)));
        campo_linhas.push(Line::from(spans));
    }
    campo_linhas.push(Line::from(Span::styled(format!("▝{}▘", "▀".repeat(w.saturating_sub(2))), borda_campo)));
    if c.prompt.as_ref().is_some_and(|p| !p.pendentes().is_empty()) {
        campo_linhas.push(Line::from(Span::styled(
            " Preencha todos os marcadores antes de visualizar ou enviar.  p preencher",
            Style::default().fg(t.var("warning")),
        )));
    }
    let rodando = c.trabalho.is_some() || c.na_fila.is_some();
    let botao = if rodando {
        Span::styled(" Parar  Ctrl+X ", Style::default().bg(t.var("bg-elevated")).fg(t.var("text-primary")))
    } else {
        let cor = crate::tema::misturar(t.var("accent-blue"), t.var("accent-purple"), 0.5);
        Span::styled(" Enviar  ↵ ", Style::default().bg(cor).fg(Color::Rgb(255, 255, 255)).add_modifier(Modifier::BOLD))
    };
    // O botão do prompt, com o nome do que está em uso.
    let nome_do_prompt = c.prompt.as_ref().map(|p| p.titulo.clone()).unwrap_or_else(|| "Prompt padrão".into());
    let mut esquerda: Vec<Span<'static>> = Vec::new();
    if c.escrevendo {
        esquerda.push(Span::styled(
            " -- INSERÇÃO -- ",
            Style::default().bg(t.var("accent-blue")).fg(t.var("bg-base")).add_modifier(Modifier::BOLD),
        ));
        esquerda.push(Span::styled(" Enter envia · Ctrl+J quebra linha · Ctrl+P prompt · Esc sai", apagado));
    } else {
        esquerda.push(Span::styled(" p ", t.estilo(crate::tema::Realce::Marca)));
        esquerda.push(Span::styled(
            format!("▤ {nome_do_prompt} ⌃"),
            if c.prompt.is_some() { texto } else { apagado },
        ));
        esquerda.push(Span::styled("   : anexar, trocar agente", apagado));
    }
    let usado: usize = esquerda.iter().map(|s| s.content.chars().count()).sum::<usize>() + botao.content.chars().count();
    esquerda.push(Span::raw(" ".repeat(w.saturating_sub(usado + 1))));
    esquerda.push(botao);
    campo_linhas.push(Line::from(esquerda));

    // As mensagens, com a selecionada à vista.
    let altura_msgs = (dentro.height as usize).saturating_sub(topo.len() + campo_linhas.len());
    let box_w = (w * 3 / 4).clamp(20.min(w), w.saturating_sub(2));
    let mut msgs: Vec<Line<'static>> = Vec::new();
    let mut inicio_da_selecionada = 0usize;
    let mut fim_da_selecionada = 0usize;
    if c.mensagens.is_empty() {
        msgs.push(Line::from(Span::styled(" Pergunte algo, peça uma spec, ou mande analisar a página aberta.", apagado)));
    }
    for (i, m) in c.mensagens.iter().enumerate() {
        let aceso = no_foco && i == c.selecionada && !c.escrevendo;
        let (fundo, deslocamento) = match m.autor {
            Autor::Voce => (crate::tema::misturar(t.var("accent-blue"), t.var("bg-base"), 0.14), w.saturating_sub(box_w + 1)),
            Autor::Agente => (t.var("bg-surface"), 1),
        };
        let lado = Style::default().fg(if aceso { t.var("accent-blue") } else { fundo });
        let dentro_w = box_w.saturating_sub(4);
        let mut corpo: Vec<Line<'static>> = vec![Line::from(vec![
            Span::styled(
                m.autor.slug().to_uppercase(),
                Style::default().fg(t.var("text-muted")).add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", m.quando), apagado),
        ])];
        corpo.extend(corpo_da_mensagem(&m.texto, t, dentro_w));
        if m.autor == Autor::Agente && aceso {
            corpo.push(Line::from(Span::styled("↵ virar spec · proposta · execução   y copiar", Style::default().fg(t.var("accent-blue")))));
        }
        if i == c.selecionada {
            inicio_da_selecionada = msgs.len();
        }
        msgs.push(Line::from(vec![
            Span::raw(" ".repeat(deslocamento)),
            Span::styled(format!("▗{}▖", "▄".repeat(box_w.saturating_sub(2))), lado),
        ]));
        for l in corpo {
            let mut spans = vec![Span::raw(" ".repeat(deslocamento)), Span::styled("▐", lado.bg(fundo)), Span::styled(" ", Style::default().bg(fundo))];
            spans.extend(com_fundo(l, dentro_w, fundo));
            spans.push(Span::styled(" ", Style::default().bg(fundo)));
            spans.push(Span::styled("▌", lado.bg(fundo)));
            msgs.push(Line::from(spans));
        }
        msgs.push(Line::from(vec![
            Span::raw(" ".repeat(deslocamento)),
            Span::styled(format!("▝{}▘", "▀".repeat(box_w.saturating_sub(2))), lado),
        ]));
        if i == c.selecionada {
            fim_da_selecionada = msgs.len();
        }
    }
    if let Some(posicao) = c.na_fila.filter(|_| c.trabalho.is_none()) {
        // Esperando vaga (ciclo 408): a pergunta ainda não foi pro
        // arquivo, então a fila é o que a tela tem pra mostrar.
        msgs.push(Line::from(vec![
            Span::styled(" ⋯ ", Style::default().fg(t.var("accent-blue"))),
            Span::styled(format!("{posicao}º na fila, esperando vaga"), apagado),
            Span::styled("   Ctrl+X desistir", apagado),
        ]));
    }
    if let Some((segundos, parcial)) = &c.trabalho {
        let quadros = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
        msgs.push(Line::from(vec![
            Span::styled(format!(" {} ", quadros[*segundos as usize % quadros.len()]), Style::default().fg(t.var("accent-blue"))),
            Span::styled(format!("pensando há {}", anotadinho_core::conversa::duracao_legivel(*segundos)), apagado),
            Span::styled("   Ctrl+X interromper", apagado),
        ]));
        // A saída CRUA, as últimas linhas, como `.conversa__parcial`.
        let linhas: Vec<&str> = parcial.lines().collect();
        for l in &linhas[linhas.len().saturating_sub(8)..] {
            let l: String = l.chars().take(w.saturating_sub(4)).collect();
            msgs.push(Line::from(Span::styled(format!("  │ {l}"), Style::default().fg(t.var("text-muted")))));
        }
    }
    if let Some(erro) = &c.erro {
        for l in super::quebrar_texto(Line::from(Span::styled(format!(" {erro}"), Style::default().fg(t.var("error")))), w, 1) {
            msgs.push(l);
        }
    }
    // Com o campo (ou o fim) selecionado, a lista cola no fim; senão a
    // mensagem selecionada fica à vista, pelo começo dela.
    let topo_msgs = if c.selecionada >= c.mensagens.len() {
        msgs.len().saturating_sub(altura_msgs)
    } else if fim_da_selecionada - inicio_da_selecionada > altura_msgs {
        inicio_da_selecionada
    } else {
        fim_da_selecionada.saturating_sub(altura_msgs).min(inicio_da_selecionada)
    };
    let mut todas = topo;
    let pedaco: Vec<Line<'static>> = msgs.into_iter().skip(topo_msgs).take(altura_msgs).collect();
    let vazio = altura_msgs.saturating_sub(pedaco.len());
    todas.extend(pedaco);
    todas.extend(std::iter::repeat_n(Line::from(""), vazio));
    todas.extend(campo_linhas);
    f.render_widget(Paragraph::new(todas), dentro);
}

/// A prévia do prompt (ciclo 417): o cabeçalho com o peso de cada parte
/// e, embaixo, o texto que vai pro agente — inteiro, como ele recebe.
pub fn mostrar_previa(
    e: &mut Estado,
    prompt: &str,
    orcamento: &anotadinho_core::orcamento::Orcamento,
    cortes: &[anotadinho_core::orcamento::Corte],
) {
    use super::modais::Modal;
    let mut texto = String::new();
    texto.push_str(&format!("O que vai pro agente — {}\n", orcamento.resumo()));
    if orcamento.estourou() {
        texto.push_str("PASSOU DO TETO: tire um anexo ou aumente o teto nas preferências.\n");
    } else if orcamento.apertado() {
        texto.push_str("Perto do teto.\n");
    }
    texto.push('\n');
    for p in &orcamento.partes {
        texto.push_str(&format!(
            "  {:>7}  {}\n",
            format!("~{}", anotadinho_core::orcamento::humano(p.tokens)),
            p.nome
        ));
    }
    if !cortes.is_empty() {
        texto.push_str("\nPodado pra caber:\n");
        for c in cortes {
            texto.push_str(&format!("  · {}\n", c.rotulo()));
        }
    }
    texto.push_str("\n────────────────────────────────────────\n\n");
    texto.push_str(prompt);
    e.modal = Some(Modal::TextoCru { titulo: "Prévia do prompt".into(), texto, rolagem: 0 });
}
