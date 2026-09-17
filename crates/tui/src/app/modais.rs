//! Os modais da TUI (ciclo 339): a barra de comandos, os seletores, a
//! confirmação, a entrada de texto, os atalhos e o editor de opções de uma
//! coluna de seleção — todos feitos dos [`componentes`](crate::componentes).
//!
//! A barra de comandos é a `CommandPalette` da janela: `:` (o modo de
//! comando do vim) ou `Ctrl+K` abrem; digitar filtra comandos e páginas;
//! `Enter` escolhe. Os comandos são os mesmos da janela, mais os que a TUI
//! precisa pra ser personalizada (tema, sidebar, agente).
//!
//! Ações que mexem em arquivo não acontecem aqui: viram [`Pedido`]s que o
//! `main` executa, como a gravação de edição.

use anotadinho_core::unidade::Caminho;
use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use ratatui::Frame;

use super::Estado;
use crate::componentes::{self, Campo, Item, Lista, Resposta};

/// Algo que só o `main` (que tem o vault) pode fazer.
#[derive(Debug, Clone, PartialEq)]
pub enum Pedido {
    /// Abrir esta página.
    AbrirPagina(String),
    /// Ler do vault os dados da tela de tags, assets ou propostas (ciclo 346).
    CarregarEspecial(super::especiais::TipoEspecial),
    /// Abrir a página de tags, assets ou propostas, criando se falta.
    AbrirEspecial(super::especiais::TipoEspecial),
    /// Abrir a conversa de planejar, executar ou alterar a página
    /// (ciclo 348), com a pergunta pronta no campo.
    ConversaDoFluxo {
        /// A spec ou proposta.
        pagina: String,
        /// "Pedir alteração"; senão, avançar.
        alterar: bool,
    },
    /// Listar os templates pra "Nova página" (ciclo 350).
    ListarTemplates,
    /// Ver o status do git e as ações (ciclo 349).
    StatusDoGit,
    /// `git pull`.
    GitPull,
    /// Commit de tudo e push, com a mensagem.
    GitCommit(String),
    /// Os commits que mexeram na página.
    HistoricoDoGit(String),
    /// Listar `assets/` pro menu `/` (ciclo 347).
    AssetsParaInserir,
    /// Excluir um arquivo de `assets/`.
    ExcluirAsset(String),
    /// Aplicar (ou recusar) uma proposta do agente.
    DecidirProposta {
        /// O id da proposta.
        id: String,
        /// Aplicar; `false` recusa.
        aplicar: bool,
    },
    /// Gravar uma página nova com este conteúdo e abri-la.
    CriarPagina {
        /// Caminho no vault.
        path: String,
        /// O arquivo inteiro.
        conteudo: String,
    },
    /// Criar uma página pelo título (e tipo) e abri-la.
    CriarPaginaComTitulo {
        /// O título.
        titulo: String,
        /// `type:` do frontmatter.
        tipo: Option<String>,
    },
    /// Abrir (ou criar) o diário de hoje.
    AbrirHoje,
    /// Apagar a página.
    ExcluirPagina(String),
    /// Gravar as preferências da TUI.
    GravarPreferencias,
    /// Gravar a pergunta na conversa e disparar o agente (ciclo 340).
    EnviarNaConversa {
        /// A conversa.
        path: String,
        /// O que a pessoa escreveu.
        pergunta: String,
        /// As páginas que vão de contexto.
        anexos: Vec<String>,
    },
    /// Parar o agente desta conversa.
    InterromperAgente(String),
    /// Criar a página de execução a partir de uma resposta, anexar e pedir
    /// a implementação.
    ExecutarDaConversa {
        /// A conversa.
        conversa: String,
        /// A resposta do agente.
        texto: String,
    },
    /// Criar uma página a partir de um template (botão de ação, ciclo 342).
    CriarDeTemplate {
        /// O template.
        template: String,
        /// O título.
        titulo: String,
        /// A pasta.
        pasta: Option<String>,
    },
    /// Gravar uma propriedade no frontmatter de uma página.
    DefinirPropriedade {
        /// A página.
        path: String,
        /// O campo.
        campo: String,
        /// O valor.
        valor: String,
    },
    /// Buscar no conteúdo das páginas.
    BuscarConteudo(String),
    /// Criar uma pasta (`pages/…`).
    CriarPasta(String),
    /// Criar uma página numa pasta.
    CriarPaginaNaPasta {
        /// A pasta (`pages/…`).
        pasta: String,
        /// O título.
        titulo: String,
    },
    /// Mover uma página.
    MoverPagina {
        /// De onde.
        de: String,
        /// Pra onde.
        para: String,
    },
    /// Exportar uma pasta num arquivo só.
    ExportarPasta(String),
    /// Ler a página de um prompt padrão e aplicar ao campo da conversa.
    CarregarPrompt(String),
    /// Regravar a lista de anexos da conversa.
    AnexosDaConversa {
        /// A conversa.
        conversa: String,
        /// A lista nova.
        lista: Vec<String>,
    },
}

/// As preferências da TUI, gravadas fora do vault (ciclo 339).
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Preferencias {
    /// O tema (`escuro`, `papel`, `contraste`, `claro`).
    #[serde(default = "tema_padrao")]
    pub tema: String,
    /// A lista de páginas à esquerda.
    #[serde(default = "verdadeiro")]
    pub sidebar: bool,
    /// O agente das conversas.
    #[serde(default)]
    pub agente: Option<anotadinho_core::agente::Adaptador>,
}

fn tema_padrao() -> String {
    "escuro".into()
}
fn verdadeiro() -> bool {
    true
}

impl Default for Preferencias {
    fn default() -> Self {
        Self { tema: tema_padrao(), sidebar: true, agente: None }
    }
}

/// O modal aberto.
#[derive(Debug, Clone, PartialEq)]
pub enum Modal {
    /// A barra de comandos.
    Paleta(Lista),
    /// Uma escolha numa lista, com o que fazer com ela.
    Escolha {
        /// O título da caixa.
        titulo: String,
        /// As opções.
        lista: Lista,
        /// O que a escolha faz.
        acao: AcaoDaEscolha,
    },
    /// Sim ou não.
    Confirmar {
        /// O título.
        titulo: String,
        /// A pergunta.
        mensagem: String,
        /// O que o sim faz.
        acao: Pedido,
    },
    /// Um texto pra digitar.
    Entrada {
        /// O título.
        titulo: String,
        /// O campo.
        campo: Campo,
        /// O que o texto faz.
        acao: AcaoDaEntrada,
    },
    /// Os atalhos, com a rolagem.
    Atalhos(usize),
    /// As opções de uma coluna de seleção da tabela.
    Opcoes(EditorDeOpcoes),
    /// O seletor de prompt padrão da conversa (ciclo 341).
    Prompt(SeletorDePrompt),
    /// Um texto longo pra ler, com a rolagem (o "Visualizar").
    Visualizar(String, usize),
    /// Os detalhes de um item num formulário (ciclo 343): o cartão do
    /// kanban, o evento do calendário.
    Detalhe {
        /// O título da caixa.
        titulo: String,
        /// Os campos.
        form: crate::componentes::Formulario,
        /// O que o formulário edita.
        alvo: AlvoDoDetalhe,
    },
}

