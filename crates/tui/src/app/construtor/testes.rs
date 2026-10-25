//! O construtor de diagramas (ciclo 500), pelas teclas, como a pessoa usa.

use super::*;
use crate::app::{desenhar as desenhar_tela, tecla as tecla_da_tela, Foco as FocoDaTela};
use anotadinho_core::analise::analisar;
use anotadinho_ipc::PageMeta;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

const FRONT: &str = "---\ntitle: Notas\n---\n";

fn pagina(corpo: &str) -> Estado {
    let paginas = vec![PageMeta { path: "pages/notas.md".into(), title: "Notas".into(), section: "pages".into() }];
    let mut e = Estado::novo(paginas, analisar(""));
    e.abrir_texto(&format!("{FRONT}{corpo}"), Some("v1".into()));
    e.foco = FocoDaTela::Conteudo;
    e
}

fn teclas(e: &mut Estado, ts: &[&str]) {
    for t in ts {
        tecla_da_tela(e, t);
    }
}

fn digitar(e: &mut Estado, texto: &str) {
    for c in texto.chars() {
        tecla_da_tela(e, &c.to_string());
    }
}

fn corpo_gravado(e: &Estado) -> String {
    let texto = e.gravacao.clone().expect("nada pra gravar");
    assert!(texto.starts_with(FRONT), "o frontmatter mudou:\n{texto}");
    texto[FRONT.len()..].to_string()
}

fn construtor(e: &Estado) -> &Construtor {
    match &e.modal {
        Some(Modal::Construtor(c)) => c,
        outro => panic!("o construtor não está aberto: {outro:?} (aviso: {:?})", e.aviso),
    }
}

fn codigo(e: &Estado) -> String {
    construtor(e).codigo.texto().to_string()
}

fn desenho(e: &mut Estado, largura: u16, altura: u16) -> Vec<String> {
    let mut term = Terminal::new(TestBackend::new(largura, altura)).unwrap();
    term.draw(|f| desenhar_tela(f, e)).unwrap();
    let buf = term.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
        .collect()
}

/// Abre o construtor no bloco `i` da página: Enter entra na vista (498),
/// `c` abre.
fn abrir(e: &mut Estado, bloco: usize) {
    e.cursor = vec![bloco];
    teclas(e, &["Enter", "c"]);
    construtor(e);
}

const FLUXO: &str = "Antes.\n\n```mermaid\ngraph TD\n  A[Começo]\n  B{Decide}\n  C(Fim)\n  A --> B\n  B -->|sim| C\n  A --> C\n```\n\nDepois.\n";

#[test]
fn abre_pela_vista_do_bloco_e_pela_barra_de_comandos() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    let c = construtor(&e);
    assert_eq!(c.aba, Aba::Construtor);
    assert_eq!(c.codigo.texto(), "graph TD\n  A[Começo]\n  B{Decide}\n  C(Fim)\n  A --> B\n  B -->|sim| C\n  A --> C");
    // A barra de comandos só oferece com o cursor num bloco mermaid.
    let mut e = pagina(FLUXO);
    e.cursor = vec![0];
    tecla_da_tela(&mut e, ":");
    digitar(&mut e, "construtor do diagrama");
    tecla_da_tela(&mut e, "Enter");
    assert!(!matches!(e.modal, Some(Modal::Construtor(_))), "abriu fora de um bloco mermaid");
    e.modal = None;
    e.cursor = vec![1];
    tecla_da_tela(&mut e, ":");
    digitar(&mut e, "construtor do diagrama");
    tecla_da_tela(&mut e, "Enter");
    construtor(&e);
    // `c` num bloco que não é diagrama avisa e não abre nada.
    let mut e = pagina("```rust\nfn main() {}\n```\n");
    e.cursor = vec![0];
    teclas(&mut e, &["Enter", "c"]);
    assert!(e.modal.is_none() && e.aviso.as_deref().is_some_and(|a| a.contains("mermaid")), "{:?}", e.aviso);
}

