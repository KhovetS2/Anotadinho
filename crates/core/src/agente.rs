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
    /// Diretório de trabalho. Vazio = o PROJETO que contém o vault
    /// (ver `raiz_do_projeto`), não o vault.
    ///
    /// Rodar no vault deixava o agente sem enxergar o código: pedir a
    /// execução de uma proposta que mexe em `ui/` e `crates/` dava
    /// "acesso negado", porque a sessão dele só alcançava as notas. E é
    /// o pior dos dois mundos: sem acesso ao que precisa mudar, e com
    /// acesso de escrita justamente às notas, que é o que o fluxo de
    /// propostas existe pra proteger.
    #[serde(default)]
    pub cwd: String,
    /// Outras pastas que o agente pode alcançar (ciclo 216).
    ///
    /// O caso real: o vault mora num lugar e os repositórios em que se
    /// trabalha moram noutro, às vezes vários. Sem isto, escolher a
    /// pasta de trabalho seria escolher UM repositório e perder os
    /// outros.
    #[serde(default)]
    pub pastas_extras: Vec<String>,
    /// Argumento que o agente usa pra receber pasta extra (`--add-dir`).
    /// Vazio = ele não sabe receber, e `pastas_extras` é ignorado.
    #[serde(default)]
    pub arg_pasta_extra: String,
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
}

/// Uma ressalva sobre a configuração. Não impede nada (ciclo 241).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AvisoConfig {
    /// O executável parece uma linha de comando inteira.
    ParecComandoDeShell,
}

impl AvisoConfig {
    pub fn mensagem(&self) -> &'static str {
        "isto parece uma linha de comando. Aqui vai só o executável, e o resto \
         em argumentos separados — não existe shell no meio, então uma linha \
         inteira não vai iniciar"
    }
}

impl ProblemaConfig {
    pub fn mensagem(&self) -> &'static str {
        match self {
            Self::SemBinario => "informe o executável do agente",
            Self::SemMarcador => "um dos argumentos precisa conter {prompt}",
            Self::MarcadorRepetido => "{prompt} pode aparecer em um argumento só",
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
    /// Entende os dois dialetos que os agentes suportados falam, e eles
    /// não se confundem porque usam nomes de evento disjuntos:
    ///
    /// - **Claude Code** (`--output-format stream-json --verbose`):
    ///   eventos `assistant` com o texto e as ferramentas, `result`
    ///   final com a resposta.
    /// - **Codex** (`exec --json`): `item.completed` com
    ///   `agent_message` (texto) ou `command_execution` (comando
    ///   rodado); a resposta é o ÚLTIMO `agent_message`.
    ///
    /// Sem nenhum evento reconhecido, o texto acumulado serve de
    /// resposta — melhor entregar algo do que falhar pelo formato.
    StreamJson,
}

/// Piso do timeout, em segundos (30 minutos).
///
/// Aplicado na LEITURA da configuração, não na escrita: um app que
/// já rodou antes tem `timeout_s: 180` gravado, e sem o piso essa
/// configuração velha continuaria matando o agente aos 3 minutos
/// mesmo depois da correção.
pub const TIMEOUT_MINIMO_S: u64 = 1800;

/// O texto parece uma linha de comando em vez de um executável?
///
/// Um caminho, mesmo com espaço, não tem operador de shell nem token que
/// comece com `-`. Uma linha colada tem quase sempre um dos dois.
fn parece_comando_de_shell(texto: &str) -> bool {
    const OPERADORES: &[char] = &['|', '&', ';', '>', '<', '$', '`', '\n'];
    if texto.contains(OPERADORES) {
        return true;
    }
    // `claude -p` → linha. `/opt/My Tools/claude` → caminho.
    texto
        .split_whitespace()
        .skip(1)
        .any(|t| t.starts_with('-'))
}

impl Adaptador {
    /// Confere a configuração ANTES de qualquer execução.
    pub fn validar(&self) -> Option<ProblemaConfig> {
        if self.binario.trim().is_empty() {
            return Some(ProblemaConfig::SemBinario);
        }
        let com_marcador = self.args.iter().filter(|a| a.contains(MARCADOR_PROMPT)).count();
        match com_marcador {
            0 => Some(ProblemaConfig::SemMarcador),
            1 => None,
            _ => Some(ProblemaConfig::MarcadorRepetido),
        }
    }

