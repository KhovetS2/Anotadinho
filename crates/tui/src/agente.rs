//! O agente das conversas, rodando pela TUI (ciclo 340).
//!
//! Na janela quem roda o agente é o backend do Tauri. A TUI não tem esse
//! processo, então roda ela mesma, com as MESMAS peças do núcleo: o
//! [`Adaptador`] monta os argumentos (sem shell; o prompt vai pelo stdin
//! ou como um argumento só, ciclo 449), o [`LeitorStream`] lê a saída em stream do Claude Code, e
//! `raiz_do_projeto` decide a pasta quando a configuração não diz.
//!
//! Um [`Trabalho`] é um processo filho numa thread: a tela pergunta
//! quanto tempo passou e o que já saiu, e quando ele acaba recebe a
//! resposta (ou o erro) UMA vez.

use std::io::{BufRead, BufReader, Read};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use anotadinho_core::agente::{Adaptador, FormatoSaida, LeitorStream};

/// Uma execução do agente.
pub struct Trabalho {
    inicio: Instant,
    parcial: Arc<Mutex<String>>,
    /// O que a execução consumiu, quando o agente conta (ciclo 422).
    uso: Arc<Mutex<Option<anotadinho_core::agente::Uso>>>,
    /// A sessão que o agente abriu, pra continuar na próxima (433).
    sessao: Arc<Mutex<Option<String>>>,
    fim: Arc<Mutex<Option<Result<String, String>>>>,
    cancelado: Arc<AtomicBool>,
    entregue: bool,
    /// Uma linha fixa no topo do progresso (ciclo 450): o motivo de o
    /// agente estar rodando sem alguma coisa, que a saída dele não diz.
    aviso: Option<String>,
}

impl Trabalho {
    /// Dispara o agente com o prompt, trabalhando em `cwd`.
    pub fn iniciar(
        adaptador: &Adaptador,
        prompt: &str,
        cwd: &str,
        config_mcp: &str,
        sessao: &str,
    ) -> Result<Self, String> {
        if let Some(problema) = adaptador.validar() {
            return Err(format!("configuração do agente inválida: {}", problema.mensagem()));
        }
        let anotadinho_core::agente::Execucao { args, entrada } = adaptador.montar_execucao(prompt, config_mcp, sessao);
        let modo_de_entrada = adaptador.entrada;
        let executavel = anotadinho_core::agente::executavel_para_spawn(&adaptador.binario);
        let binario = adaptador.binario.clone();
        let limite = adaptador.timeout_s;
        let formato = adaptador.formato;
        let mut comando = Command::new(&executavel);
        // O mesmo preparo da janela: sem ele, interromper mata só o
        // wrapper, e o neto Node segue rodando — no Windows, sempre.
        anotadinho_vault::processo::preparar(&mut comando);
        let mut filho: Child = comando
            .args(&args)
            .current_dir(cwd)
            .stdin(anotadinho_vault::processo::stdin_para(&entrada))
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| anotadinho_core::agente::explicar_erro_de_spawn(&binario, &e, modo_de_entrada))?;
        anotadinho_vault::processo::entregar_entrada(&mut filho, entrada);

