//! O conteúdo com que um vault novo nasce (ciclo 233).
//!
//! Antes deste ciclo não existia "criar vault": só apontar para uma
//! pasta. Apontar para uma pasta vazia funcionava — e abria um app sem
//! sidebar, sem template, sem prompt e sem nenhum sinal do que fazer.
//! Pior: as pastas com significado (`pages/specs`, `pages/conversas`,
//! `pages/prompts-default`) são esperadas pelo código e nunca eram
//! criadas, então o fluxo inteiro de spec → proposta → execução ficava
//! mudo até alguém adivinhar os nomes exatos.
//!
//! O conteúdo mora em `crates/core/seeds/` como markdown de verdade,
//! embutido no binário por `include_str!`. Assim dá pra editar a semente
//! olhando para ela, e o app e o `anotadinho-cli init` semeiam igual.

/// Uma pasta que precisa existir mesmo vazia.
pub const PASTAS: &[&str] = &[
    "pages",
    "pages/specs",
    "pages/propostas",
    "pages/execucoes",
    "pages/conversas",
    "pages/padroes",
    "pages/prompts-default",
    "pages/projetos",
    "pages/guias",
    "journals",
    "templates",
    "assets",
];

/// Um arquivo da semente: caminho relativo ao vault, e conteúdo.
pub struct Arquivo {
    /// Caminho relativo, sempre com `/`.
    pub caminho: &'static str,
    /// Conteúdo do arquivo.
    pub conteudo: &'static str,
}

/// A página inicial: o painel (ciclo 479). O guia que ela era mora em
/// [`GUIA`].
pub const PAGINA_INICIAL: &str = "pages/inicio.md";

/// O guia do vault: os tipos de página, os embeds e as pastas.
pub const GUIA: &str = "pages/guias/guia-do-vault.md";

/// Troca as datas relativas da semente pela data do dia (ciclo 479).
///
/// `@DIA@`, `@DIA+7@` e `@DIA-3@` viram `AAAA-MM-DD` a partir de `hoje`.
/// O projeto de exemplo tem sprint ativa e prazos; com data fixa, um
/// vault criado daqui a um ano nasceria com a sprint "ativa" no passado
/// e toda tarefa vencida.
pub fn no_dia(conteudo: &str, hoje: &str) -> Result<String, String> {
    if !crate::detalhes::data_valida(hoje) {
        return Err(format!("\"{hoje}\" não é uma data AAAA-MM-DD"));
    }
    let mut fora = String::with_capacity(conteudo.len());
    let mut resto = conteudo;
    while let Some(i) = resto.find("@DIA") {
        fora.push_str(&resto[..i]);
        let depois = &resto[i + 4..];
        let fim = depois.find('@').ok_or_else(|| "marcador @DIA sem o @ do fim".to_string())?;
        let delta: i64 = match depois[..fim].trim() {
            "" => 0,
            d => d.parse().map_err(|_| format!("marcador @DIA{d}@ inválido"))?,
        };
        let data = crate::date_util::add_days(hoje, delta).ok_or_else(|| format!("não somei {delta} dias a {hoje}"))?;
        fora.push_str(&data);
        resto = &depois[fim + 1..];
    }
    fora.push_str(resto);
    Ok(fora)
}

