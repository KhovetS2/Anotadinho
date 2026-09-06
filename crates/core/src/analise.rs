//! `markdown → Unidade`: o corpo de uma página vira a árvore.
//!
//! Primeiro passo da unificação dos dois modelos de bloco (ciclo 271).
//! Hoje o markdown vira HTML direto e a estrutura só existe no DOM; aqui
//! ela passa a existir como DADO, antes de qualquer tela.
//!
//! ## O que este módulo é e o que não é
//!
//! Não substitui `markdown_render` nem `embed::segment` — reusa o
//! segundo. O que ele acrescenta é a divisão do TEXTO em unidades, que
//! era justamente o pedaço que só existia depois de virar DOM: para
//! `embed::segment`, um trecho de markdown é uma `String` opaca.
//!
//! Ninguém consome ainda, de propósito. É o mesmo desenho dos ciclos 261
//! e 262 — acertar o vocabulário com teste puro antes de o DOM entrar na
//! conversa.
//!
//! ## O que fica de fora, por ora
//!
//! Marcas inline (negrito, link, código) continuam dentro do `texto` da
//! unidade, como markdown. Modelá-las é outra árvore, e a spec pede a de
//! BLOCOS primeiro — `dd` e `j` não olham para dentro da linha.

use crate::embed::{self, DocSegment};
use crate::unidade::{Tipo, Unidade};

/// Transforma o corpo de uma página na árvore de unidades.
///
/// A raiz é o documento; os filhos são as unidades de primeiro nível.
pub fn analisar(corpo: &str) -> Unidade {
    let mut filhos = Vec::new();
    for seg in embed::segment(corpo) {
        match seg {
            DocSegment::Embed(dados) => {
                // O embed é UMA unidade atômica. O conteúdo dele não é
                // reanalisado como markdown: ele tem estrutura própria,
                // e é o componente que a conhece.
                // `to_fence_text` NORMALIZA o YAML (reordena campos,
                // completa padrões). O editor já faz isso ao salvar
                // desde sempre, então não é perda nova — mas guardar a
                // fonte aqui também não custa, e mantém a árvore fiel
                // pra quem só lê.
                let texto = dados.to_fence_text();
                filhos.push(
                    Unidade::com_texto(
                        Tipo::Embed(dados.kind().type_name().to_string()),
                        texto.clone(),
                    )
                    .da_fonte(texto),
                );
            }
            DocSegment::Markdown(texto) => filhos.extend(unidades_de_texto(&texto)),
        }
    }
    Unidade::com_filhos(Tipo::Paragrafo, filhos)
}

