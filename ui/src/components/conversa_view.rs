//! Painel de conversa com o agente (ciclo 202).
//!
//! Aparece quando a página tem `type: conversa`, do mesmo jeito que
//! `type: kanban` mostra um board — a conversa É uma página, e o
//! markdown dela continua legível fora do app.
//!
//! O que dá fluidez, e é o motivo de existir:
//!
//! - **Contexto automático**: a página que estava aberta antes vai junto
//!   no prompt, sem copiar e colar.
//! - **Histórico como arquivo**: o que se manda pro modelo sai do
//!   próprio `.md`, então a conversa sobrevive a fechar o app, entra no
//!   git e pode ser lida pela consulta como qualquer página.

use wasm_bindgen::JsCast;
use crate::api;
use crate::components::icon::Icon;
use crate::components::modal::Modal;
use anotadinho_core::agente::Adaptador;
use anotadinho_core::agente::EstadoJob;
use anotadinho_core::conversa::{self, Autor, Mensagem};
use anotadinho_core::fluxo::{self, Artefato};
use anotadinho_core::prompt_padrao::{self, PromptPadrao};
use std::collections::BTreeMap;
use yew::prelude::*;

/// Quantas mensagens do histórico vão no prompt. Corta as mais ANTIGAS.
const HISTORICO_NO_PROMPT: usize = 12;

/// Pergunta que uma conversa deve trazer já escrita, e pra QUAL conversa
/// ela é (ciclo 227).
///
/// O caminho não é decoração. Sem ele a pergunta ficava pendurada no app
/// pra sempre — e como o efeito que a injeta reage à troca de página,
/// abrir qualquer outra conversa depois reescrevia o rascunho com o
/// último pedido de planejamento, do nada.
#[derive(Clone, PartialEq, Debug)]
pub struct PerguntaInicial {
    /// Conversa a que a pergunta pertence.
    pub conversa: String,
    /// Texto a escrever no campo.
    pub texto: String,
}

#[derive(Properties, PartialEq, Clone)]
pub struct ConversaViewProps {
    /// Abre a página criada ao promover uma mensagem (ciclo 203).
    #[prop_or_default]
    pub on_page_selected: Callback<api::PageMeta>,
    pub vault_path: String,
    pub page: api::PageMeta,
    /// Página que estava aberta antes — vai como contexto.
    #[prop_or_default]
    pub contexto_path: Option<String>,
    /// Pergunta já escrita ao abrir (ciclo 209), endereçada a uma
    /// conversa específica (ciclo 227).
    #[prop_or_default]
    pub pergunta_inicial: Option<PerguntaInicial>,
    /// Avisa que a pergunta inicial foi escrita no campo, pra quem a
    /// guardou poder esquecê-la (ciclo 227).
    #[prop_or_default]
    pub on_pergunta_consumida: Callback<()>,
}

