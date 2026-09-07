//! A costura entre o que uma página É e como ela é DESENHADA.
//!
//! Padrão **Bridge**: de um lado a árvore de `Unidade` (a abstração), do
//! outro os renderizadores (a implementação). As duas dimensões crescem
//! sozinhas — nasce tipo de bloco, nasce renderizador — e é essa
//! ortogonalidade que a página do padrão dá como aplicabilidade.
//!
//! A travessia é um **Visitor**: quem desenha não sabe andar na árvore,
//! só responde "o que fazer com esta unidade". Em Rust isso é um trait
//! com dois métodos, não a maquinaria de duplo despacho do livro.
//!
//! ## Por que isto existe antes de haver um CLI
//!
//! A regra que importa é a DIREÇÃO da dependência: o modelo não pode
//! conhecer o renderizador. Enquanto o desenho e o modelo forem a mesma
//! coisa — hoje o editor guarda a estrutura em atributos do DOM — não
//! existe porta pra uma segunda saída, nem forma de testar a estrutura
//! sem navegador.
//!
//! Este módulo prova que a costura aguenta duas saídas MUITO diferentes
//! (markdown e árvore de terminal) com a mesma travessia e zero DOM. O
//! renderizador do DOM ainda não passa por aqui: o editor Yew é a
//! implementação maior do projeto e migrá-lo é trabalho à parte. O que
//! existe hoje é a costura e a prova de que ela serve.

use crate::unidade::{Tipo, Unidade};

/// Quem sabe desenhar uma unidade.
///
/// `entrar` é chamado ao chegar na unidade e `sair` ao terminar os
/// filhos dela — é o par que permite abrir e fechar delimitadores sem o
/// renderizador precisar andar na árvore sozinho.
pub trait Renderizador {
    /// Chegou nesta unidade. `nivel` é a profundidade (0 = filho direto
    /// da raiz).
    fn entrar(&mut self, unidade: &Unidade, nivel: usize);

    /// Terminou os filhos desta unidade.
    fn sair(&mut self, _unidade: &Unidade, _nivel: usize) {}

    /// Descer nos filhos de uma unidade atômica?
    ///
    /// O markdown de um embed é o fence inteiro, então quem serializa
    /// não desce. Uma árvore de terminal, ao contrário, mostra o que tem
    /// dentro. Mesma travessia, respostas diferentes — é a razão de isto
    /// ser pergunta ao renderizador e não regra da travessia.
    fn desce_no_atomico(&self) -> bool {
        false
    }
}

/// Percorre a árvore entregando cada unidade ao renderizador.
pub fn desenhar<R: Renderizador>(raiz: &Unidade, r: &mut R) {
    for filho in &raiz.filhos {
        visitar(filho, r, 0);
    }
}

fn visitar<R: Renderizador>(u: &Unidade, r: &mut R, nivel: usize) {
    r.entrar(u, nivel);
    if !u.politica().atomica || r.desce_no_atomico() {
        for filho in &u.filhos {
            visitar(filho, r, nivel + 1);
        }
    }
    r.sair(u, nivel);
}

/// Desenha a página como markdown.
#[derive(Default)]
pub struct Markdown {
    saida: String,
    /// Estado da lista sendo escrita: o item precisa saber se numera.
    em_lista_ordenada: bool,
    contador_item: usize,
    /// Nível a partir do qual estamos dentro de uma unidade já escrita
    /// pela fonte — os filhos dela não são reescritos.
    pulando_de: Option<usize>,
}

impl Markdown {
    /// O markdown acumulado.
    pub fn resultado(self) -> String {
        self.saida.trim_end().to_string()
    }
}

