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
    /// Como ler a saída (ciclo 213). Ausente em configuração antiga,
    /// onde `Texto` é o comportamento de sempre.
    #[serde(default)]
    pub formato: FormatoSaida,
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

/// Timeout padrão de uma execução, em segundos (30 minutos).
///
/// Era 180s (3 min), e isso é curto demais pro trabalho real: pedir
/// uma proposta de implementação faz o modelo ler spec, padrões e
/// código antes de escrever, e passa fácil de 3 minutos. O usuário
/// mandava a pergunta, o processo era morto no meio e a conversa
/// ficava sem resposta e sem explicação.
///
/// O limite existe pra processo travado não ficar pendurado pra
/// sempre, não pra apressar o modelo — quem quer parar antes usa o
/// botão de interromper.
pub const TIMEOUT_PADRAO_S: u64 = 1800;

/// Como interpretar a saída do agente.
///
/// Existe por causa do feedback durante execuções longas. Com o timeout
/// em 30 minutos, uma tela sem retorno vira um problema: `claude -p`
/// segura TUDO até terminar, então não há nada pra mostrar enquanto ele
/// pensa — meia hora de silêncio é indistinguível de um app travado.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FormatoSaida {
    /// A saída inteira é a resposta. Serve pra qualquer agente.
    #[default]
    Texto,
    /// Uma linha = um evento JSON, transmitido conforme acontece.
    ///
    /// Segue o esquema do Claude Code
    /// (`--output-format stream-json --verbose`): eventos `assistant`
    /// trazem o texto e as ferramentas usadas, e o `result` final traz
    /// a resposta. Se o `result` não vier, o texto acumulado dos
    /// eventos serve de resposta — melhor entregar algo do que falhar
    /// por causa do formato.
    StreamJson,
}

