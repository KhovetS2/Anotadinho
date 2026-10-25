//! Projeto: planejar e construir com o agente num lugar só (ciclo 457).
//!
//! Um projeto é uma pasta em `pages/projetos/<slug>/` com a página do
//! projeto, a arquitetura, as tarefas e as conversas. A tarefa é um tipo
//! LEVE: junta o "o quê" (objetivo, critérios) e o "como" (abordagem) na
//! mesma página, porque exigir spec e proposta pra cada tarefa pequena
//! matava o ritmo. Tarefa grande aponta pra uma spec ou proposta que já
//! existe.
//!
//! A etapa mora no `status:` do frontmatter, e não num embed: é por ele
//! que `query`, `list-pages --status` e a tabela de tarefas filtram, então
//! a tarefa aparece em todo lugar sem nada novo.
//!
//! Este módulo não faz IO. O que GUI, TUI e CLI precisam concordar (que
//! etapas existem, quem move pra cada uma, o que entra no prompt, como se
//! lê o resultado) mora aqui, uma vez só.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::conversa::Contexto;
use crate::fluxo::slug_de_titulo;
use crate::{Frontmatter, MarkdownCodec};

/// Onde os projetos moram.
pub const PASTA: &str = "pages/projetos";

/// `type:` da página do projeto.
pub const TIPO_PROJETO: &str = "projeto";

/// `type:` da página de uma tarefa.
pub const TIPO_TAREFA: &str = "tarefa";

/// Onde uma tarefa está.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EtapaTarefa {
    /// Anotada, ainda sem detalhe.
    #[default]
    Ideia,
    /// Sendo detalhada (pela pessoa ou pelo agente, por proposta).
    Detalhando,
    /// O detalhe foi aceito: dá pra mandar executar.
    Pronta,
    /// Pedido de execução: a tarefa está na fila do projeto.
    Executar,
    /// O agente está trabalhando nela.
    Executando,
    /// O agente terminou e o relato espera alguém ler.
    Revisao,
    /// Aceita.
    Feita,
    /// Parada, com o motivo na conversa da tarefa.
    Bloqueada,
}

/// Quem está movendo a etapa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Quem {
    /// A pessoa, por qualquer canal: GUI, TUI, CLI ou editando o arquivo.
    Humano,
    /// O app, como CONSEQUÊNCIA de uma decisão humana: reservar a tarefa
    /// ao começar, fechar ao terminar. O agente nunca é `Quem`: ele não
    /// grava etapa nenhuma.
    Sistema,
}

impl EtapaTarefa {
    /// Todas, na ordem do quadro.
    pub fn all() -> &'static [EtapaTarefa] {
        &[
            Self::Ideia,
            Self::Detalhando,
            Self::Pronta,
            Self::Executar,
            Self::Executando,
            Self::Revisao,
            Self::Feita,
            Self::Bloqueada,
        ]
    }

    /// Valor como aparece no frontmatter.
    pub fn slug(&self) -> &'static str {
        match self {
            Self::Ideia => "ideia",
            Self::Detalhando => "detalhando",
            Self::Pronta => "pronta",
            Self::Executar => "executar",
            Self::Executando => "executando",
            Self::Revisao => "revisao",
            Self::Feita => "feita",
            Self::Bloqueada => "bloqueada",
        }
    }

    /// Nome pra mostrar.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Ideia => "Ideia",
            Self::Detalhando => "Detalhando",
            Self::Pronta => "Pronta",
            Self::Executar => "Executar",
            Self::Executando => "Executando",
            Self::Revisao => "Revisão",
            Self::Feita => "Feita",
            Self::Bloqueada => "Bloqueada",
        }
    }

    /// Lê do frontmatter. Aceita acento e as grafias que a mão escreve,
    /// porque editar o arquivo direto é um canal legítimo de mudar etapa.
    pub fn from_slug(s: &str) -> Option<Self> {
        let s = s.trim().to_lowercase();
        Some(match s.as_str() {
            "ideia" | "backlog" => Self::Ideia,
            "detalhando" => Self::Detalhando,
            "pronta" | "pronto" | "todo" => Self::Pronta,
            "executar" => Self::Executar,
            "executando" | "doing" => Self::Executando,
            "revisao" | "revisão" | "review" => Self::Revisao,
            "feita" | "feito" | "done" => Self::Feita,
            "bloqueada" | "bloqueado" | "blocked" => Self::Bloqueada,
            _ => return None,
        })
    }

    /// Pra onde dá pra ir a partir daqui, por qualquer um.
    ///
    /// O que NÃO existe é pular o detalhe: de `ideia` não se vai direto
    /// pra `executar`. Mandar o agente trabalhar numa linha solta é
    /// exatamente o que o projeto veio evitar.
    pub fn proximas(&self) -> Vec<EtapaTarefa> {
        match self {
            Self::Ideia => vec![Self::Detalhando, Self::Pronta],
            Self::Detalhando => vec![Self::Pronta, Self::Ideia],
            // `Feita` POR ÚLTIMO (ciclo 464): resolvido sem execução — uma
            // decisão, algo feito por fora. Por último pra os números da
            // TUI e a ordem dos botões que já existiam não mudarem.
            Self::Pronta => vec![Self::Executar, Self::Detalhando, Self::Feita],
            // Voltar pra `pronta` é desistir antes de a fila pegar.
            Self::Executar => vec![Self::Executando, Self::Pronta],
            Self::Executando => vec![Self::Revisao, Self::Bloqueada],
            // `executar` de novo é "refaça": o relato não serviu.
            Self::Revisao => vec![Self::Feita, Self::Executar, Self::Detalhando],
            // Reabrir é voltar ao detalhe: o que mudou de ideia muda o
            // texto antes de rodar de novo. Era `Revisao`, que só o
            // sistema alcança — e a pessoa ficava sem saída nenhuma de
            // uma tarefa feita (achado na tela, ciclo 460).
            Self::Feita => vec![Self::Detalhando],
            // `Feita` aqui é o caso da tarefa 001 do projeto Anotadinho:
            // o agente parou porque era uma decisão, a pessoa decidiu, e
            // não havia como fechar sem pagar uma execução só pra isso.
            Self::Bloqueada => vec![Self::Detalhando, Self::Pronta, Self::Executar, Self::Feita],
        }
    }

    /// Quem pode levar uma tarefa ATÉ esta etapa.
    ///
    /// `executando` e `revisao` são só do sistema, porque são registro do
    /// que aconteceu: uma pessoa marcando `revisao` à mão diria que o
    /// agente rodou quando não rodou. `bloqueada` é dos dois: o sistema
    /// bloqueia quando o agente para, e a pessoa bloqueia quando desiste.
    pub fn movida_por(&self) -> &'static [Quem] {
        match self {
            Self::Executando | Self::Revisao => &[Quem::Sistema],
            Self::Bloqueada => &[Quem::Sistema, Quem::Humano],
            _ => &[Quem::Humano],
        }
    }

    /// A transição é permitida pra quem está pedindo?
    pub fn pode_ir_para(&self, destino: EtapaTarefa, quem: Quem) -> bool {
        self.proximas().contains(&destino) && destino.movida_por().contains(&quem)
    }

    /// As transições que um `quem` vê como botão.
    pub fn proximas_para(&self, quem: Quem) -> Vec<EtapaTarefa> {
        self.proximas()
            .into_iter()
            .filter(|d| d.movida_por().contains(&quem))
            .collect()
    }
}

// ---------------------------------------------------------------------
// Config (fora do vault de páginas)
// ---------------------------------------------------------------------

