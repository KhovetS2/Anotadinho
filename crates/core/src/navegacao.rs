//! Onde a navegação ESTÁ, e para onde ela pode ir.
//!
//! O modelo é o de uma árvore n-ária com níveis explícitos: no nível de
//! cima anda-se bloco a bloco; entra-se num bloco e aí anda-se entre o
//! que ele contém; sai-se de volta. É o mesmo desenho de painéis
//! aninhados de um tmux, agora como DADO em vez de comportamento
//! espalhado.
//!
//! ## A regra que evita ficar preso
//!
//! **Movimento nunca muda de nível.** `Proximo` e `Anterior` andam entre
//! irmãos e param na borda; só `Entrar` desce e só `Sair` sobe.
//!
//! Isso corrige um defeito do ciclo 268, onde o movimento DESCIA sozinho
//! quando não havia irmão na direção pedida. O sintoma: andando entre
//! blocos, o cursor caía dentro de um calendário sem ninguém ter pedido
//! — e sair dali exigia saber que existe um Escape. Numa interface de
//! terminal, onde não há mouse pra resgatar, isso é um beco.
//!
//! Descer tem que ser um ATO, não uma consequência de andar.
//!
//! ## Por que no núcleo
//!
//! Porque é a mesma regra pro editor, pros dez embeds e pra uma futura
//! interface de terminal. Ela existir em um lugar só é o que faz os três
//! se comportarem igual sem combinarem nada — e é testável sem DOM.

use crate::unidade::{Caminho, Unidade};

/// A posição da navegação: o caminho da raiz até a unidade em foco.
///
/// O NÍVEL é o comprimento do caminho: `[2]` é o terceiro bloco do
/// documento, `[2, 0]` é o primeiro filho dele. Não há um campo de
/// nível separado de propósito — dois dados dizendo a mesma coisa é
/// convite pra eles discordarem.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cursor {
    pub caminho: Caminho,
}

impl Cursor {
    /// O primeiro bloco do documento.
    pub fn inicio() -> Self {
        Self { caminho: vec![0] }
    }

    pub fn em(caminho: &[usize]) -> Self {
        Self { caminho: caminho.to_vec() }
    }

    /// Quantos níveis abaixo da raiz. `1` é o nível dos blocos.
    pub fn nivel(&self) -> usize {
        self.caminho.len()
    }
}

/// O que se pede à navegação.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Passo {
    /// Irmão seguinte, no MESMO nível.
    Proximo,
    /// Irmão anterior, no MESMO nível.
    Anterior,
    /// Um nível pra dentro: o primeiro filho da unidade em foco.
    Entrar,
    /// Um nível pra fora: a unidade que contém a atual.
    Sair,
}

/// Aplica um passo. `None` quer dizer "não dá" — e não dar é uma
/// resposta legítima: na borda de um nível o cursor FICA, não pula pra
/// outro lugar.
/// O índice vizinho dentro de um nível, ou `None` na borda.
///
/// É a mesma regra de `mover` — na borda o cursor FICA — dita para quem
/// só tem uma lista de destinos, que é o caso da GUI: ela navega por
/// elementos do DOM, não por caminhos da árvore, e antes disso decidia
/// a borda sozinha, com aritmética modular. Documento circulava: `j` no
/// último bloco pulava pro primeiro (ciclo 279).
///
/// Circular continua certo em MENU, onde a lista é curta e fechada.
/// Documento não é menu, e quem escolhe é quem chama — o que esta
/// função garante é que a resposta de "não dá" seja a mesma dos dois
/// lados do modelo.
///
/// `atual` é `None` quando nada está em foco ainda: aí o destino é a
/// primeira unidade, indo pra frente ou pra trás.
pub fn proximo_indice(atual: Option<usize>, total: usize, adiante: bool) -> Option<usize> {
    if total == 0 {
        return None;
    }
    match atual {
        None => Some(0),
        Some(i) if adiante => (i + 1 < total).then_some(i + 1),
        Some(i) => i.checked_sub(1),
    }
}

pub fn mover(raiz: &Unidade, cursor: &Cursor, passo: Passo) -> Option<Cursor> {
    match passo {
        Passo::Proximo => irmao(raiz, cursor, 1),
        Passo::Anterior => irmao(raiz, cursor, -1),
        Passo::Entrar => {
            let atual = raiz.em(&cursor.caminho)?;
            if atual.filhos.is_empty() {
                return None; // não há dentro
            }
            let mut caminho = cursor.caminho.clone();
            caminho.push(0);
            Some(Cursor { caminho })
        }
        Passo::Sair => {
            // Do nível dos blocos não se sobe: acima deles está o
            // documento, que não é destino.
            if cursor.caminho.len() <= 1 {
                return None;
            }
            let mut caminho = cursor.caminho.clone();
            caminho.pop();
            Some(Cursor { caminho })
        }
    }
}

