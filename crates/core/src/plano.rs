//! O plano de um projeto (ciclo 469): o formato com que uma LLM monta a
//! base inteira — projeto, arquitetura, sprints e tarefas — de uma vez.
//!
//! Sem isto, quem pede ao Claude Code ou ao opencode "monte este
//! projeto" ganha o agente adivinhando, página a página, o frontmatter de
//! cada tipo. Com isto ele preenche UM arquivo num formato documentado
//! (`MODELO_DE_PLANO`, que ele mesmo consegue pedir ao CLI), e o núcleo
//! gera as páginas certas, numeradas a partir do que já existe.
//!
//! As páginas geradas ENTRAM POR PROPOSTA, num lote só (RNF1 da spec): o
//! plano inteiro é uma decisão de quem revisa.
//!
//! `deny_unknown_fields` em tudo: um `critérios` com acento, ou um
//! `sprint_nome` inventado, vira erro com o nome do campo — e não um
//! campo que some calado, que é o pior caso pra quem escreve YAML por
//! geração.

use serde::{Deserialize, Serialize};

use crate::projeto;
use crate::sprint;
use crate::MarkdownCodec;

/// O plano inteiro.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanoDeProjeto {
    /// O projeto. Num projeto que já existe, só o `slug` importa: as
    /// páginas dele não são tocadas.
    pub projeto: PlanoProjeto,
    /// Markdown da arquitetura (sem o título).
    #[serde(default)]
    pub arquitetura: Option<String>,
    /// As sprints, na ordem.
    #[serde(default)]
    pub sprints: Vec<PlanoSprint>,
    /// As tarefas, na ordem.
    #[serde(default)]
    pub tarefas: Vec<PlanoTarefa>,
}

/// O projeto no plano.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanoProjeto {
    /// Título.
    pub titulo: String,
    /// Nome da pasta. Omitido, sai do título.
    #[serde(default)]
    pub slug: Option<String>,
    /// O que é, pra quem.
    #[serde(default)]
    pub visao: String,
    /// O que precisa estar no ar pra valer a pena.
    #[serde(default)]
    pub objetivo: String,
    /// Tecnologias.
    #[serde(default)]
    pub stack: Vec<String>,
    /// O que vale pra toda tarefa (commitar, testar, nunca mexer).
    #[serde(default)]
    pub combinados: Vec<String>,
}

/// Uma sprint no plano.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanoSprint {
    /// Nome. É por ele que as tarefas do plano apontam.
    pub nome: String,
    /// O que entrega.
    #[serde(default)]
    pub objetivo: String,
    /// `AAAA-MM-DD`.
    #[serde(default)]
    pub inicio: Option<String>,
    /// `AAAA-MM-DD`.
    #[serde(default)]
    pub fim: Option<String>,
}

/// Uma tarefa no plano.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanoTarefa {
    /// Título. É por ele que `depende` aponta.
    pub titulo: String,
    /// Nome de uma sprint do plano (ou de uma que já existe, ou o número
    /// dela). Omitido: backlog.
    #[serde(default)]
    pub sprint: Option<String>,
    /// `alta`, `media` ou `baixa`.
    #[serde(default)]
    pub prioridade: Option<String>,
    /// Títulos de outras tarefas do plano (ou referências a tarefas que
    /// já existem: número, nome do arquivo, título).
    #[serde(default)]
    pub depende: Vec<String>,
    /// O que precisa existir quando terminar.
    #[serde(default)]
    pub objetivo: String,
    /// Critérios de aceite, um por item, verificáveis.
    #[serde(default)]
    pub criterios: Vec<String>,
    /// Como fazer.
    #[serde(default)]
    pub abordagem: String,
    /// O que a tarefa NÃO cobre.
    #[serde(default)]
    pub fora_de_escopo: Vec<String>,
    /// Tags (ciclo 476).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Prazo, `AAAA-MM-DD` (ciclo 476).
    #[serde(default)]
    pub prazo: Option<String>,
}

/// O modelo que a LLM pede (`anotadinho-cli projeto modelo-de-plano`).
///
/// Ele É o contrato: um teste lê este texto com `ler_plano` e confere que
/// vira páginas que `Tarefa::ler` e `Sprint::ler` entendem. Mudar o
/// formato sem mudar o modelo reprova.
pub const MODELO_DE_PLANO: &str = r#"# Plano de projeto do Anotadinho (YAML).
# Campos desconhecidos são RECUSADOS com o nome do campo: confira a grafia.
projeto:
  titulo: Agenda Fácil
  slug: agenda                 # opcional: sai do título
  visao: Agendamento online pra salões pequenos, sem instalar nada.
  objetivo: Um salão consegue receber 20 agendamentos por semana sozinho.
  stack: [Rust, Axum, Postgres, HTMX]
  combinados:
    - Commit por tarefa, com a mensagem dizendo o critério atendido.
    - Testes passam antes de dizer que terminou.