/// Onde mora a config dos projetos, relativo ao vault.
pub const ARQUIVO_CONFIG: &str = ".anotadinho/projetos.json";

/// A config de um projeto.
///
/// Fica em `.anotadinho/`, não na página do projeto: o padrão de agente
/// e execução diz que config vem das preferências, nunca de página. Uma
/// página que decidisse onde um agente com `acceptEdits` escreve seria
/// uma porta aberta pra quem consegue editar uma nota.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigProjeto {
    /// Caminho absoluto do repositório onde o agente trabalha.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub repositorio: Option<String>,
    /// Fila parada: nada sai de `executar`.
    #[serde(default)]
    pub pausado: bool,
    /// Quantas tarefas rodam ao mesmo tempo. Um por padrão, porque
    /// tarefas no mesmo repositório disputam os mesmos arquivos.
    #[serde(default = "concorrencia_padrao")]
    pub concorrencia: usize,
}

fn concorrencia_padrao() -> usize {
    1
}

impl Default for ConfigProjeto {
    fn default() -> Self {
        Self { repositorio: None, pausado: false, concorrencia: concorrencia_padrao() }
    }
}

/// A config de todos os projetos, por slug.
pub type ConfigProjetos = BTreeMap<String, ConfigProjeto>;

// ---------------------------------------------------------------------
// Caminhos
// ---------------------------------------------------------------------

/// Pasta do projeto.
pub fn pasta_do_projeto(slug: &str) -> String {
    format!("{PASTA}/{slug}")
}

/// Página do projeto.
pub fn caminho_do_projeto(slug: &str) -> String {
    format!("{PASTA}/{slug}/projeto.md")
}

/// Página de arquitetura.
pub fn caminho_da_arquitetura(slug: &str) -> String {
    format!("{PASTA}/{slug}/arquitetura.md")
}

/// Pasta das tarefas.
pub fn pasta_das_tarefas(slug: &str) -> String {
    format!("{PASTA}/{slug}/tarefas")
}

/// A conversa geral do projeto.
pub fn caminho_da_conversa_geral(slug: &str) -> String {
    format!("{PASTA}/{slug}/conversas/geral.md")
}

/// A conversa onde as execuções de uma tarefa ficam registradas.
///
/// Uma por TAREFA, não por execução: o `--resume` do agente continua a
/// mesma sessão quando a tarefa volta pra `executar` depois da revisão,
/// e o agente lembra do que já tentou.
pub fn caminho_da_conversa_da_tarefa(slug: &str, numero: u32) -> String {
    format!("{PASTA}/{slug}/conversas/tarefa-{numero:03}.md")
}

/// Caminho de uma tarefa nova.
pub fn caminho_da_tarefa(slug: &str, numero: u32, titulo: &str) -> String {
    format!("{}/{numero:03}-{}.md", pasta_das_tarefas(slug), slug_de_titulo(titulo))
}

/// O slug do projeto dono de um caminho, se ele estiver dentro de um.
pub fn projeto_do_caminho(caminho: &str) -> Option<&str> {
    let resto = caminho.strip_prefix(PASTA)?.strip_prefix('/')?;
    let (slug, depois) = resto.split_once('/')?;
    (!slug.is_empty() && !depois.is_empty()).then_some(slug)
}

/// O caminho é de uma tarefa (e não da página do projeto, da
/// arquitetura ou de uma conversa)?
///
/// O ciclo 458 só lê o `status` de quem passa aqui: varrer o vault
/// inteiro a cada checagem da fila seria pagar por 250 páginas pra olhar
/// dez.
pub fn e_caminho_de_tarefa(caminho: &str) -> bool {
    match projeto_do_caminho(caminho) {
        Some(slug) => caminho
            .strip_prefix(&pasta_das_tarefas(slug))
            .and_then(|r| r.strip_prefix('/'))
            .is_some_and(|nome| !nome.contains('/') && nome.ends_with(".md")),
        None => false,
    }
}

/// O número no começo do nome do arquivo (`007-login.md` → 7).
pub(crate) fn numero_do_caminho(caminho: &str) -> Option<u32> {
    let nome = caminho.rsplit('/').next()?;
    let digitos: String = nome.chars().take_while(|c| c.is_ascii_digit()).collect();
    digitos.parse().ok()
}

/// O próximo número livre entre os caminhos de tarefa já existentes.
///
/// Máximo + 1, e não contagem + 1: apagar a tarefa 2 de três faria a
/// próxima nascer como 3 e colidir com a que já existe.
pub fn proximo_numero(caminhos: &[String]) -> u32 {
    caminhos.iter().filter_map(|c| numero_do_caminho(c)).max().unwrap_or(0) + 1
}

// ---------------------------------------------------------------------
// Páginas
// ---------------------------------------------------------------------

/// Uma página do projeto lida do vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Projeto {
    /// O slug, que é também o nome da pasta.
    pub slug: String,
    /// Título.
    pub titulo: String,
    /// Caminho da página.
    pub caminho: String,
}

impl Projeto {
    /// Lê uma página de projeto. `None` se não for `type: projeto`.
    ///
    /// O slug vem da PASTA, não do frontmatter: é a pasta que diz onde as
    /// tarefas estão, e um frontmatter copiado de outro projeto apontaria
    /// as tarefas pro lugar errado.
    pub fn ler(caminho: &str, texto: &str) -> Option<Projeto> {
        let (fm, _) = MarkdownCodec::split_frontmatter(texto).ok()?;
        if fm.page_type.as_deref() != Some(TIPO_PROJETO) {
            return None;
        }
        let slug = projeto_do_caminho(caminho)?.to_string();
        Some(Projeto {
            titulo: fm.title.clone().unwrap_or_else(|| slug.clone()),
            slug,
            caminho: caminho.to_string(),
        })
    }
}

/// Uma tarefa lida do vault.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tarefa {
    /// Caminho da página.
    pub caminho: String,
    /// Slug do projeto (da pasta).
    pub projeto: String,
    /// Número, do nome do arquivo. `0` se o arquivo não começa por um.
    pub numero: u32,
    /// Título.
    pub titulo: String,
    /// Etapa. Um `status` que não se reconhece conta como `ideia`: a
    /// tarefa continua no quadro, e NUNCA vai pra fila por engano.
    pub etapa: EtapaTarefa,
    /// `alta`, `media` ou `baixa`, como no resto do vault.
    pub prioridade: Option<String>,
    /// Referências a outras tarefas: número, nome do arquivo, caminho ou
    /// título.
    pub depende: Vec<String>,
    /// Spec vinculada, se houver.
    pub spec: Option<String>,
    /// Proposta vinculada, se houver.
    pub proposta: Option<String>,
    /// O número da sprint (ciclo 469). `None` é o backlog. O NÚMERO, e não
    /// o caminho: renomear a sprint não pode soltar as tarefas dela.
    #[serde(default)]
    pub sprint: Option<u32>,
    /// As tags da pessoa (ciclo 476). A tag `tarefa`, que toda tarefa
    /// nasce com, fica de fora: num selo do cartão ela só repetiria o
    /// óbvio.
    #[serde(default)]
    pub tags: Vec<String>,
    /// O prazo, `AAAA-MM-DD`.
    #[serde(default)]
    pub prazo: Option<String>,
    /// Os itens de **Critérios de aceite** (`core::detalhes`).
    #[serde(default)]
    pub checklist: Vec<crate::detalhes::ItemDaChecklist>,
    /// A seção **Comentários**.
    #[serde(default)]
    pub comentarios: Vec<crate::detalhes::Comentario>,
    /// A seção **Anexos**.
    #[serde(default)]
    pub anexos: Vec<crate::detalhes::Anexo>,
}

