//! Onde o agente pode propor escrita (ciclo 405).
//!
//! A defesa do ciclo 204 é a revisão humana: o agente propõe e você
//! decide. Isto é a camada de antes — o que ele nem pode propor. Serve
//! pra dois casos concretos: pasta que é registro e não se reescreve
//! (`journals/`), e pasta que guarda decisão tomada (`decisoes/`).
//!
//! As regras vivem no VAULT (`.anotadinho/permissoes.json`), não nas
//! preferências do app: elas valem pro agente que roda pela janela, pela
//! TUI e pelo `anotadinho-cli`, e acompanham o vault quando ele viaja.
//!
//! Regra por PREFIXO de caminho, não por padrão com curinga: é o que se
//! consegue explicar numa frase ("tudo em pages/specs") e o que não tem
//! canto escuro. `nunca` ganha de `pode`, sempre.

use serde::{Deserialize, Serialize};

/// Arquivo das regras, relativo à raiz do vault.
pub const ARQUIVO: &str = ".anotadinho/permissoes.json";

/// As pastas onde o agente pode e não pode propor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Permissoes {
    /// Prefixos permitidos. Vazio é "qualquer lugar do vault".
    #[serde(default)]
    pub pode: Vec<String>,
    /// Prefixos proibidos, mesmo que estejam dentro de um permitido.
    #[serde(default)]
    pub nunca: Vec<String>,
}

impl Default for Permissoes {
    /// O padrão protege o que é registro: journal é diário, e proposta
    /// que reescreve o passado não é revisão, é reescrita da história.
    fn default() -> Self {
        Self { pode: Vec::new(), nunca: vec!["journals/".to_string()] }
    }
}

impl Permissoes {
    /// O agente pode propor nesta página?
    pub fn pode_propor(&self, alvo: &str) -> bool {
        let alvo = alvo.trim_start_matches("./");
        if alvo.starts_with(".anotadinho/") {
            return false;
        }
        if self.nunca.iter().any(|n| combina(alvo, n)) {
            return false;
        }
        self.pode.is_empty() || self.pode.iter().any(|p| combina(alvo, p))
    }

    /// Por que não pode, em uma frase.
    pub fn motivo(&self, alvo: &str) -> String {
        if self.pode_propor(alvo) {
            return String::new();
        }
        if alvo.starts_with(".anotadinho/") || self.nunca.iter().any(|n| combina(alvo, n)) {
            return format!("{alvo} está fora do alcance do agente");
        }
        format!("o agente só propõe em: {}", self.pode.join(", "))
    }
}

/// Um prefixo combina com o alvo por PASTA, não por texto: `pages/spec`
/// não pega `pages/species.md`.
fn combina(alvo: &str, prefixo: &str) -> bool {
    let p = prefixo.trim().trim_start_matches("./");
    if p.is_empty() {
        return false;
    }
    let p = p.trim_end_matches('/');
    alvo == p || alvo.starts_with(&format!("{p}/"))
}

#[cfg(test)]
mod testes {
    use super::*;

    #[test]
    fn o_padrao_protege_journals_e_o_interno() {
        let p = Permissoes::default();
        assert!(p.pode_propor("pages/alfa.md"));
        assert!(!p.pode_propor("journals/2026-09-17.md"));
        assert!(!p.pode_propor(".anotadinho/propostas/x.json"));
        assert!(p.motivo("journals/2026-09-17.md").contains("fora do alcance"));
    }

    #[test]
    fn com_lista_de_permitidos_o_resto_fica_de_fora() {
        let p = Permissoes { pode: vec!["pages/specs".into(), "pages/notas/".into()], nunca: vec!["pages/specs/fechadas".into()] };
        assert!(p.pode_propor("pages/specs/nova.md"));
        assert!(p.pode_propor("pages/notas/ideia.md"));
        assert!(!p.pode_propor("pages/outra.md"));
        assert!(!p.pode_propor("pages/specs/fechadas/velha.md"), "nunca ganha de pode");
        // Pasta não é prefixo de texto.
        assert!(!p.pode_propor("pages/species.md"));
        assert!(p.motivo("pages/outra.md").contains("só propõe em: pages/specs, pages/notas/"));
    }
}
