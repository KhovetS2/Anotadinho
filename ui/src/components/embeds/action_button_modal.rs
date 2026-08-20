//! Modal de configuração de um botão do embed de ações (ciclo 163).
//!
//! O ciclo 156 entregou o embed funcionando, mas configurar um botão só
//! dava escrevendo YAML. Pro agente isso é o caminho certo; pra quem
//! monta o painel pelo app, não.
//!
//! A regra que organiza a tela: mostrar SÓ os campos da ação escolhida.
//! São 6 campos possíveis somando todas as ações, e mostrar todos de
//! uma vez foi o que tornou a interface inviável no 156.

use wasm_bindgen::JsCast;
use web_sys::{HtmlInputElement, HtmlSelectElement};
use yew::prelude::*;

use crate::api::{self, PageMeta};
use crate::components::modal::Modal;
use crate::embed::ActionButton;

/// Nomes de ícone oferecidos (os mesmos que `components/icon.rs`
/// desenha — inventar um nome aqui deixaria o botão sem ícone).
const ICONES: [&str; 12] = [
    "zap", "file-text", "folder", "calendar", "search", "check", "edit", "home", "link", "clock",
    "settings", "download",
];

const ACOES: [(&str, &str); 4] = [
    ("new-from-template", "Criar página de template"),
    ("open-page", "Abrir página"),
    ("set-property", "Gravar propriedade"),
    ("run-search", "Buscar"),
];

/// Props do `ActionButtonModal`.
#[derive(Properties, PartialEq, Clone)]
pub struct ActionButtonModalProps {
    /// Botão sendo editado (novo ou existente).
    pub button: ActionButton,
    /// Path do vault — pra listar templates, pastas e páginas.
    pub vault_path: String,
    /// Disparado a cada mudança de campo.
    pub on_change: Callback<ActionButton>,
    /// Fecha o modal.
    pub on_close: Callback<()>,
}

fn valor_input(e: &Event) -> Option<String> {
    e.target()
        .and_then(|t| t.dyn_into::<HtmlInputElement>().ok())
        .map(|el| el.value())
}

fn valor_select(e: &Event) -> Option<String> {
    e.target()
        .and_then(|t| t.dyn_into::<HtmlSelectElement>().ok())
        .map(|el| el.value())
}

