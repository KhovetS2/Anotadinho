//! Testes de TELA da TUI (ciclo 327): o quadro desenhado de verdade, com
//! texto E cor, comparado com uma foto guardada.
//!
//! Os testes de unidade de `app.rs` conferem um pedaço de cada vez ("a
//! linha tem `2 imagens`"). Isto aqui pega o que eles deixam passar: um
//! embed que mudou de cor, uma borda que andou uma coluna, a galeria que
//! voltou a empilhar — a mesma classe de regressão que o
//! `scripts/uitest/snapshot.mjs` pega na janela.
//!
//! Cada CENA é uma página, uma sequência de teclas e um tamanho de tela.
//! A foto (`tests/telas/<cena>.tela`) guarda:
//!
//! - o texto da tela;
//! - uma grade de letras do mesmo tamanho, uma letra por ESTILO (cor de
//!   texto, fundo, negrito…), com a legenda embaixo — um diff mostra onde
//!   a cor mudou sem precisar ler código de cor;
//! - o arquivo como ficou, quando as teclas editaram a página.
//!
//! Uso:
//!
//! ```text
//! cargo test -p anotadinho-tui --test telas                  # confere
//! ATUALIZAR_TELAS=1 cargo test -p anotadinho-tui --test telas  # regrava
//! TELAS_ANSI=/tmp/x cargo test -p anotadinho-tui --test telas  # + ANSI
//! ```
//!
//! Quando uma foto não bate, a nova vai pra `<cena>.tela.nova` do lado,
//! pra comparar com `diff`.

use std::collections::HashMap;
use std::path::PathBuf;

use anotadinho_core::analise::analisar;
use anotadinho_core::embed::{segment_com_intervalos, DocSegment};
use anotadinho_ipc::PageMeta;
use anotadinho_tui::app::{self, especiais, Estado, Foco};
use ratatui::backend::TestBackend;
use ratatui::style::{Color, Modifier, Style};
use ratatui::Terminal;

/// A página de exemplos do vault, com um embed de cada tipo.
const EXEMPLOS: &str = include_str!("telas/paginas/embeds.md");

/// Uma conversa de verdade do vault (ciclo 340).
const CONVERSA: &str = include_str!("telas/paginas/conversa.md");

/// Uma cena: o que abrir, o que teclar, de que tamanho é a tela.
struct Cena {
    nome: &'static str,
    pagina: String,
    teclas: Vec<&'static str>,
    largura: u16,
    altura: u16,
    /// Com o índice falso do vault (as consultas precisam dele).
    com_indice: bool,
}

/// Um vault pequeno e fixo pras consultas: não pode ser o vault de
/// verdade, que muda e mudaria a foto.
fn indice_falso() -> Vec<anotadinho_core::index::PageIndexEntry> {
    let pagina = |path: &str, title: &str, tipo: &str, status: &str| {
        let mut properties = std::collections::BTreeMap::new();
        properties.insert("type".to_string(), tipo.to_string());
        if !status.is_empty() {
            properties.insert("status".to_string(), status.to_string());
        }
        anotadinho_core::index::PageIndexEntry {
            path: path.into(),
            title: title.into(),
            section: "pages".into(),
            page_type: tipo.into(),
            properties,
            ..Default::default()
        }
    };
    let mut v = vec![
        pagina("pages/ciclos/001-bootstrap.md", "Ciclo 001 — Bootstrap do projeto", "ciclo", "done"),
        pagina("pages/ciclos/002-vault-picker.md", "Ciclo 002 — Vault picker", "ciclo", "done"),
        pagina("pages/specs/editor.md", "Spec do editor", "spec", "em-revisao"),
        pagina("pages/decisoes/yew.md", "Decisão: Yew no front", "decisao", ""),
        pagina("pages/specs/sync.md", "Spec de sincronização", "spec", "rascunho"),
        pagina("journals/2026-08-12.md", "12 de agosto", "", ""),
    ];
    // Páginas com data, pros cronogramas e calendários em modo vault.
    let mut com_data = |path: &str, title: &str, start: &str, end: &str| {
        let mut p = pagina(path, title, "", "");
        p.properties.insert("start".into(), start.into());
        if !end.is_empty() {
            p.properties.insert("end".into(), end.into());
        }
        v.push(p);
    };
    com_data("journals/planejamento.md", "Planejamento", "2026-07-28", "2026-08-07");
    com_data("journals/lancamento.md", "Lançamento", "2026-08-20", "");
    v
}

