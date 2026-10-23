//! Sprints do projeto (ciclo 469).
//!
//! Num projeto que cresce, "Feita" acumula centenas de cartões e o que
//! está sendo atacado AGORA some no meio. A sprint é o recorte: nome,
//! objetivo, começo e fim, e o quadro mostra só a ativa.
//!
//! A sprint é PÁGINA, como a tarefa (`<projeto>/sprints/<n>-<nome>.md`,
//! `type: sprint`): aparece em consulta e na sidebar sem nada novo, e se
//! edita como nota. A tarefa aponta pra ela por `sprint: <n>`. Sem o
//! campo, está no backlog.
//!
//! Nada aqui faz IO. Mudar o estado de uma sprint, ou a sprint de uma
//! tarefa, é ação HUMANA, como avançar etapa: o agente propõe.

use serde::{Deserialize, Serialize};

use crate::fluxo::slug_de_titulo;
use crate::projeto::{self, EtapaTarefa, Tarefa};
use crate::{Frontmatter, MarkdownCodec};

/// `type:` da página de uma sprint.
pub const TIPO_SPRINT: &str = "sprint";

/// Onde as sprints de um projeto moram.
pub fn pasta_das_sprints(slug: &str) -> String {
    format!("{}/sprints", projeto::pasta_do_projeto(slug))
}

/// Caminho de uma sprint nova.
pub fn caminho_da_sprint(slug: &str, numero: u32, nome: &str) -> String {
    format!("{}/{numero:02}-{}.md", pasta_das_sprints(slug), slug_de_titulo(nome))
}

/// O caminho é de uma sprint?
pub fn e_caminho_de_sprint(caminho: &str) -> bool {
    match projeto::projeto_do_caminho(caminho) {
        Some(slug) => caminho
            .strip_prefix(&pasta_das_sprints(slug))
            .and_then(|r| r.strip_prefix('/'))
            .is_some_and(|nome| !nome.contains('/') && nome.ends_with(".md")),
        None => false,
    }
}

/// Onde uma sprint está.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EstadoSprint {
    /// Criada, esperando começar. É onde se planeja.
    #[default]
    Planejada,
    /// A que o quadro mostra. Uma por projeto.
    Ativa,
    /// Terminada: as feitas dela saem do quadro e ficam no histórico.
    Encerrada,
}

impl EstadoSprint {
    /// Valor como aparece no frontmatter.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Planejada => "planejada",
            Self::Ativa => "ativa",
            Self::Encerrada => "encerrada",
        }
    }

    /// Nome pra mostrar.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Planejada => "Planejada",
            Self::Ativa => "Ativa",
            Self::Encerrada => "Encerrada",
        }
    }

    /// Lê do frontmatter.
    pub fn from_slug(s: &str) -> Option<Self> {
        Some(match s.trim().to_lowercase().as_str() {
            "planejada" | "planejado" | "planned" | "futura" => Self::Planejada,
            "ativa" | "ativo" | "active" | "em-andamento" => Self::Ativa,
            "encerrada" | "encerrado" | "closed" | "fechada" | "concluida" | "concluída" => Self::Encerrada,
            _ => return None,
        })
    }

    /// Pra onde dá pra ir. Encerrada não reabre: pra continuar o
    /// trabalho, as pendentes vão pra outra sprint ao encerrar.
    pub fn proximas(&self) -> &'static [EstadoSprint] {
        match self {
            Self::Planejada => &[Self::Ativa],
            Self::Ativa => &[Self::Encerrada],
            Self::Encerrada => &[],
        }
    }
}

/// Uma sprint lida do vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sprint {
    /// Caminho da página.
    pub caminho: String,
    /// Slug do projeto (da pasta).
    pub projeto: String,
    /// Número, do nome do arquivo. É por ele que a tarefa aponta.
    pub numero: u32,
    /// Nome.
    pub nome: String,
    /// O que a sprint quer entregar.
    pub objetivo: String,
    /// `AAAA-MM-DD`.
    pub inicio: Option<String>,
    /// `AAAA-MM-DD`.
    pub fim: Option<String>,
    /// Estado. Desconhecido conta como planejada: nunca vira a ativa por
    /// engano.
    pub estado: EstadoSprint,
}

