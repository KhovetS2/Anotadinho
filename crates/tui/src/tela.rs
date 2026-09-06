//! A página virada em LINHAS de terminal (ciclo 286).
//!
//! O `render::Terminal` do núcleo desenha a árvore inteira num texto só.
//! Uma tela precisa de mais: saber a que UNIDADE cada linha pertence,
//! pra o cursor pousar nela, e quais linhas cabem na janela.
//!
//! Este módulo é puro — sem terminal, sem evento, sem I/O. É onde mora
//! tudo que dá pra testar do desenho, e é de propósito: um laço de
//! eventos não se testa, uma função de janela se testa.

use anotadinho_core::inline::{self, Trecho};
use anotadinho_core::navegacao::{mover, Cursor, Passo};
use anotadinho_core::render::{desenhar, Renderizador};
use anotadinho_core::unidade::{Caminho, Tipo, Unidade};

/// Uma linha desenhável, com o endereço de quem a produziu.
#[derive(Debug, Clone, PartialEq)]
pub struct Linha {
    /// A unidade que virou esta linha.
    pub caminho: Caminho,
    /// Profundidade na árvore — vira recuo na tela.
    pub nivel: usize,
    /// O que se lê, já sem os marcadores de markdown.
    pub texto: String,
    /// A marca que abre a linha (`##`, `-`, `[kanban]`).
    pub marca: String,
    /// O texto quebrado em trechos com estilo (ciclo 287).
    ///
    /// Vazio quando não há nada a estilizar — bloco de código, embed,
    /// grupo. Nesses o `texto` já basta, e quebrar seria mentir: dentro
    /// de código um asterisco é asterisco.
    pub trechos: Vec<Trecho>,
    /// O tipo de quem produziu a linha — o desenho estiliza por ele.
    pub tipo: Tipo,
}

/// Renderizador que produz linhas endereçadas.
///
/// É a terceira implementação de `Renderizador` (markdown, terminal, e
/// esta) e a razão de aquele trait existir: a travessia é uma só, e cada
/// destino responde à sua maneira.
#[derive(Default)]
struct Linhas {
    fora: Vec<Linha>,
    caminho: Caminho,
    /// Quantos irmãos já saíram em cada nível — o índice do caminho.
    contadores: Vec<usize>,
}

impl Renderizador for Linhas {
    fn entrar(&mut self, u: &Unidade, nivel: usize) {
        // O caminho é reconstruído na travessia: `entrar`/`sair` não o
        // carregam, e guardar um contador por nível é o suficiente pra
        // saber em que índice estamos.
        self.contadores.truncate(nivel + 1);
        while self.contadores.len() <= nivel {
            self.contadores.push(0);
        }
        self.caminho.truncate(nivel);
        self.caminho.push(self.contadores[nivel]);
        self.contadores[nivel] += 1;

        // Dentro de código nada é interpretado — a mesma regra que a
        // costura do ciclo 276 já seguia pra espaço em branco.
        let trechos = if literal(&u.tipo) {
            Vec::new()
        } else {
            inline::trechos(&corpo(u))
        };
        let texto = if trechos.is_empty() {
            corpo(u)
        } else {
            inline::visivel(&trechos)
        };
        self.fora.push(Linha {
            caminho: self.caminho.clone(),
            nivel,
            texto,
            marca: marca(&u.tipo),
            trechos,
            tipo: u.tipo.clone(),
        });
    }

    /// Desce no embed: é justamente o conteúdo que o ciclo 283 deu a
    /// ele que faz uma página com kanban valer alguma coisa num
    /// terminal.
    fn desce_no_atomico(&self) -> bool {
        true
    }
}

