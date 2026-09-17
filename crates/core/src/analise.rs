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
use crate::unidade::{Arranjo, Tipo, Unidade};

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
                let mut unidade = Unidade::com_texto(
                    Tipo::Embed(dados.kind().type_name().to_string()),
                    texto,
                )
                .da_fonte(corpo[faixa.clone()].to_string())
                .no_intervalo(faixa);
                // O embed declara o CONTEÚDO dele (ciclo 283).
                //
                // Continua atômico: `navegaveis()` para nele e a GUI não
                // vê nada disto. Quem desce é o renderizador de terminal
                // (`desce_no_atomico`), que até aqui descia e achava
                // vazio — a página inteira do porte CLI parava na borda
                // de cada embed.
                unidade.filhos = partes_do_embed(&dados);
                filhos.push(unidade);
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
/// A ação principal de um fluxo, se o estado dele tem uma.
///
/// São os dois momentos em que o fluxo oferece mais do que mudar de
/// etapa, e os dois vêm da janela (ciclos 209 e 223):
///
/// - **aprovada** é onde o trabalho passa do "o quê" pro "como";
/// - **em revisão** ganhou a terceira saída — "está quase, muda estes
///   pontos" — que antes era copiar e colar na mão.
///
/// Só pra spec e proposta: execução e conversa não têm o que planejar.
fn acao_do_fluxo(d: &embed::FluxoEmbedData) -> Option<(String, String)> {
    use crate::fluxo::{Artefato, Etapa};
    if !matches!(d.artefato, Artefato::Spec | Artefato::Proposta) {
        return None;
    }
    match d.etapa {
        Etapa::Aprovada => Some(if d.artefato == Artefato::Spec {
            (
                "Planejar implementação".into(),
                "Abre uma conversa com esta spec anexada.".into(),
            )
        } else {
            (
                "Executar".into(),
                "Abre uma conversa com esta proposta anexada. A abordagem já foi aceita."
                    .into(),
            )
        }),
        Etapa::EmRevisao => Some((
            "Pedir alteração".into(),
            "O agente devolve a mudança como proposta, pra você ver o diff antes de aplicar."
                .into(),
        )),
        _ => None,
    }
}

