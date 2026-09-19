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
    /// Mover ou renomear uma página (ciclo 438).
    ///
    /// O `alvo` é o DESTINO e a `origem` diz de onde ela sai. Existe
    /// porque refatorar um vault — renomear um conceito, reorganizar uma
    /// pasta — era impossível pelo agente: `propor` só trocava conteúdo,
    /// e criar no lugar novo deixava a página velha para trás.
    Mover,
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
    /// De onde a página sai, numa proposta de mover (ciclo 438).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub origem: Option<String>,
    /// O que o revisor achou dela (ciclo 436), quando alguém pediu uma
    /// revisão. Ausente = ninguém revisou.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub revisao: Option<Revisao>,
    /// O lote a que ela pertence (ciclo 420). Propostas do mesmo lote são
    /// UMA decisão: aplicam juntas ou não aplicam.
    ///
    /// Uma mudança que atravessa páginas — renomear um conceito em
    /// quatro specs — só faz sentido inteira. Sem isto, aprovar três de
    /// quatro deixava o vault num estado que ninguém propôs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lote: Option<String>,
}

/// As propostas agrupadas por lote, na ordem em que aparecem (ciclo 420).
///
/// Sem lote, cada proposta é o próprio grupo — é o que faz a tela de
/// aprovações tratar as duas formas pelo mesmo caminho.
pub fn por_lote(propostas: &[Proposta]) -> Vec<(Option<String>, Vec<&Proposta>)> {
    let mut fora: Vec<(Option<String>, Vec<&Proposta>)> = Vec::new();
    for p in propostas {
        match &p.lote {
            Some(lote) => match fora.iter_mut().find(|(l, _)| l.as_deref() == Some(lote.as_str())) {
                Some((_, grupo)) => grupo.push(p),
                None => fora.push((Some(lote.clone()), vec![p])),
            },
            None => fora.push((None, vec![p])),
        }
    }
    fora
}

/// O veredito de um segundo agente sobre a proposta (ciclo 436).
///
/// Trabalho sem supervisão troca "confio no modelo" por dois olhares: um
/// agente escreve, outro critica, e você lê a crítica junto do diff. O
/// revisor NÃO decide — ele não aplica nem recusa nada; só diz o que
/// viu, e a aprovação continua sendo humana.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Veredito {
    /// Não achou problema.
    Aprova,
    /// Aplicável, mas com algo a observar.
    Ressalva,
    /// Achou problema que desaconselha aplicar.
    Recusa,
}

impl Veredito {
    pub fn rotulo(&self) -> &'static str {
        match self {
            Self::Aprova => "aprova",
            Self::Ressalva => "ressalva",
            Self::Recusa => "recusa",
        }
    }

    /// Lê o veredito da resposta do revisor.
    ///
    /// Procura a PALAVRA no começo da resposta, que é onde o prompt pede
    /// que ela esteja. Sem palavra reconhecível, vale `Ressalva`: um
    /// revisor confuso não pode virar aprovação.
    pub fn da_resposta(texto: &str) -> Self {
        let inicio: String = texto
            .trim_start()
            .chars()
            .take(40)
            .collect::<String>()
            .to_lowercase();
        if inicio.starts_with("aprova") {
            Self::Aprova
        } else if inicio.starts_with("recusa") {
            Self::Recusa
        } else {
            Self::Ressalva
        }
    }
}

/// A revisão registrada numa proposta.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revisao {
    /// `"AAAA-MM-DD HH:MM"`.
    pub quando: String,
    /// Quem revisou (nome do adaptador).
    pub agente: String,
    pub veredito: Veredito,
    /// O que ele escreveu, inteiro — é o que a pessoa lê junto do diff.
    pub notas: String,
}