#[test]
fn a_tela_mostra_lista_previa_e_abas_sem_passar_da_largura() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    let tela = desenho(&mut e, 120, 32);
    let junto = tela.join("\n");
    assert!(junto.contains("Editar diagrama · Fluxograma"), "{junto}");
    assert!(junto.contains(" Construtor ") && junto.contains(" Código "), "{junto}");
    assert!(junto.contains("A · Começo · caixa") && junto.contains("B --> C |sim|"), "{junto}");
    assert!(junto.contains("Prévia") && junto.contains("Começo │"), "a prévia não desenhou:\n{junto}");
    for l in &tela {
        assert!(unicode_width::UnicodeWidthStr::width(l.as_str()) <= 120, "{l}");
    }
}

#[test]
fn editar_o_texto_de_um_no_reescreve_o_codigo_e_a_previa() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    // Direção → (rótulo) → nó A.
    teclas(&mut e, &["j", "Enter"]);
    assert!(construtor(&e).form.is_some());
    // id → texto; `c` limpa e digita.
    teclas(&mut e, &["j", "c"]);
    digitar(&mut e, "Partida");
    tecla_da_tela(&mut e, "Enter");
    assert!(codigo(&e).contains("A[Partida]"), "{}", codigo(&e));
    let tela = desenho(&mut e, 120, 32).join("\n");
    assert!(tela.contains("Partida │"), "a prévia não acompanhou:\n{tela}");
    // A forma gira com `l`.
    teclas(&mut e, &["j", "l"]);
    assert!(codigo(&e).contains("A(Partida)"), "{}", codigo(&e));
    // Esc fecha o formulário; Ctrl+S grava SÓ o bloco.
    teclas(&mut e, &["Escape", "Ctrl+s"]);
    assert!(e.modal.is_none());
    let corpo = corpo_gravado(&e);
    assert!(corpo.starts_with("Antes.\n\n```mermaid\ngraph TD\n  A(Partida)\n"), "{corpo}");
    assert!(corpo.ends_with("```\n\nDepois.\n"), "{corpo}");
}

#[test]
fn trocar_o_id_leva_as_ligacoes_e_id_repetido_e_recusado() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    teclas(&mut e, &["j", "Enter", "c"]);
    digitar(&mut e, "B");
    tecla_da_tela(&mut e, "Enter");
    assert!(construtor(&e).aviso.as_deref().is_some_and(|a| a.contains("já existe")), "{:?}", construtor(&e).aviso);
    assert!(codigo(&e).contains("A[Começo]"), "o id repetido entrou: {}", codigo(&e));
    tecla_da_tela(&mut e, "c");
    digitar(&mut e, "Inicio");
    tecla_da_tela(&mut e, "Enter");
    let c = codigo(&e);
    assert!(c.contains("Inicio[Começo]") && c.contains("Inicio --> B") && c.contains("Inicio --> C"), "{c}");
}

#[test]
fn adicionar_uma_ligacao_pelo_mais() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    // G é a última linha: "+ ligação".
    teclas(&mut e, &["G", "Enter"]);
    let c = construtor(&e);
    assert!(matches!(c.form, Some((_, Linha::Ligacao(3)))), "{:?}", c.form.as_ref().map(|f| f.1));
    // de A, para B (o segundo nó): `j` vai pro "para", `l` gira pra C.
    teclas(&mut e, &["j", "l", "Escape"]);
    assert!(codigo(&e).ends_with("  A --> C\n  A --> C\n"), "{}", codigo(&e));
    // `o` numa ligação cria logo depois dela.
    teclas(&mut e, &["k", "k", "o"]);
    assert!(matches!(construtor(&e).form, Some((_, Linha::Ligacao(2)))));
}

