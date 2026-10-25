//! O laço de revisão pelo agente (ciclo 522): reservar a revisão, fechar
//! com o veredito.
//!
//! A tarefa fica em `executando` do começo ao fim do laço, e o
//! `revisao_ia` diz o ponto (`core::revisor`). Cada passo é um trabalho que
//! a MESMA fila devolve (`handle_tarefas_a_executar`), e cada fim passa pelo
//! MESMO `handle_fechar_tarefa_da_conversa`: a TUI e a janela rodam o laço
//! sem saber que ele existe.

use anotadinho_core::conversa::{self, Autor, Mensagem};
use anotadinho_core::projeto::{self, ConfigProjeto, EtapaTarefa, Papel, Projeto, Quem, Tarefa};
use anotadinho_core::revisor::{self, estado, Veredito};
use anotadinho_core::MarkdownCodec;

use crate::projeto::{contextos_da_execucao, FimDaTarefa, ParametrosDaFila, TrabalhoDeTarefa};
use crate::{handle_read_page, handle_write_page, repositorio};

/// A tarefa está parada no laço, esperando a fila dar a vez: a revisão
/// (`pendente`) ou o executor refazer (`ajustes`).
pub(crate) fn espera_a_fila(t: &Tarefa) -> bool {
    t.etapa == EtapaTarefa::Executando && matches!(t.revisao_ia.as_deref(), Some(estado::PENDENTE | estado::AJUSTES))
}

/// Reserva a revisão de uma tarefa (`pendente` → `rodando`) e devolve o
/// trabalho do revisor.
pub(crate) fn reservar_revisao(
    vault_path: &str,
    p: &Projeto,
    lida: &Tarefa,
    todas: &[Tarefa],
    repo: &str,
    cfg: &ConfigProjeto,
    params: &ParametrosDaFila,
) -> Result<Option<TrabalhoDeTarefa>, String> {
    // Releitura: a listagem é de antes da trava.
    let texto = handle_read_page(vault_path.to_string(), lida.caminho.clone())?;
    let Some(tarefa) = Tarefa::ler(&lida.caminho, &texto) else { return Ok(None) };
    if tarefa.etapa != EtapaTarefa::Executando || tarefa.revisao_ia.as_deref() != Some(estado::PENDENTE) {
        return Ok(None);
    }
    // O revisor saiu da config enquanto a revisão esperava: vai pra pessoa,
    // em vez de ficar em `pendente` pra sempre.
    let Some(nome) = cfg.nome_do_revisor() else {
        fechar_no_humano(vault_path, &tarefa, &texto, estado::FALHOU, "a revisão automática foi desligada antes de rodar", None)?;
        return Ok(None);
    };

    let mudancas = repositorio::mudancas_desde(repo, tarefa.base_git.as_deref());
    let arquivos: Vec<String> = mudancas.arquivos.iter().map(|a| format!("{}  {}", a.estado, a.caminho)).collect();
    let material = revisor::Material {
        rodada: tarefa.rodada + 1,
        teto: cfg.rodadas_de_revisao,
        verificacao: cfg.comando_de_verificacao(),
        arquivos,
        diff: &mudancas.diff,
        aviso: mudancas.aviso.as_deref(),
    };
    let pergunta = revisor::pergunta_de_revisar(&tarefa, &material);

    let conversa_path = projeto::caminho_da_conversa_da_revisao(&p.slug, tarefa.numero);
    let base = handle_read_page(vault_path.to_string(), conversa_path.clone()).unwrap_or_else(|_| pagina_da_revisao(p, &tarefa));
    let (fm, corpo) = MarkdownCodec::split_frontmatter_text(&base);
    // O histórico das revisões anteriores vai junto: o revisor vê o que já
    // pediu, e não repete nem contradiz a rodada de antes.
    let historico = conversa::parse(corpo);
    let minha = Mensagem { autor: Autor::Voce, quando: params.agora.clone(), texto: pergunta.clone() };
    let corpo_novo = conversa::append(corpo, &minha);
    let gravado = if fm.is_empty() { corpo_novo } else { format!("{fm}\n{corpo_novo}") };

    let contextos = contextos_da_execucao(vault_path, p, &tarefa, &texto, todas);
    let envio = anotadinho_core::envio::montar(&contextos, &historico, &pergunta, params.limite_historico, params.teto, vault_path);

    handle_write_page(vault_path.to_string(), conversa_path.clone(), gravado)?;
    let reservado = revisor::com_estado(&texto, Some(estado::RODANDO), Some(tarefa.rodada))?;
    handle_write_page(vault_path.to_string(), tarefa.caminho.clone(), reservado)?;

    Ok(Some(TrabalhoDeTarefa {
        projeto: p.slug.clone(),
        tarefa: tarefa.caminho.clone(),
        conversa: conversa_path,
        prompt: envio.prompt,
        cwd: repo.to_string(),
        agente: Some(nome.to_string()),
        papel: Papel::Revisor,
    }))
}