/// O que um [`Modal::Detalhe`] edita.
#[derive(Debug, Clone, PartialEq)]
pub enum AlvoDoDetalhe {
    /// Um cartão do kanban.
    Cartao {
        /// O kanban.
        embed: Caminho,
        /// O item no arquivo.
        indice: usize,
    },
    /// Um evento do calendário.
    Evento {
        /// O calendário.
        embed: Caminho,
        /// A entrada no arquivo.
        indice: usize,
    },
    /// O frontmatter da página aberta (ciclo 344).
    Propriedades,
    /// A configuração de um botão de ações.
    Botao {
        /// As ações.
        embed: Caminho,
        /// O botão.
        indice: usize,
    },
    /// A configuração de uma consulta.
    Consulta {
        /// A consulta.
        embed: Caminho,
    },
    /// O agente das conversas (ciclo 350).
    Agente,
}

/// O seletor de prompt padrão: a lista e os campos das variáveis.
#[derive(Debug, Clone, PartialEq)]
pub struct SeletorDePrompt {
    /// "Nenhum" e os prompts do vault; a chave é o caminho.
    pub lista: Lista,
    /// As variáveis do prompt em uso, com o que já foi escrito.
    pub campos: Vec<(String, Campo)>,
    /// O campo com o teclado; `None` é a lista.
    pub foco: Option<usize>,
}

/// O editor das opções de uma coluna de seleção (ciclo 339).
#[derive(Debug, Clone, PartialEq)]
pub struct EditorDeOpcoes {
    /// A tabela.
    pub embed: Caminho,
    /// A coluna.
    pub coluna: usize,
    /// O nome dela, pro título.
    pub nome: String,
    /// As opções.
    pub lista: Lista,
    /// Uma opção sendo escrita: `Some(i)` renomeia a `i`, `None` é nova.
    pub editando: Option<(Option<usize>, Campo)>,
    /// O primeiro `d` do `dd`.
    pub d_pendente: bool,
}

/// O que o texto de uma [`Modal::Entrada`] faz.
#[derive(Debug, Clone, PartialEq)]
pub enum AcaoDaEntrada {
    /// Título de uma página nova, com o tipo.
    NovaPagina(Option<String>),
    /// Título de uma página nova dentro de uma pasta (ciclo 345).
    PaginaNaPasta(String),
    /// Nome de uma pasta nova dentro de outra.
    NovaPasta(String),
    /// Título de uma página nova a partir de um template (ciclo 342).
    PaginaDeTemplate {
        /// O template.
        template: String,
        /// A pasta onde ela nasce.
        pasta: Option<String>,
    },
    /// URL ou caminho de uma imagem pro bloco novo (ciclo 347).
    Imagem,
    /// Código Mermaid pro bloco novo.
    Mermaid,
    /// A mensagem do commit (ciclo 349).
    Commit,
}

/// O que uma [`Modal::Escolha`] faz com o item escolhido.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcaoDaEscolha {
    /// O menu de personalização.
    Personalizar,
    /// Trocar o tema pela chave.
    Tema,
    /// Trocar o agente pelo índice do preset.
    Agente,
    /// O que fazer com a resposta `n` da conversa (ciclo 340).
    RespostaDoAgente(usize),
    /// Anexar a página escolhida à conversa.
    Anexar,
    /// Tirar o anexo escolhido.
    Desanexar,
    /// Mover a página aberta pra pasta escolhida (ciclo 345).
    MoverPara,
    /// Exportar a pasta escolhida.
    Exportar,
    /// Abrir a página escolhida (chave = caminho) — resultados de busca,
    /// wikilinks (ciclo 342).
    AbrirPagina,
    /// O item do menu `/` (ciclo 347).
    Inserir,
    /// O arquivo de `assets/` a inserir.
    Asset,
    /// O menu do git (ciclo 349): pull, commit + push, abrir o arquivo.
    Git,
    /// Só mostra (o histórico da página).
    Mostrar,
    /// O template da página nova (ciclo 350); chave vazia é em branco.
    Template,
}

/// Os comandos da barra, como na janela.
const COMANDOS: &[(&str, &str)] = &[
    ("Nova conversa com o agente", "nova-conversa"),
    ("Nova página", "nova-pagina"),
    ("Nova página: Kanban", "nova-pagina:kanban"),
    ("Nova página: Calendário", "nova-pagina:calendar"),
    ("Nova página: Tabela de tarefas", "nova-pagina:table"),
    ("Nova página: Conversa", "nova-pagina:conversa"),
    ("Alternar tema", "alternar-tema"),
    ("Escolher tema…", "escolher-tema"),
    ("Alternar sidebar", "alternar-sidebar"),
    ("Ir pra Hoje (journal)", "hoje"),
    ("Personalizar…", "personalizar"),
    ("Trocar agente…", "trocar-agente"),
    ("Configurar agente…", "configurar-agente"),
    ("Nova página a partir de template…", "nova-pagina"),
    ("Propriedades da página…", "propriedades"),
    ("Ver atalhos", "atalhos"),
    ("Nova pasta…", "nova-pasta"),
    ("Mover página pra pasta…", "mover-pagina"),
    ("Exportar pasta…", "exportar-pasta"),
    ("Exportar vault inteiro", "exportar-vault"),
    ("Inserir bloco ou embed…", "inserir"),
    ("Git: status e sincronizar…", "git"),
    ("Git: pull", "git-pull"),
    ("Git: commit + push…", "git-commit"),
    ("Histórico da página (git)", "git-historico"),
    ("Próxima aba", "aba-proxima"),
    ("Aba anterior", "aba-anterior"),
    ("Fechar aba", "aba-fechar"),
    ("Ver tags", "ver-tags"),
    ("Ver assets", "ver-assets"),
    ("Propostas do agente", "propostas"),
    ("Excluir a página aberta", "excluir-pagina"),
];

