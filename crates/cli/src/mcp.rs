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

/// O esquema das URIs de recurso: `anotadinho://<path no vault>`.
const ESQUEMA: &str = "anotadinho://";

/// As páginas do vault como RECURSOS (ciclo 435).
///
/// Um cliente MCP mostra recursos pra pessoa escolher o que anexar, e o
/// agente lê sem gastar chamada de ferramenta. É a mesma lista de
/// `listar_paginas`, na forma que o protocolo espera.
fn recursos(vault: &str, id: Value) -> Value {
    match anotadinho_ipc::handle_scan_vault(vault.to_string()) {
        Ok(paginas) => {
            let itens: Vec<Value> = paginas
                .iter()
                .map(|p| {
                    let nome = if p.title.trim().is_empty() { p.path.clone() } else { p.title.clone() };
                    json!({
                        "uri": format!("{ESQUEMA}{}", p.path),
                        "name": nome,
                        "description": format!("Página do vault ({})", p.path),
                        "mimeType": "text/markdown"
                    })
                })
                .collect();
            ok(id, json!({ "resources": itens }))
        }
        Err(e) => erro(id, -32603, &e),
    }
}

/// Lê um recurso. A transclusão vem RESOLVIDA (ciclo 414): quem pede uma
/// página-recorte quer o conteúdo dela, não os marcadores.
fn ler_recurso(vault: &str, id: Value, params: Option<&Value>) -> Value {
    let uri = params
        .and_then(|p| p.get("uri"))
        .and_then(|u| u.as_str())
        .unwrap_or("")
        .to_string();
    let Some(path) = uri.strip_prefix(ESQUEMA) else {
        return erro(id, -32602, &format!("uri fora do vault: {uri}"));
    };
    match anotadinho_ipc::handle_ler_para_contexto(vault.to_string(), path.to_string()) {
        Ok(x) => ok(
            id,
            json!({ "contents": [{ "uri": uri, "mimeType": "text/markdown", "text": x.texto }] }),
        ),
        Err(e) => erro(id, -32603, &e),
    }
}

/// Os prompts padrão do vault como PROMPTS do protocolo (ciclo 435).
///
/// Eles já existem como páginas (`pages/prompts-default/`, ciclo 341) e
/// já têm variáveis `{{assim}}`. Publicá-los aqui é o que faz o trabalho
/// de escrever bons prompts valer também fora do app.
fn prompts(vault: &str, id: Value) -> Value {
    match anotadinho_ipc::handle_scan_vault(vault.to_string()) {
        Ok(paginas) => {
            let itens: Vec<Value> = anotadinho_core::prompt_padrao::descobrir(paginas)
                .into_iter()
                .map(|p| {
                    json!({
                        "name": p.title,
                        "description": format!("Prompt padrão do vault ({})", p.path),
                    })
                })
                .collect();
            ok(id, json!({ "prompts": itens }))
        }
        Err(e) => erro(id, -32603, &e),
    }
}