/// A página da conversa do revisor. `revisao_de:` é o que o fechamento
/// lê pra saber que a resposta é um veredito, e não uma execução.
fn pagina_da_revisao(p: &Projeto, tarefa: &Tarefa) -> String {
    let md = conversa::montar_pagina(
        &format!("Revisão {:03} — {}", tarefa.numero, tarefa.titulo),
        Some(&tarefa.caminho),
        &[tarefa.caminho.clone(), p.caminho.clone()],
    );
    let md = MarkdownCodec::set_frontmatter_field(&md, "projeto", &p.slug).unwrap_or(md);
    MarkdownCodec::set_frontmatter_field(&md, "revisao_de", &tarefa.caminho).unwrap_or(md)
}

/// Fecha a revisão de uma tarefa com o que o revisor respondeu.
///
/// `quando` vai no comentário (o ipc não tem relógio: vem da mensagem que
/// o executor gravou na conversa). Devolve a etapa em que a tarefa ficou.
pub(crate) fn fechar_revisao(vault_path: &str, caminho: &str, fim: FimDaTarefa, quando: &str) -> Result<EtapaTarefa, String> {
    let texto = handle_read_page(vault_path.to_string(), caminho.to_string())?;
    let tarefa = Tarefa::ler(caminho, &texto).ok_or_else(|| format!("{caminho} não é uma tarefa"))?;
    // A pessoa mexeu no meio (bloqueou, mandou de novo): vale a decisão dela.
    if tarefa.etapa != EtapaTarefa::Executando || tarefa.revisao_ia.as_deref() != Some(estado::RODANDO) {
        return Ok(tarefa.etapa);
    }
    let cfg = crate::handle_ler_projetos_cfg(vault_path.to_string())?.remove(&tarefa.projeto).unwrap_or_default();
    let rodada = tarefa.rodada + 1;
    let teto = cfg.rodadas_de_revisao.max(1);
    let quem = cfg.nome_do_revisor().unwrap_or("revisor");
    let cabeca = format!("Revisão {rodada}/{teto} do agente ({quem})");
    // Os problemas que o revisor anotou viram proposta pra base (ciclo
    // 523), seja qual for o veredito: um erro corrigido na rodada seguinte
    // ainda é um erro que o executor tende a repetir.
    let nota = match &fim {
        FimDaTarefa::Respondeu(r) => propor_problemas(vault_path, &tarefa, r, quando),
        _ => None,
    };
    let comentario = |t: &str| {
        let t = match &nota {
            Some(n) => format!("{t}\n{n}"),
            None => t.to_string(),
        };
        anotadinho_core::detalhes::acrescentar_comentario(&texto, quando, &t)
    };

    match fim {
        FimDaTarefa::Respondeu(resposta) => match revisor::ler_veredito(&resposta) {
            Veredito::Aprovada => {
                let c = comentario(&format!("{cabeca}: aprovada."))?;
                fechar_no_humano(vault_path, &tarefa, &c, estado::APROVADA, "", Some(tarefa.rodada))
            }
            Veredito::Ajustes(ajustes) if rodada < teto => {
                // Volta pro executor SEM sair de `executando`: a fila dá a
                // vez a ele na próxima checagem (`espera_a_fila`).
                let c = comentario(&format!("{cabeca}: ajustes pedidos — o executor refaz com isto:\n{ajustes}"))?;
                let novo = revisor::com_estado(&c, Some(estado::AJUSTES), Some(rodada))?;
                handle_write_page(vault_path.to_string(), caminho.to_string(), novo)?;
                Ok(EtapaTarefa::Executando)
            }
            Veredito::Ajustes(ajustes) => {
                let c = comentario(&format!(
                    "{cabeca}: ainda pede ajustes, e as {teto} rodadas acabaram — a decisão é sua:\n{ajustes}"
                ))?;
                fechar_no_humano(vault_path, &tarefa, &c, estado::ESGOTOU, "", Some(rodada))
            }
            Veredito::SemVeredito => {
                let c = comentario(&format!(
                    "{cabeca}: a resposta não terminou com `{}` — leia a conversa da revisão e decida.",
                    revisor::MARCA_DE_VEREDITO
                ))?;
                fechar_no_humano(vault_path, &tarefa, &c, estado::FALHOU, "", Some(tarefa.rodada))
            }
        },
        // O trabalho passou na verificação: um revisor quebrado não pode
        // jogá-lo fora. Vai pra pessoa, com o motivo.
        FimDaTarefa::Falhou(e) => {
            let motivo = e.lines().next().unwrap_or_default().to_string();
            let c = comentario(&format!("{cabeca} não rodou: {motivo}. O trabalho passou na verificação; a decisão é sua."))?;
            fechar_no_humano(vault_path, &tarefa, &c, estado::FALHOU, "", Some(tarefa.rodada))
        }
        FimDaTarefa::Interrompida => {
            let c = comentario(&format!("{cabeca} foi interrompida. A decisão é sua."))?;
            fechar_no_humano(vault_path, &tarefa, &c, estado::FALHOU, "", Some(tarefa.rodada))
        }
    }
}