/// Abre a barra de comandos: os comandos, depois as páginas.
pub fn abrir_paleta(e: &mut Estado) {
    // Numa conversa, os comandos dela vêm primeiro.
    let mut itens: Vec<Item> = super::conversa::comandos(e);
    itens.extend(COMANDOS.iter().map(|(r, c)| Item::novo("ϟ", *r, *c)));
    itens.extend(
        e.paginas
            .iter()
            .map(|p| Item::novo("≡", p.title.clone(), format!("pagina:{}", p.path)).com_detalhe(p.path.clone())),
    );
    e.modal = Some(Modal::Paleta(Lista::filtravel(itens)));
}

fn menu_de_personalizacao(e: &Estado) -> Lista {
    let agente = e.preferencias.agente.as_ref().map(|a| a.nome.clone()).unwrap_or_else(|| "padrão".into());
    Lista::menu(vec![
        Item::novo("◐", "Tema", "tema").com_detalhe(e.preferencias.tema.clone()),
        Item::novo("▤", "Sidebar", "sidebar").com_detalhe(if e.preferencias.sidebar { "visível" } else { "escondida" }),
        Item::novo("ϟ", "Agente das conversas", "agente").com_detalhe(agente),
        Item::novo("?", "Ver atalhos", "atalhos"),
    ])
}

fn escolha_de_tema(e: &Estado) -> Modal {
    let mut lista = Lista::menu(
        crate::tema::TEMAS
            .iter()
            .map(|t| Item::novo(if *t == e.preferencias.tema { "●" } else { "○" }, *t, *t))
            .collect(),
    );
    lista.selecionado = crate::tema::TEMAS.iter().position(|t| *t == e.preferencias.tema).unwrap_or(0);
    Modal::Escolha { titulo: "Tema".into(), lista, acao: AcaoDaEscolha::Tema }
}

fn escolha_de_agente(e: &Estado) -> Modal {
    let atual = e.preferencias.agente.as_ref().map(|a| a.nome.clone());
    let presets = anotadinho_core::agente::Adaptador::presets();
    let lista = Lista::menu(
        presets
            .iter()
            .enumerate()
            .map(|(i, a)| {
                Item::novo(if Some(&a.nome) == atual.as_ref() { "●" } else { "○" }, a.nome.clone(), i.to_string())
                    .com_detalhe(a.binario.clone())
            })
            .chain(std::iter::once(Item::novo("✎", "Configurar…", "configurar").com_detalhe("executável, argumentos, pastas")))
            .collect(),
    );
    Modal::Escolha { titulo: "Agente das conversas".into(), lista, acao: AcaoDaEscolha::Agente }
}

/// Aplica o tema e pede pra gravar a preferência.
pub fn trocar_tema(e: &mut Estado, tema: &str) {
    e.tema = crate::tema::Tema::novo(tema);
    e.preferencias.tema = tema.to_string();
    e.pedidos.push(Pedido::GravarPreferencias);
    e.aviso = Some(format!("tema: {tema}"));
}

pub(super) fn executar(e: &mut Estado, chave: &str) {
    e.modal = None;
    if super::conversa::executar(e, chave) {
        return;
    }
    if let Some(path) = chave.strip_prefix("pagina:") {
        e.pedidos.push(Pedido::AbrirPagina(path.to_string()));
        return;
    }
    if let Some(tipo) = chave.strip_prefix("nova-pagina:") {
        e.modal = Some(Modal::Entrada {
            titulo: format!("Nova página ({tipo})"),
            campo: Campo::default(),
            acao: AcaoDaEntrada::NovaPagina(Some(tipo.to_string())),
        });
        return;
    }
    match chave {
        // Com templates no vault, primeiro qual (ciclo 350), como a janela.
        "nova-pagina" => e.pedidos.push(Pedido::ListarTemplates),
        "configurar-agente" => abrir_configuracao_do_agente(e),
        "nova-conversa" => {
            let Some(carimbo) = e.agora.clone() else {
                e.aviso = Some("sem relógio pra datar a conversa".into());
                return;
            };
            let titulo = format!("Conversa de {carimbo}");
            let atual = e.paginas.get(e.pagina).map(|p| p.path.clone());
            let anexos: Vec<String> = atual.iter().cloned().collect();
            let conteudo = anotadinho_core::conversa::montar_pagina(&titulo, atual.as_deref(), &anexos);
            let path = format!("pages/conversas/{}.md", anotadinho_core::conversa::nome_de_arquivo(&carimbo));
            e.pedidos.push(Pedido::CriarPagina { path, conteudo });
        }
        "alternar-tema" => {
            let temas = crate::tema::TEMAS;
            let agora = temas.iter().position(|t| *t == e.preferencias.tema).unwrap_or(0);
            let proximo = temas[(agora + 1) % temas.len()];
            trocar_tema(e, proximo);
        }
        "escolher-tema" => e.modal = Some(escolha_de_tema(e)),
        "alternar-sidebar" => {
            e.preferencias.sidebar = !e.preferencias.sidebar;
            if !e.preferencias.sidebar {
                e.foco = super::Foco::Conteudo;
            }
            e.pedidos.push(Pedido::GravarPreferencias);
        }
        "hoje" => e.pedidos.push(Pedido::AbrirHoje),
        "inserir" => super::markdown::inserir_pela_barra(e),
        "aba-proxima" => super::comando_de_aba(e, "Alt+l"),
        "aba-anterior" => super::comando_de_aba(e, "Alt+h"),
        "aba-fechar" => super::comando_de_aba(e, "Alt+q"),
        "git" => e.pedidos.push(Pedido::StatusDoGit),
        "git-pull" => e.pedidos.push(Pedido::GitPull),
        "git-commit" => pedir_mensagem_do_commit(e),
        "git-historico" => {
            if let Some(p) = e.paginas.get(e.pagina) {
                e.pedidos.push(Pedido::HistoricoDoGit(p.path.clone()));
            }
        }
        "ver-tags" => e.pedidos.push(Pedido::AbrirEspecial(super::especiais::TipoEspecial::Tags)),
        "ver-assets" => e.pedidos.push(Pedido::AbrirEspecial(super::especiais::TipoEspecial::Assets)),
        "propostas" => e.pedidos.push(Pedido::AbrirEspecial(super::especiais::TipoEspecial::Propostas)),
        "personalizar" => {
            e.modal = Some(Modal::Escolha {
                titulo: "Personalizar".into(),
                lista: menu_de_personalizacao(e),
                acao: AcaoDaEscolha::Personalizar,
            })
        }
        "trocar-agente" => e.modal = Some(escolha_de_agente(e)),
        "propriedades" => {
            super::edicao::abrir_propriedades(e);
        }
        "atalhos" => e.modal = Some(Modal::Atalhos(0)),
        "nova-pasta" => abrir_entrada_na_pasta(e, true),
        "mover-pagina" => abrir_mover_pagina(e),
        "exportar-vault" => e.pedidos.push(Pedido::ExportarPasta(String::new())),
        "exportar-pasta" => {
            let itens = crate::sidebar::todas_as_pastas(&e.arvore_sidebar).into_iter().map(|p| Item::novo("▭", p.clone(), p)).collect();
            e.modal = Some(Modal::Escolha { titulo: "Exportar pasta".into(), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::Exportar });
        }
        "excluir-pagina" => {
            if let Some(p) = e.paginas.get(e.pagina) {
                e.modal = Some(Modal::Confirmar {
                    titulo: "Excluir página".into(),
                    mensagem: format!("Excluir \"{}\"? O arquivo {} sai do vault.", p.title, p.path),
                    acao: Pedido::ExcluirPagina(p.path.clone()),
                });
            }
        }
        _ => {}
    }
}

