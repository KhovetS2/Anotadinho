//! O construtor de diagramas mermaid na TUI (ciclo 500), o mesmo da
//! janela (ciclos 487–490).
//!
//! A fonte da verdade enquanto o modal está aberto é o TEXTO do código,
//! como na janela: a aba Construtor lê o modelo do núcleo
//! (`anotadinho_core::diagrama`), muda e reescreve o texto; a aba Código
//! edita o texto direto. Por isso abrir e aplicar sem mexer devolve o
//! código byte a byte (RF8 da spec `construtor-mermaid`): ninguém o
//! regenerou. E uma mudança que não muda o MODELO não reescreve o texto —
//! senão girar uma opção e voltar trocaria `A-->B` por `A --> B`.
//!
//! O canvas da janela (fluxograma e estados) vira lista aqui: num terminal
//! não há onde arrastar, e a posição dos nós não muda o desenho (RNF2). A
//! ordem dos nós e das ligações é a do código.

use std::cell::Cell;

use anotadinho_core::diagrama::estados::{Estado as EstadoDoDiagrama, Estados, Ponta, Transicao};
use anotadinho_core::diagrama::fluxograma::{Direcao, Fluxograma, Forma, Ligacao, No, TipoLigacao};
use anotadinho_core::diagrama::gantt::{duracao_valida, Fim, Gantt, Inicio, Marca, Secao, Tarefa};
use anotadinho_core::diagrama::sequencia::{Item, Lado, Participante, Seta, Sequencia};
use anotadinho_core::diagrama::{self, id_valido, Diagrama, Tipo};
use anotadinho_core::unidade::Tipo as TipoDeUnidade;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Paragraph};
use ratatui::Frame;

use super::markdown::Hospedeiro;
use super::modais::Modal;
use super::Estado;
use crate::codigo::{Medida, Vista};
use crate::editor_codigo::EditorDeCodigo;
use crate::componentes::{self, CampoDoFormulario as C, Formulario, RespostaDoFormulario as R, Valor};
use crate::tema::{Realce, Tema};

/// Pra onde o "Aplicar" escreve.
#[derive(Debug, Clone, PartialEq)]
pub enum Destino {
    /// Um bloco que já existe: o mesmo caminho do editor de código (390).
    Bloco { hospedeiro: Hospedeiro, inicio: usize, cerca: String, fecha: String },
    /// Um bloco novo, no lugar do menu `/` (`Estado::bloco_a_inserir`).
    Novo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Aba {
    Construtor,
    Codigo,
}

/// Onde estão as teclas.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Foco {
    /// A lista (ou o código, na aba Código).
    Edicao,
    /// A prévia: as teclas rolam o desenho, como na vista do bloco (498).
    Previa,
}

/// Uma linha da lista do construtor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linha {
    /// Só um título de grupo: o cursor passa por cima.
    Rotulo(&'static str),
    /// O tipo, no diagrama novo (o seletor da janela).
    Tipo,
    Direcao,
    No(usize),
    NovoNo,
    Ligacao(usize),
    NovaLigacao,
    Participante(usize),
    NovoParticipante,
    Item(usize),
    NovaMensagem,
    NovaNota,
    Estado(usize),
    NovoEstado,
    Transicao(usize),
    NovaTransicao,
    Titulo,
    Secao(usize),
    Tarefa(usize, usize),
    NovaTarefa(usize),
    NovaSecao,
}

/// O modal aberto.
#[derive(Debug, Clone, PartialEq)]
pub struct Construtor {
    pub destino: Destino,
    /// O código ao abrir (ou ao escolher o tipo, no novo): é contra ele que
    /// se sabe se mudou — pra pedir confirmação ao sair e pra não gravar à
    /// toa.
    pub original: String,
    /// O código: a fonte da verdade.
    pub codigo: EditorDeCodigo,
    pub aba: Aba,
    pub foco: Foco,
    /// A linha da lista com o cursor.
    pub sel: usize,
    /// O item sendo editado no formulário, e qual é.
    pub form: Option<(Formulario, Linha)>,
    /// O seletor de tipo aberto (diagrama novo), no tipo apontado.
    pub tipos: Option<usize>,
    /// O tipo ainda não foi escolhido: Esc no seletor desiste de tudo.
    pub sem_tipo: bool,
    /// "Descartar as mudanças?" à vista.
    pub confirmar: bool,
    /// O primeiro `d` do `dd`.
    pub d: bool,
    /// O que deu errado na última tecla (id repetido, duração inválida).
    pub aviso: Option<String>,
    /// A rolagem da prévia.
    pub vista: Vista,
    /// O tamanho da prévia no último quadro: até onde a tecla rola.
    pub medida: Cell<Medida>,
    /// Hoje, pro seletor de data e pra primeira tarefa do gantt.
    pub hoje: Option<String>,
}

impl Construtor {
    fn novo(destino: Destino, codigo: String, hoje: Option<String>) -> Self {
        let aba = if diagrama::ler(&codigo).is_ok() { Aba::Construtor } else { Aba::Codigo };
        let mut c = Self {
            destino,
            original: codigo.clone(),
            codigo: EditorDeCodigo::novo(codigo, Some("mermaid".into())),
            aba,
            foco: Foco::Edicao,
            sel: 0,
            form: None,
            tipos: None,
            sem_tipo: false,
            confirmar: false,
            d: false,
            aviso: None,
            vista: Vista::default(),
            medida: Cell::new(Medida::default()),
            hoje,
        };
        c.primeira_linha();
        c
    }

    fn e_novo(&self) -> bool {
        self.destino == Destino::Novo
    }

    fn mudou(&self) -> bool {
        self.codigo.texto() != self.original
    }

    fn modelo(&self) -> Result<Diagrama, diagrama::ForaDoModelo> {
        diagrama::ler(self.codigo.texto())
    }

    fn linhas(&self) -> Vec<Linha> {
        self.modelo().map(|d| linhas_de(&d, self.e_novo())).unwrap_or_default()
    }

    /// Muda o modelo e reescreve o código, se o modelo mudou mesmo.
    fn mudar(&mut self, f: impl FnOnce(&mut Diagrama) -> Result<(), String>) -> bool {
        let Ok(antes) = self.modelo() else { return false };
        let mut d = antes.clone();
        match f(&mut d) {
            Ok(()) => {
                if d != antes {
                    // Voltou ao diagrama de quando abriu (girou a forma e
                    // desgirou): volta o TEXTO de quando abriu, não a forma
                    // canônica dele — senão desfazer à mão ainda deixaria
                    // diff no arquivo.
                    let como_abriu = diagrama::ler(&self.original).is_ok_and(|o| o == d);
                    self.codigo.trocar(if como_abriu { self.original.clone() } else { diagrama::escrever(&d) });
                }
                true
            }
            Err(m) => {
                self.aviso = Some(m);
                false
            }
        }
    }

    /// O cursor nunca para num rótulo de grupo: ao abrir, ao escolher o
    /// tipo, vai pro primeiro item.
    fn primeira_linha(&mut self) {
        self.sel = self.linhas().iter().position(|l| !matches!(l, Linha::Rotulo(_))).unwrap_or(0);
    }

    fn selecionar(&mut self, alvo: Linha) {
        if let Some(i) = self.linhas().iter().position(|l| *l == alvo) {
            self.sel = i;
        }
    }

    fn atual(&self) -> Option<Linha> {
        self.linhas().get(self.sel).copied()
    }

    fn hoje_ou(&self, g: &Gantt) -> String {
        self.hoje.clone().unwrap_or_else(|| {
            // Sem relógio (testes, ou ninguém disse), a primeira data do
            // próprio cronograma: um dia qualquer o empurraria pra longe.
            g.secoes
                .iter()
                .flat_map(|s| &s.tarefas)
                .find_map(|t| match &t.inicio {
                    Inicio::Data(d) => Some(d.clone()),
                    _ => None,
                })
                .unwrap_or_else(|| "2026-01-01".into())
        })
    }
}

// ── as linhas de cada tipo ──────────────────────────────────────────────

fn linhas_de(d: &Diagrama, novo: bool) -> Vec<Linha> {
    let mut v = Vec::new();
    if novo {
        v.push(Linha::Tipo);
    }
    match d {
        Diagrama::Fluxograma(f) => {
            v.push(Linha::Direcao);
            v.push(Linha::Rotulo("Nós"));
            v.extend((0..f.nos.len()).map(Linha::No));
            v.push(Linha::NovoNo);
            v.push(Linha::Rotulo("Ligações"));
            v.extend((0..f.ligacoes.len()).map(Linha::Ligacao));
            v.push(Linha::NovaLigacao);
        }
        Diagrama::Sequencia(s) => {
            v.push(Linha::Rotulo("Participantes"));
            v.extend((0..s.participantes.len()).map(Linha::Participante));
            v.push(Linha::NovoParticipante);
            v.push(Linha::Rotulo("Mensagens e notas, em ordem"));
            v.extend((0..s.itens.len()).map(Linha::Item));
            v.push(Linha::NovaMensagem);
            v.push(Linha::NovaNota);
        }
        Diagrama::Estados(e) => {
            v.push(Linha::Rotulo("Estados"));
            v.extend((0..e.estados.len()).map(Linha::Estado));
            v.push(Linha::NovoEstado);
            v.push(Linha::Rotulo("Transições"));
            v.extend((0..e.transicoes.len()).map(Linha::Transicao));
            v.push(Linha::NovaTransicao);
        }
        Diagrama::Gantt(g) => {
            v.push(Linha::Titulo);
            for (si, s) in g.secoes.iter().enumerate() {
                v.push(Linha::Secao(si));
                v.extend((0..s.tarefas.len()).map(|ti| Linha::Tarefa(si, ti)));
                v.push(Linha::NovaTarefa(si));
            }
            v.push(Linha::NovaSecao);
        }
    }
    v
}

