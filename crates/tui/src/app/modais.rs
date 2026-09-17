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
    /// Trocar a pasta de trabalho do agente, ou dar alcance a outra
    /// (ciclo 369). Vazia, na de trabalho, é a raiz do projeto.
    PastaDoAgente {
        /// A pasta.
        pasta: String,
        /// Pasta extra.
        extra: bool,
    },
    /// Abrir uma URL ou arquivo do vault no programa do sistema (ciclo 374).
    AbrirExterno(String),
    /// Abrir outro vault, ou preparar um novo (ciclo 373).
    TrocarVault {
        /// A pasta.
        pasta: String,
        /// Preparar com a semente.
        criar: bool,
    },
    /// Gravar por cima do disco, sem a trava de versão (ciclo 363).
    GravarPorCima {
        /// A página.
        path: String,
        /// O texto.
        conteudo: String,
    },
    /// Pôr ou tirar a página como início do vault (ciclo 362).
    AlternarInicio(String),
    /// Gravar a página como HTML.
    ExportarHtml(String),
    /// Ver o registro de decisões sobre propostas (ciclo 404).
    ListarDecisoes,
    /// Gravar o conteúdo da proposta depois de a pessoa editá-lo
    /// (ciclo 411).
    AplicarPropostaEditada {
        id: String,
        conteudo: String,
    },
    /// Aplicar ou recusar várias propostas de uma vez (ciclo 409).
    DecidirVarias {
        ids: Vec<String>,
        aplicar: bool,
        motivo: String,
    },
    /// Ver o registro de execuções do agente (ciclo 406).
    ListarExecucoes,
    /// Ver o que está rodando e o que espera na fila (ciclo 408).
    VerAgentes,
    /// Medir o contexto da conversa sem abrir a prévia (ciclo 417).
    PesarContexto(String),
    /// Montar o prompt sem enviar, pra ver o que vai (ciclo 417).
    PreviaDoPrompt {
        conversa: String,
        pergunta: String,
    },
    /// Criar a página de contexto com os anexos (ciclo 416) e pôr ela no
    /// lugar deles.
    GuardarContexto {
        titulo: String,
        anexos: Vec<String>,
    },
    /// Buscar o conteúdo das transclusões da página aberta (ciclo 415).
    ResolverTransclusoes(Vec<String>),
    /// Ver os gatilhos do vault (ciclo 412).
    ListarGatilhos,
    /// Ligar ou desligar um gatilho pelo nome.
    AlternarGatilho(String),
    /// Gravar um gatilho novo ou mudado.
    GravarGatilho(anotadinho_core::gatilho::Gatilho),
    /// Apagar o gatilho pelo nome.
    ApagarGatilho(String),
    /// Abrir o formulário do gatilho pelo nome.
    EditarGatilho(String),
    /// Interromper tudo: o que roda e o que espera (ciclo 408).
    InterromperTodos,
    /// Ler as permissões de escrita do agente (ciclo 405).
    LerPermissoes,
    /// Gravar as permissões no vault.
    GravarPermissoes(anotadinho_core::permissoes::Permissoes),
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
        /// O motivo da recusa, quando a pessoa escreveu (ciclo 404).
        motivo: String,
    },
    /// Aplicar só os trechos escolhidos de uma proposta (ciclo 404).
    AplicarPropostaParcial {
        /// O id da proposta.
        id: String,
        /// O conteúdo montado com os trechos escolhidos.
        conteudo: String,
        /// Quantos trechos entraram.
        aceitos: usize,
        /// De quantos.
        de: usize,
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
    /// A cor de destaque (ciclo 358); vazia é a do tema.
    #[serde(default)]
    pub destaque: String,
    /// O estilo dos botões.
    #[serde(default = "botoes_padrao")]
    pub botoes: String,
    /// As teclas do vim trocadas (ciclo 359): ação → tecla.
    #[serde(default)]
    pub teclas_vim: std::collections::BTreeMap<String, String>,
    /// As teclas dos comandos globais: comando → tecla.
    #[serde(default)]
    pub teclas_globais: std::collections::BTreeMap<String, String>,
    /// Os agentes que a pessoa criou (ciclo 392).
    #[serde(default)]
    pub agentes: Vec<anotadinho_core::agente::Adaptador>,
    /// A página de início de cada vault (ciclo 362): vault → página.
    #[serde(default)]
    pub inicio: std::collections::BTreeMap<String, String>,
    /// O último vault aberto (ciclo 372): sem `--vault`, é ele.
    #[serde(default)]
    pub ultimo_vault: Option<String>,
    /// Grava a cada edição (ciclo 375); desligado, só no `Ctrl+S`.
    #[serde(default = "verdadeiro")]
    pub salvar_automatico: bool,
    /// A gramática do vim no conteúdo (ciclo 376); desligada, digitar
    /// edita direto.
    #[serde(default = "verdadeiro")]
    pub modo_vim: bool,
    /// Quantas execuções do agente rodam em paralelo (ciclo 408); o
    /// resto espera na fila. `0` = sem limite.
    #[serde(default = "limite_padrao")]
    pub limite_de_agentes: usize,
    /// Teto de tokens estimados do contexto (ciclo 417); `0` = sem teto.
    /// Serve de aviso, não de trava: quem manda no corte é a pessoa.
    #[serde(default = "teto_padrao")]
    pub teto_de_contexto: usize,
}

fn teto_padrao() -> usize {
    // Folgado pra modelo de 128k e apertado o bastante pra avisar antes
    // de a resposta piorar.
    40_000
}

fn limite_padrao() -> usize {
    crate::fila::LIMITE_PADRAO
}

fn botoes_padrao() -> String {
    "arredondado".into()
}

fn tema_padrao() -> String {
    // Catppuccin Mocha (ciclo 387), como na janela.
    "mocha".into()
}
fn verdadeiro() -> bool {
    true
}

