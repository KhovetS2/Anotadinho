//! Propostas de escrita do agente, sujeitas a revisão (ciclo 204).
//!
//! O problema que isto resolve: hoje o agente escreve DIRETO no vault
//! pelo CLI. É rápido e é o que trava a confiança — você não revisa
//! nada, descobre depois.
//!
//! Aqui ele grava uma PROPOSTA, e a UI mostra o diff pra você aceitar ou
//! recusar. A defesa não depende de o modelo se comportar: mesmo que ele
//! seja enganado por uma injeção, o estrago para na sua tela.
//!
//! As propostas vivem em `.anotadinho/propostas/`, fora de `pages/`, pra
//! não aparecerem como página do vault nem entrarem em consulta.

use serde::{Deserialize, Serialize};

/// Pasta das propostas, relativa à raiz do vault.
pub const PASTA: &str = ".anotadinho/propostas";

/// O que fazer com um arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Operacao {
    /// Página que ainda não existe.
    Criar,
    /// Substituir o conteúdo de uma existente.
    Substituir,
}

/// Uma escrita proposta, ainda não aplicada.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Proposta {
    /// Identificador — vira o nome do arquivo em `PASTA`.
    pub id: String,
    /// Quem propôs (nome do adaptador, ou "cli").
    pub autor: String,
    /// Quando, `"YYYY-MM-DD HH:MM"`.
    pub quando: String,
    /// Por que — o que o agente diz que está fazendo.
    #[serde(default)]
    pub motivo: String,
    /// Página alvo, relativa ao vault.
    pub alvo: String,
    pub operacao: Operacao,
    /// Conteúdo proposto, inteiro.
    pub conteudo: String,
}

/// Por que uma proposta não pode ser aplicada.
#[derive(Debug, Clone, PartialEq)]
pub enum Recusa {
    /// Caminho tentando escapar do vault.
    AlvoForaDoVault,
    /// `Criar` numa página que já existe, ou `Substituir` numa que não
    /// existe — nos dois casos o agente decidiu com uma foto velha do
    /// vault, e aplicar seria escrever por cima do que ele não viu.
    EstadoMudou,
    /// O conteúdo tem embed com erro (`EmbedData::validate`).
    ConteudoInvalido(String),
}

impl Recusa {
    pub fn mensagem(&self) -> String {
        match self {
            Self::AlvoForaDoVault => "o alvo aponta pra fora do vault".to_string(),
            Self::EstadoMudou => {
                "o vault mudou desde que a proposta foi escrita — peça de novo".to_string()
            }
            Self::ConteudoInvalido(d) => format!("o conteúdo tem embed inválido: {d}"),
        }
    }
}

impl Proposta {
    /// Confere o que dá pra conferir SEM tocar no disco.
    ///
    /// `existe_alvo` vem de fora pra esta função continuar pura e
    /// testável sem sistema de arquivos.
    pub fn validar(&self, existe_alvo: bool) -> Option<Recusa> {
        if caminho_escapa(&self.alvo) {
            return Some(Recusa::AlvoForaDoVault);
        }
        if matches!(
            (self.operacao, existe_alvo),
            (Operacao::Criar, true) | (Operacao::Substituir, false)
        ) {
            return Some(Recusa::EstadoMudou);
        }
        self.validar_conteudo()
    }

    /// Roda a validação semântica dos embeds do conteúdo proposto —
    /// a mesma do `anotadinho-cli embed check` (ciclo 189).
    fn validar_conteudo(&self) -> Option<Recusa> {
        let (_, corpo) = crate::MarkdownCodec::split_frontmatter_text(&self.conteudo);
        let ctx = crate::embed::ValidationCtx::default();
        for seg in crate::embed::segment(corpo) {
            let crate::embed::DocSegment::Embed(data) = seg else { continue };
            let problemas = data.validate(&ctx);
            if crate::embed::EmbedData::tem_erro(&problemas) {
                let detalhe = problemas
                    .iter()
                    .filter(|p| p.severidade == crate::embed::Severidade::Erro)
                    .map(|p| format!("{} — {}", p.onde, p.mensagem))
                    .collect::<Vec<_>>()
                    .join("; ");
                return Some(Recusa::ConteudoInvalido(detalhe));
            }
        }
        None
    }

