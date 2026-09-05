//! A unidade de navegação: o que é um bloco, como dado.
//!
//! Este módulo existe pra responder uma pergunta que o projeto vinha
//! respondendo de dois jeitos diferentes na mesma tela (spec
//! `o-que-e-um-bloco`): o modo de navegação achava que embed é bloco, o
//! vim e a seleção achavam que não. A discordância era possível porque
//! "ser um bloco" não era um DADO — era o efeito de `marcar_blocos()`
//! ter carimbado atributos num elemento do DOM.
//!
//! Aqui a unidade é uma árvore (padrão **Composite**), e a intenção do
//! padrão descreve o pedido: tratar o individual e o composto de forma
//! uniforme. É o que faz `dd` funcionar em dois níveis sem serem dois
//! comandos — no título de um card apaga o título, com o card
//! selecionado apaga o card.
//!
//! ## Zero DOM, de propósito
//!
//! Nada aqui conhece `web_sys`. O modelo não pode conhecer o
//! renderizador — é o que permite testar a regra sem navegador e, mais
//! adiante, desenhar a mesma árvore num terminal (spec: a versão CLI).
//!
//! ## Não confundir com `crate::block::Block`
//!
//! `Block` é a linha de markdown com `id::` e `depth`, usada por `Page`
//! e pelo parser. É lista plana, não conhece embed, não tem filhos.
//! Unificar os dois é dívida declarada na spec, não descuido.

use serde::{Deserialize, Serialize};

/// O que uma unidade sabe fazer.
///
/// É a "camada extra que traz a intenção" de cada tipo, e é o que a
/// cadeia de responsabilidade (ciclo 262) consulta pra decidir quem
/// trata uma tecla. Declarar em vez de testar a tag é o que tira os
/// `match` sobre `<p>`/`<h1>`/`<ul>` espalhados pelo editor.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Politica {
    /// Recebe digitação direta.
    pub aceita_texto: bool,
    /// Comporta um cursor de texto. Falso num embed: pousar nele é um
    /// REALCE, não um caret (RF3 da spec).
    pub aceita_cursor: bool,
    /// Pode conter outras unidades.
    pub aceita_filhos: bool,
    /// Opera como um todo indivisível na navegação de primeiro nível.
    ///
    /// É o `atom` do `NodeSpec` do ProseMirror: a unidade tem estrutura
    /// interna, mas de fora ela é UM destino. Uma tabela é um bloco;
    /// suas células não são blocos por onde o `j` do documento passa.
    pub atomica: bool,
}

impl Politica {
    /// Texto comum: aceita digitação, cursor, sem filhos.
    pub const TEXTO: Self = Self {
        aceita_texto: true,
        aceita_cursor: true,
        aceita_filhos: false,
        atomica: false,
    };

    /// Embed: estrutura própria, sem caret, atômico de fora.
    pub const EMBED: Self = Self {
        aceita_texto: false,
        aceita_cursor: false,
        aceita_filhos: true,
        atomica: true,
    };

    /// Contêiner de texto que agrupa outras unidades (uma lista e seus
    /// itens): não recebe digitação própria, mas não é atômico — a
    /// navegação DESCE nele.
    pub const GRUPO: Self = Self {
        aceita_texto: false,
        aceita_cursor: false,
        aceita_filhos: true,
        atomica: false,
    };
}

/// O tipo de uma unidade.
///
/// `Embed` carrega o nome do embed (`"kanban"`, `"table"`, …) em vez de
/// uma variante por tipo: o conjunto de embeds cresce, e uma variante
/// por embed obrigaria a mexer no núcleo a cada embed novo — que é
/// exatamente o acoplamento que esta spec existe pra desfazer.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Tipo {
    Paragrafo,
    Titulo(u8),
    Citacao,
    Codigo,
    /// A lista em si (`<ul>`/`<ol>`); os itens são filhos.
    Lista,
    /// Um item de lista.
    Item,
    /// Linha horizontal, imagem — o que ocupa uma linha e não recebe
    /// texto.
    Vazia,
    /// Um embed, pelo nome.
    Embed(String),
}

impl Tipo {
    /// A política deste tipo.
    pub fn politica(&self) -> Politica {
        match self {
            Self::Paragrafo | Self::Titulo(_) | Self::Citacao | Self::Codigo | Self::Item => {
                Politica::TEXTO
            }
            Self::Lista => Politica::GRUPO,
            Self::Vazia => Politica {
                aceita_texto: false,
                aceita_cursor: false,
                aceita_filhos: false,
                atomica: true,
            },
            Self::Embed(_) => Politica::EMBED,
        }
    }
}

