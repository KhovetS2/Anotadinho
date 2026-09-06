//! Editar a árvore (ciclo 285).
//!
//! Até aqui a árvore sabia LER (`analise::analisar`) e ESCREVER
//! (`analise::escrever_costurando`), e não sabia mudar. Editar era
//! digitar no `contenteditable` e reconstruir o markdown a partir do
//! DOM — o que amarra a edição a um navegador e deixa o porte pro
//! terminal sem chão.
//!
//! As operações aqui são as do editor de texto, ditas sobre a árvore:
//! escrever num bloco, dividir um bloco no meio, juntar com o anterior,
//! inserir e remover. Zero DOM.
//!
//! ## A regra que faz tudo isto funcionar com a costura
//!
//! `escrever_costurando` (ciclo 276) devolve pelos BYTES ORIGINAIS toda
//! unidade que ainda tem `fonte`, e só reserializa quem não tem. Os dois
//! campos respondem perguntas diferentes:
//!
//! - `intervalo` diz ONDE a unidade estava — serve pra posicionar;
//! - `fonte` diz SE ela continua como estava.
//!
//! Então **editar apaga a `fonte` e preserva o `intervalo`**. E apaga a
//! fonte de toda a LINHAGEM, não só da unidade tocada: a fonte de um
//! ancestral cobre a subárvore inteira, e a escrita para de descer
//! quando encontra uma. Editar um item de lista sem limpar a fonte da
//! lista devolveria o texto velho do item, calado.

use crate::unidade::{Caminho, Tipo, Unidade};

/// Apaga a `fonte` da unidade em `caminho` e de todos os ancestrais.
///
/// É o que separa "a árvore mudou" de "o arquivo vai mudar". Sem isto a
/// edição acontece na árvore e some na hora de gravar.
fn marcar_tocada(raiz: &mut Unidade, caminho: &[usize]) {
    raiz.fonte = None;
    let Some((i, resto)) = caminho.split_first() else {
        return;
    };
    if let Some(filho) = raiz.filhos.get_mut(*i) {
        marcar_tocada(filho, resto);
    }
}

/// Troca o texto de uma unidade. `false` se o caminho não existe.
pub fn escrever_texto(raiz: &mut Unidade, caminho: &Caminho, texto: impl Into<String>) -> bool {
    let Some(u) = raiz.em_mut(caminho) else {
        return false;
    };
    u.texto = texto.into();
    marcar_tocada(raiz, caminho);
    true
}

/// Troca o tipo de uma unidade — o que `##` no começo da linha faz.
pub fn trocar_tipo(raiz: &mut Unidade, caminho: &Caminho, tipo: Tipo) -> bool {
    let Some(u) = raiz.em_mut(caminho) else {
        return false;
    };
    u.tipo = tipo;
    marcar_tocada(raiz, caminho);
    true
}

/// Insere `nova` logo DEPOIS da unidade em `caminho`.
///
/// Devolve o caminho da unidade inserida, que é onde o cursor vai.
pub fn inserir_depois(raiz: &mut Unidade, caminho: &Caminho, nova: Unidade) -> Option<Caminho> {
    let (i, pai_caminho) = posicao(caminho)?;
    let pai = raiz.em_mut(&pai_caminho)?;
    if i + 1 > pai.filhos.len() {
        return None;
    }
    pai.filhos.insert(i + 1, nova);
    let mut destino = pai_caminho.clone();
    destino.push(i + 1);
    marcar_tocada(raiz, &pai_caminho);
    Some(destino)
}

/// Tira a unidade em `caminho` da árvore e devolve ela.
pub fn remover(raiz: &mut Unidade, caminho: &Caminho) -> Option<Unidade> {
    let (i, pai_caminho) = posicao(caminho)?;
    let pai = raiz.em_mut(&pai_caminho)?;
    if i >= pai.filhos.len() {
        return None;
    }
    let fora = pai.filhos.remove(i);
    marcar_tocada(raiz, &pai_caminho);
    Some(fora)
}

/// Divide a unidade em `caminho` no byte `em` — o Enter no meio de um
/// bloco.
///
/// Devolve o caminho da metade NOVA, que é onde o cursor fica.
///
/// O tipo da metade nova é o mesmo, com uma exceção que a janela já
/// praticava: Enter no FIM de um título começa texto comum, porque é o
/// que se espera de quem acabou de escrever um título.
///
/// `None` se o caminho não existe, se `em` passa do fim, ou se cai no
/// MEIO de um caractere — "ção" tem 4 bytes em 3 letras, e cortar entre
/// eles produziria texto inválido.
pub fn dividir(raiz: &mut Unidade, caminho: &Caminho, em: usize) -> Option<Caminho> {
    let u = raiz.em_mut(caminho)?;
    if em > u.texto.len() || !u.texto.is_char_boundary(em) {
        return None;
    }
    let resto = u.texto.split_off(em);
    let tipo = if resto.trim().is_empty() && matches!(u.tipo, Tipo::Titulo(_)) {
        Tipo::Paragrafo
    } else {
        u.tipo.clone()
    };
    // A metade nova nasce SEM intervalo: ela não estava no arquivo, então
    // não há byte original pra devolver e a costura vai serializá-la.
    let nova = Unidade::com_texto(tipo, resto);
    marcar_tocada(raiz, caminho);
    inserir_depois(raiz, caminho, nova)
}