    /// O diff entre o que está no vault e o que a proposta quer.
    ///
    /// Reusa o motor do ciclo 190 — o mesmo que a barra de conflito já
    /// usa, então a pessoa lê a mudança do agente no formato que ela já
    /// conhece.
    pub fn diff(&self, atual: &str) -> Vec<crate::diff::LinhaDiff> {
        crate::diff::diff_linhas(atual, &self.conteudo)
    }

    /// Nome do arquivo da proposta.
    pub fn arquivo(&self) -> String {
        format!("{PASTA}/{}.json", self.id)
    }
}

/// `..` ou caminho absoluto — os dois jeitos de sair do vault.
fn caminho_escapa(p: &str) -> bool {
    let p = p.trim();
    p.is_empty()
        || p.starts_with('/')
        || p.starts_with('\\')
        || p.contains("..")
        // `C:\` e afins.
        || p.chars().nth(1) == Some(':')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Proposta {
        Proposta {
            id: "p1".into(),
            autor: "falso".into(),
            quando: "2026-08-22 10:00".into(),
            motivo: "porque sim".into(),
            alvo: "pages/nova.md".into(),
            operacao: Operacao::Criar,
            conteudo: "---\ntitle: Nova\n---\ncorpo\n".into(),
        }
    }

    #[test]
    fn proposta_valida_passa() {
        assert_eq!(base().validar(false), None);
    }

    #[test]
    fn recusa_caminho_que_escapa_do_vault() {
        for alvo in ["../fora.md", "/etc/passwd", "pages/../../x.md", "", "C:\\x.md"] {
            let mut p = base();
            p.alvo = alvo.into();
            assert_eq!(
                p.validar(false),
                Some(Recusa::AlvoForaDoVault),
                "deixou passar: {alvo}"
            );
        }
    }

    #[test]
    fn recusa_criar_o_que_ja_existe() {
        // O agente decidiu com uma foto velha do vault: aplicar
        // escreveria por cima de algo que ele não viu.
        assert_eq!(base().validar(true), Some(Recusa::EstadoMudou));
    }

    #[test]
    fn recusa_substituir_o_que_nao_existe() {
        let mut p = base();
        p.operacao = Operacao::Substituir;
        assert_eq!(p.validar(false), Some(Recusa::EstadoMudou));
    }

    #[test]
    fn recusa_conteudo_com_embed_invalido() {
        // Mesma validação do `embed check` (ciclo 189): a proposta chega
        // conferida, não só bem formada.
        let mut p = base();
        p.conteudo = "---\ntitle: X\n---\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: C\n  column: Fantasma\n{{ /kanban }}\n".into();
        match p.validar(false) {
            Some(Recusa::ConteudoInvalido(d)) => assert!(d.contains("Fantasma"), "{d}"),
            outro => panic!("devia ter recusado: {outro:?}"),
        }
    }

    #[test]
    fn aceita_conteudo_com_embed_valido() {
        let mut p = base();
        p.conteudo = "---\ntitle: X\n---\n\n{{ type: \"kanban\" }}\ncolumns:\n- Backlog\nitems:\n- title: C\n  column: Backlog\n{{ /kanban }}\n".into();
        assert_eq!(p.validar(false), None);
    }

    #[test]
    fn diff_mostra_o_que_muda() {
        let mut p = base();
        p.operacao = Operacao::Substituir;
        p.conteudo = "linha um\nlinha DUAS\n".into();
        let d = p.diff("linha um\nlinha dois\n");
        let (removidas, adicionadas) = crate::diff::contar(&d);
        assert_eq!((removidas, adicionadas), (1, 1));
    }

    #[test]
    fn arquivo_fica_fora_de_pages() {
        // Senão a proposta apareceria como página e entraria em consulta.
        assert!(base().arquivo().starts_with(".anotadinho/"));
        assert!(!base().arquivo().starts_with("pages/"));
    }
}
