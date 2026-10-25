//! IO do módulo Projeto (ciclo 458).
//!
//! Tudo o que GUI, TUI e CLI fazem com projeto passa por aqui. O motivo
//! é o mesmo do ciclo 423: quem tem duas implementações tem duas UIs que
//! divergem. A regra mora em `anotadinho_core::projeto`; aqui só se lê,
//! se grava e se tranca.

use std::path::{Path, PathBuf};

use anotadinho_core::conversa::{self, Autor, Contexto, Mensagem};
use anotadinho_core::projeto::{
    self, ConfigProjeto, ConfigProjetos, EtapaTarefa, Projeto, Quem, Tarefa,
};
use anotadinho_core::MarkdownCodec;
use serde::{Deserialize, Serialize};

use crate::repositorio;
use crate::{handle_freio_do_dia, handle_read_page, handle_write_page};

/// O que `handle_criar_projeto` fez.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjetoCriado {
    /// Slug, que é também o nome da pasta.
    pub slug: String,
    /// Páginas criadas agora.
    pub criadas: Vec<String>,
    /// Páginas que já existiam e ficaram como estavam.
    pub existiam: Vec<String>,
}

/// Cria um projeto: as três páginas e a pasta de tarefas.
///
/// Nunca sobrescreve. Rodar de novo num projeto que existe só completa
/// o que falta, e diz o que já estava lá: perder a arquitetura de alguém
/// porque clicou "novo projeto" com o mesmo nome não é aceitável.
pub fn handle_criar_projeto(
    vault_path: String,
    titulo: String,
    slug: Option<String>,
    hoje: String,
) -> Result<ProjetoCriado, String> {
    let titulo = titulo.trim();
    if titulo.is_empty() {
        return Err("o projeto precisa de um título".to_string());
    }
    let slug = anotadinho_core::fluxo::slug_de_titulo(slug.as_deref().unwrap_or(titulo));
    let raiz = Path::new(&vault_path);
    let mut criado = ProjetoCriado { slug: slug.clone(), criadas: Vec::new(), existiam: Vec::new() };
    for (caminho, conteudo) in projeto::montar_projeto(&slug, titulo, &hoje) {
        if raiz.join(&caminho).exists() {
            criado.existiam.push(caminho);
            continue;
        }
        handle_write_page(vault_path.clone(), caminho.clone(), conteudo)?;
        criado.criadas.push(caminho);
    }
    let tarefas = raiz.join(projeto::pasta_das_tarefas(&slug));
    std::fs::create_dir_all(&tarefas).map_err(|e| format!("erro criando {}: {e}", tarefas.display()))?;
    Ok(criado)
}

/// Os projetos do vault, em ordem de slug.
///
/// Lê só `pages/projetos/*/projeto.md`. Uma pasta sem página de projeto
/// (ou com `type` errado) não é projeto, e fica de fora sem erro.
pub fn handle_listar_projetos(vault_path: String) -> Result<Vec<Projeto>, String> {
    let pasta = Path::new(&vault_path).join(projeto::PASTA);
    let Ok(entradas) = std::fs::read_dir(&pasta) else {
        return Ok(Vec::new());
    };
    let mut lista: Vec<Projeto> = entradas
        .flatten()
        .filter(|e| e.path().is_dir())
        .filter_map(|e| {
            let slug = e.file_name().to_string_lossy().to_string();
            let caminho = projeto::caminho_do_projeto(&slug);
            let texto = handle_read_page(vault_path.clone(), caminho.clone()).ok()?;
            Projeto::ler(&caminho, &texto)
        })
        .collect();
    lista.sort_by(|a, b| a.slug.cmp(&b.slug));
    Ok(lista)
}

/// As tarefas de um projeto, em ordem de número.
///
/// Lê só a pasta de tarefas: a fila checa isto a cada poucos segundos, e
/// varrer as 250 páginas do vault pra olhar dez seria o tipo de custo
/// que só aparece quando o vault cresce.
pub fn handle_tarefas_do_projeto(vault_path: String, slug: String) -> Result<Vec<Tarefa>, String> {
    let pasta = projeto::pasta_das_tarefas(&slug);
    let Ok(entradas) = std::fs::read_dir(Path::new(&vault_path).join(&pasta)) else {
        return Ok(Vec::new());
    };
    let mut lista: Vec<Tarefa> = entradas
        .flatten()
        .filter_map(|e| {
            let nome = e.file_name().to_string_lossy().to_string();
            let caminho = format!("{pasta}/{nome}");
            if !projeto::e_caminho_de_tarefa(&caminho) {
                return None;
            }
            let texto = handle_read_page(vault_path.clone(), caminho.clone()).ok()?;
            Tarefa::ler(&caminho, &texto)
        })
        .collect();
    lista.sort_by(|a, b| (a.numero, &a.caminho).cmp(&(b.numero, &b.caminho)));
    Ok(lista)
}

/// O texto de uma tarefa nova, e onde ela vai morar.
///
/// Não grava: a GUI e a TUI gravam (é a pessoa criando), o CLI propõe (é
/// o canal do agente). O número sai da pasta como ela está agora.
pub fn handle_montar_tarefa(
    vault_path: String,
    slug: String,
    titulo: String,
    corpo: String,
    hoje: String,
) -> Result<(String, String), String> {
    if titulo.trim().is_empty() {
        return Err("a tarefa precisa de um título".to_string());
    }
    let pasta = Path::new(&vault_path).join(projeto::pasta_das_tarefas(&slug));
    let existentes: Vec<String> = std::fs::read_dir(&pasta)
        .map(|it| it.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    let numero = projeto::proximo_numero(&existentes);
    let caminho = projeto::caminho_da_tarefa(&slug, numero, &titulo);
    Ok((caminho, projeto::montar_tarefa(&slug, &titulo, &corpo, &hoje)))
}

/// Move a etapa de uma tarefa, como a PESSOA.
///
/// Não existe variante que receba `Quem`: `executando` e `revisao` são
/// registro do que o agente fez, e o único jeito de chegar lá é pela
/// fila (`handle_tarefas_a_executar`) e pelo fechamento
/// (`handle_fechar_tarefa`).
pub fn handle_mudar_etapa_tarefa(
    vault_path: String,
    caminho: String,
    destino: EtapaTarefa,
) -> Result<String, String> {
    let atual = handle_read_page(vault_path.clone(), caminho.clone())?;
    let novo = projeto::mudar_etapa(&atual, destino, Quem::Humano)?;
    // Aceitar commita (ciclo 517), ANTES de gravar a etapa: se o commit
    // falha (um hook, o git sem nome), a tarefa fica em revisão e o erro
    // diz por quê — "feita" com o trabalho fora do histórico seria pior.
    if destino == EtapaTarefa::Feita {
        if let Some(t) = Tarefa::ler(&caminho, &atual).filter(|t| t.etapa == EtapaTarefa::Revisao) {
            let cfg = handle_ler_projetos_cfg(vault_path.clone())?.remove(&t.projeto).unwrap_or_default();
            if let Some(repo) = cfg.repositorio.as_deref().filter(|_| cfg.commitar_ao_aceitar) {
                if repositorio::e_git(repo) {
                    repositorio::commitar(repo, &format!("tarefa {:03}: {}", t.numero, t.titulo))?;
                }
            }
        }
    }
    handle_write_page(vault_path, caminho, novo.clone())?;
    Ok(novo)
}

/// A config dos projetos. Sem arquivo, vazia (cada projeto vale o padrão).
///
/// Arquivo ilegível é ERRO, e não padrão: o padrão tem `pausado: false`,
/// e uma config quebrada que despausasse a fila sem ninguém pedir é o
/// tipo de surpresa que custa um repositório.
pub fn handle_ler_projetos_cfg(vault_path: String) -> Result<ConfigProjetos, String> {
    let arquivo = Path::new(&vault_path).join(projeto::ARQUIVO_CONFIG);
    let Ok(texto) = std::fs::read_to_string(&arquivo) else {
        return Ok(ConfigProjetos::new());
    };
    serde_json::from_str(&texto).map_err(|e| format!("projetos.json ilegível: {e}"))
}

/// Grava a config dos projetos.
pub fn handle_gravar_projetos_cfg(vault_path: String, config: ConfigProjetos) -> Result<(), String> {
    let arquivo = Path::new(&vault_path).join(projeto::ARQUIVO_CONFIG);
    if let Some(pai) = arquivo.parent() {
        std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
    }
    let json = serde_json::to_string_pretty(&config).map_err(|e| e.to_string())?;
    std::fs::write(&arquivo, json).map_err(|e| format!("erro gravando projetos.json: {e}"))
}

/// Uma tarefa reservada e pronta pra ir pro agente.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrabalhoDeTarefa {
    /// Slug do projeto.
    pub projeto: String,
    /// Caminho da tarefa, já em `executando`.
    pub tarefa: String,
    /// A conversa onde a resposta vai ser gravada.
    pub conversa: String,
    /// O prompt exato, já montado e podado por `envio::montar`.
    pub prompt: String,
    /// Diretório de trabalho do agente: o repositório do projeto.
    pub cwd: String,
    /// O nome do agente que roda isto (ciclo 521), da config do projeto.
    /// `None` = o agente padrão de quem executa. Quem executa resolve com
    /// `agente::escolher`, e nome que não existe fecha a tarefa.
    #[serde(default)]
    pub agente: Option<String>,
    /// Executar ou revisar (ciclo 521).
    #[serde(default)]
    pub papel: projeto::Papel,
}