/// O que se mostra de uma unidade, numa linha só.
///
/// Duas coisas acontecem aqui, e as duas foram achadas rodando:
///
/// **O embed não mostra o texto dele.** O texto de um embed é o FENCE
/// INTEIRO — o YAML de origem —, então a linha saía como
/// `[fluxo] {{ type: "fluxo" }}artefato: execucao...`. O rótulo basta; o
/// conteúdo vem nos filhos (ciclo 283). É exatamente o mesmo defeito que
/// o ciclo 284 consertou no `render::Terminal`, cometido de novo aqui
/// porque este é OUTRO renderizador — a lição não viaja sozinha de um
/// pra outro.
///
/// **Quebra de linha vira espaço.** Uma linha de terminal é uma linha:
/// um parágrafo com quebra forte ou um bloco de código traziam `\n` pro
/// meio dela, e o desenho saía torto.
fn corpo(u: &Unidade) -> String {
    if matches!(u.tipo, Tipo::Embed(_)) {
        return String::new();
    }
    u.texto.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Tipo cujo texto é literal: nada dentro dele vira estilo.
fn literal(tipo: &Tipo) -> bool {
    matches!(tipo, Tipo::Codigo(_) | Tipo::Embed(_))
}

/// A marca que abre a linha de cada tipo.
fn marca(tipo: &Tipo) -> String {
    match tipo {
        Tipo::Titulo(n) => "#".repeat((*n).clamp(1, 6) as usize),
        Tipo::Paragrafo => " ".into(),
        Tipo::Citacao => ">".into(),
        Tipo::Codigo(_) => "```".into(),
        Tipo::Lista | Tipo::ListaOrdenada => "·".into(),
        Tipo::Item => "-".into(),
        Tipo::Vazia => "───".into(),
        Tipo::Embed(nome) => format!("[{nome}]"),
        Tipo::Parte { nome, grupo } => {
            if *grupo {
                format!("┌{nome}")
            } else {
                format!("│{nome}")
            }
        }
    }
}

/// A página inteira em linhas, na ordem em que se lê.
pub fn linhas(raiz: &Unidade) -> Vec<Linha> {
    let mut r = Linhas::default();
    desenhar(raiz, &mut r);
    r.fora
}

/// Em que linha está a unidade endereçada.
pub fn linha_de(linhas: &[Linha], caminho: &Caminho) -> Option<usize> {
    linhas.iter().position(|l| &l.caminho == caminho)
}

/// O novo topo da janela pra que `linha` fique visível.
///
/// Rola o MÍNIMO: se a linha já cabe, o topo não muda. Rolar sempre pro
/// centro faz a tela pular a cada tecla, e ler vira perseguir texto.
pub fn rolar(topo: usize, altura: usize, linha: usize) -> usize {
    if altura == 0 {
        return topo;
    }
    if linha < topo {
        linha
    } else if linha >= topo + altura {
        linha + 1 - altura
    } else {
        topo
    }
}

/// Onde o cursor vai com uma tecla de movimento.
///
/// Quem decide é `navegacao::mover`, do núcleo — o mesmo código que a
/// janela usa desde o ciclo 281. Aqui não há régua nova: se a árvore
/// diz que não dá, o cursor fica.
pub fn andar(raiz: &Unidade, cursor: &Caminho, passo: Passo) -> Caminho {
    mover(raiz, &Cursor::em(cursor), passo)
        .map(|c| c.caminho)
        .unwrap_or_else(|| cursor.clone())
}

/// O primeiro destino de uma página — onde o cursor nasce.
pub fn primeiro(raiz: &Unidade) -> Option<Caminho> {
    (!raiz.filhos.is_empty()).then(|| vec![0])
}

#[cfg(test)]
mod testes {
    use super::*;
    use anotadinho_core::analise::analisar;

    const PAGINA: &str = "# Título\n\nUm parágrafo.\n\n- um\n- dois\n\nFim.\n";

    #[test]
    fn o_caminho_de_cada_linha_bate_com_a_arvore() {
        // A reconstrução do caminho durante a travessia é a parte mais
        // fácil de errar deste módulo: `entrar` recebe o nível, não o
        // endereço. Comparar com `percorrer` — que carrega o caminho de
        // verdade — é o que prova que os dois concordam.
        let d = analisar(PAGINA);
        let linhas = linhas(&d);
        let esperado: Vec<Caminho> = d.percorrer().into_iter().map(|(c, _)| c).collect();
        let obtido: Vec<Caminho> = linhas.iter().map(|l| l.caminho.clone()).collect();
        assert_eq!(obtido, esperado);
    }

    #[test]
    fn a_lista_aparece_com_os_itens_dentro() {
        let d = analisar(PAGINA);
        let linhas = linhas(&d);
        // A marca é campo próprio desde o ciclo 287, pra o desenho poder
        // estilizá-la à parte do conteúdo.
        let pares: Vec<String> = linhas
            .iter()
            .map(|l| format!("{} {}", l.marca, l.texto).trim().to_string())
            .collect();
        assert!(pares.contains(&"# Título".to_string()), "{pares:?}");
        assert!(pares.iter().any(|t| t == "- um"), "{pares:?}");
        // O item é um nível abaixo da lista.
        let item = linhas.iter().find(|l| l.texto == "um").unwrap();
        assert_eq!(item.nivel, 1);
    }

    #[test]
    fn o_conteudo_do_embed_entra_no_desenho() {
        // É o que o ciclo 283 deu à árvore, e a razão de uma página com
        // kanban valer alguma coisa num terminal.
        let d = analisar(
            "{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: Card A\n  column: Backlog\n{{ /kanban }}\n",
        );
        let pares: Vec<String> = linhas(&d)
            .into_iter()
            .map(|l| format!("{} {}", l.marca, l.texto).trim().to_string())
            .collect();
        assert!(pares.iter().any(|t| t == "┌column Backlog"), "{pares:?}");
        assert!(pares.iter().any(|t| t == "│card Card A"), "{pares:?}");
    }

    #[test]
    fn o_embed_mostra_o_rotulo_e_nao_o_fence() {
        // Achado rodando a TUI num pty: a linha saía como
        // `[fluxo] {{ type: "fluxo" }}artefato: execucao...`. É o mesmo
        // defeito do ciclo 284, em outro renderizador.
        let d = analisar(
            "{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n",
        );
        let primeira = &linhas(&d)[0];
        assert_eq!(primeira.marca, "[callout]");
        assert_eq!(primeira.texto, "", "o fence vazou pro desenho");
    }

    #[test]
    fn quebra_de_linha_nao_entra_numa_linha_de_terminal() {
        // Bloco de código tem `\n` no texto; uma linha de terminal é uma
        // linha só, e o desenho saía torto.
        let d = analisar("```rust\nfn a() {}\nfn b() {}\n```\n");
        for l in linhas(&d) {
            assert!(!l.texto.contains('\n'), "linha com quebra: {:?}", l.texto);
        }
    }

    #[test]
    fn o_marcador_de_markdown_some_e_vira_marca() {
        // O ponto do ciclo 287: `**forte**` deixa de aparecer com
        // asteriscos e vira um trecho com a marca Negrito.
        use anotadinho_core::inline::Marca;
        let d = analisar("um **forte** e `cod` aqui\n");
        let l = &linhas(&d)[0];
        assert_eq!(l.texto, "um forte e cod aqui", "o marcador ficou na tela");

        let forte = l.trechos.iter().find(|t| t.texto == "forte").unwrap();
        assert!(forte.tem(Marca::Negrito));
        let cod = l.trechos.iter().find(|t| t.texto == "cod").unwrap();
        assert!(cod.tem(Marca::Codigo));
    }

    #[test]
    fn dentro_de_codigo_o_asterisco_continua_asterisco() {
        // Bloco de código é literal: quebrar em trechos ali mentiria, e
        // é a mesma regra que a costura do ciclo 276 já seguia pra
        // espaço em branco.
        let d = analisar("```\numa **coisa** só\n```\n");
        let bloco = linhas(&d).into_iter().find(|l| l.marca == "```").unwrap();
        assert!(bloco.trechos.is_empty(), "quebrou código em trechos");
        assert!(bloco.texto.contains("**coisa**"), "{:?}", bloco.texto);
    }

    #[test]
    fn o_wikilink_mostra_o_texto_e_nao_os_colchetes() {
        use anotadinho_core::inline::Marca;
        let d = analisar("veja [[Sobre|a página]] hoje\n");
        let l = &linhas(&d)[0];
        assert_eq!(l.texto, "veja a página hoje");
        assert!(l.trechos.iter().any(|t| t.tem(Marca::Wikilink)));
    }

    #[test]
    fn rolar_move_o_minimo() {
        // Linha já visível não rola nada.
        assert_eq!(rolar(0, 10, 5), 0);
        assert_eq!(rolar(3, 10, 5), 3);
        // Abaixo da janela: entra pela última linha, não pelo centro.
        assert_eq!(rolar(0, 10, 10), 1);
        assert_eq!(rolar(0, 10, 14), 5);
        // Acima: entra pela primeira.
        assert_eq!(rolar(5, 10, 2), 2);
    }

    #[test]
    fn janela_de_altura_zero_nao_rola() {
        // Acontece de verdade: o terminal pode ficar menor que a borda.
        assert_eq!(rolar(7, 0, 100), 7);
    }

    #[test]
    fn andar_obedece_a_borda_do_nucleo() {
        // Nenhuma régua nova aqui: quem diz que não dá é `mover`, e a
        // resposta a "não dá" é ficar (ciclo 279).
        let d = analisar(PAGINA);
        let primeiro = vec![0usize];
        assert_eq!(andar(&d, &primeiro, Passo::Anterior), primeiro);

        let ultimo = vec![d.filhos.len() - 1];
        assert_eq!(andar(&d, &ultimo, Passo::Proximo), ultimo);

        assert_eq!(andar(&d, &primeiro, Passo::Proximo), vec![1]);
    }

    #[test]
    fn entrar_e_sair_mudam_de_nivel() {
        let d = analisar(PAGINA);
        let lista = vec![2usize];
        let dentro = andar(&d, &lista, Passo::Entrar);
        assert_eq!(dentro, vec![2, 0]);
        assert_eq!(andar(&d, &dentro, Passo::Sair), lista);
    }
}