/// Só o `n`-ésimo embed do tipo `tipo` dos exemplos, entre dois
/// parágrafos — a caixa sozinha, sem o resto da página.
fn so_o_embed(tipo: &str) -> String {
    let (faixa, _) = segment_com_intervalos(EXEMPLOS)
        .into_iter()
        .find(|(_, s)| matches!(s, DocSegment::Embed(d) if d.kind().type_name() == tipo))
        .unwrap_or_else(|| panic!("os exemplos não têm {tipo}"));
    format!("Antes.\n\n{}\n\nDepois.\n", EXEMPLOS[faixa].trim_end())
}

fn cena(nome: &'static str, pagina: String, teclas: &[&'static str], largura: u16, altura: u16) -> Cena {
    Cena { nome, pagina, teclas: teclas.to_vec(), largura, altura, com_indice: true }
}

/// As teclas de um texto digitado, uma por caractere.
fn digitado(texto: &str) -> Vec<&'static str> {
    texto.chars().map(|c| &*Box::leak(c.to_string().into_boxed_str())).collect()
}

fn com(teclas: &[&[&'static str]]) -> Vec<&'static str> {
    teclas.concat()
}

fn cenas() -> Vec<Cena> {
    let mut v = Vec::new();
    // Um embed de cada tipo, parado, com o foco na lista de páginas (o
    // cursor não acende nada) — é o desenho "de repouso".
    for tipo in ["kanban", "calendar", "table", "callout", "columns", "gallery", "query", "fluxo", "timeline", "actions"] {
        let nome: &'static str = Box::leak(format!("repouso-{tipo}").into_boxed_str());
        v.push(cena(nome, so_o_embed(tipo), &[], 140, 40));
    }
    // Markdown (ciclos 333 e 334): a inserção aparece no lugar do bloco,
    // e o arquivo gravado vai junto na foto.
    let notas = "---\ntitle: Notas\n---\n# Título\n\nUm parágrafo com **negrito**.\n\n- [ ] tarefa um\n- [ ] tarefa dois\n\n> citação\n".to_string();
    v.push(Cena {
        nome: "markdown-insercao-no-lugar",
        pagina: notas.clone(),
        teclas: com(&[&["Tab", "j", "A"], &digitado(" E mais")]),
        largura: 120,
        altura: 20,
        com_indice: false,
    });
    v.push(Cena {
        nome: "markdown-enter-abre-item-seguinte",
        pagina: notas.clone(),
        teclas: com(&[&["Tab", "j", "j", "Enter", "j", "o"], &digitado("tarefa três"), &["Enter"], &digitado("quatro")]),
        largura: 120,
        altura: 20,
        com_indice: false,
    });
    v.push(Cena {
        nome: "markdown-til-e-maior-maior",
        pagina: notas,
        teclas: vec!["Tab", "j", "j", "Enter", "~", ">", ">", "Escape", "k", "k", "Ctrl+x"],
        largura: 120,
        altura: 20,
        com_indice: false,
    });
    // Edição dos embeds que faltavam (ciclo 335), com o arquivo na foto.
    v.push(cena("galeria-legenda-e-colunas", so_o_embed("gallery"), &com(&[&["Tab", "j", "Enter", "j", "Enter", "a"], &digitado(" nova"), &["Escape", "Ctrl+a"]]), 140, 40));
    v.push(cena("paineis-alarga-e-cria", so_o_embed("columns"), &["Tab", "j", "Enter", "Ctrl+a", "o"], 140, 40));
    v.push(cena("fluxo-avanco-natural", so_o_embed("fluxo"), &["Tab", "j", "Enter", ">", ">"], 140, 40));
    v.push(cena("consulta-filtro-e-visao", so_o_embed("query"), &com(&[&["Tab", "j", "A"], &digitado(" status=done"), &["Escape", "~"]]), 140, 40));
    // Colunas do kanban e da tabela (ciclo 337).
    v.push(cena("kanban-coluna-nova-e-movida", so_o_embed("kanban"), &com(&[&["Tab", "j", "Enter", "o"], &digitado("Revisão"), &["Escape", "<", "<"]]), 140, 40));
    v.push(cena("tabela-coluna-nova-e-tipo", so_o_embed("table"), &com(&[&["Tab", "j", "Enter", "Enter", "o"], &digitado("Dono"), &["Escape", "~"]]), 160, 30));
    // Componentes (ciclo 339): a barra de comandos, filtrada, e o editor
    // de opções de uma coluna de seleção.
    v.push(cena("paleta-aberta", so_o_embed("callout"), &[":"], 140, 36));
    v.push(cena("paleta-filtrada", so_o_embed("callout"), &com(&[&[":"], &digitado("tema")]), 140, 36));
    v.push(cena("opcoes-da-selecao", so_o_embed("table"), &["Tab", "j", "Enter", "Enter", "l", "Enter"], 160, 36));
    v.push(cena("atalhos", so_o_embed("callout"), &["?"], 140, 40));
    // A tela de conversa (ciclo 340): no fim, numa resposta, e escrevendo.
    // Tags, assets e propostas (ciclo 346).
    let especial = |tipo: &str| format!("---\ntitle: X\ntype: {tipo}\n---\n");
    v.push(cena("tags", especial("tags"), &["Tab", "j", "l"], 140, 36));
    v.push(cena("assets", especial("assets"), &["Tab", "j"], 140, 20));
    v.push(cena("pagina-kanban", "---\ntitle: Quadro\ntype: kanban\n---\n\n- Escrever o texto  column:: todo\n- title:: Revisar  column:: doing\n- Publicar  column:: done\n- Ideia solta\n".to_string(), &["Tab", "l", "l"], 140, 20));
    v.push(cena("pagina-grafo", especial("graph"), &["Tab", "l", "l"], 140, 30));
    v.push(cena("pagina-tarefas", especial("table"), &["Tab", "j"], 140, 16));
    v.push(cena("pagina-calendario", especial("calendar"), &[], 140, 36));
    v.push(cena("propostas-diff", especial("propostas"), &["Tab"], 140, 40));
    v.push(cena("propostas-visualizacao", especial("propostas"), &["Tab", "v"], 140, 40));
    // O menu `/` (ciclo 347).
    let notas = "# Notas\n\nUm parágrafo.\n\n- item\n".to_string();
    v.push(cena("lista-aninhada", "# Notas\n\n- primeiro\n  - dentro um\n  - dentro dois\n    - mais fundo\n- segundo\n".to_string(), &["Tab", "j", "Enter", "Enter", "j"], 120, 16));
    v.push(cena("menu-inserir", notas.clone(), &["Tab", "o", "/"], 140, 40));
    v.push(cena("wikilink-sugestoes", notas.clone(), &com(&[&["Tab", "o"], &digitado("ver [[ce")]), 140, 30));
    v.push(cena("menu-inserir-filtrado", notas, &com(&[&["Tab", "o", "/"], &digitado("tab")]), 140, 40));
    v.push(cena("conversa-no-fim", CONVERSA.to_string(), &["Tab"], 160, 50));
    v.push(cena("conversa-resposta-selecionada", CONVERSA.to_string(), &["Tab", "k"], 160, 50));
    v.push(cena("conversa-escrevendo", CONVERSA.to_string(), &com(&[&["Tab", "i"], &digitado("Detalha a etapa 2")]), 160, 50));
    // Detalhes num formulário (ciclo 343).
    v.push(cena("cartao-seletor-de-data", so_o_embed("kanban"), &["Tab", "j", "Enter", "Enter", "Enter", "j", "j", "j", "j", "j", "Enter", "l"], 140, 40));
    v.push(cena("kanban-detalhe-do-cartao", so_o_embed("kanban"), &["Tab", "j", "Enter", "Enter", "Enter", "j", "j", "j", "j"], 140, 40));
    // O cronograma na janela da tela: manual, escala Mês, e as teclas.
    let manual = so_o_embed("timeline").replace("source: vault\n", "").replace("scale: quarter\n", "scale: month\n");
    v.push(cena("cronograma-manual-mes", manual.clone(), &[], 140, 30));
    v.push(cena("cronograma-proximo-periodo", manual.clone(), &["Tab", "j", "Enter", "]"], 140, 30));
    v.push(cena("cronograma-m-troca-a-escala", manual.clone(), &["Tab", "j", "Enter", "m"], 140, 30));
    v.push(cena("cronograma-barra-acesa", manual, &["Tab", "j", "Enter", "j"], 140, 30));
    // A consulta nas três visões, e agrupada.
    let consulta = |yaml: &str| format!("Antes.\n\n{{{{ type: \"query\" }}}}\n{yaml}{{{{ /query }}}}\n\nDepois.\n");
    v.push(cena("consulta-lista", consulta("from: pages\nwhere:\n- field: type\n  op: exists\ncolumns:\n- type\n- status\n"), &[], 140, 40));
    v.push(cena("consulta-cartoes", consulta("from: pages\nview: cards\ncolumns:\n- status\n"), &[], 140, 40));
    v.push(cena("consulta-agrupada", consulta("from: pages\ngroup_by: type\naggregate:\n- op: count\n"), &[], 140, 40));
    v.push(cena("consulta-vazia", consulta("from: nada\n"), &[], 140, 20));
    v.push(cena("consulta-linha-acesa", so_o_embed("query"), &["Tab", "j", "Enter", "j"], 140, 40));
    // O cursor dentro dos embeds: o que acende, e a navegação lado a lado.
    v.push(cena("kanban-cartao-aceso", so_o_embed("kanban"), &["Tab", "j", "Enter", "Enter"], 140, 40));
    v.push(cena("kanban-l-vai-pra-coluna-vizinha", so_o_embed("kanban"), &["Tab", "j", "Enter", "l", "l", "Enter"], 140, 40));
    v.push(cena("galeria-miniatura-acesa", so_o_embed("gallery"), &["Tab", "j", "Enter", "j", "Enter", "l"], 140, 40));
    v.push(cena("colunas-painel-aceso", so_o_embed("columns"), &["Tab", "j", "Enter", "l"], 140, 40));
    v
}

/// O que o `main` leria do vault pras telas de tags, assets e propostas
/// (ciclo 346), fixo.
fn dados_falsos(tipo: especiais::TipoEspecial) -> especiais::Dados {
    use anotadinho_core::proposta::{Operacao, Proposta};
    match tipo {
        especiais::TipoEspecial::Tags => {
            let pagina = |path: &str, title: &str, tags: &[&str]| anotadinho_core::index::PageIndexEntry {
                path: path.into(),
                title: title.into(),
                embed_tags: tags.iter().map(|t| t.to_string()).collect(),
                ..Default::default()
            };
            especiais::tags_do_indice(&[
                pagina("pages/sprint.md", "Sprint 12", &["infra", "urgente", "backend"]),
                pagina("pages/roadmap.md", "Roadmap", &["infra", "produto"]),
                pagina("pages/lancamento.md", "Lançamento da versão 2", &["produto", "urgente"]),
                pagina("pages/deploy.md", "Deploy", &["infra"]),
            ])
        }
        especiais::TipoEspecial::Assets => especiais::assets_com_uso(
            vec![
                ("assets/diagrama-arquitetura.png".into(), 184_320),
                ("assets/capa.jpg".into(), 2_450_000),
                ("assets/rascunho-antigo.pdf".into(), 912),
            ],
            "![](../assets/diagrama-arquitetura.png) capa.jpg",
        ),
        especiais::TipoEspecial::Kanban => unreachable!("o kanban lê a própria página"),
        especiais::TipoEspecial::Grafo => {
            let pagina = |path: &str, title: &str, links: &[&str]| anotadinho_core::index::PageIndexEntry {
                path: path.into(),
                title: title.into(),
                wikilinks: links.iter().map(|t| t.to_string()).collect(),
                ..Default::default()
            };
            especiais::grafo_do_indice(&[
                pagina("pages/roadmap.md", "Roadmap", &["Sprint 12", "Lançamento"]),
                pagina("pages/sprint.md", "Sprint 12", &["Deploy"]),
                pagina("pages/lancamento.md", "Lançamento", &[]),
                pagina("pages/deploy.md", "Deploy", &[]),
                pagina("pages/solta.md", "Nota solta", &["Inexistente"]),
            ])
        }
        especiais::TipoEspecial::Tarefas => especiais::tarefas_das_paginas(vec![
            ("pages/deploy.md".into(), "Deploy".into(), "status:: doing\npriority:: alta".into()),
            ("pages/docs.md".into(), "Documentação".into(), "status:: todo".into()),
            ("pages/login.md".into(), "Login".into(), "status:: done\npriority:: media".into()),
            ("pages/nota.md".into(), "Nota solta".into(), "sem nada".into()),
        ]),
        especiais::TipoEspecial::Propostas => especiais::Dados::Propostas(vec![
            especiais::PropostaNaTela {
                proposta: Proposta {
                    id: "p1".into(),
                    autor: "claude".into(),
                    quando: "2026-08-12 14:05".into(),
                    motivo: "Separar as tarefas pendentes das concluídas e marcar a revisão como feita.".into(),
                    alvo: "pages/sprint.md".into(),
                    operacao: Operacao::Substituir,
                    conteudo: "# Sprint 12\n\n## Pendentes\n\n- [ ] migrar o banco\n\n## Feitas\n\n- [x] revisão do PR\n".into(),
                },
                atual: "# Sprint 12\n\n- [ ] migrar o banco\n- [ ] revisão do PR\n".into(),
            },
            especiais::PropostaNaTela {
                proposta: Proposta {
                    id: "p2".into(),
                    autor: "claude".into(),
                    quando: "2026-08-12 13:40".into(),
                    motivo: String::new(),
                    alvo: "pages/notas/ideias.md".into(),
                    operacao: Operacao::Criar,
                    conteudo: "# Ideias\n\n- atalho pra duplicar cartão\n".into(),
                },
                atual: String::new(),
            },
        ]),
    }
}

/// O que a cena produziu: a tela e o arquivo.
fn rodar(c: &Cena) -> (ratatui::buffer::Buffer, Option<String>) {
    let paginas = vec![PageMeta { path: "pages/cena.md".into(), title: "cena".into(), section: "pages".into() }];
    let mut e = Estado::novo(paginas, analisar(""));
    e.abrir_texto(&c.pagina, Some("v1".into()));
    if let Some(tipo) = e.especial.as_ref().filter(|t| t.dados.is_none()).map(|t| t.tipo) {
        especiais::carregar(&mut e, dados_falsos(tipo));
    }
    let mut e = e.com_hoje("2026-08-12");
    if c.com_indice {
        e = e.com_indice_do_vault(indice_falso());
    }
    let mut arquivo: Option<String> = None;
    for t in &c.teclas {
        app::tecla(&mut e, t);
        // O que o `main` faz depois de cada tecla: gravar o que ficou.
        if let Some(novo) = e.gravacao.take() {
            arquivo = Some(novo);
        }
    }
    if c.teclas.is_empty() {
        e.foco = Foco::Paginas;
    }
    let mut term = Terminal::new(TestBackend::new(c.largura, c.altura)).unwrap();
    term.draw(|f| app::desenhar(f, &mut e)).unwrap();
    (term.backend().buffer().clone(), arquivo)
}

fn cor(c: Option<Color>) -> String {
    match c {
        None | Some(Color::Reset) => "-".into(),
        Some(Color::Rgb(r, g, b)) => format!("#{r:02x}{g:02x}{b:02x}"),
        Some(outra) => format!("{outra:?}"),
    }
}

fn nome_do_estilo(s: Style) -> String {
    let mut n = format!("fg {} bg {}", cor(s.fg), cor(s.bg));
    for (m, rotulo) in [
        (Modifier::BOLD, "negrito"),
        (Modifier::ITALIC, "itálico"),
        (Modifier::UNDERLINED, "sublinhado"),
        (Modifier::DIM, "apagado"),
        (Modifier::REVERSED, "invertido"),
    ] {
        if s.add_modifier.contains(m) {
            n.push(' ');
            n.push_str(rotulo);
        }
    }
    n
}

/// A foto em texto: tela, grade de estilos com legenda, arquivo.
fn foto(c: &Cena, buf: &ratatui::buffer::Buffer, arquivo: &Option<String>) -> String {
    const LETRAS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789@$%&*+=?!";
    let mut s = format!("# cena {} · {}x{} · teclas: {}\n", c.nome, c.largura, c.altura, c.teclas.join(" "));
    let mut legenda: Vec<String> = Vec::new();
    let mut indice: HashMap<String, char> = HashMap::new();
    let mut grade = String::new();
    for y in 0..buf.area.height {
        let mut linha = String::new();
        for x in 0..buf.area.width {
            let cel = &buf[(x, y)];
            linha.push_str(cel.symbol());
            let nome = nome_do_estilo(cel.style());
            let letra = *indice.entry(nome.clone()).or_insert_with(|| {
                legenda.push(nome.clone());
                LETRAS.chars().nth(legenda.len() - 1).unwrap_or('~')
            });
            grade.push(letra);
        }
        s.push_str(linha.trim_end());
        s.push('\n');
        grade.push('\n');
    }
    s.push_str("== estilos ==\n");
    s.push_str(&grade);
    for (i, nome) in legenda.iter().enumerate() {
        s.push_str(&format!("{} = {}\n", LETRAS.chars().nth(i).unwrap_or('~'), nome));
    }
    if let Some(a) = arquivo {
        s.push_str("== arquivo ==\n");
        s.push_str(a);
    }
    s
}

/// A tela em ANSI de 24 bits, pra virar imagem e comparar com a janela.
fn ansi(buf: &ratatui::buffer::Buffer) -> String {
    let mut s = String::new();
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cel = &buf[(x, y)];
            let st = cel.style();
            s.push_str("\x1b[0m");
            if let Some(Color::Rgb(r, g, b)) = st.fg {
                s.push_str(&format!("\x1b[38;2;{r};{g};{b}m"));
            }
            if let Some(Color::Rgb(r, g, b)) = st.bg {
                s.push_str(&format!("\x1b[48;2;{r};{g};{b}m"));
            }
            if st.add_modifier.contains(Modifier::BOLD) {
                s.push_str("\x1b[1m");
            }
            s.push_str(cel.symbol());
        }
        s.push_str("\x1b[0m\n");
    }
    s
}

#[test]
fn as_telas_batem_com_as_fotos() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/telas");
    let atualizar = std::env::var_os("ATUALIZAR_TELAS").is_some();
    let ansi_dir = std::env::var_os("TELAS_ANSI").map(PathBuf::from);
    let filtro = std::env::var("TELAS_SO").ok();
    let mut falhas = Vec::new();
    for c in cenas() {
        if filtro.as_deref().is_some_and(|f| !c.nome.contains(f)) {
            continue;
        }
        let (buf, arquivo) = rodar(&c);
        let agora = foto(&c, &buf, &arquivo);
        if let Some(d) = &ansi_dir {
            std::fs::create_dir_all(d).unwrap();
            std::fs::write(d.join(format!("{}.ansi", c.nome)), ansi(&buf)).unwrap();
        }
        let caminho = dir.join(format!("{}.tela", c.nome));
        let nova = dir.join(format!("{}.tela.nova", c.nome));
        if atualizar {
            std::fs::write(&caminho, &agora).unwrap();
            let _ = std::fs::remove_file(&nova);
            continue;
        }
        match std::fs::read_to_string(&caminho) {
            Ok(guardada) if guardada == agora => {
                let _ = std::fs::remove_file(&nova);
            }
            Ok(guardada) => {
                std::fs::write(&nova, &agora).unwrap();
                let primeira = guardada
                    .lines()
                    .zip(agora.lines())
                    .enumerate()
                    .find(|(_, (a, b))| a != b)
                    .map(|(i, (a, b))| format!("linha {}:\n  antes: {a}\n  agora: {b}", i + 1))
                    .unwrap_or_else(|| "o tamanho mudou".into());
                falhas.push(format!("{} mudou ({})\n{primeira}", c.nome, nova.display()));
            }
            Err(_) => {
                std::fs::write(&nova, &agora).unwrap();
                falhas.push(format!("{} não tem foto: rode com ATUALIZAR_TELAS=1", c.nome));
            }
        }
    }
    assert!(falhas.is_empty(), "{}", falhas.join("\n\n"));
}
