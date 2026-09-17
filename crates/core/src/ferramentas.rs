//! O contrato de ferramentas do agente (ciclo 407).
//!
//! O conjunto do que um agente pode pedir ao Anotadinho é FECHADO e
//! declarado aqui, num lugar sem IO. Antes ele existia só como um
//! `json!` no servidor MCP do CLI — o que deixava o resto do programa
//! sem saber o que o agente alcança, e a pessoa sem onde ler isso.
//!
//! # Por que o contrato mora no núcleo
//!
//! Três leitores diferentes precisam da MESMA lista:
//!
//! - o servidor MCP, que publica as ferramentas pro agente;
//! - a UI (TUI e janela), que mostra à pessoa o que o agente pode fazer
//!   — é a resposta honesta pra "o que esse bicho consegue mexer?";
//! - o prompt, quando o agente não fala MCP e precisa da lista por
//!   escrito.
//!
//! Duas cópias divergiriam, e a que divergisse seria justamente a que a
//! pessoa lê. Uma só, aqui.
//!
//! # A invariante que a lista carrega
//!
//! Exatamente UMA ferramenta escreve — `propor` — e ela não grava
//! página: enfileira uma proposta pra revisão (ciclo 204), dentro das
//! permissões por pasta (ciclo 405). Isso é verificado em teste, pra
//! que crescer o contrato não abra escrita direta por descuido.

use serde_json::{json, Map, Value};

/// Tipo de um parâmetro, no vocabulário do JSON Schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tipo {
    Texto,
    Inteiro,
    ListaDeTexto,
}

impl Tipo {
    fn nome_json(&self) -> &'static str {
        match self {
            Self::Texto => "string",
            Self::Inteiro => "integer",
            Self::ListaDeTexto => "array",
        }
    }

    /// Como aparece pra pessoa, na tela do contrato.
    pub fn rotulo(&self) -> &'static str {
        match self {
            Self::Texto => "texto",
            Self::Inteiro => "número",
            Self::ListaDeTexto => "lista de textos",
        }
    }
}

/// Um parâmetro de ferramenta.
#[derive(Debug, Clone, PartialEq)]
pub struct Parametro {
    pub nome: &'static str,
    pub tipo: Tipo,
    pub descricao: &'static str,
    pub obrigatorio: bool,
}

/// Uma ferramenta do contrato.
#[derive(Debug, Clone, PartialEq)]
pub struct Ferramenta {
    pub nome: &'static str,
    /// O que ela faz, na voz que o agente lê.
    pub descricao: &'static str,
    /// `true` só pra quem muda o vault — hoje apenas `propor`, e ainda
    /// assim passando por revisão humana.
    pub escreve: bool,
    pub parametros: &'static [Parametro],
}

impl Ferramenta {
    /// O `inputSchema` que o MCP espera.
    pub fn esquema(&self) -> Value {
        let mut props = Map::new();
        for p in self.parametros {
            let mut campo = Map::new();
            campo.insert("type".into(), json!(p.tipo.nome_json()));
            if p.tipo == Tipo::ListaDeTexto {
                campo.insert("items".into(), json!({ "type": "string" }));
            }
            if !p.descricao.is_empty() {
                campo.insert("description".into(), json!(p.descricao));
            }
            props.insert(p.nome.into(), Value::Object(campo));
        }
        let obrigatorios: Vec<&str> =
            self.parametros.iter().filter(|p| p.obrigatorio).map(|p| p.nome).collect();
        let mut esquema = Map::new();
        esquema.insert("type".into(), json!("object"));
        esquema.insert("properties".into(), Value::Object(props));
        if !obrigatorios.is_empty() {
            esquema.insert("required".into(), json!(obrigatorios));
        }
        Value::Object(esquema)
    }

    /// A ferramenta inteira no formato do `tools/list` do MCP.
    pub fn para_mcp(&self) -> Value {
        json!({ "name": self.nome, "description": self.descricao, "inputSchema": self.esquema() })
    }

    /// Assinatura de uma linha, pra tela e pra prompt:
    /// `propor(path, conteudo, [motivo])`.
    pub fn assinatura(&self) -> String {
        let args: Vec<String> = self
            .parametros
            .iter()
            .map(|p| if p.obrigatorio { p.nome.to_string() } else { format!("[{}]", p.nome) })
            .collect();
        format!("{}({})", self.nome, args.join(", "))
    }
}

const P_PATH: Parametro = Parametro {
    nome: "path",
    tipo: Tipo::Texto,
    descricao: "Path relativo ao vault",
    obrigatorio: true,
};