/// Divide um trecho de markdown puro em unidades.
fn unidades_de_texto(texto: &str) -> Vec<Unidade> {
    let mut fora: Vec<Unidade> = Vec::new();
    let mut paragrafo: Vec<&str> = Vec::new();
    let mut lista: Vec<Unidade> = Vec::new();
    let mut lista_ordenada = false;
    let mut linhas_da_lista: Vec<&str> = Vec::new();
    let mut citacao: Vec<&str> = Vec::new();
    let mut linhas_da_citacao: Vec<&str> = Vec::new();
    let mut codigo: Option<(Option<String>, Vec<&str>, Vec<&str>)> = None;

    // Fecha o que estiver aberto antes de começar outra coisa.
    macro_rules! fechar_paragrafo {
        () => {
            if !paragrafo.is_empty() {
                let cru = paragrafo.join("\n");
                fora.push(Unidade::com_texto(Tipo::Paragrafo, cru.clone()).da_fonte(cru));
                paragrafo.clear();
            }
        };
    }
    macro_rules! fechar_lista {
        () => {
            if !lista.is_empty() {
                let tipo = if lista_ordenada { Tipo::ListaOrdenada } else { Tipo::Lista };
                let cru = std::mem::take(&mut linhas_da_lista).join("\n");
                fora.push(
                    Unidade::com_filhos(tipo, std::mem::take(&mut lista)).da_fonte(cru),
                );
            }
        };
    }
    // Linhas `>` seguidas são UMA citação, não uma por linha. Emitir uma
    // por linha inseria linha em branco entre elas, e o markdown que
    // saía já não era o que entrou (ciclo 274).
    macro_rules! fechar_citacao {
        () => {
            if !citacao.is_empty() {
                let cru = std::mem::take(&mut linhas_da_citacao).join("\n");
                fora.push(
                    Unidade::com_texto(Tipo::Citacao, citacao.join("\n")).da_fonte(cru),
                );
                citacao.clear();
            }
        };
    }

    for linha in texto.lines() {
        // Dentro de uma cerca, TUDO é conteúdo — inclusive linhas que
        // pareceriam título ou item. É a razão de o código ser tratado
        // antes de qualquer outro teste.
        if let Some((lingua, acumulado, cru)) = &mut codigo {
            cru.push(linha);
            if linha.trim_start().starts_with("```") {
                let fonte = cru.join("\n");
                fora.push(
                    Unidade::com_texto(Tipo::Codigo(lingua.clone()), acumulado.join("\n"))
                        .da_fonte(fonte),
                );
                codigo = None;
            } else {
                acumulado.push(linha);
            }
            continue;
        }
        if let Some(resto) = linha.trim_start().strip_prefix("```") {
            fechar_paragrafo!();
            fechar_citacao!();
            fechar_lista!();
            let lingua = resto.trim();
            codigo = Some((
                (!lingua.is_empty()).then(|| lingua.to_string()),
                Vec::new(),
                vec![linha],
            ));
            continue;
        }

        let sem_espaco = linha.trim();
        if sem_espaco.is_empty() {
            fechar_paragrafo!();
            fechar_citacao!();
            fechar_lista!();
            continue;
        }

        if let Some(nivel) = nivel_de_titulo(sem_espaco) {
            fechar_paragrafo!();
            fechar_citacao!();
            fechar_lista!();
            let texto = sem_espaco[nivel as usize..].trim_start().to_string();
            fora.push(Unidade::com_texto(Tipo::Titulo(nivel), texto).da_fonte(linha));
            continue;
        }

        // A régua vem ANTES da lista: `- - -` é régua, não um item cujo
        // texto é "- -".
        if e_linha_horizontal(sem_espaco) {
            fechar_paragrafo!();
            fechar_citacao!();
            fechar_lista!();
            fora.push(Unidade::nova(Tipo::Vazia).da_fonte(linha));
            continue;
        }

        if let Some((item, ordenada)) = item_de_lista(sem_espaco) {
            fechar_paragrafo!();
            fechar_citacao!();
            if lista.is_empty() {
                lista_ordenada = ordenada;
            }
            linhas_da_lista.push(linha);
            lista.push(Unidade::com_texto(Tipo::Item, item.to_string()).da_fonte(linha));
            continue;
        }

        if let Some(linha_citada) = sem_espaco.strip_prefix('>') {
            fechar_paragrafo!();
            fechar_lista!();
            citacao.push(linha_citada.strip_prefix(' ').unwrap_or(linha_citada));
            linhas_da_citacao.push(linha);
            continue;
        }
        fechar_citacao!();

        fechar_lista!();
        paragrafo.push(linha);
    }

    // Cerca que o arquivo não fechou: o conteúdo não pode sumir.
    if let Some((lingua, acumulado, cru)) = codigo {
        let fonte = cru.join("\n");
        fora.push(
            Unidade::com_texto(Tipo::Codigo(lingua), acumulado.join("\n")).da_fonte(fonte),
        );
    }
    fechar_paragrafo!();
    fechar_citacao!();
    fechar_lista!();
    fora
}