#[test]
fn apagar_um_no_apaga_as_ligacoes_dele() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    // nó B.
    teclas(&mut e, &["j", "j", "d", "d"]);
    let c = codigo(&e);
    assert!(!c.contains('B'), "{c}");
    assert!(c.contains("A --> C") && c.lines().filter(|l| l.contains("-->")).count() == 1, "{c}");
    // O cursor fica num nó, não num rótulo.
    assert!(matches!(construtor(&e).atual(), Some(Linha::No(1))), "{:?}", construtor(&e).atual());
    // `K` sobe.
    teclas(&mut e, &["K"]);
    assert!(codigo(&e).starts_with("graph TD\n  C(Fim)\n  A[Começo]\n"), "{}", codigo(&e));
}

#[test]
fn abrir_e_aplicar_sem_mexer_devolve_o_arquivo_igual() {
    // Código com a cara de quem escreveu à mão: sem espaços e com `;`. O
    // construtor escreveria `A --> B`; sem mudança no modelo, ninguém
    // regenera nada.
    let corpo = "Antes.\n\n```mermaid\ngraph LR; A[Um]-->B[Dois]\n```\n\nDepois.\n";
    let mut e = pagina(corpo);
    abrir(&mut e, 1);
    assert_eq!(construtor(&e).aba, Aba::Construtor);
    // Abre o formulário de um nó, gira a forma e volta: o modelo é o mesmo.
    teclas(&mut e, &["j", "Enter", "j", "j", "l", "h", "Escape"]);
    assert_eq!(codigo(&e), "graph LR; A[Um]-->B[Dois]");
    tecla_da_tela(&mut e, "Ctrl+s");
    assert!(e.modal.is_none());
    assert!(e.gravacao.is_none(), "gravou sem mudança: {:?}", e.gravacao);
    // E pelo caminho de gravação, o bloco intacto devolve o arquivo igual.
    let mut e = pagina(corpo);
    abrir(&mut e, 1);
    let Some(Modal::Construtor(mut c)) = e.modal.take() else { unreachable!() };
    c.original.clear();
    assert!(aplicar(&mut e, &mut c));
    assert_eq!(corpo_gravado(&e), corpo);
}

#[test]
fn fora_do_modelo_abre_no_codigo_com_o_motivo() {
    let corpo = "```mermaid\ngraph TD\n  %% um comentário\n  A --> B\n```\n";
    let mut e = pagina(corpo);
    abrir(&mut e, 0);
    assert_eq!(construtor(&e).aba, Aba::Codigo);
    let tela = desenho(&mut e, 120, 30).join("\n");
    assert!(tela.contains("O construtor não monta este diagrama") && tela.contains("comentário"), "{tela}");
    // A aba Construtor recusa, e diz por quê.
    tecla_da_tela(&mut e, "Tab");
    assert_eq!(construtor(&e).aba, Aba::Codigo);
    assert!(construtor(&e).aviso.as_deref().is_some_and(|a| a.contains("comentário")));
    // Tirar o comentário pelo código libera o construtor.
    e.modal.as_mut().map(|m| match m {
        Modal::Construtor(c) => c.codigo = crate::editor_codigo::EditorDeCodigo::novo("graph TD\n  A --> B", Some("mermaid".into())),
        _ => unreachable!(),
    });
    tecla_da_tela(&mut e, "Tab");
    assert_eq!(construtor(&e).aba, Aba::Construtor);
    // Na aba Código as teclas são texto, e Ctrl+S grava o que se digitou.
    tecla_da_tela(&mut e, "Tab");
    // O editor de 506: o cursor começa no topo, e o Enter mantém o recuo.
    teclas(&mut e, &["Ctrl+End", "Enter"]);
    digitar(&mut e, "B --> C");
    tecla_da_tela(&mut e, "Ctrl+s");
    assert_eq!(corpo_gravado(&e), "```mermaid\ngraph TD\n  A --> B\n  B --> C\n```\n");
}