/// O conteúdo de um embed, como unidades.
///
/// **Conteúdo, não controle.** O DOM de um embed mistura as duas coisas:
/// o cartão de um kanban e o botão de apagar aquele cartão são os dois
/// `[data-nav-item]`. Só o primeiro existe no markdown; o segundo é
/// cromo da GUI, e um terminal desenharia o seu próprio. Medi isso antes
/// de escrever: dos 16 itens navegáveis da galeria, 2 são as imagens.
///
/// **O que é dinâmico não entra.** As linhas de uma consulta vêm do
/// vault, não do texto da página, e `analisar` é função pura do corpo.
/// A consulta declara o que ESTÁ escrito — de onde, filtro, visão — e as
/// linhas são runtime. O mesmo vale pro calendário em modo vault.
fn partes_do_embed(dados: &embed::EmbedData) -> Vec<Unidade> {
    use embed::EmbedData;

    /// Uma parte folha: carrega texto, não comporta filhos.
    fn item(nome: &str, texto: impl Into<String>) -> Unidade {
        Unidade::com_texto(
            Tipo::Parte {
                nome: nome.to_string(),
                arranjo: Arranjo::Folha,
            },
            texto,
        )
    }
    /// Uma parte que comporta outras, empilhadas.
    fn grupo(nome: &str, texto: impl Into<String>, filhos: Vec<Unidade>) -> Unidade {
        arranjado(nome, texto, filhos, Arranjo::Coluna)
    }
    /// Uma parte cujos filhos ficam LADO A LADO (ciclo 297).
    fn fileira(nome: &str, filhos: Vec<Unidade>) -> Unidade {
        arranjado(nome, String::new(), filhos, Arranjo::Linha)
    }
    fn arranjado(
        nome: &str,
        texto: impl Into<String>,
        filhos: Vec<Unidade>,
        arranjo: Arranjo,
    ) -> Unidade {
        let mut u = Unidade::com_filhos(
            Tipo::Parte {
                nome: nome.to_string(),
                arranjo,
            },
            filhos,
        );
        u.texto = texto.into();
        u
    }

    match dados {
        // Coluna é grupo, cartão é folha — é a forma que o board tem, e
        // é o que faz `j` dentro de uma coluna significar "próximo
        // cartão" e não "próxima coluna".
        EmbedData::Kanban(d) => d
            .columns
            .iter()
            .map(|coluna| {
                let cartoes = d
                    .items
                    .iter()
                    .filter(|c| &c.column == coluna)
                    .map(|c| item("card", c.title.clone()))
                    .collect();
                grupo("column", coluna.clone(), cartoes)
            })
            .collect(),

        // Cabeçalho e linha são FILEIRA, não grupo (ciclo 305): célula
        // ao lado de célula é como se lê uma tabela — é a mesma peça
        // (`Arranjo::Linha`) que já faz o botão de ações ficar lado a
        // lado; até aqui o desenho da TUI não sabia disso, e cada
        // célula saía numa linha própria, uma embaixo da outra.
        //
        // O nome da célula carrega a cor do badge quando a coluna é
        // Select/MultiSelect — o mesmo `badge_class` que a janela usa
        // pra pintar `.badge--info`/`--success`/`--warning`/`--error`.
        // Célula comum fica "cell", sem cor própria.
        EmbedData::Table(d) => {
            let nome_da_celula = |col: Option<&embed::TableColumn>, valor: &str| -> String {
                match col.map(|c| &c.kind) {
                    Some(embed::ColumnKind::Select { options }) => {
                        embed::badge_class(options, valor).to_string()
                    }
                    Some(embed::ColumnKind::MultiSelect { options }) => {
                        let primeira = valor.split(", ").next().unwrap_or("");
                        embed::badge_class(options, primeira).to_string()
                    }
                    _ => "cell".to_string(),
                }
            };
            let cabecalho = fileira(
                "header",
                d.columns.iter().map(|c| item("cell", c.name.clone())).collect(),
            );
            std::iter::once(cabecalho)
                .chain(d.rows.iter().map(|linha| {
                    fileira(
                        "row",
                        linha
                            .iter()
                            .enumerate()
                            .map(|(i, v)| item(&nome_da_celula(d.columns.get(i), v), v.clone()))
                            .collect(),
                    )
                }))
                .collect()
        }

        // O corpo de um callout É markdown, então ele vira unidades de
        // markdown de verdade — não partes. É o único embed em que o
        // conteúdo interno é do mesmo tecido da página.
        //
        // Na frente do corpo, o que a janela desenha no cabeçalho (ciclo
        // 307): a VARIANTE, que dá a cor da caixa inteira, e o título.
        // A variante é dado de desenho — o terminal a esconde da tela e
        // o cursor passa por cima dela —, o título é conteúdo.
        EmbedData::Callout(d) => {
            let mut partes = vec![item("variante", d.variant.slug())];
            if !d.title.trim().is_empty() {
                partes.push(item("titulo", d.title.clone()));
            }
            partes.extend(unidades_de_texto(&d.body));
            partes
        }

        // Cada painel é um grupo cujo conteúdo também é markdown.
        EmbedData::Columns(d) => d
            .columns
            .iter()
            .map(|painel| grupo("pane", String::new(), unidades_de_texto(&painel.body)))
            .collect(),

        // "miniatura", não "item" (ciclo 304): o terminal não desenha a
        // imagem, então o nome da parte é quem diz que ali era pra ser
        // uma — e é o gancho que o desenho usa pra pintar o selo que
        // faz as vezes dela.
        EmbedData::Gallery(d) => d
            .items
            .iter()
            .map(|i| {
                item(
                    "miniatura",
                    if i.caption.is_empty() {
                        i.path.clone()
                    } else {
                        i.caption.clone()
                    },
                )
            })
            .collect(),

        // Cada item vira uma BARRA proporcional — a mesma aritmética
        // que a janela usa pra desenhar (`embed::bar_span`, ciclo 167),
        // calculada uma vez aqui em vez de a cada repaint (ciclo 305).
        //
        // A janela escolhe a janela de tempo pela navegação (mês/semana
        // corrente); aqui não há esse estado — TUI e CLI não têm "mês
        // atual" — então a janela é o próprio conjunto: do primeiro
        // início ao último fim. ISO (`AAAA-MM-DD`) ordena léxico igual
        // a cronológico, então `min`/`max` de string bastam, sem
        // reabrir `date_util` pra isso.
        EmbedData::Timeline(d) => {
            let inicio_janela = d.items.iter().filter_map(|i| i.start.as_deref()).min();
            let fim_janela = d
                .items
                .iter()
                .filter_map(|i| i.end.as_deref().or(i.start.as_deref()))
                .max();
            let dias_da_janela = match (inicio_janela, fim_janela) {
                (Some(a), Some(b)) => crate::date_util::days_between(a, b).unwrap_or(0) + 1,
                _ => 0,
            }
            .max(1);

            d.items
                .iter()
                .map(|i| {
                    let span = inicio_janela.and_then(|ini| {
                        embed::bar_span(i.start.as_deref(), i.end.as_deref(), ini, dias_da_janela)
                    });
                    match span {
                        // Início e duração viajam como duas partes
                        // FILHAS, em porcentagem inteira (0-100) — não
                        // no texto da barra, que seria dado disfarçado
                        // de rótulo. `linha_de_caixa` (TUI) lê as duas
                        // e desenha um retângulo proporcional; nenhum
                        // outro consumidor precisa delas.
                        Some((inicio_pct, largura_pct)) => grupo(
                            "barra",
                            i.title.clone(),
                            vec![
                                item("inicio", inicio_pct.round().to_string()),
                                item("duracao", largura_pct.round().max(1.0).to_string()),
                            ],
                        ),
                        // Sem data (ficaria na "gaveta" da janela): uma
                        // parte comum, sem barra pra desenhar.
                        None => item("item", i.title.clone()),
                    }
                })
                .collect()
        }

        // Botão é coisa clicável, e clicável fica em FILEIRA — é como a
        // janela desenha, e agora o modelo diz isso em vez de o desenho
        // adivinhar (ciclo 297).
        //
        // O `variant: primary` também é domínio, não cromo (ciclo 304):
        // é o mesmo campo que a janela lê pra pintar
        // `.actions-embed__btn--primary` de destaque, e o nome da parte
        // é como o desenho do terminal recebe a mesma informação sem
        // reabrir o YAML.
        EmbedData::Actions(d) => vec![fileira(
            "acoes",
            d.buttons
                .iter()
                .map(|b| {
                    let nome = if b.variant.as_deref() == Some("primary") {
                        "button-primary"
                    } else {
                        "button"
                    };
                    item(nome, b.label.clone())
                })
                .collect(),
        )],

        // Só os eventos ESCRITOS no embed. No modo vault a lista vem de
        // fora e não pertence à árvore desta página.
        //
        // A GRADE DO MÊS de verdade (ciclo 306), a mesma da janela: um
        // "mes" por mês que tem evento, uma "semana" por fileira de 7
        // "dia"s, e dentro de cada dia uma parte por FAIXA — as faixas
        // saem de `calendario::pack_days`, o mesmo algoritmo que a janela
        // usa (movido pra cá pra não haver duas cópias).
        //
        // Cada dia tem exatamente tantas faixas quanto a semana usa, pra
        // quem desenha ler a faixa pela POSIÇÃO: "evento" (começa aqui),
        // "evento-continua" (a barra vem do dia anterior) ou "vazio". A
        // cor do evento vai no sufixo do nome ("evento--info"), a mesma
        // de `badge_class` sobre as tags do calendário inteiro. Nenhuma
        // dessas partes vira linha na tela — é o desenho da semana que
        // as lê.
        EmbedData::Calendar(d) => {
            use crate::calendario::{existing_tags, month_cells, months_with_events, pack_days};
            let tags = existing_tags(&d.entries);
            let nome_do_evento = |e: &embed::CalendarEntry| -> String {
                match e.all_tags().first() {
                    Some(t) => {
                        let classe = embed::badge_class(&tags, t);
                        format!("evento{}", classe.strip_prefix("badge").unwrap_or(""))
                    }
                    None => "evento".to_string(),
                }
            };

            let mut partes: Vec<Unidade> = months_with_events(&d.entries)
                .into_iter()
                .map(|(ano, mes)| {
                    let semanas = month_cells(ano, mes)
                        .chunks(7)
                        // A sexta semana costuma ser toda do mês seguinte:
                        // a janela reserva a altura, o terminal não tem
                        // linha pra gastar com ela.
                        .filter(|semana| semana.iter().any(|c| c.3))
                        .map(|semana| {
                            let datas: Vec<String> = semana
                                .iter()
                                .map(|&(y, m, dia, _)| crate::date_util::format_date(y, m, dia))
                                .collect();
                            let (barras, excesso) = pack_days(&d.entries, &datas, false);
                            let faixas = barras.iter().map(|b| b.lane + 1).max().unwrap_or(0);
                            let dias = semana
                                .iter()
                                .enumerate()
                                .map(|(col, &(_, _, dia, do_mes))| {
                                    let mut slots: Vec<Unidade> = (0..faixas)
                                        .map(|faixa| {
                                            match barras.iter().find(|b| {
                                                b.lane == faixa && b.start_col <= col && col <= b.end_col
                                            }) {
                                                Some(b) if b.start_col == col => item(
                                                    &nome_do_evento(&d.entries[b.entry_idx]),
                                                    d.entries[b.entry_idx].title.clone(),
                                                ),
                                                Some(_) => item("evento-continua", String::new()),
                                                None => item("vazio", String::new()),
                                            }
                                        })
                                        .collect();
                                    if excesso[col] > 0 {
                                        slots.push(item("mais", format!("+{} mais", excesso[col])));
                                    }
                                    grupo(if do_mes { "dia" } else { "dia-fora" }, dia.to_string(), slots)
                                })
                                .collect();
                            fileira("semana", dias)
                        })
                        .collect();
                    grupo(
                        "mes",
                        format!("{} {}", crate::date_util::month_name(mes), ano),
                        semanas,
                    )
                })
                .collect();

            // A "gaveta" da janela: evento sem data não tem dia na grade.
            let sem_data: Vec<Unidade> = d
                .entries
                .iter()
                .filter(|e| e.date.is_none())
                .map(|e| item("entry", e.title.clone()))
                .collect();
            if !sem_data.is_empty() {
                partes.push(grupo("sem-data", format!("Sem data ({})", sem_data.len()), sem_data));
            }
            partes
        }

        // A consulta declara a definição, que é o que está escrito; as
        // linhas são runtime.
        // Parte vazia é ruído: uma consulta sem `from` não tem o que
        // declarar, e declarar `from: ""` fazia o renderizador achar que
        // havia conteúdo (achado pelo teste da caixa, ciclo 293).
        EmbedData::Query(q) => q
            .from
            .as_ref()
            .filter(|f| !f.trim().is_empty())
            .map(|f| vec![item("from", f.clone())])
            .unwrap_or_default(),

        // O fluxo não é coleção, e por isso eu o deixei sem filhos no
        // ciclo 283 — raciocinando que o estado dele já estava no texto.
        // Só que o ciclo 284 parou de mostrar o texto do embed (era o
        // fence YAML inteiro), e as duas decisões juntas apagaram a
        // informação: o arquivo diz `artefato: spec, etapa: concluida` e
        // a tela não dizia nada.
        //
        // Cada decisão certa sozinha, erradas juntas — o mesmo tipo de
        // composição silenciosa que o ciclo 285 achou na costura.
        // O fluxo declara o CARTÃO dele (ciclo 302), na mesma ordem que
        // a janela desenha: título, trilha de etapas, a ação principal
        // daquele estado, e as transições possíveis.
        //
        // As transições são DOMÍNIO, não decoração: saem de
        // `Etapa::proximas()`, que é quem sabe que de "em revisão" se vai
        // pra aprovada, rascunho ou bloqueada — e que não se pula a
        // revisão indo direto pra execução.
        //
        // Isso é derivável da própria página, diferente das linhas de
        // uma consulta: aqui não se pergunta nada ao vault, só ao estado
        // que está escrito no embed.
        EmbedData::Fluxo(d) => {
            use crate::fluxo::Etapa;
            let mut partes = vec![
                item(
                    "titulo",
                    format!(
                        "{}: {}",
                        d.artefato.label().to_uppercase(),
                        d.etapa.label()
                    ),
                ),
                // A trilha inteira, com a atual marcada pelo NOME da
                // parte — quem desenha pinta diferente sem ter que
                // comparar texto.
                fileira(
                    "etapas",
                    Etapa::all()
                        .iter()
                        .map(|e| {
                            item(
                                if *e == d.etapa { "etapa-atual" } else { "etapa" },
                                e.label(),
                            )
                        })
                        .collect(),
                ),
            ];
            // A ação é um BOTÃO, não um rótulo com um filho (ciclo
            // 302). Na janela ela é `<button>`, e a dica é o texto ao
            // lado; aqui era um grupo, e o desenho não tinha como
            // saber que aquilo se aperta.
            //
            // Botão é FOLHA DE FILEIRA — a mesma forma das transições.
            // Quem desenha pergunta o arranjo, não o nome da parte.
            if let Some((rotulo, dica)) = acao_do_fluxo(d) {
                partes.push(fileira("acao", vec![item("acao", rotulo)]));
                partes.push(item("dica", dica));
            }
            partes.push(fileira(
                "transicoes",
                d.etapa
                    .proximas()
                    .into_iter()
                    .map(|destino| {
                        item(
                            if d.etapa.avanco_natural() == Some(destino) {
                                "transicao-principal"
                            } else {
                                "transicao"
                            },
                            destino.label(),
                        )
                    })
                    .collect(),
            ));
            if let Some(nota) = d.nota.as_ref().filter(|n| !n.trim().is_empty()) {
                partes.push(item("nota", nota.clone()));
            }
            partes
        }
    }
}

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

