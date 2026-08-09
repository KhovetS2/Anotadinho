//! Task table - lista paginas com status:: property.

use yew::prelude::*;
use crate::api::{self, PageMeta};

#[derive(Properties, PartialEq, Clone)]
pub struct TaskTableProps {
    pub vault_path: String,
    pub on_page_selected: Callback<PageMeta>,
}

#[derive(Debug, Clone, PartialEq)]
struct Task { path: String, title: String, status: String, priority: String }

#[function_component(TaskTable)]
pub fn task_table(props: &TaskTableProps) -> Html {
    let tasks = use_state(Vec::<Task>::new);
    let loading = use_state(|| true);
    let sort_by = use_state(|| "title".to_string());

    {
        let vault_path = props.vault_path.clone();
        let tasks = tasks.clone();
        let loading = loading.clone();
        use_effect_with((), move |_| {
            let vault_path = vault_path.clone();
            wasm_bindgen_futures::spawn_local(async move {
                loading.set(true);
                if let Ok(pages) = api::list_pages(&vault_path).await {
                    let mut list = Vec::new();
                    for page in &pages {
                        if let Ok(content) = api::read_page(&vault_path, &page.path).await {
                            let mut status = "-".to_string();
                            let mut priority = "-".to_string();
                            for line in content.lines() {
                                if let Some(v) = line.trim().strip_prefix("status:: ") {
                                    status = v.trim().to_string();
                                }
                                if let Some(v) = line.trim().strip_prefix("priority:: ") {
                                    priority = v.trim().to_string();
                                }
                            }
                            if status != "-" || priority != "-" {
                                list.push(Task {
                                    path: page.path.clone(),
                                    title: page.title.clone(),
                                    status, priority,
                                });
                            }
                        }
                    }
                    tasks.set(list);
                }
                loading.set(false);
            });
            || {}
        });
    }

    if *loading {
        return html! { <div class="task-table"><p class="editor__status">{ "Carregando..." }</p></div> };
    }

    let mut sorted = (*tasks).clone();
    match sort_by.as_str() {
        "title" => sorted.sort_by(|a, b| a.title.cmp(&b.title)),
        "status" => sorted.sort_by(|a, b| a.status.cmp(&b.status)),
        "priority" => sorted.sort_by(|a, b| a.priority.cmp(&b.priority)),
        _ => {}
    }

    let on_page_selected = props.on_page_selected.clone();

    html! {
        <div class="task-table" data-nav-content-root="true">
            <div class="task-table__header-row">
                <h2>{"Tarefas"}</h2>
                <span class="badge">{ tasks.len() }</span>
            </div>
            if tasks.is_empty() {
                <p class="calendar__empty">{"Nenhuma tarefa encontrada. Use 'status::' e 'priority::' nas páginas."}</p>
            } else {
                <table class="task-table__table">
                    <thead>
                        <tr>
                            { {
                                let sort_by = sort_by.clone();
                                let activate = Callback::from(move |_: ()| sort_by.set("title".to_string()));
                                let onclick = { let activate = activate.clone(); Callback::from(move |_: MouseEvent| activate.emit(())) };
                                let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                                html! { <th class="task-table__th" tabindex="0" {onclick} {onkeydown}>{"Título"}</th> }
                            } }
                            { {
                                let sort_by = sort_by.clone();
                                let activate = Callback::from(move |_: ()| sort_by.set("status".to_string()));
                                let onclick = { let activate = activate.clone(); Callback::from(move |_: MouseEvent| activate.emit(())) };
                                let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                                html! { <th class="task-table__th" tabindex="0" {onclick} {onkeydown}>{"Status"}</th> }
                            } }
                            { {
                                let sort_by = sort_by.clone();
                                let activate = Callback::from(move |_: ()| sort_by.set("priority".to_string()));
                                let onclick = { let activate = activate.clone(); Callback::from(move |_: MouseEvent| activate.emit(())) };
                                let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                                html! { <th class="task-table__th" tabindex="0" {onclick} {onkeydown}>{"Prioridade"}</th> }
                            } }
                        </tr>
                    </thead>
                    <tbody>
                        { for sorted.iter().map(|task| {
                            let path = task.path.clone();
                            let title = task.title.clone();
                            let meta = PageMeta { path: path.clone(), title: title.clone(), section: "pages".to_string() };
                            let status_class = match task.status.as_str() {
                                "done" | "concluido" => "badge badge--success",
                                "doing" | "em-andamento" => "badge badge--info",
                                _ => "badge badge--warning"
                            };
                            let activate = {
                                let on_page_selected = on_page_selected.clone();
                                let meta = meta.clone();
                                Callback::from(move |_: ()| on_page_selected.emit(meta.clone()))
                            };
                            let onclick = {
                                let activate = activate.clone();
                                Callback::from(move |_: MouseEvent| activate.emit(()))
                            };
                            let onkeydown = crate::keyboard_activate::activate_on_enter_or_space(activate);
                            html! {
                                <tr class="task-table__row" style="cursor:pointer"
                                    tabindex="0" {onclick} {onkeydown}
                                >
                                    <td class="task-table__td">{ &task.title }</td>
                                    <td class="task-table__td"><span class={status_class}>{ &task.status }</span></td>
                                    <td class="task-table__td">{ &task.priority }</td>
                                </tr>
                            }
                        }) }
                    </tbody>
                </table>
            }
        </div>
    }
}