/// Uma tecla com um modal aberto. O modal recebe TUDO.
pub fn tecla(e: &mut Estado, tecla: &str) {
    let Some(modal) = e.modal.take() else { return };
    match modal {
        Modal::Paleta(lista)
            if tecla == "Ctrl+f" || (tecla == "Enter" && lista.visiveis().is_empty()) =>
        {
            // Busca no CONTEÚDO (ciclo 342), como a paleta da janela a
            // partir de 3 letras — aqui sob pedido, porque indexar o vault
            // a cada tecla travaria o terminal.
            let termo = lista.filtro.as_ref().map(|c| c.texto.trim().to_string()).unwrap_or_default();
            if termo.chars().count() >= 2 {
                e.pedidos.push(Pedido::BuscarConteudo(termo));
            } else {
                e.modal = Some(Modal::Paleta(lista));
            }
        }
        Modal::Paleta(mut lista) => match lista.tecla(tecla) {
            Resposta::Escolhido(chave) => executar(e, &chave),
            Resposta::Fechar => {}
            Resposta::Nada => e.modal = Some(Modal::Paleta(lista)),
        },
        Modal::Escolha { titulo, mut lista, acao } => match lista.tecla(tecla) {
            Resposta::Escolhido(chave) => match acao {
                AcaoDaEscolha::Personalizar => match chave.as_str() {
                    "tema" => e.modal = Some(escolha_de_tema(e)),
                    "sidebar" => {
                        executar(e, "alternar-sidebar");
                        e.modal = Some(Modal::Escolha {
                            titulo,
                            lista: {
                                let mut l = menu_de_personalizacao(e);
                                l.selecionado = 1;
                                l
                            },
                            acao,
                        });
                    }
                    "agente" => e.modal = Some(escolha_de_agente(e)),
                    "atalhos" => e.modal = Some(Modal::Atalhos(0)),
                    _ => {}
                },
                AcaoDaEscolha::Tema => trocar_tema(e, &chave),
                AcaoDaEscolha::RespostaDoAgente(i) => super::conversa::acao_na_resposta(e, i, &chave),
                AcaoDaEscolha::Anexar => super::conversa::mudar_anexo(e, &chave, true),
                AcaoDaEscolha::Desanexar => super::conversa::mudar_anexo(e, &chave, false),
                AcaoDaEscolha::AbrirPagina => e.pedidos.push(Pedido::AbrirPagina(chave)),
                AcaoDaEscolha::Inserir => super::markdown::escolher(e, &chave),
                AcaoDaEscolha::Git => match chave.as_str() {
                    "pull" => e.pedidos.push(Pedido::GitPull),
                    "commit" => pedir_mensagem_do_commit(e),
                    arquivo => {
                        if e.paginas.iter().any(|p| p.path == arquivo) {
                            e.pedidos.push(Pedido::AbrirPagina(arquivo.to_string()));
                        } else {
                            e.modal = Some(Modal::Escolha { titulo, lista, acao });
                        }
                    }
                },
                AcaoDaEscolha::Mostrar => {}
                AcaoDaEscolha::Template if chave.is_empty() => pedir_titulo_da_pagina(e),
                AcaoDaEscolha::Template => {
                    e.modal = Some(Modal::Entrada {
                        titulo: "Nova página".into(),
                        campo: Campo::default(),
                        acao: AcaoDaEntrada::PaginaDeTemplate { template: chave, pasta: None },
                    })
                }
                AcaoDaEscolha::Asset => super::markdown::inserir_trecho(e, &super::markdown::markdown_do_asset(&chave)),
                AcaoDaEscolha::Exportar => e.pedidos.push(Pedido::ExportarPasta(chave)),
                AcaoDaEscolha::MoverPara => {
                    if let Some(p) = e.paginas.get(e.pagina) {
                        let arquivo = std::path::Path::new(&p.path).file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
                        let para = format!("{chave}/{arquivo}");
                        if para != p.path {
                            e.pedidos.push(Pedido::MoverPagina { de: p.path.clone(), para });
                        }
                    }
                }
                AcaoDaEscolha::Agente if chave == "configurar" => abrir_configuracao_do_agente(e),
                AcaoDaEscolha::Agente => {
                    if let Some(a) = chave.parse::<usize>().ok().and_then(|i| anotadinho_core::agente::Adaptador::presets().get(i).cloned()) {
                        e.aviso = Some(format!("agente: {}", a.nome));
                        e.preferencias.agente = Some(a);
                        e.pedidos.push(Pedido::GravarPreferencias);
                    }
                }
            },
            Resposta::Fechar => {
                if matches!(acao, AcaoDaEscolha::Inserir | AcaoDaEscolha::Asset) {
                    e.bloco_a_inserir = None;
                }
            }
            Resposta::Nada => e.modal = Some(Modal::Escolha { titulo, lista, acao }),
        },
        Modal::Confirmar { titulo, mensagem, acao } => match tecla {
            "y" | "s" | "Enter" => e.pedidos.push(acao),
            "n" | "Escape" | "q" => {}
            _ => e.modal = Some(Modal::Confirmar { titulo, mensagem, acao }),
        },
        Modal::Entrada { titulo, mut campo, acao } => match tecla {
            "Escape" => {
                if matches!(acao, AcaoDaEntrada::Imagem | AcaoDaEntrada::Mermaid) {
                    e.bloco_a_inserir = None;
                }
            }
            "Enter" if matches!(acao, AcaoDaEntrada::Imagem | AcaoDaEntrada::Mermaid) => {
                let t = campo.texto.trim().to_string();
                if t.is_empty() {
                    e.bloco_a_inserir = None;
                } else if acao == AcaoDaEntrada::Imagem {
                    super::markdown::inserir_trecho(e, &format!("![imagem]({t})"));
                } else {
                    super::markdown::inserir_trecho(e, &format!("```mermaid\n{t}\n```"));
                }
            }
            "Enter" => {
                let t = campo.texto.trim().to_string();
                if !t.is_empty() {
                    e.pedidos.push(match acao {
                        AcaoDaEntrada::NovaPagina(tipo) => Pedido::CriarPaginaComTitulo { titulo: t, tipo },
                        AcaoDaEntrada::PaginaNaPasta(pasta) if pasta == "pages" => Pedido::CriarPaginaComTitulo { titulo: t, tipo: None },
                        AcaoDaEntrada::PaginaNaPasta(pasta) => Pedido::CriarPaginaNaPasta { pasta, titulo: t },
                        AcaoDaEntrada::NovaPasta(dentro) => Pedido::CriarPasta(format!("{dentro}/{t}")),
                        AcaoDaEntrada::PaginaDeTemplate { template, pasta } => Pedido::CriarDeTemplate { template, titulo: t, pasta },
                        AcaoDaEntrada::Commit => Pedido::GitCommit(t),
                        AcaoDaEntrada::Imagem | AcaoDaEntrada::Mermaid => return,
                    });
                }
            }
            outra => {
                campo.tecla(outra);
                e.modal = Some(Modal::Entrada { titulo, campo, acao });
            }
        },
        Modal::Atalhos(rolagem) => match tecla {
            "Escape" | "q" | "?" => {}
            "j" | "ArrowDown" => e.modal = Some(Modal::Atalhos(rolagem + 1)),
            "k" | "ArrowUp" => e.modal = Some(Modal::Atalhos(rolagem.saturating_sub(1))),
            _ => e.modal = Some(Modal::Atalhos(rolagem)),
        },
        Modal::Opcoes(editor) => super::edicao::tecla_nas_opcoes(e, editor, tecla),
        Modal::Detalhe { titulo, mut form, alvo } => {
            use crate::componentes::RespostaDoFormulario as R;
            match form.tecla(tecla) {
                R::Fechar => return,
                R::Botao("salvar") if alvo == AlvoDoDetalhe::Agente => {
                    if salvar_agente(e, &form) {
                        return;
                    }
                }
                R::Mudou if alvo == AlvoDoDetalhe::Agente => {}
                R::Botao("excluir") => {
                    super::edicao::excluir_do_detalhe(e, &alvo);
                    return;
                }
                R::Mudou => {
                    super::edicao::aplicar_detalhe(e, &alvo, &mut form);
                }
                _ => {}
            }
            e.modal = Some(Modal::Detalhe { titulo, form, alvo });
        }
        Modal::Visualizar(texto, rolagem) => match tecla {
            "Escape" | "q" | "Enter" => {}
            "j" | "ArrowDown" => e.modal = Some(Modal::Visualizar(texto, rolagem + 1)),
            "k" | "ArrowUp" => e.modal = Some(Modal::Visualizar(texto, rolagem.saturating_sub(1))),
            _ => e.modal = Some(Modal::Visualizar(texto, rolagem)),
        },
        Modal::Prompt(mut sel) => {
            match (tecla, sel.foco) {
                ("Escape", _) => return,
                // Visualizar: o texto final, se não falta marcador.
                ("Ctrl+v", _) => {
                    let pronto = e.conversa.as_ref().and_then(|c| {
                        let texto = c.rascunho.texto.clone();
                        let falta = c.prompt.as_ref().is_some_and(|p| !p.pendentes().is_empty());
                        (!falta && !texto.trim().is_empty()).then_some(texto)
                    });
                    match pronto {
                        Some(texto) => {
                            e.modal = Some(Modal::Visualizar(texto, 0));
                            return;
                        }
                        None => e.aviso = Some("Preencha todos os marcadores antes de visualizar.".into()),
                    }
                }
                ("Tab" | "ArrowDown", _) if !sel.campos.is_empty() => {
                    sel.foco = match sel.foco {
                        None => Some(0),
                        Some(i) if i + 1 < sel.campos.len() => Some(i + 1),
                        Some(_) => None,
                    };
                }
                ("ArrowUp", Some(i)) => sel.foco = i.checked_sub(1),
                ("Enter", Some(i)) => {
                    sel.foco = if i + 1 < sel.campos.len() { Some(i + 1) } else { None };
                    // No último campo, Enter fecha: o campo da conversa está
                    // pronto pra enviar.
                    if sel.foco.is_none() {
                        return;
                    }
                }
                (outra, Some(i)) => {
                    let (nome, campo) = &mut sel.campos[i];
                    if campo.tecla(outra) {
                        let (nome, valor) = (nome.clone(), campo.texto.clone());
                        super::conversa::preencher_variavel(e, &nome, &valor);
                    }
                }
                (outra, None) => match sel.lista.tecla(outra) {
                    Resposta::Escolhido(chave) if chave.is_empty() => {
                        super::conversa::tirar_prompt(e);
                        return;
                    }
                    Resposta::Escolhido(chave) => {
                        // O `main` lê a página e chama `aplicar_prompt`,
                        // que reabre o seletor com os campos.
                        e.pedidos.push(Pedido::CarregarPrompt(chave));
                        return;
                    }
                    Resposta::Fechar => return,
                    Resposta::Nada => {}
                },
            }
            e.modal = Some(Modal::Prompt(sel));
        }
    }
}

