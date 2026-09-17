//! A suíte de avaliação do agente (ciclo 413).
//!
//! Roda cada tarefa de `core::avaliacao` num vault temporário e julga o
//! resultado. O que este arquivo faz é só o que precisa de mundo:
//! montar o vault, chamar o binário do agente, recolher o que sobrou.
//! Quem decide se passou é o núcleo.
//!
//! O vault é NOVO a cada tarefa, num diretório temporário: uma tarefa
//! não pode ver a proposta que a anterior deixou, e nada disso toca o
//! vault de ninguém.
//!
//! # Como o agente escreve
//!
//! Ele não escreve página: propõe. O caminho é o mesmo de sempre — o
//! próprio `anotadinho-cli propor`, ou o servidor MCP (ciclo 205/407) —,
//! e por isso a suíte mede o ARRANJO de verdade, com permissões (405) e
//! validação de proposta (204) no meio do caminho. O prompt recebe o
//! contrato por escrito e o caminho do vault, que é o que um agente
//! precisa pra agir sem adivinhar.

use anotadinho_core::agente::Adaptador;
use anotadinho_core::avaliacao::{julgar, Observado, Tarefa, Veredito};

/// Monta o vault da tarefa num diretório e devolve a raiz.
fn preparar(tarefa: &Tarefa, raiz: &std::path::Path) -> Result<(), String> {
    for pasta in anotadinho_core::semente::PASTAS {
        std::fs::create_dir_all(raiz.join(pasta)).map_err(|e| e.to_string())?;
    }
    for (path, conteudo) in &tarefa.vault {
        let destino = raiz.join(path);
        if let Some(pai) = destino.parent() {
            std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
        }
        std::fs::write(&destino, conteudo).map_err(|e| e.to_string())?;
    }
    if let Some(p) = &tarefa.permissoes {
        anotadinho_ipc::handle_gravar_permissoes(raiz.to_string_lossy().to_string(), p.clone())?;
    }
    Ok(())
}

/// O prompt que o agente recebe: a tarefa, onde está o vault e o que ele
/// pode chamar.
fn prompt_da_tarefa(tarefa: &Tarefa, vault: &std::path::Path) -> String {
    format!(
        "{}\n\nO vault está em {}. Para agir, use o CLI do Anotadinho \
         (`anotadinho-cli --vault {} <comando>`) ou o servidor MCP dele.\n\n{}",
        tarefa.prompt,
        vault.display(),
        vault.display(),
        anotadinho_core::ferramentas::em_texto()
    )
}

/// Roda o agente uma vez e recolhe o que se pode observar.
fn rodar(adaptador: &Adaptador, tarefa: &Tarefa, vault: &std::path::Path) -> Observado {
    let comeco = std::time::Instant::now();
    let prompt = prompt_da_tarefa(tarefa, vault);
    let saida = executar(adaptador, &prompt, vault);
    let segundos = comeco.elapsed().as_secs();
    let propostas = anotadinho_ipc::handle_listar_propostas(vault.to_string_lossy().to_string())
        .unwrap_or_default()
        .into_iter()
        .map(|p| (p.alvo, p.conteudo))
        .collect();
    match saida {
        Ok(resposta) => Observado { propostas, resposta, falha: None, segundos },
        Err(e) => Observado { propostas, resposta: String::new(), falha: Some(e), segundos },
    }
}

/// Chama o binário do agente e espera a saída, com o timeout dele.
fn executar(adaptador: &Adaptador, prompt: &str, cwd: &std::path::Path) -> Result<String, String> {
    use std::process::{Command, Stdio};
    if let Some(problema) = adaptador.validar() {
        return Err(format!("configuração do agente inválida: {}", problema.mensagem()));
    }
    // O filho roda DENTRO do vault temporário, então caminho relativo
    // ("./scripts/agente.sh") se perderia: resolve pela pasta de quem
    // chamou a suíte, antes de trocar de diretório.
    let executavel = anotadinho_core::agente::executavel_para_spawn(&adaptador.binario);
    let executavel = if executavel.contains(std::path::MAIN_SEPARATOR) && std::path::Path::new(&executavel).is_relative() {
        std::fs::canonicalize(&executavel)
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or(executavel)
    } else {
        executavel
    };
    let mut filho = Command::new(&executavel)
        .args(adaptador.montar_args(prompt))
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("não consegui executar \"{}\": {e}", adaptador.binario))?;
    // Espera com teto: um agente pendurado não pode travar a suíte.
    let limite = std::time::Duration::from_secs(if adaptador.timeout_s == 0 { 600 } else { adaptador.timeout_s });
    let comeco = std::time::Instant::now();
    loop {
        match filho.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if comeco.elapsed() >= limite => {
                let _ = filho.kill();
                let _ = filho.wait();
                return Err(format!("o agente passou de {}s e foi interrompido", limite.as_secs()));
            }
            Ok(None) => std::thread::sleep(std::time::Duration::from_millis(100)),
            Err(e) => return Err(format!("erro esperando o agente: {e}")),
        }
    }
    let saida = filho.wait_with_output().map_err(|e| e.to_string())?;
    let texto = String::from_utf8_lossy(&saida.stdout).trim().to_string();
    if saida.status.success() {
        let resposta = match adaptador.formato {
            anotadinho_core::agente::FormatoSaida::StreamJson => {
                let mut leitor = anotadinho_core::agente::LeitorStream::novo();
                for linha in texto.lines() {
                    leitor.linha(linha);
                }
                leitor.resposta()?
            }
            anotadinho_core::agente::FormatoSaida::Texto => texto,
        };
        Ok(resposta)
    } else {
        let erro = String::from_utf8_lossy(&saida.stderr).trim().to_string();
        Err(format!("o agente falhou ({}): {}", saida.status, if erro.is_empty() { texto } else { erro }))
    }
}