impl Renderizador for Markdown {
    fn entrar(&mut self, u: &Unidade, nivel: usize) {
        // Já estamos dentro de algo escrito pela fonte: os filhos vieram
        // junto, e escrevê-los de novo duplicaria o conteúdo.
        if self.pulando_de.is_some() {
            return;
        }
        // A FONTE ganha (ciclo 274). Quem não foi editado volta byte a
        // byte, e o arquivo não muda por ter sido lido.
        //
        // É o que dispensa um analisador perfeito: sem isto, só 29 das
        // 242 páginas do vault voltavam idênticas, e cada detalhe
        // corrigido revelava o seguinte.
        if let Some(fonte) = &u.fonte {
            // Apara só QUEBRAS, nunca espaços: dois espaços no fim de
            // uma linha são quebra forte em markdown, e `trim_end()`
            // os comia. Era a única das 242 páginas do vault que ainda
            // mudava de árvore na ida e volta (ciclo 275).
            self.saida.push_str(fonte.trim_end_matches('\n'));
            self.saida.push_str("\n\n");
            self.pulando_de = Some(nivel);
            return;
        }
        match &u.tipo {
            // Parte de embed NÃO vira markdown: o texto de um embed é o
            // dele, escrito de uma vez pela fonte, e `desce_no_atomico`
            // é `false` aqui justamente pra os filhos não serem
            // reescritos. Chegar aqui significaria uma parte solta fora
            // de um embed, que não é coisa que `analisar` produza.
            Tipo::Parte { .. } => {}
            Tipo::Titulo(n) => {
                let marca = "#".repeat((*n).clamp(1, 6) as usize);
                self.saida.push_str(&format!("{marca} {}\n\n", u.texto));
            }
            Tipo::Paragrafo => self.saida.push_str(&format!("{}\n\n", u.texto)),
            Tipo::Citacao => {
                // Uma citação de várias linhas volta com `>` em CADA
                // linha — é assim que ela entrou, e escrever tudo numa
                // linha só mudaria o arquivo.
                for linha in u.texto.split('\n') {
                    if linha.is_empty() {
                        self.saida.push_str(">\n");
                    } else {
                        self.saida.push_str(&format!("> {linha}\n"));
                    }
                }
                self.saida.push('\n');
            }
            Tipo::Codigo(lingua) => self.saida.push_str(&format!(
                "```{}\n{}\n```\n\n",
                lingua.as_deref().unwrap_or(""),
                u.texto
            )),
            Tipo::Item => {
                let recuo = "  ".repeat(nivel.saturating_sub(1));
                // Numerada volta numerada. O contador é o do
                // renderizador, não da unidade: markdown aceita `1.` em
                // todos os itens, e guardar o número por item seria
                // guardar algo que o arquivo não precisa dizer.
                if self.em_lista_ordenada {
                    self.contador_item += 1;
                    let n = self.contador_item;
                    if u.texto.is_empty() {
                        self.saida.push_str(&format!("{recuo}{n}.\n"));
                    } else {
                        self.saida.push_str(&format!("{recuo}{n}. {}\n", u.texto));
                    }
                    return;
                }
                // Item vazio sai como `-` e não `- `: espaço no fim de
                // linha é sujeira que alguns editores comem sozinhos, e
                // aí o arquivo mudaria sem ninguém ter editado.
                if u.texto.is_empty() {
                    self.saida.push_str(&format!("{recuo}-\n"));
                } else {
                    self.saida.push_str(&format!("{recuo}- {}\n", u.texto));
                }
            }
            Tipo::Lista => {
                self.em_lista_ordenada = false;
            }
            Tipo::ListaOrdenada => {
                self.em_lista_ordenada = true;
                self.contador_item = 0;
            }
            Tipo::Vazia => self.saida.push_str("---\n\n"),
            Tipo::Embed(_) => {
                // O `texto` de um embed JÁ É o markdown dele — a cerca
                // inteira, com abertura e fechamento (ver
                // `analise::analisar`). Envolvê-lo de novo produzia
                // cerca dentro de cerca, e foi o que a ida e volta do
                // ciclo 271 pegou.
                //
                // É também a razão de `desce_no_atomico` ser `false`
                // aqui: o markdown do embed não é a soma dos filhos, é o
                // texto dele.
                self.saida.push_str(u.texto.trim_end());
                self.saida.push_str("\n\n");
            }
        }
    }

    fn sair(&mut self, u: &Unidade, nivel: usize) {
        if self.pulando_de == Some(nivel) {
            self.pulando_de = None;
            return;
        }
        if self.pulando_de.is_some() {
            return;
        }
        if matches!(u.tipo, Tipo::Lista | Tipo::ListaOrdenada) {
            self.saida.push('\n');
            self.em_lista_ordenada = false;
        }
    }
}

/// Desenha a página como uma árvore de terminal.
///
/// Não é um editor de CLI — é a prova de que a mesma travessia serve
/// uma saída que não tem nada a ver com a outra: aqui não há sintaxe de
/// markdown, há recuo, e o atômico é ABERTO em vez de virar um bloco só.
#[derive(Default)]
pub struct Terminal {
    saida: String,
}

