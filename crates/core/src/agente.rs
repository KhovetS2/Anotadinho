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

/// O nome do servidor MCP do Anotadinho na config do agente: as
/// ferramentas chegam como `mcp__anotadinho__propor`, e é por ele que a
/// execução as libera (ciclo 529).
pub const SERVIDOR_MCP: &str = "anotadinho";

/// O JSON de configuração MCP que liga o servidor do Anotadinho num
/// agente que aceita `--mcp-config` (ciclo 426).
///
/// `cli` é o caminho do `anotadinho-cli`; `vault` é o vault daquela
/// conversa — o servidor nasce apontado pra ele, então o agente não
/// consegue mexer noutro por engano.
///
/// O nome do servidor é `anotadinho`, o que faz as ferramentas chegarem
/// ao agente como `mcp__anotadinho__propor` e afins.
pub fn config_mcp(cli: &str, vault: &str) -> String {
    let escapar = |s: &str| s.replace('\\', "\\\\").replace('"', "\\\"");
    format!(
        "{{\n  \"mcpServers\": {{\n    \"{SERVIDOR_MCP}\": {{\n      \"command\": \"{}\",\n      \"args\": [\"--vault\", \"{}\", \"mcp\"]\n    }}\n  }}\n}}\n",
        escapar(cli),
        escapar(vault)
    )
}

/// O `anotadinho-cli` que vai atender o MCP daquela execução (ciclos
/// 426 e 450), ou `None` quando não há nenhum.
///
/// Mora aqui porque TUI e janela precisam da MESMA resposta. Primeiro o
/// vizinho do executável atual: é onde ele está em `target/debug` e,
/// desde o ciclo 450, no app instalado (o instalador o leva como
/// `externalBin`). Depois o `PATH`, de quem instalou pelo cargo.
///
/// Devolver `None` em vez do nome cru é o ponto: antes, sem CLI em lugar
/// nenhum, o agente recebia `--mcp-config` apontando pra um programa que
/// não existia, subia sem as ferramentas do vault, e a única pista era a
/// qualidade da resposta.
///
/// O sistema e o disco entram por parâmetro, como em
/// `resolver_executavel`, pra o ramo do Windows rodar no teste.
pub fn localizar_cli(
    executavel_atual: Option<&std::path::Path>,
    path_env: &str,
    windows: bool,
    existe: impl Fn(&std::path::Path) -> bool,
) -> Option<String> {
    let nome = if windows { "anotadinho-cli.exe" } else { "anotadinho-cli" };
    let sep_dir = if windows { '\\' } else { '/' };
    // A pasta pelo TEXTO, não por `Path::parent`: no teste, que roda no
    // Linux, `parent` não reconhece `\` e devolveria a pasta errada.
    let pasta = executavel_atual.map(|exe| exe.to_string_lossy().to_string()).and_then(|exe| {
        let corte = if windows { exe.rfind(['\\', '/']) } else { exe.rfind('/') };
        corte.map(|i| exe[..i].to_string())
    });
    if let Some(pasta) = pasta {
        let vizinho = format!("{pasta}{sep_dir}{nome}");
        if existe(std::path::Path::new(&vizinho)) {
            return Some(vizinho);
        }
    }
    let sep_lista = if windows { ';' } else { ':' };
    path_env
        .split(sep_lista)
        .map(|d| d.trim().trim_matches('"').trim_end_matches(['/', '\\']))
        .filter(|d| !d.is_empty())
        .map(|d| format!("{d}{sep_dir}{nome}"))
        .find(|c| existe(std::path::Path::new(c)))
}

/// `localizar_cli` com o processo, o `PATH` e o disco de verdade.
pub fn cli_ao_lado(executavel_atual: Option<std::path::PathBuf>) -> Option<String> {
    let path_env = std::env::var("PATH").unwrap_or_default();
    localizar_cli(executavel_atual.as_deref(), &path_env, cfg!(windows), |p| p.is_file())
}

/// O aviso de quando o agente sabe receber MCP e não há CLI pra atender
/// (ciclo 450). Uma frase só, a MESMA na janela e na TUI.
pub const AVISO_SEM_CLI: &str =
    "sem as ferramentas do vault: não achei o anotadinho-cli ao lado do app nem no PATH";

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
    /// Argumentos. Com a entrega por argumento, exatamente um deles deve
    /// conter `{prompt}`; pelo stdin, ele pode faltar (ver `entrada`).
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
    /// Argumento que o agente usa pra CONTINUAR uma sessão (`--resume`)
    /// — ciclo 433.
    ///
    /// Com ele, a segunda pergunta de uma conversa não remonta o
    /// histórico: o agente já tem. Vazio = ele não sabe continuar, e
    /// cada envio é um disparo novo, como era até aqui.
    #[serde(default)]
    pub arg_sessao: String,
    /// Argumento que o agente usa pra receber um arquivo de configuração
    /// MCP (`--mcp-config`) — ciclo 426.
    ///
    /// Com isto o app LIGA o servidor MCP do próprio Anotadinho na
    /// execução, e o agente ganha `ler_pagina`, `buscar`, `propor` e o
    /// resto do contrato sem ninguém configurar nada por fora. Vazio = o
    /// agente não sabe receber, e sobra o caminho do CLI, que o prompt
    /// explica (ciclo 425).
    #[serde(default)]
    pub arg_mcp: String,
    /// Segundos até desistir. 0 = sem limite (desaconselhado).
    #[serde(default = "timeout_padrao")]
    pub timeout_s: u64,
    /// Como ler a saída (ciclo 213). Ausente em configuração antiga,
    /// onde `Texto` é o comportamento de sempre.
    #[serde(default)]
    pub formato: FormatoSaida,
    /// Por onde o prompt chega ao agente (ciclo 449). Ausente em
    /// configuração antiga, onde `Argumento` é o comportamento de sempre.
    #[serde(default)]
    pub entrada: EntradaPrompt,
}

/// Por onde o prompt chega ao agente (ciclo 449).
///
/// A linha de comando tem dois limites que o Anotadinho estoura. No
/// Windows, `claude`/`codex` instalados pelo npm são shims `.cmd`, e a
/// biblioteca padrão RECUSA argumento com `\r` ou `\n` pra `.cmd`/`.bat`
/// (`batch file arguments are invalid`) — e todo prompt montado aqui tem
/// quebra de linha, porque a blindagem do vault é feita em linhas
/// próprias. Ou seja: no Windows nenhuma conversa chegava a rodar. No
/// Linux, um argumento sozinho não passa de 128 KiB (`E2BIG`), e uma
/// conversa com algumas transclusões chega lá.
///
/// Pelo stdin não há nenhum dos dois. Continua sem shell: são bytes num
/// pipe, ninguém os interpreta.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntradaPrompt {
    /// O prompt no lugar de `{prompt}`, como um argumento só.
    #[default]
    Argumento,
    /// O prompt pela entrada padrão. O argumento que for exatamente
    /// `{prompt}` sai da lista; o agente que precisa de um marcador pra
    /// ler o stdin (o `-` do Codex) o recebe nos próprios argumentos.
    Stdin,
}

