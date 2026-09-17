//! O peso do contexto que vai pro agente (ciclo 417).
//!
//! Anexar era escolher no escuro: nada dizia quanto texto já estava indo.
//! Com transclusão (ciclo 414) piorou — um anexo pode trazer outros
//! cinco —, e estourar a janela do modelo é falha silenciosa: a resposta
//! sai pior e ninguém sabe por quê.
//!
//! Aqui se mede o que já existe: cada parte do prompt, em caracteres e
//! em tokens estimados, com um teto opcional. É a régua de que a prévia
//! (a tela que mostra o prompt montado) e o cabeçalho da conversa
//! precisam.
//!
//! # Sobre a estimativa
//!
//! Quatro caracteres por token é a regra de bolso para português em
//! markdown. É ESTIMATIVA e o nome diz isso em toda a UI (`~3,2k`):
//! contar de verdade exigiria o tokenizador de cada modelo, que muda com
//! o modelo e não cabe num app que chama um CLI de fora. Errar por 15%
//! não muda decisão nenhuma; não ter número nenhum muda.

/// Caracteres por token, na estimativa.
pub const CARACTERES_POR_TOKEN: usize = 4;

/// Tokens estimados de um texto.
pub fn tokens(texto: &str) -> usize {
    texto.chars().count().div_ceil(CARACTERES_POR_TOKEN)
}

/// Uma parte do prompt e o que ela pesa.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peso {
    /// Como se lê na tela (`pages/specs/a.md`, `histórico`, `pergunta`).
    pub nome: String,
    pub caracteres: usize,
    pub tokens: usize,
}

impl Peso {
    pub fn novo(nome: impl Into<String>, texto: &str) -> Self {
        Self { nome: nome.into(), caracteres: texto.chars().count(), tokens: tokens(texto) }
    }
}

/// O peso do prompt inteiro, parte por parte.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Orcamento {
    /// As partes, da maior pra menor — quem quer cortar quer ver a maior
    /// primeiro.
    pub partes: Vec<Peso>,
    /// Teto de tokens; `0` = sem teto.
    pub teto: usize,
}

impl Orcamento {
    /// Monta o orçamento ordenando as partes pelo peso.
    pub fn novo(mut partes: Vec<Peso>, teto: usize) -> Self {
        partes.sort_by(|a, b| b.tokens.cmp(&a.tokens));
        Self { partes, teto }
    }

    pub fn tokens(&self) -> usize {
        self.partes.iter().map(|p| p.tokens).sum()
    }

    pub fn caracteres(&self) -> usize {
        self.partes.iter().map(|p| p.caracteres).sum()
    }

    /// Passou do teto?
    pub fn estourou(&self) -> bool {
        self.teto > 0 && self.tokens() > self.teto
    }

    /// Está chegando perto (80% do teto)? É o aviso que vale a pena dar
    /// ANTES de a resposta piorar.
    pub fn apertado(&self) -> bool {
        self.teto > 0 && !self.estourou() && self.tokens() * 5 >= self.teto * 4
    }

    /// Uma linha pro cabeçalho: `~3,2k tokens de 40k`.
    pub fn resumo(&self) -> String {
        let atual = humano(self.tokens());
        match self.teto {
            0 => format!("~{atual} tokens"),
            teto => format!("~{atual} de {} tokens", humano(teto)),
        }
    }
}

/// Número curto de ler: 980, 3,2k, 41k.
pub fn humano(n: usize) -> String {
    if n < 1000 {
        return n.to_string();
    }
    let milhares = n as f64 / 1000.0;
    if milhares < 10.0 {
        // Uma casa só abaixo de 10k: `3,2k` informa, `3,25k` não.
        format!("{:.1}k", milhares).replace('.', ",")
    } else {
        format!("{}k", milhares.round() as usize)
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_token_e_estimado_por_caractere() {
        assert_eq!(tokens(""), 0);
        assert_eq!(tokens("abcd"), 1);
        // Arredonda pra cima: cinco caracteres já são dois tokens.
        assert_eq!(tokens("abcde"), 2);
        // Conta CARACTERE, não byte: acento não vale dois.
        assert_eq!(tokens("ação"), tokens("acao"));
    }

    #[test]
    fn as_partes_saem_da_maior_pra_menor() {
        let o = Orcamento::novo(
            vec![
                Peso::novo("pergunta", "curta"),
                Peso::novo("spec.md", &"x".repeat(4000)),
                Peso::novo("histórico", &"y".repeat(400)),
            ],
            0,
        );
        let nomes: Vec<&str> = o.partes.iter().map(|p| p.nome.as_str()).collect();
        assert_eq!(nomes, ["spec.md", "histórico", "pergunta"]);
        assert_eq!(o.tokens(), 1000 + 100 + 2);
        assert_eq!(o.caracteres(), 4000 + 400 + 5);
    }

    #[test]
    fn o_teto_diz_quando_aperta_e_quando_estoura() {
        let peso = |n: usize| Peso::novo("x", &"a".repeat(n * CARACTERES_POR_TOKEN));
        let com = |t: usize, teto: usize| Orcamento::novo(vec![peso(t)], teto);
        assert!(!com(100, 1000).apertado() && !com(100, 1000).estourou());
        // 80% é o aviso.
        assert!(com(800, 1000).apertado() && !com(800, 1000).estourou());
        assert!(com(1001, 1000).estourou() && !com(1001, 1000).apertado());
        // Sem teto, nunca aperta nem estoura.
        assert!(!com(99_999, 0).apertado() && !com(99_999, 0).estourou());
    }

    #[test]
    fn o_resumo_e_curto_de_ler() {
        let peso = |n: usize| Peso::novo("x", &"a".repeat(n * CARACTERES_POR_TOKEN));
        assert_eq!(Orcamento::novo(vec![peso(3200)], 40_000).resumo(), "~3,2k de 40k tokens");
        assert_eq!(Orcamento::novo(vec![peso(120)], 0).resumo(), "~120 tokens");
        assert_eq!(humano(999), "999");
        assert_eq!(humano(1000), "1,0k");
        assert_eq!(humano(41_400), "41k");
    }
}