/// Junta a unidade em `caminho` com a anterior — o Backspace no começo
/// de um bloco.
///
/// Devolve o caminho de quem sobrou e o byte onde o cursor fica: a
/// emenda, que é o fim do texto que já estava lá.
///
/// `None` quando não há anterior (a primeira unidade de um nível não
/// tem com quem juntar) ou quando alguma das duas comporta filhos —
/// juntar uma lista com um parágrafo não é uma emenda de texto, é outra
/// operação.
pub fn juntar_com_anterior(raiz: &mut Unidade, caminho: &Caminho) -> Option<(Caminho, usize)> {
    let (i, pai_caminho) = posicao(caminho)?;
    if i == 0 {
        return None;
    }
    let pai = raiz.em_mut(&pai_caminho)?;
    let atual = pai.filhos.get(i)?;
    let anterior = pai.filhos.get(i - 1)?;
    if anterior.politica().aceita_filhos || atual.politica().aceita_filhos {
        return None;
    }
    let emenda = anterior.texto.len();
    let texto_atual = atual.texto.clone();
    pai.filhos.remove(i);
    pai.filhos[i - 1].texto.push_str(&texto_atual);

    let mut destino = pai_caminho.clone();
    destino.push(i - 1);
    marcar_tocada(raiz, &destino);
    Some((destino, emenda))
}