/// Costura duas versões do corpo, preservando o que não mudou.
///
/// É o passo 4 chegando ao arquivo (ciclo 276). Hoje o editor reconstrói
/// o corpo INTEIRO a partir do DOM ao salvar: cada embed é reserializado
/// e cada trecho de markdown volta pela travessia do HTML. O resultado é
/// que abrir uma página e salvar já muda bytes que ninguém tocou —
/// aspas, ordem de campos de YAML, espaçamento.
///
/// Aqui a ÁRVORE é quem decide o que mudou. Unidade estruturalmente
/// igual volta pelos bytes ORIGINAIS; o resto vem da versão nova.
///
/// Quando as duas versões têm contagens diferentes de unidades — bloco
/// inserido, apagado, dividido —, devolve a nova inteira. Alinhar
/// árvores de tamanhos diferentes é um problema de diff, e errar nele
/// custaria conteúdo; devolver a nova é o que o editor já fazia, então o
/// pior caso é o comportamento de hoje.
pub fn costurar_mudancas(original: &str, novo: &str) -> String {
    let arvore_velha = analisar(original);
    let arvore_nova = analisar(novo);

    // Alinha as duas listas de unidades pela maior subsequência comum.
    //
    // Comparar par a par por posição não serve, e isso foi MEDIDO: a
    // volta pelo DOM não muda só formatação, muda ESTRUTURA. Um
    // parágrafo com quebra forte vira dois blocos, e a partir daí todas
    // as posições saem de sincronia — com comparação posicional, uma
    // edição em qualquer lugar reescrevia o arquivo inteiro.
    //
    // Com o alinhamento, o que casa volta pelos bytes originais mesmo
    // que tenha andado de posição.
    let velhas = &arvore_velha.filhos;
    let novas = &arvore_nova.filhos;
    let pares = alinhar(velhas, novas);

    let mut fora = String::new();
    let mut cursor_novo = 0usize;
    for (i_nova, nova) in novas.iter().enumerate() {
        let Some(faixa_nova) = &nova.intervalo else {
            return novo.to_string();
        };
        // O que vem ANTES desta unidade sai da versão nova: é lá que
        // está a separação que a edição pode ter mexido.
        fora.push_str(&novo[cursor_novo..faixa_nova.start]);
        match pares.get(&i_nova).and_then(|i| velhas[*i].intervalo.clone()) {
            Some(faixa_velha) if faixa_velha.end <= original.len() => {
                fora.push_str(&original[faixa_velha]);
            }
            _ => fora.push_str(&novo[faixa_nova.clone()]),
        }
        cursor_novo = faixa_nova.end;
    }
    fora.push_str(&novo[cursor_novo..]);
    fora
}