impl Terminal {
    /// A árvore acumulada.
    pub fn resultado(self) -> String {
        self.saida.trim_end().to_string()
    }

    fn rotulo(tipo: &Tipo) -> String {
        match tipo {
            Tipo::Paragrafo => "¶".to_string(),
            Tipo::Titulo(n) => format!("h{n}"),
            Tipo::Citacao => "❝".to_string(),
            Tipo::Codigo(_) => "```".to_string(),
            Tipo::Lista => "•••".to_string(),
            Tipo::ListaOrdenada => "1••".to_string(),
            Tipo::Item => "•".to_string(),
            Tipo::Vazia => "───".to_string(),
            Tipo::Embed(nome) => format!("[{nome}]"),
            // A parte se anuncia pelo vocabulário do embed. É o que o
            // terminal tem pra mostrar a estrutura de dentro de um
            // kanban ou de uma tabela — até o ciclo 283 ele descia no
            // atômico e não achava nada.
            Tipo::Parte { nome, arranjo } => match arranjo {
                crate::unidade::Arranjo::Folha => format!("│{nome}"),
                crate::unidade::Arranjo::Coluna => format!("┌{nome}"),
                crate::unidade::Arranjo::Linha => format!("├{nome}"),
            },
        }
    }
}

impl Renderizador for Terminal {
    fn entrar(&mut self, u: &Unidade, nivel: usize) {
        let recuo = "  ".repeat(nivel);
        let rotulo = Self::rotulo(&u.tipo);
        // O texto de um embed é o FENCE INTEIRO — o YAML de origem, com
        // quebras de linha. Num desenho de estrutura isso não é
        // conteúdo, é fonte: despejava dez linhas de `columns:`/`items:`
        // no meio do desenho, e o conteúdo de verdade vinha depois, nos
        // filhos que o ciclo 283 deu a ele.
        //
        // O rótulo basta. Quem quer o fence tem `read`.
        let e_embed = matches!(u.tipo, Tipo::Embed(_));
        if u.texto.is_empty() || e_embed {
            self.saida.push_str(&format!("{recuo}{rotulo}\n"));
        } else {
            self.saida.push_str(&format!("{recuo}{rotulo} {}\n", u.texto));
        }
    }

