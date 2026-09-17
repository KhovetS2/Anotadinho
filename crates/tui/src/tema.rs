//! A paleta do Anotadinho, lida do CSS da janela (ciclo 288).
//!
//! O terminal não renderiza CSS — grade de células não tem box model,
//! `padding: 12px` não significa nada ali. Mas metade do CSS não é
//! geometria: são 40 variáveis de design e quatro temas, e ISSO é o que
//! faz alguém reconhecer o app.
//!
//! Então a paleta não é copiada à mão: ela é LIDA do `main.css`, em
//! tempo de compilação, pelo `include_str!` logo abaixo. Trocar
//! `--accent-blue` na janela muda a TUI junto, e mover o arquivo quebra
//! a compilação — que é o jeito certo de descobrir.
//!
//! ## Realce é nome, não cor
//!
//! Como highlight group do Neovim: o desenho pede `Realce::Titulo(1)` e
//! o tema resolve. Sem isso, personalizar tema significaria caçar
//! `Color::` espalhado pelo desenho — que era exatamente como estava
//! antes deste ciclo.

use std::collections::HashMap;

use ratatui::style::{Color, Modifier, Style};

/// O CSS da janela, embutido na compilação.
const CSS: &str = include_str!("../../../ui/src/styles/main.css");

/// Os temas que o app oferece.
pub const TEMAS: [&str; 4] = ["escuro", "papel", "contraste", "claro"];

/// O papel de um pedaço da tela — o nome que o tema resolve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Realce {
    /// Título, pelo nível.
    Titulo(u8),
    /// Texto comum.
    Texto,
    /// Citação.
    Citacao,
    /// Código, em bloco ou inline.
    Codigo,
    /// O rótulo de um embed.
    Embed,
    /// Uma parte de dentro de um embed.
    Parte,
    /// O título de um cartão (ciclo 302).
    TituloCartao,
    /// Uma etapa já percorrida ou futura, na trilha do fluxo.
    Etapa,
    /// A etapa ATUAL — a que diz onde o artefato está.
    EtapaAtual,
    /// A ação principal daquele estado.
    Acao,
    /// A explicação da ação principal.
    Dica,
    /// Uma transição possível.
    Transicao,
    /// A transição de avanço natural — o botão que se aperta sem
    /// pensar.
    TransicaoPrincipal,
    /// O fundo da tela.
    Fundo,
    /// O marcador que abre a linha (`##`, `-`) — estrutura, não texto.
    Marca,
    /// `[[wikilink]]`.
    Wikilink,
    /// `[texto](destino)`.
    Link,
    /// Onde o cursor está.
    Cursor,
    /// Cursor no painel sem foco.
    CursorApagado,
    /// A borda de um painel.
    Borda,
    /// A borda do painel com foco.
    BordaFoco,
    /// A moldura da unidade em foco (ciclo 294).
    ///
    /// A unidade sob o cursor é desenhada como CARTÃO: caixa em volta
    /// dela e dos filhos. Numa árvore a pergunta não é só "que linha" —
    /// é "até onde vai o que estou olhando", e um `##` com nove itens
    /// embaixo é uma coisa só.
    BordaUnidade,
    /// A cor que IDENTIFICA um TIPO de embed — a moldura aberta e o
    /// rótulo fechado (`[kanban]`) usam a mesma, pra abrir e fechar não
    /// trocar de cor (ciclo 304).
    ///
    /// Nove variantes, não uma genérica parametrizada por nome: o
    /// conjunto de embeds é FECHADO (`EmbedKind::all()`), e cada cor é
    /// uma decisão de design, não um cálculo — a mesma razão que fez o
    /// fluxo ganhar `TituloCartao`/`Etapa`/`Acao` em vez de um "papel
    /// genérico" no ciclo 302. O fluxo fica de fora: já tinha a dele
    /// (`Embed`, roxo) desde o ciclo 293.
    EmbedKanban,
    EmbedCalendar,
    EmbedTable,
    EmbedCallout,
    EmbedColumns,
    EmbedGallery,
    EmbedQuery,
    EmbedTimeline,
    EmbedActions,
    /// O cartão de um kanban: um retângulo preenchido por linha,
    /// empilhado dentro da coluna — não fileira, cada cartão é dono da
    /// própria linha (ciclo 304).
    Cartao,
    /// A miniatura de uma galeria: mesma ideia do cartão, um selo no
    /// lugar da imagem que o terminal não desenha.
    Miniatura,
    /// Botão de ação com `variant: primary` — o preenchido da janela
    /// (`.actions-embed__btn--primary`), cor de destaque. O botão sem
    /// variante fica no `Parte` genérico de sempre — só o primário
    /// precisava se destacar dos outros.
    BotaoPrimario,
    /// Um evento do calendário: mesma ideia do cartão do kanban, ecoa a
    /// cor da moldura (`EmbedCalendar`) — ciclo 305.
    Entrada,
    /// A barra do cronograma: retângulo PROPORCIONAL à duração, ecoa a
    /// cor da moldura (`EmbedTimeline`) — ciclo 305.
    Barra,
    /// Célula de tabela sem tipo especial: texto comum, não a cor
    /// âmbar de parte genérica — é dado, não decoração (ciclo 305).
    Celula,
    /// O cabeçalho de uma coluna de tabela.
    CabecalhoDeTabela,
    /// Badge de célula `select`/`multiselect` — as mesmas quatro cores
    /// da janela (`.badge--info/success/warning/error`, ciclo 305).
    BadgeInfo,
    BadgeSucesso,
    BadgeAtencao,
    BadgeErro,
    /// Um evento sem tag na grade do mês — a barra neutra da janela
    /// (`.calendar-grid__bar`: fundo `--bg-elevated`, texto comum),
    /// ciclo 306.
    Evento,
    /// A caixa de um callout, pela VARIANTE (ciclo 307): a mesma
    /// `--callout-accent` da janela — `info` no destaque, `success`,
    /// `warning`, `error`, e `tip` no roxo.
    CalloutInfo,
    CalloutSucesso,
    CalloutAtencao,
    CalloutErro,
    CalloutDica,
    /// As linhas de GRADE dentro de um embed: o `│` entre colunas da
    /// tabela, as bordas dos dias do calendário (ciclo 308).
    ///
    /// Não é a `Borda` dos painéis: com o embed preenchido, `--border`
    /// fica quase da cor do fundo e a grade some. É o texto apagado
    /// misturado à superfície — traço visível, sem competir com o dado.
    Grade,
}