/// O que disparar: os argumentos e, se houver, o que vai pelo stdin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Execucao {
    /// Argumentos finais, já com MCP, sessão e pastas extras.
    pub args: Vec<String>,
    /// O prompt, quando a entrega é pelo stdin. `None` = stdin fechado.
    pub entrada: Option<String>,
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
    /// Pelo stdin, `--msg={prompt}` não tem tradução: o marcador só pode
    /// ser um argumento inteiro, que sai da lista.
    MarcadorEmbutidoComStdin,
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
            Self::MarcadorEmbutidoComStdin => {
                "com o prompt pela entrada padrão, {prompt} só pode ser um argumento \
                 inteiro (ou nenhum)"
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
        match (self.entrada, com_marcador) {
            (_, 2..) => Some(ProblemaConfig::MarcadorRepetido),
            (EntradaPrompt::Argumento, 0) => Some(ProblemaConfig::SemMarcador),
            (EntradaPrompt::Stdin, _)
                if self.args.iter().any(|a| a.contains(MARCADOR_PROMPT) && a != MARCADOR_PROMPT) =>
            {
                Some(ProblemaConfig::MarcadorEmbutidoComStdin)
            }
            _ => None,
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
        self.montar_args_com_mcp(prompt, "")
    }

    /// Como `montar_args`, mas ligando o servidor MCP do Anotadinho pelo
    /// arquivo de configuração em `config_mcp` (ciclo 426).
    ///
    /// Caminho vazio, ou agente que não sabe receber, devolve os mesmos
    /// argumentos de antes: ligar MCP é melhoria, não requisito.
    pub fn montar_args_com_mcp(&self, prompt: &str, config_mcp: &str) -> Vec<String> {
        self.montar_args_completo(prompt, config_mcp, "")
    }

    /// Com MCP e com SESSÃO (ciclo 433): só os argumentos. Pelo stdin, o
    /// prompt NÃO está neles — quem dispara usa `montar_execucao`.
    pub fn montar_args_completo(&self, prompt: &str, config_mcp: &str, sessao: &str) -> Vec<String> {
        self.montar_execucao(prompt, config_mcp, sessao).args
    }

    /// O que disparar (ciclo 449): argumentos e, pelo stdin, o prompt.
    pub fn montar_execucao(&self, prompt: &str, config_mcp: &str, sessao: &str) -> Execucao {
        let mut out: Vec<String> = Vec::new();
        if !self.arg_sessao.trim().is_empty() && !sessao.trim().is_empty() {
            out.push(self.arg_sessao.clone());
            out.push(sessao.to_string());
        }
        if !self.arg_mcp.trim().is_empty() && !config_mcp.trim().is_empty() {
            out.push(self.arg_mcp.clone());
            out.push(config_mcp.to_string());
            // Ligar o servidor não basta (ciclo 529): no `-p`, o Claude Code
            // NEGA toda ferramenta MCP que não foi liberada, e o agente
            // respondia "o acesso às ferramentas do Anotadinho não foi
            // autorizado" — sem `propor`, sem `ler_pagina`. Medido em
            // 2026-10-26: sem isto, NEGADA; com isto, leu a página. Uma
            // forma só (`=`): o `--allowedTools` aceita vários valores e
            // engoliria o argumento seguinte. Aqui, e não no preset, pra
            // valer também nos agentes que a pessoa copiou do Claude Code.
            if self.arg_mcp.trim() == "--mcp-config" {
                out.push(format!("--allowedTools=mcp__{SERVIDOR_MCP}"));
            }
        }
        // As pastas extras entram ANTES do resto: o `{prompt}` costuma
        // ser o último argumento, e alguns agentes tratam tudo depois
        // dele como parte do prompt.
        if !self.arg_pasta_extra.trim().is_empty() {
            for pasta in self.pastas_extras.iter().filter(|p| !p.trim().is_empty()) {
                out.push(self.arg_pasta_extra.clone());
                out.push(pasta.clone());
            }
        }
        match self.entrada {
            EntradaPrompt::Argumento => {
                out.extend(self.args.iter().map(|a| a.replace(MARCADOR_PROMPT, prompt)));
                Execucao { args: out, entrada: None }
            }
            EntradaPrompt::Stdin => {
                out.extend(self.args.iter().filter(|a| *a != MARCADOR_PROMPT).cloned());
                Execucao { args: out, entrada: Some(prompt.to_string()) }
            }
        }
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
        const ANTIGOS: [(&str, &[&str]); 8] = [
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
            // ciclo 449: o prompt saiu da linha de comando pro stdin
            (
                "Claude Code",
                &[
                    "-p", "--output-format", "stream-json", "--verbose",
                    "--permission-mode", "acceptEdits", MARCADOR_PROMPT,
                ],
            ),
            (
                "Codex",
                &[
                    "exec", "--json", "--sandbox", "workspace-write", "-c",
                    "sandbox_workspace_write.network_access=true", MARCADOR_PROMPT,
                ],
            ),
            // ciclo 527: o opencode medido — JSON, stdin e `--auto`.
            ("opencode", &["run", MARCADOR_PROMPT]),
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
                    entrada: preset.entrada,
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
                    // Sem prompt posicional, o `claude -p` lê do stdin
                    // (medido em 2026-10-06, ciclo 449).
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                arg_pasta_extra: "--add-dir".into(),
                arg_mcp: "--mcp-config".into(),
                arg_sessao: "--resume".into(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
                entrada: EntradaPrompt::Stdin,
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
                    // `-` = instruções pelo stdin (ciclo 449; o `--help`
                    // dele diz, e foi medido em 2026-10-06).
                    "-".into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                arg_pasta_extra: "--add-dir".into(),
                arg_mcp: String::new(),
                arg_sessao: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
                entrada: EntradaPrompt::Stdin,
            },
            // Medido em 2026-10-25 com o opencode 1.18.14 (ciclo 527): o
            // `--format json` emite `text`, `tool_use`, `step_finish` (com
            // tokens e custo) e `error`; sem mensagem no argumento, ele lê
            // o prompt do stdin; `--auto` aprova as ferramentas, sem o que
            // uma execução sem terminal pararia perguntando. O modelo
            // padrão daqui respondeu 401 (chave do plano expirada); o
            // `opencode/big-pickle`, grátis, editou, testou e commitou em
            // 14 s. Troque o `-m` pelo seu (`opencode models` lista).
            Adaptador {
                nome: "opencode".into(),
                binario: "opencode".into(),
                args: vec![
                    "run".into(),
                    "--format".into(),
                    "json".into(),
                    "--auto".into(),
                    "-m".into(),
                    "opencode/big-pickle".into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                // Não tem argumento de pasta extra.
                arg_pasta_extra: String::new(),
                arg_mcp: String::new(),
                // O `-s` dele continua a sessão, mas tem que vir DEPOIS do
                // `run`, e o app põe o par da sessão antes dos argumentos.
                // Sem sessão, o histórico vai no prompt.
                arg_sessao: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
                entrada: EntradaPrompt::Stdin,
            },
            // O "júnior" grátis (ciclo 521): um modelo local pelo aider.
            // Medido em 2026-10-25 com o aider 0.86.2 e o
            // `qwen2.5-coder:7b`: edita, commita sozinho (o histórico da
            // tarefa fica no branch dela) e termina com a linha
            // `RESULTADO:`. Na primeira vez ele põe `.aider*` no
            // `.gitignore` do repositório — aparece nas Mudanças uma vez.
            Adaptador {
                nome: "aider + Ollama (local)".into(),
                binario: "aider".into(),
                args: vec![
                    "--model".into(),
                    "ollama_chat/qwen2.5-coder:7b".into(),
                    // Sem isto o aider não acha o Ollama e para pedindo
                    // a variável — e numa execução sem terminal isso é
                    // travar até o prazo.
                    "--set-env".into(),
                    "OLLAMA_API_BASE=http://127.0.0.1:11434".into(),
                    // Sem pergunta nenhuma: não há ninguém pra responder.
                    "--yes-always".into(),
                    "--no-pretty".into(),
                    "--no-stream".into(),
                    "--no-check-update".into(),
                    "--no-show-model-warnings".into(),
                    "--no-analytics".into(),
                    "--no-fancy-input".into(),
                    "--message".into(),
                    MARCADOR_PROMPT.into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                // O `--read` do aider é de ARQUIVO, não de pasta.
                arg_pasta_extra: String::new(),
                arg_mcp: String::new(),
                arg_sessao: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::Texto,
                // O `--message-file` não lê o stdin no Windows; o
                // argumento serve nos dois, até os 128 KiB do Linux.
                entrada: EntradaPrompt::Argumento,
            },
            // O "tech lead" (ciclo 521): o Claude Code que LÊ e não edita.
            // No `-p`, `--permission-mode default` nega o que pede
            // permissão — medido em 2026-10-25: pedido pra editar, ele
            // respondeu que a escrita foi negada e o arquivo ficou igual.
            // Quem revisa não conserta: devolve pro executor.
            Adaptador {
                nome: "Claude Code (revisor)".into(),
                binario: "claude".into(),
                args: vec![
                    "-p".into(),
                    "--output-format".into(),
                    "stream-json".into(),
                    "--verbose".into(),
                    "--permission-mode".into(),
                    "default".into(),
                ],
                cwd: String::new(),
                pastas_extras: Vec::new(),
                arg_pasta_extra: "--add-dir".into(),
                arg_mcp: "--mcp-config".into(),
                arg_sessao: String::new(),
                timeout_s: TIMEOUT_PADRAO_S,
                formato: FormatoSaida::StreamJson,
                entrada: EntradaPrompt::Stdin,
            },
        ]
    }

    /// O preset é de quem só lê (o revisor)? É o que separa os perfis que
    /// TÊM que poder escrever dos que não podem.
    pub fn so_le(&self) -> bool {
        self.args.windows(2).any(|w| w[0] == "--permission-mode" && w[1] == "default")
    }
}

/// O consumo que um agente de TEXTO conta no fim da saída (ciclo 524).
///
/// É o formato do aider — medido no 0.86.2: `Tokens: 813 sent, 65
/// received.` com o Ollama, e `Tokens: 1.2k sent, 65 received. Cost:
/// $0.01 message, $0.02 session.` com modelo pago. A ÚLTIMA linha vale (o
/// aider conta a cada mensagem), e o custo é o da mensagem: o da sessão
/// somaria as rodadas anteriores de novo.
pub fn uso_do_texto(saida: &str) -> Option<Uso> {
    fn numero(t: &str) -> Option<u64> {
        let t = t.trim().replace(',', "");
        let (base, mult) = match t.chars().last()? {
            'k' | 'K' => (&t[..t.len() - 1], 1_000.0),
            'm' | 'M' => (&t[..t.len() - 1], 1_000_000.0),
            _ => (t.as_str(), 1.0),
        };
        base.parse::<f64>().ok().map(|n| (n * mult).round() as u64)
    }
    let linha = saida.lines().rev().find(|l| l.trim_start().starts_with("Tokens:"))?;
    let resto = linha.trim_start().strip_prefix("Tokens:")?;
    let (tokens, custo) = match resto.split_once("Cost:") {
        Some((t, c)) => (t, Some(c)),
        None => (resto, None),
    };
    let mut entrada = None;
    let mut saida_t = None;
    for parte in tokens.split(',') {
        let parte = parte.trim().trim_end_matches('.');
        if let Some(n) = parte.strip_suffix(" sent") {
            entrada = numero(n);
        } else if let Some(n) = parte.strip_suffix(" received") {
            saida_t = numero(n);
        }
    }
    let custo_usd = custo
        .and_then(|c| c.split(',').find(|p| p.contains("message")))
        .and_then(|p| p.trim().trim_start_matches('$').split_whitespace().next())
        .and_then(|n| n.trim_start_matches('$').parse::<f64>().ok());
    Some(Uso { entrada: entrada?, saida: saida_t.unwrap_or(0), custo_usd })
}

/// O agente de um papel do projeto (ciclo 521), pelo nome. A mesma regra
/// pra TUI e janela, cada uma com a lista dela (`opcoes`).
///
/// Sem nome, o `padrao` (o agente das conversas). Nome que não existe é
/// ERRO: rodar outro agente no lugar, calado, mandaria a tarefa pro
/// modelo caro que a pessoa quis poupar — ou o revisor que edita pra
/// onde ela pediu um que só lê.
pub fn escolher(nome: Option<&str>, opcoes: &[Adaptador], padrao: &Adaptador) -> Result<Adaptador, String> {
    let Some(nome) = nome.map(str::trim).filter(|n| !n.is_empty()) else {
        return Ok(padrao.clone());
    };
    if padrao.nome == nome {
        return Ok(padrao.clone());
    }
    if let Some(a) = opcoes.iter().find(|a| a.nome == nome) {
        return Ok(a.clone().migrado());
    }
    if let Some(a) = Adaptador::presets().into_iter().find(|a| a.nome == nome) {
        return Ok(a);
    }
    Err(format!("o agente \"{nome}\" da config do projeto não existe nas preferências"))
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

/// As marcas em volta do `PATH` que o shell de login imprime (ciclo 528).
pub const MARCA_DO_PATH: (&str, &str) = ("__ANOTADINHO_PATH__", "__FIM_DO_PATH__");

/// O `PATH` na saída do shell de login, entre as marcas. Um `.zshrc`
/// imprime o que quiser antes e depois (boas-vindas, o prompt, avisos), e
/// só o que está entre as marcas é o `PATH`.
pub fn path_da_saida_do_shell(saida: &str) -> Option<String> {
    let (abre, fecha) = MARCA_DO_PATH;
    let ini = saida.rfind(abre)? + abre.len();
    let fim = saida[ini..].find(fecha)? + ini;
    Some(saida[ini..fim].trim().to_string()).filter(|p| !p.is_empty())
}

/// O `PATH` que o app dá aos processos que sobe (ciclo 528): o do próprio
/// processo, mais o que só o shell de login conhece, mais `~/.local/bin`
/// de reserva — sem repetir pasta, nessa ordem.
///
/// O app aberto pelo menu herda o `PATH` da sessão gráfica
/// (`/usr/local/bin:/usr/bin` no Fedora), e o `claude`, o `codex`, o
/// `aider` e o `opencode` moram em `~/.local/bin` e afins, que só o
/// `.zshrc` põe no `PATH`. O do processo vem primeiro: quem abriu o app
/// pelo terminal continua com o `PATH` dele valendo.
pub fn juntar_path(processo: &str, do_login: Option<&str>, home: Option<&str>, windows: bool) -> String {
    let sep = if windows { ';' } else { ':' };
    let reserva = home.filter(|_| !windows).map(|h| format!("{}/.local/bin", h.trim_end_matches('/')));
    let mut fora: Vec<String> = Vec::new();
    let fontes = [Some(processo), do_login, reserva.as_deref()];
    for parte in fontes.into_iter().flatten().flat_map(|p| p.split(sep)) {
        let parte = parte.trim();
        if !parte.is_empty() && !fora.iter().any(|f| f == parte) {
            fora.push(parte.to_string());
        }
    }
    fora.join(&sep.to_string())
}

/// A mensagem de quando o spawn falha, na língua de quem configura.
///
/// O caso que importa é o do Windows (ciclo 449): prompt com quebra de
/// linha indo como ARGUMENTO pra um shim `.cmd`. A biblioteca padrão
/// recusa com `batch file arguments are invalid`, que não diz nem o que
/// está errado nem o que fazer. Pelo stdin isso não acontece.
pub fn explicar_erro_de_spawn(binario: &str, erro: &std::io::Error, entrada: EntradaPrompt) -> String {
    let e_bat = erro.kind() == std::io::ErrorKind::InvalidInput
        && erro.to_string().contains("batch file arguments are invalid");
    if e_bat && entrada == EntradaPrompt::Argumento {
        format!(
            "\"{binario}\" é um script .cmd/.bat, e o Windows não deixa um script \
             desses receber o prompt (que tem quebras de linha) como argumento. Na \
             configuração do agente, troque a entrega do prompt para a entrada \
             padrão (stdin)"
        )
    } else if erro.kind() == std::io::ErrorKind::NotFound && !binario.contains('/') && !binario.contains('\\') {
        // Ciclo 528: depois de juntar o PATH do shell de login, o que
        // sobra é o programa que não está instalado — ou está num lugar
        // que nem o shell conhece.
        format!(
            "não achei \"{binario}\" nem no PATH do app nem no do seu shell de login ({erro}). \
             Se ele roda no terminal, ponha o caminho completo no agente (o que `command -v {binario}` mostra)"
        )
    } else {
        format!("não consegui executar \"{binario}\": {erro}")
    }
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
            timeout_s: 30,
            formato: FormatoSaida::Texto,
            entrada: EntradaPrompt::Argumento,
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
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
            // O revisor (ciclo 521) é o único que NÃO pode escrever.
            if p.formato != FormatoSaida::StreamJson || p.so_le() {
                continue;
            }
            let linha = p.args.join(" ");
            assert!(
                // `--auto` é o do opencode (ciclo 527).
                linha.contains("workspace-write") || linha.contains("acceptEdits") || linha.contains("--auto"),
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
            timeout_s: 180,
            formato: FormatoSaida::Texto,
            entrada: EntradaPrompt::Argumento,
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
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
            arg_mcp: String::new(),
            arg_sessao: String::new(),
            timeout_s: 180,
            formato: FormatoSaida::Texto,
            entrada: EntradaPrompt::Argumento,
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
    fn le_o_dialeto_do_opencode_medido() {
        // Linhas da saída real do opencode 1.18.14 (2026-10-25), cortadas
        // ao que o leitor usa.
        let linhas = [
            r#"{"type":"step_start","sessionID":"ses_e8bd","part":{"type":"step-start"}}"#,
            r#"{"type":"tool_use","sessionID":"ses_e8bd","part":{"type":"tool","tool":"bash","state":{"status":"completed","input":{"command":"ls -la"}}}}"#,
            r#"{"type":"step_finish","sessionID":"ses_e8bd","part":{"type":"step-finish","reason":"tool-calls","tokens":{"total":11363,"input":9503,"output":61,"reasoning":7,"cache":{"write":0,"read":1792}},"cost":0}}"#,
            r#"{"type":"tool_use","sessionID":"ses_e8bd","part":{"type":"tool","tool":"write","state":{"status":"completed","input":{"filePath":"/r/soma.js"}}}}"#,
            r#"{"type":"text","sessionID":"ses_e8bd","part":{"type":"text","text":"Vou conferir."}}"#,
            r#"{"type":"text","sessionID":"ses_e8bd","part":{"type":"text","text":"Feito e commitado.\n\nRESULTADO: concluida"}}"#,
            r#"{"type":"step_finish","sessionID":"ses_e8bd","part":{"type":"step-finish","reason":"stop","tokens":{"total":12147,"input":311,"output":60,"reasoning":0,"cache":{"write":0,"read":11776}},"cost":0}}"#,
        ];
        let mut l = LeitorStream::novo();
        for x in linhas {
            l.linha(x);
        }
        assert_eq!(l.resposta().unwrap(), "Feito e commitado.\n\nRESULTADO: concluida", "o último recado é a resposta");
        assert_eq!(l.sessao().as_deref(), Some("ses_e8bd"));
        assert_eq!(l.uso(), Some(Uso { entrada: 9503 + 1792 + 311 + 11776, saida: 61 + 7 + 60, custo_usd: Some(0.0) }), "soma os passos");
        assert!(l.progresso().contains("· bash ls -la") && l.progresso().contains("· write /r/soma.js"), "{}", l.progresso());
        // O erro medido: a chave do plano expirada.
        let mut e = LeitorStream::novo();
        e.linha(r#"{"type":"error","sessionID":"s","error":{"name":"APIError","data":{"message":"This API key has expired.","statusCode":401}}}"#);
        assert_eq!(e.resposta(), Err("This API key has expired.".to_string()));
    }

    #[test]
    fn o_opencode_antigo_migra_pro_medido() {
        let velho = Adaptador {
            nome: "opencode".into(),
            binario: "/home/x/.opencode/bin/opencode".into(),
            args: vec!["run".into(), MARCADOR_PROMPT.into()],
            formato: FormatoSaida::Texto,
            entrada: EntradaPrompt::Argumento,
            ..base()
        };
        let novo = velho.migrado();
        assert_eq!((novo.formato, novo.entrada), (FormatoSaida::StreamJson, EntradaPrompt::Stdin));
        assert!(novo.args.iter().any(|a| a == "--auto"));
        assert_eq!(novo.binario, "/home/x/.opencode/bin/opencode", "o caminho da pessoa fica");
        assert!(novo.validar().is_none());
    }

    #[test]
    fn o_uso_do_aider_sai_da_ultima_linha_de_tokens() {
        // A saída real do aider 0.86.2 com o Ollama (2026-10-25).
        let real = "Applied edit to soma.js\n\nRESULTADO: concluída\n\nTokens: 813 sent, 65 received.\nApplied edit to soma.js\nCommit ad13deb refactor";
        assert_eq!(uso_do_texto(real), Some(Uso { entrada: 813, saida: 65, custo_usd: None }));
        let pago = "Tokens: 2.0k sent, 10 received. Cost: $0.10 message, $0.10 session.\n...\nTokens: 12k sent, 1.5k received. Cost: $0.04 message, $0.14 session.";
        assert_eq!(uso_do_texto(pago), Some(Uso { entrada: 12_000, saida: 1_500, custo_usd: Some(0.04) }), "a última, e o custo da mensagem");
        assert_eq!(uso_do_texto("RESPOSTA sem contagem"), None);
    }

    #[test]
    fn o_path_do_shell_sai_entre_as_marcas_mesmo_com_lixo_do_zshrc() {
        let saida = "Bem-vindo!\n\x1b]0;~\x07\n__ANOTADINHO_PATH__/home/e/.local/bin:/usr/bin__FIM_DO_PATH__\n% ";
        assert_eq!(path_da_saida_do_shell(saida).as_deref(), Some("/home/e/.local/bin:/usr/bin"));
        assert_eq!(path_da_saida_do_shell("nada"), None);
        assert_eq!(path_da_saida_do_shell("__ANOTADINHO_PATH____FIM_DO_PATH__"), None, "vazio não é PATH");
    }

    #[test]
    fn juntar_o_path_sem_repetir_e_com_o_do_processo_primeiro() {
        // O medido: o app aberto pelo menu tinha só /usr/local/bin:/usr/bin.
        let j = juntar_path("/usr/local/bin:/usr/bin", Some("/home/e/.local/bin:/usr/bin:/home/e/.opencode/bin"), Some("/home/e"), false);
        assert_eq!(j, "/usr/local/bin:/usr/bin:/home/e/.local/bin:/home/e/.opencode/bin");
        // Sem resposta do shell, a reserva.
        assert_eq!(juntar_path("/usr/bin", None, Some("/home/e/"), false), "/usr/bin:/home/e/.local/bin");
        // No Windows, o separador é `;` e não há reserva.
        assert_eq!(juntar_path(r"C:\a;C:\b", Some(r"C:\b;C:\c"), Some(r"C:\Users\e"), true), r"C:\a;C:\b;C:\c");
    }

    #[test]
    fn o_erro_de_programa_nao_achado_diz_o_que_fazer() {
        let e = std::io::Error::from(std::io::ErrorKind::NotFound);
        let m = explicar_erro_de_spawn("claude", &e, EntradaPrompt::Stdin);
        assert!(m.contains("shell de login") && m.contains("command -v claude"), "{m}");
        assert!(explicar_erro_de_spawn("/opt/x/claude", &e, EntradaPrompt::Stdin).starts_with("não consegui executar"), "caminho completo: o erro do sistema basta");
    }

    #[test]
    fn ligar_o_mcp_do_claude_libera_as_ferramentas_dele() {
        // O relato do ciclo 529: "o acesso às ferramentas do Anotadinho
        // (onde_posso_escrever, ler_pagina) não foi autorizado".
        let claude = Adaptador::default();
        let com = claude.montar_execucao("p", "/v/.anotadinho/mcp.json", "");
        let i = com.args.iter().position(|a| a == "--mcp-config").expect("liga o servidor");
        assert_eq!(com.args[i + 1], "/v/.anotadinho/mcp.json");
        assert!(com.args.contains(&format!("--allowedTools=mcp__{SERVIDOR_MCP}")), "{:?}", com.args);
        // Também no agente copiado do Claude Code, e no revisor.
        let meu = Adaptador { nome: "Sonnet júnior".into(), args: vec!["-p".into(), "--model".into(), "sonnet".into()], ..Adaptador::default() };
        assert!(meu.montar_execucao("p", "/m.json", "").args.iter().any(|a| a.starts_with("--allowedTools=")));
        let revisor = Adaptador::presets().into_iter().find(|a| a.so_le()).unwrap();
        assert!(revisor.montar_execucao("p", "/m.json", "").args.iter().any(|a| a.starts_with("--allowedTools=")));
        // Sem MCP, nada; e quem não fala o dialeto do Claude não recebe a flag dele.
        assert!(!claude.montar_execucao("p", "", "").args.iter().any(|a| a.starts_with("--allowedTools")));
        let outro = Adaptador { arg_mcp: "--mcp".into(), ..base() };
        assert!(!outro.montar_execucao("p", "/m.json", "").args.iter().any(|a| a.starts_with("--allowedTools")));
        assert!(config_mcp("/cli", "/v").contains(&format!("\"{SERVIDOR_MCP}\"")), "o nome liberado é o do servidor");
    }

    #[test]
    fn escolher_pelo_nome_e_nunca_trocar_calado() {
        let padrao = Adaptador::default();
        let meu = Adaptador { nome: "meu júnior".into(), binario: "aider".into(), ..base() };
        let opcoes = vec![meu.clone()];
        assert_eq!(escolher(None, &opcoes, &padrao).unwrap().nome, padrao.nome);
        assert_eq!(escolher(Some("  "), &opcoes, &padrao).unwrap().nome, padrao.nome, "em branco é o padrão");
        assert_eq!(escolher(Some("meu júnior"), &opcoes, &padrao).unwrap().binario, "aider");
        assert!(escolher(Some("Claude Code (revisor)"), &[], &padrao).unwrap().so_le(), "o preset vale sem estar nas opções");
        let erro = escolher(Some("sumiu"), &opcoes, &padrao).unwrap_err();
        assert!(erro.contains("\"sumiu\""), "{erro}");
    }

    #[test]
    fn o_revisor_so_le_e_o_aider_escreve_e_fecha_com_resultado() {
        let presets = Adaptador::presets();
        let achar = |n: &str| presets.iter().find(|p| p.nome == n).unwrap().clone();
        let revisor = achar("Claude Code (revisor)");
        assert!(revisor.so_le());
        assert!(!revisor.args.iter().any(|a| a == "acceptEdits"));
        assert!(!achar("Claude Code").so_le(), "o executor padrão continua escrevendo");
        let aider = achar("aider + Ollama (local)");
        assert!(aider.validar().is_none());
        assert!(aider.args.iter().any(|a| a == "--yes-always"), "sem terminal, pergunta é trava");
        assert_eq!(aider.args.last().map(String::as_str), Some(MARCADOR_PROMPT));
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
    uso: Option<Uso>,
    sessao: Option<String>,
}

/// O que a execução consumiu, quando o agente conta (ciclo 422).
///
/// Quem paga por token precisa do mostrador: duração não é preço, e uma
/// conversa barata e uma cara levam o mesmo tempo. Nem todo agente
/// informa — daí ser `Option` em tudo que carrega isto.
#[derive(Debug, Clone, Copy, PartialEq, Default, serde::Serialize, serde::Deserialize)]
pub struct Uso {
    /// Tokens que entraram (prompt + contexto).
    #[serde(default)]
    pub entrada: u64,
    /// Tokens que saíram.
    #[serde(default)]
    pub saida: u64,
    /// Custo em dólares, quando o agente calcula.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub custo_usd: Option<f64>,
}

impl Uso {
    /// Como se lê numa linha: `12k↓ 800↑ · US$ 0,04`.
    pub fn rotulo(&self) -> String {
        let curto = |n: u64| crate::orcamento::humano(n as usize);
        let mut s = format!("{}↓ {}↑", curto(self.entrada), curto(self.saida));
        if let Some(c) = self.custo_usd {
            s.push_str(&format!(" · US$ {:.2}", c).replace('.', ","));
        }
        s
    }

    /// Soma de duas execuções — o total do dia.
    pub fn somar(&self, outro: &Uso) -> Uso {
        Uso {
            entrada: self.entrada + outro.entrada,
            saida: self.saida + outro.saida,
            custo_usd: match (self.custo_usd, outro.custo_usd) {
                (None, None) => None,
                (a, b) => Some(a.unwrap_or(0.0) + b.unwrap_or(0.0)),
            },
        }
    }
}

impl LeitorStream {
    pub fn novo() -> Self {
        Self::default()
    }

    /// O que a execução consumiu, se o agente contou.
    pub fn uso(&self) -> Option<Uso> {
        self.uso
    }

    /// O identificador da sessão, quando o agente tem uma (ciclo 433).
    ///
    /// É o que permite CONTINUAR a conversa na próxima pergunta em vez
    /// de remontar o histórico inteiro a cada envio — mais barato e mais
    /// rápido, e o agente mantém o que já entendeu.
    pub fn sessao(&self) -> Option<String> {
        self.sessao.clone()
    }

    /// Guarda o id de sessão dos dois dialetos, se vier.
    fn anotar_sessao(&mut self, v: &serde_json::Value) {
        // `sessionID` é o do opencode (ciclo 527).
        for campo in ["session_id", "thread_id", "sessionID"] {
            if let Some(id) = v.get(campo).and_then(|x| x.as_str()) {
                if !id.trim().is_empty() {
                    self.sessao = Some(id.to_string());
                    return;
                }
            }
        }
    }

    /// Lê o bloco de uso dos dois dialetos. Campo ausente vira zero, não
    /// erro: contar errado é pior que não contar, mas perder a resposta
    /// por causa da contabilidade seria pior ainda.
    fn anotar_uso(&mut self, usage: Option<&serde_json::Value>, custo: Option<&serde_json::Value>) {
        let Some(u) = usage else { return };
        let campo = |nomes: [&str; 2]| -> u64 {
            nomes.iter().find_map(|n| u.get(*n).and_then(|v| v.as_u64())).unwrap_or(0)
        };
        let novo = Uso {
            entrada: campo(["input_tokens", "prompt_tokens"]),
            saida: campo(["output_tokens", "completion_tokens"]),
            custo_usd: custo.and_then(|c| c.as_f64()),
        };
        if novo != Uso::default() {
            self.uso = Some(novo);
        }
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
                self.anotar_sessao(&v);
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
                self.anotar_sessao(&v);
                self.anotar_uso(v.get("usage"), v.get("total_cost_usd"));
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
            Some("thread.started") => {
                self.anotar_sessao(&v);
                self.anotar("conectado");
            }
            Some("turn.completed") => self.anotar_uso(v.get("usage"), v.get("total_cost_usd")),
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
                    // O opencode (ciclo 527) põe a mensagem em
                    // `error.data.message` — a chave expirada, por exemplo.
                    let m = v.get("message").and_then(|m| m.as_str()).or_else(|| {
                        let e = v.get("error")?;
                        e.get("data").and_then(|d| d.get("message")).or_else(|| e.get("message")).and_then(|m| m.as_str())
                    });
                    if let Some(m) = m {
                        self.erro = Some(m.to_string());
                    }
                }
            }
            // ── dialeto do opencode (ciclo 527, medido no 1.18.14) ──
            Some("step_start") => {
                if self.sessao.is_none() {
                    self.anotar("conectado");
                }
                self.anotar_sessao(&v);
            }
            Some("text") => {
                if let Some(txt) = v.get("part").and_then(|p| p.get("text")).and_then(|t| t.as_str()) {
                    self.texto.push_str(txt);
                    // O último recado é a resposta, como no Codex: o que
                    // vem antes das ferramentas é narração.
                    self.resultado = Some(txt.to_string());
                    self.anotar(txt);
                }
            }
            Some("tool_use") => {
                let parte = v.get("part");
                let ferramenta = parte.and_then(|p| p.get("tool")).and_then(|t| t.as_str()).unwrap_or("ferramenta");
                let entrada = parte.and_then(|p| p.get("state")).and_then(|s| s.get("input"));
                let alvo = entrada
                    .and_then(|i| i.get("command").or_else(|| i.get("filePath")))
                    .and_then(|c| c.as_str())
                    .unwrap_or("");
                self.anotar(&format!("· {ferramenta} {alvo}"));
            }
            Some("step_finish") => {
                // Cada passo conta o SEU consumo: soma. A entrada inclui o
                // que veio do cache — foi mandado do mesmo jeito.
                let parte = v.get("part");
                let tokens = parte.and_then(|p| p.get("tokens"));
                let n = |caminho: &[&str]| -> u64 {
                    let mut x = tokens;
                    for c in caminho {
                        x = x.and_then(|v| v.get(*c));
                    }
                    x.and_then(|v| v.as_u64()).unwrap_or(0)
                };
                let passo = Uso {
                    entrada: n(&["input"]) + n(&["cache", "read"]) + n(&["cache", "write"]),
                    saida: n(&["output"]) + n(&["reasoning"]),
                    custo_usd: parte.and_then(|p| p.get("cost")).and_then(|c| c.as_f64()),
                };
                if passo != Uso::default() {
                    self.uso = Some(match self.uso {
                        Some(u) => u.somar(&passo),
                        None => passo,
                    });
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
        a.entrada = EntradaPrompt::Argumento;
        a.args = vec!["-p".into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::SemMarcador));
        a.args = vec!["{prompt}".into(), "{prompt}".into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::MarcadorRepetido));
    }

    // --- Ciclo 422: o que a execução consumiu ---------------------------------------

    #[test]
    fn o_uso_vem_do_result_do_claude_code() {
        let mut l = LeitorStream::novo();
        assert_eq!(l.uso(), None, "sem o campo, não se inventa número");
        l.linha(r#"{"type":"assistant","message":{"content":[{"type":"text","text":"pronto"}]}}"#);
        l.linha(
            r#"{"type":"result","result":"pronto","usage":{"input_tokens":12000,"output_tokens":800},"total_cost_usd":0.0432}"#,
        );
        let u = l.uso().expect("o result trouxe o uso");
        assert_eq!((u.entrada, u.saida), (12000, 800));
        assert_eq!(u.custo_usd, Some(0.0432));
        assert_eq!(u.rotulo(), "12k↓ 800↑ · US$ 0,04");
    }

    #[test]
    fn o_uso_vem_do_turn_completed_do_codex() {
        let mut l = LeitorStream::novo();
        l.linha(r#"{"type":"turn.completed","usage":{"input_tokens":300,"output_tokens":90}}"#);
        let u = l.uso().expect("o turn.completed trouxe o uso");
        assert_eq!((u.entrada, u.saida), (300, 90));
        assert_eq!(u.custo_usd, None, "o Codex não diz preço");
        assert_eq!(u.rotulo(), "300↓ 90↑");
        // Bloco vazio não vira zero fingindo que contou.
        let mut vazio = LeitorStream::novo();
        vazio.linha(r#"{"type":"turn.completed","usage":{}}"#);
        assert_eq!(vazio.uso(), None);
    }

    #[test]
    fn somar_junta_o_dia() {
        let a = Uso { entrada: 100, saida: 10, custo_usd: Some(0.01) };
        let b = Uso { entrada: 200, saida: 20, custo_usd: None };
        let total = a.somar(&b);
        assert_eq!((total.entrada, total.saida), (300, 30));
        assert_eq!(total.custo_usd, Some(0.01), "quem não diz preço não zera o do outro");
        assert_eq!(Uso::default().somar(&Uso::default()).custo_usd, None);
    }

    // --- Ciclo 426: ligar o MCP na execução ------------------------------------------

    #[test]
    fn a_config_mcp_aponta_pro_cli_e_pro_vault() {
        let json = config_mcp("/usr/bin/anotadinho-cli", "/home/eu/vault");
        let v: serde_json::Value = serde_json::from_str(&json).expect("JSON válido");
        let servidor = &v["mcpServers"]["anotadinho"];
        assert_eq!(servidor["command"], "/usr/bin/anotadinho-cli");
        assert_eq!(servidor["args"], serde_json::json!(["--vault", "/home/eu/vault", "mcp"]));
        // Caminho com aspas ou barra invertida não quebra o arquivo.
        let json = config_mcp("C:\\bin\\cli.exe", "/tmp/o \"meu\" vault");
        let v: serde_json::Value = serde_json::from_str(&json).expect("JSON válido mesmo com aspas");
        assert_eq!(v["mcpServers"]["anotadinho"]["args"][1], "/tmp/o \"meu\" vault");
    }

    #[test]
    fn o_mcp_entra_nos_args_so_quando_o_agente_sabe_receber() {
        let claude = pelo_argumento(Adaptador::presets().into_iter().find(|a| a.nome == "Claude Code").unwrap());
        let com = claude.montar_args_com_mcp("oi", "/tmp/mcp.json");
        assert!(com.windows(2).any(|p| p[0] == "--mcp-config" && p[1] == "/tmp/mcp.json"), "{com:?}");
        // Antes do prompt: o que vem depois dele alguns agentes tratam
        // como parte do prompt.
        let i_mcp = com.iter().position(|a| a == "--mcp-config").unwrap();
        let i_prompt = com.iter().position(|a| a == "oi").unwrap();
        assert!(i_mcp < i_prompt, "{com:?}");
        // Sem caminho, ou num agente que não sabe receber, nada muda.
        assert_eq!(claude.montar_args_com_mcp("oi", ""), claude.montar_args("oi"));
        let codex = Adaptador::presets().into_iter().find(|a| a.nome == "Codex").unwrap();
        assert_eq!(codex.montar_args_com_mcp("oi", "/tmp/mcp.json"), codex.montar_args("oi"));
    }

    #[test]
    fn a_sessao_sai_dos_dois_dialetos() {
        let mut l = LeitorStream::novo();
        assert_eq!(l.sessao(), None);
        l.linha(r#"{"type":"system","subtype":"init","session_id":"abc-123"}"#);
        assert_eq!(l.sessao(), Some("abc-123".into()));
        // O `result` também traz, e vale o mais recente.
        l.linha(r#"{"type":"result","result":"ok","session_id":"abc-123"}"#);
        assert_eq!(l.sessao(), Some("abc-123".into()));
        // Codex: `thread_id` no `thread.started`.
        let mut c = LeitorStream::novo();
        c.linha(r#"{"type":"thread.started","thread_id":"t-9"}"#);
        assert_eq!(c.sessao(), Some("t-9".into()));
        // Campo vazio não vira sessão.
        let mut v = LeitorStream::novo();
        v.linha(r#"{"type":"system","subtype":"init","session_id":""}"#);
        assert_eq!(v.sessao(), None);
    }

    #[test]
    fn o_resume_entra_antes_do_prompt_e_so_com_id() {
        let claude = pelo_argumento(Adaptador::presets().into_iter().find(|a| a.nome == "Claude Code").unwrap());
        let com = claude.montar_args_completo("oi", "", "abc-123");
        assert!(com.windows(2).any(|p| p[0] == "--resume" && p[1] == "abc-123"), "{com:?}");
        let i_resume = com.iter().position(|a| a == "--resume").unwrap();
        let i_prompt = com.iter().position(|a| a == "oi").unwrap();
        assert!(i_resume < i_prompt, "{com:?}");
        // Sem id, ou num agente que não sabe continuar, nada muda.
        assert_eq!(claude.montar_args_completo("oi", "", ""), claude.montar_args("oi"));
        let codex = Adaptador::presets().into_iter().find(|a| a.nome == "Codex").unwrap();
        assert_eq!(codex.montar_args_completo("oi", "", "abc"), codex.montar_args("oi"));
    }

    /// O perfil do jeito de antes do ciclo 449: prompt no último
    /// argumento. É onde a ORDEM dos argumentos ainda importa.
    fn pelo_argumento(mut a: Adaptador) -> Adaptador {
        a.args.retain(|x| x != "-");
        a.args.push(MARCADOR_PROMPT.into());
        a.entrada = EntradaPrompt::Argumento;
        a
    }

    // --- Ciclo 449: o prompt pelo stdin -----------------------------------------------

    #[test]
    fn os_perfis_do_claude_e_do_codex_mandam_o_prompt_pelo_stdin() {
        let prompt = "linha 1\nlinha 2";
        for nome in ["Claude Code", "Codex"] {
            let a = Adaptador::presets().into_iter().find(|a| a.nome == nome).unwrap();
            assert_eq!(a.validar(), None, "{nome}");
            let e = a.montar_execucao(prompt, "/tmp/mcp.json", "s-1");
            assert_eq!(e.entrada.as_deref(), Some(prompt), "{nome}");
            assert!(!e.args.iter().any(|x| x.contains("linha")), "{nome}: prompt nos args {:?}", e.args);
            assert!(!e.args.iter().any(|x| x.contains(MARCADOR_PROMPT)), "{nome}: marcador sobrou {:?}", e.args);
        }
        // O Codex só lê o stdin com o `-` (ou sem posicional nenhum).
        let codex = Adaptador::presets().into_iter().find(|a| a.nome == "Codex").unwrap();
        assert_eq!(codex.montar_execucao("x", "", "").args.last().map(String::as_str), Some("-"));
    }

    #[test]
    fn pelo_argumento_nada_vai_pro_stdin() {
        let a = pelo_argumento(Adaptador::default());
        let e = a.montar_execucao("oi\ntchau", "", "");
        assert_eq!(e.entrada, None);
        assert_eq!(e.args.last().map(String::as_str), Some("oi\ntchau"));
    }

    #[test]
    fn pelo_stdin_o_marcador_e_opcional_mas_nunca_embutido() {
        let mut a = com_binario("agente");
        a.entrada = EntradaPrompt::Stdin;
        a.args = vec!["run".into()];
        assert_eq!(a.validar(), None);
        a.args = vec!["run".into(), MARCADOR_PROMPT.into()];
        assert_eq!(a.validar(), None);
        assert_eq!(a.montar_execucao("p", "", "").args, vec!["run".to_string()]);
        a.args = vec![format!("--msg={MARCADOR_PROMPT}")];
        assert_eq!(a.validar(), Some(ProblemaConfig::MarcadorEmbutidoComStdin));
        a.args = vec![MARCADOR_PROMPT.into(), MARCADOR_PROMPT.into()];
        assert_eq!(a.validar(), Some(ProblemaConfig::MarcadorRepetido));
    }

    #[test]
    fn configuracao_antiga_sem_o_campo_continua_pelo_argumento() {
        let json = r#"{"nome":"meu","binario":"x","args":["{prompt}"]}"#;
        let a: Adaptador = serde_json::from_str(json).unwrap();
        assert_eq!(a.entrada, EntradaPrompt::Argumento);
    }

    #[test]
    fn o_perfil_pronto_gravado_migra_pro_stdin_e_o_montado_a_mao_nao() {
        for nome in ["Claude Code", "Codex"] {
            let atual = Adaptador::presets().into_iter().find(|a| a.nome == nome).unwrap();
            let gravado = pelo_argumento(atual.clone());
            let migrado = gravado.migrado();
            assert_eq!(migrado.entrada, EntradaPrompt::Stdin, "{nome}");
            assert_eq!(migrado.args, atual.args, "{nome}");
        }
        let mut meu = pelo_argumento(Adaptador::default());
        meu.args.insert(0, "--model".into());
        meu.args.insert(1, "opus".into());
        let depois = meu.clone().migrado();
        assert_eq!(depois.entrada, EntradaPrompt::Argumento);
        assert_eq!(depois.args, meu.args);
    }

    #[test]
    fn o_erro_do_cmd_com_quebra_de_linha_diz_o_que_fazer() {
        let erro = std::io::Error::new(std::io::ErrorKind::InvalidInput, "batch file arguments are invalid");
        let m = explicar_erro_de_spawn("claude", &erro, EntradaPrompt::Argumento);
        assert!(m.contains("stdin") && m.contains(".cmd"), "{m}");
        assert!(!m.contains("batch file"), "{m}");
        // O "não achado" tem mensagem própria desde o 528; o resto segue cru.
        let outro = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "permission denied");
        assert_eq!(
            explicar_erro_de_spawn("claude", &outro, EntradaPrompt::Argumento),
            "não consegui executar \"claude\": permission denied"
        );
    }

    // --- Ciclo 450: o CLI do MCP, ou nenhum -------------------------------------------

    fn disco<'a>(arquivos: &'a [&'a str]) -> impl Fn(&std::path::Path) -> bool + 'a {
        move |p| arquivos.iter().any(|a| std::path::Path::new(a) == p)
    }

    #[test]
    fn o_cli_ao_lado_do_app_ganha_do_path() {
        let achado = localizar_cli(
            Some(std::path::Path::new("/usr/bin/anotadinho")),
            "/home/ana/.cargo/bin",
            false,
            disco(&["/usr/bin/anotadinho-cli", "/home/ana/.cargo/bin/anotadinho-cli"]),
        );
        assert_eq!(achado.as_deref(), Some("/usr/bin/anotadinho-cli"));
    }

    #[test]
    fn no_windows_o_cli_instalado_e_o_exe_vizinho() {
        // O instalador põe o `externalBin` na pasta do app.
        let achado = localizar_cli(
            Some(std::path::Path::new(r"C:\Program Files\Anotadinho\anotadinho.exe")),
            "",
            true,
            disco(&[r"C:\Program Files\Anotadinho\anotadinho-cli.exe"]),
        );
        assert_eq!(achado.as_deref(), Some(r"C:\Program Files\Anotadinho\anotadinho-cli.exe"));
    }

    #[test]
    fn sem_vizinho_procura_no_path_e_sem_nenhum_diz_que_nao_achou() {
        let pelo_path = localizar_cli(None, r"C:\x;C:\Users\ana\.cargo\bin", true, disco(&[r"C:\Users\ana\.cargo\bin\anotadinho-cli.exe"]));
        assert_eq!(pelo_path.as_deref(), Some(r"C:\Users\ana\.cargo\bin\anotadinho-cli.exe"));
        assert_eq!(localizar_cli(Some(std::path::Path::new("/opt/a/anotadinho")), "/usr/bin:/bin", false, disco(&[])), None);
    }
}
