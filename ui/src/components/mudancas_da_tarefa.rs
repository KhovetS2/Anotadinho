//! O que a tarefa mudou no repositório do projeto (ciclo 518), na revisão.
//!
//! A revisão era ler o relato do agente; o código ficava fora do app, no
//! `git diff` de quem lembrasse de rodar. Aqui vem o mesmo que
//! `ipc::handle_mudancas_da_tarefa` devolve pra TUI e pro CLI: os arquivos
//! e o diff desde o commit de onde a tarefa partiu.

use crate::api::Mudancas;
use yew::prelude::*;

use crate::api;
use crate::components::modal::Modal;

/// O modal das mudanças de uma tarefa.
#[derive(Properties, PartialEq, Clone)]
pub struct MudancasDaTarefaProps {
    /// O vault aberto.
    pub vault_path: String,
    /// O caminho da tarefa.
    pub caminho: String,
    /// O título do modal ("003 Listar os livros").
    pub titulo: String,
    /// Fechar.
    pub on_fechar: Callback<()>,
}

#[derive(Clone, PartialEq)]
enum Carga {
    Lendo,
    Pronta(Mudancas),
    Erro(String),
}

/// A classe de uma linha do diff unificado.
fn classe_da_linha(l: &str) -> &'static str {
    if l.starts_with("diff --git") || l.starts_with("+++ ") || l.starts_with("--- ") || l == "new file" {
        "mudancas__l mudancas__l--arquivo"
    } else if l.starts_with("@@") {
        "mudancas__l mudancas__l--trecho"
    } else if l.starts_with('+') {
        "mudancas__l mudancas__l--entra"
    } else if l.starts_with('-') {
        "mudancas__l mudancas__l--sai"
    } else {
        "mudancas__l"
    }
}

fn nome_do_estado(e: &str) -> &'static str {
    match e {
        "A" => "novo",
        "?" => "novo (fora do git)",
        "D" => "apagado",
        "R" => "renomeado",
        _ => "mudado",
    }
}

/// O modal: cabeçalho, arquivos e diff.
#[function_component(MudancasDaTarefa)]
pub fn mudancas_da_tarefa(props: &MudancasDaTarefaProps) -> Html {
    let carga = use_state(|| Carga::Lendo);
    {
        let (carga, v, c) = (carga.clone(), props.vault_path.clone(), props.caminho.clone());
        use_effect_with(props.caminho.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                carga.set(match api::mudancas_da_tarefa(&v, &c).await {
                    Ok(m) => Carga::Pronta(m),
                    Err(e) => Carga::Erro(e),
                });
            });
            || ()
        });
    }
    let fechar = props.on_fechar.clone();
    let corpo = match &*carga {
        Carga::Lendo => html! { <p class="mudancas__estado">{ "Lendo o repositório…" }</p> },
        Carga::Erro(e) => html! { <p class="mudancas__erro" role="alert">{ e }</p> },
        Carga::Pronta(m) => {
            let base = m.base.as_deref().map(|b| b.chars().take(10).collect::<String>()).unwrap_or_else(|| "—".into());
            html! {
                <>
                    <p class="mudancas__cabeca">
                        { "desde " }<code>{ base }</code>
                        if let Some(b) = &m.branch {
                            { " · branch " }<code>{ b }</code>
                        }
                        { format!(" · {} arquivo(s)", m.arquivos.len()) }
                    </p>
                    if let Some(a) = &m.aviso {
                        <p class="mudancas__aviso">{ a }</p>
                    }
                    if m.arquivos.is_empty() {
                        <p class="mudancas__estado">{ "Nada mudou no repositório desde que a tarefa começou." }</p>
                    } else {
                        <ul class="mudancas__arquivos">
                            { for m.arquivos.iter().map(|a| html! {
                                <li class="mudancas__arquivo">
                                    <span class={classes!("mudancas__selo", format!("mudancas__selo--{}", if a.estado == "?" { "novo".to_string() } else { a.estado.to_lowercase() }))}>
                                        { nome_do_estado(&a.estado) }
                                    </span>
                                    <code>{ &a.caminho }</code>
                                </li>
                            }) }
                        </ul>
                        <pre class="mudancas__diff">
                            { for m.diff.lines().map(|l| html! { <div class={classe_da_linha(l)}>{ if l.is_empty() { " " } else { l } }</div> }) }
                        </pre>
                    }
                </>
            }
        }
    };
    html! {
        <Modal title={format!("Mudanças — {}", props.titulo)} open={true} wide={true} on_close={fechar.clone()} classe={classes!("mudancas")}>
            { corpo }
            <div class="modal__actions">
                <button type="button" class="btn btn--ghost" data-nav-item="mudancas-fechar" onclick={fechar.reform(|_| ())}>{ "Fechar" }</button>
            </div>
        </Modal>
    }
}