    /// Um aviso sobre a configuração — que NÃO impede de usar.
    ///
    /// A separação importa (ciclo 241). O que `validar` recusa é o que
    /// torna a execução impossível: sem executável, sem `{prompt}`, ou
    /// com o marcador repetido. O resto é escolha de quem configura, na
    /// máquina de quem configura.
    ///
    /// Isto aqui já foi bloqueio, com o argumento de impedir "execução de
    /// shell pela porta dos fundos". Não impedia nada: **não existe shell
    /// no caminho**. O binário vai pra `Command::new` e os argumentos vão
    /// separados, então um executável chamado `sh -c 'claude'`
    /// simplesmente não existe e falha ao iniciar. O bloqueio só tirava
    /// da pessoa a chance de apontar o que ela quisesse.
    pub fn aviso(&self) -> Option<AvisoConfig> {
        parece_comando_de_shell(self.binario.trim())
            .then_some(AvisoConfig::ParecComandoDeShell)
    }

    /// Monta os argumentos finais com o prompt no lugar do marcador.
    ///
    /// O prompt entra como UM argumento, nunca concatenado numa string
    /// de shell — então aspas, quebras de linha e `$(...)` dentro dele
    /// são texto, não código.
    pub fn montar_args(&self, prompt: &str) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        // As pastas extras entram ANTES do resto: o `{prompt}` costuma
        // ser o último argumento, e alguns agentes tratam tudo depois
        // dele como parte do prompt.
        if !self.arg_pasta_extra.trim().is_empty() {
            for pasta in self.pastas_extras.iter().filter(|p| !p.trim().is_empty()) {
                out.push(self.arg_pasta_extra.clone());
                out.push(pasta.clone());
            }
        }
        out.extend(
            self.args
                .iter()
                .map(|a| a.replace(MARCADOR_PROMPT, prompt)),
        );
        out
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
        // (nome do preset, args que ALGUMA versão anterior gravava)
        //
        // A lista cresce a cada mudança de contrato do preset. Sem
        // isso, quem já usou o app fica preso na versão velha — foi
        // como a configuração do usuário ficou sem `--sandbox
        // workspace-write` e a execução travava em "somente leitura".
        const ANTIGOS: [(&str, &[&str]); 5] = [
            // ciclo 202
            ("Claude Code", &["-p", MARCADOR_PROMPT]),
            ("Codex", &["exec", MARCADOR_PROMPT]),
            // ciclos 213 e 214: ganharam streaming, ainda sem escrita
            (
                "Claude Code",
                &["-p", "--output-format", "stream-json", "--verbose", MARCADOR_PROMPT],
            ),
            ("Codex", &["exec", "--json", MARCADOR_PROMPT]),
            // ciclo 216: ganhou escrita, ainda sem rede pro harness
            (
                "Codex",
                &["exec", "--json", "--sandbox", "workspace-write", MARCADOR_PROMPT],
            ),
        ];
        let antigo = ANTIGOS.iter().find(|(nome, args)| {
            self.nome == *nome
                && self.args.len() == args.len()
                && self.args.iter().zip(args.iter()).all(|(a, b)| a == b)
        });
        let atualizado = match antigo {
            Some((nome, _)) => match Self::presets().into_iter().find(|p| p.nome == *nome) {
                Some(preset) => Self {
                    args: preset.args,
                    formato: preset.formato,
                    // A pasta EXTRA configurada é da pessoa e fica;
                    // o argumento que a transporta é do preset.
                    arg_pasta_extra: preset.arg_pasta_extra,
                    ..self
                },
                None => self,
            },
            None => self,
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
                    // Executar uma proposta é EDITAR arquivo. Sem isto,
                    // o agente lê tudo e não consegue mudar nada — foi
                    // o que travou a etapa de execução.
                    //
                    // Escrever é limitado à pasta de trabalho, que quem
                    // escolhe é a pessoa: é dela a aprovação de onde o
                    // agente pode mexer.
                    "--permission-mode".into(),
                    "acceptEdits".into(),
                    MARCADOR_PROMPT.into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                arg_pasta_extra: "--add-dir".into(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
            },
            Adaptador {
                nome: "Codex".into(),
                binario: "codex".into(),
                args: vec![
                    "exec".into(),
                    "--json".into(),
                    // O padrão do `codex exec` é somente leitura: ele
                    // respondia "este ambiente está em modo somente
                    // leitura" e a execução não saía do lugar.
                    "--sandbox".into(),
                    "workspace-write".into(),
                    // Sem rede, o sandbox recusa até abrir socket local
                    // ("Operation not permitted"), e o agente não
                    // consegue rodar o harness — que fala com o app por
                    // WebSocket em 127.0.0.1. Um ciclo de UI sem
                    // harness é metade de um ciclo.
                    //
                    // O Codex não sabe liberar só o localhost: isto
                    // abre a rede inteira. É um passo a mais sobre o
                    // `workspace-write`, que já deixa o agente editar o
                    // código — quem não quer, tira daqui.
                    "-c".into(),
                    "sandbox_workspace_write.network_access=true".into(),
                    MARCADOR_PROMPT.into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                arg_pasta_extra: "--add-dir".into(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
            },
            Adaptador {
                nome: "opencode".into(),
                binario: "opencode".into(),
                args: vec!["run".into(), MARCADOR_PROMPT.into()],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                // Não confirmado contra o binário; vazio significa
                // "não sei mandar pasta extra pra ele".
                arg_pasta_extra: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                // O opencode tem `--format json`, mas o formato dos
                // eventos dele não foi conferido contra um binário
                // rodando de verdade — aqui não havia modelo
                // configurado pra provocar uma saída. Fica em `Texto`,
                // que funciona com qualquer agente, até alguém checar.
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

/// Extensões que o Windows considera executáveis quando o `PATHEXT` não
/// diz nada. É o valor de fábrica do sistema, na mesma ordem.
const PATHEXT_PADRAO: &str = ".COM;.EXE;.BAT;.CMD";

/// Encontra o executável de verdade por trás do nome configurado.
///
/// No Linux e no macOS isto é trabalho do sistema: `execvp` procura no
/// `PATH` sozinho, e o nome cru serve. No Windows não serve, e é o item
/// B1 do diagnóstico de portabilidade:
///
/// `claude`, `codex` e `opencode` são instalados pelo npm como **shims**
/// — `claude.cmd`, não `claude.exe`. O `CreateProcessW`, que é quem o
/// `Command::new` chama, resolve `.exe` e `.com` e mais nada. Buscar
/// `claude` no `PATH` acha `claude.cmd`, mas o spawn falha antes de
/// qualquer coisa, com "programa não encontrado" — a mensagem menos útil
/// possível, porque o programa ESTÁ lá.
///
/// Resolvendo aqui, o que chega no `Command::new` é o caminho completo
/// com extensão. A partir do Rust 1.77.2 a biblioteca padrão reconhece
/// `.bat`/`.cmd` e cuida de passar pelo `cmd.exe` com o escape correto,
/// então não precisamos (nem queremos) montar essa linha à mão.
///
/// `existe` entra por parâmetro para o teste rodar na máquina de quem
/// desenvolve, que é Linux: o comportamento do Windows é decidido pelo
/// conteúdo do `PATH` e do `PATHEXT`, não pelo sistema em que o código
/// está compilando.
pub fn resolver_executavel(
    nome: &str,
    path_env: &str,
    pathext: &str,
    windows: bool,
    existe: impl Fn(&std::path::Path) -> bool,
) -> Option<String> {
    let nome = nome.trim();
    if nome.is_empty() {
        return None;
    }
    // O sistema entra por parâmetro, não por `cfg!`, pelo mesmo motivo
    // que `existe`: `C:\bin;C:\Windows` só se separa direito sabendo
    // que o separador é `;`, e num teste rodando no Linux o `cfg!`
    // responderia `:` — que corta o `C:` ao meio.
    let sep_lista = if windows { ';' } else { ':' };
    let sep_dir = if windows { '\\' } else { '/' };

    let extensoes: Vec<&str> = {
        let bruto = if pathext.trim().is_empty() { PATHEXT_PADRAO } else { pathext };
        bruto
            .split(';')
            .map(str::trim)
            .filter(|e| !e.is_empty())
            .collect()
    };

    // Caminho já dado por inteiro: não se procura no `PATH`, só se
    // completa a extensão que faltar. É o que faz `C:\Program Files\Meu
    // Agente\claude` achar `claude.cmd` sem a pessoa precisar saber que
    // a extensão existe.
    if nome.contains('/') || nome.contains('\\') {
        return completar(nome, &extensoes, &existe);
    }

    for dir in path_env.split(sep_lista).filter(|d| !d.trim().is_empty()) {
        // As aspas que o Windows deixa entrar no `PATH` não fazem parte
        // do caminho — `"C:\ferramentas"` é a pasta `C:\ferramentas`.
        let dir = dir.trim().trim_matches('"').trim_end_matches(['/', '\\']);
        let base = format!("{dir}{sep_dir}{nome}");
        if let Some(achado) = completar(&base, &extensoes, &existe) {
            return Some(achado);
        }
    }
    None
}

/// Tenta o caminho como está e, se não houver arquivo, com cada extensão.
///
/// Duas decisões:
///
/// - O nome exato ganha do nome com extensão, senão um `claude` de
///   verdade perderia pra um `claude.bat` na mesma pasta.
/// - A extensão é ACRESCENTADA, nunca trocada. É o que o `PATHEXT` faz,
///   e é a diferença entre `meu.agente` virar `meu.agente.cmd` (certo)
///   ou `meu.cmd` (que é outro programa, ou nenhum).
fn completar(
    base: &str,
    extensoes: &[&str],
    existe: &impl Fn(&std::path::Path) -> bool,
) -> Option<String> {
    if existe(std::path::Path::new(base)) {
        return Some(base.to_string());
    }
    for ext in extensoes {
        let com_ext = format!("{base}{ext}");
        if existe(std::path::Path::new(&com_ext)) {
            return Some(com_ext);
        }
    }
    None
}

/// O que passar pro `Command::new`, lido do ambiente do processo.
///
/// Fora do Windows devolve o nome cru de propósito: quem resolve `PATH`
/// lá é o sistema, e ele faz isso melhor do que nós (respeita links,
/// permissão de execução, `PATH` alterado depois que o app subiu).
/// Devolve o nome cru no Windows também quando a busca não acha nada —
/// aí o erro de spawn é do sistema, e a mensagem cita o nome que a
/// pessoa configurou, não um caminho inventado por nós.
pub fn executavel_para_spawn(binario: &str) -> String {
    if !cfg!(windows) {
        return binario.to_string();
    }
    let path_env = std::env::var("PATH").unwrap_or_default();
    let pathext = std::env::var("PATHEXT").unwrap_or_default();
    resolver_executavel(binario, &path_env, &pathext, true, |p| p.is_file())
        .unwrap_or_else(|| binario.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Sistema de arquivos de mentira, com um conjunto fixo de arquivos.
    ///
    /// Ignora caixa de propósito: o `PATHEXT` de verdade vem em
    /// MAIÚSCULAS (`.COM;.EXE;.BAT;.CMD`) e o npm instala `claude.cmd`
    /// em minúsculas. No Windows os dois são o mesmo arquivo, e um
    /// fixture sensível a caixa reprovaria um código que funciona.
    fn arvore<'a>(arquivos: &'a [&'a str]) -> impl Fn(&std::path::Path) -> bool + 'a {
        move |p| {
            let alvo = p.to_string_lossy().replace('\\', "/");
            arquivos
                .iter()
                .any(|a| a.replace('\\', "/").eq_ignore_ascii_case(&alvo))
        }
    }

    const PATHEXT: &str = ".COM;.EXE;.BAT;.CMD";

    /// Confere o caminho achado ignorando a caixa da extensão.
    ///
    /// A extensão volta como está no `PATHEXT` (`.EXE`), não como está
    /// no disco (`.exe`). No Windows os dois abrem o mesmo arquivo, e
    /// exigir a caixa do disco seria cobrar do código uma consulta que
    /// ele não precisa fazer.
    #[track_caller]
    fn achou(resultado: Option<String>, esperado: &str) {
        let r = resultado.expect("não achou o executável");
        assert!(
            r.eq_ignore_ascii_case(esperado),
            "achou `{r}`, esperava `{esperado}`"
        );
    }

    #[test]
    fn acha_o_shim_cmd_que_o_npm_instala() {
        // O caso B1 inteiro: `claude` existe no PATH, mas como `.cmd`.
        let achado = resolver_executavel(
            "claude",
            r"C:\Windows\system32;C:\Users\e\AppData\Roaming\npm",
            PATHEXT,
            true,
            arvore(&[r"C:\Users\e\AppData\Roaming\npm\claude.cmd"]),
        );
        achou(achado, r"C:\Users\e\AppData\Roaming\npm\claude.cmd");
    }

    #[test]
    fn o_nome_exato_ganha_da_extensao() {
        // Com `claude` e `claude.bat` na mesma pasta, quem a pessoa
        // pediu foi `claude`.
        let achado = resolver_executavel(
            "claude",
            "/usr/bin",
            PATHEXT,
            false,
            arvore(&["/usr/bin/claude", "/usr/bin/claude.bat"]),
        );
        achou(achado, "/usr/bin/claude");
    }

    #[test]
    fn respeita_a_ordem_do_path() {
        let achado = resolver_executavel(
            "codex",
            r"C:\primeiro;C:\segundo",
            PATHEXT,
            true,
            arvore(&[r"C:\primeiro\codex.exe", r"C:\segundo\codex.exe"]),
        );
        achou(achado, r"C:\primeiro\codex.exe");
    }

    #[test]
    fn respeita_a_ordem_do_pathext() {
        let achado = resolver_executavel(
            "opencode",
            r"C:\bin",
            PATHEXT,
            true,
            arvore(&[r"C:\bin\opencode.cmd", r"C:\bin\opencode.exe"]),
        );
        achou(achado, r"C:\bin\opencode.exe");
    }

    #[test]
    fn caminho_com_espaco_completa_a_extensao_sem_procurar_no_path() {
        // B2 já não bloqueia a configuração; falta o caminho com espaço
        // funcionar de verdade quando o executável é um shim.
        let achado = resolver_executavel(
            r"C:\Program Files\Meu Agente\claude",
            r"C:\Windows\system32",
            PATHEXT,
            true,
            arvore(&[r"C:\Program Files\Meu Agente\claude.cmd"]),
        );
        achou(achado, r"C:\Program Files\Meu Agente\claude.cmd");
    }

    #[test]
    fn caminho_absoluto_que_existe_passa_intacto() {
        let achado = resolver_executavel(
            "/opt/agentes/claude",
            "/usr/bin",
            PATHEXT,
            false,
            arvore(&["/opt/agentes/claude"]),
        );
        achou(achado, "/opt/agentes/claude");
    }

    #[test]
    fn aspas_no_path_nao_viram_parte_da_pasta() {
        let achado = resolver_executavel(
            "claude",
            r#""C:\ferramentas";C:\Windows"#,
            PATHEXT,
            true,
            arvore(&[r"C:\ferramentas\claude.cmd"]),
        );
        achou(achado, r"C:\ferramentas\claude.cmd");
    }

    #[test]
    fn pathext_vazio_cai_no_padrao_do_sistema() {
        let achado = resolver_executavel(
            "claude",
            r"C:\bin",
            "",
            true,
            arvore(&[r"C:\bin\claude.cmd"]),
        );
        achou(achado, r"C:\bin\claude.cmd");
    }

    #[test]
    fn nao_achar_devolve_nada_em_vez_de_um_caminho_inventado() {
        assert_eq!(
            resolver_executavel("claude", r"C:\bin", PATHEXT, true, arvore(&[])),
            None
        );
        assert_eq!(resolver_executavel("   ", r"C:\bin", PATHEXT, true, arvore(&[])), None);
    }

    fn base() -> Adaptador {
        Adaptador {
            nome: "teste".into(),
            binario: "eco".into(),
            args: vec!["-p".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            pastas_extras: Vec::new(),
            arg_pasta_extra: String::new(),
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
        assert!(p.contains("· Read"), "progresso sem a ferramenta: {p}");
        assert!(p.contains("Vou ler a spec"), "progresso sem o texto: {p}");
        // O progresso é ruído de execução: não pode virar a resposta.
        assert_eq!(l.resposta().unwrap(), "Proposta pronta");
    }

    #[test]
    fn o_progresso_guarda_o_texto_inteiro_nao_so_a_primeira_linha() {
        // Durante uma execução longa é o miolo do raciocínio que diz se
        // o agente entendeu o pedido. Guardar só a primeira linha
        // escondia justamente isso.
        let mut l = LeitorStream::novo();
        l.linha(
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"primeira\nsegunda\nterceira"}]}}"#,
        );
        let p = l.progresso();
        for esperada in ["primeira", "segunda", "terceira"] {
            assert!(p.contains(esperada), "progresso sem \"{esperada}\": {p}");
        }
    }

    #[test]
    fn entende_o_dialeto_do_codex() {
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"thread.started","thread_id":"x"}"#);
        l.linha(r#"{"type":"turn.started"}"#);
        l.linha(
            r#"{"type":"item.completed","item":{"id":"i0","type":"agent_message","text":"Vou listar a pasta."}}"#,
        );
        l.linha(
            r#"{"type":"item.started","item":{"id":"i1","type":"command_execution","command":"ls -1"}}"#,
        );
        l.linha(
            r#"{"type":"item.completed","item":{"id":"i2","type":"agent_message","text":"Há 4 itens."}}"#,
        );
        l.linha(r#"{"type":"turn.completed","usage":{}}"#);

        let p = l.progresso();
        assert!(p.contains("· ls -1"), "progresso sem o comando: {p}");
        assert!(p.contains("Vou listar a pasta."), "progresso sem a narração: {p}");
        // O Codex narra o que VAI fazer antes de fazer; a resposta é o
        // último recado, não a soma deles.
        assert_eq!(l.resposta().unwrap(), "Há 4 itens.");
    }

    #[test]
    fn o_codex_reporta_falha_do_turno() {
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"turn.failed","error":{"message":"sem credencial"}}"#);
        assert_eq!(l.resposta().unwrap_err(), "sem credencial");
    }

    #[test]
    fn o_evento_error_do_codex_vira_o_motivo() {
        // Caso real: a conta bateu o limite de uso. O motivo vinha no
        // stream, e a tela mostrava "Reading additional input from
        // stdin..." — ruído do stderr, que não diz nada.
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"error","message":"You've hit your usage limit."}"#);
        l.linha(r#"{"type":"turn.failed","error":{"message":"You've hit your usage limit."}}"#);
        assert_eq!(l.resposta().unwrap_err(), "You've hit your usage limit.");
    }

    #[test]
    fn os_dois_dialetos_nao_se_confundem() {
        // Os nomes de evento são disjuntos, então um leitor só dá conta
        // dos dois sem precisar saber de antemão qual agente falou.
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"item.completed","item":{"type":"agent_message","text":"do codex"}}"#);
        l.linha(r#"{"type":"result","is_error":false,"result":"do claude"}"#);
        // O `result` do Claude é explícito e ganha do acumulado.
        assert_eq!(l.resposta().unwrap(), "do claude");
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
    fn pastas_extras_viram_argumentos_antes_do_prompt() {
        let a = Adaptador {
            args: vec!["exec".into(), MARCADOR_PROMPT.into()],
            pastas_extras: vec!["/repo/a".into(), "/repo/b".into()],
            arg_pasta_extra: "--add-dir".into(),
            ..base()
        };
        assert_eq!(
            a.montar_args("pergunta"),
            vec!["--add-dir", "/repo/a", "--add-dir", "/repo/b", "exec", "pergunta"]
        );
    }

    #[test]
    fn sem_arg_de_pasta_extra_as_pastas_sao_ignoradas() {
        // O opencode não teve isso confirmado contra o binário: mandar
        // uma flag que ele não conhece derrubaria a execução inteira.
        let a = Adaptador {
            args: vec![MARCADOR_PROMPT.into()],
            pastas_extras: vec!["/repo/a".into()],
            arg_pasta_extra: String::new(),
            ..base()
        };
        assert_eq!(a.montar_args("p"), vec!["p"]);
    }

    #[test]
    fn pasta_extra_em_branco_nao_vira_argumento() {
        let a = Adaptador {
            args: vec![MARCADOR_PROMPT.into()],
            pastas_extras: vec!["  ".into(), "/repo/a".into()],
            arg_pasta_extra: "--add-dir".into(),
            ..base()
        };
        assert_eq!(a.montar_args("p"), vec!["--add-dir", "/repo/a", "p"]);
    }

    #[test]
    fn os_presets_que_editam_pedem_permissao_de_escrita() {
        // Executar uma proposta é editar arquivo: um preset que não
        // pede escrita lê tudo e não muda nada, que foi o que travou a
        // etapa de execução.
        for p in Adaptador::presets() {
            if p.formato != FormatoSaida::StreamJson {
                continue;
            }
            let linha = p.args.join(" ");
            assert!(
                linha.contains("workspace-write") || linha.contains("acceptEdits"),
                "preset \"{}\" não pede permissão de escrita: {linha}",
                p.nome
            );
        }
    }

    #[test]
    fn a_migracao_atualiza_o_preset_antigo_do_claude() {
        let velho = Adaptador {
            nome: "Claude Code".into(),
            binario: "/home/x/.local/bin/claude".into(),
            args: vec!["-p".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            pastas_extras: Vec::new(),
            arg_pasta_extra: String::new(),
            timeout_s: 180,
            formato: FormatoSaida::Texto,
        };
        let novo = velho.migrado();
        assert_eq!(novo.formato, FormatoSaida::StreamJson);
        assert!(
            novo.args.join(" ").contains("acceptEdits"),
            "a migração não trouxe a permissão de escrita: {:?}",
            novo.args
        );
        assert!(novo.args.contains(&"stream-json".to_string()));
        assert_eq!(novo.timeout_s, TIMEOUT_MINIMO_S);
        // O binário que a pessoa apontou continua sendo o dela.
        assert_eq!(novo.binario, "/home/x/.local/bin/claude");
    }

    #[test]
    fn a_migracao_atualiza_tambem_o_codex_da_versao_anterior() {
        let velho = Adaptador {
            nome: "Codex".into(),
            binario: "codex".into(),
            args: vec!["exec".into(), "--json".into(), MARCADOR_PROMPT.into()],
            pastas_extras: vec!["/repo/meu".into()],
            arg_pasta_extra: String::new(),
            ..base()
        };
        let novo = velho.migrado();
        assert!(
            novo.args.join(" ").contains("workspace-write"),
            "sem permissão de escrita: {:?}",
            novo.args
        );
        assert_eq!(novo.arg_pasta_extra, "--add-dir");
        // A pasta que a pessoa escolheu é dela e continua.
        assert_eq!(novo.pastas_extras, vec!["/repo/meu".to_string()]);
    }

    #[test]
    fn o_preset_do_codex_libera_rede_pro_harness() {
        let codex = Adaptador::presets()
            .into_iter()
            .find(|p| p.nome == "Codex")
            .expect("preset do Codex sumiu");
        assert!(
            codex.args.iter().any(|a| a.contains("network_access=true")),
            "sem rede o agente não consegue falar com o app pra rodar o harness: {:?}",
            codex.args
        );
    }

    #[test]
    fn a_migracao_leva_o_codex_do_216_pro_preset_com_rede() {
        let do_216 = Adaptador {
            nome: "Codex".into(),
            binario: "codex".into(),
            args: vec![
                "exec".into(),
                "--json".into(),
                "--sandbox".into(),
                "workspace-write".into(),
                MARCADOR_PROMPT.into(),
            ],
            ..base()
        };
        assert!(
            do_216.migrado().args.iter().any(|a| a.contains("network_access=true")),
            "quem já usou o app ficaria sem conseguir validar"
        );
    }

    #[test]
    fn a_migracao_nao_mexe_em_args_montados_a_mao() {
        let meu = Adaptador {
            nome: "Claude Code".into(),
            binario: "claude".into(),
            args: vec!["--dangerously-skip".into(), MARCADOR_PROMPT.into()],
            cwd: String::new(),
            pastas_extras: Vec::new(),
            arg_pasta_extra: String::new(),
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
    fn a_raiz_do_projeto_sobe_ate_o_git() {
        // O caso do Anotadinho: o vault mora dentro do repositório, e é
        // da raiz que dá pra ler o código E as notas.
        let raiz = raiz_do_projeto("/casa/proj/VaultX", |d| d == std::path::Path::new("/casa/proj"));
        assert_eq!(raiz, "/casa/proj");
    }

    #[test]
    fn o_proprio_vault_sendo_repositorio_vale_como_raiz() {
        let raiz = raiz_do_projeto("/casa/notas", |d| d == std::path::Path::new("/casa/notas"));
        assert_eq!(raiz, "/casa/notas");
    }

    #[test]
    fn sem_git_em_lugar_nenhum_fica_o_vault() {
        let raiz = raiz_do_projeto("/casa/notas", |_| false);
        assert_eq!(raiz, "/casa/notas");
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
    fn linha_de_shell_no_binario_avisa_mas_nao_impede() {
        // Não existe shell no caminho: o binário vai pra `Command::new` e
        // os argumentos vão separados. Uma linha inteira não vira
        // execução de shell — vira executável inexistente. Avisar serve;
        // bloquear só tirava a escolha de quem configura (ciclo 241).
        let mut a = base();
        a.binario = "sh -c".into();
        assert_eq!(a.validar(), None, "impediu o que é escolha da pessoa");
        assert_eq!(a.aviso(), Some(AvisoConfig::ParecComandoDeShell));
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
            // ── dialeto do Claude Code ──
            Some("system") => {
                if v.get("subtype").and_then(|s| s.as_str()) == Some("init") {
                    self.anotar("conectado");
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
                                self.anotar(txt);
                            }
                        }
                        Some("tool_use") => {
                            if let Some(nome) = b.get("name").and_then(|n| n.as_str()) {
                                self.anotar(&format!("· {nome}"));
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

            // ── dialeto do Codex ──
            Some("thread.started") => self.anotar("conectado"),
            Some("item.started") | Some("item.completed") => {
                let completo = v.get("type").and_then(|t| t.as_str()) == Some("item.completed");
                let Some(item) = v.get("item") else { return };
                match item.get("type").and_then(|t| t.as_str()) {
                    Some("agent_message") => {
                        // Só no `completed`: o `started` do mesmo item
                        // traz o texto pela metade e duplicaria tudo.
                        if completo {
                            if let Some(txt) = item.get("text").and_then(|t| t.as_str()) {
                                self.texto.push_str(txt);
                                // A resposta é o ÚLTIMO recado, não a
                                // soma deles: o Codex narra o que vai
                                // fazer antes de fazer, e essa narração
                                // não é resposta.
                                self.resultado = Some(txt.to_string());
                                self.anotar(txt);
                            }
                        }
                    }
                    Some("command_execution") => {
                        if !completo {
                            if let Some(cmd) = item.get("command").and_then(|c| c.as_str()) {
                                self.anotar(&format!("· {cmd}"));
                            }
                        }
                    }
                    Some(outro) if !completo => self.anotar(&format!("· {outro}")),
                    _ => {}
                }
            }
            // O Codex emite `error` antes do `turn.failed`; outros
            // agentes podem emitir só um dos dois. O primeiro que
            // chegar vale — a mensagem é a mesma.
            Some("error") => {
                if self.erro.is_none() {
                    if let Some(m) = v.get("message").and_then(|m| m.as_str()) {
                        self.erro = Some(m.to_string());
                    }
                }
            }
            Some("turn.failed") => {
                self.erro = Some(
                    v.get("error")
                        .and_then(|e| e.get("message"))
                        .and_then(|m| m.as_str())
                        .unwrap_or("o agente terminou com erro")
                        .to_string(),
                );
            }
            _ => {}
        }
    }

    /// Guarda um trecho no progresso, quebrado em linhas.
    ///
    /// Guarda o texto INTEIRO, não só a primeira linha: durante uma
    /// execução longa é justamente o miolo do raciocínio que diz se o
    /// agente entendeu o pedido. A janela rolante evita que isso cresça
    /// sem limite, e a tela dá altura fixa com rolagem própria.
    fn anotar(&mut self, texto: &str) {
        for linha in texto.lines() {
            let l = linha.trim_end();
            if l.trim().is_empty() {
                continue;
            }
            self.progresso.push(resumir(l));
        }
        let sobra = self.progresso.len().saturating_sub(LINHAS_DE_PROGRESSO);
        if sobra > 0 {
            self.progresso.drain(..sobra);
        }
    }

    /// As últimas linhas de progresso, pra mostrar enquanto roda.
    pub fn progresso(&self) -> String {
        self.progresso.join("\n")
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

/// Quantas linhas de progresso ficam guardadas.
///
/// Generoso de propósito: a tela mostra numa caixa de altura fixa com
/// rolagem, então mais linhas significam mais contexto pra rolar, não
/// uma tela empurrada pra fora.
const LINHAS_DE_PROGRESSO: usize = 200;

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

/// O diretório onde o agente deve trabalhar, partindo do vault.
///
/// Sobe procurando um `.git`: num projeto que guarda as notas dentro do
/// próprio repositório — que é o caso do Anotadinho — a raiz é o lugar
/// de onde dá pra ler o código E as notas. Sem `.git` em lugar nenhum,
/// fica o vault mesmo, que é o comportamento antigo.
///
/// Recebe a lista de ancestrais já pronta pra ser testável sem tocar em
/// disco; quem chama passa `existe` sabendo procurar `.git`.
pub fn raiz_do_projeto<F>(vault: &str, tem_git: F) -> String
where
    F: Fn(&std::path::Path) -> bool,
{
    let caminho = std::path::Path::new(vault);
    let mut atual = Some(caminho);
    while let Some(dir) = atual {
        if tem_git(dir) {
            return dir.to_string_lossy().to_string();
        }
        atual = dir.parent();
    }
    vault.to_string()
}

#[cfg(test)]
mod testes_config {
    use super::*;

    fn com_binario(binario: &str) -> Adaptador {
        Adaptador {
            nome: "teste".into(),
            binario: binario.into(),
            args: vec!["{prompt}".into()],
            ..Default::default()
        }
    }

    #[test]
    fn caminho_com_espaco_e_aceito() {
        // Recusar espaço recusava caminho legítimo: `My Tools` no Linux
        // e qualquer `C:\Program Files\...` no Windows (ciclo 239).
        for caminho in [
            "/home/eu/My Tools/claude",
            "C:\\Program Files\\Claude\\claude.cmd",
            "/usr/local/bin/claude",
        ] {
            assert_eq!(
                com_binario(caminho).validar(),
                None,
                "recusou caminho válido: {caminho}"
            );
        }
    }

    #[test]
    fn linha_de_comando_avisa_sem_impedir() {
        for linha in ["claude -p", "sh -c 'claude'", "claude && rm -rf /", "echo $(whoami)"] {
            let a = com_binario(linha);
            assert_eq!(a.validar(), None, "impediu de salvar: {linha}");
            assert_eq!(
                a.aviso(),
                Some(AvisoConfig::ParecComandoDeShell),
                "não avisou sobre: {linha}"
            );
        }
    }

    #[test]
    fn caminho_normal_nao_gera_aviso() {
        for caminho in ["claude", "/opt/My Tools/claude", "C:\\Program Files\\c.cmd"] {
            assert_eq!(com_binario(caminho).aviso(), None, "avisou à toa: {caminho}");
        }
    }

    #[test]
    fn o_marcador_continua_obrigatorio_e_unico() {
        let mut a = com_binario("claude");
        a.args = vec!["-p".into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::SemMarcador));
        a.args = vec!["{prompt}".into(), "{prompt}".into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::MarcadorRepetido));
    }
}