arquitetura: |
  ## Componentes
  - API em Axum; páginas em HTMX servidas pela própria API.
  ## Dados
  - Postgres: salao, servico, horario, agendamento.

sprints:
  - nome: Fundação
    objetivo: O dono do salão entra e cadastra os serviços.
    inicio: 2026-10-27        # opcional, AAAA-MM-DD
    fim: 2026-11-07           # opcional
  - nome: Agendamento
    objetivo: O cliente agenda sozinho pelo link do salão.

tarefas:
  - titulo: Banco de dados e migrações
    sprint: Fundação           # nome de uma sprint acima; omitido = backlog
    prioridade: alta           # alta | media | baixa
    tags: [banco]              # opcional
    prazo: 2026-11-03          # opcional, AAAA-MM-DD
    objetivo: As tabelas existem e migram do zero.
    criterios:
      - "`cargo sqlx migrate run` num banco vazio cria as 4 tabelas"
    abordagem: sqlx com migrações versionadas no repositório.
    fora_de_escopo:
      - dados de exemplo
  - titulo: Login do dono do salão
    sprint: Fundação
    depende: [Banco de dados e migrações]   # títulos de tarefas deste plano
    objetivo: O dono entra com e-mail e senha.
    criterios:
      - senha errada mostra erro sem dizer se o e-mail existe
    abordagem: sessão em cookie assinado.
"#;

/// Lê um plano, com um erro que diz o que corrigir.
pub fn ler_plano(texto: &str) -> Result<PlanoDeProjeto, String> {
    serde_yaml::from_str(texto).map_err(|e| format!("plano inválido: {e}. Peça o modelo com `anotadinho-cli projeto modelo-de-plano`."))
}

/// O que já existe no projeto, pra o plano não colidir.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct JaExiste {
    /// A página do projeto existe: o plano não a recria (nem a
    /// arquitetura, nem a conversa).
    pub projeto: bool,
    /// O próximo número livre de tarefa.
    pub proxima_tarefa: u32,
    /// O próximo número livre de sprint.
    pub proxima_sprint: u32,
    /// As sprints que já existem: (nome, número).
    pub sprints: Vec<(String, u32)>,
}

/// As páginas que o plano gera.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginasDoPlano {
    /// O slug do projeto.
    pub slug: String,
    /// `(caminho, conteúdo)`, na ordem: projeto, sprints, tarefas.
    pub paginas: Vec<(String, String)>,
    /// O que não impediu, mas vale dizer.
    pub avisos: Vec<String>,
}

fn lista(itens: &[String], marcador: &str) -> String {
    if itens.is_empty() {
        return format!("{marcador}\n");
    }
    itens.iter().map(|i| format!("{marcador}{}\n", i.trim())).collect()
}

/// Troca o corpo de uma página gerada mantendo o frontmatter dela.
fn com_corpo(pagina: &str, corpo: &str) -> String {
    let (fm, _) = MarkdownCodec::split_frontmatter_text(pagina);
    format!("{fm}\n{corpo}")
}

