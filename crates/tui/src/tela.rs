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
use anotadinho_core::unidade::{Arranjo, Caminho, Tipo, Unidade};

/// Um filho de galho em linha, desenhado dentro da linha do galho.
#[derive(Debug, Clone, PartialEq)]
pub struct Segmento {
    /// O endereço dele — é destino de navegação como qualquer outro.
    pub caminho: Caminho,
    /// O que se lê.
    pub texto: String,
    /// O nome da parte, pra quem desenha saber com que cor pintar.
    pub nome: String,
}

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
    /// O nome do embed a que esta linha pertence, se pertence a algum.
    ///
    /// É o gancho de OVERRIDE da borda (ciclo 294): a borda é da unidade
    /// base e some por padrão, e quem quiser desenhar a sua — um embed —
    /// se identifica aqui.
    pub embed_dono: Option<String>,
    /// Os filhos de um galho em LINHA, cada um com o caminho dele
    /// (ciclo 298).
    ///
    /// Vazio em todo o resto. Existe porque a fileira é desenhada numa
    /// linha só e mesmo assim cada filho é um DESTINO: sem guardar o
    /// caminho de cada um, entrar num botão movia o cursor pra um lugar
    /// que não tinha linha na tela — o Enter parecia não fazer nada.
    ///
    /// O NOME da parte vai junto (ciclo 302): sem ele todo segmento era
    /// pintado igual, e a etapa atual de um fluxo saía da mesma cor das
    /// outras cinco.
    pub segmentos: Vec<Segmento>,
    /// O CAMINHO do embed dono (ciclo 298).
    ///
    /// O nome não bastava pra desenhar: dois embeds vizinhos do mesmo
    /// tipo pediam a mesma cor e viravam uma caixa só. O caminho
    /// distingue.
    pub dono_embed: Option<Caminho>,
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
            marca: marca(&u.tipo),
            trechos,
            tipo: u.tipo.clone(),
            embed_dono: None,
            dono_embed: None,
            segmentos: Vec::new(),
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
    // Dado de desenho não conta: a variante de um callout não é um item
    // dele (ciclo 307), e "2 items" num callout de um parágrafo mentiria.
    let esconde = |f: &Unidade| {
        matches!(&f.tipo, Tipo::Parte { nome, .. } if fica_fora_da_tela(nome) || cursor_passa_por_cima(nome))
    };
    if u.filhos.iter().any(esconde) {
        let mut so_visiveis = u.clone();
        so_visiveis.filhos.retain(|f| !esconde(f));
        return contagem(&so_visiveis);
    }
    let n = u.filhos.len();
    if n == 0 {
        return String::new();
    }
    // Cartão com TÍTULO fechado mostra o título (ciclo 302). "3 items"
    // não diz nada sobre uma proposta em revisão; "PROPOSTA: Em Revisão"
    // diz tudo que um cartão fechado precisa dizer.
    if let Some(titulo) = u
        .filhos
        .iter()
        .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "titulo"))
    {
        return titulo.texto.clone();
    }
    // Partes de nomes DIFERENTES são campos, não coleção: o fluxo tem
    // um artefato e uma etapa, e contar "2 items" ali não diz nada. O
    // que serve é o valor deles.
    let folhas_distintas = u.filhos.len() <= 3
        && u.filhos.iter().all(|f| {
            matches!(&f.tipo, Tipo::Parte { arranjo: Arranjo::Folha, .. })
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

/// As partes cujo nome não vai pra tela.
///
/// São as que têm cor própria no tema: a cor já diz o que elas são, e
/// repetir em texto é dizer duas vezes.
const PARTES_SEM_ROTULO: &[&str] = &[
    "titulo",
    "acao",
    "dica",
    "etapa",
    "etapa-atual",
    "transicao",
    "transicao-principal",
    "button",
    "button-primary",
    // O cartão e a miniatura desenham em caixa própria (ciclo 304) —
    // `card cartão` ou `miniatura Legenda` repetiria o nome que a
    // caixa já mostra pela cor e pela forma.
    "card",
    "miniatura",
    // O evento do calendário e a barra do cronograma, mesma razão
    // (ciclo 305).
    "entry",
    "barra",
    // A grade do mês (ciclo 306): o mês, a semana e o dia têm desenho
    // próprio — o nome só apareceria se o desenho caísse no genérico.
    "mes",
    "semana",
    "dia",
    "dia-fora",
    "sem-data",
    "tags",
    "dia-hoje",
    "cabecalho",
    "agenda",
    "nada",
];

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
        // A parte mostra o NOME só quando ele informa (ciclo 302).
        //
        // `column Backlog` precisa do nome: sem ele não dá pra saber que
        // aquilo é uma coluna. Já `titulo PROPOSTA: Em revisão` e
        // `acao Pedir alteração` não — ali o nome repete o que a cor e a
        // posição já dizem, e vira ruído.
        //
        // A lista de quem se cala é a mesma que tem cor própria: quando
        // o desenho sabe pintar diferente, o rótulo textual sobra.
        Tipo::Parte { nome, .. } if PARTES_SEM_ROTULO.contains(&nome.as_str()) => String::new(),
        Tipo::Parte { nome, .. } => nome.clone(),
    }
}