fn nome_da_forma(f: Forma) -> &'static str {
    match f {
        Forma::Caixa => "caixa",
        Forma::Arredondado => "arredondado",
        Forma::Decisao => "decisão",
        Forma::Circulo => "círculo",
    }
}

fn nome_da_ligacao(t: TipoLigacao) -> &'static str {
    match t {
        TipoLigacao::Seta => "seta",
        TipoLigacao::Linha => "linha sem seta",
        TipoLigacao::Pontilhada => "seta tracejada",
        TipoLigacao::Grossa => "seta grossa",
    }
}

fn nome_da_direcao(d: Direcao) -> &'static str {
    match d {
        Direcao::TD => "de cima pra baixo",
        Direcao::TB => "de cima pra baixo (TB)",
        Direcao::LR => "da esquerda pra direita",
        Direcao::RL => "da direita pra esquerda",
        Direcao::BT => "de baixo pra cima",
    }
}

fn nome_da_seta(s: Seta) -> &'static str {
    match s {
        Seta::Sincrona => "pedido (->>)",
        Seta::Resposta => "resposta (-->>)",
        Seta::Assincrona => "sem esperar (-))",
        Seta::AssincronaPontilhada => "sem esperar, tracejada (--))",
        Seta::Linha => "linha (->)",
        Seta::LinhaPontilhada => "linha tracejada (-->)",
        Seta::Cruz => "falhou (-x)",
        Seta::CruzPontilhada => "falhou, tracejada (--x)",
    }
}

const LADOS: [Lado; 3] = [Lado::Direita, Lado::Esquerda, Lado::Sobre];

fn nome_do_lado(l: Lado) -> &'static str {
    match l {
        Lado::Direita => "à direita de",
        Lado::Esquerda => "à esquerda de",
        Lado::Sobre => "sobre",
    }
}

fn nome_da_marca(m: Marca) -> &'static str {
    match m {
        Marca::Feita => "feita",
        Marca::Ativa => "em andamento",
        Marca::Critica => "crítica",
        Marca::Marco => "marco",
    }
}

fn nome_da_ponta(p: &Ponta) -> String {
    match p {
        Ponta::Inicio => "início".into(),
        Ponta::Fim => "fim".into(),
        Ponta::Estado(id) => id.clone(),
    }
}

/// O que a linha mostra.
fn texto_da_linha(d: &Diagrama, l: Linha) -> String {
    match (d, l) {
        (_, Linha::Rotulo(r)) => r.to_string(),
        (_, Linha::Tipo) => format!("Tipo: {}", d.tipo().nome()),
        (Diagrama::Fluxograma(f), Linha::Direcao) => format!("Direção: {} ({})", nome_da_direcao(f.direcao), f.direcao.token()),
        (Diagrama::Fluxograma(f), Linha::No(i)) => {
            let n = &f.nos[i];
            format!("{} · {} · {}", n.id, n.texto, nome_da_forma(n.forma))
        }
        (Diagrama::Fluxograma(f), Linha::Ligacao(k)) => {
            let l = &f.ligacoes[k];
            let rotulo = if l.rotulo.is_empty() { String::new() } else { format!(" |{}|", l.rotulo) };
            format!("{} {} {}{rotulo}", l.de, l.tipo.token(), l.para)
        }
        (Diagrama::Sequencia(s), Linha::Participante(i)) => {
            let p = &s.participantes[i];
            format!("{} · {}{}", p.id, p.nome, if p.ator { " · pessoa" } else { "" })
        }
        (Diagrama::Sequencia(s), Linha::Item(i)) => match &s.itens[i] {
            Item::Mensagem { de, para, seta, texto } => format!("{de} {} {para}: {texto}", seta.token()),
            Item::Nota { lado, alvos, texto } => format!("nota {} {}: {texto}", nome_do_lado(*lado), alvos.join(", ")),
        },
        (Diagrama::Estados(e), Linha::Estado(i)) => {
            let s = &e.estados[i];
            if s.texto == s.id {
                s.id.clone()
            } else {
                format!("{} · {}", s.id, s.texto)
            }
        }
        (Diagrama::Estados(e), Linha::Transicao(k)) => {
            let t = &e.transicoes[k];
            let rotulo = if t.rotulo.is_empty() { String::new() } else { format!(" : {}", t.rotulo) };
            format!("{} → {}{rotulo}", nome_da_ponta(&t.de), nome_da_ponta(&t.para))
        }
        (Diagrama::Gantt(g), Linha::Titulo) => format!("Título: {}", g.titulo.clone().unwrap_or_else(|| "—".into())),
        (Diagrama::Gantt(g), Linha::Secao(si)) => format!("Seção: {}", g.secoes[si].nome.clone().unwrap_or_else(|| "(sem seção)".into())),
        (Diagrama::Gantt(g), Linha::Tarefa(si, ti)) => {
            let t = &g.secoes[si].tarefas[ti];
            let mut partes = vec![t.nome.clone()];
            if let Some(id) = &t.id {
                partes.push(format!("#{id}"));
            }
            partes.push(match &t.inicio {
                Inicio::Anterior => "logo depois".into(),
                Inicio::Data(d) => d.clone(),
                Inicio::Depois(ids) => format!("depois de {}", ids.join(" ")),
            });
            partes.push(match &t.fim {
                Fim::Duracao(x) => x.clone(),
                Fim::Data(x) => format!("até {x}"),
            });
            partes.extend(t.marcas.iter().map(|m| nome_da_marca(*m).to_string()));
            partes.join(" · ")
        }
        (_, Linha::NovoNo) => "+ nó".into(),
        (_, Linha::NovaLigacao) => "+ ligação".into(),
        (_, Linha::NovoParticipante) => "+ participante".into(),
        (_, Linha::NovaMensagem) => "+ mensagem".into(),
        (_, Linha::NovaNota) => "+ nota".into(),
        (_, Linha::NovoEstado) => "+ estado".into(),
        (_, Linha::NovaTransicao) => "+ transição".into(),
        (_, Linha::NovaTarefa(_)) => "+ tarefa".into(),
        (_, Linha::NovaSecao) => "+ seção".into(),
        _ => String::new(),
    }
}

/// O recuo da linha na lista: os itens ficam debaixo do grupo.
fn recuo(l: Linha) -> &'static str {
    match l {
        Linha::Rotulo(_) | Linha::Tipo | Linha::Direcao | Linha::Titulo | Linha::NovaSecao => "",
        Linha::Secao(_) => "",
        Linha::Tarefa(..) | Linha::NovaTarefa(_) => "    ",
        _ => "  ",
    }
}

// ── os ids novos, como a janela ─────────────────────────────────────────

fn proximo(prefixo: &str, usados: &[&str]) -> String {
    (1..).map(|n| format!("{prefixo}{n}")).find(|id| !usados.contains(&id.as_str())).expect("infinito")
}

fn ids_de_tarefa(g: &Gantt) -> Vec<String> {
    g.secoes.iter().flat_map(|s| &s.tarefas).filter_map(|t| t.id.clone()).collect()
}

/// Quantas mensagens e notas citam o participante: apagar um citado
/// deixaria mensagem apontando pro nada (a janela desabilita o apagar).
fn citacoes(s: &Sequencia, id: &str) -> usize {
    s.itens
        .iter()
        .filter(|i| match i {
            Item::Mensagem { de, para, .. } => de == id || para == id,
            Item::Nota { alvos, .. } => alvos.iter().any(|a| a == id),
        })
        .count()
}

/// Quantas tarefas começam depois de `id`.
fn dependentes(g: &Gantt, id: &str) -> usize {
    g.secoes
        .iter()
        .flat_map(|s| &s.tarefas)
        .filter(|t| matches!(&t.inicio, Inicio::Depois(ids) if ids.iter().any(|i| i == id)))
        .count()
}

/// As tarefas de que `(s, t)` pode depender: as que têm id, e as que podem
/// ganhar um (com início escrito — sem ele o mermaid não aceita id). A
/// pessoa escolhe pelo NOME, e o id nasce sozinho, como na janela.
fn dependencias(g: &Gantt, s: usize, t: usize) -> Vec<(String, String)> {
    g.secoes
        .iter()
        .enumerate()
        .flat_map(|(si, sec)| sec.tarefas.iter().enumerate().map(move |(ti, tar)| (si, ti, tar)))
        .filter(|(si, ti, tar)| (*si, *ti) != (s, t) && (tar.id.is_some() || tar.inicio != Inicio::Anterior))
        .map(|(si, ti, tar)| (format!("{si}:{ti}"), tar.nome.clone()))
        .collect()
}