        let parcial = Arc::new(Mutex::new(String::new()));
        let uso: Arc<Mutex<Option<anotadinho_core::agente::Uso>>> = Arc::new(Mutex::new(None));
        let sessao_aberta: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));
        let fim = Arc::new(Mutex::new(None));
        let cancelado = Arc::new(AtomicBool::new(false));

        let saida = filho.stdout.take();
        let acumulado = parcial.clone();
        let uso_lido = uso.clone();
        let sessao_lida = sessao_aberta.clone();
        let leitor = std::thread::spawn(move || -> Result<String, String> {
            let Some(saida) = saida else { return Err("não consegui ler a saída do agente".into()) };
            let mut stream = LeitorStream::novo();
            let mut bruto = String::new();
            for linha in BufReader::new(saida).lines().map_while(Result::ok) {
                match formato {
                    FormatoSaida::StreamJson => {
                        stream.linha(&linha);
                        if let Ok(mut p) = acumulado.lock() {
                            *p = stream.progresso();
                        }
                    }
                    FormatoSaida::Texto => {
                        bruto.push_str(&linha);
                        bruto.push('\n');
                        if let Ok(mut p) = acumulado.lock() {
                            p.push_str(&linha);
                            p.push('\n');
                        }
                    }
                }
            }
            match formato {
                FormatoSaida::StreamJson => {
                    if let (Ok(mut u), Some(lido)) = (uso_lido.lock(), stream.uso()) {
                        *u = Some(lido);
                    }
                    if let (Ok(mut s), Some(id)) = (sessao_lida.lock(), stream.sessao()) {
                        *s = Some(id);
                    }
                    stream.resposta()
                }
                FormatoSaida::Texto => {
                    // O agente de texto que conta o consumo (o aider,
                    // ciclo 524) também entra no registro.
                    if let (Ok(mut u), Some(lido)) = (uso_lido.lock(), anotadinho_core::agente::uso_do_texto(&bruto)) {
                        *u = Some(lido);
                    }
                    Ok(bruto)
                }
            }
        });
        let mut erro = filho.stderr.take();
        let leitor_de_erro = std::thread::spawn(move || {
            let mut s = String::new();
            if let Some(e) = erro.as_mut() {
                let _ = e.read_to_string(&mut s);
            }
            s
        });

        let (fim_t, cancelado_t) = (fim.clone(), cancelado.clone());
        std::thread::spawn(move || {
            let inicio = Instant::now();
            let resultado = loop {
                if cancelado_t.load(Ordering::Relaxed) {
                    anotadinho_vault::processo::encerrar_arvore(&mut filho);
                    let _ = filho.wait();
                    break Err("execução interrompida por você".to_string());
                }
                match filho.try_wait() {
                    Ok(Some(status)) => {
                        let lido = leitor.join().unwrap_or_else(|_| Err("a leitura da saída caiu".into()));
                        let stderr = leitor_de_erro.join().unwrap_or_default().trim().to_string();
                        break if status.success() {
                            lido.map(|s| s.trim().to_string()).and_then(|s| {
                                if !s.is_empty() {
                                    Ok(s)
                                } else if stderr.is_empty() {
                                    Err("o agente terminou sem escrever nada na saída".into())
                                } else {
                                    Err(format!("o agente terminou sem resposta. Ele disse: {}", ultimas_linhas(&stderr, 6)))
                                }
                            })
                        } else {
                            let detalhe = match lido {
                                Err(e) => e,
                                Ok(s) if !s.trim().is_empty() => s,
                                Ok(_) if !stderr.is_empty() => stderr,
                                Ok(_) => format!("terminou com código {status}, sem dizer por quê"),
                            };
                            Err(format!("o agente falhou: {}", ultimas_linhas(&detalhe, 6)))
                        };
                    }
                    Ok(None) => {
                        if limite > 0 && inicio.elapsed().as_secs() >= limite {
                            anotadinho_vault::processo::encerrar_arvore(&mut filho);
                            let _ = filho.wait();
                            break Err(format!("o agente passou de {} min e foi interrompido", limite / 60));
                        }
                        std::thread::sleep(std::time::Duration::from_millis(150));
                    }
                    Err(e) => break Err(format!("erro esperando o agente: {e}")),
                }
            };
            if let Ok(mut f) = fim_t.lock() {
                *f = Some(resultado);
            }
        });

        Ok(Self { inicio: Instant::now(), parcial, uso, sessao: sessao_aberta, fim, cancelado, entregue: false, aviso: None })
    }

    /// Há quantos segundos está rodando.
    pub fn segundos(&self) -> u64 {
        self.inicio.elapsed().as_secs()
    }

    /// Fixa um aviso no topo do progresso (ciclo 450).
    pub fn com_aviso(mut self, aviso: Option<String>) -> Self {
        self.aviso = aviso;
        self
    }

    /// O que o agente já escreveu (ou o progresso, no stream), com o
    /// aviso fixo por cima.
    pub fn parcial(&self) -> String {
        let p = self.parcial.lock().map(|p| p.clone()).unwrap_or_default();
        match &self.aviso {
            Some(a) => format!("{a}\n{p}"),
            None => p,
        }
    }

    /// O que a execução consumiu, se o agente contou (ciclo 422).
    pub fn uso(&self) -> Option<anotadinho_core::agente::Uso> {
        self.uso.lock().ok().and_then(|u| *u)
    }

    /// A sessão aberta pelo agente, pra continuar na próxima (433).
    pub fn sessao(&self) -> Option<String> {
        self.sessao.lock().ok().and_then(|s| s.clone())
    }

    /// Pede pra parar.
    pub fn interromper(&self) {
        self.cancelado.store(true, Ordering::Relaxed);
    }

    /// A resposta, na primeira vez que se pergunta depois de acabar.
    pub fn terminou(&mut self) -> Option<Result<String, String>> {
        if self.entregue {
            return None;
        }
        let pronto = self.fim.lock().ok()?.take();
        if pronto.is_some() {
            self.entregue = true;
        }
        pronto
    }
}