#[test]
fn bloco_novo_pelo_menu_escolhe_o_tipo_e_insere() {
    let mut e = pagina("# Título\n\nUm parágrafo.\n");
    e.cursor = vec![1];
    teclas(&mut e, &["o", "/"]);
    digitar(&mut e, "diagrama");
    tecla_da_tela(&mut e, "Enter");
    let c = construtor(&e);
    assert_eq!(c.tipos, Some(0));
    assert!(c.codigo.texto().starts_with("graph TD"));
    // `j` aponta Sequência e a prévia já mostra o exemplo dela.
    tecla_da_tela(&mut e, "j");
    assert!(codigo(&e).starts_with("sequenceDiagram"));
    let tela = desenho(&mut e, 120, 30).join("\n");
    assert!(tela.contains("Que diagrama?") && tela.contains("Pessoa"), "{tela}");
    tecla_da_tela(&mut e, "Enter");
    let c = construtor(&e);
    assert!(c.tipos.is_none() && c.aba == Aba::Construtor);
    // A primeira linha é o tipo: Enter reabre o seletor.
    tecla_da_tela(&mut e, "Enter");
    assert!(construtor(&e).tipos.is_some());
    teclas(&mut e, &["j", "j", "Enter"]);
    assert!(codigo(&e).starts_with("gantt"), "{}", codigo(&e));
    tecla_da_tela(&mut e, "Ctrl+s");
    assert!(e.modal.is_none() && e.bloco_a_inserir.is_none());
    let corpo = corpo_gravado(&e);
    assert!(corpo.starts_with("# Título\n\nUm parágrafo.\n\n```mermaid\ngantt\n  dateFormat YYYY-MM-DD\n"), "{corpo}");
    // Esc no seletor, antes de escolher, desiste do bloco.
    let mut e = pagina("# Título\n\nUm parágrafo.\n");
    e.cursor = vec![1];
    teclas(&mut e, &["o", "/"]);
    digitar(&mut e, "diagrama");
    teclas(&mut e, &["Enter", "Escape"]);
    assert!(e.modal.is_none() && e.bloco_a_inserir.is_none() && e.gravacao.is_none());
}

#[test]
fn esc_so_pergunta_quando_mudou() {
    let mut e = pagina(FLUXO);
    abrir(&mut e, 1);
    tecla_da_tela(&mut e, "Escape");
    assert!(e.modal.is_none(), "perguntou sem ter mudado");
    abrir(&mut e, 1);
    teclas(&mut e, &["j", "x", "Escape"]);
    assert!(construtor(&e).confirmar);
    assert!(desenho(&mut e, 120, 30).join("\n").contains("Descartar as mudanças?"));
    tecla_da_tela(&mut e, "n");
    assert!(!construtor(&e).confirmar);
    teclas(&mut e, &["Escape", "y"]);
    assert!(e.modal.is_none() && e.gravacao.is_none());
}

#[test]
fn gantt_confere_a_duracao_e_a_data() {
    let corpo = "```mermaid\ngantt\n  dateFormat YYYY-MM-DD\n  section Primeira\n    Planejar :a1, 2026-11-02, 3d\n    Fazer :after a1, 5d\n```\n";
    let mut e = pagina(corpo);
    abrir(&mut e, 0);
    // Título → seção → Planejar → Fazer.
    teclas(&mut e, &["j", "j", "j", "Enter"]);
    let Some((form, Linha::Tarefa(0, 1))) = &construtor(&e).form else { panic!("{:?}", construtor(&e).form) };
    let visiveis: Vec<&str> = form.campos.iter().filter(|c| !c.escondido).map(|c| c.chave).collect();
    assert_eq!(&visiveis[..5], ["nome", "id", "inicio", "depois", "fim"], "{visiveis:?}");
    // nome, id, começa, depois de, termina, duração.
    teclas(&mut e, &["j", "j", "j", "j", "j", "c"]);
    digitar(&mut e, "três dias");
    tecla_da_tela(&mut e, "Enter");
    assert!(construtor(&e).aviso.as_deref().is_some_and(|a| a.contains("duração")), "{:?}", construtor(&e).aviso);
    assert!(codigo(&e).contains("Fazer :after a1, 5d"), "{}", codigo(&e));
    tecla_da_tela(&mut e, "c");
    digitar(&mut e, "2w");
    tecla_da_tela(&mut e, "Enter");
    assert!(codigo(&e).contains("Fazer :after a1, 2w"), "{}", codigo(&e));
    // "Começa numa data": a data vem do seletor (Enter abre, `l` um dia).
    teclas(&mut e, &["k", "k", "k", "h"]);
    assert!(codigo(&e).contains("Fazer :2026-11-02, 2w"), "{}", codigo(&e));
    teclas(&mut e, &["j", "Enter", "l", "Enter"]);
    assert!(codigo(&e).contains("Fazer :2026-11-03, 2w"), "{}", codigo(&e));
}

