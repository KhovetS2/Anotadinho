//! Conversa com o agente, guardada como PÁGINA do vault (ciclo 202).
//!
//! A decisão de desenho: a conversa não vai pra um banco interno, vai
//! pro markdown. Isso é o que permite ligá-la ao trabalho
//! (`[[Conversa sobre X]]` dentro da spec, com backlink dos dois lados),
//! versioná-la no git junto do resto, e deixar o próprio agente lê-la
//! como contexto — porque é uma página como qualquer outra, alcançável
//! pela consulta.
//!
//! O formato é heading por mensagem, pra continuar legível fora do app:
//!
//! ```text
//! ## você · 2026-08-22 10:30
//!
//! Pergunta.
//!
//! ## agente · 2026-08-22 10:31
//!
//! Resposta.
//! ```
//!
//! Nada de HTML nem de campo escondido: quem abrir o `.md` no vim vê
//! uma conversa.

use serde::{Deserialize, Serialize};

/// Quem falou.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Autor {
    /// A pessoa.
    Voce,
    /// O modelo configurado.
    Agente,
}

impl Autor {
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Voce => "você",
            Self::Agente => "agente",
        }
    }

    fn from_slug(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "você" | "voce" | "eu" | "user" => Some(Self::Voce),
            "agente" | "assistant" | "modelo" => Some(Self::Agente),
            _ => None,
        }
    }
}

/// Uma fala.
#[derive(Debug, Clone, PartialEq)]
pub struct Mensagem {
    pub autor: Autor,
    /// `"YYYY-MM-DD HH:MM"`. Vem de fora — o core não tem relógio.
    pub quando: String,
    pub texto: String,
}

/// Lê as mensagens do corpo de uma página de conversa.
///
/// Texto antes da primeira mensagem (uma introdução escrita à mão, por
/// exemplo) é ignorado de propósito: é contexto pra quem lê, não fala de
/// ninguém.
pub fn parse(body: &str) -> Vec<Mensagem> {
    let mut out: Vec<Mensagem> = Vec::new();
    let mut atual: Option<Mensagem> = None;

    for linha in body.lines() {
        if let Some((autor, quando)) = cabecalho_de_mensagem(linha) {
            if let Some(m) = atual.take() {
                out.push(finalizar(m));
            }
            atual = Some(Mensagem { autor, quando, texto: String::new() });
            continue;
        }
        if let Some(m) = atual.as_mut() {
            m.texto.push_str(linha);
            m.texto.push('\n');
        }
    }
    if let Some(m) = atual.take() {
        out.push(finalizar(m));
    }
    out
}

fn finalizar(mut m: Mensagem) -> Mensagem {
    m.texto = m.texto.trim().to_string();
    m
}

/// `## você · 2026-08-22 10:30` → (autor, quando).
fn cabecalho_de_mensagem(linha: &str) -> Option<(Autor, String)> {
    let resto = linha.strip_prefix("## ")?;
    let (autor, quando) = resto.split_once('·')?;
    let autor = Autor::from_slug(autor)?;
    Some((autor, quando.trim().to_string()))
}

