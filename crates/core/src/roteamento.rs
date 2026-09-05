//! Quem trata a tecla: a cadeia de responsabilidade.
//!
//! O modelo pedido na spec `o-que-e-um-bloco` é o de painéis aninhados
//! de um tmux: o foco desce e sobe, e a tecla vai pra unidade mais
//! INTERNA que souber o que fazer com ela. Ninguém "desliga o vim pra
//! usar o calendário" — o calendário responde ao que faz sentido nele, e
//! o resto sobe.
//!
//! Isso é **Chain of Responsibility**, e juntá-lo ao Composite é o
//! pareamento que o próprio catálogo documenta: os componentes-folha
//! passam a requisição pelos pais até a raiz.
//!
//! ## A regra em uma frase
//!
//! Percorre a linhagem de dentro pra fora e para na primeira unidade
//! cuja política declara interesse na CATEGORIA daquela tecla. Se
//! ninguém declara, o documento trata — que é o comportamento de hoje.
//!
//! ## Por que categoria, e não tecla
//!
//! Declarar tecla a tecla faria cada embed novo listar dezenas delas, e
//! o vim ganha comando a cada ciclo. A categoria é estável: um
//! calendário quer os MOVIMENTOS (pra andar entre dias) e não quer os
//! OPERADORES (`d`, `y`, `c` continuam sendo do documento, e é por isso
//! que `dd` num calendário apaga o calendário).
//!
//! Zero DOM, como o `unidade`: a regra é testável sem navegador, e a
//! mesma cadeia serve pro terminal.

use serde::{Deserialize, Serialize};

use crate::unidade::Unidade;

/// A categoria de uma tecla, do ponto de vista de quem pode tratá-la.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interesse {
    /// Andar: `h j k l`, setas, `w b e`, `0 $`.
    Movimento,
    /// Entrar em edição: `i a o O`.
    Edicao,
    /// Agir sobre uma extensão: `d y c`, e o que os acompanha.
    Operador,
    /// Confirmar/abrir: Enter, Espaço.
    Ativacao,
    /// Sair um nível: Escape.
    Saida,
}

impl Interesse {
    /// A categoria de uma tecla, pelo nome do `KeyboardEvent.key`.
    ///
    /// Existe pra uma unidade poder declarar interesse sem listar tecla
    /// por tecla — o vim ganha comando a cada ciclo, e a categoria é
    /// estável. `None` quer dizer "esta tecla não pertence a nenhuma
    /// categoria roteável": ela segue o caminho de sempre.
    ///
    /// Só as teclas do modo NORMAL entram aqui. Letras que só significam
    /// algo em inserção são digitação, não comando.
    pub fn da_tecla(tecla: &str) -> Option<Self> {
        Some(match tecla {
            "h" | "j" | "k" | "l" | "w" | "b" | "e" | "0" | "$" | "G" | "ArrowLeft"
            | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" | "PageUp"
            | "PageDown" => Self::Movimento,
            "i" | "a" | "o" | "O" | "I" | "A" => Self::Edicao,
            "d" | "y" | "c" | "x" | "D" | "C" | "Y" => Self::Operador,
            "Enter" | " " => Self::Ativacao,
            "Escape" => Self::Saida,
            _ => return None,
        })
    }
}

/// O que uma unidade declara consumir.
///
/// Vazio (o padrão) quer dizer "não trato nada" — e é justamente o que
/// mantém um embed ainda não migrado funcionando como hoje: tudo sobe.
/// É o que torna a migração dos dez incremental de verdade.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Interesses(pub Vec<Interesse>);

impl Interesses {
    pub fn nenhum() -> Self {
        Self(Vec::new())
    }

    pub fn de(lista: &[Interesse]) -> Self {
        Self(lista.to_vec())
    }

    pub fn quer(&self, i: Interesse) -> bool {
        self.0.contains(&i)
    }
}

