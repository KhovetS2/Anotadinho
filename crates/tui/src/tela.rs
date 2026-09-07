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
    /// A unidade a que esta linha pertence.
    pub caminho: Caminho,
    /// Linha DECORATIVA: a borda de uma caixa, uma régua (ciclo 293).
    ///
    /// Ocupa espaço na tela e NÃO é destino — o cursor nunca pousa nela.
    /// Carrega o caminho do dono mesmo assim, pra sumir junto quando ele
    /// dobra.
    ///
    /// Sem esta distinção não dá pra desenhar caixa em volta de embed:
    /// cada borda desalinharia cursor e rolagem, que são contados em
    /// índice de linha visível.
    pub enfeite: bool,
    /// Profundidade na árvore — vira recuo na tela.
    pub nivel: usize,
    /// O que se lê, já sem os marcadores de markdown.
    pub texto: String,
    /// A marca que abre a linha (`##`, `-`, `[kanban]`).
    pub marca: String,
    /// O que o nível tem dentro, pra quando ele estiver FECHADO
    /// (ciclo 293).
    ///
    /// Vazio em bloco de texto. Fica separado do `texto` porque só é
    /// informação quando o conteúdo não está à vista: aberto, `· 1 item`
    /// em cima do único item é ruído, e uma página cheia de listas de um
    /// item vira uma página cheia de ruído.
    pub resumo: String,
    /// O texto quebrado em trechos com estilo (ciclo 287).
    ///
    /// Vazio quando não há nada a estilizar — bloco de código, embed,
    /// grupo. Nesses o `texto` já basta, e quebrar seria mentir: dentro
    /// de código um asterisco é asterisco.
    pub trechos: Vec<Trecho>,
    /// O tipo de quem produziu a linha — o desenho estiliza por ele.
    pub tipo: Tipo,
    /// A fileira de etapas, quando esta linha é um embed de fluxo.
    pub trilha: Option<String>,
    /// O nome do embed a que esta linha pertence, se pertence a algum.
    ///
    /// É o gancho de OVERRIDE da borda (ciclo 294): a borda é da unidade
    /// base e some por padrão, e quem quiser desenhar a sua — um embed —
    /// se identifica aqui.
    pub embed_dono: Option<String>,
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
            enfeite: false,
            nivel,
            texto,
            resumo: contagem(u),
            trilha: matches!(&u.tipo, Tipo::Embed(n) if n == "fluxo")
                .then(|| trilha_do_fluxo(u))
                .flatten(),
            marca: marca(&u.tipo),
            trechos,
            tipo: u.tipo.clone(),
            embed_dono: None,
        });
    }

    /// Desce no embed: é justamente o conteúdo que o ciclo 283 deu a
    /// ele que faz uma página com kanban valer alguma coisa num
    /// terminal.
    fn desce_no_atomico(&self) -> bool {
        true
    }
}

/// O que um NÍVEL mostra quando não tem texto próprio (ciclo 289).
///
/// Uma lista e um embed são destinos de navegação — entra-se neles com
/// Enter — e não têm texto. Desenhados só com a marca, saíam como um
/// `·` solto ou um `[kanban]` mudo, e a pessoa não tinha como saber o
/// que havia ali sem entrar.
///
/// A contagem é o que um nível FECHADO tem a dizer. Não por acaso é
/// também a representação que o colapso vai usar: a linha do grupo já é
/// a forma dobrada dele.
fn contagem(u: &Unidade) -> String {
    let n = u.filhos.len();
    if n == 0 {
        return String::new();
    }
    // Partes de nomes DIFERENTES são campos, não coleção: o fluxo tem
    // um artefato e uma etapa, e contar "2 items" ali não diz nada. O
    // que serve é o valor deles.
    let folhas_distintas = u.filhos.len() <= 3
        && u.filhos.iter().all(|f| {
            matches!(&f.tipo, Tipo::Parte { grupo: false, .. })
                && f.filhos.is_empty()
                && !f.texto.trim().is_empty()
        })
        && nomes_das_partes(u).map(|n| n.len()) == Some(u.filhos.len());
    if folhas_distintas {
        return u
            .filhos
            .iter()
            .map(|f| f.texto.trim())
            .collect::<Vec<_>>()
            .join(" · ");
    }

    // Nome do que está dentro, quando os filhos concordam — "2 columns"
    // diz mais que "2 items", e o embed já sabe o nome das partes dele.
    let nome = match &u.filhos[0].tipo {
        Tipo::Parte { nome, .. } if u.filhos.iter().all(|f| {
            matches!(&f.tipo, Tipo::Parte { nome: outro, .. } if outro == nome)
        }) =>
        {
            nome.clone()
        }
        _ => "item".to_string(),
    };
    // Plural do português: quase tudo aqui termina em consoante ou
    // vogal, e nenhum nome de parte é irregular.
    if n == 1 {
        format!("1 {nome}")
    } else {
        format!("{n} {nome}s")
    }
}

