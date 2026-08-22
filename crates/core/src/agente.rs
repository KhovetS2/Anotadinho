//! Configuração do agente externo (ciclo 202).
//!
//! O Anotadinho não embute modelo nenhum: ele INVOCA o que você já usa —
//! `claude`, `codex`, `opencode` ou qualquer outro CLI que aceite um
//! prompt e escreva a resposta na saída padrão.
//!
//! O piso do contrato é deliberadamente baixo: **um disparo por vez**.
//! Qualquer CLI atende. Sessão persistente fica pra depois, como
//! capacidade opcional — quem não tiver continua funcionando.
//!
//! # A invariante de segurança
//!
//! O comando NUNCA vem do conteúdo de uma página. Ele vem daqui, de uma
//! configuração que a pessoa escreveu nas preferências do app. Uma
//! página `.md` que chegue de terceiro pode, no máximo, pedir que você
//! aprove algo — e você lê o quê antes. É a mesma regra que mantém a
//! lista de ações do embed `actions` fechada.

use serde::{Deserialize, Serialize};

/// Marcador substituído pelo prompt na linha de comando.
pub const MARCADOR_PROMPT: &str = "{prompt}";

/// Um agente que dá pra chamar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Adaptador {
    /// Nome que aparece na UI.
    pub nome: String,
    /// Executável. Sem shell no meio: nada de `sh -c`, pra não abrir
    /// espaço pra injeção pelo texto do prompt.
    pub binario: String,
    /// Argumentos. Exatamente um deles deve conter `{prompt}`.
    pub args: Vec<String>,
    /// Diretório de trabalho. Vazio = raiz do vault.
    #[serde(default)]
    pub cwd: String,
    /// Segundos até desistir. 0 = sem limite (desaconselhado).
    #[serde(default = "timeout_padrao")]
    pub timeout_s: u64,
}

fn timeout_padrao() -> u64 {
    120
}

/// Por que uma configuração não serve.
#[derive(Debug, Clone, PartialEq)]
pub enum ProblemaConfig {
    SemBinario,
    SemMarcador,
    MarcadorRepetido,
    BinarioComEspaco,
}

impl ProblemaConfig {
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::SemBinario => "informe o executável do agente",
            Self::SemMarcador => "um dos argumentos precisa conter {prompt}",
            Self::MarcadorRepetido => "{prompt} pode aparecer em um argumento só",
            Self::BinarioComEspaco => {
                "o executável não pode ter espaço: use um argumento separado, não uma linha de shell"
            }
        }
    }
}

impl Adaptador {
    /// Confere a configuração ANTES de qualquer execução.
    pub fn validar(&self) -> Option<ProblemaConfig> {
        if self.binario.trim().is_empty() {
            return Some(ProblemaConfig::SemBinario);
        }
        // Espaço no binário quase sempre significa que a pessoa colou
        // uma linha de shell inteira. Aceitar isso viraria execução de
        // shell pela porta dos fundos.
        if self.binario.trim().contains(' ') {
            return Some(ProblemaConfig::BinarioComEspaco);
        }
        let com_marcador = self.args.iter().filter(|a| a.contains(MARCADOR_PROMPT)).count();
        match com_marcador {
            0 => Some(ProblemaConfig::SemMarcador),
            1 => None,
            _ => Some(ProblemaConfig::MarcadorRepetido),
        }
    }

    /// Monta os argumentos finais com o prompt no lugar do marcador.
    ///
    /// O prompt entra como UM argumento, nunca concatenado numa string
    /// de shell — então aspas, quebras de linha e `$(...)` dentro dele
    /// são texto, não código.
    pub fn montar_args(&self, prompt: &str) -> Vec<String> {
        self.args
            .iter()
            .map(|a| a.replace(MARCADOR_PROMPT, prompt))
            .collect()
    }

    /// Perfis prontos dos agentes mais comuns. São só um ponto de
    /// partida editável — o contrato é o `{prompt}`, não a lista.
    pub fn presets() -> Vec<Adaptador> {
        vec![
            Adaptador {
                nome: "Claude Code".into(),
                binario: "claude".into(),
                args: vec!["-p".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                timeout_s: 180,
            },
            Adaptador {
                nome: "Codex".into(),
                binario: "codex".into(),
                args: vec!["exec".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                timeout_s: 180,
            },
            Adaptador {
                nome: "opencode".into(),
                binario: "opencode".into(),
                args: vec!["run".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                timeout_s: 180,
            },
        ]
    }
}

impl Default for Adaptador {
    fn default() -> Self {
        Self::presets().remove(0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base() -> Adaptador {
        Adaptador {
            nome: "teste".into(),
            binario: "eco".into(),
            args: vec!["-p".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            timeout_s: 30,
        }
    }

    #[test]
    fn todos_os_presets_sao_validos() {
        for p in Adaptador::presets() {
            assert_eq!(p.validar(), None, "preset inválido: {}", p.nome);
        }
    }

    #[test]
    fn recusa_configuracao_sem_marcador() {
        let mut a = base();
        a.args = vec!["-p".into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::SemMarcador));
    }

    #[test]
    fn recusa_marcador_repetido() {
        let mut a = base();
        a.args = vec![MARCADOR_PROMPT.into(), MARCADOR_PROMPT.into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::MarcadorRepetido));
    }

    #[test]
    fn recusa_linha_de_shell_no_binario() {
        // Aceitar isso seria execução de shell pela porta dos fundos.
        let mut a = base();
        a.binario = "sh -c".into();
        assert_eq!(a.validar(), Some(ProblemaConfig::BinarioComEspaco));
    }

    #[test]
    fn recusa_binario_vazio() {
        let mut a = base();
        a.binario = "   ".into();
        assert_eq!(a.validar(), Some(ProblemaConfig::SemBinario));
    }

    #[test]
    fn prompt_entra_como_um_argumento_so() {
        let a = base();
        let args = a.montar_args("uma pergunta\ncom quebra e $(rm -rf /) dentro");
        assert_eq!(args.len(), 2);
        assert_eq!(args[0], "-p");
        // O texto perigoso vira ARGUMENTO, não comando — não há shell no
        // caminho pra interpretá-lo.
        assert!(args[1].contains("$(rm -rf /)"));
        assert!(args[1].contains('\n'));
    }

    #[test]
    fn substitui_o_marcador_no_meio_do_argumento() {
        let mut a = base();
        a.args = vec![format!("--msg={MARCADOR_PROMPT}")];
        assert_eq!(a.montar_args("oi"), vec!["--msg=oi".to_string()]);
    }

    #[test]
    fn adaptador_padrao_e_valido() {
        assert_eq!(Adaptador::default().validar(), None);
    }
}