/// Gera as páginas do plano.
///
/// Os números de tarefa e de sprint começam em `ja.proxima_*`: a LLM não
/// sabe quais estão livres, e o plano nunca diz número. `sprint` e
/// `depende` se resolvem pelo TÍTULO do plano. Referência que não é do
/// plano passa adiante como está (pode ser uma tarefa que já existe), e
/// sprint que não é do plano nem existe é ERRO, porque a tarefa cairia no
/// backlog sem ninguém saber.
pub fn montar_plano(plano: &PlanoDeProjeto, ja: &JaExiste, hoje: &str) -> Result<PaginasDoPlano, String> {
    let titulo = plano.projeto.titulo.trim();
    if titulo.is_empty() {
        return Err("o plano precisa de `projeto.titulo`".to_string());
    }
    let slug = crate::fluxo::slug_de_titulo(plano.projeto.slug.as_deref().unwrap_or(titulo));
    let mut avisos = Vec::new();
    let mut paginas = Vec::new();

    // Títulos repetidos tornariam `depende` ambíguo.
    for (i, t) in plano.tarefas.iter().enumerate() {
        if t.titulo.trim().is_empty() {
            return Err(format!("a tarefa {} do plano está sem título", i + 1));
        }
        if plano.tarefas[..i].iter().any(|o| o.titulo.trim().eq_ignore_ascii_case(t.titulo.trim())) {
            return Err(format!("duas tarefas com o título \"{}\": `depende` não saberia qual", t.titulo.trim()));
        }
    }
    for (i, s) in plano.sprints.iter().enumerate() {
        if s.nome.trim().is_empty() {
            return Err(format!("a sprint {} do plano está sem nome", i + 1));
        }
        if plano.sprints[..i].iter().any(|o| o.nome.trim().eq_ignore_ascii_case(s.nome.trim())) {
            return Err(format!("duas sprints com o nome \"{}\"", s.nome.trim()));
        }
    }

    // O projeto, se ainda não existe.
    if ja.projeto {
        if plano.arquitetura.is_some() {
            avisos.push("o projeto já existe: a arquitetura do plano não substitui a que está lá".to_string());
        }
    } else {
        let base = projeto::montar_projeto(&slug, titulo, hoje);
        for (caminho, conteudo) in base {
            let conteudo = if caminho == projeto::caminho_do_projeto(&slug) {
                let p = &plano.projeto;
                com_corpo(
                    &conteudo,
                    &format!(
                        "# {titulo}\n\n## Visão\n\n{}\n\n## Objetivo\n\n{}\n\n## Stack\n\n{}\n## Combinados com o agente\n\n{}",
                        if p.visao.trim().is_empty() { "O que é, pra quem, e que problema resolve." } else { p.visao.trim() },
                        if p.objetivo.trim().is_empty() { "O que precisa estar no ar pra este projeto ter valido a pena." } else { p.objetivo.trim() },
                        lista(&p.stack, "- "),
                        lista(&p.combinados, "- "),
                    ),
                )
            } else if caminho == projeto::caminho_da_arquitetura(&slug) {
                match plano.arquitetura.as_deref().map(str::trim).filter(|a| !a.is_empty()) {
                    Some(a) => com_corpo(&conteudo, &format!("# Arquitetura — {titulo}\n\n{a}\n")),
                    None => conteudo,
                }
            } else {
                conteudo
            };
            paginas.push((caminho, conteudo));
        }
    }

    // As sprints, numeradas depois das que existem.
    let mut numero_da_sprint: Vec<(String, u32)> = ja.sprints.clone();
    for (i, s) in plano.sprints.iter().enumerate() {
        let n = ja.proxima_sprint.max(1) + i as u32;
        let caminho = sprint::caminho_da_sprint(&slug, n, &s.nome);
        let conteudo = sprint::montar_sprint(&slug, &s.nome, &s.objetivo, s.inicio.as_deref(), s.fim.as_deref(), hoje);
        numero_da_sprint.push((s.nome.trim().to_string(), n));
        paginas.push((caminho, conteudo));
    }
    let achar_sprint = |r: &str| -> Option<u32> {
        let r = r.trim();
        r.parse::<u32>()
            .ok()
            .filter(|n| numero_da_sprint.iter().any(|(_, m)| m == n))
            .or_else(|| numero_da_sprint.iter().find(|(nome, _)| nome.eq_ignore_ascii_case(r)).map(|(_, n)| *n))
    };

    // As tarefas, numeradas depois das que existem.
    let primeira = ja.proxima_tarefa.max(1);
    let numero_da_tarefa = |titulo: &str| -> Option<u32> {
        plano.tarefas.iter().position(|t| t.titulo.trim().eq_ignore_ascii_case(titulo.trim())).map(|i| primeira + i as u32)
    };
    for (i, t) in plano.tarefas.iter().enumerate() {
        let n = primeira + i as u32;
        let titulo_t = t.titulo.trim();
        let corpo = format!(
            "## Objetivo\n\n{}\n\n## Critérios de aceite\n\n{}\n## Abordagem\n\n{}\n\n## Fora de escopo\n\n{}",
            if t.objetivo.trim().is_empty() { "O que precisa existir quando a tarefa terminar, e por quê." } else { t.objetivo.trim() },
            lista(&t.criterios, "- [ ] "),
            if t.abordagem.trim().is_empty() { "Como fazer." } else { t.abordagem.trim() },
            lista(&t.fora_de_escopo, "- "),
        );
        let md = projeto::montar_tarefa(&slug, titulo_t, &corpo, hoje);
        let (mut fm, _) = MarkdownCodec::split_frontmatter(&md).map_err(|e| e.to_string())?;
        if let Some(p) = t.prioridade.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
            if !matches!(p, "alta" | "media" | "baixa") {
                avisos.push(format!("\"{titulo_t}\": prioridade \"{p}\" não é alta, media ou baixa — a fila trata como media"));
            }
            fm.extra.insert("prioridade".into(), projeto::valor(p));
        }
        if let Some(r) = t.sprint.as_deref().map(str::trim).filter(|r| !r.is_empty()) {
            let n_sprint = achar_sprint(r).ok_or_else(|| {
                format!("\"{titulo_t}\" aponta pra sprint \"{r}\", que não está no plano nem no projeto")
            })?;
            fm.extra.insert("sprint".into(), serde_yaml::Value::Number(n_sprint.into()));
        }
        let mut depende = Vec::new();
        for d in &t.depende {
            if d.trim().eq_ignore_ascii_case(titulo_t) {
                return Err(format!("\"{titulo_t}\" depende de si mesma"));
            }
            depende.push(match numero_da_tarefa(d) {
                Some(m) => serde_yaml::Value::Number(m.into()),
                None => {
                    avisos.push(format!("\"{titulo_t}\" depende de \"{}\", que não é do plano: fica como referência a tarefa existente", d.trim()));
                    projeto::valor(d.trim())
                }
            });
        }
        if !depende.is_empty() {
            fm.extra.insert("depende".into(), serde_yaml::Value::Sequence(depende));
        }
        let mut md = MarkdownCodec::substituir_frontmatter(&md, &fm).map_err(|e| e.to_string())?;
        if !t.tags.is_empty() {
            md = projeto::com_tags(&md, &t.tags)?;
        }
        if let Some(p) = t.prazo.as_deref().filter(|p| !p.trim().is_empty()) {
            md = projeto::com_prazo(&md, Some(p)).map_err(|e| format!("\"{titulo_t}\": {e}"))?;
        }
        paginas.push((projeto::caminho_da_tarefa(&slug, n, titulo_t), md));
    }

    Ok(PaginasDoPlano { slug, paginas, avisos })
}