/// Onde a tecla foi parar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destino {
    /// Tratada pela unidade neste índice da linhagem (0 = a raiz/documento).
    Unidade(usize),
    /// Ninguém declarou interesse: é do documento.
    Documento,
}

/// Resolve quem trata `interesse`, dada a linhagem da raiz até a unidade
/// focada e o que cada uma declara.
///
/// `declara` recebe a unidade e devolve os interesses dela. Entra por
/// parâmetro (e não como método de `Unidade`) porque o que um embed
/// consome é decisão do COMPONENTE que o desenha, não do tipo: um
/// calendário e uma tabela são os dois `Tipo::Embed`, e querem teclas
/// diferentes.
pub fn rotear(
    linhagem: &[&Unidade],
    interesse: Interesse,
    declara: impl Fn(&Unidade) -> Interesses,
) -> Destino {
    // De dentro pra fora: a unidade mais interna tem a primeira chance.
    // Índice 0 é a raiz, e ela não participa — o documento é o fallback
    // explícito, não um elo da cadeia.
    for i in (1..linhagem.len()).rev() {
        if declara(linhagem[i]).quer(interesse) {
            return Destino::Unidade(i);
        }
    }
    Destino::Documento
}

/// Uma unidade em edição de TEXTO consome tudo.
///
/// É a proteção que já existe hoje por acidente de implementação (o
/// handler do vim está preso ao contêiner de texto) e que precisa
/// sobreviver ao modelo: digitar `dd` no título de um card não pode
/// apagar o bloco. Com o texto em edição no topo da linhagem, nenhuma
/// tecla sobe.
pub const EM_EDICAO: &[Interesse] = &[
    Interesse::Movimento,
    Interesse::Edicao,
    Interesse::Operador,
    Interesse::Ativacao,
];

#[cfg(test)]
mod testes {
    use super::*;
    use crate::unidade::Tipo;

    fn doc() -> Unidade {
        Unidade::com_filhos(
            Tipo::Paragrafo,
            vec![
                Unidade::com_texto(Tipo::Paragrafo, "alfa"),
                Unidade::com_filhos(
                    Tipo::Embed("calendar".into()),
                    vec![Unidade::com_texto(Tipo::Paragrafo, "um dia")],
                ),
            ],
        )
    }

    /// Um calendário quer andar entre dias, e nada mais.
    fn calendario(u: &Unidade) -> Interesses {
        match &u.tipo {
            Tipo::Embed(nome) if nome == "calendar" => Interesses::de(&[Interesse::Movimento]),
            _ => Interesses::nenhum(),
        }
    }

    #[test]
    fn o_calendario_trata_o_movimento() {
        // `j` dentro do calendário anda um dia: quem trata é o embed.
        let d = doc();
        let linhagem = d.linhagem(&[1]).unwrap();
        assert_eq!(
            rotear(&linhagem, Interesse::Movimento, calendario),
            Destino::Unidade(1)
        );
    }

    #[test]
    fn o_operador_sobe_e_o_documento_apaga_o_embed() {
        // `dd` no calendário: ele não declara Operador, então sobe. É
        // isto que faz `dd` apagar o bloco do calendário em vez de não
        // fazer nada — e é a resposta pro "vim no que for cabível".
        let d = doc();
        let linhagem = d.linhagem(&[1]).unwrap();
        assert_eq!(
            rotear(&linhagem, Interesse::Operador, calendario),
            Destino::Documento
        );
    }

    #[test]
    fn a_unidade_mais_interna_ganha() {
        // Calendário E célula querendo movimento: a célula, que está
        // mais dentro, trata.
        let d = doc();
        let linhagem = d.linhagem(&[1, 0]).unwrap();
        let ambos = |_: &Unidade| Interesses::de(&[Interesse::Movimento]);
        assert_eq!(
            rotear(&linhagem, Interesse::Movimento, ambos),
            Destino::Unidade(2),
            "a mais interna da linhagem tem a primeira chance"
        );
    }