/// `1..=6` quando a linha é um título ATX, e o número é quantos `#`.
fn nivel_de_titulo(linha: &str) -> Option<u8> {
    let cerquilhas = linha.chars().take_while(|c| *c == '#').count();
    // `#hashtag` não é título: o markdown exige o espaço.
    if (1..=6).contains(&cerquilhas) && linha[cerquilhas..].starts_with(' ') {
        Some(cerquilhas as u8)
    } else {
        None
    }
}

/// O texto do item, quando a linha é item de lista.
///
/// Um marcador SOZINHO (`-`) é item de lista vazio, e isso não é
/// curiosidade de especificação: quatro páginas do vault real têm
/// exatamente isso no corpo, e sem esta regra elas voltavam da ida e
/// volta como parágrafo `-`. Foi o teste contra o vault que achou —
/// nenhum fixture meu tinha uma lista vazia.
fn item_de_lista(linha: &str) -> Option<(&str, bool)> {
    if matches!(linha, "-" | "*" | "+") {
        return Some(("", false));
    }
    for marca in ["- ", "* ", "+ "] {
        if let Some(resto) = linha.strip_prefix(marca) {
            return Some((resto, false));
        }
    }
    // Lista numerada: `1. `, `12) `. O `true` é o que preserva a
    // numeração na volta.
    let digitos = linha.chars().take_while(char::is_ascii_digit).count();
    if digitos > 0 {
        let resto = &linha[digitos..];
        for marca in [". ", ") "] {
            if let Some(item) = resto.strip_prefix(marca) {
                return Some((item, true));
            }
        }
    }
    None
}

fn e_linha_horizontal(linha: &str) -> bool {
    let sem_espaco: String = linha.chars().filter(|c| !c.is_whitespace()).collect();
    sem_espaco.len() >= 3
        && (sem_espaco.chars().all(|c| c == '-')
            || sem_espaco.chars().all(|c| c == '*')
            || sem_espaco.chars().all(|c| c == '_'))
}