/// Uma paleta resolvida.
pub struct Tema {
    cores: HashMap<String, Color>,
}

impl Tema {
    /// A paleta de um tema pelo nome. Nome desconhecido cai no escuro,
    /// que é o padrão do app.
    pub fn novo(nome: &str) -> Self {
        let limpo = sem_comentarios(CSS);
        let mut cores = HashMap::new();
        // O bloco base primeiro, e o do tema por cima: é a cascata do
        // CSS, que só define o que muda.
        for (tema, corpo) in blocos(&limpo) {
            if tema.is_none() || tema.as_deref() == Some(nome) {
                for (chave, valor) in variaveis(&corpo) {
                    cores.insert(chave, valor);
                }
            }
        }
        Self { cores }
    }

    /// A cor de uma variável, com um padrão de socorro.
    fn cor(&self, nome: &str, socorro: Color) -> Color {
        self.cores.get(nome).copied().unwrap_or(socorro)
    }

    /// O estilo de um papel.
    pub fn estilo(&self, r: Realce) -> Style {
        let texto = self.cor("text-primary", Color::Gray);
        let apagado = self.cor("text-muted", Color::DarkGray);
        let destaque = self.cor("accent-blue", Color::Cyan);
        match r {
            // O h1 é o nome da página: leva o destaque. Os outros
            // descem pro roxo, que é o segundo acento do app.
            // O h1 é o nome da página: ganha FAIXA, com fundo de
            // superfície, que é o mais perto de "letra grande" que um
            // terminal tem — tamanho de fonte é do emulador, não do
            // programa (ciclo 293).
            Realce::Titulo(1) => Style::default()
                .fg(destaque)
                .bg(self.cor("bg-elevated", Color::DarkGray))
                .add_modifier(Modifier::BOLD),
            // O h2 ganha régua, por sublinhado. Uma linha de `─` de
            // verdade custaria uma LINHA, e a conta entre cursor e
            // rolagem é feita em linhas visíveis: decoração que ocupa
            // linha desalinharia as duas.
            Realce::Titulo(2) => Style::default()
                .fg(self.cor("accent-purple", Color::Magenta))
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::UNDERLINED),
            Realce::Titulo(_) => Style::default()
                .fg(self.cor("accent-purple", Color::Magenta))
                .add_modifier(Modifier::BOLD),
            Realce::Texto => Style::default().fg(texto),
            Realce::Citacao => Style::default().fg(apagado).add_modifier(Modifier::ITALIC),
            Realce::Codigo => Style::default().fg(self.cor("cor-verde", Color::Green)),
            Realce::Embed => Style::default()
                .fg(self.cor("accent-purple", Color::Magenta))
                .add_modifier(Modifier::BOLD),
            Realce::Parte => Style::default().fg(self.cor("cor-ambar", Color::Yellow)),
            // Cores POR PAPEL (ciclo 302): tudo roxo dizia só "isto é
            // embed", e um cartão de fluxo tem coisas de naturezas
            // diferentes — o que ele é, onde está, o que dá pra fazer.
            Realce::TituloCartao => Style::default()
                .fg(self.cor("text-primary", Color::White))
                .add_modifier(Modifier::BOLD),
            // Etapa não-atual é contexto: apagada de propósito, senão
            // compete com a que importa.
            Realce::Etapa => Style::default().fg(apagado),
            Realce::EtapaAtual => Style::default()
                .fg(self.cor("cor-verde", Color::Green))
                .add_modifier(Modifier::BOLD),
            Realce::Acao => Style::default()
                .fg(destaque)
                .add_modifier(Modifier::BOLD),
            Realce::Dica => Style::default().fg(apagado).add_modifier(Modifier::ITALIC),
            Realce::Transicao => Style::default().fg(self.cor("cor-azul", Color::Blue)),
            Realce::TransicaoPrincipal => Style::default()
                .fg(self.cor("accent-purple", Color::Magenta))
                .add_modifier(Modifier::BOLD),
            // O fundo da tela inteira, da mesma cor da janela.
            Realce::Fundo => Style::default().bg(self.cor("bg-base", Color::Black)),
            Realce::Marca => Style::default().fg(apagado),
            Realce::Wikilink => Style::default().fg(destaque).add_modifier(Modifier::UNDERLINED),
            Realce::Link => Style::default()
                .fg(self.cor("cor-azul", Color::Blue))
                .add_modifier(Modifier::UNDERLINED),
            // O cursor inverte: fundo de destaque, texto da cor do papel.
            Realce::Cursor => Style::default()
                .fg(self.cor("bg-base", Color::Black))
                .bg(destaque),
            Realce::CursorApagado => Style::default().add_modifier(Modifier::DIM),
            Realce::Borda => Style::default().fg(self.cor("border", Color::DarkGray)),
            Realce::BordaFoco => Style::default().fg(destaque),
            Realce::BordaUnidade => Style::default().fg(destaque),
            // Cada embed puxa um token que já existe no CSS — nenhuma
            // cor nova, só um papel novo pra uma que já era do tema. É
            // o que deixa "atualizável pelo usuário" de graça: trocar
            // `--cor-azul` na tela de aparência muda o kanban junto.
            Realce::EmbedKanban => Style::default()
                .fg(self.cor("cor-azul", Color::Blue))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedCalendar => Style::default()
                .fg(self.cor("cor-verde", Color::Green))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedTable => Style::default()
                .fg(self.cor("cor-ambar", Color::Yellow))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedCallout => Style::default()
                .fg(self.cor("warning", Color::Yellow))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedColumns => Style::default()
                .fg(self.cor("cor-roxo", Color::Magenta))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedGallery => Style::default()
                .fg(self.cor("cor-vermelho", Color::Red))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedQuery => Style::default()
                .fg(self.cor("error", Color::Red))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedTimeline => Style::default()
                .fg(self.cor("success", Color::Green))
                .add_modifier(Modifier::BOLD),
            Realce::EmbedActions => Style::default().fg(destaque).add_modifier(Modifier::BOLD),
            // A mesma cor do embed dono: o cartão do kanban ecoa o azul
            // da moldura, a miniatura da galeria ecoa o vermelho dela —
            // é o que faz o conteúdo parecer do mesmo cartão, não de
            // dois vocabulários de cor.
            Realce::Cartao => Style::default().fg(self.cor("cor-azul", Color::Blue)),
            Realce::Miniatura => Style::default().fg(self.cor("cor-vermelho", Color::Red)),
            Realce::BotaoPrimario => Style::default().fg(destaque).add_modifier(Modifier::BOLD),
            Realce::Entrada => Style::default().fg(self.cor("cor-verde", Color::Green)),
            Realce::Barra => Style::default().fg(self.cor("success", Color::Green)),
            Realce::Celula => Style::default().fg(texto),
            Realce::CabecalhoDeTabela => Style::default()
                .fg(apagado)
                .add_modifier(Modifier::BOLD),
            // Mesmos quatro tokens do `badge--*` da janela.
            Realce::BadgeInfo => Style::default().fg(destaque),
            Realce::BadgeSucesso => Style::default().fg(self.cor("success", Color::Green)),
            Realce::BadgeAtencao => Style::default().fg(self.cor("warning", Color::Yellow)),
            Realce::BadgeErro => Style::default().fg(self.cor("error", Color::Red)),
            Realce::CalloutInfo => Style::default().fg(destaque),
            Realce::CalloutSucesso => Style::default().fg(self.cor("success", Color::Green)),
            Realce::CalloutAtencao => Style::default().fg(self.cor("warning", Color::Yellow)),
            Realce::CalloutErro => Style::default().fg(self.cor("error", Color::Red)),
            Realce::CalloutDica => Style::default().fg(self.cor("accent-purple", Color::Magenta)),
            Realce::Grade => Style::default().fg(misturar(
                apagado,
                self.cor("bg-surface", Color::DarkGray),
                0.45,
            )),
            Realce::Evento => Style::default()
                .fg(texto)
                .bg(self.cor("bg-elevated", Color::DarkGray)),
        }
    }

    /// A cor que IDENTIFICA um papel de botão.
    ///
    /// Regra: se o papel já declara um fundo, o fundo é a cor dele —
    /// é o caso do cursor, que é fundo de destaque com texto escuro.
    /// Os outros papéis são cor de texto, e aí a cor é o `fg`.
    fn cor_do_botao(&self, r: Realce) -> Color {
        let e = self.estilo(r);
        e.bg.or(e.fg).unwrap_or(Color::Gray)
    }

    /// O CONTORNO de um botão daquele papel (ciclo 302).
    ///
    /// Fundo nenhum: quem carrega a cor é o GLIFO. O contorno é
    /// desenhado com meio-bloco (`▄`, `▐`, `▗`…), e meio-bloco pinta
    /// com o `fg` a metade da célula que olha PRA DENTRO do botão — a
    /// outra metade fica com o fundo da tela.
    ///
    /// Foi assim que o vão sumiu sem o botão inchar. Duas voltas antes
    /// eu tratava célula como pixel: ou a cor tomava a célula inteira
    /// do traço (e o botão ficava gordo, com o contorno engolido), ou
    /// não tomava nada (e sobrava faixa escura entre o preenchimento e
    /// o traço). Meio-bloco é a terceira opção que eu tinha decidido
    /// que não existia — e é meia célula, então o botão encolhe.
    pub fn contorno_do_botao(&self, r: Realce) -> Style {
        Style::default().fg(self.cor_do_botao(r))
    }

    /// O MIOLO de um botão: onde o rótulo fica.
    ///
    /// Fundo na cor do papel e texto na cor do fundo da tela — é como
    /// `.btn--primary` se lê na janela: fundo de acento, texto que
    /// contrasta. O contorno encosta neste preenchimento sem emenda,
    /// porque a metade de dentro da célula dele é da mesma cor.
    pub fn miolo_do_botao(&self, r: Realce) -> Style {
        Style::default()
            .fg(self.cor("bg-base", Color::Black))
            .bg(self.cor_do_botao(r))
            .add_modifier(Modifier::BOLD)
    }

    /// Uma PÍLULA: texto na cor do papel sobre um fundo tingido dela —
    /// é como `.badge--info` se lê na janela, `color-mix(in srgb,
    /// var(--accent-blue) 15%, transparent)` com o texto no acento (ciclo
    /// 306).
    ///
    /// O terminal não tem transparência, então a mistura é feita aqui,
    /// contra o fundo da tela: 15% da cor, 85% do `--bg-base`. Papel que
    /// já declara fundo próprio (o evento neutro, o cursor) sai como está.
    pub fn pilula(&self, r: Realce) -> Style {
        let e = self.estilo(r);
        if e.bg.is_some() {
            return e;
        }
        let cor = e.fg.unwrap_or(Color::Gray);
        e.bg(misturar(cor, self.cor("bg-base", Color::Black), 0.15))
    }
}