/// O id da tarefa `valor` (`"si:ti"`), dando um novo se ela não tiver.
fn id_de(g: &mut Gantt, valor: &str) -> Option<String> {
    let (si, ti) = valor.split_once(':')?;
    let (si, ti): (usize, usize) = (si.parse().ok()?, ti.parse().ok()?);
    if let Some(id) = g.secoes.get(si)?.tarefas.get(ti)?.id.clone() {
        return Some(id);
    }
    let usados = ids_de_tarefa(g);
    let novo = proximo("t", &usados.iter().map(String::as_str).collect::<Vec<_>>());
    g.secoes[si].tarefas[ti].id = Some(novo.clone());
    Some(novo)
}

fn trocar<T>(v: &mut [T], i: usize, subir: bool) -> Option<usize> {
    if subir && i > 0 && i < v.len() {
        v.swap(i, i - 1);
        Some(i - 1)
    } else if !subir && i + 1 < v.len() {
        v.swap(i, i + 1);
        Some(i + 1)
    } else {
        None
    }
}

/// Um id escrito pela pessoa: válido e sem repetir os `outros`.
fn conferir_id(id: &str, outros: &[&str]) -> Result<(), String> {
    if !id_valido(id) {
        return Err(format!("id `{id}` não serve: letras, números e _ (e não `end`)"));
    }
    if outros.contains(&id) {
        return Err(format!("já existe o id `{id}`"));
    }
    Ok(())
}

// ── adicionar, apagar, mover ────────────────────────────────────────────

/// Acrescenta um item do tipo da linha: numa linha "+ …", no fim do grupo;
/// num item, logo depois dele (como o `o` do vim). Devolve a linha nova.
fn adicionar(c: &mut Construtor, l: Linha, nota: bool) -> Option<Linha> {
    let mut nova = None;
    let hoje = c.modelo().ok().and_then(|d| match d {
        Diagrama::Gantt(g) => Some(c.hoje_ou(&g)),
        _ => None,
    });
    let ok = c.mudar(|d| {
        match (d, l) {
            (Diagrama::Fluxograma(f), Linha::NovoNo | Linha::No(_)) => {
                let usados: Vec<&str> = f.nos.iter().map(|n| n.id.as_str()).collect();
                let id = proximo("N", &usados);
                let onde = if let Linha::No(i) = l { i + 1 } else { f.nos.len() };
                f.nos.insert(onde, No { texto: format!("Nó {}", f.nos.len() + 1), id, forma: Forma::Caixa });
                nova = Some(Linha::No(onde));
            }
            (Diagrama::Fluxograma(f), Linha::NovaLigacao | Linha::Ligacao(_)) => {
                let Some(primeiro) = f.nos.first() else { return Err("primeiro um nó".into()) };
                let de = primeiro.id.clone();
                let para = f.nos.get(1).unwrap_or(primeiro).id.clone();
                let onde = if let Linha::Ligacao(k) = l { k + 1 } else { f.ligacoes.len() };
                f.ligacoes.insert(onde, Ligacao { de, para, tipo: TipoLigacao::Seta, rotulo: String::new() });
                nova = Some(Linha::Ligacao(onde));
            }
            (Diagrama::Sequencia(s), Linha::NovoParticipante | Linha::Participante(_)) => {
                let usados: Vec<&str> = s.participantes.iter().map(|p| p.id.as_str()).collect();
                let id = proximo("P", &usados);
                let onde = if let Linha::Participante(i) = l { i + 1 } else { s.participantes.len() };
                s.participantes.insert(onde, Participante { nome: id.clone(), id, ator: false });
                nova = Some(Linha::Participante(onde));
            }
            (Diagrama::Sequencia(s), Linha::NovaMensagem | Linha::NovaNota | Linha::Item(_)) => {
                let Some(primeiro) = s.participantes.first() else { return Err("primeiro um participante".into()) };
                let de = primeiro.id.clone();
                let para = s.participantes.get(1).unwrap_or(primeiro).id.clone();
                let onde = if let Linha::Item(i) = l { i + 1 } else { s.itens.len() };
                let item = if nota || l == Linha::NovaNota {
                    Item::Nota { lado: Lado::Direita, alvos: vec![de], texto: "nota".into() }
                } else {
                    Item::Mensagem { de, para, seta: Seta::Sincrona, texto: "mensagem".into() }
                };
                s.itens.insert(onde, item);
                nova = Some(Linha::Item(onde));
            }
            (Diagrama::Estados(e), Linha::NovoEstado | Linha::Estado(_)) => {
                let usados: Vec<&str> = e.estados.iter().map(|s| s.id.as_str()).collect();
                let id = proximo("E", &usados);
                let onde = if let Linha::Estado(i) = l { i + 1 } else { e.estados.len() };
                e.estados.insert(onde, EstadoDoDiagrama { texto: format!("Estado {}", e.estados.len() + 1), id });
                nova = Some(Linha::Estado(onde));
            }
            (Diagrama::Estados(e), Linha::NovaTransicao | Linha::Transicao(_)) => {
                // Início direto no fim não é transição que o mermaid desenhe.
                let Some(primeiro) = e.estados.first() else { return Err("primeiro um estado".into()) };
                let para = Ponta::Estado(primeiro.id.clone());
                let onde = if let Linha::Transicao(k) = l { k + 1 } else { e.transicoes.len() };
                e.transicoes.insert(onde, Transicao { de: Ponta::Inicio, para, rotulo: String::new() });
                nova = Some(Linha::Transicao(onde));
            }
            (Diagrama::Gantt(g), Linha::NovaTarefa(si) | Linha::Tarefa(si, _) | Linha::Secao(si)) => {
                let onde = match l {
                    Linha::Tarefa(_, ti) => ti + 1,
                    // Na seção, a tarefa nova é a primeira dela: é a linha
                    // logo abaixo, onde o `o` põe.
                    Linha::Secao(_) => 0,
                    _ => g.secoes[si].tarefas.len(),
                };
                // A primeira tarefa do cronograma precisa de data: sem
                // nenhuma, o mermaid não tem onde pôr o tempo.
                let primeira = g.secoes.iter().all(|s| s.tarefas.is_empty());
                let inicio = if primeira { Inicio::Data(hoje.clone().unwrap_or_default()) } else { Inicio::Anterior };
                g.secoes[si].tarefas.insert(onde, Tarefa { nome: "Tarefa".into(), marcas: Vec::new(), id: None, inicio, fim: Fim::Duracao("3d".into()) });
                nova = Some(Linha::Tarefa(si, onde));
            }
            (Diagrama::Gantt(g), Linha::NovaSecao) => {
                g.com_formato = true;
                g.secoes.push(Secao { nome: Some(format!("Seção {}", g.secoes.len() + 1)), tarefas: Vec::new() });
                nova = Some(Linha::Secao(g.secoes.len() - 1));
            }
            _ => {}
        }
        Ok(())
    });
    if ok {
        nova
    } else {
        None
    }
}

/// Apaga o item da linha. Um nó leva as ligações dele junto (e um estado,
/// as transições): uma ligação para um nó que não existe viraria um nó
/// novo no mermaid. O que é citado e não tem como ir junto (participante
/// com mensagem, tarefa com dependente, seção com tarefa) não se apaga, e
/// o aviso diz por quê — como o botão desabilitado da janela.
fn apagar(c: &mut Construtor, l: Linha) -> bool {
    c.mudar(|d| {
        match (d, l) {
            (Diagrama::Fluxograma(f), Linha::No(i)) if i < f.nos.len() => {
                let id = f.nos.remove(i).id;
                f.ligacoes.retain(|x| x.de != id && x.para != id);
                f.posicoes.remove(&id);
            }
            (Diagrama::Fluxograma(f), Linha::Ligacao(k)) if k < f.ligacoes.len() => {
                f.ligacoes.remove(k);
            }
            (Diagrama::Sequencia(s), Linha::Participante(i)) if i < s.participantes.len() => {
                let usado = citacoes(s, &s.participantes[i].id);
                if usado > 0 {
                    return Err(format!("{} é usado em {usado} mensagens ou notas: apague-as antes", s.participantes[i].id));
                }
                s.participantes.remove(i);
            }
            (Diagrama::Sequencia(s), Linha::Item(i)) if i < s.itens.len() => {
                s.itens.remove(i);
            }
            (Diagrama::Estados(e), Linha::Estado(i)) if i < e.estados.len() => {
                let id = e.estados.remove(i).id;
                e.transicoes.retain(|t| t.de.chave() != id && t.para.chave() != id);
                e.posicoes.remove(&id);
            }
            (Diagrama::Estados(e), Linha::Transicao(k)) if k < e.transicoes.len() => {
                e.transicoes.remove(k);
            }
            (Diagrama::Gantt(g), Linha::Titulo) => g.titulo = None,
            (Diagrama::Gantt(g), Linha::Secao(si)) if si < g.secoes.len() => {
                let n = g.secoes[si].tarefas.len();
                if n > 0 {
                    return Err(format!("a seção tem {n} tarefas: apague-as antes"));
                }
                g.secoes.remove(si);
            }
            (Diagrama::Gantt(g), Linha::Tarefa(si, ti)) => {
                let usada = g.secoes[si].tarefas[ti].id.as_deref().map(|id| dependentes(g, id)).unwrap_or(0);
                if usada > 0 {
                    return Err(format!("{usada} tarefas começam depois desta"));
                }
                g.secoes[si].tarefas.remove(ti);
            }
            _ => {}
        }
        Ok(())
    })
}