/// O que a checagem da fila achou.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct FilaDosProjetos {
    /// O que iniciar agora. Já reservado: quem chama TEM que iniciar ou
    /// fechar como bloqueada, senão a tarefa fica presa em `executando`.
    pub iniciar: Vec<TrabalhoDeTarefa>,
    /// Por que algo que estava em `executar` não saiu, pra tela contar.
    pub avisos: Vec<String>,
}

/// Quanto tempo uma trava pode ficar sem dono antes de ser considerada
/// abandonada. A checagem toda leva milissegundos; um processo que morreu
/// segurando a trava não pode parar a fila pra sempre.
const TRAVA_ABANDONADA_S: u64 = 30;

/// A trava de arquivo que deixa UM processo reservar por vez.
///
/// GUI e TUI no mesmo vault checam a fila cada uma no seu relógio. Sem a
/// trava, as duas podem ler `executar` ao mesmo tempo e as duas
/// reservarem: a releitura só protege quem chega DEPOIS da gravação do
/// outro. `create_new` é atômico no sistema de arquivos, nos dois
/// sistemas.
struct Trava(PathBuf);

impl Trava {
    fn pegar(vault_path: &str) -> Result<Trava, String> {
        let arquivo = Path::new(vault_path).join(".anotadinho/projetos.lock");
        if let Some(pai) = arquivo.parent() {
            std::fs::create_dir_all(pai).map_err(|e| e.to_string())?;
        }
        for _ in 0..2 {
            match std::fs::OpenOptions::new().write(true).create_new(true).open(&arquivo) {
                Ok(_) => return Ok(Trava(arquivo)),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                    let velha = std::fs::metadata(&arquivo)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|t| t.elapsed().ok())
                        .is_some_and(|d| d.as_secs() >= TRAVA_ABANDONADA_S);
                    if !velha {
                        return Err("outra janela está conferindo a fila agora".to_string());
                    }
                    let _ = std::fs::remove_file(&arquivo);
                }
                Err(e) => return Err(format!("não consegui travar a fila: {e}")),
            }
        }
        Err("não consegui travar a fila".to_string())
    }
}

impl Drop for Trava {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

/// O que a fila precisa pra montar o prompt — é o que a UI já tem das
/// preferências.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParametrosDaFila {
    /// `"YYYY-MM-DD HH:MM"`: o ipc não tem relógio.
    pub agora: String,
    /// Teto de tokens do prompt (`0` desliga a poda).
    pub teto: usize,
    /// Quantas mensagens recentes da conversa da tarefa entram.
    pub limite_historico: usize,
    /// Quantas tarefas o frontend consegue INICIAR agora (vagas livres
    /// na fila de agentes dele). `None` = sem limite.
    ///
    /// Reservar sem vaga pra iniciar deixaria a tarefa em `executando`
    /// sem processo nenhum, segurando o projeto (ciclo 459).
    #[serde(default)]
    pub vagas: Option<usize>,
}

/// Confere a fila de todos os projetos e RESERVA o que pode começar.
///
/// É a peça que a GUI e a TUI compartilham. Cada uma tem o seu executor
/// de agente (ciclo 459), mas a decisão do que rodar é uma só.
///
/// A vaga é contada pelo VAULT, e não pela memória do processo: uma
/// tarefa em `executando` ocupa vaga, seja qual for o frontend que a
/// pegou. Com isso, duas janelas abertas não somam concorrência.
pub fn handle_tarefas_a_executar(
    vault_path: String,
    params: ParametrosDaFila,
) -> Result<FilaDosProjetos, String> {
    let mut saida = FilaDosProjetos::default();
    let projetos = handle_listar_projetos(vault_path.clone())?;
    if projetos.is_empty() {
        return Ok(saida);
    }
    let config = handle_ler_projetos_cfg(vault_path.clone())?;

    // Barato primeiro: sem nada em `executar`, não há o que travar.
    let mut com_pedido: Vec<(Projeto, Vec<Tarefa>)> = Vec::new();
    for p in projetos {
        let tarefas = handle_tarefas_do_projeto(vault_path.clone(), p.slug.clone())?;
        if tarefas.iter().any(|t| t.etapa == EtapaTarefa::Executar) {
            com_pedido.push((p, tarefas));
        }
    }
    if com_pedido.is_empty() {
        return Ok(saida);
    }

    let freio = handle_freio_do_dia(vault_path.clone(), params.agora.clone())?;
    if freio.parou() {
        // Diz QUAIS projetos esperam (ciclo 464): "a fila parou" sem dizer
        // de quem não ajuda quem tem vários, e o texto igual a cada volta
        // fazia a tela (que só repete aviso que mudou) calar o de um
        // projeto novo esperando.
        let esperando: Vec<&str> = com_pedido.iter().map(|(p, _)| p.slug.as_str()).collect();
        saida.avisos.push(format!(
            "fila parada pelo freio do dia: {} — esperando: {}",
            freio.motivo(),
            esperando.join(", ")
        ));
        return Ok(saida);
    }

    let mut livres = params.vagas.unwrap_or(usize::MAX);
    if livres == 0 {
        return Ok(saida);
    }
    let _trava = Trava::pegar(&vault_path)?;
    for (p, tarefas) in com_pedido {
        let cfg = config.get(&p.slug).cloned().unwrap_or_default();
        let (avisos, repositorio) = avisos_do_projeto(&p.slug, &cfg, &tarefas);
        saida.avisos.extend(avisos);
        let Some(repositorio) = repositorio else { continue };
        let ocupadas = tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Executando).count();
        let vagas = cfg.concorrencia.max(1).saturating_sub(ocupadas);
        let fila = projeto::fila(&tarefas);
        for t in fila.into_iter().take(vagas) {
            if livres == 0 {
                break;
            }
            match reservar_e_preparar(&vault_path, &p, t, &tarefas, &repositorio, &params) {
                Ok(Some(trabalho)) => {
                    livres -= 1;
                    saida.iniciar.push(trabalho)
                }
                Ok(None) => {}
                Err(e) => saida.avisos.push(format!("{}: {e}", t.caminho)),
            }
        }
    }
    Ok(saida)
}

