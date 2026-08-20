//! Grade de imagens do vault (`{{ type: "gallery" }}`).
//!
//! Imagem numa nota hoje entra uma a uma como `<img>` solto (via `/img`
//! ou colar, ciclo 118), empilhada em tamanho cheio — ruim de ler numa
//! nota de referência visual (moodboard, screenshots de um bug). Aqui
//! elas viram grade com legenda, e o clique abre em tamanho grande.
//!
//! Resolução de path: o `.md` guarda `assets/foto.png` (relativo ao
//! vault, que é o que sobrevive a mover o vault de lugar); o WebView
//! não consegue ler o disco direto, então cada item é resolvido pra
//! data URL por `api::read_asset_data_url` e memorizado num mapa —
//! mesma travessia que o editor faz pras imagens do markdown.

use std::collections::HashMap;

use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::prelude::*;

use crate::api;
use crate::components::icon::Icon;
use crate::components::modal::Modal;
use crate::embed::{GalleryEmbedData, GallerySize};

/// Extensões tratadas como imagem no picker.
const IMAGE_EXTS: [&str; 6] = ["png", "jpg", "jpeg", "gif", "svg", "webp"];

fn is_image(path: &str) -> bool {
    let lower = path.to_lowercase();
    IMAGE_EXTS.iter().any(|ext| lower.ends_with(&format!(".{ext}")))
}

fn is_external(path: &str) -> bool {
    path.starts_with("http://") || path.starts_with("https://") || path.starts_with("data:")
}

/// Props do `InlineGallery`.
#[derive(Properties, PartialEq, Clone)]
pub struct InlineGalleryProps {
    /// Itens e configuração da grade.
    pub data: GalleryEmbedData,
    /// Path do vault (pra listar e resolver assets).
    pub vault_path: String,
    /// Disparado quando itens, legenda, ordem, colunas ou tamanho mudam.
    pub on_change: Callback<GalleryEmbedData>,
    /// Id do grupo de navegação por teclado deste embed (ciclo 165).
    /// Vem do editor e é ÚNICO por segmento — dois embeds do mesmo tipo
    /// na mesma página não podem compartilhar grupo, senão as setas
    /// andariam pelos controles dos dois de uma vez.
    pub nav_group: String,
}