/// Junta um galho em LINHA numa linha só (ciclo 297).
///
/// O arranjo vem do MODELO — `Arranjo::Linha` — e não de um caso
/// especial aqui. Antes eu juntava botões olhando o nome da parte; era
/// adivinhação, e valia só pra botão.
///
/// Cada filho vira um SEGMENTO da linha, e quem desenha os põe lado a
/// lado — como botões numa barra de ações. A navegação dentro dela é
/// `h`/`l`, e quem decide isso também é o arranjo.
fn achatar_fileiras(linhas: Vec<Linha>, raiz: &Unidade) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::with_capacity(linhas.len());
    let mut fileira: Option<Caminho> = None;
    for l in linhas {
        // Filho de uma fileira já aberta: entra na linha dela.
        if let Some(dono) = &fileira {
            if l.caminho.len() == dono.len() + 1 && l.caminho.starts_with(dono) {
                if let Some(atual) = fora.last_mut() {
                    // Guarda caminho e nome de cada um: a linha é uma,
                    // os destinos são vários, e as cores também.
                    atual.segmentos.push(Segmento {
                        caminho: l.caminho.clone(),
                        texto: l.texto.clone(),
                        nome: match &l.tipo {
                            Tipo::Parte { nome, .. } => nome.clone(),
                            _ => String::new(),
                        },
                    });
                }
                continue;
            }
            fileira = None;
        }
        let e_fileira = raiz
            .em(&l.caminho)
            .is_some_and(|u| u.tipo.arranjo() == Arranjo::Linha);
        if e_fileira && !l.enfeite {
            fileira = Some(l.caminho.clone());
            let mut l = l;
            l.marca = String::new();
            l.texto = String::new();
            l.resumo = String::new();
            fora.push(l);
            continue;
        }
        fora.push(l);
    }
    fora
}

/// Nomes de parte que só carregam DADO de desenho, não conteúdo pra
/// navegar ou mostrar.
///
/// A barra do cronograma guarda início e duração como partes-filhas em
/// porcentagem (ciclo 305) — é a mesma aritmética que a janela usa
/// (`bar_span`), calculada uma vez no núcleo em vez de a cada repaint.
/// O dia do calendário guarda as FAIXAS de evento como filhas (ciclo
/// 306), e quem as lê é o desenho da semana, que põe cada uma na sua
/// coluna. Nos dois casos "12" ou "Reunião" soltos numa linha seriam
/// cano, não conteúdo — e nem um lugar onde o `j` devesse parar.
pub fn fica_fora_da_tela(nome: &str) -> bool {
    matches!(
        nome,
        "inicio" | "duracao" | "evento-continua" | "vazio" | "mais" | "variante" | "detalhe" | "data" | "hora" | "pagina" | "indice"
            // O tamanho e as colunas da galeria, o caminho da imagem e a
            // largura do painel (ciclos 325 e 326): dado de desenho.
            | "tamanho" | "colunas" | "caminho" | "largura"
            // A contagem, os campos e os agregados da consulta (ciclo 331).
            | "contagem" | "campo" | "total" | "agregado" | "chave"
            // A janela do cronograma e a cor da barra (ciclo 332).
            | "escala" | "fonte" | "dias" | "hoje" | "cor"
    )
        || nome == "evento"
        || nome.starts_with("evento--")
        // As tags de uma célula multiselect (ciclo 308): quem as desenha
        // é a linha da tabela, uma pílula ao lado da outra.
        || nome.starts_with("tag--")
}

/// Parte sem linha própria que o cursor NÃO visita.
///
/// Quase todo dado de desenho é só isso — dado. O evento do calendário é
/// a exceção (ciclo 309): ele não tem linha (quem o desenha é a semana),
/// mas é CONTEÚDO, e Enter num dia tem que poder chegar nele. Desde o
/// ciclo 310 isso vale também no MEIO de uma barra de vários dias: a
/// continuação é o mesmo evento, visto de outro dia. Só a faixa vazia é
/// pulada.
///
/// E o contrário também existe: o EIXO do cronograma (ciclo 313) tem
/// linha — é desenhado —, mas é régua, não conteúdo, e o cursor passa
/// por cima dele.
pub fn cursor_passa_por_cima(nome: &str) -> bool {
    (fica_fora_da_tela(nome) && !e_evento(nome)) || matches!(nome, "eixo" | "cabecalho" | "nada")
}