/// Roda a suíte inteira. `so` filtra pelo nome da tarefa.
pub fn rodar_suite(
    tarefas: &[Tarefa],
    adaptador: &Adaptador,
    so: Option<&str>,
) -> Result<Vec<Veredito>, String> {
    let mut vereditos = Vec::new();
    for tarefa in tarefas.iter().filter(|t| so.is_none_or(|s| t.nome.contains(s))) {
        let dir = tempfile::TempDir::new().map_err(|e| e.to_string())?;
        preparar(tarefa, dir.path())?;
        let observado = rodar(adaptador, tarefa, dir.path());
        let veredito = julgar(tarefa, &observado);
        // Relatório enquanto roda: a suíte com agente real leva minutos,
        // e esperar o fim pra ver a primeira linha é ruim de usar.
        println!(
            "  {} {} ({}s)",
            if veredito.passou { "✓" } else { "✗" },
            veredito.tarefa,
            veredito.segundos
        );
        for falha in &veredito.falhas {
            println!("      {falha}");
        }
        // Reprovou: mostra o que o agente respondeu. Sem isto, ajustar
        // uma tarefa é adivinhação — e quem escreve tarefa erra o
        // enunciado tanto quanto o agente erra a resposta.
        if !veredito.passou {
            let resposta: String = observado.resposta.replace('\n', " ").chars().take(200).collect();
            if !resposta.trim().is_empty() {
                println!("      disse: {resposta}");
            }
            if !observado.propostas.is_empty() {
                let alvos: Vec<&str> = observado.propostas.iter().map(|(a, _)| a.as_str()).collect();
                println!("      propôs: {}", alvos.join(", "));
            }
        }
        vereditos.push(veredito);
    }
    Ok(vereditos)
}

/// Lê as tarefas de um arquivo JSON.
pub fn ler_tarefas(caminho: &str) -> Result<Vec<Tarefa>, String> {
    let texto = std::fs::read_to_string(caminho).map_err(|e| format!("{caminho}: {e}"))?;
    serde_json::from_str(&texto).map_err(|e| format!("{caminho}: {e}"))
}

#[cfg(test)]
mod testes {
    use super::*;
    use anotadinho_core::avaliacao::Espera;

    fn falso(modo: &str) -> Adaptador {
        Adaptador {
            nome: "falso".into(),
            binario: concat!(env!("CARGO_MANIFEST_DIR"), "/../../scripts/uitest/agente-falso.sh").into(),
            args: vec![modo.into(), "{prompt}".into()],
            formato: anotadinho_core::agente::FormatoSaida::Texto,
            timeout_s: 30,
            ..Adaptador::default()
        }
    }

    fn tarefa(nome: &str, espera: Vec<Espera>) -> Tarefa {
        Tarefa {
            nome: nome.into(),
            prompt: "diga algo".into(),
            vault: vec![("pages/a.md".into(), "# A\n".into())],
            permissoes: None,
            espera,
        }
    }

    /// O caminho inteiro: vault temporário, agente chamado, julgamento.
    #[test]
    fn a_suite_roda_o_agente_e_julga() {
        // O agente falso ecoa o prompt: a tarefa é o que ele responde.
        let tarefas = vec![
            tarefa("responde", vec![Espera::RespostaContem { texto: "diga algo".into() }, Espera::NaoPropoe]),
            tarefa("nao-passa", vec![Espera::PropoeEm { prefixo: "pages/specs".into() }]),
        ];
        let vereditos = rodar_suite(&tarefas, &falso("--responder"), None).expect("rodou");
        assert_eq!(vereditos.len(), 2);
        assert!(vereditos[0].passou, "{:?}", vereditos[0].falhas);
        assert!(!vereditos[1].passou);
        assert_eq!(anotadinho_core::avaliacao::resumo(&vereditos).split('/').next(), Some("1"));
    }

    /// O prompt leva o contrato e o caminho do vault — é o que o agente
    /// precisa pra propor sem adivinhar.
    #[test]
    fn o_prompt_carrega_o_contrato_e_o_vault() {
        let p = prompt_da_tarefa(&tarefa("x", vec![]), std::path::Path::new("/tmp/v"));
        assert!(p.contains("diga algo") && p.contains("/tmp/v"), "{p}");
        assert!(p.contains("propor(path, conteudo, [motivo], [lote])"), "{p}");
    }

    #[test]
    fn falha_do_agente_reprova_a_tarefa_com_a_mensagem() {
        let tarefas = vec![tarefa("falha", vec![Espera::RespostaContem { texto: "qualquer".into() }])];
        let vereditos = rodar_suite(&tarefas, &falso("--falhar"), None).expect("rodou");
        assert!(!vereditos[0].passou);
        assert!(vereditos[0].falhas.iter().any(|f| f.contains("falhou")), "{:?}", vereditos[0].falhas);
    }

    #[test]
    fn so_filtra_pelo_nome() {
        let tarefas = vec![
            tarefa("uma", vec![Espera::NaoPropoe]),
            tarefa("outra", vec![Espera::NaoPropoe]),
        ];
        let vereditos = rodar_suite(&tarefas, &falso("--responder"), Some("out")).expect("rodou");
        assert_eq!(vereditos.len(), 1);
        assert_eq!(vereditos[0].tarefa, "outra");
    }
}