pub(crate) fn texto_do_valor(v: &serde_yaml::Value) -> Option<String> {
    match v {
        serde_yaml::Value::String(s) => Some(s.trim().to_string()).filter(|s| !s.is_empty()),
        serde_yaml::Value::Number(n) => Some(n.to_string()),
        _ => None,
    }
}

fn lista_do_valor(v: Option<&serde_yaml::Value>) -> Vec<String> {
    match v {
        Some(serde_yaml::Value::Sequence(itens)) => itens.iter().filter_map(texto_do_valor).collect(),
        // `depende: 3` ou `depende: "3, 4"` — a mão escreve assim.
        Some(outro) => texto_do_valor(outro)
            .map(|s| s.split(',').map(|p| p.trim().to_string()).filter(|p| !p.is_empty()).collect())
            .unwrap_or_default(),
        None => Vec::new(),
    }
}

impl Tarefa {
    /// Lê uma tarefa. `None` se não for `type: tarefa` dentro de um
    /// projeto.
    pub fn ler(caminho: &str, texto: &str) -> Option<Tarefa> {
        let (fm, _) = MarkdownCodec::split_frontmatter(texto).ok()?;
        if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
            return None;
        }
        let projeto = projeto_do_caminho(caminho)?.to_string();
        let campo = |k: &str| fm.extra.get(k).and_then(texto_do_valor);
        Some(Tarefa {
            caminho: caminho.to_string(),
            projeto,
            numero: numero_do_caminho(caminho).unwrap_or(0),
            titulo: fm.title.clone().unwrap_or_else(|| caminho.to_string()),
            etapa: campo("status").and_then(|s| EtapaTarefa::from_slug(&s)).unwrap_or_default(),
            prioridade: campo("prioridade"),
            depende: lista_do_valor(fm.extra.get("depende")),
            spec: campo("spec"),
            proposta: campo("proposta"),
            // `sprint: 3` ou `sprint: "3"`: a mão escreve dos dois jeitos.
            sprint: campo("sprint").and_then(|s| s.trim().parse().ok()),
            tags: fm.tags.iter().filter(|t| t.as_str() != TIPO_TAREFA).cloned().collect(),
            prazo: campo("prazo"),
            checklist: crate::detalhes::checklist(texto),
            comentarios: crate::detalhes::comentarios(texto),
            anexos: crate::detalhes::anexos(texto),
        })
    }

    /// Passou do prazo sem estar feita (ciclo 476). `hoje` em
    /// `AAAA-MM-DD`: a data é comparável como texto, e o prazo que não é
    /// data válida nunca vence — um aviso errado é pior que nenhum.
    pub fn vencida(&self, hoje: &str) -> bool {
        self.etapa != EtapaTarefa::Feita
            && self
                .prazo
                .as_deref()
                .is_some_and(|p| crate::detalhes::data_valida(p) && crate::detalhes::data_valida(hoje) && p < hoje)
    }

    /// `(feitos, total)` da checklist.
    pub fn progresso(&self) -> (usize, usize) {
        (self.checklist.iter().filter(|i| i.feito).count(), self.checklist.len())
    }

    /// Esta tarefa é a que `referencia` aponta?
    ///
    /// Desde o ciclo 462 quem responde à fila é o [`IndiceDeTarefas`]; isto
    /// fica como GABARITO do teste que confere que os dois concordam, e pra
    /// recusar a tarefa que dependeria de si (ciclo 513).
    pub(crate) fn casa_com(&self, referencia: &str) -> bool {
        let r = referencia.trim().trim_start_matches("[[").trim_end_matches("]]").trim();
        if r.is_empty() {
            return false;
        }
        if let Ok(n) = r.parse::<u32>() {
            return n == self.numero;
        }
        let nome = self.caminho.rsplit('/').next().unwrap_or("");
        r == self.caminho
            || r == nome
            || r == nome.trim_end_matches(".md")
            || r.eq_ignore_ascii_case(&self.titulo)
    }
}

/// Troca as tags da tarefa (ciclo 476), mantendo a `tarefa`.
///
/// Tags repetidas ou vazias caem; `#` na frente também, porque a mão
/// escreve `#login`.
pub fn com_tags(texto: &str, tags: &[String]) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    let mut novas = vec![TIPO_TAREFA.to_string()];
    for t in tags {
        let t = t.trim().trim_start_matches('#').trim().to_string();
        if !t.is_empty() && !novas.contains(&t) {
            novas.push(t);
        }
    }
    fm.tags = novas;
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

/// Põe ou tira o prazo (ciclo 476). Recusa o que não é `AAAA-MM-DD`: o
/// prazo é comparado como texto, e "5/11" nunca venceria.
pub fn com_prazo(texto: &str, prazo: Option<&str>) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    match prazo.map(str::trim).filter(|p| !p.is_empty()) {
        Some(p) if crate::detalhes::data_valida(p) => {
            fm.extra.insert("prazo".into(), valor(p));
        }
        Some(p) => return Err(format!("prazo \"{p}\" não é uma data AAAA-MM-DD")),
        None => {
            fm.extra.remove("prazo");
        }
    }
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

/// As prioridades que a fila entende (ciclo 513), na ordem do seletor.
pub const PRIORIDADES: [&str; 3] = ["alta", "media", "baixa"];

/// Põe ou tira a prioridade (ciclo 513). Só as de [`PRIORIDADES`]: a fila
/// ordena por elas (`peso_da_prioridade`), e "urgente" escrito à mão cairia
/// em média sem ninguém ver.
pub fn com_prioridade(texto: &str, prioridade: Option<&str>) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    match prioridade.map(|p| p.trim().to_lowercase()).filter(|p| !p.is_empty()) {
        Some(p) if PRIORIDADES.contains(&p.as_str()) => {
            fm.extra.insert("prioridade".into(), valor(&p));
        }
        Some(p) => return Err(format!("prioridade \"{p}\" não é alta, media nem baixa")),
        None => {
            fm.extra.remove("prioridade");
        }
    }
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

/// Troca as dependências (ciclo 513): a lista vai como lista YAML, sem
/// repetidas nem vazias; vazia, o campo sai. Uma tarefa não depende de si.
pub fn com_dependencias(texto: &str, caminho: &str, depende: &[String]) -> Result<String, String> {
    let (mut fm, _) = MarkdownCodec::split_frontmatter(texto).map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    let propria = Tarefa::ler(caminho, texto);
    let mut lista: Vec<String> = Vec::new();
    for d in depende {
        let d = d.trim().to_string();
        if d.is_empty() || lista.contains(&d) {
            continue;
        }
        if propria.as_ref().is_some_and(|t| t.casa_com(&d)) {
            return Err("uma tarefa não pode depender dela mesma".to_string());
        }
        lista.push(d);
    }
    if lista.is_empty() {
        fm.extra.remove("depende");
    } else {
        fm.extra.insert("depende".into(), serde_yaml::Value::Sequence(lista.into_iter().map(serde_yaml::Value::String).collect()));
    }
    MarkdownCodec::substituir_frontmatter(texto, &fm).map_err(|e| e.to_string())
}

pub(crate) fn frontmatter_em_texto(fm: &Frontmatter, corpo: &str) -> String {
    // `substituir_frontmatter` serializa por `serde_yaml`: título com
    // `: ` no meio continua YAML válido (o bug do ciclo 206 era montar
    // isso com `format!`).
    MarkdownCodec::substituir_frontmatter(corpo, fm)
        .expect("frontmatter de strings sempre serializa")
}