/// Casa unidades novas com velhas pela maior subsequência comum.
///
/// Devolve `índice na nova -> índice na velha` só pras que casaram. A
/// ordem é respeitada: uma unidade não casa com outra que esteja "atrás"
/// de um casamento anterior, senão o texto sairia embaralhado.
/// Acima disto, o miolo não é mais alinhado por LCS.
///
/// A tabela é `O(n×m)` em tempo E memória. Com o prefixo e o sufixo
/// aparados, o miolo é o que a pessoa realmente mexeu — dezenas de
/// unidades no pior caso realista. Se ainda assim for enorme, é porque
/// a página inteira mudou; aí não há o que costurar, e insistir no LCS
/// custaria mais do que o ganho.
const MIOLO_MAXIMO: usize = 400;

fn alinhar(velhas: &[Unidade], novas: &[Unidade]) -> std::collections::HashMap<usize, usize> {
    // Apara o que é igual nas pontas ANTES do LCS (ciclo 282).
    //
    // Eu escrevi aqui que "as páginas têm dezenas de unidades, então
    // O(n×m) é ruído", e nunca medi. A bateria de estresse tem uma
    // página de 1200 blocos, e esta função roda a cada TECLA, pelo
    // `oninput` do editor: 1,44 milhão de comparações de unidade por
    // caractere digitado. O webview parava de responder.
    //
    // Digitar muda UMA unidade. Aparando as pontas, o problema que sobra
    // tem o tamanho da edição, não o da página — e a resposta é a mesma,
    // porque unidade igual na mesma ponta casa consigo em qualquer
    // alinhamento ótimo.
    let mut pares = std::collections::HashMap::new();

    let mut inicio = 0usize;
    while inicio < velhas.len()
        && inicio < novas.len()
        && mesma_intencao(&velhas[inicio], &novas[inicio])
    {
        pares.insert(inicio, inicio);
        inicio += 1;
    }

    let mut fim = 0usize;
    while fim < (velhas.len() - inicio).min(novas.len() - inicio)
        && mesma_intencao(
            &velhas[velhas.len() - 1 - fim],
            &novas[novas.len() - 1 - fim],
        )
    {
        pares.insert(novas.len() - 1 - fim, velhas.len() - 1 - fim);
        fim += 1;
    }

    let velhas_miolo = &velhas[inicio..velhas.len() - fim];
    let novas_miolo = &novas[inicio..novas.len() - fim];
    let (n, m) = (velhas_miolo.len(), novas_miolo.len());
    if n == 0 || m == 0 {
        return pares;
    }
    if n.saturating_mul(m) > MIOLO_MAXIMO * MIOLO_MAXIMO {
        // Página inteira reescrita: o que não casou pelas pontas volta
        // pelos bytes novos, que é o comportamento de antes da costura.
        return pares;
    }

    let (velhas, novas) = (velhas_miolo, novas_miolo);
    // Tabela clássica de LCS sobre o MIOLO.
    let mut tabela = vec![vec![0usize; m + 1]; n + 1];
    for i in (0..n).rev() {
        for j in (0..m).rev() {
            tabela[i][j] = if mesma_intencao(&velhas[i], &novas[j]) {
                tabela[i + 1][j + 1] + 1
            } else {
                tabela[i + 1][j].max(tabela[i][j + 1])
            };
        }
    }
    let (mut i, mut j) = (0usize, 0usize);
    while i < n && j < m {
        if mesma_intencao(&velhas[i], &novas[j]) {
            // Os índices do miolo voltam pros do documento inteiro.
            pares.insert(inicio + j, inicio + i);
            i += 1;
            j += 1;
        } else if tabela[i + 1][j] >= tabela[i][j + 1] {
            i += 1;
        } else {
            j += 1;
        }
    }
    pares
}

