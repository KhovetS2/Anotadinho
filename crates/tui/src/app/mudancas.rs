//! O que a tarefa mudou no repositório (ciclo 519), em tela inteira: a
//! base, o branch, a lista dos arquivos e o diff unificado com cor.
//!
//! É a revisão do CÓDIGO. Antes, "revisão" na TUI era ler a conversa e
//! acreditar no agente; o que ele mexeu no repositório só se via saindo
//! pro terminal e rodando `git diff` contra um commit que a pessoa nem
//! sabia qual era. A base é a que o 517 guarda no frontmatter da tarefa.
//!
//! O IO é do `main` (`Pedido::MudancasDaTarefa`, que chama o
//! `handle_mudancas_da_tarefa`); aqui só se monta, desenha e rola. As
//! linhas são classificadas UMA vez, na abertura: o diff pode ter 400 KB
//! (o teto do 517), e reclassificar a cada `j` faria a rolagem pesar.

use std::cell::Cell;

use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::Estado;
use crate::componentes;
use crate::tema::{misturar, Realce};

/// O papel de uma linha da vista, que decide a cor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    /// "3 arquivos mudados".
    Titulo,
    /// Um arquivo da lista do topo, com o estado.
    Lista,
    /// O começo de um arquivo no diff (`diff --git`).
    Arquivo,
    /// `index`, `---`, `+++`, `new file`: o cabeçalho do arquivo.
    Meta,
    /// `@@ -1,3 +1,4 @@`.
    Trecho,
    Entra,
    Sai,
    Igual,
    Vazio,
}

/// A vista aberta.
#[derive(Debug, Clone, PartialEq)]
pub struct VistaDasMudancas {
    pub titulo: String,
    /// O commit de onde a tarefa partiu, inteiro.
    pub base: Option<String>,
    pub branch: Option<String>,
    pub aviso: Option<String>,
    pub linhas: Vec<(Tipo, String)>,
    /// O índice em `linhas` de cada `diff --git`, pro `n`/`N`.
    pub arquivos: Vec<usize>,
    pub rolagem: usize,
    /// Quantas linhas do corpo couberam no último desenho: o `Ctrl+D` anda
    /// meia tela, e o `G` para com a última linha no pé, não no topo.
    pub altura: Cell<usize>,
}

/// O estado do `git` por extenso: "M" sozinho não diz nada a quem não
/// decorou o `--name-status`.
fn estado_por_extenso(estado: &str) -> &'static str {
    match estado {
        "M" => "mudou",
        "A" => "novo",
        "D" => "apagado",
        "R" => "renomeado",
        "C" => "copiado",
        "T" => "tipo mudou",
        "?" => "novo, fora do git",
        _ => "mudou",
    }
}