pub(crate) fn valor(s: &str) -> serde_yaml::Value {
    serde_yaml::Value::String(s.to_string())
}

/// O esqueleto do corpo de uma tarefa.
///
/// As duas primeiras seções são a metade "spec" (o que, e como saber que
/// terminou); as duas últimas, a metade "proposta" (como, e o que não
/// fazer). O prompt de execução se apoia nestes nomes.
pub const CORPO_PADRAO_TAREFA: &str = "## Objetivo\n\nO que precisa existir quando a tarefa terminar, e por quê.\n\n## Critérios de aceite\n\n- [ ] \n\n## Abordagem\n\nComo fazer. Se houver proposta vinculada, é ela que vale.\n\n## Fora de escopo\n\n- \n";

/// Monta a página de uma tarefa nova, em `ideia`.
pub fn montar_tarefa(projeto: &str, titulo: &str, corpo: &str, hoje: &str) -> String {
    let mut fm = Frontmatter {
        title: Some(titulo.trim().to_string()),
        tags: vec![TIPO_TAREFA.to_string()],
        page_type: Some(TIPO_TAREFA.to_string()),
        ..Default::default()
    };
    fm.extra.insert("projeto".into(), valor(projeto));
    fm.extra.insert("status".into(), valor(EtapaTarefa::Ideia.slug()));
    fm.extra.insert("date".into(), valor(hoje));

    let corpo = corpo.trim();
    let corpo = if corpo.is_empty() { CORPO_PADRAO_TAREFA } else { corpo };
    frontmatter_em_texto(&fm, &format!("# {}\n\n{}\n", titulo.trim(), corpo.trim_end()))
}

/// As páginas com que um projeto nasce: `(caminho, conteúdo)`.
///
/// A pasta de tarefas não entra: é o IO (ciclo 458) que a cria, porque
/// pasta vazia não é arquivo.
pub fn montar_projeto(slug: &str, titulo: &str, hoje: &str) -> Vec<(String, String)> {
    let titulo = titulo.trim();

    let mut fm = Frontmatter {
        title: Some(titulo.to_string()),
        tags: vec![TIPO_PROJETO.to_string()],
        page_type: Some(TIPO_PROJETO.to_string()),
        ..Default::default()
    };
    fm.extra.insert("projeto".into(), valor(slug));
    fm.extra.insert("date".into(), valor(hoje));
    let projeto = frontmatter_em_texto(
        &fm,
        &format!(
            "# {titulo}\n\n## Visão\n\nO que é, pra quem, e que problema resolve.\n\n## Objetivo\n\nO que precisa estar no ar pra este projeto ter valido a pena.\n\n## Stack\n\n- \n\n## Combinados com o agente\n\nO que vale pra TODA tarefa: commitar ou não, como testar, o que nunca mexer.\n"
        ),
    );

    let mut fm_arq = Frontmatter {
        title: Some(format!("Arquitetura — {titulo}")),
        tags: vec!["arquitetura".to_string()],
        page_type: Some("docs".to_string()),
        ..Default::default()
    };
    fm_arq.extra.insert("projeto".into(), valor(slug));
    fm_arq.extra.insert("date".into(), valor(hoje));
    let arquitetura = frontmatter_em_texto(
        &fm_arq,
        &format!(
            "# Arquitetura — {titulo}\n\n## Componentes\n\n## Dados\n\n## Integrações\n\n## Decisões\n"
        ),
    );

    let contexto = vec![caminho_do_projeto(slug), caminho_da_arquitetura(slug)];
    let conversa = crate::conversa::montar_pagina(
        &format!("Conversa — {titulo}"),
        Some(&caminho_do_projeto(slug)),
        &contexto,
    );
    let conversa = MarkdownCodec::set_frontmatter_field(&conversa, "projeto", slug)
        .expect("frontmatter da conversa é o que o próprio core montou");

    vec![
        (caminho_do_projeto(slug), projeto),
        (caminho_da_arquitetura(slug), arquitetura),
        (caminho_da_conversa_geral(slug), conversa),
    ]
}

/// Move a etapa de uma tarefa, devolvendo o texto novo.
///
/// Só o `status` muda, pelo `MarkdownCodec`, e o corpo fica intocado. O
/// erro diz pra onde a tarefa PODE ir: é a mensagem que o CLI mostra
/// quando alguém tenta pular etapa.
pub fn mudar_etapa(texto: &str, destino: EtapaTarefa, quem: Quem) -> Result<String, String> {
    let (fm, _) = MarkdownCodec::split_frontmatter(texto)
        .map_err(|e| format!("frontmatter ilegível: {e}"))?;
    if fm.page_type.as_deref() != Some(TIPO_TAREFA) {
        return Err("a página não é uma tarefa (type: tarefa)".to_string());
    }
    let atual = fm
        .extra
        .get("status")
        .and_then(texto_do_valor)
        .and_then(|s| EtapaTarefa::from_slug(&s))
        .unwrap_or_default();
    if !atual.pode_ir_para(destino, quem) {
        let possiveis: Vec<&str> = atual.proximas_para(quem).iter().map(|e| e.slug()).collect();
        return Err(format!(
            "de \"{}\" não dá pra ir pra \"{}\". Daqui dá pra: {}.",
            atual.slug(),
            destino.slug(),
            if possiveis.is_empty() { "nada, por este canal".to_string() } else { possiveis.join(", ") }
        ));
    }
    MarkdownCodec::set_frontmatter_field(texto, "status", destino.slug())
        .map_err(|e| format!("não consegui gravar o status: {e}"))
}

// ---------------------------------------------------------------------
// Fila
// ---------------------------------------------------------------------

fn peso_da_prioridade(p: Option<&str>) -> u8 {
    match p.map(|s| s.trim().to_lowercase()).as_deref() {
        Some("alta") | Some("high") => 0,
        Some("baixa") | Some("low") => 2,
        _ => 1,
    }
}

/// As tarefas `feita`, indexadas pelas formas com que uma dependência as
/// cita (ciclo 462): número, caminho, nome do arquivo e título.
///
/// Antes, cada dependência era comparada com TODAS as tarefas, e quem
/// pergunta é cada tarefa da fila e cada cartão do quadro: `O(n²)`. A
/// bateria de escala pegou o custo por tarefa subindo de 76 µs (250
/// tarefas) pra 143 µs (2000). O índice monta uma vez e responde cada
/// dependência numa busca.
#[derive(Debug, Default)]
pub struct IndiceDeTarefas {
    feitas: std::collections::HashSet<String>,
}

impl IndiceDeTarefas {
    /// Monta a partir das tarefas (de um ou mais projetos).
    pub fn novo(tarefas: &[Tarefa]) -> Self {
        let mut feitas = std::collections::HashSet::new();
        for t in tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Feita) {
            let nome = t.caminho.rsplit('/').next().unwrap_or("");
            feitas.insert(chave(&t.projeto, "n", &t.numero.to_string()));
            for exata in [t.caminho.as_str(), nome, nome.trim_end_matches(".md")] {
                feitas.insert(chave(&t.projeto, "p", exata));
            }
            // O título casa sem diferenciar maiúscula (só ASCII), como
            // `casa_com` sempre fez.
            feitas.insert(chave(&t.projeto, "t", &t.titulo.to_ascii_lowercase()));
        }
        Self { feitas }
    }

    /// A referência aponta pra uma tarefa `feita` do mesmo projeto?
    fn satisfeita(&self, projeto: &str, referencia: &str) -> bool {
        let r = referencia.trim().trim_start_matches("[[").trim_end_matches("]]").trim();
        if r.is_empty() {
            return false;
        }
        if let Ok(n) = r.parse::<u32>() {
            return self.feitas.contains(&chave(projeto, "n", &n.to_string()));
        }
        self.feitas.contains(&chave(projeto, "p", r)) || self.feitas.contains(&chave(projeto, "t", &r.to_ascii_lowercase()))
    }

    /// As dependências de `tarefa` que ainda não estão `feita`.
    ///
    /// Referência que não casa com tarefa nenhuma também volta aqui, e
    /// segura a tarefa: rodar achando que a dependência existia é pior do
    /// que esperar com o motivo à vista.
    pub fn pendentes(&self, tarefa: &Tarefa) -> Vec<String> {
        tarefa.depende.iter().filter(|r| !self.satisfeita(&tarefa.projeto, r)).cloned().collect()
    }
}