/// `K` sobe e `J` desce o item dentro da lista dele (o Subir/Descer da
/// janela).
fn mover(c: &mut Construtor, l: Linha, subir: bool) -> Option<Linha> {
    let mut nova = None;
    c.mudar(|d| {
        nova = match (d, l) {
            (Diagrama::Fluxograma(f), Linha::No(i)) => trocar(&mut f.nos, i, subir).map(Linha::No),
            (Diagrama::Fluxograma(f), Linha::Ligacao(k)) => trocar(&mut f.ligacoes, k, subir).map(Linha::Ligacao),
            (Diagrama::Sequencia(s), Linha::Participante(i)) => trocar(&mut s.participantes, i, subir).map(Linha::Participante),
            (Diagrama::Sequencia(s), Linha::Item(i)) => trocar(&mut s.itens, i, subir).map(Linha::Item),
            (Diagrama::Estados(e), Linha::Estado(i)) => trocar(&mut e.estados, i, subir).map(Linha::Estado),
            (Diagrama::Estados(e), Linha::Transicao(k)) => trocar(&mut e.transicoes, k, subir).map(Linha::Transicao),
            (Diagrama::Gantt(g), Linha::Secao(si)) => {
                // A seção sem nome só existe na primeira posição: o mermaid
                // não tem como escrever uma no meio.
                let destino = if subir { si.wrapping_sub(1) } else { si + 1 };
                if g.secoes.get(destino).is_some_and(|s| s.nome.is_none()) || g.secoes.get(si).is_some_and(|s| s.nome.is_none()) {
                    None
                } else {
                    trocar(&mut g.secoes, si, subir).map(Linha::Secao)
                }
            }
            (Diagrama::Gantt(g), Linha::Tarefa(si, ti)) => trocar(&mut g.secoes[si].tarefas, ti, subir).map(|t| Linha::Tarefa(si, t)),
            _ => None,
        };
        Ok(())
    });
    nova
}

// ── o formulário de cada item ───────────────────────────────────────────

fn opcoes(v: Vec<(String, String)>, atual: &str) -> Valor {
    let i = v.iter().position(|(k, _)| k == atual).unwrap_or(0);
    Valor::Opcoes(v, i)
}

fn ids_de_nos(f: &Fluxograma) -> Vec<(String, String)> {
    f.nos.iter().map(|n| (n.id.clone(), if n.texto == n.id { n.id.clone() } else { format!("{} · {}", n.id, n.texto) })).collect()
}

fn ids_de_participantes(s: &Sequencia) -> Vec<(String, String)> {
    s.participantes.iter().map(|p| (p.id.clone(), if p.nome == p.id { p.id.clone() } else { format!("{} · {}", p.id, p.nome) })).collect()
}

fn pontas(e: &Estados, de: bool) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = e.estados.iter().map(|s| (s.id.clone(), if s.texto == s.id { s.id.clone() } else { format!("{} · {}", s.id, s.texto) })).collect();
    // Do início só se sai; no fim só se chega, como no mermaid.
    if de {
        v.insert(0, ("[*]inicio".into(), "início".into()));
    } else {
        v.push(("[*]fim".into(), "fim".into()));
    }
    v
}

fn ponta_da_chave(chave: &str) -> Ponta {
    match chave {
        "[*]inicio" => Ponta::Inicio,
        "[*]fim" => Ponta::Fim,
        id => Ponta::Estado(id.to_string()),
    }
}

/// O formulário do item da linha, com o título da caixa.
fn formulario(c: &Construtor, d: &Diagrama, l: Linha) -> Option<(Formulario, String)> {
    let t = |s: &str| Valor::Texto(s.to_string());
    let (campos, titulo): (Vec<C>, String) = match (d, l) {
        (Diagrama::Fluxograma(f), Linha::Direcao) => (
            vec![C::novo("direcao", "Direção", opcoes(Direcao::TODAS.iter().map(|x| (x.token().into(), nome_da_direcao(*x).into())).collect(), f.direcao.token()))],
            "Direção".into(),
        ),
        (Diagrama::Fluxograma(f), Linha::No(i)) => {
            let n = &f.nos[i];
            (
                vec![
                    C::novo("id", "Id", t(&n.id)).com_dica("letras, números e _"),
                    C::novo("texto", "Texto", t(&n.texto)),
                    C::novo("forma", "Forma", opcoes(Forma::TODAS.iter().map(|x| (nome_da_forma(*x).into(), nome_da_forma(*x).into())).collect(), nome_da_forma(n.forma))),
                ],
                format!("Nó {}", n.id),
            )
        }
        (Diagrama::Fluxograma(f), Linha::Ligacao(k)) => {
            let x = &f.ligacoes[k];
            (
                vec![
                    C::novo("de", "De", opcoes(ids_de_nos(f), &x.de)),
                    C::novo("para", "Para", opcoes(ids_de_nos(f), &x.para)),
                    C::novo("tipo", "Tipo", opcoes(TipoLigacao::TODAS.iter().map(|y| (y.token().into(), nome_da_ligacao(*y).into())).collect(), x.tipo.token())),
                    C::novo("rotulo", "Rótulo", t(&x.rotulo)).com_dica("sem rótulo"),
                ],
                "Ligação".into(),
            )
        }
        (Diagrama::Sequencia(s), Linha::Participante(i)) => {
            let p = &s.participantes[i];
            (
                vec![
                    C::novo("id", "Id", t(&p.id)).com_dica("letras, números e _"),
                    C::novo("nome", "Nome", t(&p.nome)),
                    C::novo("ator", "Pessoa (bonequinho)", Valor::Booleano(p.ator)),
                ],
                format!("Participante {}", p.id),
            )
        }
        (Diagrama::Sequencia(s), Linha::Item(i)) => match &s.itens[i] {
            Item::Mensagem { de, para, seta, texto } => (
                vec![
                    C::novo("de", "De", opcoes(ids_de_participantes(s), de)),
                    C::novo("seta", "Seta", opcoes(Seta::TODAS.iter().map(|x| (x.token().into(), nome_da_seta(*x).into())).collect(), seta.token())),
                    C::novo("para", "Para", opcoes(ids_de_participantes(s), para)),
                    C::novo("texto", "Texto", t(texto)),
                ],
                "Mensagem".into(),
            ),
            Item::Nota { lado, alvos, texto } => {
                let mut segundo = vec![(String::new(), "—".to_string())];
                segundo.extend(ids_de_participantes(s));
                (
                    vec![
                        C::novo("lado", "Lado", opcoes(LADOS.iter().map(|x| (nome_do_lado(*x).into(), nome_do_lado(*x).into())).collect(), nome_do_lado(*lado))),
                        C::novo("alvo", "Participante", opcoes(ids_de_participantes(s), &alvos[0])),
                        C::novo("alvo2", "E também", opcoes(segundo, alvos.get(1).map(String::as_str).unwrap_or(""))),
                        C::novo("texto", "Texto", t(texto)),
                    ],
                    "Nota".into(),
                )
            }
        },
        (Diagrama::Estados(e), Linha::Estado(i)) => {
            let s = &e.estados[i];
            (
                vec![C::novo("id", "Id", t(&s.id)).com_dica("letras, números e _"), C::novo("texto", "Nome", t(&s.texto))],
                format!("Estado {}", s.id),
            )
        }
        (Diagrama::Estados(e), Linha::Transicao(k)) => {
            let x = &e.transicoes[k];
            (
                vec![
                    C::novo("de", "De", opcoes(pontas(e, true), x.de.chave())),
                    C::novo("para", "Para", opcoes(pontas(e, false), x.para.chave())),
                    C::novo("rotulo", "Rótulo", t(&x.rotulo)).com_dica("sem rótulo"),
                ],
                "Transição".into(),
            )
        }
        (Diagrama::Gantt(g), Linha::Titulo) => (vec![C::novo("titulo", "Título", t(g.titulo.as_deref().unwrap_or(""))).com_dica("sem título")], "Título".into()),
        (Diagrama::Gantt(g), Linha::Secao(si)) => (
            vec![C::novo("nome", "Nome", t(g.secoes[si].nome.as_deref().unwrap_or(""))).com_dica(if si == 0 { "sem seção" } else { "Seção" })],
            "Seção".into(),
        ),
        (Diagrama::Gantt(g), Linha::Tarefa(si, ti)) => {
            let x = &g.secoes[si].tarefas[ti];
            let hoje = c.hoje_ou(g);
            let (tipo_inicio, data_inicio, depois) = match &x.inicio {
                Inicio::Anterior => ("anterior", String::new(), String::new()),
                Inicio::Data(d) => ("data", d.clone(), String::new()),
                Inicio::Depois(ids) => {
                    let alvo = ids.first().cloned().unwrap_or_default();
                    // A opção é a TAREFA (`si:ti`), achada pelo id.
                    let valor = g
                        .secoes
                        .iter()
                        .enumerate()
                        .flat_map(|(a, s)| s.tarefas.iter().enumerate().map(move |(b, t)| (a, b, t)))
                        .find(|(_, _, t)| t.id.as_deref() == Some(alvo.as_str()))
                        .map(|(a, b, _)| format!("{a}:{b}"))
                        .unwrap_or_default();
                    ("depois", String::new(), valor)
                }
            };
            let (tipo_fim, duracao, data_fim) = match &x.fim {
                Fim::Duracao(d) => ("duracao", d.clone(), String::new()),
                Fim::Data(d) => ("data", "3d".to_string(), d.clone()),
            };
            let mut campos = vec![
                C::novo("nome", "Nome", t(&x.nome)),
                C::novo("id", "Id", t(x.id.as_deref().unwrap_or(""))).com_dica("pra outra tarefa começar depois desta"),
                C::novo("inicio", "Começa", opcoes(vec![("anterior".into(), "logo depois da anterior".into()), ("data".into(), "numa data".into()), ("depois".into(), "depois de…".into())], tipo_inicio)),
                C::novo("data_inicio", "Data de início", t(if data_inicio.is_empty() { &hoje } else { &data_inicio })).como_data().com_dica("AAAA-MM-DD"),
                C::novo("depois", "Depois de", opcoes(dependencias(g, si, ti), &depois)),
                C::novo("fim", "Termina", opcoes(vec![("duracao".into(), "dura".into()), ("data".into(), "até".into())], tipo_fim)),
                C::novo("duracao", "Duração", t(&duracao)).com_dica("3d, 2w, 12h"),
                C::novo("data_fim", "Data do fim", t(if data_fim.is_empty() { &hoje } else { &data_fim })).como_data().com_dica("AAAA-MM-DD"),
            ];
            for m in Marca::TODAS {
                campos.push(C::novo(chave_da_marca(m), nome_da_marca(m), Valor::Booleano(x.marcas.contains(&m))));
            }
            (campos, format!("Tarefa {}", x.nome))
        }
        _ => return None,
    };
    let mut form = Formulario::novo(campos);
    form.hoje = c.hoje.clone();
    ajustar(&mut form);
    Some((form, titulo))
}

