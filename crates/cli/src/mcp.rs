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

/// Descrição de cada ferramenta, no formato que o MCP espera.
fn ferramentas() -> Value {
    json!([
        {
            "name": "listar_paginas",
            "description": "Lista as páginas do vault com título, path e seção.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "ler_pagina",
            "description": "Lê o markdown de uma página.",
            "inputSchema": {
                "type": "object",
                "properties": { "path": { "type": "string", "description": "Path relativo ao vault" } },
                "required": ["path"]
            }
        },
        {
            "name": "buscar",
            "description": "Busca full-text no vault. Resultados de dentro de embeds vêm com a origem (ex: 'Kanban · coluna Backlog').",
            "inputSchema": {
                "type": "object",
                "properties": { "termo": { "type": "string" } },
                "required": ["termo"]
            }
        },
        {
            "name": "consultar",
            "description": "Recorte do vault por filtro — o MESMO motor do embed de consulta. Ex: from='pages/specs', where=['status=rascunho'].",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "from": { "type": "string" },
                    "where": { "type": "array", "items": { "type": "string" } },
                    "limit": { "type": "integer" }
                }
            }
        },
        {
            "name": "propor",
            "description": "PROPÕE uma escrita pra revisão humana. Não grava a página: a mudança só é aplicada depois que a pessoa vê o diff e aprova. Esta é a única forma de escrever.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "path": { "type": "string", "description": "Página alvo, relativa ao vault" },
                    "conteudo": { "type": "string", "description": "Markdown completo da página" },
                    "motivo": { "type": "string", "description": "Por que esta mudança" }
                },
                "required": ["path", "conteudo"]
            }
        },
        {
            "name": "propostas_pendentes",
            "description": "Lista o que já foi proposto e ainda aguarda revisão.",
            "inputSchema": { "type": "object", "properties": {} }
        }
    ])
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
        "propostas_pendentes" => {
            match anotadinho_ipc::handle_listar_propostas(vault.to_string()) {
                Ok(l) => texto(id, serde_json::to_string_pretty(&l).unwrap_or_default()),
                Err(e) => texto_erro(id, e),
            }
        }
        outro => texto_erro(id, format!("ferramenta desconhecida: {outro}")),
    }
}
