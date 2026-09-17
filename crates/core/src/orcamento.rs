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

// ---------------------------------------------------------------------
// Poda (ciclo 419)
// ---------------------------------------------------------------------

/// Quantas mensagens do histórico a poda nunca tira. Abaixo disso a
/// conversa perde o fio e o agente responde a pergunta errada.
pub const HISTORICO_MINIMO: usize = 4;

/// O que a poda fez, pra tela contar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Corte {
    /// Mensagens antigas que saíram.
    Historico { de: usize, para: usize },
    /// Anexo reduzido aos cabeçalhos.
    Esqueleto { nome: String, de: usize, para: usize },
    /// Anexo cortado no meio, porque nem o esqueleto coube.
    Truncado { nome: String, de: usize, para: usize },
}

impl Corte {
    /// Como se lê na prévia.
    pub fn rotulo(&self) -> String {
        match self {
            Self::Historico { de, para } => format!("histórico: {de} → {para} mensagens"),
            Self::Esqueleto { nome, de, para } => {
                format!("{nome}: só os cabeçalhos (~{} → ~{})", humano(*de), humano(*para))
            }
            Self::Truncado { nome, de, para } => {
                format!("{nome}: cortado (~{} → ~{})", humano(*de), humano(*para))
            }
        }
    }
}

/// O texto reduzido aos cabeçalhos, dizendo o que ficou de fora.
///
/// Um anexo grande vira o ÍNDICE dele: o agente continua sabendo que a
/// página existe e do que ela trata, e pede o resto se precisar — em vez
/// de receber metade de um parágrafo cortado no meio de uma frase.
pub fn esqueleto(texto: &str) -> String {
    let mut fora: Vec<String> = Vec::new();
    let mut omitidas = 0usize;
    for linha in texto.lines() {
        if linha.trim_start().starts_with('#') {
            if omitidas > 0 {
                fora.push(format!("… {omitidas} linha(s)"));
                omitidas = 0;
            }
            fora.push(linha.to_string());
        } else if !linha.trim().is_empty() {
            omitidas += 1;
        }
    }
    if omitidas > 0 {
        fora.push(format!("… {omitidas} linha(s)"));
    }
    fora.join("\n")
}

/// Corta um texto em `tokens` tokens, avisando no fim.
fn truncar(texto: &str, tokens_alvo: usize) -> String {
    let limite = tokens_alvo.saturating_mul(CARACTERES_POR_TOKEN);
    if texto.chars().count() <= limite {
        return texto.to_string();
    }
    let cortado: String = texto.chars().take(limite).collect();
    format!("{cortado}\n… [cortado pra caber no contexto]")
}

/// Faz o contexto caber no teto, do que menos importa pro que mais.
///
/// A ordem é deliberada: primeiro o histórico ANTIGO (numa conversa
/// longa o começo é o que menos pesa na resposta), depois o anexo mais
/// gordo vira esqueleto, e só então se corta texto no meio. Nada disso é
/// silencioso — cada corte volta na lista, e quem chama mostra.
///
/// `0` no teto desliga a poda.
pub fn podar(
    contextos: &mut Vec<crate::conversa::Contexto>,
    historico: &mut Vec<crate::conversa::Mensagem>,
    pergunta: &str,
    teto: usize,
) -> Vec<Corte> {
    let mut cortes = Vec::new();
    if teto == 0 {
        return cortes;
    }
    let total = |c: &Vec<crate::conversa::Contexto>, h: &Vec<crate::conversa::Mensagem>| -> usize {
        c.iter().map(|x| tokens(&x.conteudo)).sum::<usize>()
            + h.iter().map(|m| tokens(&m.texto)).sum::<usize>()
            + tokens(pergunta)
    };
    if total(contextos, historico) <= teto {
        return cortes;
    }

    // 1. O histórico antigo.
    if historico.len() > HISTORICO_MINIMO {
        let antes = historico.len();
        while historico.len() > HISTORICO_MINIMO && total(contextos, historico) > teto {
            historico.remove(0);
        }
        if historico.len() < antes {
            cortes.push(Corte::Historico { de: antes, para: historico.len() });
        }
    }

    // 2. O anexo mais gordo COM cabeçalho vira esqueleto, do maior pro
    //    menor. Sem cabeçalho não há esqueleto que preste — "… 400
    //    linha(s)" não é índice de nada —, e o caso é do passo 3.
    while total(contextos, historico) > teto {
        let Some(i) = maior(contextos, true) else { break };
        let de = tokens(&contextos[i].conteudo);
        let reduzido = esqueleto(&contextos[i].conteudo);
        let para = tokens(&reduzido);
        if para >= de {
            break;
        }
        let nome = contextos[i].nome.clone();
        contextos[i].conteudo = reduzido;
        cortes.push(Corte::Esqueleto { nome, de, para });
    }

    // 3. Ainda não coube: corta o maior no tamanho que sobra.
    if total(contextos, historico) > teto {
        if let Some(i) = maior(contextos, false) {
            let outros: usize = total(contextos, historico) - tokens(&contextos[i].conteudo);
            let sobra = teto.saturating_sub(outros);
            let de = tokens(&contextos[i].conteudo);
            let cortado = truncar(&contextos[i].conteudo, sobra);
            let para = tokens(&cortado);
            let nome = contextos[i].nome.clone();
            contextos[i].conteudo = cortado;
            cortes.push(Corte::Truncado { nome, de, para });
        }
    }
    cortes
}

