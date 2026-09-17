//! Servidor MCP por stdio (ciclo 205).
//!
//! Expõe o vault como ferramentas pra um agente que fale MCP — Claude
//! Code, entre outros. É o sentido de integração complementar ao ciclo
//! 202: lá o Anotadinho CHAMA o agente; aqui o agente se conecta e
//! opera o vault, com o Anotadinho como estado compartilhado.
//!
//! # A escolha que define a segurança
//!
//! As ferramentas de LEITURA são diretas. A de ESCRITA é `propor`, não
//! `escrever`: o agente não tem como gravar uma página, só sugerir. O
//! que ele propõe aparece na tela de revisão (ciclo 204) e só vira
//! arquivo depois de um clique humano.
//!
//! Isso não é um detalhe de implementação — é o que torna seguro deixar
//! um modelo conectado no vault sem supervisão contínua.
//!
//! # Protocolo
//!
//! JSON-RPC 2.0 em linhas, sobre stdin/stdout: uma requisição por linha,
//! uma resposta por linha. É o transporte stdio do MCP.

use serde_json::{json, Value};
use std::io::{BufRead, Write};

/// Versão do protocolo que este servidor fala.
const VERSAO_PROTOCOLO: &str = "2024-11-05";

/// Descrição de cada ferramenta, no formato que o MCP espera. A lista
/// vem do contrato do núcleo (ciclo 407) — aqui só se traduz.
fn ferramentas() -> Value {
    Value::Array(anotadinho_core::ferramentas::CONTRATO.iter().map(|f| f.para_mcp()).collect())
}

/// Roda o servidor até o stdin fechar.
pub fn servir(vault: String) -> Result<(), String> {
    let entrada = std::io::stdin();
    let mut saida = std::io::stdout();

    for linha in entrada.lock().lines() {
        let linha = linha.map_err(|e| format!("erro lendo stdin: {e}"))?;
        if linha.trim().is_empty() {
            continue;
        }
        let req: Value = match serde_json::from_str(&linha) {
            Ok(v) => v,
            // JSON quebrado não derruba o servidor: responde o erro e
            // segue. Um agente com um bug de serialização não pode
            // matar a sessão inteira.
            Err(e) => {
                responder(&mut saida, erro(Value::Null, -32700, &format!("JSON inválido: {e}")))?;
                continue;
            }
        };
        let id = req.get("id").cloned().unwrap_or(Value::Null);
        let metodo = req.get("method").and_then(|m| m.as_str()).unwrap_or("");

        // Notificação (sem `id`) não recebe resposta, como manda o
        // JSON-RPC — responder faria o cliente reclamar.
        if req.get("id").is_none() {
            continue;
        }

        let resposta = match metodo {
            "initialize" => ok(id, json!({
                "protocolVersion": VERSAO_PROTOCOLO,
                "capabilities": { "tools": {} },
                "serverInfo": { "name": "anotadinho", "version": env!("CARGO_PKG_VERSION") }
            })),
            "tools/list" => ok(id, json!({ "tools": ferramentas() })),
            "tools/call" => chamar(&vault, id, req.get("params")),
            "ping" => ok(id, json!({})),
            outro => erro(id, -32601, &format!("método desconhecido: {outro}")),
        };
        responder(&mut saida, resposta)?;
    }
    Ok(())
}

fn responder(saida: &mut std::io::Stdout, v: Value) -> Result<(), String> {
    writeln!(saida, "{v}").map_err(|e| format!("erro escrevendo: {e}"))?;
    saida.flush().map_err(|e| format!("erro no flush: {e}"))
}

fn ok(id: Value, resultado: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": resultado })
}

fn erro(id: Value, codigo: i32, msg: &str) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "error": { "code": codigo, "message": msg } })
}

/// Resposta de ferramenta no formato do MCP: conteúdo em blocos de texto.
fn texto(id: Value, s: String) -> Value {
    ok(id, json!({ "content": [{ "type": "text", "text": s }] }))
}

/// Erro de ferramenta — `isError` em vez de erro de protocolo, pra o
/// agente conseguir LER a mensagem e corrigir, em vez de só falhar.
fn texto_erro(id: Value, s: String) -> Value {
    ok(id, json!({ "content": [{ "type": "text", "text": s }], "isError": true }))
}

