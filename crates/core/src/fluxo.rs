//! Protocolo de trabalho: spec → proposta → execução (ciclo 201).
//!
//! O ciclo de desenvolvimento deste repositório já funciona assim
//! (`cycles/tasks/` e `cycles/status/`), mas movido à mão, de fora do
//! app. Este módulo transforma o fluxo em DADO: uma etapa é um valor,
//! as transições são explícitas, e nenhuma avança sozinha.
//!
//! A regra que dá segurança ao acoplamento com agentes: **avançar é
//! sempre ação humana**. Um agente pode preparar o conteúdo de uma
//! etapa e pedir revisão; quem move o estado é quem lê.
//!
//! Fica no core porque a UI, o CLI e (depois) o servidor MCP precisam
//! concordar sobre o que é uma etapa válida.

use serde::{Deserialize, Serialize};

/// Onde um artefato está no fluxo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum Etapa {
    /// Sendo escrito, ninguém olhou ainda.
    #[default]
    Rascunho,
    /// Pronto pra alguém ler e decidir.
    EmRevisao,
    /// Revisado e aceito — pode virar trabalho.
    Aprovada,
    /// Alguém (ou algum agente) está executando.
    EmExecucao,
    /// Terminado.
    Concluida,
    /// Parado por um motivo externo. Sai daqui voltando pra revisão.
    Bloqueada,
}

impl Etapa {
    /// Todas, na ordem do fluxo.
    pub fn all() -> &'static [Etapa] {
        &[
            Self::Rascunho,
            Self::EmRevisao,
            Self::Aprovada,
            Self::EmExecucao,
            Self::Concluida,
            Self::Bloqueada,
        ]
    }

    /// Valor como aparece no frontmatter.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Rascunho => "rascunho",
            Self::EmRevisao => "em-revisao",
            Self::Aprovada => "aprovada",
            Self::EmExecucao => "em-execucao",
            Self::Concluida => "concluida",
            Self::Bloqueada => "bloqueada",
        }
    }

    /// Nome pra mostrar.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Rascunho => "Rascunho",
            Self::EmRevisao => "Em revisão",
            Self::Aprovada => "Aprovada",
            Self::EmExecucao => "Em execução",
            Self::Concluida => "Concluída",
            Self::Bloqueada => "Bloqueada",
        }
    }

    /// Lê do frontmatter. Aceita os valores que o vault já usava antes
    /// deste ciclo (`backlog`, `in-progress`, `done`) pra uma spec
    /// existente não ficar órfã do fluxo.
    pub fn from_slug(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        Some(match s.as_str() {
            "rascunho" | "backlog" | "draft" => Self::Rascunho,
            "em-revisao" | "em-revisão" | "in-review" | "review" => Self::EmRevisao,
            "aprovada" | "aprovado" | "approved" => Self::Aprovada,
            "em-execucao" | "em-execução" | "in-progress" | "doing" => Self::EmExecucao,
            "concluida" | "concluída" | "done" | "concluido" => Self::Concluida,
            "bloqueada" | "bloqueado" | "blocked" => Self::Bloqueada,
            _ => return None,
        })
    }

    /// Pra onde dá pra ir a partir daqui.
    ///
    /// Não é uma fila rígida: voltar pra revisão é sempre possível,
    /// porque "isto não está bom" acontece em qualquer ponto. O que NÃO
    /// existe é pular a revisão indo direto de rascunho pra execução —
    /// era esse pulo que o fluxo veio impedir.
    pub fn proximas(&self) -> Vec<Etapa> {
        match self {
            Self::Rascunho => vec![Self::EmRevisao],
            Self::EmRevisao => vec![Self::Aprovada, Self::Rascunho, Self::Bloqueada],
            Self::Aprovada => vec![Self::EmExecucao, Self::EmRevisao],
            Self::EmExecucao => vec![Self::Concluida, Self::Bloqueada, Self::EmRevisao],
            Self::Concluida => vec![Self::EmRevisao],
            Self::Bloqueada => vec![Self::EmRevisao],
        }
    }

    /// A transição é permitida?
    pub fn pode_ir_para(&self, destino: Etapa) -> bool {
        self.proximas().contains(&destino)
    }

    /// O avanço natural — o botão principal.
    pub fn avanco_natural(&self) -> Option<Etapa> {
        self.proximas().first().copied()
    }

    /// Um agente pode PREPARAR conteúdo nesta etapa?
    ///
    /// Só onde o trabalho ainda está sendo formado. Depois de aprovada,
    /// mexer no texto sem passar por revisão desfaria a aprovação em
    /// silêncio.
    pub fn agente_pode_preparar(&self) -> bool {
        matches!(self, Self::Rascunho | Self::EmRevisao)
    }

    /// Terminou?
    pub fn e_final(&self) -> bool {
        matches!(self, Self::Concluida)
    }
}