/// Quebra um caminho em (índice na lista do pai, caminho do pai).
fn posicao(caminho: &Caminho) -> Option<(usize, Caminho)> {
    let (i, pai) = caminho.split_last()?;
    Some((*i, pai.to_vec()))
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::analise::{analisar, escrever_costurando};

    const PAGINA: &str = "Primeiro parágrafo.\n\n## Um título\n\n- um\n- dois\n\nFim.\n";

    #[test]
    fn escrever_num_bloco_chega_no_arquivo() {
        let mut d = analisar(PAGINA);
        assert!(escrever_texto(&mut d, &vec![0], "Trocado."));
        let fora = escrever_costurando(PAGINA, &d);
        assert!(fora.contains("Trocado."), "{fora}");
        assert!(!fora.contains("Primeiro parágrafo."), "{fora}");
        // E o resto volta intocado.
        assert!(fora.contains("## Um título"), "{fora}");
        assert!(fora.contains("Fim."), "{fora}");
    }

    #[test]
    fn editar_um_item_limpa_a_fonte_da_lista_inteira() {
        // O invariante que faz a edição profunda funcionar: a escrita
        // para de descer quando acha uma `fonte`, então a da LISTA
        // precisa sumir junto com a do item. Sem isto o item volta com o
        // texto velho e ninguém avisa.
        let mut d = analisar(PAGINA);
        let lista = 2usize;
        assert!(d.filhos[lista].fonte.is_some(), "a lista devia ter fonte");

        assert!(escrever_texto(&mut d, &vec![lista, 1], "DOIS EDITADO"));

        // O COMPORTAMENTO primeiro: a edição tem que chegar no arquivo.
        let fora = escrever_costurando(PAGINA, &d);
        assert!(
            fora.contains("DOIS EDITADO"),
            "o item editado voltou com o texto velho:\n{fora}"
        );

        // E o invariante que explica por quê.
        assert!(d.filhos[lista].fonte.is_none(), "a fonte da lista ficou");
        assert!(d.filhos[lista].filhos[1].fonte.is_none());
        // O vizinho não foi tocado.
        assert!(d.filhos[0].fonte.is_some());
    }

    #[test]
    fn o_intervalo_sobrevive_a_edicao() {
        // `fonte` some, `intervalo` fica: é ele que diz ONDE trocar.
        let mut d = analisar(PAGINA);
        let antes = d.filhos[0].intervalo.clone();
        assert!(antes.is_some());
        escrever_texto(&mut d, &vec![0], "outro");
        assert_eq!(d.filhos[0].intervalo, antes);
    }

    #[test]
    fn dividir_no_meio_da_dois_blocos_do_mesmo_tipo() {
        let mut d = analisar(PAGINA);
        let novo = dividir(&mut d, &vec![0], "Primeiro ".len()).unwrap();
        assert_eq!(novo, vec![1]);
        assert_eq!(d.filhos[0].texto, "Primeiro ");
        assert_eq!(d.filhos[1].texto, "parágrafo.");
        assert_eq!(d.filhos[1].tipo, Tipo::Paragrafo);

        let fora = escrever_costurando(PAGINA, &d);
        assert!(fora.contains("Primeiro"), "{fora}");
        assert!(fora.contains("parágrafo."), "{fora}");
    }

    #[test]
    fn enter_no_fim_de_um_titulo_comeca_texto_comum() {
        // A regra que a janela já praticava: dividir um título no meio dá
        // dois títulos, mas no FIM dele o natural é escrever parágrafo.
        let mut d = analisar(PAGINA);
        let titulo = 1usize;
        assert_eq!(d.filhos[titulo].tipo, Tipo::Titulo(2));

        let fim = d.filhos[titulo].texto.len();
        let novo = dividir(&mut d, &vec![titulo], fim).unwrap();
        assert_eq!(d.em(&novo).unwrap().tipo, Tipo::Paragrafo);

        // E no MEIO continua título.
        let mut d2 = analisar(PAGINA);
        let novo2 = dividir(&mut d2, &vec![titulo], "Um ".len()).unwrap();
        assert_eq!(d2.em(&novo2).unwrap().tipo, Tipo::Titulo(2));
    }

    #[test]
    fn dividir_no_meio_de_um_caractere_nao_acontece() {
        // "ção" tem 4 bytes em 3 letras. Cortar entre eles produziria
        // texto inválido, e devolver `None` é a resposta certa.
        let md = "ação\n";
        let mut d = analisar(md);
        assert_eq!(dividir(&mut d, &vec![0], 2), None);
        assert_eq!(d.filhos[0].texto, "ação", "a árvore foi mexida mesmo assim");
        // No limite certo, divide.
        let mut d2 = analisar(md);
        assert!(dividir(&mut d2, &vec![0], 1).is_some());
    }

    #[test]
    fn juntar_com_o_anterior_emenda_o_texto_e_diz_onde_o_cursor_fica() {
        let mut d = analisar("um\n\ndois\n");
        let (destino, emenda) = juntar_com_anterior(&mut d, &vec![1]).unwrap();
        assert_eq!(destino, vec![0]);
        assert_eq!(emenda, "um".len());
        assert_eq!(d.filhos.len(), 1);
        assert_eq!(d.filhos[0].texto, "umdois");
    }

    #[test]
    fn a_primeira_unidade_nao_tem_com_quem_juntar() {
        let mut d = analisar("um\n\ndois\n");
        assert_eq!(juntar_com_anterior(&mut d, &vec![0]), None);
        assert_eq!(d.filhos.len(), 2, "a árvore mudou numa operação recusada");
    }

    #[test]
    fn juntar_nao_mistura_um_grupo_com_um_paragrafo() {
        // Emendar uma lista num parágrafo não é emenda de texto, é outra
        // operação — e fazer a coisa errada calada é pior que recusar.
        let mut d = analisar(PAGINA);
        assert_eq!(juntar_com_anterior(&mut d, &vec![2]), None);
        assert_eq!(d.filhos.len(), 4);
    }

    #[test]
    fn inserir_e_remover_mexem_no_lugar_certo() {
        let mut d = analisar(PAGINA);
        let onde = inserir_depois(
            &mut d,
            &vec![0],
            Unidade::com_texto(Tipo::Paragrafo, "no meio"),
        )
        .unwrap();
        assert_eq!(onde, vec![1]);
        assert_eq!(d.filhos[1].texto, "no meio");
        assert_eq!(d.filhos.len(), 5);

        let fora = escrever_costurando(PAGINA, &d);
        assert!(fora.contains("no meio"), "{fora}");
        assert!(fora.contains("## Um título"), "{fora}");

        let tirado = remover(&mut d, &vec![1]).unwrap();
        assert_eq!(tirado.texto, "no meio");
        assert_eq!(d.filhos.len(), 4);
    }

    #[test]
    fn caminho_que_nao_existe_nao_muda_nada() {
        let mut d = analisar(PAGINA);
        let antes = d.clone();
        assert!(!escrever_texto(&mut d, &vec![99], "x"));
        assert!(!trocar_tipo(&mut d, &vec![99], Tipo::Paragrafo));
        assert_eq!(remover(&mut d, &vec![99]), None);
        assert_eq!(dividir(&mut d, &vec![99], 0), None);
        assert_eq!(d, antes);
    }

    #[test]
    fn trocar_o_tipo_reserializa_o_bloco() {
        let mut d = analisar(PAGINA);
        assert!(trocar_tipo(&mut d, &vec![0], Tipo::Titulo(1)));
        let fora = escrever_costurando(PAGINA, &d);
        assert!(fora.contains("# Primeiro parágrafo."), "{fora}");
    }
}