#[cfg(test)]
mod testes {
    use super::*;
    use crate::projeto::{EtapaTarefa, Tarefa};
    use crate::sprint::{EstadoSprint, Sprint};

    fn novo() -> JaExiste {
        JaExiste { projeto: false, proxima_tarefa: 1, proxima_sprint: 1, sprints: vec![] }
    }

    #[test]
    fn prazo_invalido_no_plano_e_recusado_dizendo_qual_tarefa() {
        let plano = ler_plano("projeto:\n  titulo: A\ntarefas:\n  - titulo: T\n    prazo: 5/11\n").unwrap();
        let erro = montar_plano(&plano, &novo(), "2026-10-22").unwrap_err();
        assert!(erro.contains("\"T\"") && erro.contains("5/11"), "{erro}");
    }

    #[test]
    fn o_modelo_vira_paginas_que_o_app_entende() {
        // O modelo É o contrato com a LLM: se ele não virar páginas
        // legíveis, o comando que o imprime está ensinando errado.
        let plano = ler_plano(MODELO_DE_PLANO).unwrap();
        let r = montar_plano(&plano, &novo(), "2026-10-22").unwrap();
        assert_eq!(r.slug, "agenda");
        let caminhos: Vec<&str> = r.paginas.iter().map(|(c, _)| c.as_str()).collect();
        assert!(caminhos.contains(&"pages/projetos/agenda/projeto.md"));
        assert!(caminhos.contains(&"pages/projetos/agenda/sprints/01-fundacao.md"));
        assert!(caminhos.contains(&"pages/projetos/agenda/sprints/02-agendamento.md"));
        let tarefa = |c: &str| {
            let (_, md) = r.paginas.iter().find(|(x, _)| x == c).unwrap();
            Tarefa::ler(c, md).unwrap()
        };
        let login = tarefa("pages/projetos/agenda/tarefas/002-login-do-dono-do-salao.md");
        assert_eq!(login.sprint, Some(1));
        assert_eq!(login.depende, ["1"], "depende pelo título vira o número");
        assert_eq!(login.etapa, EtapaTarefa::Ideia);
        let banco = tarefa("pages/projetos/agenda/tarefas/001-banco-de-dados-e-migracoes.md");
        assert_eq!(banco.prioridade.as_deref(), Some("alta"));
        // Tags e prazo do modelo chegam na página (ciclo 476), e a
        // checklist é a dos critérios.
        assert_eq!(banco.tags, ["banco"]);
        assert_eq!(banco.prazo.as_deref(), Some("2026-11-03"));
        assert_eq!(banco.progresso(), (0, 1));
        let (_, md) = r.paginas.iter().find(|(c, _)| c.ends_with("01-fundacao.md")).unwrap();
        let s = Sprint::ler("pages/projetos/agenda/sprints/01-fundacao.md", md).unwrap();
        assert_eq!((s.estado, s.inicio.as_deref()), (EstadoSprint::Planejada, Some("2026-10-27")));
        let (_, projeto) = r.paginas.iter().find(|(c, _)| c.ends_with("projeto.md")).unwrap();
        assert!(projeto.contains("- Axum") && projeto.contains("Commit por tarefa"), "{projeto}");
        let (_, arq) = r.paginas.iter().find(|(c, _)| c.ends_with("arquitetura.md")).unwrap();
        assert!(arq.contains("Postgres: salao"), "{arq}");
        assert!(r.avisos.is_empty(), "{:?}", r.avisos);
    }