/// Propõe os problemas que o revisor anotou na base do projeto (ciclo
/// 523). Devolve o que dizer no comentário: quantos foram propostos, ou
/// por que não deu — nunca derruba o fechamento.
fn propor_problemas(vault_path: &str, tarefa: &Tarefa, resposta: &str, quando: &str) -> Option<String> {
    use anotadinho_core::problemas;
    let novos = problemas::ler(resposta);
    if novos.is_empty() {
        return None;
    }
    let alvo = projeto::caminho_dos_problemas(&tarefa.projeto);
    // A proposta PENDENTE pra mesma página é a base desta: aplicar a nova
    // por cima de uma página que ainda não tem a anterior apagaria o que
    // a anterior trouxe.
    let pendente = crate::handle_listar_propostas(vault_path.to_string())
        .ok()
        .and_then(|ps| ps.into_iter().filter(|p| p.alvo == alvo).max_by(|a, b| a.quando.cmp(&b.quando).then(a.id.cmp(&b.id))))
        .map(|p| p.conteudo);
    let base = pendente.or_else(|| handle_read_page(vault_path.to_string(), alvo.clone()).ok()).unwrap_or_else(|| {
        let caminho = projeto::caminho_do_projeto(&tarefa.projeto);
        let titulo = handle_read_page(vault_path.to_string(), caminho.clone())
            .ok()
            .and_then(|t| Projeto::ler(&caminho, &t))
            .map(|p| p.titulo)
            .unwrap_or_else(|| tarefa.projeto.clone());
        problemas::pagina_nova(&tarefa.projeto, &titulo)
    });
    let novo = problemas::acrescentar(&base, &novos, &tarefa.caminho, quando)?;
    let n = problemas::titulos(&novo).len().saturating_sub(problemas::titulos(&base).len());
    let p = crate::projeto::proposta_nova(
        vault_path,
        "revisor",
        alvo.clone(),
        novo,
        format!("problemas vistos na revisão de {}", tarefa.caminho),
        quando.to_string(),
        None,
    );
    Some(match crate::handle_propor(vault_path.to_string(), p) {
        Ok(_) => format!("Propôs {n} problema(s) pra base do projeto ({alvo}): aprove na fila de propostas."),
        Err(e) => format!("Não consegui propor os problemas pra base ({alvo}): {e}"),
    })
}

/// Fim do laço: a tarefa vai pra `revisao`, a da pessoa. `aviso`, quando
/// há, entra como comentário (o caminho sem resposta do revisor).
fn fechar_no_humano(
    vault_path: &str,
    tarefa: &Tarefa,
    texto: &str,
    fim: &str,
    aviso: &str,
    rodada: Option<u32>,
) -> Result<EtapaTarefa, String> {
    let texto = if aviso.is_empty() {
        texto.to_string()
    } else {
        // Sem a hora da conversa (não houve resposta): fica sem `quando`
        // legível, mas fica — calar o motivo é pior.
        anotadinho_core::detalhes::acrescentar_comentario(texto, "—", aviso)?
    };
    let com = revisor::com_estado(&texto, Some(fim), rodada)?;
    let novo = projeto::mudar_etapa(&com, EtapaTarefa::Revisao, Quem::Sistema)?;
    handle_write_page(vault_path.to_string(), tarefa.caminho.clone(), novo)?;
    Ok(EtapaTarefa::Revisao)
}

/// A hora da última mensagem da conversa: é a da resposta que o executor
/// acabou de gravar.
pub(crate) fn hora_da_ultima_mensagem(texto_da_conversa: &str) -> Option<String> {
    let (_, corpo) = MarkdownCodec::split_frontmatter_text(texto_da_conversa);
    conversa::parse(corpo).last().map(|m| m.quando.clone()).filter(|q| !q.trim().is_empty())
}