fn chave(projeto: &str, tipo: &str, valor: &str) -> String {
    format!("{projeto}\u{0}{tipo}\u{0}{valor}")
}

/// As dependências pendentes de UMA tarefa. Pra várias (a fila, o
/// quadro), monte o [`IndiceDeTarefas`] uma vez e pergunte a ele: chamar
/// isto num laço é o `O(n²)` que o ciclo 462 tirou.
pub fn dependencias_pendentes(tarefa: &Tarefa, todas: &[Tarefa]) -> Vec<String> {
    IndiceDeTarefas::novo(todas).pendentes(tarefa)
}

/// A fila do projeto: o que está em `executar` e pode rodar, na ordem.
///
/// É por NÍVEL, e não por borda. A proposta previa uma memória de
/// "o que já vi" (`projetos-visto.json`), e ela perdia pedido: a tarefa
/// posta em `executar` com a fila pausada ficava marcada como vista e
/// nunca rodava depois de retomar. O que impede rodar duas vezes é a
/// reserva (`executar` → `executando`, gravada ANTES de começar): depois
/// dela a tarefa sai daqui sozinha.
///
/// Ordem: prioridade, depois número, que é a ordem em que a pessoa
/// escreveu.
pub fn fila(tarefas: &[Tarefa]) -> Vec<&Tarefa> {
    let indice = IndiceDeTarefas::novo(tarefas);
    let mut prontas: Vec<&Tarefa> = tarefas
        .iter()
        .filter(|t| t.etapa == EtapaTarefa::Executar)
        .filter(|t| indice.pendentes(t).is_empty())
        .collect();
    prontas.sort_by_key(|t| (peso_da_prioridade(t.prioridade.as_deref()), t.numero, t.caminho.clone()));
    prontas
}

/// Quantas tarefas por etapa, na ordem do quadro.
pub fn contagem_por_etapa(tarefas: &[Tarefa]) -> BTreeMap<EtapaTarefa, usize> {
    let mut c = BTreeMap::new();
    for t in tarefas {
        *c.entry(t.etapa).or_insert(0) += 1;
    }
    c
}

// ---------------------------------------------------------------------
// Contexto e prompts
// ---------------------------------------------------------------------