/// Serializa uma conversa inteira.
pub fn serializar(mensagens: &[Mensagem]) -> String {
    mensagens
        .iter()
        .map(|m| format!("## {} · {}\n\n{}\n", m.autor.slug(), m.quando, m.texto.trim()))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Acrescenta uma mensagem ao corpo, preservando o que já estava lá.
pub fn append(body: &str, mensagem: &Mensagem) -> String {
    let base = body.trim_end();
    let nova = format!("## {} · {}\n\n{}\n", mensagem.autor.slug(), mensagem.quando, mensagem.texto.trim());
    if base.is_empty() {
        nova
    } else {
        format!("{base}\n\n{nova}")
    }
}

/// Monta o prompt que vai pro agente.
///
/// Ordem pensada pro modelo: primeiro o CONTEXTO (a página que a pessoa
/// estava olhando), depois o histórico, e a pergunta por último — o que
/// está mais perto do fim é o que pesa mais na resposta.
///
/// `limite_historico` corta as mensagens mais ANTIGAS, não as recentes:
/// numa conversa longa, o começo é o que menos importa.
pub fn montar_prompt(
    historico: &[Mensagem],
    pergunta: &str,
    contexto: Option<&str>,
    limite_historico: usize,
) -> String {
    let mut partes: Vec<String> = Vec::new();

    if let Some(ctx) = contexto.map(str::trim).filter(|c| !c.is_empty()) {
        partes.push(format!("# Contexto: a página aberta\n\n{ctx}"));
    }

    let recentes: &[Mensagem] = if historico.len() > limite_historico {
        &historico[historico.len() - limite_historico..]
    } else {
        historico
    };
    if !recentes.is_empty() {
        partes.push(format!("# Conversa até aqui\n\n{}", serializar(recentes)));
    }

    partes.push(format!("# Pergunta\n\n{}", pergunta.trim()));
    partes.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn msg(autor: Autor, texto: &str) -> Mensagem {
        Mensagem { autor, quando: "2026-08-22 10:00".into(), texto: texto.into() }
    }

    #[test]
    fn parse_de_conversa_vazia() {
        assert!(parse("").is_empty());
        assert!(parse("só um texto solto\n").is_empty());
    }

    #[test]
    fn parse_le_autor_e_texto() {
        let md = "## você · 2026-08-22 10:30\n\nOlá\n\n## agente · 2026-08-22 10:31\n\nOi\n";
        let m = parse(md);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].autor, Autor::Voce);
        assert_eq!(m[0].quando, "2026-08-22 10:30");
        assert_eq!(m[0].texto, "Olá");
        assert_eq!(m[1].autor, Autor::Agente);
        assert_eq!(m[1].texto, "Oi");
    }

    #[test]
    fn texto_antes_da_primeira_mensagem_e_ignorado() {
        let m = parse("uma introdução\n\n## você · 2026-08-22 10:30\n\nPergunta\n");
        assert_eq!(m.len(), 1);
        assert_eq!(m[0].texto, "Pergunta");
    }

    #[test]
    fn mensagem_multilinha_com_markdown_sobrevive() {
        let md = "## agente · 2026-08-22 10:31\n\nUma lista:\n\n- um\n- dois\n\n```rust\nfn x() {}\n```\n";
        let m = parse(md);
        assert_eq!(m.len(), 1);
        assert!(m[0].texto.contains("- dois"), "{}", m[0].texto);
        assert!(m[0].texto.contains("fn x() {}"), "{}", m[0].texto);
    }

    #[test]
    fn heading_comum_nao_vira_mensagem() {
        // Um `## Resumo` dentro da resposta é conteúdo, não cabeçalho de
        // fala — o `·` com autor conhecido é o que distingue.
        let m = parse("## agente · 2026-08-22 10:31\n\n## Resumo\n\ntexto\n");
        assert_eq!(m.len(), 1);
        assert!(m[0].texto.contains("## Resumo"));
    }

    #[test]
    fn round_trip_preserva_as_mensagens() {
        let originais = vec![msg(Autor::Voce, "pergunta"), msg(Autor::Agente, "resposta")];
        assert_eq!(parse(&serializar(&originais)), originais);
    }

    #[test]
    fn append_preserva_o_que_ja_existia() {
        let body = serializar(&[msg(Autor::Voce, "primeira")]);
        let novo = append(&body, &msg(Autor::Agente, "segunda"));
        let m = parse(&novo);
        assert_eq!(m.len(), 2);
        assert_eq!(m[0].texto, "primeira");
        assert_eq!(m[1].texto, "segunda");
    }

    #[test]
    fn append_em_corpo_vazio_nao_deixa_lixo() {
        let novo = append("", &msg(Autor::Voce, "só esta"));
        assert!(!novo.starts_with('\n'), "{novo:?}");
        assert_eq!(parse(&novo).len(), 1);
    }

    #[test]
    fn prompt_tem_contexto_historico_e_pergunta_nessa_ordem() {
        let h = vec![msg(Autor::Voce, "antiga")];
        let p = montar_prompt(&h, "nova pergunta", Some("conteúdo da página"), 10);
        let i_ctx = p.find("Contexto").unwrap();
        let i_hist = p.find("Conversa até aqui").unwrap();
        let i_perg = p.find("# Pergunta").unwrap();
        assert!(i_ctx < i_hist && i_hist < i_perg, "ordem errada:\n{p}");
        assert!(p.contains("nova pergunta"));
    }

    #[test]
    fn prompt_corta_as_mensagens_mais_antigas() {
        let h: Vec<Mensagem> = (0..10).map(|i| msg(Autor::Voce, &format!("m{i}"))).collect();
        let p = montar_prompt(&h, "pergunta", None, 3);
        assert!(!p.contains("m0"), "devia ter cortado o começo:\n{p}");
        assert!(p.contains("m9"), "não podia cortar o fim:\n{p}");
    }

    #[test]
    fn prompt_sem_contexto_nem_historico_e_so_a_pergunta() {
        let p = montar_prompt(&[], "oi", None, 10);
        assert!(!p.contains("Contexto"));
        assert!(!p.contains("Conversa até aqui"));
        assert!(p.contains("oi"));
    }
}