/// Por que a fila de um projeto não anda, e o repositório se ela pode
/// andar.
///
/// Uma função só pra fila e pra tela (ciclo 460): o cabeçalho do projeto
/// tem que dizer EXATAMENTE o que segura a fila, e duas cópias da regra
/// acabariam dizendo coisas diferentes.
fn avisos_do_projeto(slug: &str, cfg: &ConfigProjeto, tarefas: &[Tarefa]) -> (Vec<String>, Option<String>) {
    let mut avisos = Vec::new();
    if cfg.pausado {
        avisos.push(format!("{slug}: fila pausada"));
        return (avisos, None);
    }
    let Some(repositorio) = cfg.repositorio.clone().filter(|r| !r.trim().is_empty()) else {
        avisos.push(format!("{slug}: sem repositório configurado — escolha a pasta do código"));
        return (avisos, None);
    };
    if !Path::new(&repositorio).is_dir() {
        avisos.push(format!("{slug}: o repositório {repositorio} não é uma pasta"));
        return (avisos, None);
    }
    let indice = projeto::IndiceDeTarefas::novo(tarefas);
    for t in tarefas.iter().filter(|t| t.etapa == EtapaTarefa::Executar) {
        let pendentes = indice.pendentes(t);
        if !pendentes.is_empty() {
            avisos.push(format!("{}: esperando {}", t.caminho, pendentes.join(", ")));
        }
    }
    (avisos, Some(repositorio))
}

/// Tudo que a tela de um projeto mostra, numa chamada (ciclo 460).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstadoDoProjeto {
    /// A página do projeto.
    pub projeto: Projeto,
    /// A config (o padrão, se não houver).
    pub config: ConfigProjeto,
    /// As tarefas, por número.
    pub tarefas: Vec<Tarefa>,
    /// O que segura a fila agora. Só LÊ: não trava nem reserva nada.
    pub avisos: Vec<String>,
    /// As sprints do projeto (ciclo 470), por número.
    #[serde(default)]
    pub sprints: Vec<anotadinho_core::sprint::Sprint>,
}

/// O estado de um projeto pra tela, sem efeito nenhum.
pub fn handle_estado_do_projeto(vault_path: String, slug: String) -> Result<EstadoDoProjeto, String> {
    let caminho = projeto::caminho_do_projeto(&slug);
    let texto = handle_read_page(vault_path.clone(), caminho.clone())?;
    let p = Projeto::ler(&caminho, &texto).ok_or_else(|| format!("{caminho} não é um projeto"))?;
    let config = handle_ler_projetos_cfg(vault_path.clone())?.remove(&slug).unwrap_or_default();
    let tarefas = handle_tarefas_do_projeto(vault_path.clone(), slug.clone())?;
    let sprints = crate::handle_sprints_do_projeto(vault_path, slug.clone())?;
    let (mut avisos, _) = avisos_do_projeto(&slug, &config, &tarefas);
    avisos.extend(anotadinho_core::sprint::avisos(&sprints, &tarefas));
    Ok(EstadoDoProjeto { projeto: p, config, tarefas, avisos, sprints })
}

/// Grava a config de UM projeto, sem tocar na dos outros.
pub fn handle_gravar_config_do_projeto(
    vault_path: String,
    slug: String,
    config: ConfigProjeto,
) -> Result<(), String> {
    let mut todas = handle_ler_projetos_cfg(vault_path.clone())?;
    todas.insert(slug, config);
    handle_gravar_projetos_cfg(vault_path, todas)
}

/// Cria uma tarefa, como a PESSOA: grava direto e devolve o caminho.
pub fn handle_criar_tarefa(
    vault_path: String,
    slug: String,
    titulo: String,
    hoje: String,
) -> Result<String, String> {
    let (caminho, conteudo) = handle_montar_tarefa(vault_path.clone(), slug, titulo, String::new(), hoje)?;
    if Path::new(&vault_path).join(&caminho).exists() {
        return Err(format!("{caminho} já existe"));
    }
    handle_write_page(vault_path, caminho.clone(), conteudo)?;
    Ok(caminho)
}

/// O índice das tarefas de um projeto, pro contexto da conversa dele.
pub fn handle_indice_do_projeto(vault_path: String, slug: String) -> Result<String, String> {
    let tarefas = handle_tarefas_do_projeto(vault_path, slug)?;
    Ok(projeto::indice_de_tarefas(&tarefas))
}

pub(crate) fn proposta_nova(
    vault_path: &str,
    autor: &str,
    alvo: String,
    conteudo: String,
    motivo: String,
    quando: String,
    lote: Option<String>,
) -> anotadinho_core::proposta::Proposta {
    use anotadinho_core::proposta::{Operacao, Proposta};
    let existe = Path::new(vault_path).join(&alvo).exists();
    let segundos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    Proposta {
        // Alvo + relógio, como o CLI: legível na fila, e cada tarefa do
        // lote tem alvo próprio, então não colidem no mesmo segundo.
        id: format!("{}-{segundos}", anotadinho_core::fluxo::slug_de_titulo(&alvo)),
        autor: autor.to_string(),
        quando,
        motivo,
        alvo,
        operacao: if existe { Operacao::Substituir } else { Operacao::Criar },
        conteudo,
        origem: None,
        revisao: None,
        lote,
    }
}

/// "Virar tarefas" (ciclo 460): cada `## Tarefa:` da resposta vira uma
/// proposta de tarefa, todas num lote.
///
/// PROPÕE, não grava: o prompt semeado diz "eu escolho quais viram
/// tarefa", e escolher é aplicar parte do lote na revisão. A numeração
/// é sequencial a partir da pasta: se cada tarefa pedisse o próximo
/// número ao disco, as cinco de um lote nasceriam todas como 004, já
/// que nenhuma existe até a aplicação.
pub fn handle_propor_tarefas(
    vault_path: String,
    slug: String,
    resposta: String,
    quando: String,
) -> Result<Vec<String>, String> {
    if !Path::new(&vault_path).join(projeto::caminho_do_projeto(&slug)).exists() {
        return Err(format!("não existe o projeto {slug}"));
    }
    let tarefas = projeto::separar_tarefas(&resposta);
    if tarefas.is_empty() {
        return Err(format!(
            "a resposta não tem nenhuma tarefa no formato \"{} <título>\"",
            projeto::CABECALHO_DE_TAREFA
        ));
    }
    let pasta = Path::new(&vault_path).join(projeto::pasta_das_tarefas(&slug));
    let existentes: Vec<String> = std::fs::read_dir(&pasta)
        .map(|it| it.flatten().map(|e| e.file_name().to_string_lossy().to_string()).collect())
        .unwrap_or_default();
    let primeiro = projeto::proximo_numero(&existentes);
    let hoje = quando.split_whitespace().next().unwrap_or("").to_string();
    let lote = format!("tarefas-{slug}-{}", anotadinho_core::fluxo::slug_de_titulo(&quando));
    let mut ids = Vec::new();
    for (i, (titulo, corpo)) in tarefas.into_iter().enumerate() {
        let caminho = projeto::caminho_da_tarefa(&slug, primeiro + i as u32, &titulo);
        let conteudo = projeto::montar_tarefa(&slug, &titulo, &corpo, &hoje);
        let p = proposta_nova(
            &vault_path,
            "conversa",
            caminho,
            conteudo,
            format!("tarefa da conversa: {titulo}"),
            quando.clone(),
            Some(lote.clone()),
        );
        ids.push(crate::handle_propor(vault_path.clone(), p)?);
    }
    Ok(ids)
}