impl VistaDasMudancas {
    /// Monta a vista das mudanças `m` da tarefa `titulo`.
    pub fn nova(titulo: &str, m: &anotadinho_ipc::repositorio::Mudancas) -> Self {
        let mut linhas: Vec<(Tipo, String)> = Vec::new();
        let n = m.arquivos.len();
        if n == 0 && m.diff.trim().is_empty() {
            linhas.push((Tipo::Titulo, "Nada mudou no repositório desde a base.".into()));
        } else {
            linhas.push((Tipo::Titulo, if n == 1 { "1 arquivo mudado".into() } else { format!("{n} arquivos mudados") }));
            for a in &m.arquivos {
                linhas.push((Tipo::Lista, format!("{:<2} {}  ({})", a.estado, a.caminho, estado_por_extenso(&a.estado))));
            }
            linhas.push((Tipo::Vazio, String::new()));
        }
        let mut arquivos = Vec::new();
        // Dentro de um trecho, `--- x` é uma linha REMOVIDA que começava
        // com `-- `, não o cabeçalho do arquivo. Só antes do primeiro `@@`
        // de cada arquivo é que `---`/`+++` são cabeçalho.
        let mut no_trecho = false;
        for l in m.diff.lines() {
            let l = l.replace('\t', "    ");
            let tipo = if let Some(resto) = l.strip_prefix("diff --git ") {
                no_trecho = false;
                arquivos.push(linhas.len());
                // `a/x b/x`: o caminho de destino é o que se lê.
                let caminho = resto.rsplit_once(" b/").map(|(_, b)| b.to_string()).unwrap_or_else(|| resto.to_string());
                linhas.push((Tipo::Arquivo, format!("▸ {caminho}")));
                continue;
            } else if l.starts_with("@@") {
                no_trecho = true;
                Tipo::Trecho
            } else if !no_trecho {
                Tipo::Meta
            } else if l.starts_with('+') {
                Tipo::Entra
            } else if l.starts_with('-') {
                Tipo::Sai
            } else if l.starts_with('\\') {
                // "\ No newline at end of file"
                Tipo::Meta
            } else {
                Tipo::Igual
            };
            linhas.push((tipo, l));
        }
        Self {
            titulo: titulo.to_string(),
            base: m.base.clone(),
            branch: m.branch.clone(),
            aviso: m.aviso.clone(),
            linhas,
            arquivos,
            rolagem: 0,
            altura: Cell::new(0),
        }
    }

    /// A maior rolagem: a última linha no pé da tela.
    fn maximo(&self) -> usize {
        self.linhas.len().saturating_sub(self.altura.get().max(1))
    }
}

/// Abre a vista sobre o que estiver na tela.
pub fn abrir(e: &mut Estado, titulo: &str, m: &anotadinho_ipc::repositorio::Mudancas) {
    e.modal = Some(super::modais::Modal::Mudancas(VistaDasMudancas::nova(titulo, m)));
}

/// Uma tecla na vista.
pub fn tecla(e: &mut Estado, mut v: VistaDasMudancas, tecla: &str) {
    let meia = (v.altura.get() / 2).max(1);
    v.rolagem = match tecla {
        "Escape" | "q" => return,
        "j" | "ArrowDown" => v.rolagem + 1,
        "k" | "ArrowUp" => v.rolagem.saturating_sub(1),
        "Ctrl+d" | "PageDown" | " " => v.rolagem + meia,
        "Ctrl+u" | "PageUp" => v.rolagem.saturating_sub(meia),
        "g" | "Home" => 0,
        "G" | "End" => usize::MAX,
        // O arquivo seguinte é o primeiro que começa DEPOIS do topo; o
        // anterior, o último que começa antes — e não "o índice atual ± 1",
        // que se perderia assim que a pessoa rolasse com `j`.
        "n" => v.arquivos.iter().copied().find(|&i| i > v.rolagem).unwrap_or(v.rolagem),
        "N" => v.arquivos.iter().copied().rev().find(|&i| i < v.rolagem).unwrap_or(0),
        _ => v.rolagem,
    }
    .min(v.maximo());
    e.modal = Some(super::modais::Modal::Mudancas(v));
}

