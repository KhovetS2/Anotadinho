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
        }
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
        ] {
            let e = t.estilo(r);
            let cor = e.fg.or(e.bg).unwrap_or(Color::Reset);
            assert!(
                matches!(cor, Color::Rgb(..)),
                "{r:?} não veio do CSS: {cor:?}"
            );
        }
    }
}