/// Entrega um prompt expandido com os argumentos que o cliente mandou.
fn pegar_prompt(vault: &str, id: Value, params: Option<&Value>) -> Value {
    let nome = params
        .and_then(|p| p.get("name"))
        .and_then(|n| n.as_str())
        .unwrap_or("")
        .to_string();
    let paginas = match anotadinho_ipc::handle_scan_vault(vault.to_string()) {
        Ok(p) => p,
        Err(e) => return erro(id, -32603, &e),
    };
    let Some(pagina) = anotadinho_core::prompt_padrao::descobrir(paginas)
        .into_iter()
        .find(|p| p.title == nome)
    else {
        return erro(id, -32602, &format!("prompt desconhecido: {nome}"));
    };
    let conteudo = match anotadinho_ipc::handle_read_page(vault.to_string(), pagina.path.clone()) {
        Ok(c) => c,
        Err(e) => return erro(id, -32603, &e),
    };
    let (_, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
    let molde = anotadinho_core::prompt_padrao::PromptPadrao::parse(corpo);
    // Argumento que falta fica como estava: um prompt meio preenchido é
    // mais útil que um erro, e o marcador diz o que falta.
    let valores: std::collections::BTreeMap<String, String> = params
        .and_then(|p| p.get("arguments"))
        .and_then(|a| a.as_object())
        .map(|o| {
            o.iter()
                .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
                .collect()
        })
        .unwrap_or_default();
    let texto = molde.visualizar_parcial(&valores);
    ok(
        id,
        json!({
            "description": format!("Prompt padrão do vault ({})", pagina.path),
            "messages": [{ "role": "user", "content": { "type": "text", "text": texto } }]
        }),
    )
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
                // Recursos e prompts entraram no ciclo 435: um agente
                // que fala MCP passa a LER o vault sem gastar uma
                // chamada de ferramenta, e a usar os prompts padrão que
                // já estão escritos nele.
                "capabilities": { "tools": {}, "resources": {}, "prompts": {} },
                "serverInfo": { "name": "anotadinho", "version": env!("CARGO_PKG_VERSION") }
            })),
            "tools/list" => ok(id, json!({ "tools": ferramentas() })),
            "tools/call" => chamar(&vault, id, req.get("params")),
            "resources/list" => recursos(&vault, id),
            "resources/read" => ler_recurso(&vault, id, req.get("params")),
            "prompts/list" => prompts(&vault, id),
            "prompts/get" => pegar_prompt(&vault, id, req.get("params")),
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

    // --- Ciclo 435: recursos e prompts -----------------------------------------------

    fn vault_com_prompt() -> tempfile::TempDir {
        let dir = tempfile::TempDir::new().unwrap();
        std::fs::create_dir_all(dir.path().join("pages/prompts-default")).unwrap();
        std::fs::write(
            dir.path().join("pages/nota.md"),
            "---\ntitle: Nota\n---\n\nconteúdo da nota\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("pages/prompts-default/entender.md"),
            "---\ntitle: Entender um trecho\ntype: prompt\n---\n\nExplique {{trecho}} pra quem chega agora.\n",
        )
        .unwrap();
        dir
    }

    #[test]
    fn as_paginas_aparecem_como_recursos_e_leem_resolvidas() {
        let dir = vault_com_prompt();
        let vault = dir.path().to_string_lossy().to_string();
        std::fs::write(
            dir.path().join("pages/recorte.md"),
            "---\ntitle: Recorte\n---\n\n![[Nota]]\n",
        )
        .unwrap();

        let lista = recursos(&vault, json!(1));
        let uris: Vec<String> = lista["result"]["resources"]
            .as_array()
            .unwrap()
            .iter()
            .map(|r| r["uri"].as_str().unwrap().to_string())
            .collect();
        assert!(uris.contains(&"anotadinho://pages/nota.md".to_string()), "{uris:?}");

        // Ler traz o conteúdo — e a transclusão já resolvida (414).
        let lido = ler_recurso(&vault, json!(2), Some(&json!({ "uri": "anotadinho://pages/recorte.md" })));
        let texto = lido["result"]["contents"][0]["text"].as_str().unwrap();
        assert!(texto.contains("conteúdo da nota"), "{texto}");
        // URI de fora do vault é recusada, não lida.
        let fora = ler_recurso(&vault, json!(3), Some(&json!({ "uri": "file:///etc/passwd" })));
        assert!(fora["error"]["message"].as_str().unwrap().contains("fora do vault"));
    }

    #[test]
    fn os_prompts_padrao_do_vault_viram_prompts_do_protocolo() {
        let dir = vault_com_prompt();
        let vault = dir.path().to_string_lossy().to_string();
        let lista = prompts(&vault, json!(1));
        let nomes: Vec<String> = lista["result"]["prompts"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p["name"].as_str().unwrap().to_string())
            .collect();
        assert_eq!(nomes, ["Entender um trecho"]);

        // Com argumento, vem preenchido.
        let pego = pegar_prompt(
            &vault,
            json!(2),
            Some(&json!({ "name": "Entender um trecho", "arguments": { "trecho": "o kanban" } })),
        );
        let texto = pego["result"]["messages"][0]["content"]["text"].as_str().unwrap();
        // O valor entra BLINDADO (ciclo 224): é dado, não instrução.
        assert!(texto.contains("o kanban") && texto.contains("DADO-ANOTADINHO"), "{texto}");
        assert!(texto.contains("pra quem chega agora"), "{texto}");

        // Sem argumento, o marcador fica — meio preenchido é mais útil
        // que um erro.
        let cru = pegar_prompt(&vault, json!(3), Some(&json!({ "name": "Entender um trecho" })));
        let texto = cru["result"]["messages"][0]["content"]["text"].as_str().unwrap();
        assert!(texto.contains("trecho"), "{texto}");

        // Nome que não existe avisa.
        let zero = pegar_prompt(&vault, json!(4), Some(&json!({ "name": "não existe" })));
        assert!(zero["error"]["message"].as_str().unwrap().contains("desconhecido"));
    }
}