impl Sprint {
    /// Lê uma sprint. `None` se não for `type: sprint` dentro de um
    /// projeto.
    pub fn ler(caminho: &str, texto: &str) -> Option<Sprint> {
        let (fm, _) = MarkdownCodec::split_frontmatter(texto).ok()?;
        if fm.page_type.as_deref() != Some(TIPO_SPRINT) {
            return None;
        }
        let projeto = projeto::projeto_do_caminho(caminho)?.to_string();
        let campo = |k: &str| fm.extra.get(k).and_then(projeto::texto_do_valor);
        Some(Sprint {
            caminho: caminho.to_string(),
            projeto,
            numero: projeto::numero_do_caminho(caminho).unwrap_or(0),
            nome: fm.title.clone().unwrap_or_else(|| caminho.to_string()),
            objetivo: campo("objetivo").unwrap_or_default(),
            inicio: campo("inicio"),
            fim: campo("fim"),
            estado: campo("estado").and_then(|s| EstadoSprint::from_slug(&s)).unwrap_or_default(),
        })
    }
}

/// Monta a página de uma sprint nova, planejada.
pub fn montar_sprint(
    projeto: &str,
    nome: &str,
    objetivo: &str,
    inicio: Option<&str>,
    fim: Option<&str>,
    hoje: &str,
) -> String {
    let mut fm = Frontmatter {
        title: Some(nome.trim().to_string()),
        tags: vec![TIPO_SPRINT.to_string()],
        page_type: Some(TIPO_SPRINT.to_string()),
        ..Default::default()
    };
    fm.extra.insert("projeto".into(), projeto::valor(projeto));
    fm.extra.insert("estado".into(), projeto::valor(EstadoSprint::Planejada.slug()));
    fm.extra.insert("date".into(), projeto::valor(hoje));
    if !objetivo.trim().is_empty() {
        fm.extra.insert("objetivo".into(), projeto::valor(objetivo.trim()));
    }
    for (k, v) in [("inicio", inicio), ("fim", fim)] {
        if let Some(v) = v.map(str::trim).filter(|v| !v.is_empty()) {
            fm.extra.insert(k.into(), projeto::valor(v));
        }
    }
    let objetivo = if objetivo.trim().is_empty() { "O que esta sprint quer entregar." } else { objetivo.trim() };
    projeto::frontmatter_em_texto(&fm, &format!("# {}\n\n## Objetivo\n\n{objetivo}\n", nome.trim()))
}

/// A sprint ativa do projeto.
///
/// Duas ativas só acontecem editando o arquivo à mão. Vale a de número
/// maior (a mais recente), e `avisos` diz que há duas: travar o quadro
/// por isso seria pior do que mostrar a mais nova.
pub fn ativa(sprints: &[Sprint]) -> Option<&Sprint> {
    sprints.iter().filter(|s| s.estado == EstadoSprint::Ativa).max_by_key(|s| s.numero)
}

/// O que está estranho nas sprints, pra tela contar.
pub fn avisos(sprints: &[Sprint], tarefas: &[Tarefa]) -> Vec<String> {
    let mut fora = Vec::new();
    let ativas: Vec<&Sprint> = sprints.iter().filter(|s| s.estado == EstadoSprint::Ativa).collect();
    if ativas.len() > 1 {
        let nomes: Vec<String> = ativas.iter().map(|s| format!("{} ({})", s.nome, s.numero)).collect();
        fora.push(format!("mais de uma sprint ativa: {} — o quadro mostra a mais recente", nomes.join(", ")));
    }
    // O mesmo `O(tarefas × sprints)` do recorte (ciclo 474).
    let existentes: std::collections::HashSet<(&str, u32)> = sprints.iter().map(|s| (s.projeto.as_str(), s.numero)).collect();
    for t in tarefas {
        if let Some(n) = t.sprint {
            if !existentes.contains(&(t.projeto.as_str(), n)) {
                fora.push(format!("{} aponta pra sprint {n}, que não existe — está no backlog", t.caminho));
            }
        }
    }
    fora
}

/// Muda o estado de uma sprint, devolvendo o texto novo.
///
/// Iniciar com outra ativa é RECUSADO, dizendo qual: duas ativas
/// esvaziariam o sentido de "o que está sendo atacado agora".
pub fn mudar_estado(texto: &str, caminho: &str, destino: EstadoSprint, todas: &[Sprint]) -> Result<String, String> {
    let atual = Sprint::ler(caminho, texto).ok_or_else(|| format!("{caminho} não é uma sprint"))?;
    if !atual.estado.proximas().contains(&destino) {
        return Err(format!(
            "a sprint \"{}\" está {}, e daí não vai pra {}",
            atual.nome,
            atual.estado.label().to_lowercase(),
            destino.label().to_lowercase()
        ));
    }
    if destino == EstadoSprint::Ativa {
        if let Some(outra) = todas
            .iter()
            .find(|s| s.estado == EstadoSprint::Ativa && s.projeto == atual.projeto && s.caminho != atual.caminho)
        {
            return Err(format!("já há uma sprint ativa: \"{}\" ({}). Encerre ela antes.", outra.nome, outra.numero));
        }
    }
    MarkdownCodec::set_frontmatter_field(texto, "estado", destino.slug()).map_err(|e| e.to_string())
}