/// Os atalhos, por assunto — o `CheatsheetModal` da janela.
pub const ATALHOS: &[(&str, &[(&str, &str)])] = &[
    (
        "Geral",
        &[
            (": ou Ctrl+K", "barra de comandos"),
            ("?", "estes atalhos"),
            ("Tab", "trocar entre páginas e conteúdo"),
            ("Alt+1…9", "ir pra aba"),
            ("Ctrl+W Alt+H/L", "próxima / anterior aba"),
            ("Alt+Q", "fechar a aba"),
            ("/", "filtrar"),
            ("q", "sair"),
        ],
    ),
    (
        "Navegar",
        &[
            ("j k", "próximo / anterior"),
            ("h l", "lado a lado (colunas, cartões, células)"),
            ("Enter / Esc", "entrar / sair de um nível"),
            ("gg G", "começo / fim"),
            ("z ou espaço", "dobrar"),
            ("[ ] t m", "período, hoje e visão (calendário, cronograma)"),
        ],
    ),
    (
        "Editar (modo vim)",
        &[
            ("i a I A", "editar o texto"),
            ("cc S", "reescrever do zero"),
            ("o O", "criar depois / antes"),
            ("dd x", "apagar"),
            ("yy p P", "copiar e colar"),
            (">> <<", "andar pro lado"),
            ("J K", "descer / subir na ordem"),
            ("Ctrl+A Ctrl+X", "aumentar / diminuir (duração, nível, opção)"),
            ("~", "alternar (caixa, destaque, tipo)"),
            ("=", "configurar (botão, consulta)"),
            ("u Ctrl+R", "desfazer / refazer"),
            ("Esc", "confirmar a inserção"),
        ],
    ),
    (
        "Conversa",
        &[
            ("i", "escrever a mensagem"),
            ("Enter", "enviar (escrevendo)"),
            ("Ctrl+J", "quebrar linha na mensagem"),
            ("Esc", "sair do campo (o rascunho fica)"),
            ("j k G", "passar pelas mensagens"),
            ("Enter", "virar spec/proposta/execução (na resposta)"),
            ("y", "copiar a mensagem"),
            ("Ctrl+X", "interromper o agente"),
            ("p / Ctrl+P", "prompt padrão (Tab campos, Ctrl+V visualizar)"),
            (":", "anexar, tirar anexo, trocar agente"),
        ],
    ),
];