fn irmao(raiz: &Unidade, cursor: &Cursor, delta: isize) -> Option<Cursor> {
    let (ultimo, pai) = cursor.caminho.split_last()?;
    let quantos = raiz.em(pai)?.filhos.len();
    let alvo = (*ultimo as isize).checked_add(delta)?;
    if alvo < 0 || alvo as usize >= quantos {
        return None; // borda do nível: fica onde está
    }
    let mut caminho = pai.to_vec();
    caminho.push(alvo as usize);
    Some(Cursor { caminho })
}

/// Os irmãos do cursor, em ordem — o que ele alcança sem mudar de nível.
///
/// É o que uma interface desenha como "a lista onde você está".
pub fn irmaos<'a>(raiz: &'a Unidade, cursor: &Cursor) -> Vec<&'a Unidade> {
    let Some((_, pai)) = cursor.caminho.split_last() else {
        return Vec::new();
    };
    raiz.em(pai).map(|u| u.filhos.iter().collect()).unwrap_or_default()
}

/// A trilha de onde a navegação está, do documento até o foco.
///
/// Serve pra QUALQUER interface dizer onde a pessoa está — a barra de
/// caminho de uma GUI e a linha de status de um terminal são o mesmo
/// dado desenhado de dois jeitos. Era o pedido: saber em que parte da
/// navegação você está, sem depender de olhar a tela.
pub fn trilha(raiz: &Unidade, cursor: &Cursor) -> Vec<String> {
    let mut fora = Vec::new();
    let mut caminho = Vec::new();
    for i in &cursor.caminho {
        caminho.push(*i);
        match raiz.em(&caminho) {
            Some(u) => fora.push(u.tipo.resumo()),
            None => break,
        }
    }
    fora
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::unidade::Tipo;

    /// Documento: parágrafo, lista com 3 itens, calendário com 2
    /// controles.
    fn doc() -> Unidade {
        Unidade::com_filhos(
            Tipo::Paragrafo,
            vec![
                Unidade::com_texto(Tipo::Paragrafo, "alfa"),
                Unidade::com_filhos(
                    Tipo::Lista,
                    vec![
                        Unidade::com_texto(Tipo::Item, "um"),
                        Unidade::com_texto(Tipo::Item, "dois"),
                        Unidade::com_texto(Tipo::Item, "três"),
                    ],
                ),
                Unidade::com_filhos(
                    Tipo::Embed("calendar".into()),
                    vec![
                        Unidade::com_texto(Tipo::Paragrafo, "anterior"),
                        Unidade::com_texto(Tipo::Paragrafo, "próximo"),
                    ],
                ),
            ],
        )
    }

    #[test]
    fn andar_no_nivel_de_cima_nao_entra_em_nada() {
        // O defeito que este módulo existe pra corrigir: andando entre
        // blocos, o cursor NUNCA cai dentro de um deles.
        let d = doc();
        let mut c = Cursor::inicio();
        for esperado in [vec![1], vec![2]] {
            c = mover(&d, &c, Passo::Proximo).unwrap();
            assert_eq!(c.caminho, esperado);
            assert_eq!(c.nivel(), 1, "o movimento mudou de nível");
        }
    }

    #[test]
    fn na_borda_o_cursor_fica() {
        // Não circula e não desce: `None` é a resposta, e quem chama
        // mantém o cursor. É o que impede o "andei demais e me perdi".
        let d = doc();
        assert_eq!(mover(&d, &Cursor::em(&[0]), Passo::Anterior), None);
        assert_eq!(mover(&d, &Cursor::em(&[2]), Passo::Proximo), None);
    }

    #[test]
    fn no_calendario_o_movimento_tambem_nao_desce() {
        // O exemplo que motivou o ciclo: chegar no calendário andando e
        // continuar andando NÃO pode enfiar o cursor lá dentro.
        let d = doc();
        let c = Cursor::em(&[2]);
        assert_eq!(mover(&d, &c, Passo::Proximo), None);
        assert_eq!(
            mover(&d, &c, Passo::Anterior).unwrap().caminho,
            vec![1],
            "voltou pro bloco anterior, não pra dentro do calendário"
        );
    }

    #[test]
    fn entrar_e_sair_sao_atos_explicitos() {
        let d = doc();
        let bloco = Cursor::em(&[1]); // a lista
        let dentro = mover(&d, &bloco, Passo::Entrar).unwrap();
        assert_eq!(dentro.caminho, vec![1, 0]);
        assert_eq!(dentro.nivel(), 2);

        let fora = mover(&d, &dentro, Passo::Sair).unwrap();
        assert_eq!(fora, bloco, "sair não devolveu ao mesmo bloco");
    }

    #[test]
    fn dentro_de_um_bloco_anda_entre_os_filhos() {
        let d = doc();
        let mut c = mover(&d, &Cursor::em(&[1]), Passo::Entrar).unwrap();
        c = mover(&d, &c, Passo::Proximo).unwrap();
        assert_eq!(c.caminho, vec![1, 1]);
        c = mover(&d, &c, Passo::Proximo).unwrap();
        assert_eq!(c.caminho, vec![1, 2]);
        // E para no último irmão, sem descer nem circular.
        assert_eq!(mover(&d, &c, Passo::Proximo), None);
    }

    #[test]
    fn nao_se_entra_no_que_nao_tem_dentro() {
        let d = doc();
        assert_eq!(mover(&d, &Cursor::em(&[0]), Passo::Entrar), None);
    }

    #[test]
    fn do_nivel_dos_blocos_nao_se_sobe() {
        // Acima dos blocos está o documento, que não é destino. Quem
        // chama decide o que fazer com o `None` — sair do editor, por
        // exemplo.
        let d = doc();
        assert_eq!(mover(&d, &Cursor::em(&[0]), Passo::Sair), None);
    }

    #[test]
    fn a_trilha_diz_onde_voce_esta() {
        let d = doc();
        assert_eq!(trilha(&d, &Cursor::em(&[0])), ["paragrafo"]);
        assert_eq!(trilha(&d, &Cursor::em(&[1, 2])), ["lista", "item"]);
        assert_eq!(
            trilha(&d, &Cursor::em(&[2, 1])),
            ["embed:calendar", "paragrafo"]
        );
    }

    #[test]
    fn os_irmaos_sao_o_nivel_onde_voce_esta() {
        let d = doc();
        assert_eq!(irmaos(&d, &Cursor::em(&[1])).len(), 3, "três blocos");
        assert_eq!(irmaos(&d, &Cursor::em(&[1, 0])).len(), 3, "três itens");
        assert_eq!(irmaos(&d, &Cursor::em(&[2, 0])).len(), 2, "dois controles");
    }

    #[test]
    fn caminho_invalido_nao_derruba_nada() {
        let d = doc();
        assert_eq!(mover(&d, &Cursor::em(&[9]), Passo::Entrar), None);
        assert_eq!(mover(&d, &Cursor::em(&[9, 9]), Passo::Proximo), None);
        assert!(trilha(&d, &Cursor::em(&[9])).is_empty());
        assert!(irmaos(&d, &Cursor::em(&[])).is_empty());
    }

    #[test]
    fn descer_e_subir_varios_niveis_volta_ao_mesmo_lugar() {
        // A propriedade que uma árvore n-ária precisa ter pra navegação
        // não se perder: o caminho de volta é o de ida invertido.
        let d = doc();
        let partida = Cursor::em(&[2]);
        let dentro = mover(&d, &partida, Passo::Entrar).unwrap();
        let mais = mover(&d, &dentro, Passo::Proximo).unwrap();
        let volta = mover(&d, &mais, Passo::Sair).unwrap();
        assert_eq!(volta, partida);
    }
    #[test]
    fn a_borda_do_nivel_segura_o_indice() {
        // Quatro destinos. No fim, ir adiante não dá; no começo, voltar
        // não dá. Era aqui que a GUI circulava.
        assert_eq!(proximo_indice(Some(3), 4, true), None);
        assert_eq!(proximo_indice(Some(0), 4, false), None);
        // No meio, anda normal.
        assert_eq!(proximo_indice(Some(1), 4, true), Some(2));
        assert_eq!(proximo_indice(Some(1), 4, false), Some(0));
    }

    #[test]
    fn sem_foco_o_destino_e_o_primeiro() {
        assert_eq!(proximo_indice(None, 4, true), Some(0));
        assert_eq!(proximo_indice(None, 4, false), Some(0));
    }

    #[test]
    fn nivel_vazio_nao_tem_destino() {
        assert_eq!(proximo_indice(None, 0, true), None);
        assert_eq!(proximo_indice(Some(0), 0, true), None);
    }

    #[test]
    fn um_destino_so_nao_vai_a_lugar_nenhum() {
        // O caso que a aritmética modular acertava por acidente:
        // `(0 + 1) % 1` é 0, então parecia certo. Com dois destinos ela
        // já errava.
        assert_eq!(proximo_indice(Some(0), 1, true), None);
        assert_eq!(proximo_indice(Some(0), 1, false), None);
    }
}
