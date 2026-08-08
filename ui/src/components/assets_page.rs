//! Página `type: assets` — lista arquivos em `assets/` (tamanho, se
//! está referenciado em alguma página) com ação de excluir. Mesmo
//! espírito somente-leitura/utilitário das páginas `type: calendar`
//! (ciclo 085) e `type: tags` (ciclo 093), dispatchada por
//! `page_view.rs`.

use yew::prelude::*;

use crate::api::{self, AssetInfo};
use crate::dialog::PendingDialog;

/// Props da `AssetsPage`.
#[derive(Properties, PartialEq, Clone)]
pub struct AssetsPageProps {
    /// Path do vault.
    pub vault_path: String,
    /// Abre o modal de diálogo do app (confirmação de exclusão).
    pub open_dialog: Callback<PendingDialog>,
}

#[derive(Debug, Clone, PartialEq)]
struct AssetRow {
    info: AssetInfo,
    used: bool,
}

/// Formata bytes como KB/MB — sem isso a lista mostraria só um número
/// de bytes cru, difícil de escanear visualmente.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{} B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}

#[function_component(AssetsPage)]
pub fn assets_page(props: &AssetsPageProps) -> Html {
    let rows = use_state(Vec::<AssetRow>::new);
    let loading = use_state(|| true);
    let refresh_tick = use_state(|| 0u32);

    {
        let vault_path = props.vault_path.clone();
        let rows = rows.clone();
        let loading = loading.clone();
        let tick = *refresh_tick;
        use_effect_with((vault_path.clone(), tick), move |(vault_path, _)| {
            let vault_path = vault_path.clone();
            let rows = rows.clone();
            let loading = loading.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                let assets = api::list_assets_info(&vault_path).await.unwrap_or_default();
                // Um único scan de todas as páginas, reaproveitado pra
                // decidir "usado" de TODOS os assets — mais barato do
                // que rodar `search_content` uma vez por asset.
                let mut haystack = String::new();
                if let Ok(pages) = api::list_pages(&vault_path).await {
                    for page in &pages {
                        if let Ok(content) = api::read_page(&vault_path, &page.path).await {
                            haystack.push_str(&content);
                            haystack.push('\n');
                        }
                    }
                }
                let list: Vec<AssetRow> = assets.into_iter().map(|info| {
                    let file_name = std::path::Path::new(&info.path).file_name()
                        .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                    let used = haystack.contains(&info.path) || (!file_name.is_empty() && haystack.contains(&file_name));
                    AssetRow { info, used }
                }).collect();
                rows.set(list);
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="assets-page"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let total_size: u64 = rows.iter().map(|r| r.info.size).sum();
    let unused_count = rows.iter().filter(|r| !r.used).count();

    html! {
        <div class="assets-page">
            <h2 class="assets-page__title">{ "Assets" }</h2>
            <p class="assets-page__summary">
                { format!("{} arquivos · {} · {} não referenciados", rows.len(), format_size(total_size), unused_count) }
            </p>
            if rows.is_empty() {
                <p class="assets-page__empty">{ "Nenhum arquivo em assets/ ainda." }</p>
            } else {
                <table class="assets-page__table">
                    <thead>
                        <tr>
                            <th>{ "Arquivo" }</th>
                            <th>{ "Tamanho" }</th>
                            <th>{ "Uso" }</th>
                            <th></th>
                        </tr>
                    </thead>
                    <tbody>
                        { for rows.iter().map(|row| {
                            let path = row.info.path.clone();
                            let file_name = std::path::Path::new(&path).file_name()
                                .map(|s| s.to_string_lossy().to_string()).unwrap_or_default();
                            let vault_path = props.vault_path.clone();
                            let open_dialog = props.open_dialog.clone();
                            let refresh_tick = refresh_tick.clone();
                            let path_for_delete = path.clone();
                            let onclick = Callback::from(move |_| {
                                let vault_path = vault_path.clone();
                                let refresh_tick = refresh_tick.clone();
                                let path = path_for_delete.clone();
                                let open_dialog_for_error = open_dialog.clone();
                                open_dialog.emit(PendingDialog::Confirm {
                                    message: format!("Excluir \"{}\"? Isso não pode ser desfeito.", file_name),
                                    confirm_label: "Excluir".to_string(),
                                    on_confirm: Callback::from(move |_| {
                                        let vault_path = vault_path.clone();
                                        let refresh_tick = refresh_tick.clone();
                                        let path = path.clone();
                                        let open_dialog = open_dialog_for_error.clone();
                                        wasm_bindgen_futures::spawn_local(async move {
                                            if let Err(e) = api::delete_asset(&vault_path, &path).await {
                                                open_dialog.emit(PendingDialog::Alert {
                                                    message: format!("Erro ao excluir: {}", e),
                                                });
                                            } else {
                                                refresh_tick.set(*refresh_tick + 1);
                                            }
                                        });
                                    }),
                                });
                            });
                            html! {
                                <tr>
                                    <td>{ &row.info.path }</td>
                                    <td>{ format_size(row.info.size) }</td>
                                    <td>
                                        if row.used {
                                            <span class="badge badge--success">{ "usado" }</span>
                                        } else {
                                            <span class="badge badge--warning">{ "não usado" }</span>
                                        }
                                    </td>
                                    <td>
                                        <button class="btn btn--danger btn--xs" {onclick}>{ "Excluir" }</button>
                                    </td>
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