/// Desenha o modal aberto por cima de tudo.
pub fn desenhar(f: &mut Frame, e: &Estado) {
    let Some(modal) = &e.modal else { return };
    let tela = f.area();
    let t = &e.tema;
    match modal {
        Modal::Paleta(lista) => {
            // A caixa encolhe com o que sobrou do filtro, como na janela.
            let altura = (lista.visiveis().len() as u16 + 4).clamp(6, 24);
            let area = componentes::area_do_modal(tela, 70, altura);
            let dentro = componentes::desenhar_modal(f, area, "", "↑↓ escolher · Enter · Ctrl+F buscar no conteúdo · Esc", t);
            componentes::desenhar_lista(f, dentro, lista, "Buscar página ou comando...", t);
        }
        Modal::Escolha { titulo, lista, .. } => {
            let filtro = if lista.filtro.is_some() { 2 } else { 0 };
            let altura = (lista.visiveis().len().max(1) as u16 + 2 + filtro).min(22);
            let area = componentes::area_do_modal(tela, 56, altura);
            let dentro = componentes::desenhar_modal(f, area, titulo, "j k · Enter · Esc", t);
            componentes::desenhar_lista(f, dentro, lista, "", t);
        }
        Modal::Confirmar { titulo, mensagem, .. } => {
            let area = componentes::area_do_modal(tela, 60, 6);
            let dentro = componentes::desenhar_modal(f, area, titulo, "y sim · n não", t);
            let texto = Paragraph::new(vec![
                Line::from(Span::styled(mensagem.clone(), Style::default().fg(t.var("text-primary")))),
                Line::from(""),
                Line::from(vec![
                    Span::styled(" y ", Style::default().bg(t.var("error")).fg(t.var("bg-base")).add_modifier(Modifier::BOLD)),
                    Span::raw("  "),
                    Span::styled(" n ", Style::default().bg(t.var("bg-elevated")).fg(t.var("text-primary"))),
                ]),
            ])
            .wrap(ratatui::widgets::Wrap { trim: true });
            f.render_widget(texto, dentro);
        }
        Modal::Entrada { titulo, campo, .. } => {
            let area = componentes::area_do_modal(tela, 60, 3);
            let dentro = componentes::desenhar_modal(f, area, titulo, "Enter criar · Esc", t);
            let mut spans = vec![Span::raw(" ")];
            spans.extend(campo.spans(Style::default().fg(t.var("text-primary")), "Título da página", t));
            f.render_widget(Paragraph::new(Line::from(spans)), dentro);
        }
        Modal::Atalhos(rolagem) => {
            let area = componentes::area_do_modal(tela, 72, 30);
            let dentro = componentes::desenhar_modal(f, area, "Atalhos", "j k rolar · Esc", t);
            let mut linhas = Vec::new();
            for (grupo, itens) in ATALHOS {
                linhas.push(Line::from(Span::styled(
                    grupo.to_string(),
                    Style::default().fg(t.var("accent-blue")).add_modifier(Modifier::BOLD),
                )));
                for (teclas, o_que) in *itens {
                    linhas.push(Line::from(vec![
                        Span::styled(format!("  {teclas:<16}"), Style::default().fg(t.var("text-primary")).add_modifier(Modifier::BOLD)),
                        Span::styled(o_que.to_string(), Style::default().fg(t.var("text-muted"))),
                    ]));
                }
                linhas.push(Line::from(""));
            }
            let max = linhas.len().saturating_sub(dentro.height as usize);
            f.render_widget(Paragraph::new(linhas).scroll(((*rolagem).min(max) as u16, 0)), dentro);
        }
        Modal::Detalhe { titulo, form, .. } => {
            let linhas = form.linhas(62, t);
            let altura = (linhas.len() as u16 + 2).min(tela.height.saturating_sub(2));
            let area = componentes::area_do_modal(tela, 66, altura);
            let rodape = if form.editando.is_some() { "Enter/Esc confirmar" } else { "j k · Enter editar · o item · dd · ~ marcar · Esc fechar" };
            let dentro = componentes::desenhar_modal(f, area, titulo, rodape, t);
            let cursor = form.linha_do_cursor();
            let rolagem = cursor.saturating_sub(dentro.height.saturating_sub(2) as usize);
            f.render_widget(Paragraph::new(linhas).scroll((rolagem as u16, 0)), dentro);
        }
        Modal::Visualizar(texto, rolagem) => {
            let area = componentes::area_do_modal(tela, 90, tela.height.saturating_sub(6));
            let dentro = componentes::desenhar_modal(f, area, "Visualizar", "j k rolar · Esc", t);
            let p = Paragraph::new(texto.clone())
                .style(Style::default().fg(t.var("text-primary")))
                .wrap(ratatui::widgets::Wrap { trim: false })
                .scroll((*rolagem as u16, 0));
            f.render_widget(p, dentro);
        }
        Modal::Prompt(sel) => {
            // Como o popover da janela: embaixo, à esquerda, em cima do
            // botão do prompt.
            let altura_lista = sel.lista.itens.len().min(10) as u16;
            let altura = altura_lista + if sel.campos.is_empty() { 0 } else { sel.campos.len() as u16 * 2 + 1 } + 2;
            let largura = 56.min(tela.width.saturating_sub(4));
            let x = if e.preferencias.sidebar { tela.width * 30 / 100 + 2 } else { 2 };
            let y = tela.height.saturating_sub(altura + 3);
            let area = Rect::new(x.min(tela.width.saturating_sub(largura)), y, largura, altura.min(tela.height));
            let rodape = if sel.campos.is_empty() { "j k · Enter usar · Esc fechar" } else { "Tab campos · Ctrl+V visualizar · Esc fechar" };
            let dentro = componentes::desenhar_modal(f, area, "Prompt padrão", rodape, t);
            let lista_area = Rect { height: altura_lista.min(dentro.height), ..dentro };
            let mut lista = sel.lista.clone();
            if sel.foco.is_some() {
                // A lista sem destaque enquanto o teclado está num campo.
                lista.selecionado = usize::MAX;
            }
            componentes::desenhar_lista(f, lista_area, &lista, "", t);
            let mut linhas: Vec<Line<'static>> = Vec::new();
            if !sel.campos.is_empty() {
                linhas.push(Line::from(Span::styled("─".repeat(dentro.width as usize), Style::default().fg(t.var("border")))));
            }
            for (i, (nome, campo)) in sel.campos.iter().enumerate() {
                let aceso = sel.foco == Some(i);
                linhas.push(Line::from(Span::styled(
                    format!(" {{{{{nome}}}}}"),
                    Style::default().fg(if aceso { t.var("accent-blue") } else { t.var("text-muted") }),
                )));
                let mut spans = vec![Span::styled(" ", Style::default().bg(t.var("bg-base")))];
                if aceso {
                    spans.extend(campo.spans(Style::default().fg(t.var("text-primary")).bg(t.var("bg-base")), "Preencha antes de enviar", t));
                } else if campo.texto.is_empty() {
                    spans.push(Span::styled("Preencha antes de enviar", Style::default().fg(t.var("text-muted")).bg(t.var("bg-base"))));
                } else {
                    spans.push(Span::styled(campo.texto.clone(), Style::default().fg(t.var("text-primary")).bg(t.var("bg-base"))));
                }
                let usado: usize = spans.iter().map(|s| s.content.chars().count()).sum();
                spans.push(Span::styled(" ".repeat((dentro.width as usize).saturating_sub(usado + 1)), Style::default().bg(t.var("bg-base"))));
                linhas.push(Line::from(spans));
            }
            let campos_area = Rect { y: dentro.y + lista_area.height, height: dentro.height.saturating_sub(lista_area.height), ..dentro };
            f.render_widget(Paragraph::new(linhas), campos_area);
        }
        Modal::Opcoes(ed) => {
            let altura = (ed.lista.itens.len() as u16 + 4).clamp(6, 20);
            let area = componentes::area_do_modal(tela, 50, altura);
            let dentro =
                componentes::desenhar_modal(f, area, &format!("Opções de {}", ed.nome), "o nova · a renomear · dd · J K · Esc", t);
            let lista_area = Rect { height: dentro.height.saturating_sub(2), ..dentro };
            componentes::desenhar_lista(f, lista_area, &ed.lista, "", t);
            let linha = Rect { y: dentro.y + dentro.height.saturating_sub(1), height: 1, ..dentro };
            let conteudo = match &ed.editando {
                Some((alvo, campo)) => {
                    let mut spans = vec![Span::styled(
                        if alvo.is_some() { " renomear: " } else { " nova: " },
                        Style::default().fg(t.var("text-muted")),
                    )];
                    spans.extend(campo.spans(Style::default().fg(t.var("text-primary")), "", t));
                    Line::from(spans)
                }
                None if ed.lista.itens.is_empty() => {
                    Line::from(Span::styled(" nenhuma opção ainda — o cria", Style::default().fg(t.var("text-muted"))))
                }
                None => Line::from(""),
            };
            f.render_widget(Paragraph::new(conteudo), linha);
        }
    }
}