#[function_component(ConversaView)]
pub fn conversa_view(props: &ConversaViewProps) -> Html {
    let mensagens = use_state(Vec::<Mensagem>::new);
    let rascunho = use_state(String::new);
    {
        // Preenche o campo quando a conversa nasce de um botão que já
        // sabe o que perguntar — e SÓ nela: a pergunta traz o caminho de
        // destino, e quem avisa que consumiu é este efeito, pra ela não
        // sobrar e reaparecer na próxima conversa que a pessoa abrir.
        let rascunho = rascunho.clone();
        let inicial = props.pergunta_inicial.clone();
        let path = props.page.path.clone();
        let consumida = props.on_pergunta_consumida.clone();
        use_effect_with(
            (props.page.path.clone(), props.pergunta_inicial.clone()),
            move |_| {
                if let Some(p) = inicial {
                    if p.conversa == path {
                        rascunho.set(p.texto);
                        consumida.emit(());
                    }
                }
                || ()
            },
        );
    }
    let ocupado = use_state(|| false);
    let erro = use_state(|| None::<String>);
    // Saída que o agente já escreveu, e há quanto tempo ele está nisso
    // (ciclo 213). Numa tarefa de meia hora, uma tela parada é
    // indistinguível de uma tela travada — isto é o sinal de vida.
    let parcial = use_state(String::new);
    let decorrido = use_state(|| 0u64);
    // Qual agente está configurado. É estado, não leitura solta, porque
    // agora dá pra trocar sem sair da conversa (ciclo 214).
    // Se há execução em andamento, num `RefCell` e não em `use_state`.
    //
    // O handle de `use_state` capturado num closure de efeito fica
    // CONGELADO no valor de quando o efeito rodou — o laço de
    // acompanhamento leria `false` pra sempre e nunca voltaria a
    // perguntar. É o mesmo defeito dos ciclos 155, 157 e 201.
    let ativo = use_mut_ref(|| false);
    // A lista de mensagens, pra poder rolar até o fim sozinha.
    let lista_ref = use_node_ref();
    // Se a pessoa está acompanhando o fim da conversa.
    //
    // `RefCell` e não `use_state`: quem lê isto é o efeito de rolagem,
    // e handle de `use_state` capturado em closure fica congelado.
    let colado_no_fim = use_mut_ref(|| true);
    let parcial_ref = use_node_ref();
    let adaptador = use_state(crate::state::load_adaptador);
    let trocando_agente = use_state(|| false);
    // Páginas anexadas, lidas do FRONTMATTER (ciclo 208) — sobrevivem a
    // fechar o app, diferente do contexto em memória do ciclo 202.
    let anexos = use_state(Vec::<String>::new);
    let escolhendo = use_state(|| false);
    let disponiveis = use_state(Vec::<crate::api::PageMeta>::new);
    let filtro_anexo = use_state(String::new);
    // O molde e seus valores ficam separados até a expansão. Sem isso,
    // não haveria como manter o texto inserido como DADO (ciclo 202).
    let prompts = use_state(Vec::<anotadinho_core::PageIndexEntry>::new);
    // A varredura da abertura fica guardada: validar o `contexto:` de um
    // prompt não precisa varrer o vault de novo, é a mesma foto.
    let paginas_vault = use_state(Vec::<anotadinho_core::PageIndexEntry>::new);
    let prompt_ativo = use_state(|| None::<PromptPadrao>);
    let prompt_path = use_state(String::new);
    let valores_prompt = use_state(BTreeMap::<String, String>::new);
    let rascunho_antes_prompt = use_state(String::new);
    let preview_prompt = use_state(|| false);
    // O popover do prompt padrão. Fechado por padrão: o campo de escrever
    // é o que a pessoa veio fazer aqui, e a faixa fixa que existia antes
    // comia uma linha da tela mesmo quando não havia prompt nenhum.
    let prompt_aberto = use_state(|| false);
    // Resposta que virou candidata a execução e está esperando o "sim".
    // Virar execução deixou de ser só criar um arquivo (ciclo 228): agora
    // gasta tempo de modelo, então pergunta antes.
    let confirmar_execucao = use_state(|| None::<String>);
    let carregando_prompts = use_state(|| true);

    // Descoberta usa uma única varredura e aplica simultaneamente pasta e
    // tipo. Recarrega ao trocar de conversa para enxergar páginas novas.
    {
        let prompts = prompts.clone();
        let paginas_vault = paginas_vault.clone();
        let carregando = carregando_prompts.clone();
        let erro = erro.clone();
        let vault_path = props.vault_path.clone();
        use_effect_with(props.page.path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                carregando.set(true);
                match api::scan_vault(&vault_path).await {
                    Ok(paginas) => {
                        prompts.set(prompt_padrao::descobrir(paginas.clone()));
                        paginas_vault.set(paginas);
                    }
                    Err(e) => erro.set(Some(format!("não consegui listar prompts: {e}"))),
                }
                carregando.set(false);
            });
            || ()
        });
    }

    // Carrega a conversa do arquivo.
    {
        let mensagens = mensagens.clone();
        let anexos = anexos.clone();
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        use_effect_with(props.page.path.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(conteudo) = api::read_page(&vault_path, &path).await else { return };
                let (frontmatter, corpo) =
                    anotadinho_core::MarkdownCodec::split_frontmatter_text(&conteudo);
                anexos.set(conversa::contexto_do_frontmatter(frontmatter));
                mensagens.set(conversa::parse(corpo));
            });
            || ()
        });
    }

    // Acompanha a execução desta conversa (ciclo 213).
    //
    // Roda enquanto a tela existir, não enquanto o envio existir: é o
    // que faz uma resposta que chegou com a pessoa noutra página
    // aparecer assim que ela volta. O backend guarda o resultado até
    // alguém buscar, e entrega uma vez só.
    //
    // O intervalo é criado UMA vez por conversa e destruído no cleanup;
    // sem isso, cada renderização deixaria mais um timer vivo.
    {
        let mensagens = mensagens.clone();
        let ocupado = ocupado.clone();
        let erro = erro.clone();
        let parcial = parcial.clone();
        let decorrido = decorrido.clone();
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        let ativo = ativo.clone();
        use_effect_with(props.page.path.clone(), move |_| {
            // Só pergunta quando há motivo: enquanto a conversa está
            // parada, uma ida ao backend por segundo não descobre nada
            // e ainda re-renderiza a tela. Numa conversa grande isso
            // saturou o processo de renderização e ele não voltava mais
            // — as chamadas se acumulavam mais rápido do que eram
            // atendidas.
            //
            // A primeira volta sempre pergunta: é ela que recupera um
            // trabalho que terminou enquanto a pessoa estava noutra
            // página.
            let ja_perguntou = std::rc::Rc::new(std::cell::Cell::new(false));
            let intervalo = gloo_timers::callback::Interval::new(1000, move || {
                if ja_perguntou.get() && !*ativo.borrow() {
                    return;
                }
                ja_perguntou.set(true);
                let (mensagens, ocupado, erro) =
                    (mensagens.clone(), ocupado.clone(), erro.clone());
                let (parcial, decorrido) = (parcial.clone(), decorrido.clone());
                let (vault_path, path) = (vault_path.clone(), path.clone());
                // Clonado FORA do `async move`: dentro, o closure
                // levaria o `Rc` embora e o intervalo só rodaria uma vez.
                let ativo = ativo.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let Ok(Some(estado)) = api::estado_agente(&path).await else {
                        // Sem execução: se a tela ainda se achava
                        // ocupada, é porque o resultado foi entregue
                        // noutra montagem desta mesma página.
                        *ativo.borrow_mut() = false;
                        if *ocupado {
                            ocupado.set(false);
                            parcial.set(String::new());
                        }
                        return;
                    };
                    match estado {
                        EstadoJob::Rodando { segundos, parcial: saida } => {
                            *ativo.borrow_mut() = true;
                            ocupado.set(true);
                            decorrido.set(segundos);
                            parcial.set(saida);
                        }
                        EstadoJob::Concluido { .. } => {
                            *ativo.borrow_mut() = false;
                            // Só RELÊ: a resposta já foi gravada pelo
                            // backend, que é quem tem como fazer isso
                            // mesmo com esta tela fechada. Compor a
                            // mensagem aqui criaria um segundo escritor
                            // e duplicaria a resposta.
                            //
                            // Reler também evita o handle congelado do
                            // closure, que é como mensagem some
                            // (ciclos 155, 157, 201).
                            let atual = api::read_page(&vault_path, &path).await.unwrap_or_default();
                            let (_, corpo) =
                                anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
                            mensagens.set(conversa::parse(corpo));
                            parcial.set(String::new());
                            ocupado.set(false);
                        }
                        EstadoJob::Falhou { erro: e } => {
                            *ativo.borrow_mut() = false;
                            erro.set(Some(e));
                            parcial.set(String::new());
                            ocupado.set(false);
                        }
                        EstadoJob::Cancelado => {
                            *ativo.borrow_mut() = false;
                            erro.set(Some("execução interrompida por você".to_string()));
                            parcial.set(String::new());
                            ocupado.set(false);
                        }
                    }
                });
            });
            move || drop(intervalo)
        });
    }

    // Troca o agente configurado (ciclo 214).
    //
    // A configuração é uma só, do app inteiro — trocar aqui vale pra
    // toda conversa. Fica na conversa porque é onde a pessoa percebe
    // que quer trocar, não num painel escondido de configurações.
    let trocar_agente = {
        let adaptador = adaptador.clone();
        let trocando = trocando_agente.clone();
        Callback::from(move |novo: Adaptador| {
            // Guarda o que SAI antes de gravar o que entra: é isso que
            // faz voltar pro anterior devolver o binário que a pessoa
            // apontou, em vez de recomeçar do preset.
            crate::state::lembrar_adaptador(&adaptador);
            crate::state::save_adaptador(&novo);
            adaptador.set(novo);
            trocando.set(false);
        })
    };

    // Escolhe a pasta de trabalho do agente (ciclo 216).
    //
    // É a pessoa que decide onde o agente pode mexer. Adivinhar pela
    // raiz do git só acerta quando as notas moram dentro do
    // repositório; quem tem o vault num lugar e os repositórios noutro
    // ficava com o agente apontado pro lugar errado, e sem saber disso.
    let escolher_pasta = {
        let adaptador = adaptador.clone();
        Callback::from(move |extra: bool| {
            let adaptador = adaptador.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(Some(pasta)) = api::escolher_pasta().await else { return };
                let mut novo = (*adaptador).clone();
                if extra {
                    if !novo.pastas_extras.contains(&pasta) {
                        novo.pastas_extras.push(pasta);
                    }
                } else {
                    novo.cwd = pasta;
                }
                crate::state::save_adaptador(&novo);
                adaptador.set(novo);
            });
        })
    };

    let tirar_pasta_extra = {
        let adaptador = adaptador.clone();
        Callback::from(move |pasta: String| {
            let mut novo = (*adaptador).clone();
            novo.pastas_extras.retain(|p| *p != pasta);
            crate::state::save_adaptador(&novo);
            adaptador.set(novo);
        })
    };

    // Interrompe a execução em andamento.
    let interromper = {
        let path = props.page.path.clone();
        Callback::from(move |_: MouseEvent| {
            let path = path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let _ = api::cancelar_agente(&path).await;
            });
        })
    };

    // Markdown das mensagens renderizado UMA vez por mudança da lista.
    //
    // Sem isto a conversa reparseava todo o histórico a cada segundo:
    // o acompanhamento da execução (ciclo 213) atualiza o progresso e o
    // tempo decorrido de segundo em segundo, e qualquer mudança de
    // estado re-renderiza o componente inteiro.
    //
    // Numa conversa de 26 KB isso levou o processo de renderização a
    // 85% de CPU e a janela parou de responder — parecia o app travado,
    // e a pessoa reenviava a pergunta, que entrava duplicada no
    // arquivo.
    let renderizadas = use_memo((*mensagens).clone(), |lista: &Vec<Mensagem>| {
        lista
            .iter()
            .map(|m| crate::markdown_render::render(&m.texto))
            .collect::<Vec<String>>()
    });

    // Rola até o fim quando chega coisa nova (ciclo 224).
    //
    // Sem isto, acompanhar uma execução longa exigia ficar arrastando a
    // barra: o progresso crescia embaixo, fora da vista.
    //
    // A dependência é o CONTEÚDO (quantidade de mensagens e tamanho do
    // progresso), nunca a posição de rolagem. Um efeito que reage à
    // própria rolagem e escreve rolagem de volta vira laço, e laço de
    // rolagem trava a janela — ver a nota do ciclo 222.
    {
        let lista_ref = lista_ref.clone();
        let colado_no_fim = colado_no_fim.clone();
        use_effect_with(
            (mensagens.len(), parcial.len(), *ocupado),
            move |_| {
                // Só arrasta quem já estava no fim. Quem subiu pra
                // reler alguma coisa fica onde está — puxar a pessoa de
                // volta no meio da leitura é pior do que não rolar.
                if *colado_no_fim.borrow() {
                    if let Some(el) = lista_ref.cast::<web_sys::HtmlElement>() {
                        el.set_scroll_top(el.scroll_height());
                    }
                }
                || ()
            },
        );
    }

    // A caixa de progresso tem rolagem PRÓPRIA (altura fixa): sem isto,
    // a lista rola até o fim e mesmo assim o que o agente acabou de
    // dizer fica escondido dentro dela.
    {
        let parcial_ref = parcial_ref.clone();
        use_effect_with(parcial.len(), move |_| {
            if let Some(el) = parcial_ref.cast::<web_sys::HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    // Ao trocar de conversa, o fim é o lugar certo pra começar.
    {
        let lista_ref = lista_ref.clone();
        let colado_no_fim = colado_no_fim.clone();
        use_effect_with(props.page.path.clone(), move |_| {
            *colado_no_fim.borrow_mut() = true;
            if let Some(el) = lista_ref.cast::<web_sys::HtmlElement>() {
                el.set_scroll_top(el.scroll_height());
            }
            || ()
        });
    }

    // Descobre se a pessoa saiu do fim.
    let ao_rolar = {
        let colado_no_fim = colado_no_fim.clone();
        Callback::from(move |e: Event| {
            let Some(el) = e.target().and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
            else {
                return;
            };
            // Uma folga: rolagem por linha e arredondamento de subpixel
            // deixam a conta encostar sem bater exato, e sem a folga o
            // acompanhamento desligava sozinho.
            const FOLGA: i32 = 40;
            let no_fim = el.scroll_top() + el.client_height() >= el.scroll_height() - FOLGA;
            *colado_no_fim.borrow_mut() = no_fim;
        })
    };

    // Contexto que aponta pra própria conversa não serve de nada —
    // acontece ao reabrir a mesma página.
    let contexto_path = props
        .contexto_path
        .clone()
        .filter(|p| *p != props.page.path);

    let ha_marcador_pendente = prompt_ativo.as_ref().is_some_and(|prompt| {
        prompt
            .variaveis
            .iter()
            .any(|nome| valores_prompt.get(nome).is_none_or(|v| v.trim().is_empty()))
    });

    // Mandar a pergunta pro agente. Recebe os anexos em vez de lê-los do
    // estado porque quem promove uma execução acabou de criar uma página
    // e precisa que ELA entre no contexto — o handle capturado no render
    // ainda não a conhece.
    let disparar = {
        let mensagens = mensagens.clone();
        let rascunho = rascunho.clone();
        let ocupado = ocupado.clone();
        let erro = erro.clone();
        let adaptador = adaptador.clone();
        let ativo = ativo.clone();
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        Callback::from(move |(pergunta, anexados): (String, Vec<String>)| {
            let pergunta = pergunta.trim().to_string();
            if pergunta.is_empty() || *ocupado {
                return;
            }
            let (mensagens, rascunho, ocupado, erro) =
                (mensagens.clone(), rascunho.clone(), ocupado.clone(), erro.clone());
            let (vault_path, path) = (vault_path.clone(), path.clone());
            let adaptador = (*adaptador).clone();
            // Reacende o laço de acompanhamento, que fica parado
            // enquanto não há trabalho.
            let ativo = ativo.clone();
            *ativo.borrow_mut() = true;
            ocupado.set(true);
            erro.set(None);

            wasm_bindgen_futures::spawn_local(async move {
                let agora = crate::state::agora_legivel();

                // Grava a pergunta ANTES de chamar o agente: se ele
                // falhar ou o app fechar no meio, o que a pessoa
                // escreveu não se perde.
                let minha = Mensagem { autor: Autor::Voce, quando: agora.clone(), texto: pergunta.clone() };
                let mut lista = (*mensagens).clone();
                lista.push(minha.clone());
                mensagens.set(lista.clone());
                rascunho.set(String::new());
                gravar(&vault_path, &path, &lista).await;

                // Lê cada anexo na hora do envio, não ao abrir: a página
                // pode ter mudado desde então, e mandar a versão velha
                // faria o modelo responder sobre o que não existe mais.
                let mut contextos = Vec::new();
                for a in &anexados {
                    if *a == path {
                        continue; // a própria conversa não é contexto dela
                    }
                    if let Ok(c) = api::read_page(&vault_path, a).await {
                        contextos.push(conversa::Contexto { nome: a.clone(), conteudo: c });
                    }
                }
                let prompt = conversa::montar_prompt(
                    &lista[..lista.len() - 1],
                    &pergunta,
                    &contextos,
                    HISTORICO_NO_PROMPT,
                );

                // Só DISPARA. Quem acompanha é o efeito de polling
                // abaixo — inclusive se esta tela for desmontada no
                // meio, porque o processo é do backend.
                if let Err(e) = api::iniciar_agente(&adaptador, &prompt, &vault_path, &path).await {
                    erro.set(Some(e));
                    *ativo.borrow_mut() = false;
                    ocupado.set(false);
                }
            });
        })
    };

    let enviar = {
        let disparar = disparar.clone();
        let rascunho = rascunho.clone();
        let anexos = anexos.clone();
        let ha_marcador_pendente = ha_marcador_pendente;
        Callback::from(move |_: MouseEvent| {
            if ha_marcador_pendente {
                return;
            }
            disparar.emit(((*rascunho).clone(), (*anexos).clone()));
        })
    };

    let on_input = {
        let rascunho = rascunho.clone();
        let prompt_ativo = prompt_ativo.clone();
        let prompt_path = prompt_path.clone();
        let valores_prompt = valores_prompt.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(el) = e.target_dyn_into::<web_sys::HtmlTextAreaElement>() {
                rascunho.set(el.value());
                // Editar o resultado transforma-o em mensagem livre. Os
                // blocos de DADO já visíveis permanecem no texto.
                if prompt_ativo.is_some() {
                    prompt_ativo.set(None);
                    prompt_path.set(String::new());
                    valores_prompt.set(BTreeMap::new());
                }
            }
        })
    };


    // Promove uma resposta em artefato (ciclo 203).
    //
    // É a ponte entre a conversa e o trabalho estruturado: sem ela o
    // fluxo morre no copiar-e-colar, que é onde a maioria das
    // integrações de chat com "criar tarefa" para.
    // Anexar/remover grava no FRONTMATTER na hora — o que a pessoa
    // anexa precisa continuar lá depois de fechar o app.
    let gravar_anexos = {
        let vault_path = props.vault_path.clone();
        let path = props.page.path.clone();
        let anexos = anexos.clone();
        Callback::from(move |lista: Vec<String>| {
            anexos.set(lista.clone());
            let (vault_path, path) = (vault_path.clone(), path.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(atual) = api::read_page(&vault_path, &path).await else { return };
                let novo = reescrever_contexto(&atual, &lista);
                let _ = api::write_page(&vault_path, &path, &novo).await;
            });
        })
    };

    let promover = {
        let vault_path = props.vault_path.clone();
        let conversa_path = props.page.path.clone();
        let on_page_selected = props.on_page_selected.clone();
        let erro = erro.clone();
        let anexos = anexos.clone();
        let disparar = disparar.clone();
        let ocupado = ocupado.clone();
        Callback::from(move |(artefato, texto): (Artefato, String)| {
            let (vault_path, conversa_path) = (vault_path.clone(), conversa_path.clone());
            let (on_page_selected, erro) = (on_page_selected.clone(), erro.clone());
            let anexos = anexos.clone();
            let (disparar, ocupado) = (disparar.clone(), ocupado.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let titulo = fluxo::titulo_sugerido(&texto, 60);
                let hoje = crate::state::agora_legivel();
                let hoje = hoje.split(' ').next().unwrap_or("").to_string();
                let md = fluxo::montar_pagina(
                    artefato,
                    &titulo,
                    &texto,
                    Some(&conversa_path),
                    &hoje,
                );
                let path = format!("{}/{}.md", artefato.pasta(), fluxo::slug_de_titulo(&titulo));
                if let Err(e) = api::write_page(&vault_path, &path, &md).await {
                    erro.set(Some(format!("não consegui criar a página: {e}")));
                    return;
                }

                // Spec e proposta são para LER e decidir: abre a página.
                if artefato != Artefato::Execucao {
                    on_page_selected.emit(api::PageMeta {
                        path,
                        title: titulo,
                        section: "pages".to_string(),
                    });
                    return;
                }

                // Execução é para FAZER. Fica nesta conversa e pede a
                // implementação agora — antes o botão criava o arquivo e
                // ia embora, e a pessoa redigitava o pedido na mão.
                if *ocupado {
                    erro.set(Some(
                        "já tem uma execução em andamento nesta conversa — \
                         espere ela terminar ou interrompa."
                            .to_string(),
                    ));
                    return;
                }
                let mut lista = (*anexos).clone();
                if !lista.contains(&path) {
                    lista.push(path.clone());
                }
                // Grava o contexto e ESPERA antes de disparar. As duas
                // escritas são no mesmo arquivo — o frontmatter aqui, o
                // corpo com a mensagem lá — e soltas ao mesmo tempo uma
                // sobrescreve a outra, porque cada uma lê o arquivo
                // inteiro antes de escrever.
                if let Ok(atual) = api::read_page(&vault_path, &conversa_path).await {
                    let novo = reescrever_contexto(&atual, &lista);
                    if let Err(e) = api::write_page(&vault_path, &conversa_path, &novo).await {
                        erro.set(Some(format!("não consegui anexar a execução: {e}")));
                        return;
                    }
                }
                anexos.set(lista.clone());
                disparar.emit((fluxo::pergunta_de_execucao_da_conversa(&titulo), lista));
            });
        })
    };

    let adicionar_anexo = {
        let anexos = anexos.clone();
        let gravar = gravar_anexos.clone();
        let escolhendo = escolhendo.clone();
        Callback::from(move |alvo: String| {
            let mut lista = (*anexos).clone();
            if !lista.contains(&alvo) {
                lista.push(alvo);
                gravar.emit(lista);
            }
            escolhendo.set(false);
        })
    };

    let remover_anexo = {
        let anexos = anexos.clone();
        let gravar = gravar_anexos.clone();
        Callback::from(move |alvo: String| {
            let lista: Vec<String> = (*anexos).iter().filter(|x| **x != alvo).cloned().collect();
            gravar.emit(lista);
        })
    };

    let filtrar_anexo = {
        let filtro_anexo = filtro_anexo.clone();
        Callback::from(move |e: InputEvent| {
            if let Some(el) = e.target_dyn_into::<web_sys::HtmlInputElement>() {
                filtro_anexo.set(el.value());
            }
        })
    };

    let abrir_seletor = {
        let escolhendo = escolhendo.clone();
        let disponiveis = disponiveis.clone();
        let filtro_limpar = filtro_anexo.clone();
        let vault_path = props.vault_path.clone();
        Callback::from(move |_: MouseEvent| {
            let abrir = !*escolhendo;
            escolhendo.set(abrir);
            filtro_limpar.set(String::new());
            if !abrir {
                return;
            }
            let (disponiveis, vault_path) = (disponiveis.clone(), vault_path.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let paginas = api::scan_vault(&vault_path).await.unwrap_or_default();
                disponiveis.set(
                    paginas
                        .into_iter()
                        .map(|p| crate::api::PageMeta {
                            path: p.path,
                            title: p.title,
                            section: p.section,
                        })
                        .collect(),
                );
            });
        })
    };

    let escolher_prompt = {
        let aberto = prompt_aberto.clone();
        let paginas_vault = paginas_vault.clone();
        let prompt_ativo = prompt_ativo.clone();
        let prompt_path = prompt_path.clone();
        let valores_prompt = valores_prompt.clone();
        let rascunho = rascunho.clone();
        let rascunho_anterior = rascunho_antes_prompt.clone();
        let anexos = anexos.clone();
        let gravar_anexos = gravar_anexos.clone();
        let erro = erro.clone();
        let vault_path = props.vault_path.clone();
        Callback::from(move |escolhido: String| {
            if escolhido.is_empty() {
                aberto.set(false);
                if prompt_ativo.is_some() {
                    rascunho.set((*rascunho_anterior).clone());
                }
                prompt_ativo.set(None);
                prompt_path.set(String::new());
                valores_prompt.set(BTreeMap::new());
                erro.set(None);
                return;
            }

            let paginas = (*paginas_vault).clone();
            let (prompt_ativo, prompt_path, valores_prompt) =
                (prompt_ativo.clone(), prompt_path.clone(), valores_prompt.clone());
            let (rascunho, rascunho_anterior) = (rascunho.clone(), rascunho_anterior.clone());
            let (anexos, gravar_anexos, erro) =
                (anexos.clone(), gravar_anexos.clone(), erro.clone());
            let aberto = aberto.clone();
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let conteudo = match api::read_page(&vault_path, &escolhido).await {
                    Ok(c) => c,
                    Err(e) => {
                        erro.set(Some(format!("não consegui ler o prompt: {e}")));
                        return;
                    }
                };
                let prompt = PromptPadrao::parse(&conteudo);
                let ausentes = prompt
                    .contexto
                    .iter()
                    .filter(|path| !paginas.iter().any(|p| &p.path == *path))
                    .cloned()
                    .collect::<Vec<_>>();
                if !ausentes.is_empty() {
                    erro.set(Some(format!(
                        "contexto do prompt não encontrado: {}",
                        ausentes.join(", ")
                    )));
                    return;
                }

                let base = if prompt_ativo.is_some() {
                    (*rascunho_anterior).clone()
                } else {
                    (*rascunho).clone()
                };
                rascunho_anterior.set(base.clone());
                let mut valores = BTreeMap::new();
                if let Some(primeira) = prompt.variaveis.first() {
                    if !base.trim().is_empty() {
                        valores.insert(primeira.clone(), base.clone());
                    }
                }
                let exibido = if prompt.variaveis.is_empty() {
                    prompt.com_rascunho_ao_final(&base)
                } else {
                    prompt.visualizar_parcial(&valores)
                };
                let mut lista = (*anexos).clone();
                for contexto in &prompt.contexto {
                    if !lista.contains(contexto) {
                        lista.push(contexto.clone());
                    }
                }
                if lista != *anexos {
                    gravar_anexos.emit(lista);
                }
                // Um molde sem variáveis já está pronto, então a lista
                // sai da frente. Com variáveis, os campos vivem DENTRO do
                // popover — fechá-lo aqui esconderia exatamente o que a
                // pessoa precisa preencher em seguida.
                if prompt.variaveis.is_empty() {
                    aberto.set(false);
                }
                valores_prompt.set(valores);
                prompt_path.set(escolhido);
                prompt_ativo.set(Some(prompt));
                rascunho.set(exibido);
                erro.set(None);
            });
        })
    };

    let alternar_prompt = {
        let aberto = prompt_aberto.clone();
        Callback::from(move |_: MouseEvent| aberto.set(!*aberto))
    };
    let fechar_prompt = {
        let aberto = prompt_aberto.clone();
        Callback::from(move |_: MouseEvent| aberto.set(false))
    };
    // O botão mostra o prompt em uso, não um rótulo genérico: é o único
    // sinal de que o texto no campo veio de um molde.
    let rotulo_do_prompt = prompts
        .iter()
        .find(|p| p.path == *prompt_path)
        .map(|p| p.title.clone())
        .unwrap_or_else(|| "Prompt padrão".to_string());

    let abrir_preview = {
        let preview = preview_prompt.clone();
        Callback::from(move |_: MouseEvent| preview.set(true))
    };
    let fechar_preview = {
        let preview = preview_prompt.clone();
        Callback::from(move |_: ()| preview.set(false))
    };

    html! {
        <main class="conversa">
            <header class="conversa__topo">
                <h2 class="conversa__titulo">{ &props.page.title }</h2>
                <div class="conversa__agente-caixa">
                    <button class="conversa__agente" title={format!(
                            "{} · trabalha em {} — clique pra trocar",
                            adaptador.binario,
                            if adaptador.cwd.trim().is_empty() {
                                "raiz do projeto".to_string()
                            } else {
                                adaptador.cwd.clone()
                            })}
                        onclick={{
                            let t = trocando_agente.clone();
                            let aberto = *trocando_agente;
                            Callback::from(move |_: MouseEvent| t.set(!aberto))
                        }}>
                        <Icon name="zap" />{ adaptador.nome.clone() }
                    </button>
                    if *trocando_agente {
                        <div class="conversa__agentes">
                            { for crate::state::opcoes_de_agente().into_iter().map(|preset| {
                                let atual = preset.nome == adaptador.nome;
                                let trocar = trocar_agente.clone();
                                let escolhido = preset.clone();
                                html! {
                                    <button class={classes!("conversa__agente-op", atual.then_some("conversa__agente-op--atual"))}
                                        onclick={Callback::from(move |_: MouseEvent| trocar.emit(escolhido.clone()))}
                                        title={preset.binario.clone()}>
                                        <Icon name="zap" />{ preset.nome.clone() }
                                        if atual { <span class="conversa__agente-marca">{ "•" }</span> }
                                    </button>
                                }
                            }) }
                            <div class="conversa__pastas">
                                <button class="conversa__pasta-btn" onclick={{
                                        let e = escolher_pasta.clone();
                                        Callback::from(move |_: MouseEvent| e.emit(false))
                                    }}
                                    title="Onde o agente trabalha — e a única pasta onde ele pode escrever">
                                    <Icon name="folder" />
                                    { if adaptador.cwd.trim().is_empty() {
                                        "trabalha na raiz do projeto".to_string()
                                      } else {
                                        nome_curto(&adaptador.cwd)
                                      } }
                                </button>
                                { for adaptador.pastas_extras.iter().map(|pasta| {
                                    let tirar = tirar_pasta_extra.clone();
                                    let alvo = pasta.clone();
                                    html! {
                                        <span class="conversa__pasta-extra" title={pasta.clone()}>
                                            { nome_curto(pasta) }
                                            <button class="conversa__anexo-x"
                                                onclick={Callback::from(move |_: MouseEvent| tirar.emit(alvo.clone()))}
                                                title="Tirar do alcance do agente">{ "×" }</button>
                                        </span>
                                    }
                                }) }
                                if !adaptador.arg_pasta_extra.trim().is_empty() {
                                    <button class="conversa__pasta-btn" onclick={{
                                            let e = escolher_pasta.clone();
                                            Callback::from(move |_: MouseEvent| e.emit(true))
                                        }}
                                        title="Outro repositório que o agente também precisa alcançar">
                                        { "+ pasta" }
                                    </button>
                                }
                            </div>
                        </div>
                    }
                </div>
                <button class="btn btn--ghost btn--xs conversa__anexar" onclick={abrir_seletor}
                    title="Anexar páginas que o modelo deve consultar">
                    <Icon name="paperclip" />{ format!("{} anexo(s)", anexos.len()) }
                </button>
            </header>

            if !anexos.is_empty() {
                <div class="conversa__anexos">
                    { for anexos.iter().map(|a| {
                        let remover = remover_anexo.clone();
                        let alvo = a.clone();
                        html! {
                            <span class="conversa__anexo" title={alvo.clone()}>
                                { nome_curto(a) }
                                <button class="conversa__anexo-x"
                                    onclick={Callback::from(move |_: MouseEvent| remover.emit(alvo.clone()))}
                                    title="Tirar do contexto">{ "×" }</button>
                            </span>
                        }
                    }) }
                </div>
            }

            if *escolhendo {
                <div class="conversa__seletor">
                    <input class="input input--sm conversa__seletor-busca" type="text"
                        placeholder="Filtrar páginas..." value={(*filtro_anexo).clone()}
                        oninput={filtrar_anexo} />
                    { for disponiveis.iter().filter(|p| {
                        // Sem filtro a lista é inútil com 200+ páginas: as
                        // 40 primeiras são todas do mesmo prefixo.
                        let f = filtro_anexo.trim().to_lowercase();
                        f.is_empty()
                            || p.title.to_lowercase().contains(&f)
                            || p.path.to_lowercase().contains(&f)
                    }).take(30).map(|p| {
                        let add = adicionar_anexo.clone();
                        let alvo = p.path.clone();
                        let ja = anexos.contains(&p.path);
                        html! {
                            <button class="conversa__seletor-item btn btn--ghost btn--xs" disabled={ja}
                                onclick={Callback::from(move |_: MouseEvent| add.emit(alvo.clone()))}>
                                { &p.title }
                            </button>
                        }
                    }) }
                </div>
            }

            <div class="conversa__mensagens" ref={lista_ref} onscroll={ao_rolar}>
                if mensagens.is_empty() {
                    <p class="conversa__vazia">
                        { "Pergunte algo, peça uma spec, ou mande analisar a página aberta." }
                    </p>
                }
                { for mensagens.iter().enumerate().map(|(i, m)| {
                    let classe = match m.autor {
                        Autor::Voce => "conversa__msg conversa__msg--voce",
                        Autor::Agente => "conversa__msg conversa__msg--agente",
                    };
                    html! {
                        <article class={classe}>
                            <header class="conversa__msg-topo">
                                <span class="conversa__msg-autor">{ m.autor.slug() }</span>
                                <span class="conversa__msg-quando">{ &m.quando }</span>
                            </header>
                            <div class="conversa__msg-corpo">
                                { Html::from_html_unchecked(
                                    renderizadas.get(i).cloned().unwrap_or_default().into()) }
                            </div>
                            if m.autor == Autor::Agente {
                                <div class="conversa__msg-acoes">
                                    { for [Artefato::Spec, Artefato::Proposta, Artefato::Execucao].into_iter().map(|a| {
                                        let promover = promover.clone();
                                        let pedir = confirmar_execucao.clone();
                                        let texto = m.texto.clone();
                                        // Execução passa pela confirmação: ela dispara
                                        // o agente, não só cria arquivo.
                                        let onclick = Callback::from(move |_: MouseEvent| {
                                            if a == Artefato::Execucao {
                                                pedir.set(Some(texto.clone()));
                                            } else {
                                                promover.emit((a, texto.clone()));
                                            }
                                        });
                                        let dica = if a == Artefato::Execucao {
                                            "Criar a execução e pedir a implementação agora".to_string()
                                        } else {
                                            format!("Criar uma {} a partir desta resposta", a.label())
                                        };
                                        html! {
                                            <button class="btn btn--ghost btn--xs" {onclick} title={dica}>
                                                { format!("virar {}", a.label().to_lowercase()) }
                                            </button>
                                        }
                                    }) }
                                </div>
                            }
                        </article>
                    }
                }) }
                if *ocupado {
                    <div class="conversa__trabalhando">
                        <p class="conversa__pensando">
                            <span class="spinner"></span>
                            { format!(" pensando há {}", duracao_legivel(*decorrido)) }
                            <button class="btn btn--ghost btn--xs conversa__parar"
                                onclick={interromper.clone()}
                                title="Matar o processo do agente agora">
                                <Icon name="x" />{ " interromper" }
                            </button>
                        </p>
                        if !parcial.is_empty() {
                            // A saída CRUA, sem markdown: ela chega
                            // pela metade, e renderizar markdown
                            // incompleto pisca a tela a cada linha.
                            <pre class="conversa__parcial" ref={parcial_ref.clone()}>
                                { (*parcial).clone() }
                            </pre>
                        }
                    </div>
                }
                if let Some(e) = &*erro {
                    <p class="conversa__erro">{ e }</p>
                }
            </div>

            <footer class="conversa__compositor">
                <textarea class="conversa__campo" rows="4"
                    placeholder="Escreva e mande. Shift+Enter quebra linha."
                    value={(*rascunho).clone()}
                    oninput={on_input}
                    disabled={*ocupado} />
                if ha_marcador_pendente {
                    <p class="conversa__prompt-pendente">
                        { "Preencha todos os marcadores antes de visualizar ou enviar." }
                    </p>
                }
                <div class="conversa__acoes">
                    // Sem prompt no vault não há o que escolher, e um
                    // botão que abre uma lista vazia é pior que botão
                    // nenhum.
                    if !prompts.is_empty() {
                        <div class="conversa__prompt" data-nav-group="prompt-padrao">
                            <button class="btn btn--ghost btn--sm conversa__prompt-botao"
                                onclick={alternar_prompt} disabled={*ocupado}
                                data-nav-item="true"
                                title="Começar de um prompt padrão do vault">
                                <Icon name="file-text" />
                                { rotulo_do_prompt }
                                <Icon name={if *prompt_aberto { "chevron-down" } else { "chevron-up" }} />
                            </button>
                            if *prompt_aberto {
                                <div class="conversa__prompt-popover">
                                    <ul class="conversa__prompt-lista">
                                        <li>
                                            <button class="conversa__prompt-opcao" data-nav-item="true"
                                                onclick={{
                                                    let escolher = escolher_prompt.clone();
                                                    Callback::from(move |_: MouseEvent| escolher.emit(String::new()))
                                                }}>
                                                { "Nenhum — escrever do zero" }
                                            </button>
                                        </li>
                                        { for prompts.iter().map(|p| {
                                            let escolher = escolher_prompt.clone();
                                            let alvo = p.path.clone();
                                            let atual = *prompt_path == p.path;
                                            html! {
                                                <li>
                                                    <button class={classes!("conversa__prompt-opcao",
                                                        atual.then_some("conversa__prompt-opcao--atual"))}
                                                        data-nav-item="true"
                                                        onclick={Callback::from(move |_: MouseEvent| escolher.emit(alvo.clone()))}>
                                                        { &p.title }
                                                    </button>
                                                </li>
                                            }
                                        }) }
                                    </ul>
                                    if let Some(prompt) = &*prompt_ativo {
                                        if !prompt.variaveis.is_empty() {
                                            <div class="conversa__prompt-campos">
                                                { for prompt.variaveis.iter().map(|nome| {
                                                    let prompt = prompt.clone();
                                                    let nome_atual = nome.clone();
                                                    let valor = valores_prompt.get(nome).cloned().unwrap_or_default();
                                                    let valores = valores_prompt.clone();
                                                    let rascunho = rascunho.clone();
                                                    html! {
                                                        <label class="conversa__prompt-campo">
                                                            <span>{ format!("{{{{{nome}}}}}") }</span>
                                                            <input class="input input--sm" value={valor}
                                                                placeholder="Preencha antes de enviar"
                                                                data-nav-item="true"
                                                                oninput={Callback::from(move |e: InputEvent| {
                                                                    let Some(input) = e.target_dyn_into::<web_sys::HtmlInputElement>() else { return };
                                                                    let mut novos = (*valores).clone();
                                                                    novos.insert(nome_atual.clone(), input.value());
                                                                    rascunho.set(prompt.visualizar_parcial(&novos));
                                                                    valores.set(novos);
                                                                })} />
                                                        </label>
                                                    }
                                                }) }
                                            </div>
                                        }
                                    }
                                    <div class="conversa__prompt-rodape">
                                        <button class="btn btn--ghost btn--xs"
                                            onclick={abrir_preview}
                                            disabled={rascunho.trim().is_empty() || ha_marcador_pendente}
                                            data-nav-item="true" title="Visualizar o texto final sem enviar">
                                            { "Visualizar" }
                                        </button>
                                        <button class="btn btn--ghost btn--xs" data-nav-item="true"
                                            onclick={fechar_prompt}>{ "Fechar" }</button>
                                    </div>
                                </div>
                            }
                        </div>
                    }
                    <span class="conversa__acoes-folga"></span>
                    if *ocupado {
                        <button class="btn" onclick={interromper}
                            title="Matar o processo do agente agora">{ "Parar" }</button>
                    } else {
                        <button class="btn btn--primary" onclick={enviar}
                            disabled={rascunho.trim().is_empty() || ha_marcador_pendente}
                            data-nav-item="true">{ "Enviar" }</button>
                    }
                </div>
            </footer>
            <Modal title="Pedir a implementação?" open={confirmar_execucao.is_some()}
                on_close={{
                    let pedir = confirmar_execucao.clone();
                    Callback::from(move |_: ()| pedir.set(None))
                }}>
                <p>
                    { "Isto cria a página de execução a partir desta resposta, anexa \
                       ela à conversa e pede ao agente para implementar agora, aqui \
                       mesmo." }
                </p>
                <div class="modal__actions">
                    <button class="btn" onclick={{
                        let pedir = confirmar_execucao.clone();
                        Callback::from(move |_: MouseEvent| pedir.set(None))
                    }}>{ "Cancelar" }</button>
                    <button class="btn btn--primary conversa__confirmar-execucao" onclick={{
                        let pedir = confirmar_execucao.clone();
                        let promover = promover.clone();
                        Callback::from(move |_: MouseEvent| {
                            if let Some(texto) = (*pedir).clone() {
                                promover.emit((Artefato::Execucao, texto));
                            }
                            pedir.set(None);
                        })
                    }}>{ "Criar e pedir" }</button>
                </div>
            </Modal>
            <Modal title="Visualização do prompt final" open={*preview_prompt}
                on_close={fechar_preview} wide=true>
                <pre class="conversa__prompt-final">{ (*rascunho).clone() }</pre>
                <div class="modal__actions">
                    <button class="btn" onclick={{
                        let preview = preview_prompt.clone();
                        Callback::from(move |_: MouseEvent| preview.set(false))
                    }}>{ "Fechar" }</button>
                </div>
            </Modal>
        </main>
    }
}