/// Endereço de uma unidade na árvore: os índices do caminho da raiz até
/// ela.
///
/// Caminho e não índice plano nem ponteiro: um índice plano muda quando
/// qualquer unidade antes dele muda de tamanho, e um ponteiro pro DOM
/// morre no próximo render — que é a objeção registrada na spec (RNF4).
/// Um caminho continua válido enquanto a ÁRVORE for a verdade.
pub type Caminho = Vec<usize>;

/// Uma unidade da página.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Unidade {
    pub tipo: Tipo,
    /// Texto próprio da unidade. Vazio num grupo ou num embed.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub texto: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub filhos: Vec<Unidade>,
}

impl Unidade {
    pub fn nova(tipo: Tipo) -> Self {
        Self { tipo, texto: String::new(), filhos: Vec::new() }
    }

    pub fn com_texto(tipo: Tipo, texto: impl Into<String>) -> Self {
        Self { tipo, texto: texto.into(), filhos: Vec::new() }
    }

    pub fn com_filhos(tipo: Tipo, filhos: Vec<Unidade>) -> Self {
        Self { tipo, texto: String::new(), filhos }
    }

    pub fn politica(&self) -> Politica {
        self.tipo.politica()
    }

    /// A unidade endereçada por `caminho`, se existir.
    pub fn em(&self, caminho: &[usize]) -> Option<&Unidade> {
        match caminho.split_first() {
            None => Some(self),
            Some((i, resto)) => self.filhos.get(*i)?.em(resto),
        }
    }

    /// O caminho da raiz até `caminho`, unidade por unidade — do mais
    /// externo ao mais interno.
    ///
    /// É a entrada da cadeia de responsabilidade: percorrida ao
    /// contrário, dá a ordem em que uma tecla é oferecida.
    pub fn linhagem(&self, caminho: &[usize]) -> Option<Vec<&Unidade>> {
        let mut fora = vec![self];
        let mut atual = self;
        for i in caminho {
            atual = atual.filhos.get(*i)?;
            fora.push(atual);
        }
        Some(fora)
    }

    /// Todas as unidades, em ordem de documento, com o endereço de cada
    /// uma. Desce em tudo, inclusive no que é atômico.
    ///
    /// Serve pra quem precisa do conteúdo inteiro (serializar, buscar).
    pub fn percorrer(&self) -> Vec<(Caminho, &Unidade)> {
        let mut fora = Vec::new();
        self.juntar(&mut Vec::new(), &mut fora, false);
        fora
    }

    /// As unidades que a navegação de primeiro nível alcança.
    ///
    /// A diferença pro `percorrer`: **não desce em unidade atômica**.
    /// Uma tabela aparece uma vez; as células dela não aparecem. É o que
    /// faz `j` tratar a tabela como um destino só, e é a definição de
    /// bloco que o RF1 pede — a mesma pro nav mode, pro vim e pra
    /// seleção.
    pub fn navegaveis(&self) -> Vec<(Caminho, &Unidade)> {
        let mut fora = Vec::new();
        self.juntar(&mut Vec::new(), &mut fora, true);
        fora
    }