impl Default for Preferencias {
    fn default() -> Self {
        Self { tema: tema_padrao(), sidebar: true, agente: None, destaque: String::new(), botoes: botoes_padrao(), teclas_vim: Default::default(), teclas_globais: Default::default(), inicio: Default::default(), agentes: Vec::new(), ultimo_vault: None, salvar_automatico: true, modo_vim: true, limite_de_agentes: limite_padrao(), teto_de_contexto: teto_padrao() }
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
    /// Texto CRU com rolagem (ciclo 417): linha por linha, sem render de
    /// markdown. É o que a prévia do prompt precisa — desenhar o prompt
    /// como página juntaria linhas e mostraria algo que o agente não
    /// recebe.
    TextoCru {
        titulo: String,
        texto: String,
        rolagem: usize,
    },
    /// Um editor de várias linhas (ciclo 390): Enter quebra a linha, as
    /// setas andam, Esc grava e fecha, Ctrl+C desiste.
    EditorDeTexto {
        /// O título.
        titulo: String,
        /// O texto e o cursor.
        campo: Campo,
        /// O que gravar.
        alvo: AlvoDoTexto,
    },
    /// A página mudou no disco enquanto se editava (ciclo 363), como a
    /// barra de conflito da janela.
    Conflito(Conflito),
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
    /// As pastas onde o agente pode propor (ciclo 405).
    Permissoes,
    /// Um gatilho do agente (ciclo 412); nome vazio é novo.
    Gatilho {
        nome: String,
    },
    /// O conteúdo de uma proposta, editado antes de aplicar (ciclo 411).
    PropostaEditada {
        /// O id da proposta.
        id: String,
        /// O alvo, pro título e pra confirmação.
        alvo: String,
    },
    /// O remapeamento de teclas (ciclo 359).
    Teclas,
    /// A imagem do menu `/` (ciclo 388).
    Imagem,
    /// Uma tabela markdown comum (ciclo 389): onde mora e onde começa.
    TabelaMd {
        /// O markdown que a contém.
        hospedeiro: super::markdown::Hospedeiro,
        /// O byte onde a tabela começa no corpo.
        inicio: usize,
    },
    /// Uma etapa do cronograma (ciclo 385).
    Barra {
        /// O cronograma.
        embed: Caminho,
        /// O item no arquivo.
        indice: usize,
    },
}

/// O que o [`Modal::EditorDeTexto`] grava.
#[derive(Debug, Clone, PartialEq)]
pub enum AlvoDoTexto {
    /// O miolo de um bloco de código, entre as cercas.
    Codigo {
        /// O markdown que o contém.
        hospedeiro: super::markdown::Hospedeiro,
        /// Onde o bloco começa.
        inicio: usize,
        /// A cerca de abertura (```` ```rust ````).
        cerca: String,
        /// A de fechamento.
        fecha: String,
    },
}

/// O conflito entre o que se escreveu e o que está no disco.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflito {
    /// A página.
    pub path: String,
    /// O texto que não gravou.
    pub meu: String,
    /// O que está no disco agora.
    pub disco: String,
    /// A opção escolhida: 0 ver a diferença, 1 manter o meu, 2 recarregar.
    pub opcao: usize,
    /// A diferença à vista.
    pub diff: bool,
    /// A rolagem da diferença.
    pub rolagem: usize,
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
    /// A URL do link do menu Formatar (ciclo 357).
    Link,
    /// A cor personalizada do menu Formatar (ciclo 394).
    CorLivre,
    /// A pasta do agente (ciclo 369): `true` é uma pasta extra.
    PastaDoAgente(bool),
    /// A pasta de outro vault (ciclo 373): `true` prepara um novo.
    Vault(bool),
    /// O motivo da recusa de uma proposta (ciclo 404).
    MotivoDaRecusa(String),
    /// Quantos agentes rodam em paralelo (ciclo 408).
    LimiteDeAgentes,
    /// O nome da página de contexto feita dos anexos (ciclo 416).
    PaginaDeContexto,
    /// O motivo de recusar várias propostas de uma vez (ciclo 409).
    MotivoDaRecusaVarias(Vec<String>),
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
    /// A tela dos agentes em andamento (ciclo 408): Enter abre a
    /// conversa, `x` interrompe tudo.
    Agentes,
    /// A página escolhida pra transcluir (ciclo 416).
    Transcluir,
    /// A tela dos gatilhos (ciclo 412): Enter liga/desliga, `o` cria,
    /// `dd` apaga.
    Gatilhos,
    /// O template da página nova (ciclo 350); chave vazia é em branco.
    Template,
    /// A marca do menu Formatar (ciclo 357).
    Formatar,
    /// A cor de destaque (ciclo 358); chave vazia é a do tema.
    Destaque,
    /// O estilo dos botões.
    Botoes,
    /// A página de uma célula de tabela (ciclo 367).
    PaginaDaCelula,
    /// A pasta extra que sai do alcance do agente (ciclo 369).
    TirarPasta,
    /// Um link de fora, no programa do sistema (ciclo 374).
    AbrirExterno,
}