/// Tudo que um vault novo ganha.
pub fn arquivos() -> Vec<Arquivo> {
    let mut lista = vec![Arquivo {
        caminho: PAGINA_INICIAL,
        conteudo: include_str!("../seeds/inicio.md"),
    }];

    // Lista explícita em vez de macro: `include_str!` precisa de
    // literal em tempo de compilação, e a macro que montaria os
    // caminhos ficaria mais difícil de ler do que as linhas.
    lista.extend([
        Arquivo { caminho: "templates/spec.md", conteudo: include_str!("../seeds/templates/spec.md") },
        Arquivo { caminho: "templates/proposta.md", conteudo: include_str!("../seeds/templates/proposta.md") },
        Arquivo { caminho: "templates/decisao.md", conteudo: include_str!("../seeds/templates/decisao.md") },
        Arquivo { caminho: "templates/nota-de-reuniao.md", conteudo: include_str!("../seeds/templates/nota-de-reuniao.md") },
        Arquivo { caminho: "templates/padrao-codigo.md", conteudo: include_str!("../seeds/templates/padrao-codigo.md") },
        Arquivo { caminho: "templates/sessao-de-trabalho.md", conteudo: include_str!("../seeds/templates/sessao-de-trabalho.md") },

        Arquivo { caminho: "pages/padroes/nomenclatura.md", conteudo: include_str!("../seeds/padroes/nomenclatura.md") },
        Arquivo { caminho: "pages/padroes/estado-em-closure.md", conteudo: include_str!("../seeds/padroes/estado-em-closure.md") },
        Arquivo { caminho: "pages/padroes/escrita-no-vault.md", conteudo: include_str!("../seeds/padroes/escrita-no-vault.md") },
        Arquivo { caminho: "pages/padroes/editor-e-dom.md", conteudo: include_str!("../seeds/padroes/editor-e-dom.md") },
        Arquivo { caminho: "pages/padroes/agente-e-execucao.md", conteudo: include_str!("../seeds/padroes/agente-e-execucao.md") },
        Arquivo { caminho: "pages/padroes/spec-proposta-execucao.md", conteudo: include_str!("../seeds/padroes/spec-proposta-execucao.md") },
        Arquivo { caminho: "pages/padroes/validacao.md", conteudo: include_str!("../seeds/padroes/validacao.md") },
        Arquivo { caminho: "pages/padroes/fronteira-do-sistema.md", conteudo: include_str!("../seeds/padroes/fronteira-do-sistema.md") },

        Arquivo { caminho: "pages/prompts-default/investigar-comportamento-errado.md", conteudo: include_str!("../seeds/prompts-default/investigar-comportamento-errado.md") },
        Arquivo { caminho: "pages/prompts-default/revisar-spec.md", conteudo: include_str!("../seeds/prompts-default/revisar-spec.md") },
        Arquivo { caminho: "pages/prompts-default/escrever-cenario-de-harness.md", conteudo: include_str!("../seeds/prompts-default/escrever-cenario-de-harness.md") },
        Arquivo { caminho: "pages/prompts-default/planejar-a-partir-de-uma-spec.md", conteudo: include_str!("../seeds/prompts-default/planejar-a-partir-de-uma-spec.md") },
        Arquivo { caminho: "pages/prompts-default/entender-um-trecho-do-codigo.md", conteudo: include_str!("../seeds/prompts-default/entender-um-trecho-do-codigo.md") },
        Arquivo { caminho: "pages/prompts-default/quebrar-em-tarefas.md", conteudo: include_str!("../seeds/prompts-default/quebrar-em-tarefas.md") },
        Arquivo { caminho: "pages/prompts-default/montar-a-base-do-projeto.md", conteudo: include_str!("../seeds/prompts-default/montar-a-base-do-projeto.md") },

        // Os guias e o projeto de exemplo (ciclo 479).
        Arquivo { caminho: GUIA, conteudo: include_str!("../seeds/guias/guia-do-vault.md") },
        Arquivo { caminho: "pages/guias/como-usar-projetos.md", conteudo: include_str!("../seeds/guias/como-usar-projetos.md") },
        Arquivo { caminho: "pages/guias/como-usar-o-modo-agentico.md", conteudo: include_str!("../seeds/guias/como-usar-o-modo-agentico.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/projeto.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/projeto.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/arquitetura.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/arquitetura.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/conversas/geral.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/conversas/geral.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/sprints/01-o-basico.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/sprints/01-o-basico.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/sprints/02-leitura.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/sprints/02-leitura.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/tarefas/001-esqueleto-do-app.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/tarefas/001-esqueleto-do-app.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/tarefas/002-cadastrar-um-livro.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/tarefas/002-cadastrar-um-livro.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/tarefas/003-listar-os-livros.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/tarefas/003-listar-os-livros.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/tarefas/004-marcar-como-lido.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/tarefas/004-marcar-como-lido.md") },
        Arquivo { caminho: "pages/projetos/meu-primeiro-projeto/tarefas/005-exportar-a-lista.md", conteudo: include_str!("../seeds/projetos/meu-primeiro-projeto/tarefas/005-exportar-a-lista.md") },
    ]);

    lista
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_semente_cobre_as_pastas_que_o_codigo_espera() {
        // Estas são hardcoded em `fluxo::Artefato::pasta` e em
        // `prompt_padrao::descobrir`. Se uma delas não nascer, a função
        // correspondente fica muda sem dar erro nenhum.
        for esperada in [
            "pages/specs",
            "pages/propostas",
            "pages/execucoes",
            "pages/conversas",
            "pages/prompts-default",
            "pages/projetos",
            "templates",
        ] {
            assert!(PASTAS.contains(&esperada), "faltou semear {esperada}");
        }
    }

    #[test]
    fn todo_arquivo_da_semente_tem_frontmatter_e_titulo() {
        for a in arquivos() {
            assert!(
                a.conteudo.starts_with("---\n"),
                "{} não começa com frontmatter",
                a.caminho
            );
            assert!(
                a.conteudo.contains("title:"),
                "{} não tem título",
                a.caminho
            );
        }
    }

    #[test]
    fn os_prompts_da_semente_sao_descobriveis() {
        // `prompt_padrao::descobrir` exige os DOIS: `type: prompt` e o
        // prefixo da pasta. Um prompt semeado sem o tipo não apareceria
        // no seletor, e ninguém perceberia.
        for a in arquivos() {
            if a.caminho.starts_with("pages/prompts-default/") {
                assert!(
                    a.conteudo.contains("type: prompt"),
                    "{} não seria descoberto pelo seletor",
                    a.caminho
                );
            }
        }
    }

    const HOJE: &str = "2026-10-23";

    fn no_dia_de_hoje() -> Vec<(&'static str, String)> {
        arquivos().into_iter().map(|a| (a.caminho, no_dia(a.conteudo, HOJE).unwrap())).collect()
    }

    #[test]
    fn as_datas_relativas_viram_datas_do_dia() {
        assert_eq!(no_dia("a @DIA@ b @DIA+11@ c @DIA-3@", HOJE).unwrap(), "a 2026-10-23 b 2026-11-03 c 2026-10-20");
        assert!(no_dia("x", "ontem").is_err());
        assert!(no_dia("@DIA+x@", HOJE).is_err());
        for (c, texto) in no_dia_de_hoje() {
            assert!(!texto.contains("@DIA"), "{c} ficou com marcador");
        }
    }

    #[test]
    fn o_projeto_de_exemplo_e_lido_pelo_app() {
        use crate::projeto::{EtapaTarefa, Projeto, Tarefa};
        use crate::sprint::{self, EstadoSprint, Filtro, Sprint};
        let pags = no_dia_de_hoje();
        let do_exemplo = |pasta: &str| {
            let prefixo = format!("pages/projetos/meu-primeiro-projeto/{pasta}");
            pags.iter().filter(move |(c, _)| c.starts_with(&prefixo))
        };
        let projeto = do_exemplo("projeto.md").find_map(|(c, t)| Projeto::ler(c, t)).expect("o projeto");
        assert_eq!(projeto.slug, "meu-primeiro-projeto");
        let tarefas: Vec<Tarefa> = do_exemplo("tarefas/").map(|(c, t)| Tarefa::ler(c, t).unwrap_or_else(|| panic!("{c}"))).collect();
        let sprints: Vec<Sprint> = do_exemplo("sprints/").map(|(c, t)| Sprint::ler(c, t).unwrap_or_else(|| panic!("{c}"))).collect();
        assert_eq!(tarefas.len(), 5);
        // O nome do arquivo é o que o app daria: criar a 6ª pelo app não
        // pode colidir nem pular número.
        for t in &tarefas {
            assert_eq!(t.caminho, crate::projeto::caminho_da_tarefa(&t.projeto, t.numero, &t.titulo), "nome fora do padrão");
        }
        for s in &sprints {
            assert_eq!(s.caminho, sprint::caminho_da_sprint(&s.projeto, s.numero, &s.nome), "nome fora do padrão");
        }
        // Uma ativa, que contém hoje; uma planejada.
        let ativa = sprint::ativa(&sprints).expect("uma sprint ativa");
        assert!(ativa.inicio.as_deref().unwrap() <= HOJE && HOJE <= ativa.fim.as_deref().unwrap());
        assert_eq!(sprints.iter().filter(|s| s.estado == EstadoSprint::Planejada).count(), 1);
        // O quadro abre na ativa com as três dela, e há backlog.
        assert_eq!(sprint::tarefas_do_quadro(&tarefas, &sprints, Filtro::SprintAtiva).len(), 3);
        assert_eq!(sprint::tarefas_do_quadro(&tarefas, &sprints, Filtro::Backlog).len(), 1);
        assert!(sprint::avisos(&sprints, &tarefas).is_empty());
        // Os detalhes do kanban preenchidos, e nada vencido no dia.
        let cadastro = tarefas.iter().find(|t| t.numero == 2).unwrap();
        assert_eq!(cadastro.progresso(), (1, 3));
        assert_eq!(cadastro.comentarios.len(), 1);
        assert_eq!(cadastro.tags, ["livros"]);
        assert!(tarefas.iter().all(|t| !t.vencida(HOJE)), "o exemplo nasce sem nada vencido");
        // Etapas variadas, e nenhuma vai pra fila sozinha.
        let etapas: std::collections::BTreeSet<_> = tarefas.iter().map(|t| t.etapa.slug()).collect();
        assert!(etapas.len() >= 4, "{etapas:?}");
        assert!(tarefas.iter().all(|t| t.etapa != EtapaTarefa::Executar && t.etapa != EtapaTarefa::Executando));
    }

    #[test]
    fn os_embeds_da_semente_sao_validos() {
        for (c, texto) in no_dia_de_hoje() {
            let (_, corpo) = crate::MarkdownCodec::split_frontmatter_text(&texto);
            // Sem vault: o que depende do disco (asset existe?) não é
            // conferido aqui; a forma de cada embed é.
            let ctx = crate::embed::ValidationCtx::default();
            for seg in crate::embed::segment(corpo) {
                if let crate::embed::DocSegment::Embed(data) = seg {
                    let erros: Vec<_> = data.validate(&ctx).into_iter().filter(|p| p.severidade == crate::embed::Severidade::Erro).collect();
                    assert!(erros.is_empty(), "{c}: {erros:?}");
                }
            }
        }
    }

    #[test]
    fn as_consultas_da_semente_sao_yaml_de_verdade() {
        // O leitor de embed é TOLERANTE: YAML quebrado vira consulta vazia,
        // que traz o vault inteiro, e o `validate` não reclama. Visto ao
        // quebrar o painel de propósito (ciclo 479): o teste de embeds
        // acima passava. Aqui o corpo de cada consulta é lido estrito.
        let mut n = 0;
        for (c, texto) in no_dia_de_hoje() {
            for bloco in texto.split("{{ type: \"query\" }}\n").skip(1) {
                let corpo = bloco.split("{{ /query }}").next().unwrap();
                serde_yaml::from_str::<crate::query::Query>(corpo).unwrap_or_else(|e| panic!("{c}: consulta inválida: {e}\n{corpo}"));
                n += 1;
            }
        }
        assert!(n >= 7, "o painel tem sete consultas; achei {n}");
    }

    #[test]
    fn quebrar_em_tarefas_pede_o_formato_que_o_core_le() {
        // O prompt e `projeto::separar_tarefas` são as duas pontas do
        // mesmo combinado. Mudar o cabeçalho num lado só faria "virar
        // tarefas" achar zero tarefa numa resposta perfeita.
        let prompt = arquivos()
            .into_iter()
            .find(|a| a.caminho == "pages/prompts-default/quebrar-em-tarefas.md")
            .expect("o prompt de quebrar em tarefas precisa ser semeado");
        assert!(prompt.conteudo.contains(crate::projeto::CABECALHO_DE_TAREFA));
        let (_, corpo) = crate::MarkdownCodec::split_frontmatter_text(prompt.conteudo);
        let exemplo = corpo.split("```").nth(1).expect("o prompt traz um exemplo cercado");
        let tarefas = crate::projeto::separar_tarefas(exemplo);
        assert_eq!(tarefas.len(), 2, "o exemplo do prompt tem que ser lido pelo core");
    }

    #[test]
    fn o_guia_explica_os_tipos_de_pagina() {
        let guia = arquivos()
            .into_iter()
            .find(|a| a.caminho == GUIA)
            .expect("a semente precisa do guia");
        for tipo in ["conversa", "spec", "proposta", "execucao", "kanban", "prompt", "projeto", "tarefa", "sprint"] {
            assert!(
                guia.conteudo.contains(tipo),
                "o guia não cita o tipo {tipo}"
            );
        }
    }
}