/// Põe a tarefa numa sprint, ou tira (`None` = backlog).
///
/// O número entra como NÚMERO no YAML, pelo `serde_yaml`, e o backlog
/// TIRA o campo em vez de gravar vazio: `sprint: ''` seria lido de novo
/// como "sem sprint", mas deixaria lixo pra cada tarefa que já passou por
/// uma sprint.
pub fn mudar_sprint_da_tarefa(texto: &str, sprint: Option<u32>) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(projeto::TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    match sprint {
        Some(n) => {
            fm.extra.insert("sprint".into(), serde_yaml::Value::Number(n.into()));
        }
        None => {
            fm.extra.remove("sprint");
        }
    }
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

/// O que o quadro mostra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(tag = "filtro", content = "sprint", rename_all = "kebab-case")]
pub enum Filtro {
    /// A sprint ativa. Sem nenhuma ativa, vale `Todas` (RNF4: projeto
    /// sem sprint funciona como antes).
    #[default]
    SprintAtiva,
    /// As tarefas sem sprint.
    Backlog,
    /// Uma sprint específica.
    Sprint(u32),
    /// Tudo, menos as feitas de sprints encerradas, que são histórico.
    Todas,
}

impl Filtro {
    /// Como se lê no seletor.
    pub fn rotulo(&self, sprints: &[Sprint]) -> String {
        match self {
            Self::SprintAtiva => match ativa(sprints) {
                Some(s) => format!("Sprint ativa: {}", s.nome),
                None => "Todas (nenhuma sprint ativa)".to_string(),
            },
            Self::Backlog => "Backlog".to_string(),
            Self::Sprint(n) => match sprints.iter().find(|s| s.numero == *n) {
                Some(s) => format!("Sprint {n}: {}", s.nome),
                None => format!("Sprint {n}"),
            },
            Self::Todas => "Todas".to_string(),
        }
    }

    /// Os filtros que o seletor oferece: a ativa, as planejadas, o
    /// backlog e "todas". As encerradas ficam de fora: são o histórico,
    /// não o quadro (RF5).
    pub fn opcoes(sprints: &[Sprint]) -> Vec<Filtro> {
        let mut v = vec![Filtro::SprintAtiva];
        let mut planejadas: Vec<&Sprint> = sprints.iter().filter(|s| s.estado == EstadoSprint::Planejada).collect();
        planejadas.sort_by_key(|s| s.numero);
        v.extend(planejadas.into_iter().map(|s| Filtro::Sprint(s.numero)));
        v.push(Filtro::Backlog);
        v.push(Filtro::Todas);
        v
    }
}

/// As tarefas que o quadro mostra com este filtro.
///
/// Uma tarefa que aponta pra sprint inexistente conta como backlog: some
/// do quadro da sprint, mas não da vida (e `avisos` diz por quê).
pub fn tarefas_do_quadro<'a>(tarefas: &'a [Tarefa], sprints: &[Sprint], filtro: Filtro) -> Vec<&'a Tarefa> {
    // Um mapa por chamada, e não `sprints.iter().any` por tarefa: as
    // sprints crescem JUNTO com o projeto, e a varredura por tarefa era
    // `O(tarefas × sprints)` — 8× de custo por tarefa entre 300 e 3000
    // no estresse do ciclo 474.
    let estado_de: std::collections::HashMap<u32, EstadoSprint> = sprints.iter().map(|s| (s.numero, s.estado)).collect();
    let existe = |n: u32| estado_de.contains_key(&n);
    let encerrada = |n: u32| estado_de.get(&n) == Some(&EstadoSprint::Encerrada);
    let alvo = match filtro {
        Filtro::SprintAtiva => match ativa(sprints) {
            Some(s) => Filtro::Sprint(s.numero),
            None => Filtro::Todas,
        },
        outro => outro,
    };
    tarefas
        .iter()
        .filter(|t| match alvo {
            Filtro::Sprint(n) => t.sprint == Some(n),
            Filtro::Backlog => t.sprint.is_none_or(|n| !existe(n)),
            Filtro::Todas => !(t.etapa == EtapaTarefa::Feita && t.sprint.is_some_and(encerrada)),
            Filtro::SprintAtiva => unreachable!("resolvido acima"),
        })
        .collect()
}