/// Modal de configuração de um botão de ação.
#[function_component(ActionButtonModal)]
pub fn action_button_modal(props: &ActionButtonModalProps) -> Html {
    let templates = use_state(Vec::<PageMeta>::new);
    let paginas = use_state(Vec::<PageMeta>::new);
    let pastas = use_state(Vec::<String>::new);

    {
        let (templates, paginas, pastas) = (templates.clone(), paginas.clone(), pastas.clone());
        let vault_path = props.vault_path.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                templates.set(api::list_templates(&vault_path).await.unwrap_or_default());
                paginas.set(api::list_pages(&vault_path).await.unwrap_or_default());
                pastas.set(api::list_folders(&vault_path).await.unwrap_or_default());
            });
            || {}
        });
    }

    // Todo campo segue o mesmo formato: clona o botão, muda um campo,
    // emite. Sem estado local — o dono é a fonte da verdade.
    let atualizar = {
        let on_change = props.on_change.clone();
        let button = props.button.clone();
        std::rc::Rc::new(move |f: Box<dyn Fn(&mut ActionButton)>| {
            let mut novo = button.clone();
            f(&mut novo);
            on_change.emit(novo);
        })
    };

    let campo_texto = |rotulo: &'static str, valor: String, placeholder: &'static str, set: std::rc::Rc<dyn Fn(String)>| {
        let onchange = Callback::from(move |e: Event| {
            if let Some(v) = valor_input(&e) {
                set(v);
            }
        });
        html! {
            <label class="query-settings__row">
                <span class="query-settings__label">{ rotulo }</span>
                <input class="query-settings__input" type="text" value={valor} placeholder={placeholder} {onchange} />
            </label>
        }
    };

    let campo_select = |rotulo: &'static str, valor: String, opcoes: Vec<(String, String)>, set: std::rc::Rc<dyn Fn(String)>| {
        let onchange = Callback::from(move |e: Event| {
            if let Some(v) = valor_select(&e) {
                set(v);
            }
        });
        html! {
            <label class="query-settings__row">
                <span class="query-settings__label">{ rotulo }</span>
                <select class="query-settings__select" {onchange}>
                    { for opcoes.into_iter().map(|(v, label)| html! {
                        <option value={v.clone()} selected={v == valor}>{ label }</option>
                    }) }
                </select>
            </label>
        }
    };

    let b = &props.button;
    let acao_atual = b.action.clone();

    html! {
        <Modal title="Configurar botão" open={true} wide={true} on_close={props.on_close.clone()}>
            <div class="query-settings">
                { campo_texto("Texto", b.label.clone(), "Nova spec", {
                    let atualizar = atualizar.clone();
                    std::rc::Rc::new(move |v: String| atualizar(Box::new(move |b| b.label = v.clone())))
                }) }

                { campo_select("Ícone", b.icon.clone().unwrap_or_default(),
                    std::iter::once((String::new(), "— sem ícone —".to_string()))
                        .chain(ICONES.iter().map(|i| (i.to_string(), i.to_string())))
                        .collect(), {
                    let atualizar = atualizar.clone();
                    std::rc::Rc::new(move |v: String| {
                        let v = v.clone();
                        atualizar(Box::new(move |b| b.icon = if v.is_empty() { None } else { Some(v.clone()) }))
                    })
                }) }

                { campo_select("Estilo", b.variant.clone().unwrap_or_default(),
                    vec![(String::new(), "Fantasma".to_string()), ("primary".to_string(), "Primário".to_string())], {
                    let atualizar = atualizar.clone();
                    std::rc::Rc::new(move |v: String| {
                        let v = v.clone();
                        atualizar(Box::new(move |b| b.variant = if v.is_empty() { None } else { Some(v.clone()) }))
                    })
                }) }

                { campo_select("Ação", acao_atual.clone(),
                    ACOES.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect(), {
                    let atualizar = atualizar.clone();
                    std::rc::Rc::new(move |v: String| {
                        let v = v.clone();
                        atualizar(Box::new(move |b| b.action = v.clone()))
                    })
                }) }

                // Só os campos da ação escolhida (ver doc do módulo).
                if acao_atual == "new-from-template" {
                    { campo_select("Template", b.template.clone().unwrap_or_default(),
                        std::iter::once((String::new(), "— escolha —".to_string()))
                            .chain(templates.iter().map(|t| (t.path.clone(), t.title.clone())))
                            .collect(), {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.template = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                    { campo_select("Pasta", b.folder.clone().unwrap_or_default(),
                        std::iter::once((String::new(), "pages/ (padrão)".to_string()))
                            .chain(pastas.iter().map(|f| (f.clone(), f.clone())))
                            .collect(), {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.folder = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                }

                if acao_atual == "open-page" || acao_atual == "set-property" {
                    { campo_select("Página", b.path.clone().unwrap_or_default(),
                        std::iter::once((String::new(), "— escolha —".to_string()))
                            .chain(paginas.iter().map(|p| (p.path.clone(), p.title.clone())))
                            .collect(), {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.path = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                }

                if acao_atual == "set-property" {
                    { campo_texto("Campo", b.field.clone().unwrap_or_default(), "status", {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.field = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                    { campo_texto("Valor", b.value.clone().unwrap_or_default(), "done", {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.value = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                }

                if acao_atual == "run-search" {
                    { campo_texto("Termo", b.query.clone().unwrap_or_default(), "agent os", {
                        let atualizar = atualizar.clone();
                        std::rc::Rc::new(move |v: String| {
                            let v = v.clone();
                            atualizar(Box::new(move |b| b.query = if v.is_empty() { None } else { Some(v.clone()) }))
                        })
                    }) }
                }

                if !props.button.is_runnable() {
                    <p class="query-embed__erro">
                        { "Falta preencher o alvo — o botão fica desabilitado até lá." }
                    </p>
                }
            </div>
        </Modal>
    }
}