fn chave_da_marca(m: Marca) -> &'static str {
    match m {
        Marca::Feita => "feita",
        Marca::Ativa => "ativa",
        Marca::Critica => "critica",
        Marca::Marco => "marco",
    }
}

/// Mostra só os campos que valem na escolha atual (a data só com "numa
/// data", o segundo alvo só com "sobre").
fn ajustar(form: &mut Formulario) {
    if form.valor("inicio").is_some() {
        let inicio = form.escolha("inicio");
        let fim = form.escolha("fim");
        form.esconder("data_inicio", inicio != "data");
        form.esconder("depois", inicio != "depois");
        form.esconder("duracao", fim != "duracao");
        form.esconder("data_fim", fim != "data");
    }
    if form.valor("lado").is_some() {
        let sobre = form.escolha("lado") == nome_do_lado(Lado::Sobre);
        form.esconder("alvo2", !sobre);
    }
}

/// O formulário mudou: o item ganha os valores, se valem.
fn aplicar_formulario(c: &mut Construtor, l: Linha, form: &Formulario) -> bool {
    let txt = |k: &str| form.texto(k).trim().to_string();
    c.mudar(|d| {
        match (d, l) {
            (Diagrama::Fluxograma(f), Linha::Direcao) => {
                if let Some(x) = Direcao::TODAS.into_iter().find(|x| x.token() == form.escolha("direcao")) {
                    f.direcao = x;
                }
            }
            (Diagrama::Fluxograma(f), Linha::No(i)) => {
                let velho = f.nos[i].id.clone();
                let id = txt("id");
                if id != velho {
                    let outros: Vec<&str> = f.nos.iter().map(|n| n.id.as_str()).filter(|x| *x != velho).collect();
                    conferir_id(&id, &outros)?;
                    // O id é o que as ligações citam: troca junto.
                    for x in &mut f.ligacoes {
                        if x.de == velho {
                            x.de = id.clone();
                        }
                        if x.para == velho {
                            x.para = id.clone();
                        }
                    }
                    if let Some(p) = f.posicoes.remove(&velho) {
                        f.posicoes.insert(id.clone(), p);
                    }
                }
                let n = &mut f.nos[i];
                n.id = id;
                // Texto vazio vira o id: `A[]` não é mermaid.
                n.texto = if txt("texto").is_empty() { n.id.clone() } else { txt("texto") };
                if let Some(x) = Forma::TODAS.into_iter().find(|x| nome_da_forma(*x) == form.escolha("forma")) {
                    n.forma = x;
                }
            }
            (Diagrama::Fluxograma(f), Linha::Ligacao(k)) => {
                let x = &mut f.ligacoes[k];
                x.de = form.escolha("de");
                x.para = form.escolha("para");
                if let Some(y) = TipoLigacao::TODAS.into_iter().find(|y| y.token() == form.escolha("tipo")) {
                    x.tipo = y;
                }
                x.rotulo = txt("rotulo");
            }
            (Diagrama::Sequencia(s), Linha::Participante(i)) => {
                let velho = s.participantes[i].id.clone();
                let id = txt("id");
                if id != velho {
                    let outros: Vec<&str> = s.participantes.iter().map(|p| p.id.as_str()).filter(|x| *x != velho).collect();
                    conferir_id(&id, &outros)?;
                    for item in &mut s.itens {
                        match item {
                            Item::Mensagem { de, para, .. } => {
                                for p in [de, para] {
                                    if *p == velho {
                                        *p = id.clone();
                                    }
                                }
                            }
                            Item::Nota { alvos, .. } => alvos.iter_mut().filter(|a| **a == velho).for_each(|a| *a = id.clone()),
                        }
                    }
                }
                let p = &mut s.participantes[i];
                p.id = id;
                p.nome = if txt("nome").is_empty() { p.id.clone() } else { txt("nome") };
                p.ator = form.booleano("ator");
            }
            (Diagrama::Sequencia(s), Linha::Item(i)) => match &mut s.itens[i] {
                Item::Mensagem { de, para, seta, texto } => {
                    *de = form.escolha("de");
                    *para = form.escolha("para");
                    if let Some(x) = Seta::TODAS.into_iter().find(|x| x.token() == form.escolha("seta")) {
                        *seta = x;
                    }
                    *texto = txt("texto");
                }
                Item::Nota { lado, alvos, texto } => {
                    if let Some(x) = LADOS.into_iter().find(|x| nome_do_lado(*x) == form.escolha("lado")) {
                        *lado = x;
                    }
                    *alvos = vec![form.escolha("alvo")];
                    // Dois alvos só existem em "sobre".
                    let segundo = form.escolha("alvo2");
                    if *lado == Lado::Sobre && !segundo.is_empty() && segundo != alvos[0] {
                        alvos.push(segundo);
                    }
                    *texto = txt("texto");
                }
            },
            (Diagrama::Estados(e), Linha::Estado(i)) => {
                let velho = e.estados[i].id.clone();
                let id = txt("id");
                if id != velho {
                    let outros: Vec<&str> = e.estados.iter().map(|s| s.id.as_str()).filter(|x| *x != velho).collect();
                    conferir_id(&id, &outros)?;
                    for t in &mut e.transicoes {
                        for p in [&mut t.de, &mut t.para] {
                            if *p == Ponta::Estado(velho.clone()) {
                                *p = Ponta::Estado(id.clone());
                            }
                        }
                    }
                    if let Some(p) = e.posicoes.remove(&velho) {
                        e.posicoes.insert(id.clone(), p);
                    }
                }
                let s = &mut e.estados[i];
                s.id = id;
                s.texto = if txt("texto").is_empty() { s.id.clone() } else { txt("texto") };
            }
            (Diagrama::Estados(e), Linha::Transicao(k)) => {
                let (de, para) = (ponta_da_chave(&form.escolha("de")), ponta_da_chave(&form.escolha("para")));
                if de == Ponta::Inicio && para == Ponta::Fim {
                    return Err("do início direto pro fim não é transição que o mermaid desenhe".into());
                }
                let t = &mut e.transicoes[k];
                t.de = de;
                t.para = para;
                t.rotulo = txt("rotulo");
            }
            (Diagrama::Gantt(g), Linha::Titulo) => g.titulo = (!txt("titulo").is_empty()).then(|| txt("titulo")),
            (Diagrama::Gantt(g), Linha::Secao(si)) => {
                // A seção sem nome só existe na primeira posição: vazio
                // volta a ser "sem seção" na primeira, e "Seção" nas outras.
                g.secoes[si].nome = if !txt("nome").is_empty() {
                    Some(txt("nome"))
                } else if si == 0 {
                    None
                } else {
                    Some("Seção".into())
                };
            }
            (Diagrama::Gantt(g), Linha::Tarefa(si, ti)) => {
                let velho = g.secoes[si].tarefas[ti].id.clone();
                let id = txt("id");
                let inicio = match form.escolha("inicio").as_str() {
                    "data" => {
                        let d = txt("data_inicio");
                        if !anotadinho_core::detalhes::data_valida(&d) {
                            return Err("data de início: AAAA-MM-DD (Enter abre o calendário)".into());
                        }
                        Inicio::Data(d)
                    }
                    "depois" => {
                        let alvo = form.escolha("depois");
                        if alvo.is_empty() {
                            return Err("não há tarefa com início escrito pra esta depender".into());
                        }
                        let id = id_de(g, &alvo).ok_or("a tarefa escolhida sumiu")?;
                        // `after a1 a2`: o formulário escolhe UMA. Sem trocar
                        // a primeira, as outras ficam — mudar o nome da
                        // tarefa não pode apagar dependência.
                        match &g.secoes[si].tarefas[ti].inicio {
                            Inicio::Depois(ids) if ids.first() == Some(&id) => Inicio::Depois(ids.clone()),
                            _ => Inicio::Depois(vec![id]),
                        }
                    }
                    _ => Inicio::Anterior,
                };
                // O `id_de` pode ter dado id a outra tarefa: os outros ids
                // são lidos DEPOIS dele.
                if Some(id.as_str()) != velho.as_deref() {
                    if let Some(v) = &velho {
                        let n = dependentes(g, v);
                        if n > 0 {
                            return Err(format!("{n} tarefas começam depois desta: o id fica"));
                        }
                    }
                    if !id.is_empty() {
                        let todos = ids_de_tarefa(g);
                        let outros: Vec<&str> = todos.iter().map(String::as_str).filter(|x| Some(*x) != velho.as_deref()).collect();
                        conferir_id(&id, &outros)?;
                    }
                }
                if !id.is_empty() && inicio == Inicio::Anterior {
                    return Err("com início logo depois da anterior, o mermaid não aceita id".into());
                }
                let fim = match form.escolha("fim").as_str() {
                    "data" => {
                        let d = txt("data_fim");
                        if !anotadinho_core::detalhes::data_valida(&d) {
                            return Err("data do fim: AAAA-MM-DD (Enter abre o calendário)".into());
                        }
                        Fim::Data(d)
                    }
                    _ => {
                        let d = txt("duracao");
                        if !duracao_valida(&d) {
                            return Err("duração: número e d, w ou h (3d, 2w, 12h)".into());
                        }
                        Fim::Duracao(d)
                    }
                };
                let t = &mut g.secoes[si].tarefas[ti];
                t.nome = if txt("nome").is_empty() { "Tarefa".into() } else { txt("nome") };
                t.id = (!id.is_empty()).then_some(id);
                t.inicio = inicio;
                t.fim = fim;
                // A ordem das marcas é a do mermaid, pra o código não mudar
                // por ter marcado numa ordem diferente.
                t.marcas = Marca::TODAS.into_iter().filter(|m| form.booleano(chave_da_marca(*m))).collect();
            }
            _ => {}
        }
        Ok(())
    })
}