/// Desenha a vista na tela inteira.
pub fn desenhar(f: &mut Frame, e: &Estado, v: &VistaDasMudancas) {
    let t = &e.tema;
    let tela = f.area();
    let area = componentes::area_do_modal(tela, tela.width, tela.height);
    let dica = super::especiais::dica_que_cabe(&["j k", "Ctrl+D/U meia tela", "g G", "n N arquivo", "Esc fecha"], area.width.saturating_sub(4) as usize);
    let dentro = componentes::desenhar_modal(f, area, &v.titulo, &dica, t);
    let w = dentro.width as usize;
    if w < 10 || dentro.height < 3 {
        return;
    }
    let apagado = Style::default().fg(t.var("text-muted"));
    let texto = Style::default().fg(t.var("text-primary"));
    let destaque = Style::default().fg(t.var("accent-blue"));

    // O topo FIXO: de onde se compara e em que branch. Rolando o diff, a
    // pessoa não perde de vista contra o que está olhando.
    let mut topo: Vec<Line<'static>> = Vec::new();
    let mut l1 = vec![Span::styled("base ", apagado)];
    match &v.base {
        Some(b) => l1.push(Span::styled(b.chars().take(7).collect::<String>(), texto.add_modifier(Modifier::BOLD))),
        None => l1.push(Span::styled("HEAD", texto)),
    }
    if let Some(b) = &v.branch {
        l1.push(Span::styled(" · branch ", apagado));
        l1.push(Span::styled(b.clone(), destaque));
    }
    // Em qual arquivo o topo está, depois que a lista do começo saiu da
    // tela: "arquivo 0 de 3" na abertura lia como erro.
    let atual = v.arquivos.iter().filter(|&&i| i <= v.rolagem).count();
    if atual > 0 {
        l1.push(Span::styled(format!(" · arquivo {atual} de {}", v.arquivos.len()), apagado));
    }
    topo.push(super::especiais::caber(l1, w));
    if let Some(a) = &v.aviso {
        for l in super::especiais::quebrar(&format!("⚠ {a}"), w, "  ") {
            topo.push(Line::from(Span::styled(l, t.estilo(Realce::BadgeAtencao))));
        }
    }
    topo.push(Line::from(Span::styled("─".repeat(w), Style::default().fg(t.var("border")))));

    let altura = (dentro.height as usize).saturating_sub(topo.len());
    v.altura.set(altura);
    let superficie = t.var("bg-surface");
    let entra = Style::default().fg(t.var("success")).bg(misturar(t.var("success"), superficie, 0.16));
    let sai = Style::default().fg(t.var("error")).bg(misturar(t.var("error"), superficie, 0.16));
    let rolagem = v.rolagem.min(v.linhas.len().saturating_sub(altura.max(1)));
    let corpo = v.linhas.iter().skip(rolagem).take(altura).map(|(tipo, s)| {
        let cortado = crate::codigo::cortar(s, w);
        // As de + e − pintam a linha INTEIRA: a faixa de cor é o que o
        // olho segue descendo, e um fundo que acaba no fim do texto some
        // nas linhas curtas.
        let faixa = |estilo: Style| {
            let n = unicode_width::UnicodeWidthStr::width(cortado.as_str());
            Line::from(Span::styled(format!("{cortado}{}", " ".repeat(w.saturating_sub(n))), estilo))
        };
        match tipo {
            Tipo::Titulo => Line::from(Span::styled(cortado, texto.add_modifier(Modifier::BOLD))),
            Tipo::Lista => {
                let (estado, resto) = s.split_once(' ').unwrap_or((s.as_str(), ""));
                let cor = match estado {
                    "A" | "?" => t.var("success"),
                    "D" => t.var("error"),
                    _ => t.var("warning"),
                };
                Line::from(vec![
                    Span::styled(format!("  {estado}"), Style::default().fg(cor).add_modifier(Modifier::BOLD)),
                    Span::styled(crate::codigo::cortar(&format!(" {resto}"), w.saturating_sub(2 + estado.chars().count())), texto),
                ])
            }
            Tipo::Arquivo => faixa(texto.bg(t.var("bg-elevated")).add_modifier(Modifier::BOLD)),
            Tipo::Meta => Line::from(Span::styled(cortado, apagado)),
            Tipo::Trecho => Line::from(Span::styled(cortado, Style::default().fg(t.var("accent-purple")))),
            Tipo::Entra => faixa(entra),
            Tipo::Sai => faixa(sai),
            Tipo::Igual => Line::from(Span::styled(cortado, texto)),
            Tipo::Vazio => Line::default(),
        }
    });
    let linhas: Vec<Line<'static>> = topo.into_iter().chain(corpo).collect();
    f.render_widget(Paragraph::new(linhas), dentro);
}