/// Os comandos da barra, como na janela.
const COMANDOS: &[(&str, &str)] = &[
    ("Nova conversa com o agente", "nova-conversa"),
    ("Nova página", "nova-pagina"),
    ("Nova página: Kanban", "nova-pagina:kanban"),
    ("Nova página: Calendário", "nova-pagina:calendar"),
    ("Nova página: Tabela de tarefas", "nova-pagina:table"),
    ("Nova página: Grafo de conexões", "nova-pagina:graph"),
    ("Nova página inicial (landing)", "nova-landing"),
    ("Nova página: Conversa", "nova-pagina:conversa"),
    ("Alternar tema", "alternar-tema"),
    ("Escolher tema…", "escolher-tema"),
    ("Cor de destaque…", "destaque"),
    ("Estilo dos botões…", "botoes"),
    ("Remapear teclas…", "remapear-teclas"),
    ("Alternar sidebar", "alternar-sidebar"),
    ("Alternar salvamento automático", "salvamento-automatico"),
    ("Alternar modo vim", "modo-vim"),
    ("Salvar (Ctrl+S)", "salvar"),
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
    ("Abrir outro vault…", "abrir-vault"),
    ("Criar vault novo…", "criar-vault"),
    ("Ver tags", "ver-tags"),
    ("Ver assets", "ver-assets"),
    ("Propostas do agente", "propostas"),
    ("Decisões sobre as propostas", "decisoes"),
    ("Execuções do agente", "execucoes"),
    ("Ferramentas do agente", "ferramentas"),
    ("Agentes em andamento…", "agentes_rodando"),
    ("Gatilhos do agente…", "gatilhos"),
    ("Limite de agentes em paralelo…", "limite_agentes"),
    ("Onde o agente pode propor…", "permissoes"),
    ("Definir/remover como início", "inicio"),
    ("Exportar HTML da página", "exportar-html"),
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
        Item::novo("●", "Cor de destaque", "destaque").com_detalhe(if e.preferencias.destaque.is_empty() { "do tema".to_string() } else { e.preferencias.destaque.clone() }),
        Item::novo("▢", "Botões", "botoes").com_detalhe(e.preferencias.botoes.clone()),
        Item::novo("▤", "Sidebar", "sidebar").com_detalhe(if e.preferencias.sidebar { "visível" } else { "escondida" }),
        Item::novo("⌨", "Modo vim", "vim").com_detalhe(if e.preferencias.modo_vim { "ligado" } else { "desligado" }),
        Item::novo("⤓", "Salvamento automático", "salvamento").com_detalhe(if e.preferencias.salvar_automatico { "ligado" } else { "desligado" }),
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

fn escolha_de_destaque(e: &Estado) -> Modal {
    let atual = e.preferencias.destaque.as_str();
    let marca = |id: &str| if id == atual { "●" } else { "○" };
    let mut itens = vec![Item::novo(marca(""), "Do tema", "")];
    itens.extend(crate::tema::DESTAQUES.iter().map(|(id, nome, hex)| Item::novo(marca(id), *nome, *id).com_detalhe(*hex)));
    let mut lista = Lista::menu(itens);
    lista.selecionado = crate::tema::DESTAQUES.iter().position(|(id, _, _)| *id == atual).map_or(0, |i| i + 1);
    Modal::Escolha { titulo: "Cor de destaque".into(), lista, acao: AcaoDaEscolha::Destaque }
}

fn escolha_de_botoes(e: &Estado) -> Modal {
    let atual = e.preferencias.botoes.as_str();
    let mut lista = Lista::menu(
        crate::tema::BOTOES.iter().map(|(id, nome)| Item::novo(if *id == atual { "●" } else { "○" }, *nome, *id)).collect(),
    );
    lista.selecionado = crate::tema::BOTOES.iter().position(|(id, _)| *id == atual).unwrap_or(0);
    Modal::Escolha { titulo: "Botões".into(), lista, acao: AcaoDaEscolha::Botoes }
}

fn escolha_de_agente(e: &Estado) -> Modal {
    let atual = e.preferencias.agente.as_ref().map(|a| a.nome.clone());
    let marca = |nome: &str| if Some(nome) == atual.as_deref() { "●" } else { "○" };
    let presets = anotadinho_core::agente::Adaptador::presets();
    // Os presets e os agentes que a pessoa criou (ciclo 392), como a lista
    // do AgenteConfig da janela.
    let lista = Lista::menu(
        presets
            .iter()
            .enumerate()
            .map(|(i, a)| Item::novo(marca(&a.nome), a.nome.clone(), i.to_string()).com_detalhe(a.binario.clone()))
            .chain(
                e.preferencias
                    .agentes
                    .iter()
                    .map(|a| Item::novo(marca(&a.nome), a.nome.clone(), format!("meu:{}", a.nome)).com_detalhe(a.binario.clone())),
            )
            .chain([
                Item::novo("+", "Novo agente…", "novo").com_detalhe("outro executável"),
                Item::novo("✎", "Configurar…", "configurar").com_detalhe("executável, argumentos, pastas"),
            ])
            .collect(),
    );
    Modal::Escolha { titulo: "Agente das conversas".into(), lista, acao: AcaoDaEscolha::Agente }
}

/// Aplica o tema e pede pra gravar a preferência.
pub fn trocar_tema(e: &mut Estado, tema: &str) {
    e.tema = crate::tema::Tema::novo(tema).com_aparencia(&e.preferencias.destaque, &e.preferencias.botoes);
    e.preferencias.tema = tema.to_string();
    e.pedidos.push(Pedido::GravarPreferencias);
    e.aviso = Some(format!("tema: {tema}"));
}

pub(super) fn executar(e: &mut Estado, chave: &str) {
    e.modal = None;
    if super::conversa::executar(e, chave) {
        return;
    }
    // Um resultado no conteúdo: abre no trecho (ciclos 371 e 379).
    if let Some((termo, resto)) = chave.strip_prefix("resultado:").and_then(|r| r.split_once('\u{0}')) {
        let (path, ancora) = resto.split_once('\u{0}').unwrap_or((resto, ""));
        e.alvo_de_busca = Some(termo.to_string());
        e.alvo_ancora = (!ancora.is_empty()).then(|| ancora.to_string());
        e.pedidos.push(Pedido::AbrirPagina(path.to_string()));
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
        "destaque" => e.modal = Some(escolha_de_destaque(e)),
        "botoes" => e.modal = Some(escolha_de_botoes(e)),
        "remapear-teclas" => super::teclas::abrir(e),
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
        "salvar" => e.salvar_agora = true,
        // O "+" com a casinha da sidebar da janela (ciclo 384).
        "nova-landing" => {
            e.modal = Some(Modal::Entrada {
                titulo: "Título da página inicial".into(),
                campo: Campo::com("Início"),
                acao: AcaoDaEntrada::NovaPagina(Some("landing".into())),
            })
        }
        "modo-vim" => {
            e.preferencias.modo_vim = !e.preferencias.modo_vim;
            e.aviso = Some(if e.preferencias.modo_vim { "modo vim ligado" } else { "modo vim desligado: digite pra editar, Ctrl+K abre a barra" }.into());
            e.pedidos.push(Pedido::GravarPreferencias);
        }
        "salvamento-automatico" => {
            e.preferencias.salvar_automatico = !e.preferencias.salvar_automatico;
            e.aviso = Some(if e.preferencias.salvar_automatico { "salvamento automático ligado" } else { "salvamento automático desligado: Ctrl+S salva" }.into());
            // Religar grava o que estava pendente.
            if e.preferencias.salvar_automatico {
                e.salvar_agora = true;
            }
            e.pedidos.push(Pedido::GravarPreferencias);
        }
        "abrir-vault" | "criar-vault" => {
            let criar = chave == "criar-vault";
            e.modal = Some(Modal::Entrada {
                titulo: if criar { "Criar vault novo em (pasta)".into() } else { "Abrir o vault da pasta".into() },
                campo: Campo::default(),
                acao: AcaoDaEntrada::Vault(criar),
            });
        }
        "inicio" | "exportar-html" => {
            if let Some(p) = e.paginas.get(e.pagina).map(|p| p.path.clone()) {
                e.pedidos.push(if chave == "inicio" { Pedido::AlternarInicio(p) } else { Pedido::ExportarHtml(p) });
            }
        }
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
        "decisoes" => e.pedidos.push(Pedido::ListarDecisoes),
        "execucoes" => e.pedidos.push(Pedido::ListarExecucoes),
        "ferramentas" => mostrar_ferramentas(e),
        "agentes_rodando" => e.pedidos.push(Pedido::VerAgentes),
        "gatilhos" => e.pedidos.push(Pedido::ListarGatilhos),
        "limite_agentes" => {
            let atual = e.preferencias.limite_de_agentes;
            e.modal = Some(Modal::Entrada {
                titulo: "Quantos agentes em paralelo (0 = sem limite)".into(),
                campo: Campo::com(atual.to_string()),
                acao: AcaoDaEntrada::LimiteDeAgentes,
            });
        }
        "permissoes" => e.pedidos.push(Pedido::LerPermissoes),
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
        Modal::Paleta(mut lista) => {
            let antes = lista.filtro.as_ref().map(|c| c.texto.clone()).unwrap_or_default();
            match lista.tecla(tecla) {
                Resposta::Escolhido(chave) => executar(e, &chave),
                Resposta::Fechar => {}
                Resposta::Nada => {
                    // Com 3 letras, o conteúdo também (ciclo 379), como a
                    // paleta da janela.
                    let agora = lista.filtro.as_ref().map(|c| c.texto.trim().to_string()).unwrap_or_default();
                    if agora != antes.trim() {
                        lista.itens.retain(|it| !it.chave.starts_with("resultado:"));
                        // Só a última vale: digitar rápido não enfileira buscas.
                        e.busca_na_paleta = (agora.chars().count() >= 3).then_some(agora);
                    }
                    e.modal = Some(Modal::Paleta(lista));
                }
            }
        }
        // Na tela dos gatilhos (ciclo 412): `o` cria, `=` edita, `dd`
        // apaga o que está sob o cursor.
        Modal::Escolha { titulo, lista, acao: AcaoDaEscolha::Gatilhos } if matches!(tecla, "o" | "=" | "d") => {
            let escolhido = lista.atual().map(|it| it.chave.clone()).unwrap_or_default();
            match tecla {
                "o" => editar_gatilho(e, &gatilho_vazio()),
                "=" if !escolhido.starts_with('\u{0}') => e.pedidos.push(Pedido::EditarGatilho(escolhido)),
                "d" if !escolhido.starts_with('\u{0}') => {
                    e.modal = Some(Modal::Confirmar {
                        titulo: "Apagar gatilho".into(),
                        mensagem: format!("Apagar o gatilho {escolhido}?"),
                        acao: Pedido::ApagarGatilho(escolhido),
                    });
                }
                _ => e.modal = Some(Modal::Escolha { titulo, lista, acao: AcaoDaEscolha::Gatilhos }),
            }
        }
        // `x` na tela dos agentes interrompe tudo (ciclo 408) — a
        // lista não usa essa tecla.
        Modal::Escolha { titulo, lista, acao: AcaoDaEscolha::Agentes } if tecla == "x" => {
            e.pedidos.push(Pedido::InterromperTodos);
            // A tela fica aberta: quem manda parar quer ver parando.
            e.modal = Some(Modal::Escolha { titulo, lista, acao: AcaoDaEscolha::Agentes });
        }
        Modal::Escolha { titulo, mut lista, acao } => match lista.tecla(tecla) {
            Resposta::Escolhido(chave) => match acao {
                AcaoDaEscolha::Personalizar => match chave.as_str() {
                    "tema" => e.modal = Some(escolha_de_tema(e)),
                    "destaque" => e.modal = Some(escolha_de_destaque(e)),
                    "botoes" => e.modal = Some(escolha_de_botoes(e)),
                    "sidebar" => {
                        executar(e, "alternar-sidebar");
                        e.modal = Some(Modal::Escolha {
                            titulo,
                            lista: {
                                let mut l = menu_de_personalizacao(e);
                                l.selecionado = 3;
                                l
                            },
                            acao,
                        });
                    }
                    "agente" => e.modal = Some(escolha_de_agente(e)),
                    "salvamento" => executar(e, "salvamento-automatico"),
                    "vim" => executar(e, "modo-vim"),
                    "atalhos" => e.modal = Some(Modal::Atalhos(0)),
                    _ => {}
                },
                AcaoDaEscolha::Tema => trocar_tema(e, &chave),
                AcaoDaEscolha::RespostaDoAgente(i) => super::conversa::acao_na_resposta(e, i, &chave),
                AcaoDaEscolha::Anexar => super::conversa::mudar_anexo(e, &chave, true),
                AcaoDaEscolha::Desanexar => super::conversa::mudar_anexo(e, &chave, false),
                AcaoDaEscolha::AbrirPagina if chave.starts_with("resultado:") => executar(e, &chave),
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
                AcaoDaEscolha::Agentes => e.pedidos.push(Pedido::AbrirPagina(chave)),
                AcaoDaEscolha::Transcluir => super::markdown::inserir_trecho(e, &format!("![[{chave}]]")),
                // Enter liga/desliga o gatilho; a linha de "nenhum" cria.
                AcaoDaEscolha::Gatilhos if chave.starts_with('\u{0}') => {
                    editar_gatilho(e, &gatilho_vazio());
                }
                AcaoDaEscolha::Gatilhos => e.pedidos.push(Pedido::AlternarGatilho(chave)),
                AcaoDaEscolha::Formatar => super::formatar::escolher(e, &chave),
                AcaoDaEscolha::PaginaDaCelula => super::edicao::pagina_escolhida(e, &chave),
                AcaoDaEscolha::AbrirExterno => e.pedidos.push(Pedido::AbrirExterno(chave)),
                AcaoDaEscolha::TirarPasta => {
                    let mut a = e.preferencias.agente.clone().unwrap_or_default();
                    a.pastas_extras.retain(|p| *p != chave);
                    e.preferencias.agente = Some(a);
                    e.pedidos.push(Pedido::GravarPreferencias);
                }
                AcaoDaEscolha::Destaque => {
                    e.preferencias.destaque = chave;
                    let tema = e.preferencias.tema.clone();
                    trocar_tema(e, &tema);
                }
                AcaoDaEscolha::Botoes => {
                    e.preferencias.botoes = chave;
                    let tema = e.preferencias.tema.clone();
                    trocar_tema(e, &tema);
                }
                AcaoDaEscolha::Template if chave.is_empty() => pedir_titulo_da_pagina(e),
                AcaoDaEscolha::Template => {
                    e.modal = Some(Modal::Entrada {
                        titulo: "Nova página".into(),
                        campo: Campo::default(),
                        acao: AcaoDaEntrada::PaginaDeTemplate { template: chave, pasta: None },
                    })
                }
                AcaoDaEscolha::Asset => super::markdown::asset_escolhido(e, &chave),
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
                AcaoDaEscolha::Agente if chave == "novo" => {
                    let novo = anotadinho_core::agente::Adaptador {
                        nome: String::new(),
                        binario: String::new(),
                        args: vec!["{prompt}".into()],
                        timeout_s: anotadinho_core::agente::TIMEOUT_MINIMO_S,
                        ..Default::default()
                    };
                    abrir_formulario_do_agente(e, novo);
                }
                AcaoDaEscolha::Agente if chave.starts_with("meu:") => {
                    if let Some(a) = e.preferencias.agentes.iter().find(|a| Some(a.nome.as_str()) == chave.strip_prefix("meu:")).cloned() {
                        e.aviso = Some(format!("agente: {}", a.nome));
                        e.preferencias.agente = Some(a);
                        e.pedidos.push(Pedido::GravarPreferencias);
                    }
                }
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
                    e.imagem_pendente = None;
                }
                if acao == AcaoDaEscolha::Formatar {
                    super::formatar::retomar(e);
                }
                if acao == AcaoDaEscolha::AbrirPagina {
                    e.alvo_de_busca = None;
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
                if matches!(acao, AcaoDaEntrada::Link | AcaoDaEntrada::CorLivre) {
                    super::formatar::retomar(e);
                }
            }
            "Enter" if acao == AcaoDaEntrada::PaginaDeContexto => {
                let nome = campo.texto.trim().to_string();
                let anexos = e.conversa.as_ref().map(|c| c.anexos.clone()).unwrap_or_default();
                if nome.is_empty() || anexos.is_empty() {
                    e.aviso = Some("preciso de um nome e de pelo menos um anexo".into());
                } else {
                    e.pedidos.push(Pedido::GuardarContexto { titulo: nome, anexos });
                }
            }
            "Enter" if acao == AcaoDaEntrada::LimiteDeAgentes => {
                match campo.texto.trim().parse::<usize>() {
                    Ok(n) => {
                        e.preferencias.limite_de_agentes = n;
                        e.pedidos.push(Pedido::GravarPreferencias);
                        e.aviso = Some(if n == 0 {
                            "sem limite de agentes em paralelo".into()
                        } else {
                            format!("até {n} agente(s) em paralelo; o resto espera na fila")
                        });
                    }
                    // Texto que não é número devolve o campo, em vez de
                    // engolir o que a pessoa digitou.
                    Err(_) => {
                        e.aviso = Some("digite um número".into());
                        e.modal = Some(Modal::Entrada { titulo, campo, acao });
                    }
                }
            }
            "Enter" if matches!(acao, AcaoDaEntrada::MotivoDaRecusaVarias(_)) => {
                let AcaoDaEntrada::MotivoDaRecusaVarias(ids) = acao else { unreachable!() };
                e.pedidos.push(Pedido::DecidirVarias { ids, aplicar: false, motivo: campo.texto.trim().to_string() });
            }
            "Enter" if matches!(acao, AcaoDaEntrada::MotivoDaRecusa(_)) => {
                let AcaoDaEntrada::MotivoDaRecusa(id) = acao else { unreachable!() };
                e.pedidos.push(Pedido::DecidirProposta { id, aplicar: false, motivo: campo.texto.trim().to_string() });
            }
            "Enter" if matches!(acao, AcaoDaEntrada::Vault(_)) => {
                let AcaoDaEntrada::Vault(criar) = acao else { unreachable!() };
                let pasta = campo.texto.trim().to_string();
                if pasta.is_empty() {
                    e.modal = Some(Modal::Entrada { titulo, campo, acao: AcaoDaEntrada::Vault(criar) });
                } else {
                    e.pedidos.push(Pedido::TrocarVault { pasta, criar });
                }
            }
            "Enter" if matches!(acao, AcaoDaEntrada::PastaDoAgente(_)) => {
                let AcaoDaEntrada::PastaDoAgente(extra) = acao else { unreachable!() };
                e.pedidos.push(Pedido::PastaDoAgente { pasta: campo.texto.trim().to_string(), extra });
            }
            "Enter" if acao == AcaoDaEntrada::CorLivre => super::formatar::aplicar_cor_livre(e, &campo.texto),
            "Enter" if acao == AcaoDaEntrada::Link => {
                let url = campo.texto.trim().to_string();
                if url.is_empty() {
                    super::formatar::retomar(e);
                } else {
                    super::formatar::aplicar_link(e, &url);
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
                        AcaoDaEntrada::Imagem | AcaoDaEntrada::Mermaid | AcaoDaEntrada::Link
                        | AcaoDaEntrada::CorLivre
                        | AcaoDaEntrada::PastaDoAgente(_)
                        | AcaoDaEntrada::Vault(_)
                        | AcaoDaEntrada::MotivoDaRecusa(_)
                        | AcaoDaEntrada::MotivoDaRecusaVarias(_)
                        | AcaoDaEntrada::PaginaDeContexto
                        | AcaoDaEntrada::LimiteDeAgentes => return,
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
                R::Fechar if alvo == AlvoDoDetalhe::Imagem => {
                    e.bloco_a_inserir = None;
                    return;
                }
                R::Fechar => return,
                R::Botao("salvar") if alvo == AlvoDoDetalhe::Agente => {
                    if salvar_agente(e, &form) {
                        return;
                    }
                }
                R::Mudou if alvo == AlvoDoDetalhe::Agente => {}
                R::Botao("salvar") if alvo == AlvoDoDetalhe::Permissoes => {
                    let p = anotadinho_core::permissoes::Permissoes {
                        pode: form.lista("pode").into_iter().filter(|x| !x.trim().is_empty()).collect(),
                        nunca: form.lista("nunca").into_iter().filter(|x| !x.trim().is_empty()).collect(),
                    };
                    e.pedidos.push(Pedido::GravarPermissoes(p));
                    return;
                }
                R::Mudou if alvo == AlvoDoDetalhe::Permissoes => {}
                // Aplicar o conteúdo editado (ciclo 411): não é mais o
                // que o agente escreveu, e o registro vai dizer isso.
                R::Botao("aplicar") if matches!(alvo, AlvoDoDetalhe::PropostaEditada { .. }) => {
                    let AlvoDoDetalhe::PropostaEditada { id, .. } = &alvo else { unreachable!() };
                    let conteudo = form.texto("conteudo");
                    if conteudo.trim().is_empty() {
                        e.aviso = Some("conteúdo vazio: recuse a proposta em vez de gravar nada".into());
                        e.modal = Some(Modal::Detalhe { titulo, form, alvo });
                        return;
                    }
                    e.pedidos.push(Pedido::AplicarPropostaEditada { id: id.clone(), conteudo });
                    return;
                }
                R::Mudou if matches!(alvo, AlvoDoDetalhe::PropostaEditada { .. }) => {}
                R::Botao("salvar") if matches!(alvo, AlvoDoDetalhe::Gatilho { .. }) => {
                    if salvar_gatilho(e, &form) {
                        return;
                    }
                    e.modal = Some(Modal::Detalhe { titulo, form, alvo });
                    return;
                }
                R::Botao("remover") if matches!(alvo, AlvoDoDetalhe::Gatilho { .. }) => {
                    let AlvoDoDetalhe::Gatilho { nome } = &alvo else { unreachable!() };
                    e.pedidos.push(Pedido::ApagarGatilho(nome.clone()));
                    return;
                }
                R::Mudou if matches!(alvo, AlvoDoDetalhe::Gatilho { .. }) => {}
                R::Botao("remover") if alvo == AlvoDoDetalhe::Agente => {
                    remover_agente(e);
                    return;
                }
                R::Botao("salvar") if alvo == AlvoDoDetalhe::Teclas => {
                    if super::teclas::salvar(e, &form) {
                        return;
                    }
                }
                R::Botao("padrao") if alvo == AlvoDoDetalhe::Teclas => {
                    super::teclas::padrao(e);
                    return;
                }
                R::Mudou if alvo == AlvoDoDetalhe::Teclas => {}
                R::Botao("inserir") if alvo == AlvoDoDetalhe::Imagem => {
                    if super::markdown::inserir_imagem_do_formulario(e, &form) {
                        return;
                    }
                }
                R::Mudou if alvo == AlvoDoDetalhe::Imagem => {}
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
        Modal::EditorDeTexto { titulo, mut campo, alvo } => {
            match tecla {
                "Escape" => {
                    match &alvo {
                        AlvoDoTexto::Codigo { hospedeiro, inicio, cerca, fecha } => {
                            super::markdown::gravar_codigo(e, hospedeiro, *inicio, cerca, fecha, &campo.texto)
                        }
                    }
                    return;
                }
                "Ctrl+c" => {
                    e.aviso = Some("edição descartada".into());
                    return;
                }
                "Enter" => {
                    campo.tecla("\n");
                }
                "Tab" => {
                    campo.tecla(" ");
                    campo.tecla(" ");
                    campo.tecla(" ");
                    campo.tecla(" ");
                }
                "ArrowUp" | "ArrowDown" => crate::componentes::cursor_vertical(&mut campo, tecla == "ArrowDown"),
                "Home" | "End" => crate::componentes::cursor_na_linha(&mut campo, tecla == "End"),
                outra => {
                    campo.tecla(outra);
                }
            }
            e.modal = Some(Modal::EditorDeTexto { titulo, campo, alvo });
        }
        Modal::Conflito(mut c) => {
            match tecla {
                "j" | "ArrowDown" | "l" | "ArrowRight" | "Tab" => c.opcao = (c.opcao + 1) % 3,
                "k" | "ArrowUp" | "h" | "ArrowLeft" => c.opcao = (c.opcao + 2) % 3,
                "Ctrl+d" | "PageDown" => c.rolagem += 10,
                "Ctrl+u" | "PageUp" => c.rolagem = c.rolagem.saturating_sub(10),
                "Enter" if c.opcao == 0 => c.diff = !c.diff,
                "Enter" if c.opcao == 1 => {
                    e.pedidos.push(Pedido::GravarPorCima { path: c.path, conteudo: c.meu });
                    return;
                }
                "Enter" => {
                    e.pedidos.push(Pedido::AbrirPagina(c.path));
                    e.aviso = Some("recarregada do disco".into());
                    return;
                }
                _ => {}
            }
            e.modal = Some(Modal::Conflito(c));
        }
        Modal::TextoCru { titulo, texto, rolagem } => match tecla {
            "Escape" | "q" | "Enter" => {}
            "j" | "ArrowDown" => e.modal = Some(Modal::TextoCru { titulo, texto, rolagem: rolagem + 1 }),
            "k" | "ArrowUp" => e.modal = Some(Modal::TextoCru { titulo, texto, rolagem: rolagem.saturating_sub(1) }),
            "Ctrl+d" | "PageDown" => e.modal = Some(Modal::TextoCru { titulo, texto, rolagem: rolagem + 10 }),
            "Ctrl+u" | "PageUp" => e.modal = Some(Modal::TextoCru { titulo, texto, rolagem: rolagem.saturating_sub(10) }),
            _ => e.modal = Some(Modal::TextoCru { titulo, texto, rolagem }),
        },
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
            ("Alt+. Alt+,", "embed seguinte / anterior"),
            ("Ctrl+E Ctrl+L", "foco na sidebar / no conteúdo"),
            ("Ctrl+N Ctrl+F", "nova página / nova pasta"),
            ("Ctrl+T Ctrl+B", "alternar tema / sidebar"),
            ("Ctrl+D Ctrl+G Ctrl+U", "hoje / tags / assets"),
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
            ("Ctrl+B Ctrl+T", "negrito / formatar a palavra (inserindo)"),
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
        Modal::EditorDeTexto { titulo, campo, .. } => {
            let area = componentes::area_do_modal(tela, 100, tela.height.saturating_sub(4));
            let dentro = componentes::desenhar_modal(f, area, titulo, "Enter quebra linha · Esc grava · Ctrl+C desiste", t);
            let texto = Style::default().fg(t.var("text-primary"));
            // Uma linha da tela por linha do texto, com o número apagado e
            // o cursor em vídeo inverso.
            let invertido = Style::default().fg(t.var("bg-base")).bg(t.var("text-primary"));
            let chars: Vec<char> = campo.texto.chars().collect();
            let mut linhas: Vec<Line<'static>> = Vec::new();
            let mut atual: Vec<Span<'static>> = vec![Span::styled("  1 ", Style::default().fg(t.var("text-muted")))];
            let mut n: usize = 1;
            let mut linha_do_cursor: usize = 0;
            for (i, ch) in chars.iter().enumerate() {
                if i == campo.cursor {
                    linha_do_cursor = n - 1;
                    atual.push(Span::styled(if *ch == '\n' { " ".to_string() } else { ch.to_string() }, invertido));
                    if *ch != '\n' {
                        continue;
                    }
                }
                if *ch == '\n' {
                    linhas.push(Line::from(std::mem::take(&mut atual)));
                    n += 1;
                    atual = vec![Span::styled(format!("{n:>3} "), Style::default().fg(t.var("text-muted")))];
                } else if i != campo.cursor {
                    atual.push(Span::styled(ch.to_string(), texto));
                }
            }
            if campo.cursor >= chars.len() {
                linha_do_cursor = n - 1;
                atual.push(Span::styled(" ", invertido));
            }
            linhas.push(Line::from(atual));
            let rolagem = linha_do_cursor.saturating_sub(dentro.height.saturating_sub(1) as usize);
            f.render_widget(Paragraph::new(linhas).scroll((rolagem as u16, 0)), dentro);
        }
        Modal::Conflito(c) => {
            let linhas_diff = if c.diff { anotadinho_core::diff::diff_linhas(&c.disco, &c.meu) } else { Vec::new() };
            let altura = if c.diff { tela.height.saturating_sub(4) } else { 9 };
            let area = componentes::area_do_modal(tela, 90, altura);
            let dentro = componentes::desenhar_modal(f, area, "Conflito", "h l escolher · Enter · Ctrl+D rola", t);
            let apagado = Style::default().fg(t.var("text-muted"));
            let mut linhas = vec![
                Line::from(Span::styled(" ⚠ Esta página mudou no disco enquanto você editava.", Style::default().fg(t.var("warning")).add_modifier(Modifier::BOLD))),
                Line::default(),
            ];
            let rotulos = [
                if c.diff { "Esconder a diferença" } else { "Ver a diferença" },
                "Manter o meu",
                "Recarregar (perde o que você escreveu)",
            ];
            let mut botoes = vec![Span::raw(" ")];
            for (i, r) in rotulos.iter().enumerate() {
                let estilo = if i == c.opcao { t.estilo(crate::tema::Realce::Cursor) } else { Style::default().fg(t.var("text-primary")).bg(t.var("bg-elevated")) };
                botoes.push(Span::styled(format!(" {r} "), estilo));
                botoes.push(Span::raw("  "));
            }
            linhas.push(Line::from(botoes));
            if c.diff {
                linhas.push(Line::default());
                linhas.push(Line::from(Span::styled(" - no disco   + o seu", apagado)));
                let w = dentro.width as usize;
                for l in linhas_diff.iter().skip(c.rolagem) {
                    let (marca, estilo) = match l {
                        anotadinho_core::diff::LinhaDiff::Igual { .. } => (" ", apagado),
                        anotadinho_core::diff::LinhaDiff::Removida { .. } => ("-", Style::default().fg(t.var("text-primary")).bg(crate::tema::misturar(t.var("error"), t.var("bg-surface"), 0.16))),
                        anotadinho_core::diff::LinhaDiff::Adicionada { .. } => ("+", Style::default().fg(t.var("text-primary")).bg(crate::tema::misturar(t.var("success"), t.var("bg-surface"), 0.16))),
                    };
                    let texto: String = format!("{marca}{}", l.texto()).chars().take(w).collect();
                    let falta = w.saturating_sub(texto.chars().count());
                    linhas.push(Line::from(vec![Span::styled(texto, estilo), Span::styled(" ".repeat(falta), estilo)]));
                }
            }
            f.render_widget(Paragraph::new(linhas), dentro);
        }
        Modal::TextoCru { titulo, texto, rolagem } => {
            let area = componentes::area_do_modal(tela, 92, tela.height.saturating_sub(4));
            let dentro = componentes::desenhar_modal(f, area, titulo, "j k rolar · Ctrl+D/U de 10 · Esc", t);
            let largura = dentro.width as usize;
            let comum = Style::default().fg(t.var("text-primary"));
            let linhas: Vec<Line> = texto
                .lines()
                .skip(*rolagem)
                .take(dentro.height as usize)
                .map(|l| {
                    // Corta, não quebra: o que importa aqui é a FORMA do
                    // que vai — a linha longa continua sendo uma linha.
                    Line::from(Span::styled(l.chars().take(largura).collect::<String>(), comum))
                })
                .collect();
            f.render_widget(Paragraph::new(linhas), dentro);
        }
        Modal::Visualizar(texto, rolagem) => {
            let area = componentes::area_do_modal(tela, 90, tela.height.saturating_sub(6));
            let dentro = componentes::desenhar_modal(f, area, "Visualizar", "j k rolar · Esc", t);
            // O texto final desenhado como página (ciclo 399), como a
            // visualização da janela.
            let linhas = super::especiais::pagina_renderizada(texto, t, dentro.width.saturating_sub(1) as usize);
            let p = Paragraph::new(linhas).scroll((*rolagem as u16, 0));
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
            Item::novo("⌕", titulo, format!("resultado:{termo}\u{0}{}\u{0}{}", h.path, h.ancora.clone().unwrap_or_default()))
                .com_detalhe(format!("{origem}{}", trecho.chars().take(60).collect::<String>()))
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
    let a = e.preferencias.agente.clone().unwrap_or_default();
    abrir_formulario_do_agente(e, a);
}

fn abrir_formulario_do_agente(e: &mut Estado, a: anotadinho_core::agente::Adaptador) {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    use anotadinho_core::agente::FormatoSaida;
    e.agente_original = Some(a.nome.clone());
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
    // Remover só o que a pessoa criou; preset não se apaga (ciclo 392).
    if !a.nome.is_empty() && e.preferencias.agentes.iter().any(|x| x.nome == a.nome) {
        form.botoes.push(("remover", "Remover".into()));
    }
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
    // Guardado na lista (ciclo 392): renomear é trocar, não duplicar.
    let e_preset = anotadinho_core::agente::Adaptador::presets().iter().any(|p| p.nome == a.nome);
    if let Some(original) = e.agente_original.take().filter(|o| !o.is_empty() && *o != a.nome) {
        e.preferencias.agentes.retain(|x| x.nome != original);
    }
    if !e_preset || e.preferencias.agentes.iter().any(|x| x.nome == a.nome) {
        e.preferencias.agentes.retain(|x| x.nome != a.nome);
        e.preferencias.agentes.push(a.clone());
    }
    e.preferencias.agente = Some(a);
    e.pedidos.push(Pedido::GravarPreferencias);
    true
}

/// "Remover": esquece o agente criado; se era o em uso, volta ao padrão.
fn remover_agente(e: &mut Estado) {
    let Some(nome) = e.agente_original.take() else { return };
    e.preferencias.agentes.retain(|x| x.nome != nome);
    if e.preferencias.agente.as_ref().is_some_and(|a| a.nome == nome) {
        e.preferencias.agente = None;
    }
    e.aviso = Some(format!("agente {nome} removido"));
    e.pedidos.push(Pedido::GravarPreferencias);
}

/// Os resultados no conteúdo chegaram pra barra aberta (ciclo 379): entram
/// depois dos comandos e páginas, se o filtro ainda é aquele.
pub fn resultados_na_paleta(e: &mut Estado, termo: &str, hits: &[anotadinho_core::embed::SearchHit]) {
    let Some(Modal::Paleta(lista)) = e.modal.as_mut() else { return };
    if lista.filtro.as_ref().map(|c| c.texto.trim()) != Some(termo) {
        return;
    }
    lista.itens.retain(|it| !it.chave.starts_with("resultado:"));
    for h in hits {
        let titulo = e.paginas.iter().find(|p| p.path == h.path).map(|p| p.title.clone()).unwrap_or_else(|| h.path.clone());
        let trecho: String = h.snippet.replace("**", "").split_whitespace().collect::<Vec<_>>().join(" ");
        let origem = h.origem.as_ref().map(|o| format!("{o} · ")).unwrap_or_default();
        let ancora = h.ancora.clone().unwrap_or_default();
        lista.itens.push(Item::novo("⌕", titulo, format!("resultado:{termo}\u{0}{}\u{0}{ancora}", h.path)).com_detalhe(format!("{origem}{trecho}")));
    }
}

/// O registro de decisões chegou (ciclo 404): a sequência, do mais novo
/// pro mais velho, com o que foi decidido e por quê.
pub fn mostrar_decisoes(e: &mut Estado, decisoes: &[anotadinho_core::decisao::Decisao]) {
    if decisoes.is_empty() {
        e.aviso = Some("nenhuma decisão registrada ainda".into());
        return;
    }
    let itens = decisoes
        .iter()
        .map(|d| {
            let glifo = match d.acao {
                anotadinho_core::decisao::Acao::Aplicada => "✓",
                anotadinho_core::decisao::Acao::Parcial { .. } => "◐",
                anotadinho_core::decisao::Acao::Editada => "✎",
                anotadinho_core::decisao::Acao::Recusada => "✗",
            };
            let motivo = if d.motivo.is_empty() { String::new() } else { format!(" — {}", d.motivo) };
            Item::novo(glifo, format!("{} · {}", d.acao.rotulo(), d.alvo), d.alvo.clone())
                .com_detalhe(format!("{} · {}{motivo}", d.quando, d.autor))
        })
        .collect();
    e.modal = Some(Modal::Escolha { titulo: "Decisões sobre as propostas".into(), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::AbrirPagina });
}

/// O formulário das permissões do agente (ciclo 405): onde ele pode
/// propor e onde nunca.
pub fn abrir_permissoes(e: &mut Estado, p: &anotadinho_core::permissoes::Permissoes) {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let mut form = Formulario::novo(vec![
        C::novo("pode", "Pode propor em", Valor::Lista(p.pode.clone())).com_dica("pages/specs (vazio: qualquer lugar)"),
        C::novo("nunca", "Nunca", Valor::Lista(p.nunca.clone())).com_dica("journals/"),
    ]);
    form.botoes.push(("salvar", "Salvar no vault".into()));
    e.modal = Some(Modal::Detalhe { titulo: "Onde o agente pode propor".into(), form, alvo: AlvoDoDetalhe::Permissoes });
}

/// O registro de execuções (ciclo 406): quando rodou, quanto durou e como
/// terminou. Enter abre a conversa.
pub fn mostrar_execucoes(e: &mut Estado, execucoes: &[anotadinho_core::execucao::Execucao]) {
    use anotadinho_core::execucao::Fim;
    if execucoes.is_empty() {
        e.aviso = Some("nenhuma execução registrada ainda".into());
        return;
    }
    let itens = execucoes
        .iter()
        .map(|x| {
            let glifo = match x.fim {
                Fim::Respondeu => "✓",
                Fim::Falhou(_) => "✗",
                Fim::Interrompida => "■",
            };
            // A linha é curta de propósito: o detalhe à direita só aparece
            // se couber. Como terminou e quanto durou no rótulo, quando e
            // com quem no detalhe; o resto fica no `execucoes.jsonl`.
            let rotulo = format!("{} · {} · {}s", x.quando, x.fim.rotulo(), x.segundos);
            Item::novo(glifo, rotulo, x.conversa.clone()).com_detalhe(x.agente.clone())
        })
        .collect();
    e.modal = Some(Modal::Escolha { titulo: "Execuções do agente".into(), lista: Lista::filtravel(itens), acao: AcaoDaEscolha::AbrirPagina });
}

fn nome_de_arquivo(path: &str) -> String {
    std::path::Path::new(path).file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| path.to_string())
}

/// O contrato de ferramentas (ciclo 407): o que o agente alcança do
/// vault, com a de escrita marcada. Vem do núcleo, não de IO.
pub fn mostrar_ferramentas(e: &mut Estado) {
    let itens = anotadinho_core::ferramentas::CONTRATO
        .iter()
        .map(|f| {
            let glifo = if f.escreve { "✎" } else { "⌕" };
            let assinatura = f.assinatura();
            Item::novo(glifo, assinatura.clone(), assinatura)
                .com_detalhe(if f.escreve { "escreve · revisão" } else { "leitura" })
        })
        .collect();
    e.modal = Some(Modal::Escolha {
        titulo: "Ferramentas do agente".into(),
        lista: Lista::filtravel(itens),
        acao: AcaoDaEscolha::Mostrar,
    });
}

/// O que o agente está fazendo agora (ciclo 408): o que roda, há quanto
/// tempo, e quem espera vaga. `Ctrl+X`/`x` interrompe tudo.
pub fn mostrar_agentes(e: &mut Estado, rodando: &[(String, String, u64)], esperando: &[String]) {
    if rodando.is_empty() && esperando.is_empty() {
        e.aviso = Some("nenhum agente rodando".into());
        return;
    }
    let mut itens: Vec<Item> = rodando
        .iter()
        .map(|(conversa, agente, segundos)| {
            Item::novo(
                "▶",
                format!("{} · {}", nome_de_arquivo(conversa), anotadinho_core::conversa::duracao_legivel(*segundos)),
                conversa.clone(),
            )
            .com_detalhe(agente.clone())
        })
        .collect();
    for (i, conversa) in esperando.iter().enumerate() {
        itens.push(
            Item::novo("⋯", format!("{} · {}º na fila", nome_de_arquivo(conversa), i + 1), conversa.clone())
                .com_detalhe("esperando"),
        );
    }
    e.modal = Some(Modal::Escolha {
        titulo: format!("Agentes em andamento ({} rodando, {} na fila)", rodando.len(), esperando.len()),
        lista: Lista::filtravel(itens),
        acao: AcaoDaEscolha::Agentes,
    });
}

/// A proposta aberta pra editar antes de aplicar (ciclo 411): o texto
/// proposto num campo de várias linhas.
pub fn editar_proposta(e: &mut Estado, id: &str, alvo: &str, conteudo: &str) {
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let mut form = Formulario::novo(vec![C::novo("conteudo", "Conteúdo", Valor::Texto(conteudo.to_string()))
        .como_multilinha()
        .com_dica("o que vai ser gravado em ".to_string() + alvo)]);
    form.botoes.push(("aplicar", "Aplicar editada".into()));
    e.modal = Some(Modal::Detalhe {
        titulo: format!("Editar antes de aplicar — {alvo}"),
        form,
        alvo: AlvoDoDetalhe::PropostaEditada { id: id.to_string(), alvo: alvo.to_string() },
    });
}

/// Os gatilhos do vault (ciclo 412): a regra, o prompt e se está ligado.
/// Enter liga ou desliga; `o` cria; `dd` apaga.
pub fn mostrar_gatilhos(e: &mut Estado, gatilhos: &[anotadinho_core::gatilho::Gatilho]) {
    let mut itens: Vec<Item> = gatilhos
        .iter()
        .map(|g| {
            let primeira: String = g.prompt.lines().next().unwrap_or("").chars().take(40).collect();
            Item::novo(
                if g.ativo { "◉" } else { "○" },
                format!("{} · {}", g.nome, g.quando.rotulo()),
                g.nome.clone(),
            )
            .com_detalhe(primeira)
        })
        .collect();
    if itens.is_empty() {
        itens.push(Item::novo("+", "nenhum gatilho — o cria um", "\u{0}novo").com_detalhe("vazio"));
    }
    e.modal = Some(Modal::Escolha {
        titulo: "Gatilhos do agente".into(),
        lista: Lista::filtravel(itens),
        acao: AcaoDaEscolha::Gatilhos,
    });
}

/// O formulário de um gatilho (ciclo 412). `nome` vazio é gatilho novo.
pub fn editar_gatilho(e: &mut Estado, g: &anotadinho_core::gatilho::Gatilho) {
    use anotadinho_core::gatilho::Quando;
    use crate::componentes::{CampoDoFormulario as C, Formulario, Valor};
    let (tipo, alvo) = match &g.quando {
        Quando::Mudou { prefixo } => (0, prefixo.clone()),
        Quando::Consulta { de, onde } => (1, if onde.is_empty() { de.clone() } else { format!("{de} {}", onde.join(" ")) }),
        Quando::Diario { hora } => (2, hora.clone()),
    };
    let mut form = Formulario::novo(vec![
        C::novo("nome", "Nome", Valor::Texto(g.nome.clone())).com_dica("revisar-specs"),
        C::novo(
            "quando",
            "Quando",
            Valor::Opcoes(
                vec![
                    ("mudou".into(), "uma página mudar".into()),
                    ("consulta".into(), "uma consulta ter resultado".into()),
                    ("diario".into(), "todo dia".into()),
                ],
                tipo,
            ),
        ),
        C::novo("alvo", "Alvo", Valor::Texto(alvo))
            .com_dica("pages/specs · pages/specs status=rascunho · 07:30"),
        C::novo("prompt", "Prompt", Valor::Texto(g.prompt.clone())).como_multilinha(),
        C::novo("ativo", "Ligado", Valor::Booleano(g.ativo)),
    ]);
    form.botoes.push(("salvar", "Salvar no vault".into()));
    if !g.nome.is_empty() {
        form.botoes.push(("remover", "Apagar".into()));
    }
    e.modal = Some(Modal::Detalhe { titulo: "Gatilho".into(), form, alvo: AlvoDoDetalhe::Gatilho { nome: g.nome.clone() } });
}

/// Um gatilho em branco, pro formulário de criação.
pub fn gatilho_vazio() -> anotadinho_core::gatilho::Gatilho {
    anotadinho_core::gatilho::Gatilho {
        nome: String::new(),
        quando: anotadinho_core::gatilho::Quando::Mudou { prefixo: "pages".into() },
        prompt: String::new(),
        ativo: true,
        ultimo: String::new(),
    }
}

/// Lê o formulário do gatilho. `true` quando salvou.
fn salvar_gatilho(e: &mut Estado, form: &crate::componentes::Formulario) -> bool {
    use anotadinho_core::gatilho::{Gatilho, Quando};
    let nome = form.texto("nome").trim().to_string();
    if nome.is_empty() {
        e.aviso = Some("o gatilho precisa de um nome".into());
        return false;
    }
    let alvo = form.texto("alvo").trim().to_string();
    let quando = match form.escolha("quando").as_str() {
        "diario" => {
            // Hora inválida viraria um gatilho que nunca dispara, ou que
            // dispara sempre.
            if anotadinho_core::date_util::parse_time(&alvo).is_none() {
                e.aviso = Some("hora: use HH:MM".into());
                return false;
            }
            Quando::Diario { hora: alvo }
        }
        "consulta" => {
            let mut partes = alvo.split_whitespace();
            let de = partes.next().unwrap_or("pages").to_string();
            Quando::Consulta { de, onde: partes.map(|x| x.to_string()).collect() }
        }
        _ => Quando::Mudou { prefixo: if alvo.is_empty() { "pages".into() } else { alvo } },
    };
    let prompt = form.texto("prompt").trim().to_string();
    if prompt.is_empty() {
        e.aviso = Some("o gatilho precisa de um prompt".into());
        return false;
    }
    e.pedidos.push(Pedido::GravarGatilho(Gatilho {
        nome,
        quando,
        prompt,
        ativo: form.booleano("ativo"),
        ultimo: String::new(),
    }));
    true
}