// ── abrir ───────────────────────────────────────────────────────────────

fn mermaid_no_cursor(e: &Estado) -> bool {
    e.arvore
        .em(&e.cursor)
        .is_some_and(|u| matches!(&u.tipo, TipoDeUnidade::Codigo(Some(l)) if l.eq_ignore_ascii_case("mermaid")))
}

/// Abre o construtor no bloco mermaid do cursor. O código e onde gravar
/// vêm do mesmo caminho do editor de 390 (abre-se ele e troca-se pelo
/// construtor): duas extrações do miolo do bloco divergiriam, e o "aplicar
/// sem mexer" deixaria de devolver o arquivo igual.
pub(super) fn abrir_do_cursor(e: &mut Estado) -> bool {
    if !mermaid_no_cursor(e) {
        return false;
    }
    let antes = e.modal.take();
    // Desde o 506, editar um bloco mermaid já abre o construtor
    // (`abrir_no_bloco`, chamado do caminho do editor de 390).
    super::markdown::enter_na_tabela_md(e);
    if matches!(e.modal, Some(Modal::Construtor(_))) {
        return true;
    }
    e.modal = e.modal.take().or(antes);
    false
}

/// O construtor no bloco que o editor de código abriria: o mesmo miolo e
/// o mesmo lugar de gravar, extraídos uma vez só (`markdown::no_markdown`).
pub(super) fn abrir_no_bloco(e: &mut Estado, hospedeiro: Hospedeiro, inicio: usize, cerca: String, fecha: String, codigo: String) {
    let c = Construtor::novo(Destino::Bloco { hospedeiro, inicio, cerca, fecha }, codigo, e.hoje.clone());
    e.modal = Some(Modal::Construtor(c));
}

/// O diagrama novo do menu `/`: o seletor de tipo, já com o exemplo do
/// primeiro na prévia, como o "Novo diagrama" da janela.
pub(super) fn abrir_novo(e: &mut Estado) {
    let codigo = diagrama::escrever(&Diagrama::exemplo(Tipo::Fluxograma));
    let mut c = Construtor::novo(Destino::Novo, codigo, e.hoje.clone());
    c.tipos = Some(0);
    c.sem_tipo = true;
    e.modal = Some(Modal::Construtor(c));
}

/// A entrada da barra de comandos, só com o cursor num bloco mermaid.
pub(super) fn comandos(e: &Estado) -> Vec<componentes::Item> {
    if e.conversa.is_some() || e.especial.is_some() || !mermaid_no_cursor(e) {
        return Vec::new();
    }
    vec![componentes::Item::novo("⋈", "Construtor do diagrama…", "construtor-mermaid").com_detalhe("montar o mermaid do bloco por listas")]
}

pub(super) fn executar(e: &mut Estado, chave: &str) -> bool {
    if chave != "construtor-mermaid" {
        return false;
    }
    if !abrir_do_cursor(e) {
        e.aviso = Some("o cursor não está num bloco ```mermaid".into());
    }
    true
}

// ── teclas ──────────────────────────────────────────────────────────────

/// Fecha sem gravar. O bloco novo do menu `/` desiste junto.
fn desistir(e: &mut Estado, c: &Construtor) {
    if c.e_novo() {
        e.bloco_a_inserir = None;
    }
}

/// "Aplicar" (ou "Inserir", no novo). `false` deixa o modal aberto.
fn aplicar(e: &mut Estado, c: &mut Construtor) -> bool {
    let texto = c.codigo.texto().to_string();
    match &c.destino {
        Destino::Novo => {
            let t = texto.trim_end();
            if t.trim().is_empty() {
                c.aviso = Some("o diagrama está vazio".into());
                return false;
            }
            super::markdown::inserir_trecho(e, &format!("```mermaid\n{t}\n```"));
        }
        // Sem mudança não grava: o arquivo fica igual por construção, e não
        // nasce um passo de desfazer que não desfaz nada.
        Destino::Bloco { .. } if !c.mudou() => e.aviso = Some("o diagrama não mudou".into()),
        Destino::Bloco { hospedeiro, inicio, cerca, fecha } => {
            let (h, i, a, b) = (hospedeiro.clone(), *inicio, cerca.clone(), fecha.clone());
            super::markdown::gravar_codigo(e, &h, i, &a, &b, &texto);
        }
    }
    true
}

pub(super) fn tecla(e: &mut Estado, mut c: Construtor, tecla: &str) {
    c.aviso = None;
    let fechar = if c.confirmar {
        tecla_na_confirmacao(e, &mut c, tecla)
    } else if let Some(i) = c.tipos {
        tecla_nos_tipos(e, &mut c, i, tecla)
    } else if c.foco == Foco::Previa {
        tecla_na_previa(&mut c, tecla);
        false
    } else if tecla == "Ctrl+s" && c.form.as_ref().is_none_or(|(f, _)| f.editando.is_none() && f.calendario.is_none()) {
        aplicar(e, &mut c)
    } else if c.form.is_some() {
        tecla_no_formulario(&mut c, tecla);
        false
    } else if c.aba == Aba::Codigo {
        tecla_no_codigo(e, &mut c, tecla)
    } else {
        tecla_na_lista(e, &mut c, tecla)
    };
    if !fechar {
        e.modal = Some(Modal::Construtor(c));
    }
}

fn sair(e: &mut Estado, c: &mut Construtor) -> bool {
    if c.mudou() {
        c.confirmar = true;
        return false;
    }
    desistir(e, c);
    true
}

fn tecla_na_confirmacao(e: &mut Estado, c: &mut Construtor, tecla: &str) -> bool {
    match tecla {
        "y" | "s" | "Enter" => {
            desistir(e, c);
            e.aviso = Some("mudanças no diagrama descartadas".into());
            true
        }
        "n" | "Escape" | "q" => {
            c.confirmar = false;
            false
        }
        _ => false,
    }
}