/// O prompt do revisor (ciclo 436).
///
/// Pede a palavra do veredito na PRIMEIRA linha e o resto em prosa: é o
/// que permite ler a decisão sem adivinhação e ainda ter o argumento.
/// O conteúdo revisado entra como DADO, com a mesma blindagem do resto
/// (ciclo 202): a proposta pode ter vindo de fora.
pub fn prompt_de_revisao(p: &Proposta, atual: &str) -> String {
    let diff = crate::diff::diff_linhas(atual, &p.conteudo);
    let corpo: String = diff
        .iter()
        .map(|l| match l {
            crate::diff::LinhaDiff::Igual { texto } => format!(" {texto}\n"),
            crate::diff::LinhaDiff::Removida { texto } => format!("-{texto}\n"),
            crate::diff::LinhaDiff::Adicionada { texto } => format!("+{texto}\n"),
        })
        .collect();
    let motivo = if p.motivo.trim().is_empty() { "(sem motivo declarado)" } else { p.motivo.trim() };
    format!(
        "# Revisão de uma proposta\n\n         Outro agente propôs mudar `{}` dizendo: {motivo}\n\n         Você NÃO aplica nem recusa nada — só revisa. Responda com UMA palavra na primeira          linha: APROVA, RESSALVA ou RECUSA. Depois, em poucas linhas, o porquê: o que quebra,          o que contradiz o resto do vault, o que ficou pela metade.\n\n{}",
        p.alvo,
        crate::conversa::blindar_dado(&format!("DIFF {}", p.alvo), &corpo)
    )
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
    /// Mover sem dizer de onde, ou de um lugar que não existe mais.
    OrigemInvalida(String),
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
            Self::OrigemInvalida(d) => format!("não dá pra mover: {d}"),
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
        self.validar_com_origem(existe_alvo, self.operacao != Operacao::Mover)
    }

    /// Como `validar`, dizendo também se a ORIGEM existe (ciclo 438).
    ///
    /// Mover é a única operação que precisa de dois estados do disco, e
    /// quem tem disco é quem chama — aqui continua tudo puro.
    pub fn validar_com_origem(&self, existe_alvo: bool, existe_origem: bool) -> Option<Recusa> {
        if caminho_escapa(&self.alvo) {
            return Some(Recusa::AlvoForaDoVault);
        }
        if self.operacao == Operacao::Mover {
            let Some(origem) = self.origem.as_deref().filter(|o| !o.trim().is_empty()) else {
                return Some(Recusa::OrigemInvalida("a proposta não diz de onde".into()));
            };
            if caminho_escapa(origem) {
                return Some(Recusa::AlvoForaDoVault);
            }
            if origem == self.alvo {
                return Some(Recusa::OrigemInvalida("origem e destino são a mesma página".into()));
            }
            if !existe_origem {
                return Some(Recusa::OrigemInvalida(format!("{origem} não existe mais")));
            }
            // Mover PRA CIMA de uma página existente apagaria a outra
            // sem ninguém ter proposto isso.
            if existe_alvo {
                return Some(Recusa::EstadoMudou);
            }
            return self.validar_conteudo();
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
            origem: None,
            revisao: None,
            lote: None,
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

    // --- Ciclo 420: lote ------------------------------------------------------------

    #[test]
    fn o_lote_junta_as_propostas_e_o_resto_fica_sozinho() {
        let mut a = base();
        a.id = "a".into();
        a.lote = Some("renomear".into());
        let mut b = base();
        b.id = "b".into();
        b.lote = Some("renomear".into());
        let mut sozinha = base();
        sozinha.id = "c".into();
        let lista = vec![a.clone(), sozinha.clone(), b.clone()];
        let grupos = por_lote(&lista);
        assert_eq!(grupos.len(), 2);
        assert_eq!(grupos[0].0.as_deref(), Some("renomear"));
        assert_eq!(grupos[0].1.len(), 2, "as duas do lote entram juntas, mesmo separadas na lista");
        assert_eq!(grupos[1].0, None);
        assert_eq!(grupos[1].1[0].id, "c");
    }

    #[test]
    fn proposta_sem_lote_continua_lendo_do_json_antigo() {
        let antigo = r#"{"id":"p1","autor":"cli","quando":"2026-09-17 10:00","alvo":"pages/a.md","operacao":"substituir","conteudo":"x"}"#;
        let p: Proposta = serde_json::from_str(antigo).expect("json de antes do 420");
        assert_eq!(p.lote, None);
        // E quem não tem lote não ganha campo no arquivo.
        assert!(!serde_json::to_string(&p).unwrap().contains("lote"));
    }

    // --- Ciclo 436: revisor ----------------------------------------------------------

    #[test]
    fn o_veredito_sai_da_primeira_palavra_e_o_resto_vira_ressalva() {
        assert_eq!(Veredito::da_resposta("APROVA\n\nnada a dizer"), Veredito::Aprova);
        assert_eq!(Veredito::da_resposta("  recusa: quebra o kanban"), Veredito::Recusa);
        assert_eq!(Veredito::da_resposta("RESSALVA — falta a data"), Veredito::Ressalva);
        // Revisor confuso não vira aprovação.
        assert_eq!(Veredito::da_resposta("acho que talvez esteja ok"), Veredito::Ressalva);
        assert_eq!(Veredito::da_resposta(""), Veredito::Ressalva);
    }

    #[test]
    fn o_prompt_do_revisor_leva_o_diff_blindado_e_pede_a_palavra() {
        let mut p = base();
        p.operacao = Operacao::Substituir;
        p.alvo = "pages/spec.md".into();
        p.motivo = "renomear o conceito".into();
        p.conteudo = "linha nova\n".into();
        let prompt = prompt_de_revisao(&p, "linha velha\n");
        assert!(prompt.contains("APROVA, RESSALVA ou RECUSA"), "{prompt}");
        assert!(prompt.contains("renomear o conceito"), "{prompt}");
        // O diff vai como DADO, com as duas linhas marcadas.
        assert!(prompt.contains("DADO-ANOTADINHO DIFF pages/spec.md"), "{prompt}");
        assert!(prompt.contains("-linha velha") && prompt.contains("+linha nova"), "{prompt}");
        // E diz que o revisor não decide.
        assert!(prompt.contains("NÃO aplica nem recusa"), "{prompt}");
    }

    #[test]
    fn proposta_sem_revisao_nao_ganha_campo_no_arquivo() {
        let p = base();
        let json = serde_json::to_string(&p).unwrap();
        assert!(!json.contains("revisao"), "{json}");
        let lida: Proposta = serde_json::from_str(&json).unwrap();
        assert_eq!(lida.revisao, None);
    }

    // --- Ciclo 438: mover como proposta ----------------------------------------------

    fn mudanca(origem: &str, destino: &str) -> Proposta {
        Proposta {
            operacao: Operacao::Mover,
            alvo: destino.into(),
            origem: Some(origem.into()),
            conteudo: "---\ntitle: X\n---\ncorpo\n".into(),
            ..base()
        }
    }

    #[test]
    fn mover_exige_origem_que_existe_e_destino_livre() {
        let p = mudanca("pages/velha.md", "pages/nova.md");
        assert_eq!(p.validar_com_origem(false, true), None, "o caso bom passa");
        // Destino ocupado apagaria a outra página sem ninguém propor.
        assert_eq!(p.validar_com_origem(true, true), Some(Recusa::EstadoMudou));
        // Origem sumiu no meio do caminho.
        let r = p.validar_com_origem(false, false);
        assert!(matches!(&r, Some(Recusa::OrigemInvalida(m)) if m.contains("pages/velha.md")), "{r:?}");
    }

    #[test]
    fn mover_sem_origem_ou_pra_si_mesma_e_recusado() {
        let mut sem = mudanca("pages/velha.md", "pages/nova.md");
        sem.origem = None;
        assert!(matches!(sem.validar_com_origem(false, true), Some(Recusa::OrigemInvalida(_))));
        let mesma = mudanca("pages/a.md", "pages/a.md");
        assert!(matches!(mesma.validar_com_origem(false, true), Some(Recusa::OrigemInvalida(_))));
        // E a fuga do vault continua barrada pelos dois lados.
        let fuga = mudanca("../fora.md", "pages/nova.md");
        assert_eq!(fuga.validar_com_origem(false, true), Some(Recusa::AlvoForaDoVault));
    }

    #[test]
    fn as_outras_operacoes_nao_mudaram() {
        let mut criar = base();
        criar.operacao = Operacao::Criar;
        assert_eq!(criar.validar(false), None);
        assert_eq!(criar.validar(true), Some(Recusa::EstadoMudou));
        // E proposta sem `origem` não ganha o campo no arquivo.
        assert!(!serde_json::to_string(&criar).unwrap().contains("origem"));
    }
}