impl Tema {
    /// O FUNDO de uma caixa preenchida, quando a região pede um.
    ///
    /// Começou no callout (ciclo 307): é a caixa da janela com
    /// `background: color-mix(in srgb, var(--callout-accent) 8%,
    /// var(--bg-surface))` — o tom da variante bem de leve sobre a
    /// superfície. No ciclo 308 todo EMBED ganhou a mesma caixa, tingida
    /// pela cor do tipo (a do ciclo 304): um kanban é um bloco azul-claro
    /// na página, não só um contorno azul.
    ///
    /// A moldura do FOCO (a unidade sob o cursor fora de embed) continua
    /// só borda: ela diz "até onde vai o que você olha", não "isto é um
    /// objeto", e pintar o fundo de um parágrafo seria mentir.
    pub fn fundo_da_regiao(&self, r: Realce) -> Option<Color> {
        match r {
            Realce::CalloutInfo
            | Realce::CalloutSucesso
            | Realce::CalloutAtencao
            | Realce::CalloutErro
            | Realce::CalloutDica
            | Realce::Embed
            | Realce::EmbedKanban
            | Realce::EmbedCalendar
            | Realce::EmbedTable
            | Realce::EmbedCallout
            | Realce::EmbedColumns
            | Realce::EmbedGallery
            | Realce::EmbedQuery
            | Realce::EmbedTimeline
            | Realce::EmbedActions => {
                let acento = self.estilo(r).fg.unwrap_or(Color::Gray);
                Some(misturar(acento, self.cor("bg-surface", Color::DarkGray), 0.08))
            }
            _ => None,
        }
    }
}