/// Piso do timeout, em segundos (30 minutos).
///
/// Aplicado na LEITURA da configuração, não na escrita: um app que
/// já rodou antes tem `timeout_s: 180` gravado, e sem o piso essa
/// configuração velha continuaria matando o agente aos 3 minutos
/// mesmo depois da correção.
pub const TIMEOUT_MINIMO_S: u64 = 1800;

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

    /// Devolve uma cópia com o timeout elevado ao piso, se estiver
    /// abaixo dele.
    ///
    /// Chamado na LEITURA da configuração. Sem isso, quem já usou o app
    /// antes continua com `timeout_s: 180` gravado no navegador e o
    /// agente segue morrendo aos 3 minutos, por mais que o padrão novo
    /// diga outra coisa.
    pub fn com_piso_de_timeout(mut self) -> Self {
        if self.timeout_s < TIMEOUT_MINIMO_S {
            self.timeout_s = TIMEOUT_MINIMO_S;
        }
        self
    }

    /// Atualiza uma configuração gravada por uma versão anterior.
    ///
    /// Preferência do usuário é sagrada, então isto só age sobre um
    /// valor que RECONHECE como padrão antigo — nunca sobre args que a
    /// pessoa montou. Sem isso, quem já usou o app fica preso na
    /// configuração velha e não vê nem o timeout novo nem o progresso
    /// em tempo real, sem ter como saber por quê.
    pub fn migrado(self) -> Self {
        const ARGS_ANTIGOS_CLAUDE: [&str; 2] = ["-p", MARCADOR_PROMPT];
        let e_o_preset_antigo = self.nome == "Claude Code"
            && self.args.len() == ARGS_ANTIGOS_CLAUDE.len()
            && self.args.iter().zip(ARGS_ANTIGOS_CLAUDE).all(|(a, b)| a == b);
        let atualizado = if e_o_preset_antigo {
            let preset = Self::presets().remove(0);
            Self { args: preset.args, formato: preset.formato, ..self }
        } else {
            self
        };
        atualizado.com_piso_de_timeout()
    }

    /// Perfis prontos dos agentes mais comuns. São só um ponto de
    /// partida editável — o contrato é o `{prompt}`, não a lista.
    pub fn presets() -> Vec<Adaptador> {
        vec![
            Adaptador {
                nome: "Claude Code".into(),
                binario: "claude".into(),
                args: vec![
                    "-p".into(),
                    "--output-format".into(),
                    "stream-json".into(),
                    "--verbose".into(),
                    MARCADOR_PROMPT.into(),
                ],
                cwd: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
            },
            Adaptador {
                nome: "Codex".into(),
                binario: "codex".into(),
                args: vec!["exec".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::Texto,
            },
            Adaptador {
                nome: "opencode".into(),
                binario: "opencode".into(),
                args: vec!["run".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::Texto,
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
            formato: FormatoSaida::Texto,
        }
    }

    #[test]
    fn o_piso_levanta_um_timeout_velho_e_nao_mexe_num_maior() {
        let velho = Adaptador { timeout_s: 180, ..base() };
        assert_eq!(velho.com_piso_de_timeout().timeout_s, TIMEOUT_MINIMO_S);

        let generoso = Adaptador { timeout_s: 7200, ..base() };
        assert_eq!(generoso.com_piso_de_timeout().timeout_s, 7200);
    }

    #[test]
    fn nenhum_preset_nasce_abaixo_do_piso() {
        for p in Adaptador::presets() {
            assert!(
                p.timeout_s >= TIMEOUT_MINIMO_S,
                "preset \"{}\" nasce com timeout de {}s",
                p.nome,
                p.timeout_s
            );
        }
    }

    #[test]
    fn o_stream_separa_progresso_de_resposta() {
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"system","subtype":"init"}"#);
        l.linha(
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Read"}]}}"#,
        );
        l.linha(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"Vou ler a spec"}]}}"#,
        );
        l.linha(r#"{"type":"result","is_error":false,"result":"Proposta pronta"}"#);

        let p = l.progresso();
        assert!(p.contains("usando Read"), "progresso sem a ferramenta: {p}");
        assert!(p.contains("Vou ler a spec"), "progresso sem o texto: {p}");
        // O progresso é ruído de execução: não pode virar a resposta.
        assert_eq!(l.resposta().unwrap(), "Proposta pronta");
    }

    #[test]
    fn o_stream_reporta_o_erro_do_agente() {
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"result","is_error":true,"result":"estourou o limite"}"#);
        assert_eq!(l.resposta().unwrap_err(), "estourou o limite");
    }

    #[test]
    fn sem_evento_result_cai_no_texto_acumulado() {
        // Agente que fala outro dialeto: entregar o que deu é melhor do
        // que falhar por causa do formato.
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"assistant","message":{"content":[{"type":"text","text":"resposta solta"}]}}"#);
        assert_eq!(l.resposta().unwrap(), "resposta solta");
    }

    #[test]
    fn linha_que_nao_e_json_nao_derruba_a_leitura() {
        let mut l = LeitorStream::novo();
        l.linha("aviso solto do binário");
        l.linha("");
        l.linha(r#"{"type":"result","is_error":false,"result":"ok"}"#);
        assert_eq!(l.resposta().unwrap(), "ok");
    }

    #[test]
    fn o_progresso_nao_cresce_sem_limite() {
        let mut l = LeitorStream::novo();
        for i in 0..200 {
            l.linha(&format!(
                r#"{{"type":"assistant","message":{{"content":[{{"type":"tool_use","name":"T{i}"}}]}}}}"#
            ));
        }
        assert_eq!(l.progresso().lines().count(), LINHAS_DE_PROGRESSO);
    }

    #[test]
    fn a_migracao_atualiza_o_preset_antigo_do_claude() {
        let velho = Adaptador {
            nome: "Claude Code".into(),
            binario: "/home/x/.local/bin/claude".into(),
            args: vec!["-p".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            timeout_s: 180,
            formato: FormatoSaida::Texto,
        };
        let novo = velho.migrado();
        assert_eq!(novo.formato, FormatoSaida::StreamJson);
        assert!(novo.args.contains(&"stream-json".to_string()));
        assert_eq!(novo.timeout_s, TIMEOUT_MINIMO_S);
        // O binário que a pessoa apontou continua sendo o dela.
        assert_eq!(novo.binario, "/home/x/.local/bin/claude");
    }

    #[test]
    fn a_migracao_nao_mexe_em_args_montados_a_mao() {
        let meu = Adaptador {
            nome: "Claude Code".into(),
            binario: "claude".into(),
            args: vec!["--dangerously-skip".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            timeout_s: 180,
            formato: FormatoSaida::Texto,
        };
        let depois = meu.clone().migrado();
        assert_eq!(depois.args, meu.args, "mexeu em configuração do usuário");
        assert_eq!(depois.formato, FormatoSaida::Texto);
        // O piso do timeout vale pra todo mundo.
        assert_eq!(depois.timeout_s, TIMEOUT_MINIMO_S);
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

/// Estado de uma execução do agente, do ponto de vista de quem espera.
///
/// Existe porque a execução deixou de ser uma chamada que bloqueia até
/// responder. Antes, a requisição vivia dentro do componente da
/// conversa: sair da página desmontava o componente, os handles de
/// estado morriam e a resposta se perdia — a pessoa não sabia se o
/// modelo ainda estava pensando ou se tinha falhado calado.
///
/// Agora quem guarda isso é o backend, e a tela só pergunta "como está
/// o trabalho desta conversa?". Voltar pra página recupera o estado,
/// inclusive uma resposta que chegou enquanto ela estava noutro lugar.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "estado", rename_all = "snake_case")]
pub enum EstadoJob {
    /// Em andamento. `parcial` é o que o agente já escreveu na saída —
    /// é o que dá sinal de vida durante uma execução longa.
    Rodando { segundos: u64, parcial: String },
    /// Terminou bem. `texto` é a resposta inteira.
    Concluido { texto: String },
    /// Terminou mal: binário que não existe, erro do agente, ou o
    /// timeout estourou.
    Falhou { erro: String },
    /// Interrompido por quem pediu.
    Cancelado,
}

impl EstadoJob {
    /// Se ainda há trabalho acontecendo.
    pub fn em_andamento(&self) -> bool {
        matches!(self, EstadoJob::Rodando { .. })
    }
}

/// Acumula os eventos de um agente que fala em `stream-json`.
///
/// Duas saídas ao mesmo tempo, e elas servem a públicos diferentes:
/// `progresso()` é pra pessoa ver que algo está acontecendo agora
/// ("lendo spec.md", "escrevendo..."), e `resposta()` é o texto final
/// que vai pra conversa. Misturar os dois deixaria a conversa cheia de
/// ruído de execução.
#[derive(Debug, Default)]
pub struct LeitorStream {
    progresso: Vec<String>,
    texto: String,
    resultado: Option<String>,
    erro: Option<String>,
}

impl LeitorStream {
    pub fn novo() -> Self {
        Self::default()
    }

    /// Consome uma linha do stream. Linha que não é JSON é ignorada —
    /// um agente pode escrever aviso solto na saída, e isso não pode
    /// derrubar a execução inteira.
    pub fn linha(&mut self, linha: &str) {
        let linha = linha.trim();
        if linha.is_empty() {
            return;
        }
        let Ok(v) = serde_json::from_str::<serde_json::Value>(linha) else {
            return;
        };
        match v.get("type").and_then(|t| t.as_str()) {
            Some("system") => {
                if v.get("subtype").and_then(|s| s.as_str()) == Some("init") {
                    self.progresso.push("conectado".to_string());
                }
            }
            Some("assistant") => {
                let blocos = v
                    .get("message")
                    .and_then(|m| m.get("content"))
                    .and_then(|c| c.as_array());
                for b in blocos.into_iter().flatten() {
                    match b.get("type").and_then(|t| t.as_str()) {
                        Some("text") => {
                            if let Some(txt) = b.get("text").and_then(|t| t.as_str()) {
                                self.texto.push_str(txt);
                                // Só a primeira linha vira progresso: o
                                // texto inteiro aparece no fim, e
                                // repeti-lo aqui encheria o painel.
                                if let Some(primeira) = txt.lines().find(|l| !l.trim().is_empty()) {
                                    self.progresso.push(resumir(primeira));
                                }
                            }
                        }
                        Some("tool_use") => {
                            if let Some(nome) = b.get("name").and_then(|n| n.as_str()) {
                                self.progresso.push(format!("usando {nome}"));
                            }
                        }
                        _ => {}
                    }
                }
            }
            Some("result") => {
                if v.get("is_error").and_then(|e| e.as_bool()) == Some(true) {
                    self.erro = Some(
                        v.get("result")
                            .and_then(|r| r.as_str())
                            .unwrap_or("o agente terminou com erro")
                            .to_string(),
                    );
                } else if let Some(r) = v.get("result").and_then(|r| r.as_str()) {
                    self.resultado = Some(r.to_string());
                }
            }
            _ => {}
        }
    }

    /// As últimas linhas de progresso, pra mostrar enquanto roda.
    pub fn progresso(&self) -> String {
        let n = self.progresso.len().saturating_sub(LINHAS_DE_PROGRESSO);
        self.progresso[n..].join("\n")
    }

    /// A resposta final, ou o erro que o agente reportou.
    ///
    /// Sem evento `result` (agente que fala outro dialeto), cai no
    /// texto acumulado: entregar o que deu é melhor do que falhar por
    /// causa do formato.
    pub fn resposta(&self) -> Result<String, String> {
        if let Some(e) = &self.erro {
            return Err(e.clone());
        }
        let texto = self
            .resultado
            .clone()
            .unwrap_or_else(|| self.texto.clone())
            .trim()
            .to_string();
        if texto.is_empty() {
            Err("o agente terminou sem escrever nada na saída".to_string())
        } else {
            Ok(texto)
        }
    }
}

/// Quantas linhas de progresso ficam visíveis.
const LINHAS_DE_PROGRESSO: usize = 12;

/// Encurta uma linha pra caber no painel de progresso.
fn resumir(linha: &str) -> String {
    const LIMITE: usize = 110;
    let l = linha.trim();
    if l.chars().count() <= LIMITE {
        return l.to_string();
    }
    let cortado: String = l.chars().take(LIMITE).collect();
    format!("{cortado}…")
}