    fn desce_no_atomico(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn documento() -> Unidade {
        Unidade::com_filhos(
            Tipo::Paragrafo,
            vec![
                Unidade::com_texto(Tipo::Titulo(1), "Título"),
                Unidade::com_texto(Tipo::Paragrafo, "um parágrafo"),
                Unidade::com_filhos(
                    Tipo::Lista,
                    vec![
                        Unidade::com_texto(Tipo::Item, "primeiro"),
                        Unidade::com_texto(Tipo::Item, "segundo"),
                    ],
                ),
                // Um embed traz o markdown DELE no texto, e os filhos
                // representam o que ele desenha por dentro — que só o
                // renderizador de tela abre.
                Unidade {
                    tipo: Tipo::Embed("callout".into()),
                    texto: "{{ type: \"callout\" }}\nbody: oi\n{{ /callout }}".into(),
                    filhos: vec![Unidade::com_texto(Tipo::Paragrafo, "dentro do embed")],
                    fonte: None,
                    intervalo: None,
                },
            ],
        )
    }

    #[test]
    fn a_mesma_arvore_sai_em_markdown() {
        let mut r = Markdown::default();
        desenhar(&documento(), &mut r);
        let md = r.resultado();
        assert!(md.starts_with("# Título"), "{md}");
        assert!(md.contains("um parágrafo"), "{md}");
        assert!(md.contains("- primeiro\n- segundo"), "{md}");
        assert!(md.contains(r#"{{ type: "callout" }}"#), "{md}");
    }

    #[test]
    fn a_mesma_arvore_sai_em_terminal() {
        let mut r = Terminal::default();
        desenhar(&documento(), &mut r);
        let txt = r.resultado();
        assert!(txt.contains("h1 Título"), "{txt}");
        assert!(txt.contains("  • primeiro"), "recuo do item:\n{txt}");
        assert!(txt.contains("[callout]"), "{txt}");
    }

    #[test]
    fn cada_renderizador_decide_se_desce_no_atomico() {
        // A mesma árvore, a mesma travessia, e uma diferença de
        // conteúdo que vem SÓ da política do renderizador. É o que o
        // Bridge existe pra permitir: as duas dimensões variam sem uma
        // saber da outra.
        let d = documento();

        let mut md = Markdown::default();
        desenhar(&d, &mut md);
        assert!(
            !md.resultado().contains("dentro do embed"),
            "o markdown não devia descer no embed: o fence é o conteúdo"
        );

        let mut term = Terminal::default();
        desenhar(&d, &mut term);
        assert!(
            term.resultado().contains("dentro do embed"),
            "o terminal devia ABRIR o embed"
        );
    }

    #[test]
    fn o_terminal_desenha_o_conteudo_de_um_kanban_de_verdade() {
        // O ciclo 266 fez o terminal descer no atômico; o 283 deu ao
        // embed o que descer. Entre um e outro, ele descia e achava
        // vazio — a página do porte CLI parava na borda de cada embed.
        //
        // Este teste liga as duas pontas: o markdown de um kanban entra
        // pelo `analisar` e sai desenhado, coluna por coluna.
        let d = crate::analise::analisar(
            "{{ type: \"kanban\" }}\ncolumns:\n- Backlog\n- Feito\nitems:\n- title: Card A\n  column: Backlog\n{{ /kanban }}\n",
        );
        let mut term = Terminal::default();
        desenhar(&d, &mut term);
        let saida = term.resultado();

        assert!(saida.contains("[kanban]"), "faltou o embed:\n{saida}");
        // O rótulo, e NÃO o fence: o YAML de origem não é conteúdo.
        assert!(
            !saida.contains("columns:"),
            "o terminal despejou o fence do embed:\n{saida}"
        );
        assert!(saida.contains("┌column Backlog"), "faltou a coluna:\n{saida}");
        assert!(saida.contains("│card Card A"), "faltou o cartão:\n{saida}");
        // E o markdown continua não descendo: o fence é o conteúdo dele.
        let mut md = Markdown::default();
        desenhar(&d, &mut md);
        assert!(!md.resultado().contains("┌column"));
    }

    #[test]
    fn documento_vazio_sai_vazio_nos_dois() {
        let vazio = Unidade::nova(Tipo::Paragrafo);
        let mut md = Markdown::default();
        desenhar(&vazio, &mut md);
        assert_eq!(md.resultado(), "");
        let mut term = Terminal::default();
        desenhar(&vazio, &mut term);
        assert_eq!(term.resultado(), "");
    }

    #[test]
    fn o_nivel_chega_certo_no_renderizador() {
        /// Renderizador de mentira que só anota (nível, tipo).
        #[derive(Default)]
        struct Espiao(Vec<(usize, String)>);
        impl Renderizador for Espiao {
            fn entrar(&mut self, u: &Unidade, nivel: usize) {
                self.0.push((nivel, format!("{:?}", u.tipo)));
            }
        }
        let mut e = Espiao::default();
        desenhar(&documento(), &mut e);
        let niveis: Vec<usize> = e.0.iter().map(|(n, _)| *n).collect();
        // título, parágrafo, lista, item, item, embed — o embed não
        // desce porque o espião herda `desce_no_atomico = false`.
        assert_eq!(niveis, [0, 0, 0, 1, 1, 0]);
    }

    #[test]
    fn sair_e_chamado_depois_dos_filhos() {
        // A garantia que permite abrir e fechar delimitador: `sair` da
        // lista vem DEPOIS dos itens dela.
        #[derive(Default)]
        struct Ordem(Vec<String>);
        impl Renderizador for Ordem {
            fn entrar(&mut self, u: &Unidade, _n: usize) {
                self.0.push(format!("entra {:?}", u.tipo));
            }
            fn sair(&mut self, u: &Unidade, _n: usize) {
                self.0.push(format!("sai {:?}", u.tipo));
            }
        }
        let lista = Unidade::com_filhos(
            Tipo::Paragrafo,
            vec![Unidade::com_filhos(
                Tipo::Lista,
                vec![Unidade::com_texto(Tipo::Item, "x")],
            )],
        );
        let mut o = Ordem::default();
        desenhar(&lista, &mut o);
        assert_eq!(o.0, ["entra Lista", "entra Item", "sai Item", "sai Lista"]);
    }
}