fn tecla_nos_tipos(e: &mut Estado, c: &mut Construtor, i: usize, tecla: &str) -> bool {
    let n = Tipo::TODOS.len();
    let mostrar = |c: &mut Construtor, i: usize| {
        c.tipos = Some(i);
        // O exemplo do tipo apontado já vai pra prévia: a pessoa vê o
        // que escolhe antes de escolher.
        let codigo = diagrama::escrever(&Diagrama::exemplo(Tipo::TODOS[i]));
        c.codigo = EditorDeCodigo::novo(codigo.clone(), Some("mermaid".into()));
        c.original = codigo;
        c.vista = Vista::default();
    };
    match tecla {
        "j" | "ArrowDown" => mostrar(c, (i + 1).min(n - 1)),
        "k" | "ArrowUp" => mostrar(c, i.saturating_sub(1)),
        "Enter" | "l" => {
            mostrar(c, i);
            c.tipos = None;
            c.sem_tipo = false;
            c.aba = Aba::Construtor;
            c.primeira_linha();
        }
        "Escape" | "q" if c.sem_tipo => {
            desistir(e, c);
            return true;
        }
        "Escape" | "q" => c.tipos = None,
        _ => {}
    }
    false
}

fn tecla_na_previa(c: &mut Construtor, tecla: &str) {
    let m = c.medida.get();
    let meia = (m.alto / 2).max(1);
    let (x, y) = (c.vista.x, c.vista.y);
    let (nx, ny) = match tecla {
        "j" | "ArrowDown" => (x, y + 1),
        "k" | "ArrowUp" => (x, y.saturating_sub(1)),
        "l" => (x + 4, y),
        "h" => (x.saturating_sub(4), y),
        "ArrowRight" => (x + 1, y),
        "ArrowLeft" => (x.saturating_sub(1), y),
        "Ctrl+d" | "PageDown" => (x, y + meia),
        "Ctrl+u" | "PageUp" => (x, y.saturating_sub(meia)),
        "L" => (x + m.largo / 2, y),
        "H" => (x.saturating_sub(m.largo / 2), y),
        "g" | "Home" => (x, 0),
        "G" | "End" => (x, m.max_y()),
        "0" => (0, y),
        "$" => (m.max_x(), y),
        "Escape" | "q" | "p" | "Ctrl+p" | "Tab" => {
            c.foco = Foco::Edicao;
            return;
        }
        _ => return,
    };
    c.vista.x = nx.min(m.max_x());
    c.vista.y = ny.min(m.max_y());
}

fn tecla_no_formulario(c: &mut Construtor, tecla: &str) {
    let Some((mut form, l)) = c.form.take() else { return };
    match form.tecla(tecla) {
        R::Fechar => return,
        R::Mudou => {
            ajustar(&mut form);
            aplicar_formulario(c, l, &form);
        }
        _ => {}
    }
    c.form = Some((form, l));
}

fn tecla_no_codigo(e: &mut Estado, c: &mut Construtor, tecla: &str) -> bool {
    // A busca do editor (509) segura o Esc e as letras enquanto está aberta.
    if c.codigo.prende(tecla) {
        c.codigo.tecla(tecla);
        return false;
    }
    match tecla {
        "Escape" => return sair(e, c),
        "Tab" => match c.modelo() {
            Ok(_) => {
                c.aba = Aba::Construtor;
                c.sel = c.sel.min(c.linhas().len().saturating_sub(1));
            }
            Err(f) => c.aviso = Some(format!("o construtor não monta este diagrama — {}", f.mensagem())),
        },
        "Ctrl+p" => c.foco = Foco::Previa,
        // A linguagem do bloco é mermaid enquanto o construtor está
        // aberto: trocar seria sair dele.
        "Ctrl+l" => c.aviso = Some("o construtor é de mermaid: troque a linguagem depois, no editor de código".into()),
        // O editor de código de 506: indentação (o mapa mental vive dela),
        // desfazer, Home esperto, a vista que segue o cursor.
        outra => {
            c.codigo.tecla(outra);
        }
    }
    false
}

fn andar(c: &mut Construtor, passo: i64) {
    let linhas = c.linhas();
    let mut i = c.sel as i64;
    loop {
        let prox = i + passo;
        if prox < 0 || prox >= linhas.len() as i64 {
            return;
        }
        i = prox;
        if !matches!(linhas[i as usize], Linha::Rotulo(_)) {
            c.sel = i as usize;
            return;
        }
    }
}

fn abrir_formulario(c: &mut Construtor, l: Linha) {
    let Ok(d) = c.modelo() else { return };
    if let Some((form, _)) = formulario(c, &d, l) {
        c.form = Some((form, l));
    }
}

fn tecla_na_lista(e: &mut Estado, c: &mut Construtor, tecla: &str) -> bool {
    // Fora do modelo (o código mudou por fora), a lista não tem o que
    // mostrar: o código é o caminho.
    if c.modelo().is_err() {
        c.aba = Aba::Codigo;
        return tecla_no_codigo(e, c, tecla);
    }
    let d_antes = std::mem::take(&mut c.d);
    let l = c.atual();
    // O cursor nunca para num rótulo de grupo.
    if matches!(l, Some(Linha::Rotulo(_))) {
        andar(c, 1);
    }
    let l = c.atual();
    match tecla {
        "j" | "ArrowDown" => andar(c, 1),
        "k" | "ArrowUp" => andar(c, -1),
        "g" | "Home" => {
            c.sel = 0;
            if matches!(c.atual(), Some(Linha::Rotulo(_))) {
                andar(c, 1);
            }
        }
        "G" | "End" => c.sel = c.linhas().len().saturating_sub(1),
        "Tab" => c.aba = Aba::Codigo,
        "p" | "Ctrl+p" => c.foco = Foco::Previa,
        "Escape" | "q" => return sair(e, c),
        "Enter" | "i" | "a" => match l {
            Some(Linha::Tipo) => {
                let atual = c.modelo().map(|d| d.tipo()).ok();
                c.tipos = Some(Tipo::TODOS.iter().position(|t| Some(*t) == atual).unwrap_or(0));
            }
            Some(
                x @ (Linha::NovoNo
                | Linha::NovaLigacao
                | Linha::NovoParticipante
                | Linha::NovaMensagem
                | Linha::NovaNota
                | Linha::NovoEstado
                | Linha::NovaTransicao
                | Linha::NovaTarefa(_)
                | Linha::NovaSecao),
            ) => {
                if let Some(nova) = adicionar(c, x, false) {
                    c.selecionar(nova);
                    abrir_formulario(c, nova);
                }
            }
            Some(x) => abrir_formulario(c, x),
            None => {}
        },
        // `o` cria logo depois (e `n`, uma nota, na sequência), e já abre o
        // formulário do novo, como o `o` do vim já entra em inserção.
        "o" | "n" => {
            if let Some(x) = l {
                let nota = tecla == "n";
                if nota && !matches!(x, Linha::Item(_) | Linha::NovaMensagem | Linha::NovaNota) {
                    return false;
                }
                let x = if nota && !matches!(x, Linha::Item(_)) { Linha::NovaNota } else { x };
                if let Some(nova) = adicionar(c, x, nota) {
                    c.selecionar(nova);
                    abrir_formulario(c, nova);
                }
            }
        }
        "d" if !d_antes => c.d = true,
        "d" | "x" | "Delete" => {
            if let Some(x) = l {
                let n = c.linhas().len();
                if apagar(c, x) {
                    // O cursor fica no lugar (agora no item seguinte), sem
                    // cair num rótulo.
                    let novas = c.linhas();
                    if novas.len() < n {
                        c.sel = c.sel.min(novas.len().saturating_sub(1));
                        if matches!(c.atual(), Some(Linha::Rotulo(_))) {
                            andar(c, -1);
                        }
                    }
                }
            }
        }
        "J" | "K" => {
            if let Some(nova) = l.and_then(|x| mover(c, x, tecla == "K")) {
                c.selecionar(nova);
            }
        }
        _ => {}
    }
    false
}

// ── desenho ─────────────────────────────────────────────────────────────

fn rodape(c: &Construtor) -> &'static str {
    if c.confirmar {
        "y descarta · n volta"
    } else if c.tipos.is_some() {
        "j k tipo · Enter escolhe · Esc"
    } else if c.foco == Foco::Previa {
        "prévia: hjkl rola · g G 0 $ · Esc volta"
    } else if let Some((f, _)) = &c.form {
        if f.editando.is_some() {
            "Enter/Esc confirmar"
        } else {
            "j k · Enter edita · h l opção · ~ marca · Esc volta"
        }
    } else if c.aba == Aba::Codigo {
        "Tab construtor · Ctrl+P prévia · Ctrl+S aplica · Esc cancela"
    } else {
        "j k · Enter edita · o novo · dd apaga · J K move · p prévia · Tab código · Ctrl+S aplica · Esc"
    }
}