#[test]
fn participante_citado_nao_se_apaga() {
    let corpo = "```mermaid\nsequenceDiagram\n  participant A\n  participant B\n  participant C\n  A->>B: oi\n```\n";
    let mut e = pagina(corpo);
    abrir(&mut e, 0);
    teclas(&mut e, &["d", "d"]);
    assert!(construtor(&e).aviso.as_deref().is_some_and(|a| a.contains("usado em 1")), "{:?}", construtor(&e).aviso);
    assert!(codigo(&e).contains("participant A"));
    // O que não é citado vai.
    teclas(&mut e, &["j", "j", "x"]);
    assert!(!codigo(&e).contains("participant C"), "{}", codigo(&e));
    // `n` numa mensagem cria uma nota logo depois.
    teclas(&mut e, &["G", "k", "k", "n", "Escape"]);
    assert!(codigo(&e).contains("A->>B: oi\n  Note right of A: nota"), "{}", codigo(&e));
}

#[test]
fn a_previa_rola_com_as_teclas_da_vista() {
    let mut corpo = String::from("```mermaid\ngraph TD\n");
    for i in 0..30 {
        corpo.push_str(&format!("  N{i}[nó número {i}] --> N{}\n", i + 1));
    }
    corpo.push_str("```\n");
    let mut e = pagina(&corpo);
    abrir(&mut e, 0);
    let tela = desenho(&mut e, 100, 24).join("\n");
    assert!(tela.contains("Prévia · ↕ 1–"), "{tela}");
    teclas(&mut e, &["p", "G"]);
    desenho(&mut e, 100, 24);
    let c = construtor(&e);
    assert_eq!(c.foco, Foco::Previa);
    let m = c.medida.get();
    assert!(m.max_y() > 0 && c.vista.y == m.max_y(), "{m:?} {:?}", c.vista);
    teclas(&mut e, &["j", "j"]);
    assert_eq!(construtor(&e).vista.y, m.max_y(), "rolou além do fim");
    teclas(&mut e, &["g", "Escape"]);
    assert_eq!(construtor(&e).foco, Foco::Edicao);
    assert_eq!(construtor(&e).vista.y, 0);
}

#[test]
fn mexer_numa_tarefa_com_duas_dependencias_nao_perde_a_segunda() {
    let corpo = "```mermaid\ngantt\n  dateFormat YYYY-MM-DD\n  section S\n    Um :a1, 2026-11-02, 3d\n    Dois :a2, 2026-11-02, 2d\n    Tres :after a1 a2, 1d\n```\n";
    let mut e = pagina(corpo);
    abrir(&mut e, 0);
    teclas(&mut e, &["j", "j", "j", "j", "Enter", "c"]);
    digitar(&mut e, "Três");
    tecla_da_tela(&mut e, "Enter");
    assert!(codigo(&e).contains("Três :after a1 a2, 1d"), "{}", codigo(&e));
}