/// A barra vizinha NO TEMPO, com o cursor numa barra do cronograma
/// (ciclo 313).
///
/// `j`/`k` andam pela ordem do arquivo, que é a ordem em que as barras
/// estão empilhadas. `h`/`l` andam pela ordem em que elas ACONTECEM:
/// quem começa antes, e, empatado o começo, a mais curta primeiro — a
/// leitura de um cronograma da esquerda pra direita. Sem barra adiante,
/// `None`.
pub fn barra_ao_lado(raiz: &Unidade, cursor: &[usize], adiante: bool) -> Option<Caminho> {
    let (idx, pai_c) = cursor.split_last()?;
    let pai = raiz.em(pai_c)?;
    if !matches!(&pai.tipo, Tipo::Embed(n) if n == "timeline") {
        return None;
    }
    let pct = |u: &Unidade, campo: &str| -> i64 {
        u.filhos
            .iter()
            .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == campo))
            .and_then(|f| f.texto.parse().ok())
            .unwrap_or(0)
    };
    let mut barras: Vec<(i64, i64, usize)> = pai
        .filhos
        .iter()
        .enumerate()
        .filter(|(_, u)| matches!(&u.tipo, Tipo::Parte { nome, .. } if nome == "barra"))
        .map(|(i, u)| (pct(u, "inicio"), pct(u, "duracao"), i))
        .collect();
    barras.sort();
    let pos = barras.iter().position(|b| b.2 == *idx)?;
    let alvo = if adiante { barras.get(pos + 1) } else { pos.checked_sub(1).and_then(|p| barras.get(p)) }?;
    let mut c = pai_c.to_vec();
    c.push(alvo.2);
    Some(c)
}

/// Um evento na grade do mês — o começo dele (`evento`,
/// `evento--info`…) ou a continuação da barra num dia seguinte.
pub fn e_evento(nome: &str) -> bool {
    nome == "evento" || nome == "evento-continua" || nome.starts_with("evento--")
}

/// `(ano, mês)` de um "mes" do calendário, lido do rótulo que o núcleo
/// montou (`Agosto 2026`).
///
/// O rótulo sai de `date_util::month_name`, e é a MESMA função que o lê
/// de volta aqui — as duas pontas não têm como discordar sobre o nome de
/// um mês.
pub fn ano_e_mes(rotulo: &str) -> Option<(i32, u32)> {
    let (nome, ano) = rotulo.rsplit_once(' ')?;
    let mes = (1..=12).find(|m| anotadinho_core::date_util::month_name(*m) == nome)?;
    Some((ano.parse().ok()?, mes))
}

/// Os dias de um "mes" do calendário, na ordem da grade, como
/// `(data, semana, dia, é_do_mês)`.
fn dias_do_mes(mes: &Unidade) -> Vec<(String, usize, usize, bool)> {
    // Cada dia carrega a própria data numa parte "data" (ciclo 316): numa
    // semana solta (a visão Semana), a posição do dia não diz mais a data.
    mes.filhos
        .iter()
        .enumerate()
        .flat_map(|(s, semana)| {
            semana.filhos.iter().enumerate().filter_map(move |(d, dia)| {
                let Tipo::Parte { nome, .. } = &dia.tipo else { return None };
                let data = dia
                    .filhos
                    .iter()
                    .rev()
                    .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "data"))?;
                Some((data.texto.clone(), s, d, nome != "dia-fora"))
            })
        })
        .collect()
}

/// O calendário que contém o cursor (ou é ele), como caminho do embed.
pub fn calendario_do_cursor(raiz: &Unidade, cursor: &[usize]) -> Option<Caminho> {
    (1..=cursor.len())
        .map(|n| &cursor[..n])
        .find(|c| matches!(raiz.em(c).map(|u| &u.tipo), Some(Tipo::Embed(n)) if n == "calendar"))
        .map(|c| c.to_vec())
}

/// A data (`AAAA-MM-DD`) do dia em que o cursor está — no dia ou num
/// evento dele —, dentro de um calendário.
pub fn data_do_cursor(raiz: &Unidade, cursor: &[usize]) -> Option<String> {
    let embed = calendario_do_cursor(raiz, cursor)?;
    let k = embed.len();
    // Na agenda do Dia, a data é a da agenda.
    if let Some(agenda) = cursor.get(k).and_then(|m| raiz.em(&embed)?.filhos.get(*m)) {
        if matches!(&agenda.tipo, Tipo::Parte { nome, .. } if nome == "agenda") {
            return agenda
                .filhos
                .iter()
                .find(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "data"))
                .map(|f| f.texto.clone());
        }
    }
    let (m, s, d) = (*cursor.get(k)?, *cursor.get(k + 1)?, *cursor.get(k + 2)?);
    let mes = raiz.em(&embed)?.filhos.get(m)?;
    dias_do_mes(mes).into_iter().find(|(_, ss, dd, _)| (*ss, *dd) == (s, d)).map(|x| x.0)
}