/// Reescreve o corpo da página com as mensagens, preservando o
/// frontmatter — que é onde vivem `type: conversa` e o que mais a pessoa
/// tiver posto lá.
async fn gravar(vault_path: &str, path: &str, mensagens: &[Mensagem]) {
    let atual = api::read_page(vault_path, path).await.unwrap_or_default();
    let (frontmatter, _) = anotadinho_core::MarkdownCodec::split_frontmatter_text(&atual);
    let corpo = conversa::serializar(mensagens);
    let novo = if frontmatter.is_empty() { corpo } else { format!("{frontmatter}\n{corpo}") };
    let _ = api::write_page(vault_path, path, &novo).await;
}

/// Nome curto pra mostrar no chip — o path inteiro não cabe.
fn nome_curto(path: &str) -> String {
    std::path::Path::new(path)
        .file_stem()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Reescreve só a lista `contexto:` do frontmatter, preservando o resto.
///
/// Não usa serialização do `Frontmatter` inteiro de propósito: isso
/// reordenaria os campos e mexeria em coisas que a pessoa escreveu à
/// mão, num arquivo que ela também edita fora do app.
fn reescrever_contexto(conteudo: &str, lista: &[String]) -> String {
    let (frontmatter, corpo) = anotadinho_core::MarkdownCodec::split_frontmatter_text(conteudo);
    let mut linhas: Vec<String> = Vec::new();
    let mut pulando = false;
    for linha in frontmatter.lines() {
        if linha.starts_with("contexto:") {
            pulando = true;
            continue;
        }
        if pulando {
            if linha.starts_with("- ") || linha.starts_with("  ") {
                continue;
            }
            pulando = false;
        }
        // A linha de fecho do bloco entra depois, junto da lista nova.
        if linha.trim() == "---" && linhas.iter().any(|l| l.trim() == "---") {
            continue;
        }
        linhas.push(linha.to_string());
    }
    // Tira o `---` de abertura pra remontar de forma previsível.
    let corpo_fm: Vec<String> = linhas
        .into_iter()
        .filter(|l| l.trim() != "---")
        .collect();

    let mut fm = String::from("---\n");
    for l in corpo_fm {
        fm.push_str(&l);
        fm.push('\n');
    }
    if !lista.is_empty() {
        fm.push_str("contexto:\n");
        for c in lista {
            fm.push_str(&format!("- {}\n", anotadinho_core::markdown::escapar_escalar_yaml(c)));
        }
    }
    fm.push_str("---\n");
    format!("{fm}{corpo}")
}

/// "12s", "3 min", "1h04" — o suficiente pra saber se vale esperar.
fn duracao_legivel(segundos: u64) -> String {
    match segundos {
        s if s < 60 => format!("{s}s"),
        s if s < 3600 => format!("{} min", s / 60),
        s => format!("{}h{:02}", s / 3600, (s % 3600) / 60),
    }
}

#[cfg(test)]
mod tests {
    use super::duracao_legivel;

    #[test]
    fn duracao_muda_de_unidade_nos_limites() {
        assert_eq!(duracao_legivel(0), "0s");
        assert_eq!(duracao_legivel(59), "59s");
        assert_eq!(duracao_legivel(60), "1 min");
        assert_eq!(duracao_legivel(3599), "59 min");
        assert_eq!(duracao_legivel(3600), "1h00");
        assert_eq!(duracao_legivel(3900), "1h05");
    }
}