/// Os nomes distintos das partes filhas, se todas forem partes.
fn nomes_das_partes(u: &Unidade) -> Option<std::collections::BTreeSet<&str>> {
    u.filhos
        .iter()
        .map(|f| match &f.tipo {
            Tipo::Parte { nome, .. } => Some(nome.as_str()),
            _ => None,
        })
        .collect()
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

/// A trilha de etapas de um fluxo, como a GUI desenha (ciclo 293).
///
/// Na janela isso é uma fileira de pílulas com a atual acesa. No
/// terminal é a mesma fileira, com a atual entre colchetes — a pessoa
/// precisa ver EM QUE PONTO o artefato está, e "Concluída" sozinho não
/// diz de onde ele veio nem pra onde vai.
fn trilha_do_fluxo(u: &Unidade) -> Option<String> {
    use anotadinho_core::fluxo::Etapa;
    let atual = u
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "etapa"))?
        .texto
        .clone();
    let fileira: Vec<String> = Etapa::all()
        .iter()
        .map(|e| {
            let r = e.label();
            if r == atual {
                format!("[{r}]")
            } else {
                r.to_string()
            }
        })
        .collect();
    Some(fileira.join("  "))
}

/// A página inteira em linhas, na ordem em que se lê.
pub fn linhas(raiz: &Unidade) -> Vec<Linha> {
    let mut r = Linhas::default();
    desenhar(raiz, &mut r);
    encaixotar_embeds(r.fora)
}

/// Põe uma borda de fechamento embaixo de cada embed (ciclo 293).
///
/// Um embed na GUI é um CARTÃO — tem moldura, e é ela que diz onde ele
/// começa e acaba. No terminal a moldura é uma linha decorativa: o
/// rótulo `[kanban]` abre, e esta fecha.
///
/// Só quando o embed tem conteúdo à vista. Um embed dobrado é uma linha
/// só, e emoldurar uma linha só é enfeite sem função.
fn encaixotar_embeds(linhas: Vec<Linha>) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::with_capacity(linhas.len());
    // O embed aberto agora, e se ele já teve conteúdo.
    //
    // Estado explícito, e não varredura pra trás: a primeira versão
    // procurava "o último embed em `fora`" a cada linha, e depois de
    // fechar a caixa continuava achando o mesmo embed — saía um `└`
    // sobrando a cada bloco seguinte. Apareceu na tela em três segundos.
    let mut aberto: Option<(Caminho, usize, bool, String)> = None;

    for mut l in linhas {
        if let Some((dono, nivel, teve, _)) = &aberto {
            let ainda_dentro = l.caminho.len() > dono.len() && l.caminho.starts_with(dono);
            if !ainda_dentro {
                if *teve {
                    // O fecho leva o nome do dono: sem isso a régua da
                    // unidade some justo na linha que fecha o cartão, e
                    // a moldura fica pela metade.
                    fora.push(fecho(dono, *nivel, &aberto.as_ref().unwrap().3));
                }
                aberto = None;
            } else {
                let nome = aberto.as_ref().map(|a| a.3.clone()).unwrap_or_default();
                l.embed_dono = Some(nome.clone());
                aberto = Some((dono.clone(), *nivel, true, nome));
            }
        }
        if let Tipo::Embed(nome) = &l.tipo {
            if !l.enfeite {
                l.embed_dono = Some(nome.clone());
                aberto = Some((l.caminho.clone(), l.nivel, false, nome.clone()));
            }
        }

        let trilha = (!l.enfeite).then(|| l.trilha.clone()).flatten();
        let dono = l.caminho.clone();
        let nivel = l.nivel;
        fora.push(l);
        if let Some(t) = trilha {
            // A trilha é conteúdo do embed pra efeito de caixa.
            let nome = aberto.as_ref().map(|a| a.3.clone());
            if let Some((_, _, teve, _)) = aberto.as_mut() {
                *teve = true;
            }
            fora.push(Linha {
                caminho: dono,
                enfeite: true,
                nivel: nivel + 1,
                texto: t,
                resumo: String::new(),
                marca: "│".to_string(),
                trechos: Vec::new(),
                tipo: Tipo::Vazia,
                trilha: None,
                embed_dono: nome,
            });
        }
    }
    // O embed pode ser a última coisa da página.
    if let Some((dono, nivel, true, nome)) = aberto {
        fora.push(fecho(&dono, nivel, &nome));
    }
    fora
}