/// Mostra os resultados da busca no conteúdo numa escolha (ciclo 342).
pub fn mostrar_resultados_da_busca(e: &mut Estado, termo: &str, hits: &[anotadinho_core::embed::SearchHit]) {
    if hits.is_empty() {
        e.aviso = Some(format!("nada com \"{termo}\" no conteúdo"));
        return;
    }
    let itens = hits
        .iter()
        .map(|h| {
            let titulo = e.paginas.iter().find(|p| p.path == h.path).map(|p| p.title.clone()).unwrap_or_else(|| h.path.clone());
            let trecho: String = h.snippet.replace("**", "").split_whitespace().collect::<Vec<_>>().join(" ");
            let origem = h.origem.as_ref().map(|o| format!("{o} · ")).unwrap_or_default();
            Item::novo("⌕", titulo, h.path.clone()).com_detalhe(format!("{origem}{}", trecho.chars().take(60).collect::<String>()))
        })
        .collect();
    e.modal = Some(Modal::Escolha {
        titulo: format!("\"{termo}\" no conteúdo"),
        lista: Lista::filtravel(itens),
        acao: AcaoDaEscolha::AbrirPagina,
    });
}

/// Entrada de página nova (ou pasta nova) na pasta do cursor da sidebar.
pub fn abrir_entrada_na_pasta(e: &mut Estado, pasta_nova: bool) {
    let pasta = if e.foco == super::Foco::Paginas {
        e.pasta_da_sidebar()
    } else {
        e.paginas
            .get(e.pagina)
            .and_then(|p| std::path::Path::new(&p.path).parent().map(|x| x.to_string_lossy().to_string()))
            .unwrap_or_else(|| "pages".into())
    };
    let pasta = if pasta.starts_with("journals") { "pages".to_string() } else { pasta };
    e.modal = Some(if pasta_nova {
        Modal::Entrada { titulo: format!("Nova pasta em {pasta}"), campo: Campo::default(), acao: AcaoDaEntrada::NovaPasta(pasta) }
    } else {
        Modal::Entrada { titulo: format!("Nova página em {pasta}"), campo: Campo::default(), acao: AcaoDaEntrada::PaginaNaPasta(pasta) }
    });
}

/// Escolha da pasta de destino da página selecionada.
pub fn abrir_mover_pagina(e: &mut Estado) {
    let Some(p) = e.paginas.get(e.pagina) else { return };
    let atual = std::path::Path::new(&p.path).parent().map(|x| x.to_string_lossy().to_string()).unwrap_or_default();
    let itens = crate::sidebar::todas_as_pastas(&e.arvore_sidebar)
        .into_iter()
        .filter(|x| *x != atual)
        .map(|x| Item::novo("▭", x.clone(), x))
        .collect();
    e.modal = Some(Modal::Escolha { titulo: format!("Mover {} pra…", p.title), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::MoverPara });
}