/// Galeria inline.
#[function_component(InlineGallery)]
pub fn inline_gallery(props: &InlineGalleryProps) -> Html {
    // path relativo → data URL. Cresce sob demanda; itens externos
    // (http/data) nunca entram aqui, vão direto no `src`.
    let resolved = use_state(HashMap::<String, String>::new);
    let picker_assets = use_state(|| None::<Vec<String>>);
    let lightbox = use_state(|| None::<usize>);

    {
        let resolved = resolved.clone();
        let vault_path = props.vault_path.clone();
        let paths: Vec<String> = props
            .data
            .items
            .iter()
            .map(|i| i.path.clone())
            .filter(|p| !is_external(p))
            .collect();
        use_effect_with(paths, move |paths| {
            let pending: Vec<String> = paths
                .iter()
                .filter(|p| !resolved.contains_key(*p))
                .cloned()
                .collect();
            if !pending.is_empty() {
                let resolved = resolved.clone();
                let vault_path = vault_path.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let mut map = (*resolved).clone();
                    for path in pending {
                        if let Ok(url) = api::read_asset_data_url(&vault_path, &path).await {
                            map.insert(path, url);
                        }
                    }
                    resolved.set(map);
                });
            }
            || {}
        });
    }

    let open_picker = {
        let picker_assets = picker_assets.clone();
        let vault_path = props.vault_path.clone();
        Callback::from(move |_| {
            let picker_assets = picker_assets.clone();
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let assets = api::list_assets_info(&vault_path)
                    .await
                    .unwrap_or_default()
                    .into_iter()
                    .map(|a| a.path)
                    .filter(|p| is_image(p))
                    .collect::<Vec<_>>();
                picker_assets.set(Some(assets));
            });
        })
    };

    let close_picker = {
        let picker_assets = picker_assets.clone();
        Callback::from(move |_| picker_assets.set(None))
    };

    let columns_step = |delta: i8| {
        let data = props.data.clone();
        let on_change = props.on_change.clone();
        Callback::from(move |_: MouseEvent| {
            let mut new_data = data.clone();
            new_data.adjust_columns(delta);
            on_change.emit(new_data);
        })
    };

    let nav_group = props.nav_group.clone();
    let size_slug = props.data.size.slug();

    html! {
        <div class="gallery" data-nav-group={nav_group.clone()}>
            <div class="gallery__bar">
                <span class="gallery__count">
                    { format!("{} {}", props.data.items.len(), if props.data.items.len() == 1 { "imagem" } else { "imagens" }) }
                </span>
                <div class="gallery__sizes">
                    { for GallerySize::all().iter().map(|s| {
                        let s = *s;
                        let is_active = s == props.data.size;
                        let onclick = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            Callback::from(move |_| {
                                let mut new_data = data.clone();
                                new_data.set_size(s);
                                on_change.emit(new_data);
                            })
                        };
                        html! {
                            <button class={classes!("gallery__size", is_active.then_some("gallery__size--active"))}
                                type="button" title={format!("Miniatura {}", s.label())}
                                data-nav-item="gallery-size" data-nav-parent={nav_group.clone()}
                                {onclick}>{ s.label() }</button>
                        }
                    }) }
                </div>
                <span class="gallery__cols">{ format!("{} col", props.data.columns) }</span>
                <button class="gallery__btn" type="button" title="Menos colunas"
                    data-nav-item="gallery-fewer" data-nav-parent={nav_group.clone()}
                    onclick={columns_step(-1)}><Icon name="chevron-left" /></button>
                <button class="gallery__btn" type="button" title="Mais colunas"
                    data-nav-item="gallery-more" data-nav-parent={nav_group.clone()}
                    onclick={columns_step(1)}><Icon name="chevron-right" /></button>
                <button class="gallery__add" type="button"
                    data-nav-item="gallery-add" data-nav-parent={nav_group.clone()}
                    onclick={open_picker}>{ "+ imagem" }</button>
            </div>

            if props.data.items.is_empty() {
                <p class="gallery__empty">{ "Nenhuma imagem ainda — use \"+ imagem\" pra escolher um arquivo de assets/." }</p>
            } else {
                <div class={classes!("gallery__grid", format!("gallery__grid--{size_slug}"))}
                    style={format!("grid-template-columns: repeat({}, minmax(0, 1fr));", props.data.columns)}>
                    { for props.data.items.iter().enumerate().map(|(idx, item)| {
                        let src = if is_external(&item.path) {
                            Some(item.path.clone())
                        } else {
                            resolved.get(&item.path).cloned()
                        };
                        let open_light: Callback<()> = {
                            let lightbox = lightbox.clone();
                            Callback::from(move |_| lightbox.set(Some(idx)))
                        };
                        let on_caption = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            Callback::from(move |e: FocusEvent| {
                                let Some(el) = e.target().and_then(|t| t.dyn_into::<HtmlInputElement>().ok()) else { return };
                                let mut new_data = data.clone();
                                new_data.set_caption(idx, el.value());
                                on_change.emit(new_data);
                            })
                        };
                        let move_item = |delta: i8| {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            Callback::from(move |_: MouseEvent| {
                                let mut new_data = data.clone();
                                new_data.move_item(idx, delta);
                                on_change.emit(new_data);
                            })
                        };
                        let on_remove = {
                            let data = props.data.clone();
                            let on_change = props.on_change.clone();
                            Callback::from(move |_| {
                                let mut new_data = data.clone();
                                new_data.remove_item(idx);
                                on_change.emit(new_data);
                            })
                        };

                        html! {
                            <figure class="gallery__item" key={idx}>
                                <div class="gallery__thumb"
                                    tabindex="0" role="button"
                                    title="Abrir em tamanho grande"
                                    data-nav-item="gallery-item" data-nav-parent={nav_group.clone()}
                                    onclick={open_light.reform(|_: MouseEvent| ())}
                                    onkeydown={crate::keyboard_activate::activate_on_enter_or_space(open_light.clone())}>
                                    if let Some(src) = src {
                                        <img class="gallery__img" src={src} alt={item.caption.clone()} />
                                    } else {
                                        // Arquivo apagado/renomeado por fora: mostra o
                                        // path pra dar como consertar, em vez de um
                                        // buraco na grade.
                                        <span class="gallery__missing">{ item.path.clone() }</span>
                                    }
                                </div>
                                <figcaption class="gallery__caption">
                                    <input class="gallery__caption-input" type="text"
                                        value={item.caption.clone()} placeholder="Legenda"
                                        data-nav-item="gallery-caption" data-nav-parent={nav_group.clone()}
                                        onblur={on_caption} />
                                </figcaption>
                                <div class="gallery__item-bar">
                                    <button class="gallery__btn" type="button" title="Mover pra esquerda"
                                        data-nav-item="gallery-left" data-nav-parent={nav_group.clone()}
                                        onclick={move_item(-1)}><Icon name="chevron-left" /></button>
                                    <button class="gallery__btn" type="button" title="Mover pra direita"
                                        data-nav-item="gallery-right" data-nav-parent={nav_group.clone()}
                                        onclick={move_item(1)}><Icon name="chevron-right" /></button>
                                    <button class="gallery__btn gallery__btn--danger" type="button" title="Remover da galeria"
                                        data-nav-item="gallery-remove" data-nav-parent={nav_group.clone()}
                                        onclick={on_remove}><Icon name="x" /></button>
                                </div>
                            </figure>
                        }
                    }) }
                </div>
            }

            if let Some(assets) = (*picker_assets).clone() {
                <Modal title="Adicionar imagem" open={true} on_close={close_picker.clone()}>
                    if assets.is_empty() {
                        <p class="gallery__empty">{ "Nenhuma imagem em assets/. Cole uma imagem no editor ou use /img primeiro." }</p>
                    } else {
                        <div class="gallery__picker">
                            { for assets.into_iter().map(|path| {
                                let onclick = {
                                    let data = props.data.clone();
                                    let on_change = props.on_change.clone();
                                    let picker_assets = picker_assets.clone();
                                    let path = path.clone();
                                    Callback::from(move |_| {
                                        let mut new_data = data.clone();
                                        new_data.add_item(path.clone());
                                        on_change.emit(new_data);
                                        picker_assets.set(None);
                                    })
                                };
                                html! {
                                    <button class="gallery__picker-item" type="button" {onclick}>
                                        <Icon name="image" />{ path }
                                    </button>
                                }
                            }) }
                        </div>
                    }
                </Modal>
            }

            if let Some(idx) = *lightbox {
                { render_lightbox(props, idx, &resolved, lightbox.clone()) }
            }
        </div>
    }
}

