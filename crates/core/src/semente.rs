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

/// A página inicial, que é também o guia.
pub const PAGINA_INICIAL: &str = "pages/inicio.md";

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

    #[test]
    fn o_guia_explica_os_tipos_de_pagina() {
        let guia = arquivos()
            .into_iter()
            .find(|a| a.caminho == PAGINA_INICIAL)
            .expect("a semente precisa de página inicial");
        for tipo in ["conversa", "spec", "proposta", "execucao", "kanban", "prompt"] {
            assert!(
                guia.conteudo.contains(tipo),
                "o guia não cita o tipo {tipo}"
            );
        }
    }
}