/// "Atualizar arquitetura" (ciclo 460): a resposta entra como seção nova
/// no fim de `arquitetura.md`, por proposta.
///
/// Acrescenta em vez de substituir: a resposta é um trecho da conversa,
/// não a página inteira, e trocar a arquitetura pelo trecho apagaria o
/// resto. Quem revisa vê o diff e corta o que não quer antes de aplicar.
pub fn handle_propor_arquitetura(
    vault_path: String,
    slug: String,
    texto: String,
    quando: String,
) -> Result<String, String> {
    let caminho = projeto::caminho_da_arquitetura(&slug);
    let atual = handle_read_page(vault_path.clone(), caminho.clone())
        .map_err(|_| format!("não existe {caminho}"))?;
    let novo = format!("{}\n\n## Da conversa — {quando}\n\n{}\n", atual.trim_end(), texto.trim());
    let p = proposta_nova(&vault_path, "conversa", caminho, novo, "arquitetura vinda da conversa".to_string(), quando, None);
    crate::handle_propor(vault_path, p)
}

/// Reserva uma tarefa e monta o que o agente recebe.
///
/// `None` = alguém mudou a tarefa entre a listagem e agora (a pessoa
/// desistiu, ou outro processo reservou antes da trava existir): não é
/// erro, só não é mais nossa.
fn reservar_e_preparar(
    vault_path: &str,
    p: &Projeto,
    lida: &Tarefa,
    todas: &[Tarefa],
    repositorio: &str,
    params: &ParametrosDaFila,
) -> Result<Option<TrabalhoDeTarefa>, String> {
    // Releitura: a listagem é de antes da trava.
    let texto = handle_read_page(vault_path.to_string(), lida.caminho.clone())?;
    let Some(tarefa) = Tarefa::ler(&lida.caminho, &texto) else { return Ok(None) };
    if tarefa.etapa != EtapaTarefa::Executar {
        return Ok(None);
    }

    // A conversa e a pergunta vêm ANTES da reserva: se gravar a conversa
    // falhar, a tarefa continua em `executar` e a próxima checagem tenta
    // de novo, em vez de ficar presa em `executando` sem agente nenhum.
    let conversa_path = projeto::caminho_da_conversa_da_tarefa(&p.slug, tarefa.numero);
    let conversa_atual = handle_read_page(vault_path.to_string(), conversa_path.clone()).ok();
    let base = conversa_atual.clone().unwrap_or_else(|| pagina_da_conversa(p, &tarefa));
    let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&base);
    // Com sessão aberta (ciclo 433), o agente já tem a conversa: repetir
    // o histórico no prompt só pagaria tokens duas vezes. É o mesmo corte
    // que a TUI faz na conversa comum.
    let historico = if conversa::sessao_do_frontmatter(fm).is_some() {
        Vec::new()
    } else {
        conversa::parse(corpo)
    };
    // O repositório (ciclo 517): o branch da tarefa, se o projeto pede, e
    // o commit de onde ela parte — é contra ele que a revisão mostra o que
    // mudou. ANTES de qualquer gravação: um `git switch` que falha (árvore
    // suja em conflito) deixa a tarefa em `executar`, com o motivo no
    // aviso, em vez de rodar o agente no branch errado.
    let cfg = handle_ler_projetos_cfg(vault_path.to_string())?.remove(&p.slug).unwrap_or_default();
    let mut tarefa = tarefa;
    let mut texto = texto;
    if repositorio::e_git(repositorio) {
        if cfg.branch_por_tarefa {
            let nome = tarefa.branch_git.clone().unwrap_or_else(|| repositorio::nome_do_branch(tarefa.numero, &tarefa.titulo));
            repositorio::ir_pro_branch(repositorio, &nome).map_err(|e| format!("não troquei pro branch {nome}: {e}"))?;
            texto = MarkdownCodec::set_frontmatter_field(&texto, "branch_git", &nome).map_err(|e| e.to_string())?;
            tarefa.branch_git = Some(nome);
        }
        // A segunda rodada (refaça, ou depois de bloqueada) continua com a
        // base da PRIMEIRA: a revisão mostra tudo o que a tarefa mudou, e
        // não só a última rodada.
        if tarefa.base_git.is_none() {
            if let Some(b) = repositorio::base_atual(repositorio) {
                texto = MarkdownCodec::set_frontmatter_field(&texto, "base_git", &b).map_err(|e| e.to_string())?;
                tarefa.base_git = Some(b);
            }
        }
    }
    let pergunta = projeto::pergunta_de_executar_com(&tarefa, cfg.comando_de_verificacao());
    let minha = Mensagem { autor: Autor::Voce, quando: params.agora.clone(), texto: pergunta.clone() };
    let corpo_novo = conversa::append(corpo, &minha);
    let gravado = if fm.is_empty() { corpo_novo } else { format!("{fm}\n{corpo_novo}") };

    let contextos = contextos_da_execucao(vault_path, p, &tarefa, &texto, todas);
    let envio = anotadinho_core::envio::montar(
        &contextos,
        &historico,
        &pergunta,
        params.limite_historico,
        params.teto,
        vault_path,
    );

    handle_write_page(vault_path.to_string(), conversa_path.clone(), gravado)?;
    let reservado = projeto::mudar_etapa(&texto, EtapaTarefa::Executando, Quem::Sistema)?;
    handle_write_page(vault_path.to_string(), tarefa.caminho.clone(), reservado)?;

    Ok(Some(TrabalhoDeTarefa {
        projeto: p.slug.clone(),
        tarefa: tarefa.caminho.clone(),
        conversa: conversa_path,
        prompt: envio.prompt,
        cwd: repositorio.to_string(),
        agente: cfg.nome_do_executor().map(str::to_string),
        papel: projeto::Papel::Executor,
    }))
}

/// A página de conversa de uma tarefa que ainda não tinha.
///
/// O `contexto:` leva a tarefa, o projeto e a arquitetura: é o que a
/// janela e a TUI anexam quando a PESSOA conversa ali (detalhando, por
/// exemplo). A execução monta os anexos dela por conta própria
/// (`contextos_da_execucao`), sem ler esta lista.
fn pagina_da_conversa(p: &Projeto, tarefa: &Tarefa) -> String {
    let md = conversa::montar_pagina(
        &format!("Tarefa {:03} — {}", tarefa.numero, tarefa.titulo),
        Some(&tarefa.caminho),
        &[tarefa.caminho.clone(), p.caminho.clone(), projeto::caminho_da_arquitetura(&p.slug)],
    );
    let md = MarkdownCodec::set_frontmatter_field(&md, "projeto", &p.slug).unwrap_or(md);
    MarkdownCodec::set_frontmatter_field(&md, "tarefa", &tarefa.caminho).unwrap_or(md)
}

/// O que vai anexado na execução: a tarefa, o projeto, a arquitetura, o
/// índice das tarefas e a proposta vinculada, se houver.
///
/// A página que não dá pra ler fica de fora sem derrubar a execução: uma
/// arquitetura apagada não é motivo pra tarefa não rodar.
fn contextos_da_execucao(
    vault_path: &str,
    p: &Projeto,
    tarefa: &Tarefa,
    texto_da_tarefa: &str,
    todas: &[Tarefa],
) -> Vec<Contexto> {
    let ler = |c: &str| handle_read_page(vault_path.to_string(), c.to_string()).ok();
    let projeto_txt = ler(&p.caminho).unwrap_or_default();
    let arq_caminho = projeto::caminho_da_arquitetura(&p.slug);
    let arq = ler(&arq_caminho);
    let mut v = vec![Contexto { nome: tarefa.caminho.clone(), conteudo: texto_da_tarefa.to_string() }];
    v.extend(projeto::contexto_do_projeto(
        (&p.caminho, &projeto_txt),
        arq.as_deref().map(|a| (arq_caminho.as_str(), a)),
        todas,
    ));
    for vinculo in [tarefa.proposta.as_deref(), tarefa.spec.as_deref()].into_iter().flatten() {
        let caminho = vinculo.trim().trim_start_matches("[[").trim_end_matches("]]").trim();
        if let Some(conteudo) = ler(caminho) {
            v.push(Contexto { nome: caminho.to_string(), conteudo });
        }
    }
    v
}