/// A linha que fecha a caixa de um embed.
fn fecho(dono: &Caminho, nivel: usize, embed: &str) -> Linha {
    Linha {
        caminho: dono.clone(),
        enfeite: true,
        nivel,
        texto: String::new(),
        resumo: String::new(),
        marca: "└".to_string(),
        trechos: Vec::new(),
        tipo: Tipo::Vazia,
        trilha: None,
        embed_dono: Some(embed.to_string()),
    }
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

/// Acima disto, um nível nasce DOBRADO.
///
/// Responde ao caso que a pessoa levantou: uma consulta com mil itens é
/// impossível de percorrer de `j` em `j`. Dobrada, ela ocupa uma linha
/// até alguém abrir — e a linha já diz quantos itens tem (ciclo 289).
pub const DOBRA_AUTOMATICA: usize = 30;

/// Os caminhos que nascem dobrados numa página.
pub fn dobras_iniciais(raiz: &Unidade) -> std::collections::HashSet<Caminho> {
    raiz.percorrer()
        .into_iter()
        .filter(|(_, u)| u.filhos.len() > DOBRA_AUTOMATICA)
        .map(|(c, _)| c)
        .collect()
}

/// As linhas que aparecem, escondendo o que está dentro de uma dobra.
///
/// Esconde o CONTEÚDO da dobra, nunca a linha dela: quem dobrou precisa
/// continuar vendo onde dobrou, e é essa linha que ele vai reabrir.
pub fn visiveis<'a>(
    linhas: &'a [Linha],
    dobrados: &std::collections::HashSet<Caminho>,
) -> Vec<&'a Linha> {
    linhas
        .iter()
        .filter(|l| {
            !dobrados
                .iter()
                .any(|d| d.len() < l.caminho.len() && l.caminho.starts_with(d))
        })
        .collect()
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
        assert_eq!(primeira.resumo, "1 item");
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
    fn um_nivel_diz_quantos_itens_tem() {
        // O `·` solto era o defeito: a lista é destino de navegação e
        // não tinha o que mostrar sem que alguém entrasse nela.
        let d = analisar("antes\n\n- um\n- dois\n- tres\n");
        let lista = linhas(&d).into_iter().find(|l| l.marca == "·").unwrap();
        assert_eq!(lista.resumo, "3 items");
        assert_eq!(lista.texto, "", "o resumo vazou pro texto");
    }

    #[test]
    fn o_embed_diz_o_nome_do_que_tem_dentro() {
        // "4 colunas" diz mais que "4 itens", e o embed já sabe o nome
        // das partes dele desde o ciclo 283.
        let d = analisar(
            "{{ type: \"kanban\" }}\ncolumns:\n- A\n- B\nitems:\n- title: X\n  column: A\n{{ /kanban }}\n",
        );
        let e = &linhas(&d)[0];
        assert_eq!(e.marca, "[kanban]");
        // O resumo é campo próprio: só aparece quando o nível está
        // FECHADO (ciclo 293).
        assert_eq!(e.resumo, "2 columns");
        assert_eq!(e.texto, "");
    }

    #[test]
    fn o_fluxo_deixou_de_ser_uma_linha_muda() {
        // Ele aparecia como `[fluxo]` e mais nada, porque o ciclo 283
        // não lhe deu filhos e o 284 parou de mostrar o texto do embed.
        let d = analisar(
            "{{ type: \"fluxo\" }}\nartefato: spec\netapa: concluida\n{{ /fluxo }}\n",
        );
        let ls = linhas(&d);
        assert_eq!(ls[0].marca, "[fluxo]");
        assert_eq!(ls[0].resumo, "Spec · Concluída");
        // E o estado aparece nos filhos.
        let textos: Vec<&str> = ls.iter().map(|l| l.texto.as_str()).collect();
        assert!(textos.contains(&"Spec"), "{textos:?}");
        assert!(textos.contains(&"Concluída"), "{textos:?}");
    }

    #[test]
    fn um_item_so_nao_vira_plural() {
        let d = analisar("- só um\n");
        let lista = linhas(&d).into_iter().find(|l| l.marca == "·").unwrap();
        assert_eq!(lista.resumo, "1 item");
    }

    #[test]
    fn o_embed_com_conteudo_ganha_caixa() {
        let d = analisar(
            "antes\n\n{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\ndepois\n",
        );
        let ls = linhas(&d);
        let fechos: Vec<&Linha> = ls.iter().filter(|l| l.enfeite && l.marca == "└").collect();
        assert_eq!(fechos.len(), 1, "esperava UMA caixa fechada");
        // Fecha depois do conteúdo e antes do bloco seguinte.
        let i = ls.iter().position(|l| l.enfeite && l.marca == "└").unwrap();
        assert_eq!(ls[i - 1].texto, "oi");
        assert_eq!(ls[i + 1].texto, "depois");
    }

    #[test]
    fn a_caixa_nao_sobra_depois_do_embed() {
        // A primeira versão procurava "o último embed" varrendo pra trás
        // e, depois de fechar, continuava achando o mesmo — saía um `└`
        // a cada bloco seguinte. Apareceu na tela em três segundos.
        let d = analisar(
            "{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\na\n\nb\n\nc\n",
        );
        let fechos = linhas(&d).iter().filter(|l| l.enfeite && l.marca == "└").count();
        assert_eq!(fechos, 1, "sobrou fecho de caixa");
    }

    #[test]
    fn embed_sem_conteudo_nao_ganha_caixa() {
        // Emoldurar uma linha só é enfeite sem função.
        let d = analisar("{{ type: \"query\" }}\nview: list\n{{ /query }}\n");
        let com_from = linhas(&d).iter().any(|l| l.texto == "pages");
        assert!(!com_from, "a consulta sem `from` não devia ter parte");
        let fechos = linhas(&d).iter().filter(|l| l.enfeite).count();
        assert_eq!(fechos, 0);
    }

    #[test]
    fn o_fluxo_desenha_a_trilha_de_etapas() {
        // Como a GUI: a fileira inteira, com a atual destacada. "Concluída"
        // sozinho não diz de onde veio nem pra onde vai.
        let d = analisar(
            "{{ type: \"fluxo\" }}\nartefato: spec\netapa: aprovada\n{{ /fluxo }}\n",
        );
        let trilha = linhas(&d)
            .into_iter()
            .find(|l| l.enfeite && l.marca == "│")
            .expect("faltou a trilha");
        assert!(trilha.texto.contains("Rascunho"), "{:?}", trilha.texto);
        assert!(trilha.texto.contains("[Aprovada]"), "a atual não está marcada: {:?}", trilha.texto);
        assert!(trilha.texto.contains("Concluída"), "{:?}", trilha.texto);
    }

    #[test]
    fn enfeite_nao_e_destino() {
        // A borda ocupa linha e não recebe cursor: o modelo não sabe que
        // ela existe, e pousar nela seria pousar em nada.
        let d = analisar(
            "{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n",
        );
        for l in linhas(&d).iter().filter(|l| l.enfeite) {
            assert!(
                d.em(&l.caminho).is_some(),
                "enfeite com caminho que não existe: {:?}",
                l.caminho
            );
        }
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