/// Visualização grande de um item, com navegação entre os vizinhos.
fn render_lightbox(
    props: &InlineGalleryProps,
    idx: usize,
    resolved: &HashMap<String, String>,
    lightbox: UseStateHandle<Option<usize>>,
) -> Html {
    let Some(item) = props.data.items.get(idx) else {
        return html! {};
    };
    let src = if is_external(&item.path) {
        Some(item.path.clone())
    } else {
        resolved.get(&item.path).cloned()
    };
    let total = props.data.items.len();
    let step = |delta: isize| {
        let lightbox = lightbox.clone();
        Callback::from(move |_: MouseEvent| {
            let next = (idx as isize + delta).rem_euclid(total as isize) as usize;
            lightbox.set(Some(next));
        })
    };
    let on_close = {
        let lightbox = lightbox.clone();
        Callback::from(move |_| lightbox.set(None))
    };

    html! {
        <Modal title={if item.caption.is_empty() { item.path.clone() } else { item.caption.clone() }}
            open={true} wide={true} on_close={on_close} focus_nonce={idx as u32}>
            <div class="gallery__lightbox">
                if total > 1 {
                    <button class="gallery__btn" type="button" title="Anterior" onclick={step(-1)}>
                        <Icon name="chevron-left" />
                    </button>
                }
                if let Some(src) = src {
                    <img class="gallery__lightbox-img" src={src} alt={item.caption.clone()} />
                } else {
                    <span class="gallery__missing">{ item.path.clone() }</span>
                }
                if total > 1 {
                    <button class="gallery__btn" type="button" title="Próxima" onclick={step(1)}>
                        <Icon name="chevron-right" />
                    </button>
                }
            </div>
        </Modal>
    }
}