/// Tipo de artefato do fluxo — o que a página É.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Artefato {
    /// O que se quer, e por quê.
    Spec,
    /// Como pretende ser feito.
    Proposta,
    /// O registro do que foi feito.
    Execucao,
    /// Uma conversa com o agente (ciclo 202).
    Conversa,
}

impl Artefato {
    pub fn all() -> &'static [Artefato] {
        &[Self::Spec, Self::Proposta, Self::Execucao, Self::Conversa]
    }

    pub fn slug(&self) -> &'static str {
        match self {
            Self::Spec => "spec",
            Self::Proposta => "proposta",
            Self::Execucao => "execucao",
            Self::Conversa => "conversa",
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Spec => "Spec",
            Self::Proposta => "Proposta",
            Self::Execucao => "Execução",
            Self::Conversa => "Conversa",
        }
    }

    pub fn from_slug(s: &str) -> Option<Self> {
        Some(match s.trim().to_lowercase().as_str() {
            "spec" => Self::Spec,
            "proposta" => Self::Proposta,
            "execucao" | "execução" => Self::Execucao,
            "conversa" => Self::Conversa,
            _ => return None,
        })
    }

    /// Pasta onde este artefato nasce.
    pub fn pasta(&self) -> &'static str {
        match self {
            Self::Spec => "pages/specs",
            Self::Proposta => "pages/propostas",
            Self::Execucao => "pages/execucoes",
            Self::Conversa => "pages/conversas",
        }
    }

    /// O que vem depois dele no fluxo de trabalho.
    pub fn proximo_artefato(&self) -> Option<Artefato> {
        match self {
            Self::Spec => Some(Self::Proposta),
            Self::Proposta => Some(Self::Execucao),
            Self::Execucao | Self::Conversa => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slug_round_trip_em_todas_as_etapas() {
        for e in Etapa::all() {
            assert_eq!(Etapa::from_slug(e.slug()), Some(*e), "{}", e.slug());
        }
    }

    #[test]
    fn aceita_os_status_antigos_do_vault() {
        // O vault já tinha specs com `backlog`/`in-progress`/`done`
        // antes do fluxo existir — elas não podem ficar órfãs.
        assert_eq!(Etapa::from_slug("backlog"), Some(Etapa::Rascunho));
        assert_eq!(Etapa::from_slug("in-progress"), Some(Etapa::EmExecucao));
        assert_eq!(Etapa::from_slug("done"), Some(Etapa::Concluida));
    }

    #[test]
    fn nao_da_pra_pular_a_revisao() {
        // É a regra inteira do ciclo: rascunho não vira execução direto.
        assert!(!Etapa::Rascunho.pode_ir_para(Etapa::EmExecucao));
        assert!(!Etapa::Rascunho.pode_ir_para(Etapa::Aprovada));
        assert!(!Etapa::Rascunho.pode_ir_para(Etapa::Concluida));
    }

    #[test]
    fn caminho_feliz_completo() {
        let mut e = Etapa::Rascunho;
        for esperada in [Etapa::EmRevisao, Etapa::Aprovada, Etapa::EmExecucao, Etapa::Concluida] {
            let proxima = e.avanco_natural().expect("tem avanço");
            assert_eq!(proxima, esperada, "de {:?}", e);
            assert!(e.pode_ir_para(proxima));
            e = proxima;
        }
        assert!(e.e_final());
    }

    #[test]
    fn voltar_pra_revisao_vale_de_qualquer_lugar() {
        for e in Etapa::all() {
            if *e == Etapa::EmRevisao || *e == Etapa::Rascunho {
                continue;
            }
            assert!(e.pode_ir_para(Etapa::EmRevisao), "{:?} não volta pra revisão", e);
        }
    }

    #[test]
    fn agente_so_prepara_antes_da_aprovacao() {
        assert!(Etapa::Rascunho.agente_pode_preparar());
        assert!(Etapa::EmRevisao.agente_pode_preparar());
        // Depois de aprovada, mexer no texto desfaria a aprovação sem
        // ninguém perceber.
        assert!(!Etapa::Aprovada.agente_pode_preparar());
        assert!(!Etapa::EmExecucao.agente_pode_preparar());
        assert!(!Etapa::Concluida.agente_pode_preparar());
    }

    #[test]
    fn etapa_desconhecida_nao_inventa_valor() {
        assert_eq!(Etapa::from_slug("qualquer-coisa"), None);
        assert_eq!(Etapa::from_slug(""), None);
    }

    #[test]
    fn artefato_round_trip_e_pastas() {
        for a in Artefato::all() {
            assert_eq!(Artefato::from_slug(a.slug()), Some(*a));
            assert!(a.pasta().starts_with("pages/"), "{}", a.pasta());
        }
    }

    #[test]
    fn pagina_montada_nasce_em_rascunho_com_fluxo_embutido() {
        let md = montar_pagina(Artefato::Spec, "Exportar PDF", "corpo aqui", None, "2026-08-22");
        assert!(md.starts_with("---\n"), "{md}");
        assert!(md.contains("type: spec"), "{md}");
        assert!(md.contains("status: rascunho"), "{md}");
        assert!(md.contains(r#"{{ type: "fluxo" }}"#), "{md}");
        assert!(md.contains("etapa: rascunho"), "{md}");
        assert!(md.contains("corpo aqui"), "{md}");
    }

    #[test]
    fn proposta_guarda_a_origem() {
        let md = montar_pagina(
            Artefato::Proposta,
            "Como fazer",
            "plano",
            Some("pages/specs/x.md"),
            "2026-08-22",
        );
        assert!(md.contains("origem: pages/specs/x.md"), "{md}");
    }

    #[test]
    fn pagina_montada_parseia_de_volta_no_fluxo() {
        // O embed precisa voltar como Fluxo — se o wrapper sair errado,
        // a página nasce sem a máquina de estados.
        let md = montar_pagina(Artefato::Spec, "T", "corpo", None, "2026-08-22");
        let (_, corpo) = crate::MarkdownCodec::split_frontmatter_text(&md);
        let segs = crate::embed::segment(corpo);
        let tem_fluxo = segs.iter().any(|s| {
            matches!(s, crate::embed::DocSegment::Embed(crate::embed::EmbedData::Fluxo(_)))
        });
        assert!(tem_fluxo, "o embed de fluxo não foi reconhecido:\n{md}");
    }

    #[test]
    fn slug_tira_acento_e_pontuacao() {
        assert_eq!(slug_de_titulo("Exportar Nota em PDF"), "exportar-nota-em-pdf");
        assert_eq!(slug_de_titulo("Ação: análise!"), "acao-analise");
        assert_eq!(slug_de_titulo("  --  "), "sem-titulo");
        assert_eq!(slug_de_titulo(""), "sem-titulo");
    }

    #[test]
    fn titulo_sugerido_usa_a_primeira_linha_util() {
        assert_eq!(titulo_sugerido("\n\n# Um título\n\nresto", 60), "Um título");
        assert_eq!(titulo_sugerido("- item\n", 60), "item");
        assert_eq!(titulo_sugerido("", 60), "Sem título");
    }

    #[test]
    fn titulo_sugerido_corta_no_limite() {
        let longo = "a".repeat(200);
        assert_eq!(titulo_sugerido(&longo, 20).len(), 20);
    }

    #[test]
    fn cadeia_de_artefatos_termina() {
        assert_eq!(Artefato::Spec.proximo_artefato(), Some(Artefato::Proposta));
        assert_eq!(Artefato::Proposta.proximo_artefato(), Some(Artefato::Execucao));
        assert_eq!(Artefato::Execucao.proximo_artefato(), None);
        assert_eq!(Artefato::Conversa.proximo_artefato(), None);
    }
}

/// Monta a página de um artefato a partir de um texto (ciclo 203).
///
/// É a ponte entre a conversa solta e o trabalho estruturado: uma
/// resposta boa do agente vira spec ou proposta com um clique, já no
/// lugar certo, com o fluxo embutido e o rastro de onde veio.
///
/// Sem isso o fluxo morre no copiar-e-colar — que é onde a maioria das
/// integrações de chat com "criar tarefa" para.
pub fn montar_pagina(
    artefato: Artefato,
    titulo: &str,
    corpo: &str,
    origem: Option<&str>,
    hoje: &str,
) -> String {
    let mut fm = String::from("---\n");
    fm.push_str(&format!("title: {}\n", titulo.trim()));
    fm.push_str(&format!("type: {}\n", artefato.slug()));
    fm.push_str(&format!("date: {hoje}\n"));
    // `status` espelha a etapa: é o campo que as consultas filtram.
    fm.push_str(&format!("status: {}\n", Etapa::Rascunho.slug()));
    fm.push_str("tags:\n- ");
    fm.push_str(artefato.slug());
    fm.push('\n');
    fm.push_str("---\n");

    let mut corpo_final = String::new();
    corpo_final.push_str(&format!("# {}\n\n", titulo.trim()));
    corpo_final.push_str("{{ type: \"fluxo\" }}\n");
    corpo_final.push_str(&format!("artefato: {}\n", artefato.slug()));
    corpo_final.push_str(&format!("etapa: {}\n", Etapa::Rascunho.slug()));
    if let Some(o) = origem.map(str::trim).filter(|o| !o.is_empty()) {
        corpo_final.push_str(&format!("origem: {o}\n"));
    }
    corpo_final.push_str("{{ /fluxo }}\n\n");
    corpo_final.push_str(corpo.trim());
    corpo_final.push('\n');

    format!("{fm}{corpo_final}")
}

/// Nome de arquivo a partir de um título — minúsculas, sem acento, com
/// hífen no lugar de espaço.
pub fn slug_de_titulo(titulo: &str) -> String {
    let mut out = String::with_capacity(titulo.len());
    let mut ultimo_hifen = true;
    for ch in titulo.trim().chars() {
        let c = match ch {
            'á' | 'à' | 'ã' | 'â' | 'ä' => 'a',
            'é' | 'ê' | 'è' | 'ë' => 'e',
            'í' | 'î' | 'ì' | 'ï' => 'i',
            'ó' | 'õ' | 'ô' | 'ò' | 'ö' => 'o',
            'ú' | 'û' | 'ù' | 'ü' => 'u',
            'ç' => 'c',
            outro => outro,
        };
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
            ultimo_hifen = false;
        } else if !ultimo_hifen {
            out.push('-');
            ultimo_hifen = true;
        }
    }
    let s = out.trim_matches('-').to_string();
    if s.is_empty() { "sem-titulo".to_string() } else { s }
}

/// Primeira linha útil de um texto, pra sugerir o título.
pub fn titulo_sugerido(texto: &str, limite: usize) -> String {
    let linha = texto
        .lines()
        .map(str::trim)
        .find(|l| !l.is_empty())
        .unwrap_or("")
        .trim_start_matches(['#', '-', '*', '>', ' ']);
    let corte: String = linha.chars().take(limite).collect();
    if corte.trim().is_empty() {
        "Sem título".to_string()
    } else {
        corte.trim().to_string()
    }
}