/// O índice das tarefas, uma por linha: é o que o agente lê pra saber o
/// que já existe antes de propor tarefa nova.
pub fn indice_de_tarefas(tarefas: &[Tarefa]) -> String {
    let mut ordenadas: Vec<&Tarefa> = tarefas.iter().collect();
    ordenadas.sort_by_key(|t| (t.numero, t.caminho.clone()));
    if ordenadas.is_empty() {
        return "(nenhuma tarefa ainda)".to_string();
    }
    ordenadas
        .iter()
        .map(|t| {
            let mut linha = format!("- {:03} [{}] {} — {}", t.numero, t.etapa.slug(), t.titulo, t.caminho);
            if !t.depende.is_empty() {
                linha.push_str(&format!(" (depende: {})", t.depende.join(", ")));
            }
            if let Some(n) = t.sprint {
                linha.push_str(&format!(" · sprint {n}"));
            }
            linha
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// O que a conversa do projeto anexa sozinha: a página do projeto, a
/// arquitetura e o índice das tarefas.
///
/// Sai como `Contexto`, e quem chama passa por `envio::montar`: é lá que
/// o orçamento poda. Ter uma poda própria aqui seria a terceira
/// implementação divergente que o ciclo 423 veio acabar.
pub fn contexto_do_projeto(
    projeto: (&str, &str),
    arquitetura: Option<(&str, &str)>,
    tarefas: &[Tarefa],
) -> Vec<Contexto> {
    let mut v = vec![Contexto { nome: projeto.0.to_string(), conteudo: projeto.1.to_string() }];
    if let Some((nome, conteudo)) = arquitetura {
        v.push(Contexto { nome: nome.to_string(), conteudo: conteudo.to_string() });
    }
    let pasta = tarefas
        .first()
        .map(|t| pasta_das_tarefas(&t.projeto))
        .or_else(|| projeto_do_caminho(projeto.0).map(pasta_das_tarefas))
        .unwrap_or_else(|| "tarefas".to_string());
    v.push(Contexto { nome: format!("{pasta} (índice)"), conteudo: indice_de_tarefas(tarefas) });
    v
}

/// A pergunta que pede pro agente DETALHAR uma tarefa.
///
/// A trava é a do planejamento (`fluxo::pergunta_de_planejamento`):
/// detalhar não é mudar o escopo. Sem ela o modelo transforma "tela de
/// login" em "sistema de identidade".
pub fn pergunta_de_detalhar(tarefa: &Tarefa) -> String {
    [
        format!("Detalhe a tarefa \"{}\" ({}), anexada.", tarefa.titulo, tarefa.caminho),
        String::new(),
        "Reescreva a página com estas quatro seções, nesta ordem:".to_string(),
        String::new(),
        "1. Objetivo: o que precisa existir quando terminar, e por quê.".to_string(),
        "2. Critérios de aceite: uma lista de `- [ ]`, cada um verificável.".to_string(),
        "3. Abordagem: como fazer, considerando a arquitetura anexada.".to_string(),
        "4. Fora de escopo: o que esta tarefa NÃO cobre.".to_string(),
        String::new(),
        format!(
            "NÃO grave o arquivo. Proponha a página inteira com `propor` em {}, \
             mantendo o frontmatter como está — inclusive o `status`, que só eu mudo.",
            tarefa.caminho
        ),
        String::new(),
        "Não aumente o escopo. Se a tarefa for grande demais pra uma execução, \
         diga isso e sugira como dividir, em vez de detalhar tudo numa só. Se algo \
         estiver ambíguo, aponte em vez de decidir por conta."
            .to_string(),
    ]
    .join("\n")
}

/// A linha com que a resposta da execução tem que terminar.
pub const MARCA_DE_RESULTADO: &str = "RESULTADO:";

/// A pergunta que manda EXECUTAR uma tarefa.
///
/// O agente roda dentro do repositório do projeto, então "aqui" é o
/// código. A trava é a da execução de proposta: a abordagem já foi
/// aceita e mudá-la exige parar e explicar. E a linha final é o que o
/// app lê pra decidir entre `revisao` e `bloqueada`: o agente não toca
/// no `status`.
pub fn pergunta_de_executar(tarefa: &Tarefa) -> String {
    let base = match tarefa.proposta.as_deref() {
        Some(p) => format!("Siga a abordagem da proposta vinculada ({p}), que foi aprovada."),
        None => "Siga a seção Abordagem da tarefa: ela foi aceita quando a tarefa ficou pronta."
            .to_string(),
    };
    [
        format!("Execute a tarefa \"{}\" ({}), anexada.", tarefa.titulo, tarefa.caminho),
        String::new(),
        "Você está no diretório do repositório deste projeto: é aqui que o código muda."
            .to_string(),
        base,
        "Cumpra os critérios de aceite e respeite o fora de escopo e os combinados da \
         página do projeto."
            .to_string(),
        String::new(),
        "Se a abordagem se mostrar inviável, PARE e explique o problema em vez de mudar \
         de rumo por conta. Não mexa no `status` da tarefa: quem registra é o app."
            .to_string(),
        String::new(),
        "Ao terminar, relate:".to_string(),
        String::new(),
        "1. O que foi feito.".to_string(),
        "2. Como foi validado, critério por critério.".to_string(),
        "3. O que ficou de fora, e por quê.".to_string(),
        String::new(),
        format!(
            "A ÚLTIMA linha da resposta tem que ser `{MARCA_DE_RESULTADO} concluida` ou \
             `{MARCA_DE_RESULTADO} bloqueada`. Use bloqueada sempre que algum critério \
             não foi cumprido — dizer que terminou quando não terminou é pior do que parar."
        ),
    ]
    .join("\n")
}

/// O que o agente disse que aconteceu.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Resultado {
    /// Disse que concluiu: a tarefa vai pra revisão humana.
    Concluida,
    /// Disse que não deu.
    Bloqueada,
    /// Não disse: a resposta não tem a linha, ou ela é ilegível.
    SemResultado,
}

impl Resultado {
    /// Pra onde a tarefa vai.
    ///
    /// Sem resultado vai pra `bloqueada`, não pra `revisao`: uma resposta
    /// cortada pelo timeout, ou um agente que esqueceu o combinado,
    /// pareceria trabalho terminado esperando aprovação.
    pub fn destino(&self) -> EtapaTarefa {
        match self {
            Self::Concluida => EtapaTarefa::Revisao,
            Self::Bloqueada | Self::SemResultado => EtapaTarefa::Bloqueada,
        }
    }
}

/// Lê o resultado da resposta do agente.
///
/// Vale a ÚLTIMA ocorrência: o agente às vezes cita o combinado no meio
/// do relato ("vou terminar com RESULTADO: concluida"), e é o fim que
/// diz o que aconteceu. Tolera o markdown que o modelo põe em volta
/// (`**`, crase, marcador de lista).
pub fn ler_resultado(resposta: &str) -> Resultado {
    for linha in resposta.lines().rev() {
        let limpa: String = linha.chars().filter(|c| !matches!(c, '*' | '`' | '_')).collect();
        let limpa = limpa.trim().trim_start_matches(['-', '>', ' ']).trim();
        // ASCII, e não `to_uppercase`: este preserva o comprimento em bytes,
        // e é o índice dele que corta `limpa` logo abaixo.
        let maiusc = limpa.to_ascii_uppercase();
        let Some(pos) = maiusc.find(MARCA_DE_RESULTADO) else { continue };
        let valor = limpa[pos + MARCA_DE_RESULTADO.len()..].trim().to_lowercase();
        let valor = valor.trim_end_matches(['.', '!']);
        return match valor {
            "concluida" | "concluída" | "concluido" | "concluído" => Resultado::Concluida,
            "bloqueada" | "bloqueado" => Resultado::Bloqueada,
            _ => Resultado::SemResultado,
        };
    }
    Resultado::SemResultado
}

/// O cabeçalho que abre cada tarefa numa resposta "quebre em tarefas".
///
/// O prompt semeado (`quebrar-em-tarefas.md`) pede exatamente isto, e
/// `separar_tarefas` lê isto: os dois têm que concordar, e o teste da
/// semente confere.
pub const CABECALHO_DE_TAREFA: &str = "## Tarefa:";

/// Separa uma resposta em tarefas: `(título, corpo)`.
///
/// Cada `## Tarefa: <título>` abre uma, e o corpo vai até a próxima.
/// Os `##` das seções dentro da tarefa (Objetivo, Critérios…) descem um
/// nível, pra caber debaixo do `#` do título na página da tarefa. Texto
/// antes da primeira tarefa é conversa do modelo, e fica de fora.
pub fn separar_tarefas(resposta: &str) -> Vec<(String, String)> {
    let mut tarefas: Vec<(String, String)> = Vec::new();
    for linha in resposta.lines() {
        let t = linha.trim_start();
        if let Some(titulo) = t.strip_prefix(CABECALHO_DE_TAREFA) {
            tarefas.push((titulo.trim().to_string(), String::new()));
            continue;
        }
        if let Some((_, corpo)) = tarefas.last_mut() {
            corpo.push_str(linha);
            corpo.push('\n');
        }
    }
    tarefas
        .into_iter()
        .filter(|(titulo, _)| !titulo.is_empty())
        .map(|(titulo, corpo)| {
            let corpo = corpo
                .lines()
                .map(|l| if l.starts_with("### ") { &l[1..] } else { l })
                .collect::<Vec<_>>()
                .join("\n");
            (titulo, corpo.trim().to_string())
        })
        .collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn tarefa_md(status: &str, extra: &str) -> String {
        format!("---\ntitle: Tela de login\ntype: tarefa\nprojeto: saas\nstatus: {status}\n{extra}---\n# Tela de login\n\ncorpo\n")
    }

    fn tarefa(numero: u32, status: &str, extra: &str) -> Tarefa {
        let caminho = format!("pages/projetos/saas/tarefas/{numero:03}-t.md");
        Tarefa::ler(&caminho, &tarefa_md(status, extra)).unwrap()
    }

    #[test]
    fn config_sem_campos_vale_o_padrao() {
        let c: ConfigProjeto = serde_json::from_str("{}").unwrap();
        assert_eq!(c, ConfigProjeto::default());
        assert_eq!(c.concorrencia, 1);
    }

    #[test]
    fn toda_etapa_faz_ida_e_volta_pelo_slug() {
        for e in EtapaTarefa::all() {
            assert_eq!(EtapaTarefa::from_slug(e.slug()), Some(*e));
        }
        assert_eq!(EtapaTarefa::from_slug("Revisão"), Some(EtapaTarefa::Revisao));
        assert_eq!(EtapaTarefa::from_slug("talvez"), None);
    }

    #[test]
    fn nao_se_pula_o_detalhe() {
        assert!(!EtapaTarefa::Ideia.pode_ir_para(EtapaTarefa::Executar, Quem::Humano));
        assert!(EtapaTarefa::Pronta.pode_ir_para(EtapaTarefa::Executar, Quem::Humano));
    }

    #[test]
    fn executando_e_revisao_sao_so_do_sistema() {
        assert!(!EtapaTarefa::Executar.pode_ir_para(EtapaTarefa::Executando, Quem::Humano));
        assert!(EtapaTarefa::Executar.pode_ir_para(EtapaTarefa::Executando, Quem::Sistema));
        assert!(!EtapaTarefa::Executando.pode_ir_para(EtapaTarefa::Revisao, Quem::Humano));
        assert!(EtapaTarefa::Executando.pode_ir_para(EtapaTarefa::Revisao, Quem::Sistema));
        // Bloquear é dos dois.
        assert!(EtapaTarefa::Executando.pode_ir_para(EtapaTarefa::Bloqueada, Quem::Humano));
        assert!(EtapaTarefa::Executando.pode_ir_para(EtapaTarefa::Bloqueada, Quem::Sistema));
    }

    #[test]
    fn o_sistema_nunca_decide_por_humano() {
        // A regra "só humano avança etapa", escrita como varredura: o
        // sistema só alcança o que é registro do que aconteceu.
        for de in EtapaTarefa::all() {
            for para in de.proximas_para(Quem::Sistema) {
                assert!(
                    matches!(para, EtapaTarefa::Executando | EtapaTarefa::Revisao | EtapaTarefa::Bloqueada),
                    "o sistema levaria {} → {}",
                    de.slug(),
                    para.slug()
                );
            }
        }
    }

    #[test]
    fn toda_etapa_tem_saida_pra_alguem() {
        for e in EtapaTarefa::all() {
            assert!(!e.proximas().is_empty(), "{} é beco sem saída", e.slug());
        }
    }

    #[test]
    fn resolvido_sem_execucao_fecha_direto() {
        for de in [EtapaTarefa::Pronta, EtapaTarefa::Bloqueada] {
            assert!(de.pode_ir_para(EtapaTarefa::Feita, Quem::Humano), "{} → feita", de.slug());
            assert!(!de.pode_ir_para(EtapaTarefa::Feita, Quem::Sistema), "o sistema fecharia {}", de.slug());
            assert_eq!(de.proximas_para(Quem::Humano).last(), Some(&EtapaTarefa::Feita), "feita tem que vir por último");
        }
        // Sem detalhe, não se fecha: quase sempre é tarefa pra apagar.
        assert!(!EtapaTarefa::Ideia.pode_ir_para(EtapaTarefa::Feita, Quem::Humano));
        assert!(!EtapaTarefa::Detalhando.pode_ir_para(EtapaTarefa::Feita, Quem::Humano));
    }

    #[test]
    fn a_pessoa_sempre_tem_pra_onde_ir() {
        // `toda_etapa_tem_saida_pra_alguem` deixou passar `feita`: a única
        // saída era do sistema, e no quadro o cartão ficava sem botão.
        for e in EtapaTarefa::all() {
            assert!(!e.proximas_para(Quem::Humano).is_empty(), "a pessoa não sai de {}", e.slug());
        }
    }

    #[test]
    fn mudar_etapa_mexe_so_no_status() {
        let md = tarefa_md("pronta", "");
        let novo = mudar_etapa(&md, EtapaTarefa::Executar, Quem::Humano).unwrap();
        assert!(novo.contains("status: executar"));
        let (_, corpo_antes) = MarkdownCodec::split_frontmatter_text(&md);
        let (_, corpo_depois) = MarkdownCodec::split_frontmatter_text(&novo);
        assert_eq!(corpo_antes, corpo_depois);
        assert_eq!(Tarefa::ler("pages/projetos/saas/tarefas/001-t.md", &novo).unwrap().projeto, "saas");
    }

    #[test]
    fn mudar_etapa_invalida_diz_pra_onde_da() {
        let erro = mudar_etapa(&tarefa_md("ideia", ""), EtapaTarefa::Executar, Quem::Humano).unwrap_err();
        assert!(erro.contains("detalhando"), "{erro}");
        assert!(erro.contains("pronta"), "{erro}");
    }

    #[test]
    fn mudar_etapa_recusa_pagina_que_nao_e_tarefa() {
        let md = "---\ntitle: x\ntype: spec\nstatus: pronta\n---\ncorpo\n";
        assert!(mudar_etapa(md, EtapaTarefa::Executar, Quem::Humano).is_err());
    }

    #[test]
    fn titulo_com_dois_pontos_continua_yaml_valido() {
        let md = montar_tarefa("saas", "Login: com Google", "", "2026-10-21");
        let t = Tarefa::ler("pages/projetos/saas/tarefas/001-login.md", &md).unwrap();
        assert_eq!(t.titulo, "Login: com Google");
        assert_eq!(t.etapa, EtapaTarefa::Ideia);
        assert!(md.contains("## Critérios de aceite"), "corpo vazio ganha o esqueleto");
    }

    #[test]
    fn projeto_nasce_com_tres_paginas_legiveis() {
        let paginas = montar_projeto("saas", "Meu SaaS: v1", "2026-10-21");
        let caminhos: Vec<&str> = paginas.iter().map(|(c, _)| c.as_str()).collect();
        assert_eq!(
            caminhos,
            [
                "pages/projetos/saas/projeto.md",
                "pages/projetos/saas/arquitetura.md",
                "pages/projetos/saas/conversas/geral.md"
            ]
        );
        let p = Projeto::ler(&paginas[0].0, &paginas[0].1).unwrap();
        assert_eq!(p.slug, "saas");
        assert_eq!(p.titulo, "Meu SaaS: v1");
        for (caminho, texto) in &paginas {
            let (fm, _) = MarkdownCodec::split_frontmatter(texto)
                .unwrap_or_else(|e| panic!("{caminho}: {e}"));
            assert_eq!(fm.extra.get("projeto").and_then(texto_do_valor).as_deref(), Some("saas"), "{caminho}");
        }
        let (fm, _) = MarkdownCodec::split_frontmatter(&paginas[2].1).unwrap();
        assert_eq!(fm.effective_type(), "conversa");
    }

    #[test]
    fn ler_exige_o_tipo_e_a_pasta() {
        assert!(Tarefa::ler("pages/projetos/saas/tarefas/001-t.md", "---\ntype: spec\n---\n").is_none());
        assert!(Tarefa::ler("pages/outra/001-t.md", &tarefa_md("ideia", "")).is_none());
        assert!(Projeto::ler("pages/projetos/saas/projeto.md", "---\ntype: docs\n---\n").is_none());
    }

    #[test]
    fn status_desconhecido_nunca_entra_na_fila() {
        let t = tarefa(1, "exec", "");
        assert_eq!(t.etapa, EtapaTarefa::Ideia);
        assert!(fila(&[t]).is_empty());
    }

    #[test]
    fn caminhos_de_tarefa() {
        assert!(e_caminho_de_tarefa("pages/projetos/saas/tarefas/001-x.md"));
        assert!(!e_caminho_de_tarefa("pages/projetos/saas/projeto.md"));
        assert!(!e_caminho_de_tarefa("pages/projetos/saas/tarefas/sub/001-x.md"));
        assert!(!e_caminho_de_tarefa("pages/specs/x.md"));
        assert_eq!(projeto_do_caminho("pages/projetos/saas/projeto.md"), Some("saas"));
        assert_eq!(projeto_do_caminho("pages/projetos/saas"), None);
        assert_eq!(caminho_da_tarefa("saas", 7, "Tela de Login"), "pages/projetos/saas/tarefas/007-tela-de-login.md");
    }

    #[test]
    fn numero_novo_nao_colide_depois_de_apagar() {
        let existentes = vec![
            "pages/projetos/s/tarefas/001-a.md".to_string(),
            "pages/projetos/s/tarefas/003-c.md".to_string(),
        ];
        assert_eq!(proximo_numero(&existentes), 4);
        assert_eq!(proximo_numero(&[]), 1);
    }

    #[test]
    fn prioridade_e_dependencias_gravam_no_frontmatter() {
        let t = ("pages/projetos/saas/tarefas/003-t.md".to_string(), tarefa_md("pronta", ""));
        let texto = &t.1;
        let com = com_prioridade(texto, Some("Alta")).unwrap();
        assert_eq!(Tarefa::ler(&t.0, &com).unwrap().prioridade.as_deref(), Some("alta"));
        assert!(com_prioridade(texto, Some("urgente")).is_err());
        let sem = com_prioridade(&com, None).unwrap();
        assert_eq!(Tarefa::ler(&t.0, &sem).unwrap().prioridade, None);
        let dep = com_dependencias(texto, &t.0, &["1".into(), " 2 ".into(), "1".into(), String::new()]).unwrap();
        assert_eq!(Tarefa::ler(&t.0, &dep).unwrap().depende, ["1", "2"]);
        assert!(dep.contains("depende:\n- '1'\n- '2'") || dep.contains("depende:\n- \"1\"\n- \"2\""), "{dep}");
        assert!(com_dependencias(texto, &t.0, &["3".into()]).is_err(), "depender de si");
        let limpa = com_dependencias(&dep, &t.0, &[]).unwrap();
        assert!(!limpa.contains("depende"), "{limpa}");
        // O corpo fica intocado.
        assert_eq!(MarkdownCodec::split_frontmatter_text(&limpa).1, MarkdownCodec::split_frontmatter_text(texto).1);
        // Não é tarefa: recusa.
        assert!(com_prioridade("---\ntitle: x\n---\nc\n", Some("alta")).is_err());
    }

    #[test]
    fn fila_respeita_prioridade_e_numero() {
        let ts = vec![
            tarefa(1, "executar", "prioridade: baixa\n"),
            tarefa(2, "executar", ""),
            tarefa(3, "executar", "prioridade: alta\n"),
            tarefa(4, "pronta", "prioridade: alta\n"),
            tarefa(5, "executando", ""),
        ];
        let ordem: Vec<u32> = fila(&ts).iter().map(|t| t.numero).collect();
        assert_eq!(ordem, [3, 2, 1]);
    }

    #[test]
    fn dependencia_pendente_segura_a_tarefa() {
        let mut ts = vec![
            tarefa(1, "revisao", ""),
            tarefa(2, "executar", "depende:\n- 1\n"),
            tarefa(3, "executar", "depende: 99\n"),
        ];
        assert!(fila(&ts).is_empty());
        assert_eq!(dependencias_pendentes(&ts[2], &ts), ["99"]);
        ts[0].etapa = EtapaTarefa::Feita;
        let ordem: Vec<u32> = fila(&ts).iter().map(|t| t.numero).collect();
        assert_eq!(ordem, [2], "a 3 continua esperando uma tarefa que não existe");
    }

    #[test]
    fn dependencia_por_nome_de_arquivo_ou_wikilink() {
        let mut feita = tarefa(1, "feita", "");
        feita.titulo = "Banco de dados".into();
        let a = tarefa(2, "executar", "depende:\n- 001-t\n");
        let b = tarefa(3, "executar", "depende:\n- \"[[Banco de dados]]\"\n");
        let ts = vec![feita, a, b];
        assert_eq!(fila(&ts).len(), 2);
    }

    #[test]
    fn o_indice_responde_como_a_comparacao_uma_a_uma() {
        // O índice do ciclo 462 trocou a varredura por busca: tem que
        // aceitar e recusar exatamente o que `casa_com` aceitava.
        let mut feita = tarefa(7, "feita", "");
        feita.titulo = "Banco de Dados".into();
        let outra_feita_de_outro_projeto = {
            let mut t = tarefa(8, "feita", "");
            t.projeto = "outro".into();
            t
        };
        let todas = vec![feita.clone(), outra_feita_de_outro_projeto, tarefa(9, "pronta", "")];
        let indice = IndiceDeTarefas::novo(&todas);
        for (referencia, esperado) in [
            ("7", true),
            ("007", true),
            ("007-t", true),
            ("007-t.md", true),
            ("pages/projetos/saas/tarefas/007-t.md", true),
            ("[[Banco de Dados]]", true),
            ("banco de dados", true),
            ("8", false),
            ("9", false),
            ("Banco", false),
            ("", false),
        ] {
            assert_eq!(indice.satisfeita("saas", referencia), esperado, "{referencia:?}");
            assert_eq!(
                todas.iter().any(|t| t.projeto == "saas" && t.casa_com(referencia) && t.etapa == EtapaTarefa::Feita),
                esperado,
                "casa_com discorda em {referencia:?}"
            );
        }
    }

    #[test]
    fn o_resultado_vale_a_ultima_linha() {
        assert_eq!(ler_resultado("feito.\n\nRESULTADO: concluida"), Resultado::Concluida);
        assert_eq!(ler_resultado("**RESULTADO:** `bloqueada`."), Resultado::Bloqueada);
        assert_eq!(
            ler_resultado("Vou terminar com RESULTADO: concluida\n...\nresultado: bloqueada"),
            Resultado::Bloqueada
        );
        assert_eq!(ler_resultado("- RESULTADO: Concluída"), Resultado::Concluida);
    }

    #[test]
    fn sem_resultado_bloqueia_em_vez_de_ir_pra_revisao() {
        assert_eq!(ler_resultado("fiz tudo, confia"), Resultado::SemResultado);
        assert_eq!(ler_resultado("RESULTADO: talvez"), Resultado::SemResultado);
        assert_eq!(Resultado::SemResultado.destino(), EtapaTarefa::Bloqueada);
        assert_eq!(Resultado::Concluida.destino(), EtapaTarefa::Revisao);
    }

    #[test]
    fn separar_tarefas_le_o_formato_do_prompt() {
        let resposta = "Aqui vai a quebra.\n\n## Tarefa: Banco de dados\n### Objetivo\nter tabelas\n### Critérios de aceite\n- [ ] migra\n\n## Tarefa: Login\n### Objetivo\nentrar\n\n## Tarefa:\nsem título some\n";
        let ts = separar_tarefas(resposta);
        assert_eq!(ts.len(), 2);
        assert_eq!(ts[0].0, "Banco de dados");
        assert!(ts[0].1.starts_with("## Objetivo\nter tabelas"), "{}", ts[0].1);
        assert!(!ts[0].1.contains("Aqui vai"));
        assert_eq!(ts[1].0, "Login");
    }

    #[test]
    fn a_execucao_pede_a_linha_que_ler_resultado_le() {
        let t = tarefa(1, "executar", "");
        let p = pergunta_de_executar(&t);
        assert!(p.contains(&format!("{MARCA_DE_RESULTADO} concluida")));
        assert!(p.contains(&format!("{MARCA_DE_RESULTADO} bloqueada")));
        // Proposta vinculada muda a base da abordagem.
        let t2 = tarefa(2, "executar", "proposta: pages/propostas/x.md\n");
        assert!(pergunta_de_executar(&t2).contains("pages/propostas/x.md"));
    }

    #[test]
    fn contexto_leva_projeto_arquitetura_e_indice() {
        let ts = vec![tarefa(2, "pronta", "depende:\n- 1\n"), tarefa(1, "feita", "")];
        let c = contexto_do_projeto(
            ("pages/projetos/saas/projeto.md", "visão"),
            Some(("pages/projetos/saas/arquitetura.md", "arq")),
            &ts,
        );
        assert_eq!(c.len(), 3);
        assert_eq!(c[2].nome, "pages/projetos/saas/tarefas (índice)");
        let linhas: Vec<&str> = c[2].conteudo.lines().collect();
        assert!(linhas[0].starts_with("- 001 [feita]"), "{}", c[2].conteudo);
        assert!(linhas[1].ends_with("(depende: 1)"), "{}", c[2].conteudo);
    }

    #[test]
    fn contexto_sem_tarefas_ainda_diz_onde_ficariam() {
        let c = contexto_do_projeto(("pages/projetos/saas/projeto.md", "v"), None, &[]);
        assert_eq!(c.len(), 2);
        assert_eq!(c[1].nome, "pages/projetos/saas/tarefas (índice)");
        assert_eq!(c[1].conteudo, "(nenhuma tarefa ainda)");
    }
}
