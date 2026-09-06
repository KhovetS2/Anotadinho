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
    for (faixa, seg) in embed::segment_com_intervalos(corpo) {
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
                        texto,
                    )
                    .da_fonte(corpo[faixa.clone()].to_string())
                    .no_intervalo(faixa),
                );
            }
            DocSegment::Markdown(texto) => {
                // As unidades de um segmento nascem com o intervalo
                // RELATIVO a ele; somar o início do segmento leva pro
                // corpo inteiro, que é a régua da costura.
                for mut u in unidades_de_texto(&texto) {
                    if let Some(r) = u.intervalo.take() {
                        u.intervalo = Some((faixa.start + r.start)..(faixa.start + r.end));
                    }
                    filhos.push(u);
                }
            }
        }
    }
    Unidade::com_filhos(Tipo::Paragrafo, filhos)
}

/// Divide um trecho de markdown puro em unidades.
///
/// Cada unidade sai sabendo a faixa de bytes de onde veio (relativa a
/// `texto`) — é o que permite a costura do ciclo 275.
fn unidades_de_texto(texto: &str) -> Vec<Unidade> {
    /// O que está sendo acumulado, e desde onde.
    struct Aberto {
        inicio: usize,
        fim: usize,
        linhas: Vec<String>,
    }
    impl Aberto {
        fn novo(inicio: usize, fim: usize, linha: &str) -> Self {
            Self { inicio, fim, linhas: vec![linha.to_string()] }
        }
        fn mais(&mut self, fim: usize, linha: &str) {
            self.fim = fim;
            self.linhas.push(linha.to_string());
        }
    }

    let mut fora: Vec<Unidade> = Vec::new();
    let mut paragrafo: Option<Aberto> = None;
    let mut citacao: Option<Aberto> = None;
    let mut itens: Vec<Unidade> = Vec::new();
    let mut lista: Option<Aberto> = None;
    let mut lista_ordenada = false;
    let mut codigo: Option<(Option<String>, Aberto)> = None;

    macro_rules! fechar_paragrafo {
        () => {
            if let Some(a) = paragrafo.take() {
                let cru = a.linhas.join("\n");
                fora.push(
                    Unidade::com_texto(Tipo::Paragrafo, cru.clone())
                        .da_fonte(cru)
                        .no_intervalo(a.inicio..a.fim),
                );
            }
        };
    }
    macro_rules! fechar_citacao {
        () => {
            if let Some(a) = citacao.take() {
                // O texto perde o `>` de cada linha; a fonte o mantém.
                let limpo: Vec<&str> = a
                    .linhas
                    .iter()
                    .map(|l| {
                        let sem = l.trim_start().strip_prefix('>').unwrap_or(l);
                        sem.strip_prefix(' ').unwrap_or(sem)
                    })
                    .collect();
                fora.push(
                    Unidade::com_texto(Tipo::Citacao, limpo.join("\n"))
                        .da_fonte(a.linhas.join("\n"))
                        .no_intervalo(a.inicio..a.fim),
                );
            }
        };
    }
    macro_rules! fechar_lista {
        () => {
            if let Some(a) = lista.take() {
                let tipo = if lista_ordenada { Tipo::ListaOrdenada } else { Tipo::Lista };
                fora.push(
                    Unidade::com_filhos(tipo, std::mem::take(&mut itens))
                        .da_fonte(a.linhas.join("\n"))
                        .no_intervalo(a.inicio..a.fim),
                );
            }
        };
    }

    let mut pos = 0usize;
    for linha_bruta in texto.split_inclusive('\n') {
        let linha = linha_bruta.strip_suffix('\n').unwrap_or(linha_bruta);
        let comeco = pos;
        pos += linha_bruta.len();
        // O fim da unidade é o fim do CONTEÚDO, sem a quebra final — a
        // quebra pertence à separação entre unidades, e é ela que a
        // costura preserva verbatim.
        let fim = comeco + linha.len();

        // Dentro de uma cerca, TUDO é conteúdo — inclusive linhas que
        // pareceriam título ou item.
        if let Some((_, aberto)) = &mut codigo {
            aberto.mais(fim, linha);
            let fechou = linha.trim_start().starts_with("```");
            if fechou {
                let (lingua, aberto) = codigo.take().expect("acabou de existir");
                // O conteúdo é o miolo: sem a linha de abertura nem a de
                // fechamento. A fonte guarda as três partes.
                let interno = aberto.linhas[1..aberto.linhas.len() - 1].join("\n");
                fora.push(
                    Unidade::com_texto(Tipo::Codigo(lingua), interno)
                        .da_fonte(aberto.linhas.join("\n"))
                        .no_intervalo(aberto.inicio..aberto.fim),
                );
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
                Aberto::novo(comeco, fim, linha),
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
            let t = sem_espaco[nivel as usize..].trim_start().to_string();
            fora.push(
                Unidade::com_texto(Tipo::Titulo(nivel), t)
                    .da_fonte(linha)
                    .no_intervalo(comeco..fim),
            );
            continue;
        }

        // A régua vem ANTES da lista: `- - -` é régua, não um item.
        if e_linha_horizontal(sem_espaco) {
            fechar_paragrafo!();
            fechar_citacao!();
            fechar_lista!();
            fora.push(Unidade::nova(Tipo::Vazia).da_fonte(linha).no_intervalo(comeco..fim));
            continue;
        }

        if let Some((item, ordenada)) = item_de_lista(sem_espaco) {
            fechar_paragrafo!();
            fechar_citacao!();
            if itens.is_empty() {
                lista_ordenada = ordenada;
            }
            itens.push(
                Unidade::com_texto(Tipo::Item, item.to_string())
                    .da_fonte(linha)
                    .no_intervalo(comeco..fim),
            );
            match &mut lista {
                Some(a) => a.mais(fim, linha),
                None => lista = Some(Aberto::novo(comeco, fim, linha)),
            }
            continue;
        }

        // Continuação INDENTADA de um item: pertence ao item, não é
        // parágrafo novo. Sem isto, a lista era cortada no meio e a
        // linha em branco que entrava no lugar mudava o arquivo — é a
        // perda que o ciclo 274 mediu e não conseguiu fechar.
        if lista.is_some() && linha.starts_with(char::is_whitespace) {
            if let (Some(a), Some(ultimo)) = (&mut lista, itens.last_mut()) {
                a.mais(fim, linha);
                ultimo.texto.push('\n');
                ultimo.texto.push_str(sem_espaco);
                if let Some(f) = &mut ultimo.fonte {
                    f.push('\n');
                    f.push_str(linha);
                }
                if let Some(r) = &mut ultimo.intervalo {
                    r.end = fim;
                }
            }
            continue;
        }

        if linha.trim_start().starts_with('>') {
            fechar_paragrafo!();
            fechar_lista!();
            match &mut citacao {
                Some(a) => a.mais(fim, linha),
                None => citacao = Some(Aberto::novo(comeco, fim, linha)),
            }
            continue;
        }
        fechar_citacao!();
        fechar_lista!();
        match &mut paragrafo {
            Some(a) => a.mais(fim, linha),
            None => paragrafo = Some(Aberto::novo(comeco, fim, linha)),
        }
    }

    // Cerca que o arquivo não fechou: o conteúdo não pode sumir.
    if let Some((lingua, aberto)) = codigo.take() {
        let interno = aberto.linhas[1..].join("\n");
        fora.push(
            Unidade::com_texto(Tipo::Codigo(lingua), interno)
                .da_fonte(aberto.linhas.join("\n"))
                .no_intervalo(aberto.inicio..aberto.fim),
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

/// Escreve a árvore de volta COSTURANDO com o corpo original.
///
/// A diferença pra `escrever` é o que acontece com o que ninguém tocou:
/// aqui ele volta pelos BYTES, e o que está ENTRE as unidades — linhas
/// em branco, indentação, o que o analisador não entendeu — volta junto.
///
/// É isso que dá fidelidade sem analisador perfeito (ciclo 275). O ciclo
/// 274 tentou com o texto de cada unidade e chegou a 35 de 242 páginas
/// idênticas: guardar o texto não bastava porque a perda estava nas
/// FRONTEIRAS, que não pertencem a unidade nenhuma.
///
/// Uma unidade sem intervalo é uma unidade que o editor criou ou mudou —
/// essa é serializada, que é o certo: não há original pra devolver.
pub fn escrever_costurando(corpo: &str, raiz: &Unidade) -> String {
    let mut fora = String::new();
    let mut cursor = 0usize;

    // Os dois campos respondem perguntas diferentes, e é isso que faz a
    // costura funcionar:
    //
    // - `intervalo` diz ONDE a unidade estava — serve pra posicionar, e
    //   sobrevive à edição;
    // - `fonte` diz SE ela continua como estava — some quando o editor
    //   mexe nela.
    //
    // Na primeira versão eu limpava os dois ao editar, e o texto velho
    // ficava no arquivo: sem o intervalo não havia o que substituir, só
    // onde inserir.
    let serializar = |u: &Unidade| {
        let mut r = crate::render::Markdown::default();
        crate::render::desenhar(&Unidade::com_filhos(Tipo::Paragrafo, vec![u.clone()]), &mut r);
        r.resultado()
    };

    for filho in &raiz.filhos {
        let posicionada = filho
            .intervalo
            .as_ref()
            .filter(|f| f.end <= corpo.len() && f.start >= cursor);

        match (posicionada, &filho.fonte) {
            // Intacta: volta pelos bytes, e o que vinha antes dela junto.
            (Some(faixa), Some(_)) => {
                fora.push_str(&corpo[cursor..faixa.start]);
                fora.push_str(&corpo[faixa.clone()]);
                cursor = faixa.end;
            }
            // Editada: o lugar é o mesmo, o conteúdo é novo.
            (Some(faixa), None) => {
                fora.push_str(&corpo[cursor..faixa.start]);
                fora.push_str(serializar(filho).trim_end());
                cursor = faixa.end;
            }
            // Nova: não tem lugar de origem.
            _ => {
                if !fora.is_empty() && !fora.ends_with('\n') {
                    fora.push('\n');
                }
                fora.push_str(&serializar(filho));
                fora.push('\n');
            }
        }
    }
    // E o rabo do arquivo — o que vinha depois da última unidade.
    if cursor < corpo.len() {
        fora.push_str(&corpo[cursor..]);
    }
    fora
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

#[cfg(test)]
mod costura {
    use super::*;

    #[test]
    fn o_que_ninguem_tocou_volta_byte_a_byte() {
        // Formatação que o analisador NÃO entende — recuo de
        // continuação, três espaços, linha em branco dupla — volta
        // igual, porque volta copiada.
        let corpo = "# T\n\n\n- item\n   continuação indentada\n- outro\n\n\ntexto   \n";
        let arvore = analisar(corpo);
        assert_eq!(escrever_costurando(corpo, &arvore), corpo);
    }

    #[test]
    fn so_a_unidade_mudada_e_reescrita() {
        // O caso que o passo 4 precisa: editar UM bloco não pode
        // reformatar os vizinhos.
        let corpo = "# Título\n\n- a\n   recuo estranho\n- b\n\nfim\n";
        let mut arvore = analisar(corpo);
        // "edita" o parágrafo final: perde o intervalo, como faria o
        // editor ao mexer nele.
        // Editar limpa a FONTE e mantém o intervalo: o lugar continua
        // sendo o mesmo, só o conteúdo mudou.
        let ultimo = arvore.filhos.last_mut().unwrap();
        ultimo.texto = "outro fim".into();
        ultimo.fonte = None;

        let saida = escrever_costurando(corpo, &arvore);
        assert!(saida.contains("   recuo estranho"), "o vizinho foi reformatado:\n{saida}");
        assert!(saida.contains("# Título"), "o título sumiu:\n{saida}");
        assert!(saida.contains("outro fim"), "a edição não entrou:\n{saida}");
        assert!(!saida.contains("\nfim\n"), "o texto velho ficou:\n{saida}");
    }

    #[test]
    fn embed_intocado_volta_com_o_yaml_original() {
        // `to_fence_text` normaliza o YAML. Costurando, um embed que
        // ninguém editou volta como estava — o que o editor de hoje NÃO
        // faz (ele reserializa tudo ao salvar).
        let corpo = "antes\n\n{{ type: \"callout\" }}\ntitle: Nota\nvariant: info\n{{ /callout }}\n\ndepois\n";
        let arvore = analisar(corpo);
        assert_eq!(escrever_costurando(corpo, &arvore), corpo);
    }

    #[test]
    fn arvore_sem_intervalo_nenhum_ainda_escreve() {
        // Documento montado do zero (sem original): a costura cai na
        // serialização e não perde nada.
        let arvore = Unidade::com_filhos(
            Tipo::Paragrafo,
            vec![
                Unidade::com_texto(Tipo::Titulo(1), "Novo"),
                Unidade::com_texto(Tipo::Paragrafo, "corpo"),
            ],
        );
        let saida = escrever_costurando("", &arvore);
        assert!(saida.contains("# Novo"), "{saida}");
        assert!(saida.contains("corpo"), "{saida}");
    }

    #[test]
    fn corpo_vazio_continua_vazio() {
        assert_eq!(escrever_costurando("", &analisar("")), "");
    }
}