/// As datas que o calendário mostra agora, em ordem: os dias do mês (ou
/// da semana) da grade, ou o dia da agenda (ciclo 316).
pub fn datas_visiveis(raiz: &Unidade, embed: &[usize]) -> Vec<String> {
    let Some(cal) = raiz.em(embed) else { return Vec::new() };
    let mut datas: Vec<String> = cal
        .filhos
        .iter()
        .flat_map(|parte| match &parte.tipo {
            Tipo::Parte { nome, .. } if nome == "mes" => {
                dias_do_mes(parte).into_iter().filter(|x| x.3).map(|x| x.0).collect()
            }
            Tipo::Parte { nome, .. } if nome == "agenda" => parte
                .filhos
                .iter()
                .filter(|f| matches!(&f.tipo, Tipo::Parte { nome, .. } if nome == "data"))
                .map(|f| f.texto.clone())
                .collect(),
            _ => Vec::new(),
        })
        .collect();
    datas.sort();
    datas
}

/// O caminho do DIA com essa data num calendário — na grade do próprio
/// mês dele, não na célula de fora de outra grade.
pub fn dia_com_data(raiz: &Unidade, embed: &[usize], data: &str) -> Option<Caminho> {
    let cal = raiz.em(embed)?;
    cal.filhos.iter().enumerate().find_map(|(m, mes)| {
        if !matches!(&mes.tipo, Tipo::Parte { nome, .. } if nome == "mes") {
            return None;
        }
        dias_do_mes(mes)
            .into_iter()
            .find(|(dt, _, _, do_mes)| *do_mes && dt == data)
            .map(|(_, s, d, _)| {
                let mut c = embed.to_vec();
                c.extend([m, s, d]);
                c
            })
    })
}

/// O evento do dia vizinho, com o cursor num evento do calendário
/// (ciclos 311 e 312).
///
/// `h`/`l` num evento andam de DIA, não de irmão: os irmãos de um evento
/// são as outras faixas do mesmo dia, empilhadas — `j`/`k` já andam
/// entre elas. Andar de dia mantém o nível: o cursor sai de um evento e
/// pousa num evento.
///
/// No dia ao lado, dentro do mesmo mês, a preferência é a MESMA faixa —
/// numa barra de vários dias é a continuação dela, e `l` percorre a barra
/// dia a dia com o evento selecionado. Se a faixa ali não é evento, vai
/// pro primeiro evento do dia; dia sem evento é pulado.
///
/// A conta é por DATA, e atravessa os meses (ciclo 312): do último
/// evento de agosto, `l` vai pro primeiro dia com evento do próximo mês
/// que o calendário mostra. Só os dias do PRÓPRIO mês de cada grade
/// contam — os de fora (o 1º de setembro no fim da grade de agosto) são
/// os mesmos dias da grade seguinte, e contá-los duas vezes faria o
/// cursor voltar no tempo ao virar o mês. Sem evento adiante, `None`.
pub fn evento_ao_lado(raiz: &Unidade, cursor: &[usize], adiante: bool) -> Option<Caminho> {
    let nome_de = |u: &Unidade| match &u.tipo {
        Tipo::Parte { nome, .. } => Some(nome.clone()),
        _ => None,
    };
    let n = cursor.len();
    if n < 5 {
        return None;
    }
    let atual = raiz.em(cursor)?;
    if !nome_de(atual).is_some_and(|nm| e_evento(&nm)) {
        return None;
    }
    let embed_c = &cursor[..n - 4];
    let (mes_atual, semana, dia, faixa) = (cursor[n - 4], cursor[n - 3], cursor[n - 2], cursor[n - 1]);
    let embed = raiz.em(embed_c)?;
    if nome_de(embed.filhos.get(mes_atual)?).as_deref() != Some("mes") {
        return None;
    }

    // A data de onde se parte.
    let hoje = dias_do_mes(&embed.filhos[mes_atual])
        .into_iter()
        .find(|(_, s, d, _)| (*s, *d) == (semana, dia))?
        .0;

    // Todos os dias de todos os meses, em ordem de data.
    let mut dias: Vec<(String, usize, usize, usize)> = embed
        .filhos
        .iter()
        .enumerate()
        .filter(|(_, u)| nome_de(u).as_deref() == Some("mes"))
        .flat_map(|(m, u)| {
            dias_do_mes(u)
                .into_iter()
                .filter(|(_, _, _, do_mes)| *do_mes)
                .map(move |(data, s, d, _)| (data, m, s, d))
        })
        .filter(|(data, ..)| if adiante { *data > hoje } else { *data < hoje })
        .collect();
    dias.sort_by(|a, b| a.0.cmp(&b.0));
    if !adiante {
        dias.reverse();
    }

    for (_, m, s, d) in dias {
        let o_dia = embed.filhos.get(m)?.filhos.get(s)?.filhos.get(d)?;
        let e_ev = |i: usize| o_dia.filhos.get(i).and_then(nome_de).is_some_and(|nm| e_evento(&nm));
        let alvo = if m == mes_atual && e_ev(faixa) {
            Some(faixa)
        } else {
            (0..o_dia.filhos.len()).find(|i| e_ev(*i))
        };
        if let Some(f) = alvo {
            let mut c = embed_c.to_vec();
            c.extend([m, s, d, f]);
            return Some(c);
        }
    }
    None
}