fn ultimas_linhas(texto: &str, n: usize) -> String {
    let linhas: Vec<&str> = texto.lines().filter(|l| !l.trim().is_empty()).collect();
    linhas[linhas.len().saturating_sub(n)..].join("\n")
}

#[cfg(test)]
mod testes {
    use super::*;

    fn agente(binario: &str, args: &[&str]) -> Adaptador {
        Adaptador {
            nome: "teste".into(),
            binario: binario.into(),
            args: args.iter().map(|a| a.to_string()).collect(),
            formato: FormatoSaida::Texto,
            timeout_s: 20,
            entrada: anotadinho_core::agente::EntradaPrompt::Argumento,
            ..Adaptador::default()
        }
    }

    fn esperar(t: &mut Trabalho) -> Result<String, String> {
        for _ in 0..200 {
            if let Some(r) = t.terminou() {
                return r;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        panic!("o agente não terminou");
    }

    #[test]
    fn a_resposta_e_a_saida_e_o_prompt_vai_inteiro() {
        let mut t = Trabalho::iniciar(&agente("echo", &["{prompt}"]), "olá \"mundo\" $(x)", ".", "", "").unwrap();
        assert_eq!(esperar(&mut t), Ok("olá \"mundo\" $(x)".into()));
        assert_eq!(t.terminou(), None, "a resposta é entregue uma vez só");
    }

    /// Ciclo 450: o aviso fica por cima do que o agente escreve, do
    /// começo ao fim — a saída dele não sabe que faltou o MCP.
    #[test]
    fn o_aviso_fica_no_topo_do_progresso() {
        let t = Trabalho::iniciar(&agente("sh", &["-c", "echo linha; sleep 5", "{prompt}"]), "x", ".", "", "")
            .unwrap()
            .com_aviso(Some("sem as ferramentas".into()));
        for _ in 0..100 {
            if t.parcial().contains("linha") {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        assert_eq!(t.parcial(), "sem as ferramentas\nlinha\n");
        t.interromper();
        let sem = Trabalho::iniciar(&agente("true", &["{prompt}"]), "x", ".", "", "").unwrap();
        assert!(!sem.parcial().contains("sem as ferramentas"));
    }

    #[test]
    fn falha_e_interrupcao_viram_erro() {
        let mut t = Trabalho::iniciar(&agente("false", &["{prompt}"]), "x", ".", "", "").unwrap();
        assert!(esperar(&mut t).unwrap_err().contains("falhou"));
        let mut t = Trabalho::iniciar(&agente("sleep", &["{prompt}"]), "30", ".", "", "").unwrap();
        t.interromper();
        assert!(esperar(&mut t).unwrap_err().contains("interrompida"));
        assert!(Trabalho::iniciar(&agente("nao-existe-mesmo-xyz", &["{prompt}"]), "x", ".", "", "").is_err());
    }
}

#[cfg(test)]
mod testes_de_uso {
    use super::*;

    /// Ciclo 422: o que o agente contou chega ao `Trabalho` — é o que o
    /// registro de execuções guarda.
    #[test]
    fn o_uso_do_stream_chega_ao_trabalho() {
        let falso = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/uitest/agente-falso.sh");
        let adaptador = Adaptador {
            nome: "falso".into(),
            binario: falso.into(),
            args: vec!["--stream".into(), "{prompt}".into()],
            formato: FormatoSaida::StreamJson,
            timeout_s: 30,
            entrada: anotadinho_core::agente::EntradaPrompt::Argumento,
            ..Adaptador::default()
        };
        let mut t = Trabalho::iniciar(&adaptador, "oi", ".", "", "").expect("subiu");
        for _ in 0..300 {
            if t.terminou().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let u = t.uso().expect("o stream trouxe o uso");
        assert_eq!((u.entrada, u.saida), (12_000, 800));
        assert_eq!(u.custo_usd, Some(0.0432));
    }

    /// Ciclo 524: o agente de TEXTO que conta o consumo (o aider) também.
    #[test]
    fn o_uso_do_aider_chega_ao_trabalho() {
        let falso = concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/uitest/agente-falso.sh");
        let adaptador = Adaptador {
            nome: "falso".into(),
            binario: falso.into(),
            args: vec!["--aider".into(), "{prompt}".into()],
            formato: FormatoSaida::Texto,
            timeout_s: 30,
            entrada: anotadinho_core::agente::EntradaPrompt::Argumento,
            ..Adaptador::default()
        };
        let mut t = Trabalho::iniciar(&adaptador, "oi", ".", "", "").expect("subiu");
        for _ in 0..300 {
            if t.terminou().is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let u = t.uso().expect("a linha de tokens do aider virou uso");
        assert_eq!((u.entrada, u.saida, u.custo_usd), (813, 65, None));
    }
}