/// Confirmação de excluir a página selecionada.
pub fn confirmar_exclusao(e: &mut Estado) {
    executar(e, "excluir-pagina");
}

/// "Commit + Push": pede a mensagem (ciclo 349).
fn pedir_mensagem_do_commit(e: &mut Estado) {
    e.modal = Some(Modal::Entrada {
        titulo: "Mensagem do commit".into(),
        campo: crate::componentes::Campo::default(),
        acao: AcaoDaEntrada::Commit,
    });
}

/// O status do git chegou (ciclo 349): o popover da janela — os arquivos
/// mudados com o código e as ações Pull e Commit + Push. `None` é vault
/// fora de repositório.
pub fn mostrar_git(e: &mut Estado, arquivos: Option<Vec<(String, String)>>) {
    let Some(arquivos) = arquivos else {
        e.aviso = Some("Este vault não é um repositório git (ou git não está instalado).".into());
        return;
    };
    let mut itens = vec![
        Item::novo("↓", "Pull", "pull").com_detalhe("traz o que mudou no remoto"),
        Item::novo("↑", "Commit + Push…", "commit").com_detalhe("grava tudo e envia"),
    ];
    itens.extend(arquivos.into_iter().map(|(status, path)| Item::novo("⑂", path.clone(), path).com_detalhe(status)));
    let n = itens.len() - 2;
    e.modal = Some(Modal::Escolha {
        titulo: if n == 0 { "Git · Sem mudanças".into() } else { format!("Git · {n} mudança(s)") },
        lista: Lista::menu(itens),
        acao: AcaoDaEscolha::Git,
    });
}

/// O histórico da página chegou (ciclo 349), como o modal "Histórico" da
/// janela.
pub fn mostrar_historico(e: &mut Estado, commits: Option<Vec<(String, String, String)>>) {
    let Some(commits) = commits else {
        e.aviso = Some("Este vault não é um repositório git (ou git não está instalado).".into());
        return;
    };
    if commits.is_empty() {
        e.aviso = Some("Nenhum commit encontrado pra esta página.".into());
        return;
    }
    let itens = commits
        .into_iter()
        .map(|(hash, data, mensagem)| Item::novo("◷", mensagem, hash.clone()).com_detalhe(format!("{hash} · {data}")))
        .collect();
    e.modal = Some(Modal::Escolha { titulo: "Histórico".into(), lista: Lista::menu(itens), acao: AcaoDaEscolha::Mostrar });
}

fn pedir_titulo_da_pagina(e: &mut Estado) {
    e.modal = Some(Modal::Entrada { titulo: "Nova página".into(), campo: Campo::default(), acao: AcaoDaEntrada::NovaPagina(None) });
}

/// Os templates chegaram (ciclo 350): sem nenhum, direto o título; com
/// algum, "Página em branco" e eles, como o "Escolher template" da janela.
pub fn escolher_template(e: &mut Estado, templates: Vec<(String, String)>) {
    if templates.is_empty() {
        pedir_titulo_da_pagina(e);
        return;
    }
    let mut itens = vec![Item::novo("≡", "Página em branco", "")];
    itens.extend(templates.into_iter().map(|(path, titulo)| Item::novo("▤", titulo, path.clone()).com_detalhe(path)));
    e.modal = Some(Modal::Escolha { titulo: "Escolher template".into(), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::Template });
}

/// O formulário do agente (ciclo 350), como o `AgenteConfig` da janela:
/// executável, argumentos (um tem `{prompt}`), formato, tempo limite e
/// pastas.
pub fn abrir_configuracao_do_agente(e: &mut Estado) {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    use anotadinho_core::agente::FormatoSaida;
    let a = e.preferencias.agente.clone().unwrap_or_default();
    let formatos = vec![
        ("texto".to_string(), "Texto — a saída inteira é a resposta".to_string()),
        ("stream".to_string(), "JSON por linha — mostra o progresso".to_string()),
    ];
    let mut form = Formulario::novo(vec![
        C::novo("nome", "Nome", Valor::Texto(a.nome.clone())),
        C::novo("binario", "Executável", Valor::Texto(a.binario.clone())).com_dica("claude, codex, ou o caminho completo"),
        C::novo("args", "Argumentos, um por item ({prompt} em um)", Valor::Lista(a.args.clone())),
        C::novo("formato", "Formato da saída", Valor::Opcoes(formatos, usize::from(a.formato == FormatoSaida::StreamJson))),
        C::novo("timeout", "Tempo limite (minutos)", Valor::Texto((a.timeout_s / 60).to_string())),
        C::novo("cwd", "Pasta de trabalho", Valor::Texto(a.cwd.clone())).com_dica("vazia: a raiz do projeto"),
        C::novo("arg_pasta_extra", "Argumento de pasta extra", Valor::Texto(a.arg_pasta_extra.clone())).com_dica("--add-dir"),
        C::novo("pastas_extras", "Pastas extras", Valor::Lista(a.pastas_extras.clone())),
    ]);
    form.botoes.push(("salvar", "Salvar e usar".into()));
    e.modal = Some(Modal::Detalhe { titulo: "Agente das conversas".into(), form, alvo: AlvoDoDetalhe::Agente });
}

/// "Salvar e usar": valida como a janela e grava. `false` deixa o
/// formulário aberto, com o problema no aviso.
fn salvar_agente(e: &mut Estado, form: &crate::componentes::Formulario) -> bool {
    use anotadinho_core::agente::{Adaptador, FormatoSaida, TIMEOUT_MINIMO_S};
    let minutos: u64 = form.texto("timeout").trim().parse().unwrap_or(0);
    let a = Adaptador {
        nome: Some(form.texto("nome").trim().to_string()).filter(|n| !n.is_empty()).unwrap_or_else(|| "personalizado".into()),
        binario: form.texto("binario").trim().to_string(),
        args: form.lista("args").into_iter().filter(|x| !x.trim().is_empty()).collect(),
        cwd: form.texto("cwd").trim().to_string(),
        pastas_extras: form.lista("pastas_extras").into_iter().filter(|x| !x.trim().is_empty()).collect(),
        arg_pasta_extra: form.texto("arg_pasta_extra").trim().to_string(),
        timeout_s: (minutos * 60).max(TIMEOUT_MINIMO_S),
        formato: if form.escolha("formato") == "stream" { FormatoSaida::StreamJson } else { FormatoSaida::Texto },
    };
    if let Some(p) = a.validar() {
        e.aviso = Some(p.mensagem().to_string());
        return false;
    }
    e.aviso = Some(format!("agente: {}", a.nome));
    e.preferencias.agente = Some(a);
    e.pedidos.push(Pedido::GravarPreferencias);
    true
}
