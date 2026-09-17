//! O agente das conversas, rodando pela TUI (ciclo 340).
//!
//! Na janela quem roda o agente é o backend do Tauri. A TUI não tem esse
//! processo, então roda ela mesma, com as MESMAS peças do núcleo: o
//! [`Adaptador`] monta os argumentos (o prompt como um argumento só, sem
//! shell), o [`LeitorStream`] lê a saída em stream do Claude Code, e
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
    fim: Arc<Mutex<Option<Result<String, String>>>>,
    cancelado: Arc<AtomicBool>,
    entregue: bool,
}

impl Trabalho {
    /// Dispara o agente com o prompt, trabalhando em `cwd`.
    pub fn iniciar(adaptador: &Adaptador, prompt: &str, cwd: &str) -> Result<Self, String> {
        if let Some(problema) = adaptador.validar() {
            return Err(format!("configuração do agente inválida: {}", problema.mensagem()));
        }
        let args = adaptador.montar_args(prompt);
        let executavel = anotadinho_core::agente::executavel_para_spawn(&adaptador.binario);
        let binario = adaptador.binario.clone();
        let limite = adaptador.timeout_s;
        let formato = adaptador.formato;
        let mut filho: Child = Command::new(&executavel)
            .args(&args)
            .current_dir(cwd)
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("não consegui executar \"{binario}\": {e}"))?;

        let parcial = Arc::new(Mutex::new(String::new()));
        let fim = Arc::new(Mutex::new(None));
        let cancelado = Arc::new(AtomicBool::new(false));

        let saida = filho.stdout.take();
        let acumulado = parcial.clone();
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
                FormatoSaida::StreamJson => stream.resposta(),
                FormatoSaida::Texto => Ok(bruto),
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
                    let _ = filho.kill();
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
                            let _ = filho.kill();
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

        Ok(Self { inicio: Instant::now(), parcial, fim, cancelado, entregue: false })
    }

    /// Há quantos segundos está rodando.
    pub fn segundos(&self) -> u64 {
        self.inicio.elapsed().as_secs()
    }

    /// O que o agente já escreveu (ou o progresso, no stream).
    pub fn parcial(&self) -> String {
        self.parcial.lock().map(|p| p.clone()).unwrap_or_default()
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
        let mut t = Trabalho::iniciar(&agente("echo", &["{prompt}"]), "olá \"mundo\" $(x)", ".").unwrap();
        assert_eq!(esperar(&mut t), Ok("olá \"mundo\" $(x)".into()));
        assert_eq!(t.terminou(), None, "a resposta é entregue uma vez só");
    }

    #[test]
    fn falha_e_interrupcao_viram_erro() {
        let mut t = Trabalho::iniciar(&agente("false", &["{prompt}"]), "x", ".").unwrap();
        assert!(esperar(&mut t).unwrap_err().contains("falhou"));
        let mut t = Trabalho::iniciar(&agente("sleep", &["{prompt}"]), "30", ".").unwrap();
        t.interromper();
        assert!(esperar(&mut t).unwrap_err().contains("interrompida"));
        assert!(Trabalho::iniciar(&agente("nao-existe-mesmo-xyz", &["{prompt}"]), "x", ".").is_err());
    }
}