    #[test]
    fn campo_errado_e_erro_com_o_nome_do_campo() {
        let erro = ler_plano("projeto:\n  titulo: X\ntarefas:\n  - titulo: A\n    critérios: [x]\n").unwrap_err();
        assert!(erro.contains("critérios"), "{erro}");
        assert!(erro.contains("modelo-de-plano"), "o erro tem que dizer onde ver o formato: {erro}");
    }

    #[test]
    fn num_projeto_que_existe_numera_depois_e_nao_recria() {
        let plano = ler_plano("projeto:\n  titulo: Agenda\n  slug: agenda\narquitetura: nova\nsprints:\n  - nome: Dois\ntarefas:\n  - titulo: Nova\n    sprint: Um\n  - titulo: Outra\n    sprint: Dois\n").unwrap();
        let ja = JaExiste { projeto: true, proxima_tarefa: 8, proxima_sprint: 2, sprints: vec![("Um".into(), 1)] };
        let r = montar_plano(&plano, &ja, "2026-10-22").unwrap();
        let caminhos: Vec<&str> = r.paginas.iter().map(|(c, _)| c.as_str()).collect();
        assert_eq!(
            caminhos,
            ["pages/projetos/agenda/sprints/02-dois.md", "pages/projetos/agenda/tarefas/008-nova.md", "pages/projetos/agenda/tarefas/009-outra.md"]
        );
        let sprint_de = |i: usize| Tarefa::ler(&r.paginas[i].0, &r.paginas[i].1).unwrap().sprint;
        assert_eq!((sprint_de(1), sprint_de(2)), (Some(1), Some(2)), "sprint existente e do plano, pelo nome");
        assert!(r.avisos.iter().any(|a| a.contains("arquitetura")), "{:?}", r.avisos);
    }

    #[test]
    fn sprint_que_nao_existe_e_erro_e_nao_backlog_calado() {
        let plano = ler_plano("projeto:\n  titulo: A\ntarefas:\n  - titulo: T\n    sprint: Fantasma\n").unwrap();
        let erro = montar_plano(&plano, &novo(), "hoje").unwrap_err();
        assert!(erro.contains("Fantasma"), "{erro}");
    }

    #[test]
    fn titulo_repetido_e_dependencia_de_si_sao_erro() {
        let rep = ler_plano("projeto:\n  titulo: A\ntarefas:\n  - titulo: T\n  - titulo: t\n").unwrap();
        assert!(montar_plano(&rep, &novo(), "hoje").unwrap_err().contains("duas tarefas"));
        let si = ler_plano("projeto:\n  titulo: A\ntarefas:\n  - titulo: T\n    depende: [T]\n").unwrap();
        assert!(montar_plano(&si, &novo(), "hoje").unwrap_err().contains("si mesma"));
    }

    #[test]
    fn dependencia_fora_do_plano_passa_como_referencia_com_aviso() {
        let plano = ler_plano("projeto:\n  titulo: A\ntarefas:\n  - titulo: T\n    depende: [\"3\"]\n    prioridade: urgente\n").unwrap();
        let r = montar_plano(&plano, &novo(), "hoje").unwrap();
        let t = r.paginas.iter().find(|(c, _)| c.contains("/tarefas/")).unwrap();
        assert_eq!(Tarefa::ler(&t.0, &t.1).unwrap().depende, ["3"]);
        assert_eq!(r.avisos.len(), 2, "{:?}", r.avisos);
    }
}