/// Pra onde vão as pendentes quando a sprint encerra.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DestinoDasPendentes {
    /// A próxima sprint planejada (a de número menor).
    ProximaSprint,
    /// O backlog.
    Backlog,
}

/// O que encerrar uma sprint causa, pra o IO gravar tudo de uma vez.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Encerramento {
    /// A sprint que encerra.
    pub sprint: String,
    /// Cada tarefa pendente e a sprint nova dela (`None` = backlog).
    pub mudancas: Vec<(String, Option<u32>)>,
    /// Quantas feitas ficam no histórico.
    pub feitas: usize,
}

/// Monta o encerramento de `sprint`.
///
/// "Pendente" é tudo que não está `feita`. Uma tarefa em `executando`
/// também muda de sprint: o agente dela continua, e o resultado cai onde
/// a tarefa estiver.
pub fn plano_de_encerramento(
    sprint: &Sprint,
    sprints: &[Sprint],
    tarefas: &[Tarefa],
    destino: DestinoDasPendentes,
) -> Result<Encerramento, String> {
    if sprint.estado != EstadoSprint::Ativa {
        return Err(format!("só se encerra a sprint ativa; \"{}\" está {}", sprint.nome, sprint.estado.label().to_lowercase()));
    }
    let nova = match destino {
        DestinoDasPendentes::Backlog => None,
        DestinoDasPendentes::ProximaSprint => Some(
            sprints
                .iter()
                .filter(|s| s.projeto == sprint.projeto && s.estado == EstadoSprint::Planejada)
                .min_by_key(|s| s.numero)
                .map(|s| s.numero)
                .ok_or_else(|| "não há sprint planejada pra receber as pendentes — crie uma, ou mande pro backlog".to_string())?,
        ),
    };
    let da_sprint: Vec<&Tarefa> = tarefas.iter().filter(|t| t.sprint == Some(sprint.numero)).collect();
    Ok(Encerramento {
        sprint: sprint.caminho.clone(),
        mudancas: da_sprint
            .iter()
            .filter(|t| t.etapa != EtapaTarefa::Feita)
            .map(|t| (t.caminho.clone(), nova))
            .collect(),
        feitas: da_sprint.iter().filter(|t| t.etapa == EtapaTarefa::Feita).count(),
    })
}

/// O resumo de uma sprint pro histórico: feitas e total.
pub fn resumo(sprint: &Sprint, tarefas: &[Tarefa]) -> (usize, usize) {
    PorSprint::novo(tarefas).resumo(sprint.numero)
}

/// As tarefas agrupadas por sprint, numa passada (ciclo 474).
///
/// Listar as sprints com `resumo` cada uma varre o projeto uma vez POR
/// SPRINT — e o histórico é justamente a lista que cresce. Quem mostra
/// várias sprints monta isto uma vez e pergunta a cada uma.
pub struct PorSprint<'a> {
    grupos: std::collections::HashMap<u32, Vec<&'a Tarefa>>,
}

impl<'a> PorSprint<'a> {
    /// Agrupa numa passada; tarefa sem sprint fica de fora.
    pub fn novo(tarefas: &'a [Tarefa]) -> Self {
        let mut grupos: std::collections::HashMap<u32, Vec<&'a Tarefa>> = std::collections::HashMap::new();
        for t in tarefas {
            if let Some(n) = t.sprint {
                grupos.entry(n).or_default().push(t);
            }
        }
        Self { grupos }
    }

    /// As tarefas da sprint `numero`, na ordem em que vieram.
    pub fn tarefas(&self, numero: u32) -> &[&'a Tarefa] {
        self.grupos.get(&numero).map(Vec::as_slice).unwrap_or(&[])
    }

    /// `(feitas, total)` da sprint `numero`.
    pub fn resumo(&self, numero: u32) -> (usize, usize) {
        let da = self.tarefas(numero);
        (da.iter().filter(|t| t.etapa == EtapaTarefa::Feita).count(), da.len())
    }
}

#[cfg(test)]
mod testes {
    use super::*;