/// `pct` de `a` sobre `b`, canal a canal. Cor sem RGB (a de socorro)
/// não mistura: volta `a`.
fn misturar(a: Color, b: Color, pct: f64) -> Color {
    match (a, b) {
        (Color::Rgb(ar, ag, ab), Color::Rgb(br, bg, bb)) => {
            let canal = |x: u8, y: u8| (x as f64 * pct + y as f64 * (1.0 - pct)).round() as u8;
            Color::Rgb(canal(ar, br), canal(ag, bg), canal(ab, bb))
        }
        _ => a,
    }
}

/// Tira os comentários do CSS.
///
/// Não é frescura: há comentário no `main.css` com `#ffee00` dentro,
/// explicando por que NÃO se grava hex solto. Sem tirar, o leitor
/// engoliria o exemplo como se fosse token.
fn sem_comentarios(css: &str) -> String {
    let mut fora = String::with_capacity(css.len());
    let mut resto = css;
    while let Some(i) = resto.find("/*") {
        fora.push_str(&resto[..i]);
        match resto[i..].find("*/") {
            Some(j) => resto = &resto[i + j + 2..],
            None => return fora,
        }
    }
    fora.push_str(resto);
    fora
}

/// Os blocos `:root` do CSS, como (tema, corpo).
///
/// `None` no tema é o bloco base — o `:root` sem seletor de tema, que
/// vale pra todos.
fn blocos(css: &str) -> Vec<(Option<String>, String)> {
    let mut fora = Vec::new();
    let mut cursor = 0usize;
    while let Some(i) = css[cursor..].find(":root") {
        let i = cursor + i;
        let Some(abre) = css[i..].find('{').map(|a| i + a) else { break };
        let Some(fecha) = css[abre..].find('}').map(|f| abre + f) else { break };
        let seletor = &css[i..abre];
        let tema = seletor
            .find("data-theme=\"")
            .map(|p| &seletor[p + 12..])
            .and_then(|s| s.find('"').map(|f| s[..f].to_string()));
        fora.push((tema, css[abre + 1..fecha].to_string()));
        cursor = fecha + 1;
    }
    fora
}