    fn juntar<'a>(
        &'a self,
        caminho: &mut Caminho,
        fora: &mut Vec<(Caminho, &'a Unidade)>,
        parar_no_atomico: bool,
    ) {
        // A raiz é o documento, não uma unidade: ela não é destino de
        // navegação nem aparece na travessia.
        if !caminho.is_empty() {
            fora.push((caminho.clone(), self));
            if parar_no_atomico && self.politica().atomica {
                return;
            }
        }
        for (i, filho) in self.filhos.iter().enumerate() {
            caminho.push(i);
            filho.juntar(caminho, fora, parar_no_atomico);
            caminho.pop();
        }
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// Documento de exemplo: parágrafo, uma tabela (atômica, com
    /// células dentro) e uma lista (grupo, com itens dentro).
    fn documento() -> Unidade {
        Unidade::com_filhos(
            Tipo::Paragrafo, // a raiz; o tipo dela não é usado
            vec![
                Unidade::com_texto(Tipo::Paragrafo, "alfa"),
                Unidade::com_filhos(
                    Tipo::Embed("table".into()),
                    vec![
                        Unidade::com_texto(Tipo::Paragrafo, "célula 1"),
                        Unidade::com_texto(Tipo::Paragrafo, "célula 2"),
                    ],
                ),
                Unidade::com_filhos(
                    Tipo::Lista,
                    vec![
                        Unidade::com_texto(Tipo::Item, "um"),
                        Unidade::com_texto(Tipo::Item, "dois"),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn percorrer_desce_em_tudo() {
        let d = documento();
        let textos: Vec<&str> = d
            .percorrer()
            .iter()
            .map(|(_, u)| u.texto.as_str())
            .collect();
        assert_eq!(
            textos,
            ["alfa", "", "célula 1", "célula 2", "", "um", "dois"],
            "a travessia completa passa pelas células e pelos itens"
        );
    }

    #[test]
    fn a_navegacao_nao_desce_no_atomico() {
        // O RF1 em uma asserção: a tabela é UM destino, e as células
        // dela não são destinos do `j` do documento. A lista, que é
        // grupo e não atômica, continua sendo atravessada.
        let d = documento();
        let tipos: Vec<&Tipo> = d.navegaveis().iter().map(|(_, u)| &u.tipo).collect();
        assert_eq!(
            tipos,
            [
                &Tipo::Paragrafo,
                &Tipo::Embed("table".into()),
                &Tipo::Lista,
                &Tipo::Item,
                &Tipo::Item,
            ]
        );
    }

    #[test]
    fn o_embed_aparece_na_navegacao() {
        // É o bug que motivou a spec: hoje `j` pula a tabela como se
        // ela não existisse.
        let d = documento();
        assert!(
            d.navegaveis()
                .iter()
                .any(|(_, u)| matches!(u.tipo, Tipo::Embed(_))),
            "o embed sumiu da navegação — é o defeito que o RF1 corrige"
        );
    }

    #[test]
    fn o_caminho_enderecca_a_unidade() {
        let d = documento();
        assert_eq!(d.em(&[0]).unwrap().texto, "alfa");
        assert_eq!(d.em(&[1, 1]).unwrap().texto, "célula 2");
        assert_eq!(d.em(&[2, 0]).unwrap().texto, "um");
        assert!(d.em(&[9]).is_none());
        assert!(d.em(&[0, 0]).is_none());
    }

    #[test]
    fn o_caminho_da_travessia_reencontra_a_unidade() {
        // A garantia que a cadeia de responsabilidade vai precisar: o
        // endereço devolvido pela travessia é utilizável.
        let d = documento();
        for (caminho, unidade) in d.percorrer() {
            assert_eq!(
                d.em(&caminho).map(|u| &u.tipo),
                Some(&unidade.tipo),
                "caminho {caminho:?} não reencontrou a unidade"
            );
        }
    }

    #[test]
    fn a_linhagem_vai_de_fora_pra_dentro() {
        let d = documento();
        let linha = d.linhagem(&[1, 0]).unwrap();
        assert_eq!(linha.len(), 3, "raiz, tabela, célula");
        assert_eq!(linha[1].tipo, Tipo::Embed("table".into()));
        assert_eq!(linha[2].texto, "célula 1");
        assert!(d.linhagem(&[1, 9]).is_none());
    }

    #[test]
    fn cada_tipo_declara_a_politica() {
        assert!(Tipo::Paragrafo.politica().aceita_texto);
        assert!(Tipo::Paragrafo.politica().aceita_cursor);
        assert!(!Tipo::Paragrafo.politica().atomica);

        // Um embed não comporta caret: pousar nele é realce (RF3).
        let e = Tipo::Embed("kanban".into()).politica();
        assert!(!e.aceita_cursor);
        assert!(e.atomica);
        assert!(e.aceita_filhos);

        // Lista agrupa mas não é atômica: a navegação desce nela.
        assert!(Tipo::Lista.politica().aceita_filhos);
        assert!(!Tipo::Lista.politica().atomica);
    }

    #[test]
    fn titulo_de_qualquer_nivel_e_texto() {
        for n in 1..=6u8 {
            assert!(Tipo::Titulo(n).politica().aceita_texto, "h{n}");
        }
    }

    #[test]
    fn documento_vazio_nao_tem_navegavel() {
        let d = Unidade::nova(Tipo::Paragrafo);
        assert!(d.navegaveis().is_empty());
        assert!(d.percorrer().is_empty());
        // A raiz é endereçável pelo caminho vazio, mas não é destino.
        assert!(d.em(&[]).is_some());
    }
}