/// A página inteira em linhas, na ordem em que se lê.
pub fn linhas(raiz: &Unidade) -> Vec<Linha> {
    let mut r = Linhas::default();
    desenhar(raiz, &mut r);
    // Tirar ANTES de achatar as fileiras: a semana é fileira de dias, e
    // uma faixa de evento no meio dela interromperia a fileira — os dias
    // depois dela deixariam de virar segmento.
    let visiveis: Vec<Linha> = r
        .fora
        .into_iter()
        .filter(|l| !matches!(&l.tipo, Tipo::Parte { nome, .. } if fica_fora_da_tela(nome)))
        .collect();
    encaixotar_embeds(achatar_fileiras(visiveis, raiz))
}

/// Marca a que embed cada linha pertence, e injeta a trilha do fluxo.
///
/// A BORDA saiu daqui (ciclo 296). Eu desenhava um `└────` na mão, e a
/// unidade em foco ganhou moldura de verdade no 294 — ficaram dois
/// vocabulários de caixa na mesma tela. Agora o embed usa a MESMA
/// moldura, com cor própria e sempre visível; quem desenha é o painel,
/// por região, e aqui só se diz de quem é cada linha.
fn encaixotar_embeds(linhas: Vec<Linha>) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::with_capacity(linhas.len());
    let mut aberto: Option<(Caminho, String)> = None;

    for mut l in linhas {
        if let Some((dono, nome)) = &aberto {
            let dentro = l.caminho.len() > dono.len() && l.caminho.starts_with(dono);
            if dentro {
                l.embed_dono = Some(nome.clone());
                l.dono_embed = Some(dono.clone());
            } else {
                aberto = None;
            }
        }
        if let Tipo::Embed(nome) = &l.tipo {
            l.embed_dono = Some(nome.clone());
            l.dono_embed = Some(l.caminho.clone());
            aberto = Some((l.caminho.clone(), nome.clone()));
        }

        fora.push(l);
    }
    fora
}


impl Linha {
    /// Esta linha MOSTRA a unidade endereçada?
    ///
    /// Não é só "tem o mesmo caminho": um filho de fileira aparece
    /// DENTRO da linha do galho, como um segmento. Sem isto, o cursor
    /// num botão não acha linha nenhuma — e a rolagem, a moldura e o
    /// realce ficam todos sem alvo (ciclo 298).
    ///
    /// E o que está DENTRO de um segmento também é desta linha (ciclo
    /// 309): o evento de um dia do calendário não tem linha própria — a
    /// semana o desenha na coluna do dia —, e sem isto o cursor num
    /// evento ficaria, de novo, sem alvo.
    pub fn mostra(&self, caminho: &[usize]) -> bool {
        self.caminho == caminho || self.segmentos.iter().any(|s| caminho.starts_with(&s.caminho))
    }
}

/// Quantas linhas de uma transclusão cabem na página (ciclo 415).
///
/// A janela mostra a página transcluída inteira numa caixa rolável;
/// aqui quem rola é a página toda, então uma transclusão de 300 linhas
/// enterraria o resto do documento. Doze é o que deixa ver o começo sem
/// perder o fio — o resto se lê abrindo a página, que é um Enter.
pub const LINHAS_DA_TRANSCLUSAO: usize = 12;