    #[test]
    fn embed_que_nao_declara_nada_se_comporta_como_hoje() {
        // A garantia da migração incremental: um embed ainda não
        // migrado não declara interesse, tudo sobe, e o comportamento é
        // o de antes do modelo.
        let d = doc();
        let linhagem = d.linhagem(&[1]).unwrap();
        for i in [
            Interesse::Movimento,
            Interesse::Edicao,
            Interesse::Operador,
            Interesse::Ativacao,
            Interesse::Saida,
        ] {
            assert_eq!(
                rotear(&linhagem, i, |_| Interesses::nenhum()),
                Destino::Documento,
                "{i:?} devia ter subido"
            );
        }
    }

    #[test]
    fn texto_em_edicao_engole_tudo() {
        // Digitar `dd` num campo não pode apagar o bloco.
        let d = doc();
        let linhagem = d.linhagem(&[1, 0]).unwrap();
        let em_edicao = |u: &Unidade| {
            if u.politica().aceita_texto {
                Interesses::de(EM_EDICAO)
            } else {
                Interesses::nenhum()
            }
        };
        assert_eq!(
            rotear(&linhagem, Interesse::Operador, em_edicao),
            Destino::Unidade(2),
            "o operador não podia ter subido de um campo em edição"
        );
    }

    #[test]
    fn a_raiz_nao_e_elo_da_cadeia() {
        // Mesmo que a raiz declare tudo, quem responde é o `Documento`
        // — o fallback é explícito, não um elo que calhou de estar no
        // fim da lista.
        let d = doc();
        let linhagem = d.linhagem(&[]).unwrap();
        assert_eq!(linhagem.len(), 1, "só a raiz");
        assert_eq!(
            rotear(&linhagem, Interesse::Movimento, |_| Interesses::de(&[
                Interesse::Movimento
            ])),
            Destino::Documento
        );
    }

    #[test]
    fn a_tecla_vira_categoria() {
        use Interesse::*;
        for (tecla, esperado) in [
            ("j", Movimento),
            ("k", Movimento),
            ("ArrowDown", Movimento),
            ("w", Movimento),
            ("i", Edicao),
            ("o", Edicao),
            ("d", Operador),
            ("y", Operador),
            ("Enter", Ativacao),
            (" ", Ativacao),
            ("Escape", Saida),
        ] {
            assert_eq!(Interesse::da_tecla(tecla), Some(esperado), "tecla {tecla:?}");
        }
    }

    #[test]
    fn tecla_sem_categoria_nao_e_roteavel() {
        // Uma letra que só significa algo em inserção é digitação, não
        // comando — e não pode ser roteada como se fosse.
        for tecla in ["z", "1", "F5", "Tab", "Shift"] {
            assert_eq!(Interesse::da_tecla(tecla), None, "tecla {tecla:?}");
        }
    }

    #[test]
    fn um_calendario_quer_movimento_e_nao_quer_operador() {
        // A declaração que o embed de calendário faz (ciclo 267), como
        // teste puro: `j` é dele, `d` não é.
        let cal = Interesses::de(&[Interesse::Movimento]);
        assert!(cal.quer(Interesse::da_tecla("j").unwrap()));
        assert!(cal.quer(Interesse::da_tecla("ArrowUp").unwrap()));
        assert!(!cal.quer(Interesse::da_tecla("d").unwrap()));
        assert!(!cal.quer(Interesse::da_tecla("Escape").unwrap()));
    }

    #[test]
    fn interesses_vazio_nao_quer_nada() {
        assert!(!Interesses::nenhum().quer(Interesse::Movimento));
        assert!(Interesses::de(&[Interesse::Saida]).quer(Interesse::Saida));
        assert!(!Interesses::de(&[Interesse::Saida]).quer(Interesse::Edicao));
    }
}