fn chamar(vault: &str, id: Value, params: Option<&Value>) -> Value {
    let Some(params) = params else {
        return erro(id, -32602, "faltou `params`");
    };
    let nome = params.get("name").and_then(|n| n.as_str()).unwrap_or("");
    let args = params.get("arguments").cloned().unwrap_or(json!({}));
    let arg_str = |chave: &str| args.get(chave).and_then(|v| v.as_str()).unwrap_or("").to_string();

    match nome {
        "listar_paginas" => match anotadinho_ipc::handle_scan_vault(vault.to_string()) {
            Ok(p) => texto(id, serde_json::to_string_pretty(&p).unwrap_or_default()),
            Err(e) => texto_erro(id, e),
        },
        "ler_pagina" => match anotadinho_ipc::handle_read_page(vault.to_string(), arg_str("path")) {
            Ok(c) => texto(id, c),
            Err(e) => texto_erro(id, e),
        },
        "buscar" => {
            match anotadinho_ipc::handle_search_content(vault.to_string(), arg_str("termo")) {
                Ok(r) => texto(id, serde_json::to_string_pretty(&r).unwrap_or_default()),
                Err(e) => texto_erro(id, e),
            }
        }
        "consultar" => {
            let mut q = anotadinho_core::query::Query {
                from: Some(arg_str("from")).filter(|f| !f.is_empty()),
                ..Default::default()
            };
            if let Some(w) = args.get("where").and_then(|v| v.as_array()) {
                for cond in w.iter().filter_map(|c| c.as_str()) {
                    match crate::parse_condition(cond) {
                        Ok(c) => q.conditions.push(c),
                        Err(e) => return texto_erro(id, e),
                    }
                }
            }
            q.limit = args.get("limit").and_then(|v| v.as_u64()).map(|n| n as usize);
            match anotadinho_ipc::handle_scan_vault(vault.to_string()) {
                Ok(paginas) => {
                    let achados: Vec<_> = q.run(&paginas).into_iter().cloned().collect();
                    texto(id, serde_json::to_string_pretty(&achados).unwrap_or_default())
                }
                Err(e) => texto_erro(id, e),
            }
        }
        "propor" => {
            let path = arg_str("path");
            let existe = std::path::Path::new(vault).join(&path).exists();
            let proposta = anotadinho_core::proposta::Proposta {
                id: format!(
                    "{}-{}",
                    anotadinho_core::fluxo::slug_de_titulo(&path),
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map(|d| d.as_secs())
                        .unwrap_or(0)
                ),
                autor: "mcp".to_string(),
                quando: crate::agora_legivel(),
                motivo: arg_str("motivo"),
                alvo: path,
                operacao: if existe {
                    anotadinho_core::proposta::Operacao::Substituir
                } else {
                    anotadinho_core::proposta::Operacao::Criar
                },
                conteudo: arg_str("conteudo"),
                lote: Some(arg_str("lote")).filter(|l| !l.trim().is_empty()),
            };
            match anotadinho_ipc::handle_propor(vault.to_string(), proposta) {
                Ok(id_p) => texto(
                    id,
                    format!(
                        "proposta {id_p} criada. A página NÃO foi escrita — ela aguarda revisão humana."
                    ),
                ),
                Err(e) => texto_erro(id, e),
            }
        }
        "onde_posso_escrever" => match anotadinho_ipc::handle_ler_permissoes(vault.to_string()) {
            Ok(p) => texto(id, serde_json::to_string_pretty(&p).unwrap_or_default()),
            Err(e) => texto_erro(id, e),
        },
        "propostas_pendentes" => {
            match anotadinho_ipc::handle_listar_propostas(vault.to_string()) {
                Ok(l) => texto(id, serde_json::to_string_pretty(&l).unwrap_or_default()),
                Err(e) => texto_erro(id, e),
            }
        }
        outro => texto_erro(id, format!("ferramenta desconhecida: {outro}")),
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    /// O contrato do núcleo e o despachante daqui não podem divergir
    /// (ciclo 407): publicar uma ferramenta que ninguém atende daria
    /// "ferramenta desconhecida" na cara do agente.
    #[test]
    fn toda_ferramenta_publicada_tem_quem_atenda() {
        let dir = tempfile::TempDir::new().unwrap();
        let vault = dir.path().to_string_lossy().to_string();
        let publicadas = ferramentas();
        assert_eq!(
            publicadas.as_array().unwrap().len(),
            anotadinho_core::ferramentas::CONTRATO.len()
        );
        for f in anotadinho_core::ferramentas::CONTRATO {
            let params = json!({ "name": f.nome, "arguments": {} });
            let r = chamar(&vault, json!(1), Some(&params));
            let texto = serde_json::to_string(&r).unwrap();
            assert!(!texto.contains("ferramenta desconhecida"), "{} não tem branch", f.nome);
        }
        // E o contrário: nome de fora do contrato é recusado.
        let params = json!({ "name": "apagar_vault", "arguments": {} });
        let r = chamar(&vault, json!(1), Some(&params));
        assert!(serde_json::to_string(&r).unwrap().contains("ferramenta desconhecida"));
    }

    /// A ferramenta de permissões (ciclo 407) responde o padrão num
    /// vault sem arquivo — é o que o agente lê antes de propor.
    #[test]
    fn onde_posso_escrever_responde_as_permissoes() {
        let dir = tempfile::TempDir::new().unwrap();
        let vault = dir.path().to_string_lossy().to_string();
        let params = json!({ "name": "onde_posso_escrever", "arguments": {} });
        let r = chamar(&vault, json!(1), Some(&params));
        let texto = serde_json::to_string(&r).unwrap();
        assert!(texto.contains("nunca") && texto.contains("journals/"), "{texto}");
    }
}