    fn sprint(n: u32, estado: &str) -> Sprint {
        let caminho = format!("pages/projetos/saas/sprints/{n:02}-s.md");
        Sprint::ler(&caminho, &format!("---\ntitle: Sprint {n}\ntype: sprint\nprojeto: saas\nestado: {estado}\n---\n# S\n")).unwrap()
    }

    fn tarefa(n: u32, etapa: &str, sprint: Option<u32>) -> Tarefa {
        let caminho = format!("pages/projetos/saas/tarefas/{n:03}-t.md");
        let extra = sprint.map(|s| format!("sprint: {s}\n")).unwrap_or_default();
        Tarefa::ler(&caminho, &format!("---\ntitle: T{n}\ntype: tarefa\nprojeto: saas\nstatus: {etapa}\n{extra}---\n# T\n")).unwrap()
    }

    fn numeros(v: Vec<&Tarefa>) -> Vec<u32> {
        v.into_iter().map(|t| t.numero).collect()
    }

    #[test]
    fn a_sprint_nasce_planejada_e_se_le_de_volta() {
        let md = montar_sprint("saas", "Fundação: login", "Login e banco", Some("2026-10-22"), Some("2026-11-05"), "2026-10-22");
        let s = Sprint::ler("pages/projetos/saas/sprints/01-fundacao-login.md", &md).unwrap();
        assert_eq!((s.numero, s.estado), (1, EstadoSprint::Planejada));
        assert_eq!(s.nome, "Fundação: login", "título com dois-pontos é YAML válido");
        assert_eq!(s.objetivo, "Login e banco");
        assert_eq!(s.fim.as_deref(), Some("2026-11-05"));
        assert_eq!(caminho_da_sprint("saas", 1, "Fundação: login"), "pages/projetos/saas/sprints/01-fundacao-login.md");
        assert!(e_caminho_de_sprint("pages/projetos/saas/sprints/01-x.md"));
        assert!(!e_caminho_de_sprint("pages/projetos/saas/tarefas/001-x.md"));
    }

    #[test]
    fn so_uma_ativa_por_projeto() {
        let a = sprint(1, "ativa");
        let b = sprint(2, "planejada");
        let todas = vec![a.clone(), b.clone()];
        let texto = "---\ntitle: Sprint 2\ntype: sprint\nprojeto: saas\nestado: planejada\n---\n# S\n";
        let erro = mudar_estado(texto, &b.caminho, EstadoSprint::Ativa, &todas).unwrap_err();
        assert!(erro.contains("Sprint 1"), "o erro tem que dizer qual está ativa: {erro}");
        // Encerrada a 1, a 2 pode começar.
        let todas = vec![Sprint { estado: EstadoSprint::Encerrada, ..a }, b.clone()];
        let novo = mudar_estado(texto, &b.caminho, EstadoSprint::Ativa, &todas).unwrap();
        assert_eq!(Sprint::ler(&b.caminho, &novo).unwrap().estado, EstadoSprint::Ativa);
    }

    #[test]
    fn encerrada_nao_reabre_e_planejada_nao_encerra() {
        let texto = "---\ntitle: S\ntype: sprint\nprojeto: saas\nestado: encerrada\n---\n";
        assert!(mudar_estado(texto, "pages/projetos/saas/sprints/01-s.md", EstadoSprint::Ativa, &[]).is_err());
        let texto = "---\ntitle: S\ntype: sprint\nprojeto: saas\nestado: planejada\n---\n";
        assert!(mudar_estado(texto, "pages/projetos/saas/sprints/01-s.md", EstadoSprint::Encerrada, &[]).is_err());
    }

    #[test]
    fn o_quadro_mostra_a_ativa_e_esconde_o_historico() {
        let sprints = vec![sprint(1, "encerrada"), sprint(2, "ativa"), sprint(3, "planejada")];
        let tarefas = vec![
            tarefa(1, "feita", Some(1)),   // histórico
            tarefa(2, "pronta", Some(1)),  // pendente de sprint encerrada: aparece em "todas"
            tarefa(3, "executar", Some(2)),
            tarefa(4, "feita", Some(2)),   // feita da ATIVA: aparece
            tarefa(5, "ideia", Some(3)),
            tarefa(6, "ideia", None),
            tarefa(7, "ideia", Some(9)),   // sprint que não existe: backlog
        ];
        assert_eq!(numeros(tarefas_do_quadro(&tarefas, &sprints, Filtro::SprintAtiva)), [3, 4]);
        assert_eq!(numeros(tarefas_do_quadro(&tarefas, &sprints, Filtro::Backlog)), [6, 7]);
        assert_eq!(numeros(tarefas_do_quadro(&tarefas, &sprints, Filtro::Sprint(3))), [5]);
        assert_eq!(numeros(tarefas_do_quadro(&tarefas, &sprints, Filtro::Todas)), [2, 3, 4, 5, 6, 7]);
        assert!(avisos(&sprints, &tarefas).iter().any(|a| a.contains("sprint 9")));
    }