/// O índice do anexo mais pesado que ainda tem o que cortar.
///
/// `com_cabecalho` restringe aos que têm `#` — os únicos que viram
/// esqueleto útil.
fn maior(contextos: &[crate::conversa::Contexto], com_cabecalho: bool) -> Option<usize> {
    contextos
        .iter()
        .enumerate()
        .filter(|(_, c)| !c.conteudo.trim().is_empty())
        .filter(|(_, c)| !com_cabecalho || c.conteudo.lines().any(|l| l.trim_start().starts_with('#')))
        .max_by_key(|(_, c)| tokens(&c.conteudo))
        .map(|(i, _)| i)
}

#[cfg(test)]
mod testes_de_poda {
    use super::*;
    use crate::conversa::{Autor, Contexto, Mensagem};

    fn ctx(nome: &str, tokens: usize) -> Contexto {
        Contexto { nome: nome.into(), conteudo: "a".repeat(tokens * CARACTERES_POR_TOKEN) }
    }

    fn msg(i: usize) -> Mensagem {
        Mensagem { autor: Autor::Voce, quando: "2026-09-17 10:00".into(), texto: format!("mensagem {i}") }
    }

    #[test]
    fn cabendo_no_teto_nada_e_cortado() {
        let mut c = vec![ctx("a.md", 100)];
        let mut h = vec![msg(1), msg(2)];
        assert!(podar(&mut c, &mut h, "pergunta", 1000).is_empty());
        assert_eq!(tokens(&c[0].conteudo), 100);
        assert_eq!(h.len(), 2);
        // Teto 0 desliga.
        let mut c = vec![ctx("a.md", 100_000)];
        assert!(podar(&mut c, &mut h, "pergunta", 0).is_empty());
    }

    #[test]
    fn o_historico_antigo_sai_primeiro_e_o_minimo_fica() {
        let mut c = vec![ctx("a.md", 50)];
        let mut h: Vec<Mensagem> = (0..10).map(msg).collect();
        // Teto com folga pro anexo: o histórico basta pra caber.
        let cortes = podar(&mut c, &mut h, "p", 75);
        assert!(matches!(cortes[0], Corte::Historico { de: 10, .. }), "{cortes:?}");
        assert!(h.len() >= HISTORICO_MINIMO, "o fio da conversa não se perde: {}", h.len());
        // As que ficaram são as ÚLTIMAS.
        assert_eq!(h.last().unwrap().texto, "mensagem 9");
        // O anexo não foi tocado: o histórico bastou.
        assert_eq!(tokens(&c[0].conteudo), 50);
    }

    #[test]
    fn o_anexo_mais_gordo_vira_esqueleto() {
        let grande = Contexto {
            nome: "spec.md".into(),
            conteudo: format!("# Título\n\n{}\n\n## Parte\n\n{}\n", "x".repeat(4000), "y".repeat(4000)),
        };
        let mut c = vec![grande, ctx("pequeno.md", 10)];
        let mut h = vec![msg(1)];
        let cortes = podar(&mut c, &mut h, "p", 100);
        assert!(matches!(&cortes[0], Corte::Esqueleto { nome, .. } if nome == "spec.md"), "{cortes:?}");
        assert!(c[0].conteudo.contains("# Título") && c[0].conteudo.contains("## Parte"), "{}", c[0].conteudo);
        assert!(c[0].conteudo.contains("… 1 linha(s)"), "diz o que sumiu:\n{}", c[0].conteudo);
        assert_eq!(tokens(&c[1].conteudo), 10, "o pequeno fica inteiro");
    }

    #[test]
    fn sem_cabecalho_o_corte_e_no_texto() {
        let mut c = vec![ctx("plano.md", 1000)];
        let mut h = vec![msg(1)];
        let cortes = podar(&mut c, &mut h, "p", 100);
        assert!(matches!(&cortes.last(), Some(Corte::Truncado { nome, .. }) if nome == "plano.md"), "{cortes:?}");
        assert!(c[0].conteudo.contains("cortado pra caber"), "{}", c[0].conteudo);
        let sobrou: usize = tokens(&c[0].conteudo) + h.iter().map(|m| tokens(&m.texto)).sum::<usize>() + tokens("p");
        assert!(sobrou <= 110, "coube com folga de arredondamento: {sobrou}");
    }

    #[test]
    fn o_rotulo_do_corte_e_legivel() {
        assert_eq!(
            Corte::Historico { de: 12, para: 4 }.rotulo(),
            "histórico: 12 → 4 mensagens"
        );
        assert_eq!(
            Corte::Esqueleto { nome: "spec.md".into(), de: 3200, para: 120 }.rotulo(),
            "spec.md: só os cabeçalhos (~3,2k → ~120)"
        );
    }

    #[test]
    fn o_esqueleto_guarda_os_cabecalhos_e_conta_o_resto() {
        let e = esqueleto("# A\n\numa\nduas\n\n## B\n\ntrês\n");
        assert_eq!(e, "# A\n… 2 linha(s)\n## B\n… 1 linha(s)");
    }
}