/// A conversa de uma tarefa, criada se ainda não existe (ciclo 460).
///
/// É a MESMA conversa onde a execução fica registrada: detalhar a tarefa
/// com o agente e depois executá-la acontece no mesmo fio, e a execução
/// chega sabendo o que foi combinado no detalhe.
pub fn handle_conversa_da_tarefa(vault_path: String, caminho: String) -> Result<String, String> {
    let texto = handle_read_page(vault_path.clone(), caminho.clone())?;
    let tarefa = Tarefa::ler(&caminho, &texto).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    let conversa = projeto::caminho_da_conversa_da_tarefa(&tarefa.projeto, tarefa.numero);
    if Path::new(&vault_path).join(&conversa).exists() {
        return Ok(conversa);
    }
    let p_caminho = projeto::caminho_do_projeto(&tarefa.projeto);
    let p_texto = handle_read_page(vault_path.clone(), p_caminho.clone())?;
    let p = Projeto::ler(&p_caminho, &p_texto).ok_or_else(|| format!("{p_caminho} não é um projeto"))?;
    handle_write_page(vault_path, conversa.clone(), pagina_da_conversa(&p, &tarefa))?;
    Ok(conversa)
}

/// Como uma execução de tarefa acabou, do ponto de vista do executor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "fim", content = "texto", rename_all = "kebab-case")]
pub enum FimDaTarefa {
    /// O agente respondeu: o resultado sai da resposta.
    Respondeu(String),
    /// O processo falhou ou não subiu.
    Falhou(String),
    /// A pessoa parou.
    Interrompida,
}

/// Fecha uma tarefa que estava em `executando`.
///
/// Só a etapa. A resposta na conversa e o registro de execuções já são
/// feitos pelos dois executores pra toda conversa; repetir aqui gravaria
/// a resposta duas vezes.
///
/// Se a tarefa não está mais em `executando` (a pessoa moveu pra
/// `bloqueada` durante a execução), não mexe: a decisão da pessoa vale
/// mais que o relato do agente. Devolve a etapa em que a tarefa ficou.
pub fn handle_fechar_tarefa(
    vault_path: String,
    caminho: String,
    fim: FimDaTarefa,
) -> Result<EtapaTarefa, String> {
    handle_fechar_tarefa_com(vault_path, caminho, fim, repositorio::Shell::do_sistema())
}

/// [`handle_fechar_tarefa`] com o shell da verificação por parâmetro
/// (ciclo 517): é por aqui que o teste exerce o caminho de cada sistema.
///
/// Com `verificar` no projeto, a tarefa que o agente deu por concluída só
/// vai pra `revisao` se o comando passar no repositório; senão vai pra
/// `bloqueada`. O relato da verificação entra na seção **Verificação** da
/// tarefa, que é anexada na próxima rodada: o agente lê o que quebrou.
pub fn handle_fechar_tarefa_com(
    vault_path: String,
    caminho: String,
    fim: FimDaTarefa,
    shell: repositorio::Shell,
) -> Result<EtapaTarefa, String> {
    let texto = handle_read_page(vault_path.clone(), caminho.clone())?;
    let tarefa = Tarefa::ler(&caminho, &texto).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    if tarefa.etapa != EtapaTarefa::Executando {
        return Ok(tarefa.etapa);
    }
    let mut destino = match &fim {
        FimDaTarefa::Respondeu(resposta) => projeto::ler_resultado(resposta).destino(),
        FimDaTarefa::Falhou(_) | FimDaTarefa::Interrompida => EtapaTarefa::Bloqueada,
    };
    let mut texto = texto;
    if destino == EtapaTarefa::Revisao {
        let cfg = handle_ler_projetos_cfg(vault_path.clone())?.remove(&tarefa.projeto).unwrap_or_default();
        if let (Some(cmd), Some(repo)) = (cfg.comando_de_verificacao(), cfg.repositorio.as_deref().filter(|r| Path::new(r).is_dir())) {
            let v = repositorio::verificar(repo, cmd, std::time::Duration::from_secs(cfg.prazo_da_verificacao.max(1)), shell);
            if !v.passou {
                destino = EtapaTarefa::Bloqueada;
            }
            texto = projeto::com_verificacao(&texto, v.passou, &v.relato())?;
        }
    }
    // A pessoa pode ter mexido na tarefa enquanto a verificação rodava
    // (minutos): relê e só fecha se ainda está em `executando`.
    let agora = handle_read_page(vault_path.clone(), caminho.clone())?;
    if Tarefa::ler(&caminho, &agora).is_some_and(|t| t.etapa != EtapaTarefa::Executando) {
        return Ok(Tarefa::ler(&caminho, &agora).map(|t| t.etapa).unwrap_or(destino));
    }
    let novo = projeto::mudar_etapa(&texto, destino, Quem::Sistema)?;
    handle_write_page(vault_path, caminho, novo)?;
    Ok(destino)
}

/// O que a tarefa mudou no repositório do projeto (ciclo 517), contra o
/// commit de onde a primeira rodada partiu.
pub fn handle_mudancas_da_tarefa(vault_path: String, caminho: String) -> Result<repositorio::Mudancas, String> {
    let texto = handle_read_page(vault_path.clone(), caminho.clone())?;
    let tarefa = Tarefa::ler(&caminho, &texto).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    let cfg = handle_ler_projetos_cfg(vault_path)?.remove(&tarefa.projeto).unwrap_or_default();
    let Some(repo) = cfg.repositorio.filter(|r| Path::new(r).is_dir()) else {
        return Err(format!("o projeto {} não tem repositório configurado", tarefa.projeto));
    };
    let mut m = repositorio::mudancas_desde(&repo, tarefa.base_git.as_deref());
    if m.aviso.is_none() && tarefa.base_git.is_none() {
        m.aviso = Some("a tarefa ainda não rodou pelo app: mostrando o que está mudado no repositório agora".into());
    }
    Ok(m)
}

/// A tarefa dona desta conversa, se houver uma (o `tarefa:` do
/// frontmatter), sem mexer em nada.
pub fn handle_tarefa_da_conversa(vault_path: String, conversa_path: String) -> Option<String> {
    let texto = handle_read_page(vault_path, conversa_path).ok()?;
    let (fm, _) = MarkdownCodec::split_frontmatter(&texto).ok()?;
    fm.extra.get("tarefa").and_then(|v| v.as_str()).map(str::to_string)
}

/// Fecha a tarefa dona desta conversa, se houver uma.
///
/// É o que os executores chamam no fim de TODA execução, sem precisar
/// saber se a conversa é de tarefa: a conversa de execução tem `tarefa:`
/// no frontmatter (gravado por `handle_tarefas_a_executar`), e as
/// outras não têm, então pra elas isto não faz nada. Uma pergunta que a
/// pessoa mande à mão na conversa da tarefa, com a tarefa em `revisao`,
/// também passa aqui e também não faz nada: `fechar` só mexe em
/// `executando`.
pub fn handle_fechar_tarefa_da_conversa(
    vault_path: String,
    conversa_path: String,
    fim: FimDaTarefa,
) -> Result<Option<EtapaTarefa>, String> {
    let Ok(texto) = handle_read_page(vault_path.clone(), conversa_path) else {
        return Ok(None);
    };
    let Ok((fm, _)) = MarkdownCodec::split_frontmatter(&texto) else {
        return Ok(None);
    };
    let Some(tarefa) = fm.extra.get("tarefa").and_then(|v| v.as_str()).map(str::to_string) else {
        return Ok(None);
    };
    handle_fechar_tarefa(vault_path, tarefa, fim).map(Some)
}