/// O contrato. Fechado: crescer esta lista é uma decisão de projeto, não
/// um efeito colateral de outra mudança.
pub const CONTRATO: &[Ferramenta] = &[
    Ferramenta {
        nome: "listar_paginas",
        descricao: "Lista as páginas do vault com título, path e seção.",
        escreve: false,
        parametros: &[],
    },
    Ferramenta {
        nome: "ler_pagina",
        descricao: "Lê o markdown de uma página.",
        escreve: false,
        parametros: &[P_PATH],
    },
    Ferramenta {
        nome: "buscar",
        descricao: "Busca full-text no vault. Resultados de dentro de embeds vêm com a origem (ex: 'Kanban · coluna Backlog').",
        escreve: false,
        parametros: &[Parametro {
            nome: "termo",
            tipo: Tipo::Texto,
            descricao: "",
            obrigatorio: true,
        }],
    },
    Ferramenta {
        nome: "consultar",
        descricao: "Recorte do vault por filtro — o MESMO motor do embed de consulta. Ex: from='pages/specs', where=['status=rascunho'].",
        escreve: false,
        parametros: &[
            Parametro { nome: "from", tipo: Tipo::Texto, descricao: "", obrigatorio: false },
            Parametro {
                nome: "where",
                tipo: Tipo::ListaDeTexto,
                descricao: "",
                obrigatorio: false,
            },
            Parametro { nome: "limit", tipo: Tipo::Inteiro, descricao: "", obrigatorio: false },
        ],
    },
    Ferramenta {
        nome: "onde_posso_escrever",
        descricao: "As pastas em que propor é aceito, e as proibidas. Consulte ANTES de propor: fora delas a proposta é recusada na hora.",
        escreve: false,
        parametros: &[],
    },
    Ferramenta {
        nome: "propor",
        descricao: "PROPÕE uma escrita pra revisão humana. Não grava a página: a mudança só é aplicada depois que a pessoa vê o diff e aprova. Esta é a única forma de escrever.",
        escreve: true,
        parametros: &[
            P_PATH,
            Parametro {
                nome: "conteudo",
                tipo: Tipo::Texto,
                descricao: "Markdown completo da página",
                obrigatorio: true,
            },
            Parametro {
                nome: "motivo",
                tipo: Tipo::Texto,
                descricao: "Por que esta mudança",
                obrigatorio: false,
            },
            Parametro {
                nome: "lote",
                tipo: Tipo::Texto,
                descricao: "Nome do lote: propostas com o mesmo lote são UMA decisão, aplicadas juntas ou nenhuma. Use quando a mudança atravessa páginas.",
                obrigatorio: false,
            },
        ],
    },
    Ferramenta {
        nome: "propostas_pendentes",
        descricao: "Lista o que já foi proposto e ainda aguarda revisão.",
        escreve: false,
        parametros: &[],
    },
];

/// Busca pelo nome que o agente chamou.
pub fn por_nome(nome: &str) -> Option<&'static Ferramenta> {
    CONTRATO.iter().find(|f| f.nome == nome)
}

/// O contrato por escrito, pra colar em prompt de agente que não fala
/// MCP (ciclo 407).
pub fn em_texto() -> String {
    let mut s = String::from("Ferramentas do vault (só estas):\n");
    for f in CONTRATO {
        let marca = if f.escreve { " [escreve, passa por revisão]" } else { "" };
        s.push_str(&format!("- {}{marca} — {}\n", f.assinatura(), f.descricao));
    }
    s
}

#[cfg(test)]
mod testes {
    use super::*;

    /// A invariante do ciclo 204 escrita como teste: uma única
    /// ferramenta de escrita, e ela é a que enfileira proposta.
    #[test]
    fn so_uma_ferramenta_escreve_e_ela_propoe() {
        let escrevem: Vec<&str> = CONTRATO.iter().filter(|f| f.escreve).map(|f| f.nome).collect();
        assert_eq!(escrevem, ["propor"]);
        assert!(por_nome("propor").unwrap().descricao.contains("revisão humana"));
        // E nenhuma ferramenta promete gravar direto.
        for f in CONTRATO {
            assert!(f.nome != "escrever" && f.nome != "gravar", "{}", f.nome);
        }
    }

    #[test]
    fn o_esquema_declara_tipos_e_obrigatorios() {
        let propor = por_nome("propor").unwrap();
        let e = propor.esquema();
        assert_eq!(e["type"], "object");
        assert_eq!(e["properties"]["conteudo"]["type"], "string");
        assert_eq!(e["required"], json!(["path", "conteudo"]));
        // Opcional não entra em `required`.
        assert!(!e["required"].as_array().unwrap().iter().any(|v| v == "motivo"));
        // Lista vira array com itens de texto.
        let consultar = por_nome("consultar").unwrap().esquema();
        assert_eq!(consultar["properties"]["where"]["items"]["type"], "string");
        assert!(consultar.get("required").is_none());
        // Sem parâmetro, o esquema fica vazio mas válido.
        let listar = por_nome("listar_paginas").unwrap().esquema();
        assert_eq!(listar["properties"], json!({}));
    }

    #[test]
    fn a_assinatura_e_o_texto_servem_pra_prompt() {
        assert_eq!(por_nome("propor").unwrap().assinatura(), "propor(path, conteudo, [motivo], [lote])");
        assert_eq!(por_nome("listar_paginas").unwrap().assinatura(), "listar_paginas()");
        let texto = em_texto();
        assert!(texto.contains("propor(path, conteudo, [motivo], [lote]) [escreve, passa por revisão]"));
        assert!(texto.contains("ler_pagina(path)"));
        assert!(texto.lines().count() == CONTRATO.len() + 1);
    }

    #[test]
    fn nome_desconhecido_nao_existe() {
        assert!(por_nome("apagar_vault").is_none());
    }
}