    #[test]
    fn sem_sprint_nenhuma_o_quadro_e_o_de_antes() {
        let tarefas = vec![tarefa(1, "feita", None), tarefa(2, "ideia", None)];
        assert_eq!(numeros(tarefas_do_quadro(&tarefas, &[], Filtro::SprintAtiva)), [1, 2]);
        assert_eq!(Filtro::SprintAtiva.rotulo(&[]), "Todas (nenhuma sprint ativa)");
    }

    #[test]
    fn o_seletor_nao_oferece_as_encerradas() {
        let sprints = vec![sprint(1, "encerrada"), sprint(2, "ativa"), sprint(4, "planejada"), sprint(3, "planejada")];
        assert_eq!(
            Filtro::opcoes(&sprints),
            [Filtro::SprintAtiva, Filtro::Sprint(3), Filtro::Sprint(4), Filtro::Backlog, Filtro::Todas]
        );
    }

    #[test]
    fn encerrar_manda_as_pendentes_pra_proxima_ou_pro_backlog() {
        let sprints = vec![sprint(1, "ativa"), sprint(3, "planejada"), sprint(2, "planejada")];
        let tarefas = vec![tarefa(1, "feita", Some(1)), tarefa(2, "pronta", Some(1)), tarefa(3, "executando", Some(1)), tarefa(4, "ideia", None)];
        let e = plano_de_encerramento(&sprints[0], &sprints, &tarefas, DestinoDasPendentes::ProximaSprint).unwrap();
        assert_eq!(e.feitas, 1);
        // A próxima é a planejada de número MENOR, não a primeira da lista.
        assert_eq!(
            e.mudancas,
            [("pages/projetos/saas/tarefas/002-t.md".to_string(), Some(2)), ("pages/projetos/saas/tarefas/003-t.md".to_string(), Some(2))]
        );
        let b = plano_de_encerramento(&sprints[0], &sprints, &tarefas, DestinoDasPendentes::Backlog).unwrap();
        assert!(b.mudancas.iter().all(|(_, s)| s.is_none()));
    }

    #[test]
    fn encerrar_sem_planejada_pra_receber_e_erro() {
        let sprints = vec![sprint(1, "ativa")];
        let erro = plano_de_encerramento(&sprints[0], &sprints, &[], DestinoDasPendentes::ProximaSprint).unwrap_err();
        assert!(erro.contains("backlog"), "{erro}");
        // E só se encerra a ativa.
        let p = sprint(2, "planejada");
        assert!(plano_de_encerramento(&p, &[p.clone()], &[], DestinoDasPendentes::Backlog).is_err());
    }

    #[test]
    fn mudar_sprint_grava_numero_e_backlog_tira_o_campo() {
        let md = "---\ntitle: T\ntype: tarefa\nprojeto: saas\nstatus: ideia\n---\n# T\n\ncorpo\n";
        let com = mudar_sprint_da_tarefa(md, Some(3)).unwrap();
        assert!(com.contains("sprint: 3\n"), "número, não texto:\n{com}");
        assert_eq!(Tarefa::ler("pages/projetos/saas/tarefas/001-t.md", &com).unwrap().sprint, Some(3));
        let sem = mudar_sprint_da_tarefa(&com, None).unwrap();
        assert!(!sem.contains("sprint"), "{sem}");
        assert!(sem.ends_with("# T\n\ncorpo\n"), "o corpo mudou:\n{sem}");
        assert!(mudar_sprint_da_tarefa("---\ntype: spec\n---\n", Some(1)).is_err());
    }

    #[test]
    fn duas_ativas_vale_a_mais_recente_com_aviso() {
        let sprints = vec![sprint(1, "ativa"), sprint(2, "ativa")];
        assert_eq!(ativa(&sprints).map(|s| s.numero), Some(2));
        assert!(avisos(&sprints, &[]).iter().any(|a| a.contains("mais de uma")));
    }
}