/// `Unidade → markdown`: o caminho de volta.
///
/// Fecha o ciclo com `analisar` (passo 2 da unificação). O que importa
/// não é ser bonito — é ser FIEL: reanalizar o que isto produz tem que
/// dar a mesma árvore. É essa propriedade que permite, mais adiante,
/// trocar a fonte da verdade sem o arquivo mudar sozinho.
///
/// Reusa o `render::Markdown` do ciclo 266 — que existia sem consumidor
/// e agora tem um.
pub fn escrever(raiz: &Unidade) -> String {
    let mut r = crate::render::Markdown::default();
    crate::render::desenhar(raiz, &mut r);
    r.resultado()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tipos(u: &Unidade) -> Vec<Tipo> {
        u.filhos.iter().map(|f| f.tipo.clone()).collect()
    }

    #[test]
    fn titulo_paragrafo_e_citacao() {
        let d = analisar("# Título\n\nUm parágrafo.\n\n> uma citação\n");
        assert_eq!(
            tipos(&d),
            [Tipo::Titulo(1), Tipo::Paragrafo, Tipo::Citacao]
        );
        assert_eq!(d.filhos[0].texto, "Título");
        assert_eq!(d.filhos[1].texto, "Um parágrafo.");
        assert_eq!(d.filhos[2].texto, "uma citação");
    }

    #[test]
    fn linhas_seguidas_sao_um_paragrafo_so() {
        // Markdown junta linhas até a linha em branco. Tratar cada linha
        // como um bloco quebraria todo parágrafo escrito com quebra
        // manual.
        let d = analisar("primeira\nsegunda\n\noutra\n");
        assert_eq!(tipos(&d), [Tipo::Paragrafo, Tipo::Paragrafo]);
        assert_eq!(d.filhos[0].texto, "primeira\nsegunda");
    }

    #[test]
    fn a_lista_agrupa_os_itens() {
        let d = analisar("- um\n- dois\n- três\n");
        assert_eq!(tipos(&d), [Tipo::Lista]);
        assert_eq!(d.filhos[0].filhos.len(), 3);
        assert_eq!(d.filhos[0].filhos[1].texto, "dois");
        // A lista é GRUPO: a navegação desce nela (ciclo 261).
        assert!(!d.filhos[0].politica().atomica);
    }

    #[test]
    fn lista_numerada_e_um_tipo_proprio() {
        // A numeração é observável no arquivo (`1.` contra `-`), e
        // perdê-la reescrevia toda lista numerada do vault como
        // marcador — o que a medição de fidelidade do ciclo 274 mostrou.
        let d = analisar("1. um\n2) dois\n");
        assert_eq!(tipos(&d), [Tipo::ListaOrdenada]);
        assert_eq!(d.filhos[0].filhos[0].texto, "um");
        assert_eq!(d.filhos[0].filhos[1].texto, "dois");
        // E a com marcador continua sendo a outra.
        assert_eq!(tipos(&analisar("- um\n")), [Tipo::Lista]);
    }

    #[test]
    fn linhas_de_citacao_seguidas_sao_uma_citacao_so() {
        // Uma por linha inseria linha em branco entre elas na volta, e o
        // markdown que saía já não era o que entrou.
        let d = analisar("> primeira\n> segunda\n\noutro\n");
        assert_eq!(tipos(&d), [Tipo::Citacao, Tipo::Paragrafo]);
        assert_eq!(d.filhos[0].texto, "primeira\nsegunda");
    }

    #[test]
    fn dentro_da_cerca_nada_e_interpretado() {
        // O caso que separa um analisador de uma sequência de `if`s: o
        // `#` e o `-` aqui são CÓDIGO, não título nem item.
        let d = analisar("```rust\n# não é título\n- não é item\n```\n");
        // A linguagem da cerca sobrevive (ciclo 274): é observável no
        // arquivo e é o que o realce de sintaxe usa.
        assert_eq!(tipos(&d), [Tipo::Codigo(Some("rust".into()))]);
        assert_eq!(d.filhos[0].texto, "# não é título\n- não é item");
    }

    #[test]
    fn cerca_sem_fechamento_nao_perde_conteudo() {
        let d = analisar("```\nsobrou\n");
        assert_eq!(tipos(&d), [Tipo::Codigo(None)]);
        assert_eq!(d.filhos[0].texto, "sobrou");
    }

    #[test]
    fn cerquilha_sem_espaco_nao_e_titulo() {
        // `#tag` é texto. Sem esta regra, toda hashtag viraria um h1.
        let d = analisar("#tag no meio\n");
        assert_eq!(tipos(&d), [Tipo::Paragrafo]);
    }

    #[test]
    fn os_seis_niveis_de_titulo() {
        let d = analisar("# a\n\n## b\n\n### c\n\n#### d\n\n##### e\n\n###### f\n");
        assert_eq!(
            tipos(&d),
            (1..=6).map(Tipo::Titulo).collect::<Vec<_>>()
        );
        // Sete cerquilhas não é título nenhum.
        assert_eq!(tipos(&analisar("####### g\n")), [Tipo::Paragrafo]);
    }

    #[test]
    fn o_embed_vira_uma_unidade_atomica() {
        let d = analisar(
            "antes\n\n{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\ndepois\n",
        );
        assert_eq!(
            tipos(&d),
            [
                Tipo::Paragrafo,
                Tipo::Embed("callout".into()),
                Tipo::Paragrafo
            ]
        );
        // O texto guardado é o FENCE inteiro — é o que o ciclo 264
        // descobriu que o `yy` precisa, e não existe no DOM.
        assert!(d.filhos[1].texto.contains(r#"{{ type: "callout" }}"#));
        assert!(d.filhos[1].politica().atomica);
    }

    #[test]
    fn a_navegacao_ve_o_embed_e_nao_desce_nele() {
        // O RF1 da spec, agora a partir do markdown de verdade.
        let d = analisar("um\n\n{{ type: \"table\" }}\ncolumns:\n- name: A\n{{ /table }}\n\ndois\n");
        let destinos: Vec<Tipo> = d.navegaveis().iter().map(|(_, u)| u.tipo.clone()).collect();
        assert_eq!(
            destinos,
            [
                Tipo::Paragrafo,
                Tipo::Embed("table".into()),
                Tipo::Paragrafo
            ]
        );
    }

    #[test]
    fn linha_horizontal_e_unidade_sem_texto() {
        let d = analisar("a\n\n---\n\nb\n");
        assert_eq!(tipos(&d), [Tipo::Paragrafo, Tipo::Vazia, Tipo::Paragrafo]);
        assert!(!d.filhos[1].politica().aceita_texto);
    }

    #[test]
    fn corpo_vazio_da_arvore_vazia() {
        assert!(analisar("").filhos.is_empty());
        assert!(analisar("\n\n\n").filhos.is_empty());
    }

    #[test]
    fn a_ordem_do_documento_e_preservada() {
        let d = analisar("# t\n\np\n\n- i\n\n> c\n\n```\nx\n```\n");
        assert_eq!(
            tipos(&d),
            [
                Tipo::Titulo(1),
                Tipo::Paragrafo,
                Tipo::Lista,
                Tipo::Citacao,
                Tipo::Codigo(None)
            ]
        );
    }
}

#[cfg(test)]
mod ida_e_volta {
    use super::*;

    /// A propriedade que interessa: **reanalisar o que foi escrito dá a
    /// mesma árvore**.
    ///
    /// Não é "o texto sai idêntico" — o markdown tem várias formas de
    /// escrever a mesma coisa (`*` ou `-` na lista, `1)` ou `1.`), e
    /// exigir o texto igual seria cobrar do escritor uma memória que ele
    /// não tem. O que não pode mudar é o SENTIDO.
    #[track_caller]
    fn estavel(corpo: &str) {
        let uma = analisar(corpo);
        let texto = escrever(&uma);
        let duas = analisar(&texto);
        assert_eq!(
            uma, duas,
            "a ida e volta mudou a árvore.\n--- original ---\n{corpo}\n--- escrito ---\n{texto}"
        );
    }

    #[test]
    fn texto_comum_sobrevive() {
        estavel("# Título\n\nUm parágrafo com duas\nlinhas.\n\n> citação\n");
    }

    #[test]
    fn listas_sobrevivem() {
        estavel("- um\n- dois\n\ntexto\n\n- outra lista\n");
    }

    #[test]
    fn codigo_sobrevive() {
        estavel("antes\n\n```\nfn f() {}\n```\n\ndepois\n");
    }

    #[test]
    fn embed_sobrevive() {
        estavel(
            "antes\n\n{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\ndepois\n",
        );
    }

    #[test]
    fn o_documento_de_exemplo_do_vault_sobrevive() {
        // Um corpo com um pouco de tudo, na ordem em que aparece numa
        // página real.
        estavel(
            "# Página\n\nIntrodução com **negrito** e `código`.\n\n\
             ## Seção\n\n- item um\n- item dois\n\n\
             > uma nota\n\n\
             ```rust\nfn main() {}\n```\n\n\
             {{ type: \"table\" }}\ncolumns:\n- name: A\n{{ /table }}\n\n\
             Fim.\n",
        );
    }

    #[test]
    fn duas_voltas_nao_mudam_mais_nada() {
        // Se a primeira volta muda algo (e ela pode: normaliza a marca
        // da lista), a SEGUNDA já tem que ser idêntica em TEXTO. Sem
        // isso, salvar a mesma página duas vezes daria diffs em git
        // eternamente.
        let corpo = "* estrela\n* outra\n\n1) numerada\n";
        let uma = escrever(&analisar(corpo));
        let duas = escrever(&analisar(&uma));
        assert_eq!(uma, duas, "a segunda volta ainda mexeu no texto");
    }

    #[test]
    fn corpo_vazio_e_estavel() {
        estavel("");
        assert_eq!(escrever(&analisar("")), "");
    }
}