/// As variáveis de COR de um corpo de bloco.
///
/// Só cor: `--sp-4: 1rem` e `--font-sans: 'Inter'...` não têm o que
/// fazer num terminal, e engolir tudo faria a paleta carregar lixo.
fn variaveis(corpo: &str) -> Vec<(String, Color)> {
    corpo
        .split(';')
        .filter_map(|linha| {
            let linha = linha.trim();
            let sem_traco = linha.strip_prefix("--")?;
            let (chave, valor) = sem_traco.split_once(':')?;
            hex(valor.trim()).map(|c| (chave.trim().to_string(), c))
        })
        .collect()
}

/// Lê `#RRGGBB`, inclusive quando vem dentro de `var(--x, #RRGGBB)`.
///
/// O fallback do `var()` é a cor de verdade do tema: `--accent-blue` é
/// `var(--destaque, #00B5FF)`, e `--destaque` só existe quando a pessoa
/// escolheu um destaque na tela de aparência.
fn hex(valor: &str) -> Option<Color> {
    let bruto = valor.rfind('#').map(|p| &valor[p..])?;
    let digitos: String = bruto[1..].chars().take(6).collect();
    if digitos.len() != 6 || !digitos.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let n = u32::from_str_radix(&digitos, 16).ok()?;
    Some(Color::Rgb(
        ((n >> 16) & 0xFF) as u8,
        ((n >> 8) & 0xFF) as u8,
        (n & 0xFF) as u8,
    ))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn le_a_cor_de_destaque_do_css_da_janela() {
        // O valor vem do `main.css`, não de uma cópia aqui: se alguém
        // trocar `--accent-blue` lá, este teste avisa. É o ponto do
        // ciclo — uma fonte só pra paleta.
        let t = Tema::novo("escuro");
        assert_eq!(t.cor("accent-blue", Color::Reset), Color::Rgb(0x00, 0xB5, 0xFF));
        assert_eq!(t.cor("bg-base", Color::Reset), Color::Rgb(0x27, 0x29, 0x30));
    }

    #[test]
    fn o_var_com_fallback_e_lido_pelo_fallback() {
        // `--accent-blue: var(--destaque, #00B5FF)` — o `--destaque` só
        // existe quando a pessoa escolheu um na tela de aparência, então
        // a cor do tema é o fallback.
        assert_eq!(hex("var(--destaque, #00B5FF)"), Some(Color::Rgb(0, 0xB5, 0xFF)));
        assert_eq!(hex("#3D404F"), Some(Color::Rgb(0x3D, 0x40, 0x4F)));
    }

    #[test]
    fn o_que_nao_e_cor_fica_de_fora() {
        // Espaçamento e fonte não têm o que fazer num terminal, e
        // engolir tudo faria a paleta carregar lixo.
        assert_eq!(hex("1rem"), None);
        assert_eq!(hex("'Inter', system-ui, sans-serif"), None);
        assert_eq!(hex("#ZZZ"), None);
    }

    #[test]
    fn comentario_com_hex_dentro_nao_vira_token() {
        // O `main.css` tem um comentário explicando por que NÃO se grava
        // hex solto, e o exemplo dele é um hex. Sem tirar comentário, o
        // leitor engoliria o exemplo.
        let css = "/* nunca grave #ffee00 assim */\n:root { --cor-azul: #112233; }";
        let vars = variaveis(&blocos(&sem_comentarios(css))[0].1);
        assert_eq!(vars, [("cor-azul".to_string(), Color::Rgb(0x11, 0x22, 0x33))]);
    }

    #[test]
    fn os_quatro_temas_existem_e_sao_diferentes() {
        // Se um deles parasse de ser lido, ele viraria o escuro em
        // silêncio — que é o defeito mais fácil de não perceber aqui.
        let fundos: Vec<Color> = TEMAS
            .iter()
            .map(|t| Tema::novo(t).cor("bg-base", Color::Reset))
            .collect();
        for (i, a) in fundos.iter().enumerate() {
            for b in fundos.iter().skip(i + 1) {
                assert_ne!(a, b, "dois temas com o mesmo fundo: {fundos:?}");
            }
        }
        // O papel é claro de verdade.
        assert_eq!(
            Tema::novo("papel").cor("bg-base", Color::Reset),
            Color::Rgb(0xF6, 0xF1, 0xE7)
        );
        // E o contraste é preto puro.
        assert_eq!(
            Tema::novo("contraste").cor("bg-base", Color::Reset),
            Color::Rgb(0, 0, 0)
        );
    }

    #[test]
    fn o_tema_cai_no_escuro_quando_o_nome_nao_existe() {
        assert_eq!(
            Tema::novo("inexistente").cor("bg-base", Color::Reset),
            Tema::novo("escuro").cor("bg-base", Color::Reset)
        );
    }

    #[test]
    fn cada_papel_resolve_pra_uma_cor_do_tema() {
        // Nenhum realce pode cair no socorro: se cair, é porque o token
        // sumiu do CSS e a TUI está desenhando com cor de emergência.
        let t = Tema::novo("escuro");
        for r in [
            Realce::Titulo(1),
            Realce::Titulo(2),
            Realce::Texto,
            Realce::Citacao,
            Realce::Codigo,
            Realce::Embed,
            Realce::Parte,
            Realce::Marca,
            Realce::Wikilink,
            Realce::Link,
            Realce::Cursor,
            Realce::Borda,
            Realce::BordaFoco,
            Realce::EmbedKanban,
            Realce::EmbedCalendar,
            Realce::EmbedTable,
            Realce::EmbedCallout,
            Realce::EmbedColumns,
            Realce::EmbedGallery,
            Realce::EmbedQuery,
            Realce::EmbedTimeline,
            Realce::EmbedActions,
            Realce::Cartao,
            Realce::Miniatura,
            Realce::BotaoPrimario,
            Realce::Entrada,
            Realce::Barra,
            Realce::Celula,
            Realce::CabecalhoDeTabela,
            Realce::BadgeInfo,
            Realce::BadgeSucesso,
            Realce::BadgeAtencao,
            Realce::BadgeErro,
            Realce::Evento,
            Realce::CalloutInfo,
            Realce::CalloutSucesso,
            Realce::CalloutAtencao,
            Realce::CalloutErro,
            Realce::CalloutDica,
            Realce::Grade,
        ] {
            let e = t.estilo(r);
            let cor = e.fg.or(e.bg).unwrap_or(Color::Reset);
            assert!(
                matches!(cor, Color::Rgb(..)),
                "{r:?} não veio do CSS: {cor:?}"
            );
        }
    }

    #[test]
    fn cada_tipo_de_embed_tem_sua_propria_cor() {
        // Nove tipos, todos diferentes entre si — senão dois embeds
        // vizinhos voltam a se fundir numa caixa só, o defeito que o
        // ciclo 298 corrigiu pra unidade, agora checado pra cor.
        let t = Tema::novo("escuro");
        let papeis = [
            Realce::EmbedKanban,
            Realce::EmbedCalendar,
            Realce::EmbedTable,
            Realce::EmbedCallout,
            Realce::EmbedColumns,
            Realce::EmbedGallery,
            Realce::EmbedQuery,
            Realce::EmbedTimeline,
            Realce::EmbedActions,
        ];
        let cores: Vec<Color> = papeis.iter().map(|r| t.estilo(*r).fg.unwrap()).collect();
        for (i, a) in cores.iter().enumerate() {
            for (j, b) in cores.iter().enumerate().skip(i + 1) {
                assert_ne!(
                    a, b,
                    "{:?} e {:?} saíram com a mesma cor: {a:?}",
                    papeis[i], papeis[j]
                );
            }
        }
    }

    #[test]
    fn a_pilula_tinge_o_fundo_com_15_por_cento_da_cor() {
        // `color-mix(in srgb, X 15%, transparent)` da janela, contra o
        // fundo da tela.
        assert_eq!(
            misturar(Color::Rgb(200, 100, 0), Color::Rgb(0, 0, 100), 0.15),
            Color::Rgb(30, 15, 85)
        );
        let t = Tema::novo("escuro");
        let p = t.pilula(Realce::BadgeInfo);
        assert_eq!(p.fg, t.estilo(Realce::BadgeInfo).fg);
        assert_ne!(p.bg, None);
        assert_ne!(p.bg, p.fg, "o fundo tingido não pode ser a própria cor do texto");
        // O neutro já tem fundo: sai como está.
        assert_eq!(t.pilula(Realce::Evento), t.estilo(Realce::Evento));
    }

    #[test]
    fn o_foco_nao_tem_fundo_e_cada_variante_do_callout_tem_o_seu() {
        let t = Tema::novo("escuro");
        assert_eq!(t.fundo_da_regiao(Realce::BordaUnidade), None);
        let fundos: Vec<Color> = [
            Realce::CalloutInfo,
            Realce::CalloutSucesso,
            Realce::CalloutAtencao,
            Realce::CalloutErro,
            Realce::CalloutDica,
        ]
        .iter()
        .map(|r| t.fundo_da_regiao(*r).expect("callout sem fundo"))
        .collect();
        for (i, a) in fundos.iter().enumerate() {
            for b in fundos.iter().skip(i + 1) {
                assert_ne!(a, b, "duas variantes com o mesmo fundo: {fundos:?}");
            }
        }
        // O fundo não é a superfície pura nem o fundo da tela: é tingido.
        let base = t.cor("bg-base", Color::Reset);
        let superficie = t.cor("bg-surface", Color::Reset);
        assert!(fundos.iter().all(|f| *f != base && *f != superficie));
    }

    #[test]
    fn todo_embed_tem_fundo_tingido_pela_sua_cor() {
        // A caixa preenchida do callout, estendida (ciclo 308): cada tipo
        // de embed com o fundo no tom dele — e fundos diferentes entre
        // si, senão dois embeds vizinhos voltam a parecer um só.
        let t = Tema::novo("escuro");
        let papeis = [
            Realce::Embed,
            Realce::EmbedKanban,
            Realce::EmbedCalendar,
            Realce::EmbedTable,
            Realce::EmbedColumns,
            Realce::EmbedGallery,
            Realce::EmbedQuery,
            Realce::EmbedTimeline,
            Realce::EmbedActions,
        ];
        let fundos: Vec<Color> = papeis
            .iter()
            .map(|r| t.fundo_da_regiao(*r).unwrap_or_else(|| panic!("{r:?} sem fundo")))
            .collect();
        for (i, a) in fundos.iter().enumerate() {
            for (j, b) in fundos.iter().enumerate().skip(i + 1) {
                assert_ne!(a, b, "{:?} e {:?} com o mesmo fundo", papeis[i], papeis[j]);
            }
        }
    }

    #[test]
    fn a_grade_se_destaca_do_fundo_de_todo_embed() {
        // Com `--border`, as bordas dos dias do calendário sumiam dentro
        // da caixa preenchida (visto na tela, ciclo 308). A grade precisa
        // de distância de TODO fundo de embed.
        let t = Tema::novo("escuro");
        let Some(Color::Rgb(gr, gg, gb)) = t.estilo(Realce::Grade).fg else { panic!() };
        for r in [Realce::EmbedCalendar, Realce::EmbedTable, Realce::EmbedKanban] {
            let Some(Color::Rgb(fr, fg, fb)) = t.fundo_da_regiao(r) else { panic!() };
            let distancia = (gr as i32 - fr as i32).abs() + (gg as i32 - fg as i32).abs() + (gb as i32 - fb as i32).abs();
            assert!(distancia > 60, "{r:?}: grade e fundo quase iguais ({distancia})");
        }
    }
}
