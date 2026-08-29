//! Componente de estado vazio.
//!
//! Antes do ciclo 233 daqui só saía "Abrir vault", e apontar para uma
//! pasta vazia levava a um app sem sidebar, sem template e sem nenhum
//! sinal do que fazer. Agora dá pra CRIAR: a pasta nasce com estrutura,
//! templates, padrões, prompts e um guia.

use yew::prelude::*;

#[derive(Properties, PartialEq, Clone)]
pub struct EmptyStateProps {
    pub on_vault_selected: Callback<String>,
}

#[function_component(EmptyState)]
pub fn empty_state(props: &EmptyStateProps) -> Html {
    let erro = use_state(|| None::<String>);
    let ocupado = use_state(|| false);

    let abrir = {
        let on_vault_selected = props.on_vault_selected.clone();
        Callback::from(move |_: MouseEvent| {
            let on_vault_selected = on_vault_selected.clone();
            wasm_bindgen_futures::spawn_local(async move {
                if let Ok(Some(path)) = crate::api::open_folder_dialog().await {
                    on_vault_selected.emit(path);
                }
            });
        })
    };

    // Escolher a pasta e semear. A mesma ação serve pra pasta nova e pra
    // pasta que já tem coisa dentro: a semente nunca sobrescreve, só
    // completa o que falta.
    let criar = {
        let on_vault_selected = props.on_vault_selected.clone();
        let erro = erro.clone();
        let ocupado = ocupado.clone();
        Callback::from(move |_: MouseEvent| {
            let on_vault_selected = on_vault_selected.clone();
            let (erro, ocupado) = (erro.clone(), ocupado.clone());
            wasm_bindgen_futures::spawn_local(async move {
                let Ok(Some(path)) = crate::api::open_folder_dialog().await else {
                    return;
                };
                ocupado.set(true);
                erro.set(None);
                match crate::api::criar_vault(&path).await {
                    Ok(_) => on_vault_selected.emit(path),
                    Err(e) => erro.set(Some(e)),
                }
                ocupado.set(false);
            });
        })
    };

    html! {
        <div class="empty-state">
            <div class="empty-state__inner">
                <h1 class="empty-state__title">{ "Anotadinho" }</h1>
                <p class="empty-state__message">
                    { "Abra uma pasta que já é um vault, ou escolha uma pasta \
                       pra começar do zero." }
                </p>
                <div class="empty-state__acoes">
                    <button class="btn btn--primary" onclick={abrir} disabled={*ocupado}>
                        { "Abrir vault" }
                    </button>
                    <button class="btn empty-state__criar" onclick={criar} disabled={*ocupado}>
                        { if *ocupado { "Preparando…" } else { "Criar vault novo" } }
                    </button>
                </div>
                <p class="empty-state__nota">
                    { "Criar deixa a pasta com estrutura, modelos, padrões, prompts \
                       e um guia. Nada que já existir é sobrescrito." }
                </p>
                if let Some(e) = &*erro {
                    <p class="empty-state__erro">{ e }</p>
                }
            </div>
        </div>
    }
}