#[cfg(test)]
mod testes {
    use super::*;

    const AGORA: &str = "2026-10-21 10:00";

    fn params() -> ParametrosDaFila {
        ParametrosDaFila { agora: AGORA.into(), teto: 0, limite_historico: 12, vagas: None }
    }

    /// Vault com um projeto `saas`, um repositório de verdade e `n`
    /// tarefas, todas nascidas em `ideia`.
    fn montar(n: usize) -> (tempfile::TempDir, tempfile::TempDir, String, Vec<String>) {
        let vault = tempfile::TempDir::new().unwrap();
        let repo = tempfile::TempDir::new().unwrap();
        let v = vault.path().to_string_lossy().to_string();
        handle_criar_projeto(v.clone(), "SaaS".into(), Some("saas".into()), "2026-10-21".into()).unwrap();
        let mut cfg = ConfigProjetos::new();
        cfg.insert(
            "saas".into(),
            ConfigProjeto { repositorio: Some(repo.path().to_string_lossy().to_string()), ..Default::default() },
        );
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        let mut caminhos = Vec::new();
        for i in 0..n {
            let (c, md) =
                handle_montar_tarefa(v.clone(), "saas".into(), format!("Tarefa {i}"), String::new(), "2026-10-21".into())
                    .unwrap();
            handle_write_page(v.clone(), c.clone(), md).unwrap();
            caminhos.push(c);
        }
        (vault, repo, v, caminhos)
    }

    fn pedir_execucao(v: &str, caminho: &str) {
        handle_mudar_etapa_tarefa(v.into(), caminho.into(), EtapaTarefa::Pronta).unwrap();
        handle_mudar_etapa_tarefa(v.into(), caminho.into(), EtapaTarefa::Executar).unwrap();
    }

    fn etapa(v: &str, caminho: &str) -> EtapaTarefa {
        Tarefa::ler(caminho, &handle_read_page(v.into(), caminho.into()).unwrap()).unwrap().etapa
    }

    // ── O repositório (ciclo 517) ───────────────────────────────────

    fn git(repo: &str, args: &[&str]) {
        let s = std::process::Command::new("git").arg("-C").arg(repo).args(args).output().unwrap();
        assert!(s.status.success(), "git {args:?}: {}", String::from_utf8_lossy(&s.stderr));
    }

    /// O `montar`, com o repositório virando git (um commit) e a config do
    /// projeto mexida por `f`.
    fn montar_com_git(n: usize, f: impl Fn(&mut ConfigProjeto)) -> (tempfile::TempDir, tempfile::TempDir, String, String, Vec<String>) {
        let (g, r, v, c) = montar(n);
        let repo = r.path().to_string_lossy().to_string();
        for a in [&["init", "-q", "-b", "main"][..], &["config", "user.email", "t@t"], &["config", "user.name", "t"]] {
            git(&repo, a);
        }
        std::fs::write(r.path().join("app.txt"), "v1\n").unwrap();
        git(&repo, &["add", "-A"]);
        git(&repo, &["commit", "-qm", "início"]);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        f(cfg.get_mut("saas").unwrap());
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        (g, r, v, repo, c)
    }

    #[test]
    fn verificacao_que_falha_bloqueia_e_conta_o_que_quebrou() {
        let (_g, _r, v, _repo, c) = montar_com_git(1, |cfg| cfg.verificar = Some("echo 2 testes falharam; exit 1".into()));
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        assert!(t.prompt.contains("rode `echo 2 testes falharam; exit 1`"), "o prompt não avisou do comando");
        let fim = handle_fechar_tarefa_com(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("feito\nRESULTADO: concluida".into()), repositorio::Shell::Sh).unwrap();
        assert_eq!(fim, EtapaTarefa::Bloqueada, "o agente disse concluída, mas o teste falhou");
        let texto = handle_read_page(v.clone(), c[0].clone()).unwrap();
        assert!(texto.contains("verificacao: falhou") && texto.contains("FALHOU (código 1)") && texto.contains("2 testes falharam"), "{texto}");
        // A próxima rodada sabe disso.
        let tarefa = Tarefa::ler(&c[0], &texto).unwrap();
        assert!(projeto::pergunta_de_executar_com(&tarefa, Some("x")).contains("já rodou antes"));
    }