/// Insere, depois de cada `![[Página]]`, o conteúdo dela (ciclo 415).
///
/// As linhas trazidas são ENFEITE: ocupam a tela, carregam o caminho do
/// dono — somem junto quando ele dobra —, e o cursor nunca pousa nelas.
/// Isso é o que mantém a edição honesta: o bloco continua sendo o
/// marcador `![[…]]`, que é o que está no arquivo; ninguém edita o texto
/// de outra página achando que edita esta.
///
/// `resolvidas` vai do alvo cru (`Página#Seção`) pro conteúdo, ou pro
/// motivo de não ter dado. Alvo ausente do mapa é alvo ainda carregando:
/// não desenha nada, pra a página não piscar aviso enquanto lê o disco.
pub fn com_transclusoes(
    linhas: Vec<Linha>,
    raiz: &Unidade,
    resolvidas: &std::collections::BTreeMap<String, Result<String, String>>,
) -> Vec<Linha> {
    let mut fora: Vec<Linha> = Vec::with_capacity(linhas.len());
    for l in linhas {
        // O `texto` da linha é o VISÍVEL — o `[[…]]` já virou o rótulo.
        // O marcador só existe na fonte, então a busca é na unidade.
        let fonte = raiz.em(&l.caminho).map(corpo).unwrap_or_default();
        let alvos = anotadinho_core::links::extract_transclusion_targets(&fonte);
        let mut l = l;
        // Bloco que é SÓ o marcador: mostra a fonte crua. O desenho de
        // wikilink comeria os colchetes (`![Página`), e aí a linha não
        // seria nem o que está no arquivo nem o conteúdo trazido.
        if alvos.len() == 1 && fonte.trim().starts_with("![[") && fonte.trim().ends_with("]]") {
            l.texto = fonte.trim().to_string();
            l.trechos = Vec::new();
        }
        let modelo = l.clone();
        fora.push(l);
        for alvo in alvos {
            let Some(estado) = resolvidas.get(&alvo) else { continue };
            let trazidas: Vec<String> = match estado {
                Ok(conteudo) => {
                    let mut v: Vec<String> = vec![format!("▤ {alvo}")];
                    // Embed transcluído é AVISO, não YAML (ciclo 447): um
                    // kanban trazia quinze linhas de `columns:`/`cards:`
                    // que não são leitura nem são quadro, e ainda comiam
                    // o limite de linhas da caixa.
                    let conteudo = anotadinho_core::transclusao::resumir_embeds(conteudo);
                    let corpo: Vec<&str> = conteudo.lines().collect();
                    v.extend(corpo.iter().take(LINHAS_DA_TRANSCLUSAO).map(|t| format!("│ {t}")));
                    if corpo.len() > LINHAS_DA_TRANSCLUSAO {
                        v.push(format!("│ … mais {} linha(s) — Enter abre", corpo.len() - LINHAS_DA_TRANSCLUSAO));
                    }
                    v
                }
                Err(motivo) => vec![format!("▤ {alvo} — {motivo}")],
            };
            for texto in trazidas {
                fora.push(Linha {
                    caminho: modelo.caminho.clone(),
                    enfeite: true,
                    nivel: modelo.nivel + 1,
                    texto,
                    resumo: String::new(),
                    marca: String::new(),
                    trechos: Vec::new(),
                    tipo: Tipo::Paragrafo,
                    embed_dono: modelo.embed_dono.clone(),
                    dono_embed: modelo.dono_embed.clone(),
                    segmentos: Vec::new(),
                });
            }
        }
    }
    fora
}