/// As duas unidades dizem a mesma coisa?
///
/// Mais frouxo que `==`: ignora espaço no FIM das linhas.
///
/// A razão é concreta. O editor sempre apara espaço final ao trazer o
/// texto de volta do DOM — e em markdown dois espaços no fim são uma
/// QUEBRA FORTE. Comparando byte a byte, todo parágrafo com quebra forte
/// seria marcado como editado, reescrito pela versão do editor, e a
/// quebra sumiria. O usuário nunca tocou nele.
///
/// A troca assumida: quem apagar espaços finais de propósito não vê o
/// efeito. É um gesto que o editor não sabe expressar de qualquer forma.
fn mesma_intencao(a: &Unidade, b: &Unidade) -> bool {
    /// Espaço em branco vira um espaço só, e some das pontas.
    ///
    /// Vale porque o DOM não sabe representar espaço branco fielmente:
    /// a quebra dentro de um item de lista, o recuo da continuação, os
    /// dois espaços da quebra forte — tudo isso volta achatado, e o
    /// usuário não tocou em nada.
    ///
    /// Uma edição de verdade mexe em caractere que não é espaço.
    /// Inserir um espaço entre duas palavras (`a.B` -> `a. B`) continua
    /// contando como mudança, porque não é uma RUN de espaço que muda,
    /// é a presença dele.
    fn achatado(t: &str) -> String {
        t.split_whitespace().collect::<Vec<_>>().join(" ")
    }
    if a.tipo != b.tipo {
        return false;
    }
    // Dentro de código, espaço é conteúdo. Achatar ali apagaria
    // indentação que importa.
    let texto_bate = if matches!(a.tipo, Tipo::Codigo(_)) {
        a.texto == b.texto
    } else {
        achatado(&a.texto) == achatado(&b.texto)
    };
    texto_bate
        && a.filhos.len() == b.filhos.len()
        && a.filhos.iter().zip(b.filhos.iter()).all(|(x, y)| mesma_intencao(x, y))
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

#[cfg(test)]
mod costura_de_mudancas {
    use super::*;

    #[test]
    fn nada_mudou_devolve_o_original_byte_a_byte() {
        // O caso que motiva tudo: abrir e salvar sem editar não pode
        // mudar o arquivo. Aqui o "novo" chega normalizado (aspas
        // trocadas, espaçamento diferente), como o editor devolve hoje.
        let original = "# T\n\n- a\n   recuo\n- b\n\ntexto   \n";
        let normalizado = "# T\n\n- a\n  recuo\n- b\n\ntexto\n";
        // As árvores são iguais em estrutura, então tudo volta original.
        assert_eq!(costurar_mudancas(original, normalizado), original);
    }

    #[test]
    fn so_o_bloco_editado_muda() {
        let original = "# Título\n\nprimeiro\n\nsegundo   \n";
        let novo = "# Título\n\nPRIMEIRO editado\n\nsegundo\n";
        let saida = costurar_mudancas(original, novo);
        assert!(saida.contains("PRIMEIRO editado"), "a edição não entrou:\n{saida}");
        // O terceiro bloco não foi tocado: os dois espaços do fim
        // sobrevivem, mesmo o editor tendo devolvido sem eles.
        assert!(saida.contains("segundo   "), "o vizinho foi normalizado:\n{saida}");
    }

    #[test]
    fn o_embed_intocado_mantem_o_yaml_como_estava() {
        // O ganho mais visível: o editor reserializa todo embed ao
        // salvar, reordenando campos que ninguém tocou.
        let original = "{{ type: \"callout\" }}\ntitle: Nota\nvariant: info\n{{ /callout }}\n\nfim\n";
        let reserializado = "{{ type: \"callout\" }}\nvariant: info\ntitle: Nota\n{{ /callout }}\n\nfim\n";
        let saida = costurar_mudancas(original, reserializado);
        assert!(
            saida.starts_with("{{ type: \"callout\" }}\ntitle: Nota"),
            "a ordem do YAML mudou sem ninguém editar:\n{saida}"
        );
    }

    #[test]
    fn bloco_inserido_no_meio_nao_reformata_os_vizinhos() {
        // Contagens diferentes já não desistem: o alinhamento acha quem
        // é quem. Aqui os espaços do fim de "um" e "dois" sobrevivem à
        // inserção de um bloco entre eles.
        let original = "um   \n\ndois   \n";
        let novo = "um\n\nmeio\n\ndois\n";
        let saida = costurar_mudancas(original, novo);
        assert!(saida.contains("um   "), "o de cima foi normalizado:\n{saida:?}");
        assert!(saida.contains("dois   "), "o de baixo foi normalizado:\n{saida:?}");
        assert!(saida.contains("meio"), "o bloco novo não entrou:\n{saida:?}");
    }

    #[test]
    fn bloco_partido_em_dois_preserva_o_resto() {
        // O caso que a medição encontrou: a volta pelo DOM parte um
        // parágrafo com quebra forte em dois blocos. Antes do
        // alinhamento, isso dessincronizava tudo e o arquivo inteiro era
        // reescrito.
        let original = "# T\n\na  \nb\n\n- item\n   recuo\n\nfim   \n";
        let novo = "# T\n\na\n\nb\n\n- item\n  recuo\n\nfim\n";
        let saida = costurar_mudancas(original, novo);
        assert!(saida.contains("   recuo"), "a lista foi reformatada:\n{saida:?}");
        assert!(saida.contains("fim   "), "o último bloco foi normalizado:\n{saida:?}");
    }

    #[test]
    fn bloco_apagado_some_e_os_outros_ficam() {
        let original = "um   \n\ndois\n\ntres   \n";
        let novo = "um\n\ntres\n";
        let saida = costurar_mudancas(original, novo);
        assert!(!saida.contains("dois"), "o bloco apagado voltou:\n{saida:?}");
        assert!(saida.contains("um   "), "{saida:?}");
        assert!(saida.contains("tres   "), "{saida:?}");
    }

    #[test]
    fn apagar_tudo_devolve_vazio_e_nao_o_original() {
        // A guarda que importa na direção contrária: se o novo está
        // vazio, isto não pode "restaurar" o original. Quem decide se
        // uma gravação vazia passa é a trava do ciclo 248, não daqui.
        assert_eq!(costurar_mudancas("um\n\ndois\n", ""), "");
    }
}

#[cfg(test)]
mod comparacao {
    use super::*;

    #[test]
    fn espaco_achatado_nao_conta_como_edicao() {
        // O DOM não representa a quebra dentro de um item nem o recuo da
        // continuação; tudo volta achatado. Marcar isso como edição
        // fazia o vizinho ser reescrito por culpa do editor.
        let a = Unidade::com_texto(Tipo::Item, "item\n   continuação");
        let b = Unidade::com_texto(Tipo::Item, "item continuação");
        assert!(mesma_intencao(&a, &b));
    }

    #[test]
    fn inserir_um_espaco_entre_palavras_conta() {
        // A guarda do outro lado: uma edição de verdade não pode passar
        // por artefato.
        let a = Unidade::com_texto(Tipo::Paragrafo, "frase.Outra");
        let b = Unidade::com_texto(Tipo::Paragrafo, "frase. Outra");
        assert!(!mesma_intencao(&a, &b));
    }

    #[test]
    fn dentro_de_codigo_o_espaco_e_conteudo() {
        let a = Unidade::com_texto(Tipo::Codigo(None), "fn f() {\n    1\n}");
        let b = Unidade::com_texto(Tipo::Codigo(None), "fn f() {\n1\n}");
        assert!(!mesma_intencao(&a, &b), "achatou indentação de código");
    }

    #[test]
    fn tipos_diferentes_nunca_batem() {
        let a = Unidade::com_texto(Tipo::Paragrafo, "x");
        let b = Unidade::com_texto(Tipo::Titulo(1), "x");
        assert!(!mesma_intencao(&a, &b));
    }
    /// Documento grande com um bloco editado no fim.
    fn pagina_grande(n: usize) -> String {
        (0..n)
            .map(|i| format!("Parágrafo {i} com texto.\n"))
            .collect::<Vec<_>>()
            .join("\n")
    }

    #[test]
    fn costurar_pagina_grande_preserva_o_que_ninguem_tocou() {
        // O caso que a bateria de estresse tem: página grande, uma
        // edição só. Antes do ciclo 282 isto montava uma tabela de
        // n×m e rodava a cada TECLA — 1,44 milhão de comparações por
        // caractere numa página de 1200 blocos.
        let original = pagina_grande(1200);
        let novo = original.replace("Parágrafo 1199 com texto.", "Parágrafo 1199 EDITADO.");

        let fora = costurar_mudancas(&original, &novo);

        assert!(fora.contains("Parágrafo 1199 EDITADO."), "a edição não entrou");
        // Tudo antes dela volta pelos bytes originais.
        assert!(fora.contains("Parágrafo 0 com texto."));
        assert!(fora.contains("Parágrafo 1198 com texto."));
        assert_eq!(fora.matches("com texto.").count(), 1199);
    }

    #[test]
    fn o_aparo_das_pontas_da_a_mesma_resposta_do_lcs() {
        // Bloco INSERIDO no meio: as pontas casam, o miolo é o que o
        // LCS resolve. É o caso que o ciclo 276 media, e o aparo não
        // pode mudá-lo.
        let original = "um\n\ndois\n\ntres\n";
        let novo = "um\n\nNOVO\n\ndois\n\ntres\n";
        let fora = costurar_mudancas(original, novo);
        assert!(fora.contains("NOVO"));
        assert!(fora.contains("um"));
        assert!(fora.contains("tres"));
    }

    #[test]
    fn pagina_inteira_reescrita_nao_monta_tabela() {
        // Nada casa nas pontas e o miolo passa do teto: a costura
        // desiste e devolve o texto novo, que é o comportamento de
        // antes dela existir. Desistir é a resposta certa — não há o
        // que preservar.
        let original = pagina_grande(600);
        let novo = (0..600)
            .map(|i| format!("Outra coisa {i} aqui.\n"))
            .collect::<Vec<_>>()
            .join("\n");
        let fora = costurar_mudancas(&original, &novo);
        assert_eq!(fora, novo);
    }
}

/// O conteúdo declarado por cada embed (ciclo 283).
#[cfg(test)]
mod partes_de_embed {
    use super::*;

    /// Os resumos dos filhos diretos, em ordem.
    fn partes(u: &Unidade) -> Vec<String> {
        u.filhos.iter().map(|f| f.tipo.resumo()).collect()
    }

    /// O embed sozinho numa página, já expandido.
    fn embed_de(md: &str) -> Unidade {
        let d = analisar(md);
        assert_eq!(d.filhos.len(), 1, "esperava um embed só: {:?}", partes(&d));
        d.filhos[0].clone()
    }



    #[test]
    fn o_embed_continua_atomico_depois_de_ganhar_filhos() {
        // A garantia que torna este ciclo seguro: `navegaveis()` para no
        // atômico, então a GUI não vê nada disto e nenhum caminho muda.
        // Quem desce é `percorrer`, e é o terminal que a usa.
        let e = embed_de(
            "{{ type: \"gallery\" }}\nitems:\n- path: a.png\n  caption: A\n{{ /gallery }}\n",
        );
        assert!(e.politica().atomica);
        assert_eq!(e.filhos.len(), 1);

        let d = analisar(
            "{{ type: \"gallery\" }}\nitems:\n- path: a.png\n  caption: A\n{{ /gallery }}\n",
        );
        // Um destino só: o embed. A imagem está na árvore e não na
        // navegação.
        assert_eq!(d.navegaveis().len(), 1);
        assert_eq!(d.percorrer().len(), 2);
    }

    #[test]
    fn o_kanban_declara_colunas_com_seus_cartoes() {
        let e = embed_de(
            "{{ type: \"kanban\" }}\ncolumns:\n- Backlog\n- Feito\nitems:\n- title: Card A\n  column: Backlog\n- title: Card B\n  column: Feito\n{{ /kanban }}\n",
        );
        assert_eq!(partes(&e), ["parte:column", "parte:column"]);
        assert_eq!(e.filhos[0].texto, "Backlog");
        assert_eq!(partes(&e.filhos[0]), ["parte:card"]);
        assert_eq!(e.filhos[0].filhos[0].texto, "Card A");
        assert_eq!(e.filhos[1].filhos[0].texto, "Card B");
    }

    #[test]
    fn a_tabela_declara_cabecalho_e_linhas() {
        let e = embed_de(
            "{{ type: \"table\" }}\ncolumns:\n- name: Tarefa\n- name: Status\n---\n| Tarefa | Status |\n| --- | --- |\n| API | done |\n{{ /table }}\n",
        );
        assert_eq!(partes(&e), ["parte:header", "parte:row"]);
        assert_eq!(
            e.filhos[0]
                .filhos
                .iter()
                .map(|c| c.texto.clone())
                .collect::<Vec<_>>(),
            ["Tarefa", "Status"]
        );
        assert_eq!(
            e.filhos[1]
                .filhos
                .iter()
                .map(|c| c.texto.clone())
                .collect::<Vec<_>>(),
            ["API", "done"]
        );
        // Cabeçalho e linha são FILEIRA (ciclo 305): célula ao lado de
        // célula é o desenho de uma tabela, não uma embaixo da outra.
        assert_eq!(e.filhos[0].tipo.arranjo(), Arranjo::Linha);
        assert_eq!(e.filhos[1].tipo.arranjo(), Arranjo::Linha);
    }

    #[test]
    fn celula_de_select_e_multiselect_carrega_a_cor_do_badge() {
        // O mesmo `badge_class` que a janela usa pra pintar
        // `.badge--info`/`--success`/`--warning`/`--error` — só que
        // aqui vira o NOME da parte, e é o gancho que o desenho do
        // terminal usa (ciclo 305).
        let e = embed_de(
            "{{ type: \"table\" }}\ncolumns:\n\
             - name: Tarefa\n\
             - name: Status\n  type: select\n  options: [todo, doing, done]\n\
             - name: Tags\n  type: multiselect\n  options: [urgente, bug, infra]\n\
             ---\n\
             | Tarefa | Status | Tags |\n| --- | --- | --- |\n\
             | API | done | infra |\n\
             | UI | doing | urgente, bug |\n\
             {{ /table }}\n",
        );
        assert_eq!(partes(&e), ["parte:header", "parte:row", "parte:row"]);
        // options=[todo,doing,done]: done é a TERCEIRA (índice 2).
        assert_eq!(partes(&e.filhos[1]), ["parte:cell", "parte:badge--warning", "parte:badge--warning"]);
        // A cor vem da PRIMEIRA tag ("infra", índice 2 em
        // [urgente,bug,infra]) — mostrar as duas juntas em badges
        // separados fica pro próximo corte.
        assert_eq!(e.filhos[1].filhos[2].texto, "infra");
        // options=[todo,doing,done]: doing é a segunda (índice 1).
        assert_eq!(partes(&e.filhos[2]), ["parte:cell", "parte:badge--success", "parte:badge--info"]);
        assert_eq!(e.filhos[2].filhos[2].texto, "urgente, bug");
    }

    /// O calendário da página de exemplos, mínimo.
    fn calendario_de_agosto() -> Unidade {
        embed_de(
            "{{ type: \"calendar\" }}\nentries:\n\
             - date: 2026-08-06\n  title: Revisão de código\n  tags:\n  - urgente\n\
             - date: 2026-08-10\n  title: Sprint de agosto\n  end_date: 2026-08-14\n  tags:\n  - infra\n\
             - date: 2026-08-12\n  title: Reunião\n\
             - title: Ligar pro fornecedor\n\
             {{ /calendar }}\n",
        )
    }

    #[test]
    fn o_calendario_vira_grade_do_mes() {
        let c = calendario_de_agosto();
        assert_eq!(partes(&c), ["parte:mes", "parte:sem-data"]);
        let mes = &c.filhos[0];
        assert_eq!(mes.texto, "Agosto 2026");
        // Agosto de 2026 começa num sábado e tem 31 dias: são seis
        // semanas com dia do mês (a última, 30/08 a 05/09).
        assert_eq!(mes.filhos.len(), 6);
        let semana1 = &mes.filhos[0];
        assert_eq!(semana1.tipo.arranjo(), Arranjo::Linha);
        assert_eq!(
            semana1.filhos.iter().map(|d| d.texto.as_str()).collect::<Vec<_>>(),
            ["26", "27", "28", "29", "30", "31", "1"]
        );
        assert_eq!(partes(semana1)[..2], ["parte:dia-fora", "parte:dia-fora"]);
        assert_eq!(partes(semana1)[6], "parte:dia");
    }

    #[test]
    fn o_evento_ocupa_a_faixa_e_a_barra_continua_nos_dias_seguintes() {
        let c = calendario_de_agosto();
        // Semana de 9 a 15: Sprint de 10 a 14 na faixa 0, Reunião no dia
        // 12 na faixa 1 — então TODO dia da semana tem duas faixas.
        let semana = &c.filhos[0].filhos[2];
        assert!(semana.filhos.iter().all(|d| d.filhos.len() == 2));
        // tags do calendário: [infra, urgente] → infra é índice 0.
        assert_eq!(partes(&semana.filhos[1]), ["parte:evento--info", "parte:vazio"]);
        assert_eq!(semana.filhos[1].filhos[0].texto, "Sprint de agosto");
        assert_eq!(partes(&semana.filhos[3]), ["parte:evento-continua", "parte:evento"]);
        assert_eq!(semana.filhos[3].filhos[1].texto, "Reunião");
        assert_eq!(partes(&semana.filhos[6]), ["parte:vazio", "parte:vazio"]);
        // Semana de 2 a 8: Revisão (urgente, índice 1) só no dia 6.
        let semana2 = &c.filhos[0].filhos[1];
        assert_eq!(partes(&semana2.filhos[4]), ["parte:evento--success"]);
    }

    #[test]
    fn evento_sem_data_vai_pra_gaveta() {
        let c = calendario_de_agosto();
        let gaveta = &c.filhos[1];
        assert_eq!(gaveta.texto, "Sem data (1)");
        assert_eq!(partes(gaveta), ["parte:entry"]);
        assert_eq!(gaveta.filhos[0].texto, "Ligar pro fornecedor");
    }

    #[test]
    fn celula_comum_nao_ganha_cor_de_badge() {
        let e = embed_de(
            "{{ type: \"table\" }}\ncolumns:\n- name: Tarefa\n  type: number\n---\n| Tarefa |\n| --- |\n| 8 |\n{{ /table }}\n",
        );
        assert_eq!(partes(&e.filhos[1]), ["parte:cell"]);
    }

    #[test]
    fn o_callout_traz_markdown_de_verdade() {
        // O corpo dele é do mesmo tecido da página, então vira unidade
        // de markdown — não "parte".
        let e = embed_de(
            "{{ type: \"callout\" }}\nvariant: info\nbody: |\n  # Título\n\n  Um parágrafo.\n{{ /callout }}\n",
        );
        assert_eq!(partes(&e), ["parte:variante", "titulo1", "paragrafo"]);
        assert_eq!(e.filhos[0].texto, "info");
    }

    #[test]
    fn o_callout_declara_a_variante_e_o_titulo() {
        let e = embed_de(
            "{{ type: \"callout\" }}\nvariant: warning\ntitle: Cuidado\nbody: |\n  Texto.\n{{ /callout }}\n",
        );
        assert_eq!(partes(&e), ["parte:variante", "parte:titulo", "paragrafo"]);
        assert_eq!(e.filhos[0].texto, "warning");
        assert_eq!(e.filhos[1].texto, "Cuidado");
        // Variante desconhecida cai no padrão, como na janela.
        let d = embed_de(
            "{{ type: \"callout\" }}\nvariant: roxo\nbody: |\n  Texto.\n{{ /callout }}\n",
        );
        assert_eq!(d.filhos[0].texto, "info");
    }

    #[test]
    fn as_colunas_sao_paineis_de_markdown() {
        let e = embed_de(
            "{{ type: \"columns\" }}\ncolumns:\n- width: 1\n  body: |\n    Esquerda.\n- width: 2\n  body: |\n    Direita.\n{{ /columns }}\n",
        );
        assert_eq!(partes(&e), ["parte:pane", "parte:pane"]);
        assert_eq!(partes(&e.filhos[0]), ["paragrafo"]);
        assert_eq!(e.filhos[0].filhos[0].texto, "Esquerda.");
    }

    #[test]
    fn timeline_galeria_e_acoes_declaram_seus_itens() {
        let t = embed_de(
            "{{ type: \"timeline\" }}\nitems:\n- title: Etapa um\n  start: '2026-01-01'\n- title: Etapa dois\n  start: '2026-02-01'\n{{ /timeline }}\n",
        );
        // "barra", não "item" (ciclo 305): os dois têm data, então os
        // dois viram retângulo proporcional — não rótulo solto.
        assert_eq!(partes(&t), ["parte:barra", "parte:barra"]);
        assert_eq!(t.filhos[0].texto, "Etapa um");

        let g = embed_de(
            "{{ type: \"gallery\" }}\nitems:\n- path: a.png\n  caption: Legenda\n- path: b.png\n{{ /gallery }}\n",
        );
        // Sem legenda, o caminho identifica.
        assert_eq!(
            g.filhos.iter().map(|i| i.texto.clone()).collect::<Vec<_>>(),
            ["Legenda", "b.png"]
        );
        // "miniatura", não "item" (ciclo 304): o nome é o gancho que o
        // desenho do terminal usa pra pintar o selo que faz as vezes da
        // imagem que ele não consegue mostrar.
        assert_eq!(partes(&g), ["parte:miniatura", "parte:miniatura"]);

        let a = embed_de(
            "{{ type: \"actions\" }}\nbuttons:\n- label: Abrir\n  action: open-page\n{{ /actions }}\n",
        );
        // Os botões ficam numa FILEIRA declarada (ciclo 297): o galho
        // diz que os filhos vão lado a lado, e o desenho e a navegação
        // seguem daí em vez de adivinhar pela tela.
        assert_eq!(partes(&a), ["parte:acoes"]);
        assert_eq!(a.filhos[0].tipo.arranjo(), Arranjo::Linha);
        assert_eq!(partes(&a.filhos[0]), ["parte:button"]);
        assert_eq!(a.filhos[0].filhos[0].texto, "Abrir");
    }

    #[test]
    fn a_barra_do_cronograma_carrega_inicio_e_duracao_em_porcentagem() {
        // A mesma conta da página de exemplos: três itens de agosto,
        // 29 dias de janela (do primeiro início ao último fim).
        let t = embed_de(
            "{{ type: \"timeline\" }}\nitems:\n\
             - title: Levantar requisitos\n  start: '2026-08-03'\n  end: '2026-08-10'\n\
             - title: Implementar\n  start: '2026-08-11'\n  end: '2026-08-24'\n\
             - title: Revisar e publicar\n  start: '2026-08-25'\n  end: '2026-08-31'\n\
             {{ /timeline }}\n",
        );
        assert_eq!(partes(&t), ["parte:barra", "parte:barra", "parte:barra"]);

        fn geometria(barra: &Unidade) -> (String, String) {
            let campo = |nome: &str| {
                barra
                    .filhos
                    .iter()
                    .find(|f| matches!(&f.tipo, Tipo::Parte { nome: n, .. } if n == nome))
                    .map(|f| f.texto.clone())
                    .unwrap_or_default()
            };
            (campo("inicio"), campo("duracao"))
        }

        // A primeira barra começa NO início da janela (0%) — é ela quem
        // define a janela.
        assert_eq!(geometria(&t.filhos[0]), ("0".to_string(), "28".to_string()));
        // A segunda emenda onde a primeira parou.
        assert_eq!(geometria(&t.filhos[1]), ("28".to_string(), "48".to_string()));
        // A terceira termina NO fim da janela.
        let (inicio3, duracao3) = geometria(&t.filhos[2]);
        assert_eq!(inicio3, "76".to_string());
        assert_eq!(duracao3, "24".to_string());
    }

    #[test]
    fn item_do_cronograma_sem_data_nao_vira_barra() {
        // Fica na "gaveta" da janela — sem início não há o que
        // desenhar proporcional, então volta a ser rótulo comum.
        let t = embed_de(
            "{{ type: \"timeline\" }}\nitems:\n- title: Sem data\n{{ /timeline }}\n",
        );
        assert_eq!(partes(&t), ["parte:item"]);
        assert_eq!(t.filhos[0].texto, "Sem data");
    }

    #[test]
    fn o_botao_primario_das_acoes_vira_parte_propria() {
        // `variant: primary` já era lido pela janela pra pintar
        // `.actions-embed__btn--primary` de destaque; até este ciclo o
        // terminal jogava fora o campo e todo botão saía "button" —
        // igual, sem distinção nenhuma (ciclo 304).
        let a = embed_de(
            "{{ type: \"actions\" }}\nbuttons:\n- label: Cancelar\n  action: open-page\n  path: pages/a.md\n- label: Confirmar\n  action: open-page\n  path: pages/b.md\n  variant: primary\n{{ /actions }}\n",
        );
        assert_eq!(
            partes(&a.filhos[0]),
            ["parte:button", "parte:button-primary"]
        );
        assert_eq!(a.filhos[0].filhos[1].texto, "Confirmar");

        // Qualquer outro valor — inclusive vazio — é o estilo fantasma,
        // igual à janela: `variant` só destaca quando é `primary`.
        let b = embed_de(
            "{{ type: \"actions\" }}\nbuttons:\n- label: X\n  action: open-page\n  variant: secondary\n{{ /actions }}\n",
        );
        assert_eq!(partes(&b.filhos[0]), ["parte:button"]);
    }

    #[test]
    fn o_fluxo_declara_o_artefato_e_a_etapa() {
        // Ele era o único embed sem filhos, e como o ciclo 284 parou de
        // mostrar o texto do embed, aparecia como `[fluxo]` e mais nada.
        let e = embed_de(
            "{{ type: \"fluxo\" }}\nartefato: spec\netapa: concluida\n{{ /fluxo }}\n",
        );
        // O cartão inteiro, na ordem da janela (ciclo 302).
        assert_eq!(
            partes(&e),
            ["parte:titulo", "parte:etapas", "parte:transicoes"]
        );
        assert_eq!(e.filhos[0].texto, "SPEC: Concluída");
    }

    #[test]
    fn as_transicoes_do_fluxo_saem_do_dominio() {
        // Os botões não são lista minha: vêm de `Etapa::proximas()`, que
        // é quem sabe que de "em revisão" se vai pra aprovada, rascunho
        // ou bloqueada — e que NÃO se pula a revisão indo direto pra
        // execução. Escrever a lista aqui seria uma segunda fonte da
        // mesma verdade, como a política do bloco até o ciclo 278.
        let e = embed_de(
            "{{ type: \"fluxo\" }}\nartefato: proposta\netapa: em-revisao\n{{ /fluxo }}\n",
        );
        let transicoes = e
            .filhos
            .iter()
            .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "transicoes"))
            .expect("faltaram as transições");
        let rotulos: Vec<String> = transicoes.filhos.iter().map(|f| f.texto.clone()).collect();
        let esperado: Vec<String> = crate::fluxo::Etapa::EmRevisao
            .proximas()
            .iter()
            .map(|e| e.label().to_string())
            .collect();
        assert_eq!(rotulos, esperado);
    }

    #[test]
    fn a_etapa_atual_se_identifica_pelo_nome_da_parte() {
        // Quem desenha pinta a atual diferente sem ter que comparar
        // texto — comparar texto quebraria no dia em que um rótulo
        // mudasse.
        let e = embed_de(
            "{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n",
        );
        let trilha = &e.filhos[1];
        let atuais: Vec<&str> = trilha
            .filhos
            .iter()
            .filter(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "etapa-atual"))
            .map(|f| f.texto.as_str())
            .collect();
        assert_eq!(atuais, ["Aprovada"], "esperava UMA etapa atual");
        assert_eq!(trilha.filhos.len(), crate::fluxo::Etapa::all().len());
    }

    #[test]
    fn a_acao_principal_depende_do_estado_e_do_artefato() {
        // Vem da janela (ciclos 209 e 223): aprovada é onde o trabalho
        // passa do "o quê" pro "como"; em revisão ganhou a terceira
        // saída.
        // A ação é BOTÃO: folha de uma fileira, como as transições.
        // Antes era um grupo com a dica dentro, e o desenho não tinha
        // como saber que aquilo se aperta (ciclo 302).
        let tem_acao = |md: &str| {
            embed_de(md)
                .filhos
                .iter()
                .filter(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "acao"))
                .flat_map(|f| f.filhos.iter())
                .map(|b| b.texto.clone())
                .next()
        };
        assert_eq!(
            tem_acao("{{ type: \"fluxo\" }}\nartefato: spec\netapa: aprovada\n{{ /fluxo }}\n"),
            Some("Planejar implementação".to_string())
        );
        assert_eq!(
            tem_acao("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n"),
            Some("Executar".to_string())
        );
        assert_eq!(
            tem_acao("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: em-revisao\n{{ /fluxo }}\n"),
            Some("Pedir alteração".to_string())
        );
        // Rascunho não tem ação principal, e execução não é spec nem
        // proposta.
        assert_eq!(
            tem_acao("{{ type: \"fluxo\" }}\nartefato: proposta\netapa: rascunho\n{{ /fluxo }}\n"),
            None
        );
        assert_eq!(
            tem_acao("{{ type: \"fluxo\" }}\nartefato: execucao\netapa: aprovada\n{{ /fluxo }}\n"),
            None
        );
    }

    #[test]
    fn a_consulta_declara_o_que_esta_escrito_nao_o_resultado() {
        // As linhas vêm do vault, e `analisar` é função pura do corpo da
        // página. Declarar linhas aqui seria inventar.
        let e = embed_de("{{ type: \"query\" }}\nfrom: pages\nview: list\n{{ /query }}\n");
        assert_eq!(partes(&e), ["parte:from"]);
        assert_eq!(e.filhos[0].texto, "pages");
    }

    #[test]
    fn o_controle_da_gui_nao_entra_na_arvore() {
        // O DOM da galeria tem 16 itens navegáveis; 2 são as imagens, o
        // resto é botão (tamanho, mover, remover, adicionar). Nenhum
        // deles existe no markdown, e um terminal desenharia os seus.
        let e = embed_de(
            "{{ type: \"gallery\" }}\nitems:\n- path: a.png\n- path: b.png\n{{ /gallery }}\n",
        );
        assert_eq!(e.filhos.len(), 2, "entrou controle na árvore: {:?}", partes(&e));
    }

    #[test]
    fn a_expansao_nao_muda_o_markdown_de_volta() {
        // A garantia de fidelidade: o embed volta pela fonte, e os
        // filhos não são reescritos (`desce_no_atomico` é falso no
        // Markdown).
        let md = "antes\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: Card\n  column: Backlog\n{{ /kanban }}\n\ndepois\n";
        let volta = escrever_costurando(md, &analisar(md));
        assert_eq!(volta, md);
    }
}