    #[test]
    fn verificacao_que_passa_vai_pra_revisao_e_a_que_nao_existe_e_o_agente_que_decide() {
        let (_g, _r, v, _repo, c) = montar_com_git(2, |cfg| cfg.verificar = Some("true".into()));
        pedir_execucao(&v, &c[0]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        let fim = handle_fechar_tarefa_com(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()), repositorio::Shell::Sh).unwrap();
        assert_eq!(fim, EtapaTarefa::Revisao);
        assert!(handle_read_page(v.clone(), c[0].clone()).unwrap().contains("verificacao: passou"));
        // O agente que diz bloqueada não paga a verificação.
        pedir_execucao(&v, &c[1]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        let fim = handle_fechar_tarefa_com(v.clone(), c[1].clone(), FimDaTarefa::Respondeu("RESULTADO: bloqueada".into()), repositorio::Shell::Sh).unwrap();
        assert_eq!(fim, EtapaTarefa::Bloqueada);
        assert!(!handle_read_page(v, c[1].clone()).unwrap().contains("verificacao:"));
    }

    #[test]
    fn a_tarefa_guarda_a_base_e_o_branch_e_a_revisao_mostra_o_que_mudou() {
        let (_g, r, v, repo, c) = montar_com_git(1, |cfg| cfg.branch_por_tarefa = true);
        pedir_execucao(&v, &c[0]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        let tarefa = Tarefa::ler(&c[0], &handle_read_page(v.clone(), c[0].clone()).unwrap()).unwrap();
        let base = tarefa.base_git.clone().expect("a base não foi guardada");
        let branch = tarefa.branch_git.clone().expect("o branch não foi guardado");
        assert!(branch.starts_with("tarefa/001-"), "{branch}");
        assert_eq!(repositorio::branch_atual(&repo).as_deref(), Some(branch.as_str()), "o repositório não foi pro branch");
        // O "agente" commita uma coisa e deixa outra na árvore.
        std::fs::write(r.path().join("app.txt"), "v2\n").unwrap();
        git(&repo, &["commit", "-qam", "agente"]);
        std::fs::write(r.path().join("novo.txt"), "oi\n").unwrap();
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into())).unwrap();
        let m = handle_mudancas_da_tarefa(v.clone(), c[0].clone()).unwrap();
        assert_eq!(m.base.as_deref(), Some(base.as_str()));
        assert!(m.diff.contains("-v1") && m.diff.contains("+v2") && m.diff.contains("+oi"), "{}", m.diff);
        assert_eq!(m.arquivos.len(), 2);
        // A segunda rodada mantém a base da primeira.
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Executar).unwrap();
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        let de_novo = Tarefa::ler(&c[0], &handle_read_page(v.clone(), c[0].clone()).unwrap()).unwrap();
        assert_eq!(de_novo.base_git.as_deref(), Some(base.as_str()));
    }

    #[test]
    fn aceitar_commita_quando_o_projeto_pede() {
        let (_g, r, v, repo, c) = montar_com_git(1, |cfg| cfg.commitar_ao_aceitar = true);
        pedir_execucao(&v, &c[0]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        std::fs::write(r.path().join("app.txt"), "v2\n").unwrap();
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into())).unwrap();
        let antes = repositorio::base_atual(&repo).unwrap();
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Feita).unwrap();
        let depois = repositorio::base_atual(&repo).unwrap();
        assert_ne!(antes, depois, "não commitou");
        let log = std::process::Command::new("git").args(["-C", &repo, "log", "-1", "--format=%s"]).output().unwrap();
        assert!(String::from_utf8_lossy(&log.stdout).starts_with("tarefa 001: "), "{}", String::from_utf8_lossy(&log.stdout));
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Feita);
    }

    #[test]
    fn criar_de_novo_nao_sobrescreve() {
        let (_g, _r, v, _) = montar(0);
        let arq = projeto::caminho_da_arquitetura("saas");
        handle_write_page(v.clone(), arq.clone(), "---\ntype: docs\n---\nminha arquitetura\n".into()).unwrap();
        let de_novo = handle_criar_projeto(v.clone(), "SaaS".into(), Some("saas".into()), "hoje".into()).unwrap();
        assert!(de_novo.criadas.is_empty());
        assert_eq!(de_novo.existiam.len(), 3);
        assert!(handle_read_page(v, arq).unwrap().contains("minha arquitetura"));
    }

    #[test]
    fn lista_projetos_e_tarefas_numeradas() {
        let (_g, _r, v, caminhos) = montar(3);
        let ps = handle_listar_projetos(v.clone()).unwrap();
        assert_eq!(ps.len(), 1);
        assert_eq!(ps[0].slug, "saas");
        let ts = handle_tarefas_do_projeto(v, "saas".into()).unwrap();
        assert_eq!(ts.iter().map(|t| t.numero).collect::<Vec<_>>(), [1, 2, 3]);
        assert_eq!(ts[0].caminho, caminhos[0]);
    }

    #[test]
    fn a_pessoa_nao_marca_executando_a_mao() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        assert!(handle_mudar_etapa_tarefa(v, c[0].clone(), EtapaTarefa::Executando).is_err());
    }

    #[test]
    fn reserva_uma_vez_so() {
        let (_g, repo, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let primeira = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert_eq!(primeira.iniciar.len(), 1, "{:?}", primeira.avisos);
        let t = &primeira.iniciar[0];
        assert_eq!(t.cwd, repo.path().to_string_lossy().as_ref());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executando);
        let segunda = handle_tarefas_a_executar(v, params()).unwrap();
        assert!(segunda.iniciar.is_empty(), "a mesma tarefa saiu duas vezes");
    }

    #[test]
    fn a_tarefa_reservada_diz_o_executor_do_projeto() {
        let (_g, _r, v, c) = montar(2);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        assert_eq!((t.agente.as_deref(), t.papel), (None, projeto::Papel::Executor), "sem config, o padrão");
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().executor = Some("aider + Ollama (local)".into());
        cfg.get_mut("saas").unwrap().concorrencia = 2;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        pedir_execucao(&v, &c[1]);
        let t = handle_tarefas_a_executar(v, params()).unwrap().iniciar.remove(0);
        assert_eq!(t.agente.as_deref(), Some("aider + Ollama (local)"));
    }

    #[test]
    fn a_pergunta_fica_na_conversa_antes_do_agente() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        let conversa = handle_read_page(v, t.conversa.clone()).unwrap();
        let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&conversa);
        assert!(fm.contains("type: conversa"));
        assert!(fm.contains(&format!("tarefa: {}", c[0])), "{fm}");
        let msgs = conversa::parse(corpo);
        assert_eq!(msgs.len(), 1);
        assert!(msgs[0].texto.contains(projeto::MARCA_DE_RESULTADO));
        // O prompt leva a tarefa, o projeto e o índice.
        assert!(t.prompt.contains(&c[0]));
        assert!(t.prompt.contains("projeto.md"));
        assert!(t.prompt.contains("(índice)"));
    }

    #[test]
    fn concorrencia_conta_o_que_ja_esta_executando() {
        let (_g, _r, v, c) = montar(2);
        pedir_execucao(&v, &c[0]);
        assert_eq!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.len(), 1);
        pedir_execucao(&v, &c[1]);
        // A primeira ainda roda: a vaga única está ocupada, por quem quer
        // que a tenha pegado.
        assert!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.is_empty());
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
            .unwrap();
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn pausada_nao_perde_o_pedido() {
        // O defeito que derrubou a memória por borda (ciclo 457): pedido
        // feito com a fila pausada tem que rodar quando ela voltar.
        let (_g, _r, v, c) = montar(1);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().pausado = true;
        handle_gravar_projetos_cfg(v.clone(), cfg.clone()).unwrap();
        pedir_execucao(&v, &c[0]);
        let r = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert!(r.iniciar.is_empty());
        assert!(r.avisos.iter().any(|a| a.contains("pausada")), "{:?}", r.avisos);
        cfg.get_mut("saas").unwrap().pausado = false;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn sem_repositorio_avisa_e_nao_reserva() {
        let (_g, _r, v, c) = montar(1);
        handle_gravar_projetos_cfg(v.clone(), ConfigProjetos::new()).unwrap();
        pedir_execucao(&v, &c[0]);
        let r = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert!(r.iniciar.is_empty());
        assert!(r.avisos.iter().any(|a| a.contains("sem repositório")), "{:?}", r.avisos);
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar, "não pode ficar presa em executando");
    }

    #[test]
    fn config_ilegivel_e_erro_e_nao_despausa() {
        let (_g, _r, v, c) = montar(1);
        std::fs::write(Path::new(&v).join(projeto::ARQUIVO_CONFIG), "{ quebrado").unwrap();
        pedir_execucao(&v, &c[0]);
        assert!(handle_tarefas_a_executar(v.clone(), params()).is_err());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar);
    }

    #[test]
    fn trava_ocupada_nao_reserva() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let outra = Trava::pegar(&v).unwrap();
        assert!(handle_tarefas_a_executar(v.clone(), params()).is_err());
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar);
        drop(outra);
        assert_eq!(handle_tarefas_a_executar(v, params()).unwrap().iniciar.len(), 1);
    }

    #[test]
    fn fechar_le_o_resultado() {
        let (_g, _r, v, c) = montar(3);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().concorrencia = 3;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        for x in &c {
            pedir_execucao(&v, x);
        }
        assert_eq!(handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.len(), 3);
        let fim = |i: usize, f: FimDaTarefa| handle_fechar_tarefa(v.clone(), c[i].clone(), f).unwrap();
        assert_eq!(fim(0, FimDaTarefa::Respondeu("pronto\nRESULTADO: concluida".into())), EtapaTarefa::Revisao);
        assert_eq!(fim(1, FimDaTarefa::Respondeu("acho que foi".into())), EtapaTarefa::Bloqueada);
        assert_eq!(fim(2, FimDaTarefa::Falhou("timeout".into())), EtapaTarefa::Bloqueada);
    }

    #[test]
    fn fechar_respeita_quem_mexeu_durante() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        handle_tarefas_a_executar(v.clone(), params()).unwrap();
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Bloqueada).unwrap();
        let ficou =
            handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
                .unwrap();
        assert_eq!(ficou, EtapaTarefa::Bloqueada);
    }

    #[test]
    fn a_segunda_execucao_continua_a_mesma_conversa() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESULTADO: concluida".into()))
            .unwrap();
        // revisao → executar: "refaça".
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Executar).unwrap();
        let t2 = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        assert_eq!(t.conversa, t2.conversa);
        let conversa = handle_read_page(v, t2.conversa).unwrap();
        let (_, corpo) = MarkdownCodec::split_frontmatter_text(&conversa);
        assert_eq!(conversa::parse(corpo).len(), 2);
    }

    #[test]
    fn nao_reserva_alem_das_vagas_do_frontend() {
        let (_g, _r, v, c) = montar(2);
        let mut cfg = handle_ler_projetos_cfg(v.clone()).unwrap();
        cfg.get_mut("saas").unwrap().concorrencia = 2;
        handle_gravar_projetos_cfg(v.clone(), cfg).unwrap();
        for x in &c {
            pedir_execucao(&v, x);
        }
        let zero = ParametrosDaFila { vagas: Some(0), ..params() };
        assert!(handle_tarefas_a_executar(v.clone(), zero).unwrap().iniciar.is_empty());
        let uma = ParametrosDaFila { vagas: Some(1), ..params() };
        assert_eq!(handle_tarefas_a_executar(v.clone(), uma).unwrap().iniciar.len(), 1);
        // A segunda continua em `executar`, esperando a vaga, e não presa
        // em `executando` sem processo.
        let etapas: Vec<EtapaTarefa> = c.iter().map(|x| etapa(&v, x)).collect();
        assert_eq!(etapas, [EtapaTarefa::Executando, EtapaTarefa::Executar]);
    }

    #[test]
    fn fechar_pela_conversa_acha_a_tarefa() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        let r = handle_fechar_tarefa_da_conversa(
            v.clone(),
            t.conversa,
            FimDaTarefa::Respondeu("RESULTADO: concluida".into()),
        )
        .unwrap();
        assert_eq!(r, Some(EtapaTarefa::Revisao));
        // Conversa comum: nada a fechar.
        let geral = projeto::caminho_da_conversa_geral("saas");
        assert_eq!(
            handle_fechar_tarefa_da_conversa(v, geral, FimDaTarefa::Interrompida).unwrap(),
            None
        );
    }

    #[test]
    fn com_sessao_o_historico_nao_se_repete() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v.clone(), params()).unwrap().iniciar.remove(0);
        handle_fechar_tarefa(v.clone(), c[0].clone(), FimDaTarefa::Respondeu("RESPOSTA-UM\nRESULTADO: concluida".into()))
            .unwrap();
        // O executor grava a resposta e a sessão; aqui, à mão.
        let atual = handle_read_page(v.clone(), t.conversa.clone()).unwrap();
        let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&atual);
        let msg = Mensagem { autor: Autor::Agente, quando: AGORA.into(), texto: "RESPOSTA-UM".into() };
        let com_resposta = format!("{fm}\n{}", conversa::append(corpo, &msg));
        let com_sessao = conversa::reescrever_sessao(&com_resposta, Some("sess-1"));
        handle_write_page(v.clone(), t.conversa.clone(), com_sessao).unwrap();
        handle_mudar_etapa_tarefa(v.clone(), c[0].clone(), EtapaTarefa::Executar).unwrap();
        let t2 = handle_tarefas_a_executar(v, params()).unwrap().iniciar.remove(0);
        assert!(!t2.prompt.contains("RESPOSTA-UM"), "o histórico foi junto com a sessão aberta");
    }

    #[test]
    fn estado_do_projeto_so_le() {
        let (_g, _r, v, c) = montar(1);
        handle_gravar_projetos_cfg(v.clone(), ConfigProjetos::new()).unwrap();
        pedir_execucao(&v, &c[0]);
        let e = handle_estado_do_projeto(v.clone(), "saas".into()).unwrap();
        assert_eq!(e.tarefas.len(), 1);
        assert!(e.avisos.iter().any(|a| a.contains("sem repositório")), "{:?}", e.avisos);
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar, "a tela não pode reservar");
        assert!(!Path::new(&v).join(".anotadinho/projetos.lock").exists());
    }

    #[test]
    fn gravar_config_de_um_nao_apaga_o_outro() {
        let (_g, _r, v, _) = montar(0);
        handle_gravar_config_do_projeto(v.clone(), "outro".into(), ConfigProjeto { pausado: true, ..Default::default() })
            .unwrap();
        let todas = handle_ler_projetos_cfg(v).unwrap();
        assert!(todas["saas"].repositorio.is_some());
        assert!(todas["outro"].pausado);
    }

    #[test]
    fn virar_tarefas_numera_em_sequencia_num_lote() {
        let (_g, _r, v, _) = montar(2);
        let resposta = "intro\n## Tarefa: Banco\n### Objetivo\nx\n## Tarefa: Login\n### Objetivo\ny\n";
        let ids = handle_propor_tarefas(v.clone(), "saas".into(), resposta.into(), AGORA.into()).unwrap();
        assert_eq!(ids.len(), 2);
        let props = crate::handle_listar_propostas(v.clone()).unwrap();
        let mut alvos: Vec<&str> = props.iter().map(|p| p.alvo.as_str()).collect();
        alvos.sort();
        assert_eq!(
            alvos,
            ["pages/projetos/saas/tarefas/003-banco.md", "pages/projetos/saas/tarefas/004-login.md"]
        );
        assert!(props.iter().all(|p| p.lote == props[0].lote && p.lote.is_some()));
        // Nada gravado antes de aplicar.
        assert!(!Path::new(&v).join("pages/projetos/saas/tarefas/003-banco.md").exists());
        let conteudo = &props.iter().find(|p| p.alvo.ends_with("003-banco.md")).unwrap().conteudo;
        assert_eq!(Tarefa::ler(&props[0].alvo, conteudo).unwrap().etapa, EtapaTarefa::Ideia);
    }

    #[test]
    fn virar_tarefas_sem_tarefa_e_erro() {
        let (_g, _r, v, _) = montar(0);
        assert!(handle_propor_tarefas(v, "saas".into(), "só conversa".into(), AGORA.into()).is_err());
    }

    #[test]
    fn atualizar_arquitetura_acrescenta() {
        let (_g, _r, v, _) = montar(0);
        handle_propor_arquitetura(v.clone(), "saas".into(), "Postgres e fila".into(), AGORA.into()).unwrap();
        let p = crate::handle_listar_propostas(v).unwrap().remove(0);
        assert!(p.conteudo.contains("## Componentes"), "apagou o que havia");
        assert!(p.conteudo.trim_end().ends_with("Postgres e fila"));
    }

    #[test]
    fn criar_tarefa_grava_direto_e_numera() {
        let (_g, _r, v, _) = montar(1);
        let c = handle_criar_tarefa(v.clone(), "saas".into(), "Outra".into(), "2026-10-21".into()).unwrap();
        assert_eq!(c, "pages/projetos/saas/tarefas/002-outra.md");
        assert!(Path::new(&v).join(&c).exists());
    }

    #[test]
    fn conversa_da_tarefa_e_a_mesma_da_execucao() {
        let (_g, _r, v, c) = montar(1);
        let conversa = handle_conversa_da_tarefa(v.clone(), c[0].clone()).unwrap();
        let texto = handle_read_page(v.clone(), conversa.clone()).unwrap();
        let (fm, _) = MarkdownCodec::split_frontmatter_text(&texto);
        assert_eq!(
            conversa::contexto_do_frontmatter(fm),
            [c[0].clone(), "pages/projetos/saas/projeto.md".to_string(), "pages/projetos/saas/arquitetura.md".to_string()]
        );
        // Chamar de novo não recria; a execução usa a mesma.
        assert_eq!(handle_conversa_da_tarefa(v.clone(), c[0].clone()).unwrap(), conversa);
        pedir_execucao(&v, &c[0]);
        let t = handle_tarefas_a_executar(v, params()).unwrap().iniciar.remove(0);
        assert_eq!(t.conversa, conversa);
    }

    #[test]
    fn freio_diz_quem_esta_esperando() {
        let (_g, _r, v, c) = montar(1);
        pedir_execucao(&v, &c[0]);
        let guardas = anotadinho_core::guardas::Guardas { silencio_de: "00:00".into(), silencio_ate: "23:59".into(), ..Default::default() };
        crate::handle_gravar_guardas(v.clone(), guardas).unwrap();
        let r = handle_tarefas_a_executar(v.clone(), params()).unwrap();
        assert!(r.iniciar.is_empty());
        assert!(r.avisos.iter().any(|a| a.contains("freio") && a.contains("esperando: saas")), "{:?}", r.avisos);
        assert_eq!(etapa(&v, &c[0]), EtapaTarefa::Executar, "o freio não pode reservar");
    }
}