pub(super) fn desenhar(f: &mut Frame, e: &Estado, c: &Construtor) {
    let t = &e.tema;
    let tela = f.area();
    let area = componentes::area_do_modal(tela, tela.width, tela.height);
    let lido = c.modelo();
    let titulo = match (&lido, c.e_novo()) {
        (Ok(d), true) => format!("Novo diagrama · {} · Ctrl+S insere", d.tipo().nome()),
        (Ok(d), false) => format!("Editar diagrama · {} · Ctrl+S aplica", d.tipo().nome()),
        (Err(_), true) => "Novo diagrama · Ctrl+S insere".into(),
        (Err(_), false) => "Editar diagrama · Ctrl+S aplica".into(),
    };
    let dentro = componentes::desenhar_modal(f, area, &titulo, rodape(c), t);
    if dentro.height < 4 || dentro.width < 20 {
        return;
    }
    let apagado = Style::default().fg(t.var("text-muted"));
    let normal = Style::default().fg(t.var("text-primary"));

    // As abas, e o aviso da última tecla do lado.
    let aba = |nome: &str, ativa: bool, possivel: bool| {
        let estilo = if ativa {
            t.estilo(Realce::Cursor).add_modifier(Modifier::BOLD)
        } else if possivel {
            normal.bg(t.var("bg-elevated"))
        } else {
            apagado.add_modifier(Modifier::CROSSED_OUT)
        };
        Span::styled(format!(" {nome} "), estilo)
    };
    let na_lista = c.aba == Aba::Construtor && lido.is_ok();
    let mut topo = vec![Span::raw(" "), aba("Construtor", na_lista, lido.is_ok()), Span::raw(" "), aba("Código", !na_lista, true)];
    if let Some(a) = &c.aviso {
        topo.push(Span::styled(format!("  ⚠ {a}"), Style::default().fg(t.var("warning"))));
    }
    f.render_widget(Paragraph::new(Line::from(topo)), Rect { height: 1, ..dentro });

    let corpo = Rect { y: dentro.y + 2, height: dentro.height - 2, ..dentro };
    // Com o formulário aberto a edição ganha largura: as opções de um
    // campo vão todas numa linha (as oito setas da sequência), e a
    // escolhida não pode ficar fora da caixa.
    let fracao = if c.form.is_some() { 3 } else { 2 };
    let esquerda_w = (corpo.width * fracao / 5).max(30).min(corpo.width.saturating_sub(20));
    let esquerda = Rect { width: esquerda_w, ..corpo };
    let direita = Rect { x: corpo.x + esquerda_w + 1, width: corpo.width.saturating_sub(esquerda_w + 1), ..corpo };

    desenhar_edicao(f, c, &lido, esquerda, t);
    desenhar_previa(f, c, direita, t);

    if c.confirmar {
        let caixa = componentes::area_do_modal(tela, 50, 5);
        let d = componentes::desenhar_modal(f, caixa, "Sair do construtor", "y descarta · n volta", t);
        f.render_widget(
            Paragraph::new(vec![
                Line::from(Span::styled("O diagrama mudou. Descartar as mudanças?", normal)),
                Line::default(),
                Line::from(vec![
                    Span::styled(" y ", Style::default().bg(t.var("error")).fg(t.var("bg-base")).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(" n ", Style::default().bg(t.var("bg-elevated")).fg(t.var("text-primary"))),
                ]),
            ]),
            d,
        );
    }
}

fn desenhar_edicao(f: &mut Frame, c: &Construtor, lido: &Result<Diagrama, diagrama::ForaDoModelo>, area: Rect, t: &Tema) {
    let apagado = Style::default().fg(t.var("text-muted"));
    let normal = Style::default().fg(t.var("text-primary"));
    let grupo = Style::default().fg(t.var("accent-blue")).add_modifier(Modifier::BOLD);
    let largura = area.width as usize;
    let cortar = |s: String| crate::codigo::cortar(&s, largura);

    if let Some(i) = c.tipos {
        let mut linhas = vec![Line::from(Span::styled("Que diagrama?", grupo)), Line::default()];
        for (k, tipo) in Tipo::TODOS.iter().enumerate() {
            let estilo = if k == i { t.estilo(Realce::Cursor) } else { normal };
            linhas.push(Line::from(Span::styled(cortar(format!("  {}  ", tipo.nome())), estilo)));
        }
        linhas.push(Line::default());
        linhas.push(Line::from(Span::styled("A prévia mostra o exemplo de cada um.", apagado)));
        f.render_widget(Paragraph::new(linhas), area);
        return;
    }

    if c.aba == Aba::Codigo || lido.is_err() {
        let mut topo = 0u16;
        if let Err(fora) = lido {
            let frase = format!("O construtor não monta este diagrama — {}. Edite pelo código.", fora.mensagem());
            // A altura é a da frase (até 4 linhas): o motivo cortado no
            // meio não diz o que tirar do código.
            let alto = (unicode_width::UnicodeWidthStr::width(frase.as_str()).div_ceil(largura.max(1)) as u16 + 1).min(5).min(area.height);
            let aviso = Paragraph::new(Line::from(Span::styled(frase, Style::default().fg(t.var("warning"))))).wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(aviso, Rect { height: alto, ..area });
            topo = alto;
        }
        let resto = Rect { y: area.y + topo, height: area.height.saturating_sub(topo), ..area };
        c.codigo.desenhar(f, resto, t);
        return;
    }
    let Ok(d) = lido else { return };

    if let Some((form, l)) = &c.form {
        let titulo = formulario(c, d, *l).map(|(_, t)| t).unwrap_or_default();
        let mut linhas = vec![Line::from(Span::styled(cortar(titulo), grupo))];
        linhas.extend(form.linhas(largura.saturating_sub(2), t));
        let cursor = form.linha_do_cursor() + 1;
        let rolagem = cursor.saturating_sub(area.height.saturating_sub(2) as usize);
        // Quebra em vez de cortar: o que passa da largura é opção, e cortada
        // ela some sem aviso.
        f.render_widget(Paragraph::new(linhas).wrap(ratatui::widgets::Wrap { trim: false }).scroll((rolagem as u16, 0)), area);
        return;
    }

    let linhas_do_modelo = linhas_de(d, c.e_novo());
    let mut linhas = Vec::new();
    for (i, l) in linhas_do_modelo.iter().enumerate() {
        let texto = cortar(format!("{}{}", recuo(*l), texto_da_linha(d, *l)));
        let estilo = match l {
            Linha::Rotulo(_) => grupo,
            _ if i == c.sel && c.foco == Foco::Edicao => t.estilo(Realce::Cursor),
            Linha::NovoNo
            | Linha::NovaLigacao
            | Linha::NovoParticipante
            | Linha::NovaMensagem
            | Linha::NovaNota
            | Linha::NovoEstado
            | Linha::NovaTransicao
            | Linha::NovaTarefa(_)
            | Linha::NovaSecao => apagado,
            _ if i == c.sel => normal.add_modifier(Modifier::UNDERLINED),
            _ => normal,
        };
        // Um respiro antes de cada grupo, menos o primeiro.
        if matches!(l, Linha::Rotulo(_) | Linha::Secao(_)) && i > 0 {
            linhas.push(Line::default());
        }
        linhas.push(Line::from(Span::styled(texto, estilo)));
    }
    // A linha do cursor na tela conta os respiros.
    let antes = linhas_do_modelo.iter().take(c.sel).enumerate().filter(|(i, l)| *i > 0 && matches!(l, Linha::Rotulo(_) | Linha::Secao(_))).count();
    let na_tela = c.sel + antes;
    let rolagem = na_tela.saturating_sub(area.height.saturating_sub(1) as usize);
    f.render_widget(Paragraph::new(linhas).scroll((rolagem as u16, 0)), area);
}

fn desenhar_previa(f: &mut Frame, c: &Construtor, area: Rect, t: &Tema) {
    if area.width < 4 || area.height < 3 {
        return;
    }
    let focada = c.foco == Foco::Previa;
    let borda = if focada { t.estilo(Realce::BordaFoco) } else { Style::default().fg(t.var("border")) };
    let cabe = area.width.saturating_sub(2) as usize;
    let alto_total = area.height.saturating_sub(2) as usize;
    let (conteudo, colunas, aviso) = crate::codigo::previa_mermaid(c.codigo.texto(), cabe, t);
    let alto = alto_total.saturating_sub(usize::from(aviso.is_some())).max(1);
    let medida = Medida { linhas: conteudo.len(), colunas, alto, largo: cabe };
    c.medida.set(medida);
    let y = c.vista.y.min(medida.max_y());
    let x = c.vista.x.min(medida.max_x());
    // Onde se está vai no título, curto, como na vista do bloco (498).
    let mut rotulo = String::from(" Prévia");
    if conteudo.len() > alto {
        rotulo.push_str(&format!(" · ↕ {}–{}/{}", y + 1, (y + alto).min(conteudo.len()), conteudo.len()));
    }
    if colunas > cabe {
        rotulo.push_str(&format!(" · ↔ {}–{}/{colunas}", x + 1, (x + cabe).min(colunas)));
    }
    if !focada && (conteudo.len() > alto || colunas > cabe) {
        rotulo.push_str(" · p rola");
    }
    rotulo.push(' ');
    let bloco = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(borda).title(Span::styled(rotulo, borda));
    let miolo = bloco.inner(area);
    f.render_widget(bloco, area);
    let mut linhas: Vec<Line<'static>> = Vec::new();
    if let Some(a) = aviso {
        linhas.push(Line::from(Span::styled(crate::codigo::cortar(&format!("⚠ {a}"), cabe), Style::default().fg(t.var("warning")))));
    }
    linhas.extend(conteudo.iter().skip(y).take(alto).map(|l| Line::from(crate::codigo::recortar(l, x, cabe))));
    f.render_widget(Paragraph::new(linhas), miolo);
}

#[cfg(test)]
mod testes;
