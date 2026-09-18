//! O que vai pro agente, montado uma vez só (ciclo 423).
//!
//! Prompt, peso de cada parte e poda estavam no `main` da TUI. A janela
//! montava o prompt por conta própria — e por isso ganhou a transclusão
//! (414) mas não o orçamento (417) nem a poda (419): quem tem duas
//! implementações tem duas UIs que divergem, e a que fica pra trás é
//! sempre a que a pessoa está usando naquele dia.
//!
//! Aqui está a parte que não depende de IO: dado o histórico, os anexos
//! JÁ LIDOS e a pergunta, sai o texto que o agente recebe, o peso por
//! parte e a lista do que foi cortado pra caber. Quem lê arquivo é cada
//! UI, do jeito dela — a TUI pelo `anotadinho-ipc`, a janela pelo
//! comando do Tauri.

use crate::conversa::{montar_prompt, Contexto, Mensagem};
use crate::orcamento::{podar, Corte, Orcamento, Peso};

/// O envio pronto.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Envio {
    /// O texto exato que vai pro agente.
    pub prompt: String,
    /// O peso de cada parte, da maior pra menor.
    pub orcamento: Orcamento,
    /// O que a poda tirou pra caber no teto. Vazio quando coube.
    pub cortes: Vec<Corte>,
}

/// Monta o envio: poda se precisa, junta o prompt e pesa as partes.
///
/// `historico` é a conversa ATÉ a pergunta (sem ela). `limite_historico`
/// é quantas mensagens recentes entram; `teto` é o limite de tokens
/// estimados (`0` desliga a poda).
pub fn montar(
    contextos: &[Contexto],
    historico: &[Mensagem],
    pergunta: &str,
    limite_historico: usize,
    teto: usize,
    // `vault`: o caminho, pra dizer ao agente COMO agir (ciclo 425).
    // Vazio omite o bloco.
    vault: &str,
) -> Envio {
    let mut contextos: Vec<Contexto> = contextos.to_vec();
    let recentes = historico.len().min(limite_historico);
    let mut historico: Vec<Mensagem> = historico[historico.len() - recentes..].to_vec();
    let cortes = podar(&mut contextos, &mut historico, pergunta, teto);
    // O "como agir" vai ANTES da pergunta e fora dos blocos de dado: é
    // instrução do app, não material lido do vault (ciclo 425).
    let pergunta_com_regras = if vault.trim().is_empty() {
        pergunta.to_string()
    } else {
        format!("{}\n\n{}", crate::ferramentas::como_agir(vault), pergunta.trim())
    };
    let prompt = montar_prompt(&historico, &pergunta_com_regras, &contextos, limite_historico);
    let mut partes: Vec<Peso> = contextos.iter().map(|c| Peso::novo(c.nome.clone(), &c.conteudo)).collect();
    let historico_texto: String =
        historico.iter().map(|m| m.texto.clone()).collect::<Vec<_>>().join("\n");
    partes.push(Peso::novo(format!("histórico ({} msg)", historico.len()), &historico_texto));
    partes.push(Peso::novo("pergunta", pergunta));
    if !vault.trim().is_empty() {
        // Pesa junto: são tokens que vão, e a prévia não pode escondê-los.
        partes.push(Peso::novo("como agir", &crate::ferramentas::como_agir(vault)));
    }
    Envio { prompt, orcamento: Orcamento::novo(partes, teto), cortes }
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::conversa::Autor;

    fn msg(i: usize) -> Mensagem {
        Mensagem { autor: Autor::Voce, quando: "2026-09-17 10:00".into(), texto: format!("mensagem {i}") }
    }

    fn ctx(nome: &str, tokens: usize) -> Contexto {
        Contexto { nome: nome.into(), conteudo: "a".repeat(tokens * 4) }
    }

    #[test]
    fn o_prompt_leva_contexto_historico_e_pergunta() {
        let e = montar(&[ctx("spec.md", 10)], &[msg(1), msg(2)], "e agora?", 12, 0, "");
        assert!(e.prompt.contains("spec.md"), "{}", e.prompt);
        assert!(e.prompt.contains("mensagem 2"), "{}", e.prompt);
        assert!(e.prompt.trim_end().ends_with("e agora?"), "a pergunta por último:\n{}", e.prompt);
        assert!(e.cortes.is_empty());
        // O peso sai por parte, da maior pra menor.
        assert_eq!(e.orcamento.partes[0].nome, "spec.md");
        assert!(e.orcamento.partes.iter().any(|p| p.nome == "histórico (2 msg)"));
        assert!(e.orcamento.partes.iter().any(|p| p.nome == "pergunta"));
    }

    #[test]
    fn so_o_historico_recente_entra() {
        let historico: Vec<Mensagem> = (0..20).map(msg).collect();
        let e = montar(&[], &historico, "p", 3, 0, "");
        assert!(e.prompt.contains("mensagem 19") && !e.prompt.contains("mensagem 16"), "{}", e.prompt);
        assert!(e.orcamento.partes.iter().any(|p| p.nome == "histórico (3 msg)"));
    }

    #[test]
    fn com_teto_apertado_poda_e_conta_o_que_cortou() {
        let grande = Contexto {
            nome: "spec.md".into(),
            conteudo: format!("# Título\n\n{}\n", "x".repeat(8000)),
        };
        let e = montar(&[grande], &[msg(1)], "p", 12, 100, "");
        assert!(!e.cortes.is_empty(), "tinha que podar");
        assert!(e.prompt.contains("# Título"), "o índice fica:\n{}", e.prompt);
        assert!(!e.prompt.contains(&"x".repeat(100)), "o corpo sai:\n{}", e.prompt);
        // E o peso medido é o do que VAI, não o do que se queria mandar.
        assert!(e.orcamento.tokens() < 200, "{:?}", e.orcamento.partes);
    }

    #[test]
    fn sem_teto_nada_e_podado() {
        let grande = Contexto { nome: "spec.md".into(), conteudo: "x".repeat(400_000) };
        let e = montar(&[grande], &[msg(1)], "p", 12, 0, "");
        assert!(e.cortes.is_empty());
        assert!(e.orcamento.tokens() > 99_000);
    }

    #[test]
    fn com_vault_o_prompt_diz_como_agir() {
        let e = montar(&[], &[msg(1)], "resuma", 12, 0, "/home/eu/vault");
        assert!(e.prompt.contains("# Como agir neste vault"), "{}", e.prompt);
        assert!(e.prompt.contains("/home/eu/vault"), "{}", e.prompt);
        // Antes da pergunta, e fora do bloco de DADO: é instrução do
        // app, não material lido do vault.
        let onde_regras = e.prompt.find("Como agir").unwrap();
        let onde_pergunta = e.prompt.rfind("resuma").unwrap();
        assert!(onde_regras < onde_pergunta, "{}", e.prompt);
        assert!(!e.prompt.contains("DADO-ANOTADINHO PAGINA como agir"), "{}", e.prompt);
        // E pesa: são tokens que vão.
        assert!(e.orcamento.partes.iter().any(|p| p.nome == "como agir"));
    }

    #[test]
    fn sem_vault_nada_muda() {
        let e = montar(&[], &[msg(1)], "resuma", 12, 0, "  ");
        assert!(!e.prompt.contains("Como agir"), "{}", e.prompt);
        assert!(!e.orcamento.partes.iter().any(|p| p.nome == "como agir"));
    }
}