/// Em que linha está a unidade endereçada.
pub fn linha_de(linhas: &[Linha], caminho: &Caminho) -> Option<usize> {
    linhas.iter().position(|l| l.mostra(caminho))
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
    let escondida = |c: &Caminho| {
        raiz.em(c)
            .is_some_and(|u| matches!(&u.tipo, Tipo::Parte { nome, .. } if cursor_passa_por_cima(nome)))
    };
    // Entrar numa unidade cujos filhos são TODOS dado de desenho (a barra
    // do cronograma, o dia do calendário) desceria pra um lugar sem linha
    // na tela: parece que o Enter não fez nada, e só o Backspace devolve
    // (ciclo 305). Tratar como se não houvesse filhos é o mesmo efeito
    // que um cartão de kanban já tem.
    if passo == Passo::Entrar {
        if let Some(atual) = raiz.em(cursor) {
            let so_geometria = !atual.filhos.is_empty()
                && atual.filhos.iter().all(
                    |f| matches!(&f.tipo, Tipo::Parte { nome, .. } if cursor_passa_por_cima(nome)),
                );
            if so_geometria {
                return cursor.clone();
            }
        }
    }
    let Some(mut destino) = mover(raiz, &Cursor::em(cursor), passo).map(|c| c.caminho) else {
        return cursor.clone();
    };
    // Quando o dado de desenho é IRMÃO de conteúdo — a variante do
    // callout vem antes do título e do corpo (ciclo 307) —, o cursor
    // passa por cima dela na mesma direção. Entrar pousa no primeiro
    // irmão visível; se não houver nenhum adiante, fica onde estava.
    let adiante = match passo {
        Passo::Anterior => Passo::Anterior,
        Passo::Proximo | Passo::Entrar => Passo::Proximo,
        Passo::Sair => return destino,
    };
    while escondida(&destino) {
        match mover(raiz, &Cursor::em(&destino), adiante) {
            Some(c) => destino = c.caminho,
            None => return cursor.clone(),
        }
    }
    destino
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
        // O traço da parte saiu no ciclo 296: a moldura do embed é a
        // lateral, e o recuo é a hierarquia. Sobra o nome.
        assert!(pares.iter().any(|t| t == "column Backlog"), "{pares:?}");
        // "Card A", sem o prefixo "card" (ciclo 304): o cartão vira
        // retângulo preenchido, e a caixa já diz o que ele é — repetir
        // o nome em texto seria a mesma informação duas vezes, como o
        // ciclo 302 já tinha decidido pra "titulo" e "acao".
        assert!(pares.iter().any(|t| t == "Card A"), "{pares:?}");
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
        // Fechado, o cartão mostra o TÍTULO dele (ciclo 302).
        assert_eq!(ls[0].resumo, "SPEC: Concluída");
        // Aberto, o título é uma linha e a trilha é uma fileira.
        let textos: Vec<&str> = ls.iter().map(|l| l.texto.as_str()).collect();
        assert!(textos.contains(&"SPEC: Concluída"), "{textos:?}");
        assert!(
            ls.iter().any(|l| l.segmentos.iter().any(|s| s.texto == "Concluída")),
            "a trilha não apareceu"
        );
    }

    #[test]
    fn um_item_so_nao_vira_plural() {
        let d = analisar("- só um\n");
        let lista = linhas(&d).into_iter().find(|l| l.marca == "·").unwrap();
        assert_eq!(lista.resumo, "1 item");
    }

    #[test]
    fn as_linhas_de_um_embed_sabem_de_quem_sao() {
        // A borda saiu daqui (ciclo 296): quem desenha é o painel, por
        // região. O que este módulo faz é dizer de quem é cada linha.
        let d = analisar(
            "antes\n\n{{ type: \"callout\" }}\nvariant: info\nbody: |\n  oi\n{{ /callout }}\n\ndepois\n",
        );
        let ls = linhas(&d);
        let donos: Vec<Option<&str>> = ls
            .iter()
            .map(|l| l.embed_dono.as_deref())
            .collect();
        assert_eq!(
            donos,
            [None, Some("callout"), Some("callout"), None],
            "{:?}",
            ls.iter().map(|l| l.texto.clone()).collect::<Vec<_>>()
        );
    }

    #[test]
    fn consulta_sem_from_nao_declara_parte_vazia() {
        // Parte vazia é ruído no modelo: fazia o desenho achar que havia
        // conteúdo onde não há.
        let d = analisar("{{ type: \"query\" }}\nview: list\n{{ /query }}\n");
        assert_eq!(linhas(&d).len(), 1, "a consulta ganhou parte que não tem");
    }

    #[test]
    fn o_fluxo_desenha_a_trilha_como_fileira() {
        // A trilha era feita à mão aqui (ciclo 293) e virou fileira
        // DECLARADA no 302 — o modelo diz as etapas e qual é a atual, e
        // o desenho só as põe lado a lado.
        let d = analisar(
            "{{ type: \"fluxo\" }}\nartefato: proposta\netapa: aprovada\n{{ /fluxo }}\n",
        );
        let ls = linhas(&d);
        let trilha = ls
            .iter()
            .find(|l| l.segmentos.iter().any(|s| s.texto == "Rascunho"))
            .expect("faltou a trilha");
        let etapas: Vec<&str> = trilha.segmentos.iter().map(|s| s.texto.as_str()).collect();
        assert_eq!(
            etapas,
            ["Rascunho", "Em revisão", "Aprovada", "Em execução", "Concluída", "Bloqueada"]
        );

        // E as transições, que vêm do domínio.
        let acoes = ls
            .iter()
            .find(|l| l.segmentos.iter().any(|s| s.texto == "Em execução")
                && l.segmentos.len() < etapas.len())
            .expect("faltaram as transições");
        let rotulos: Vec<&str> = acoes.segmentos.iter().map(|s| s.texto.as_str()).collect();
        assert_eq!(rotulos, ["Em execução", "Em revisão"]);
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
    fn os_botoes_de_um_embed_viram_uma_fileira() {
        // Clicável em fileira lê como barra de ações, que é como a
        // janela desenha. Um por linha lê como lista, que é outra coisa.
        let d = analisar(
            "{{ type: \"actions\" }}\nbuttons:\n- label: Abrir\n  action: open-page\n- label: Buscar\n  action: run-search\n{{ /actions }}\n",
        );
        let ls = linhas(&d);
        // Os filhos ficam em SEGMENTOS, cada um com o caminho dele: a
        // linha é uma, os destinos são vários (ciclo 298).
        let fileira = ls
            .iter()
            .find(|l| !l.segmentos.is_empty())
            .expect("faltou a fileira");
        let textos: Vec<&str> = fileira.segmentos.iter().map(|s| s.texto.as_str()).collect();
        assert_eq!(textos, ["Abrir", "Buscar"]);
        // E os caminhos são os de verdade, pra o cursor achar.
        assert_eq!(fileira.segmentos[0].caminho, vec![0, 0, 0]);
        assert_eq!(fileira.segmentos[1].caminho, vec![0, 0, 1]);
        // E não sobrou uma linha por botão.
        assert_eq!(ls.len(), 2, "{:?}", ls.iter().map(|l| &l.texto).collect::<Vec<_>>());
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

    // --- Ciclo 415: transclusão desenhada --------------------------------------------

    #[test]
    fn a_transclusao_traz_o_conteudo_como_enfeite() {
        let d = analisar("antes\n\n![[Padrões#Regras]]\n\ndepois\n");
        let mut mapa = std::collections::BTreeMap::new();
        mapa.insert("Padrões#Regras".to_string(), Ok("## Regras\n\nsempre minúsculas".to_string()));
        let base = linhas(&d);
        let com = com_transclusoes(base.clone(), &d, &mapa);
        assert!(com.len() > base.len(), "o conteúdo entrou");
        let texto: Vec<&str> = com.iter().map(|l| l.texto.as_str()).collect();
        assert!(texto.iter().any(|t| t.contains("▤ Padrões")), "{texto:?}");
        assert!(texto.iter().any(|t| t.contains("sempre minúsculas")), "{texto:?}");
        // A linha do marcador continua lá — é ela que se edita e se
        // apaga; o conteúdo trazido vem DEPOIS dela.
        let marcador = texto.iter().position(|t| t.starts_with("![")).expect("marcador na tela");
        let cabecalho = texto.iter().position(|t| t.contains("▤ Padrões")).expect("cabeçalho na tela");
        assert!(marcador < cabecalho, "{texto:?}");
        // Tudo que veio é enfeite, com o caminho do dono — some junto
        // quando ele dobra, e o cursor nunca pousa ali.
        let dono = com[marcador].caminho.clone();
        for l in com.iter().filter(|l| l.texto.contains("minúsculas") || l.texto.contains("▤")) {
            assert!(l.enfeite, "{:?}", l.texto);
            assert_eq!(l.caminho, dono);
        }
    }

    #[test]
    fn transcluir_pagina_de_kanban_mostra_aviso_e_nao_o_yaml() {
        // Relatado como "ficou quebrada" (ciclo 447): a caixa vinha com
        // `columns:`/`cards:` crus e o texto em volta era empurrado pra
        // fora do teto de linhas.
        let d = analisar("antes\n\n![[Quadro]]\n\ndepois\n");
        let corpo = "# Quadro\n\nAntes do quadro.\n\n{{ type: \"kanban\" }}\ncolumns:\n- id: todo\n  title: A fazer\n  cards:\n  - title: Tarefa 1\n    column: todo\n{{ /kanban }}\n\nDepois do quadro.\n";
        let mut mapa = std::collections::BTreeMap::new();
        mapa.insert("Quadro".to_string(), Ok(corpo.to_string()));
        let texto: Vec<String> =
            com_transclusoes(linhas(&d), &d, &mapa).into_iter().map(|l| l.texto).collect();
        let tela = texto.join("\n");
        assert!(!tela.contains("columns:"), "o YAML do quadro vazou:\n{tela}");
        assert!(!tela.contains("{{ type:"), "a cerca do embed vazou:\n{tela}");
        assert!(tela.contains("Bloco kanban — abra a página pra usar."), "{tela}");
        // E o que interessa ler continua na caixa, dos dois lados do quadro.
        assert!(tela.contains("Antes do quadro.") && tela.contains("Depois do quadro."), "{tela}");
        // Sem estourar o teto: o que sobrou cabe sem recado de corte.
        assert!(!tela.contains("mais "), "não era pra cortar nada:\n{tela}");
    }

    #[test]
    fn transclusao_grande_e_cortada_com_recado() {
        let d = analisar("![[Longa]]\n");
        let corpo: String = (1..=40).map(|i| format!("linha {i}\n")).collect();
        let mut mapa = std::collections::BTreeMap::new();
        mapa.insert("Longa".to_string(), Ok(corpo));
        let texto: Vec<String> = com_transclusoes(linhas(&d), &d, &mapa).into_iter().map(|l| l.texto).collect();
        assert!(texto.iter().any(|t| t.contains("linha 12")), "{texto:?}");
        assert!(!texto.iter().any(|t| t.contains("linha 13")), "cortou no teto:\n{texto:?}");
        assert!(texto.iter().any(|t| t.contains("mais 28 linha(s)")), "{texto:?}");
    }

    #[test]
    fn o_que_nao_resolveu_aparece_e_o_que_nao_chegou_nao_pisca() {
        let d = analisar("![[Sumida]]\n");
        let mut mapa = std::collections::BTreeMap::new();
        // Ainda carregando: nada além do marcador.
        assert_eq!(com_transclusoes(linhas(&d), &d, &mapa).len(), linhas(&d).len());
        mapa.insert("Sumida".to_string(), Err("a página Sumida não existe".to_string()));
        let texto: Vec<String> = com_transclusoes(linhas(&d), &d, &mapa).into_iter().map(|l| l.texto).collect();
        assert!(texto.iter().any(|t| t.contains("Sumida — a página Sumida não existe")), "{texto:?}");
    }
}
