//! Formulário do gantt (ciclo 489): seções e, em ordem, tarefas com
//! início, fim ou duração e dependência.

use anotadinho_core::diagrama::gantt::{duracao_valida, Fim, Gantt, Inicio, Marca, Secao, Tarefa};
use anotadinho_core::diagrama::id_valido;
use yew::prelude::*;

use super::linhas::{aplicar, botoes, com, marcado, texto_do_input, valor_do_select, AcaoLinha, CampoData};
use crate::components::icon::Icon;

#[derive(Properties, PartialEq)]
pub struct EditorGanttProps {
    pub modelo: Gantt,
    pub on_mudar: Callback<Gantt>,
}

fn nome_da_marca(m: Marca) -> &'static str {
    match m {
        Marca::Feita => "feita",
        Marca::Ativa => "em andamento",
        Marca::Critica => "crítica",
        Marca::Marco => "marco",
    }
}

/// Uma tarefa de que a `(s, t)` pode depender.
struct Dependencia {
    /// `"si:ti"`: o valor da `<option>`.
    valor: String,
    rotulo: String,
    id: Option<String>,
}

/// As tarefas de que `(s, t)` pode depender: as que têm id, e as que podem
/// ganhar um (início escrito — sem ele o mermaid não aceita id). Assim a
/// pessoa escolhe pelo NOME, e o id nasce sozinho.
fn dependencias(g: &Gantt, s: usize, t: usize) -> Vec<Dependencia> {
    g.secoes
        .iter()
        .enumerate()
        .flat_map(|(si, sec)| sec.tarefas.iter().enumerate().map(move |(ti, tar)| (si, ti, tar)))
        .filter(|(si, ti, tar)| (*si, *ti) != (s, t) && (tar.id.is_some() || tar.inicio != Inicio::Anterior))
        .map(|(si, ti, tar)| Dependencia { valor: format!("{si}:{ti}"), rotulo: tar.nome.clone(), id: tar.id.clone() })
        .collect()
}

/// O id da tarefa `valor` (`"si:ti"`), dando um novo se ela não tiver.
fn id_de(m: &mut Gantt, valor: &str) -> Option<String> {
    let (si, ti) = valor.split_once(':')?;
    let (si, ti): (usize, usize) = (si.parse().ok()?, ti.parse().ok()?);
    if let Some(id) = m.secoes.get(si)?.tarefas.get(ti)?.id.clone() {
        return Some(id);
    }
    let novo = proximo_id(m);
    m.secoes[si].tarefas[ti].id = Some(novo.clone());
    Some(novo)
}

/// Quantas tarefas dependem de `id`: apagar ou trocar o id deixaria o
/// `after` apontando pro nada.
fn dependentes(g: &Gantt, id: &str) -> usize {
    g.secoes
        .iter()
        .flat_map(|s| &s.tarefas)
        .filter(|t| matches!(&t.inicio, Inicio::Depois(ids) if ids.iter().any(|i| i == id)))
        .count()
}

fn proximo_id(g: &Gantt) -> String {
    let usados: Vec<&str> = g.secoes.iter().flat_map(|s| &s.tarefas).filter_map(|t| t.id.as_deref()).collect();
    (1..).map(|n| format!("t{n}")).find(|id| !usados.contains(&id.as_str())).expect("infinito")
}

#[function_component(EditorGantt)]
pub fn editor_gantt(props: &EditorGanttProps) -> Html {
    let g = &props.modelo;
    let on = &props.on_mudar;
    let n_secoes = g.secoes.len();

    let mudar_titulo = com(g, on, |m: &mut Gantt, e: InputEvent| {
        let t = texto_do_input(e);
        m.titulo = (!t.trim().is_empty()).then(|| t.trim().to_string());
    });

    let secoes = g.secoes.iter().enumerate().map(|(si, sec)| {
        let n_tarefas = sec.tarefas.len();
        let bloqueio = (n_tarefas > 0).then(|| format!("A seção tem {n_tarefas} tarefas"));
        let acao_secao = com(g, on, move |m: &mut Gantt, a: AcaoLinha| aplicar(&mut m.secoes, si, a));
        let mudar_nome = com(g, on, move |m: &mut Gantt, e: InputEvent| {
            let t = texto_do_input(e);
            // A seção sem nome só existe na primeira posição, e só se tiver
            // nome não se escreve `section`: vazio volta a ser "sem seção"
            // quando ela é a primeira, e "Seção" nas outras.
            m.secoes[si].nome = if !t.trim().is_empty() {
                Some(t.trim().to_string())
            } else if si == 0 {
                None
            } else {
                Some("Seção".into())
            };
        });
        let grupo = format!("construtor-gantt-s{si}");
        let tarefas = sec.tarefas.iter().enumerate().map(|(ti, t)| linha_de_tarefa(g, on, si, ti, t, n_tarefas, &grupo));
        let nova_tarefa = com(g, on, move |m: &mut Gantt, _: MouseEvent| {
            let primeira = m.secoes.iter().all(|s| s.tarefas.is_empty());
            let inicio = if primeira { Inicio::Data(crate::date_util::today_string()) } else { Inicio::Anterior };
            m.secoes[si].tarefas.push(Tarefa { nome: "Tarefa".into(), marcas: Vec::new(), id: None, inicio, fim: Fim::Duracao("3d".into()) });
        });
        html! {
            <li class="construtor-mermaid__bloco-secao">
                <div class="construtor-mermaid__linha">
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Nome da seção"
                        placeholder={if si == 0 { "sem seção" } else { "Seção" }}
                        value={sec.nome.clone().unwrap_or_default()} oninput={mudar_nome}
                        data-nav-item={format!("construtor-gantt-s{si}-nome")} data-nav-parent="construtor-gantt-secoes" />
                    { botoes("construtor-gantt-secoes", si, n_secoes, bloqueio, acao_secao) }
                </div>
                <ul class="construtor-mermaid__lista" data-nav-group={grupo.clone()}>{ for tarefas }</ul>
                <button type="button" class="btn btn--ghost btn--sm" data-nav-item={format!("construtor-gantt-s{si}-nova")}
                    data-nav-parent="construtor-gantt-secoes" onclick={nova_tarefa}>
                    <Icon name="plus" />{ " Tarefa" }
                </button>
            </li>
        }
    });

    let nova_secao = com(g, on, |m: &mut Gantt, _: MouseEvent| {
        m.com_formato = true;
        m.secoes.push(Secao { nome: Some(format!("Seção {}", m.secoes.len() + 1)), tarefas: Vec::new() });
    });

    html! {
        <div class="construtor-mermaid__form">
            <label class="construtor-mermaid__linha">
                <span class="construtor-mermaid__rotulo">{ "Título" }</span>
                <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Título do cronograma"
                    value={g.titulo.clone().unwrap_or_default()} oninput={mudar_titulo} data-nav-item="construtor-gantt-titulo" />
            </label>
            <ul class="construtor-mermaid__lista" data-nav-group="construtor-gantt-secoes">{ for secoes }</ul>
            <button type="button" class="btn btn--ghost btn--sm" data-nav-item="construtor-gantt-nova-secao" onclick={nova_secao}>
                <Icon name="plus" />{ " Seção" }
            </button>
        </div>
    }
}

fn tarefa_mut(m: &mut Gantt, si: usize, ti: usize) -> &mut Tarefa {
    &mut m.secoes[si].tarefas[ti]
}

fn linha_de_tarefa(g: &Gantt, on: &Callback<Gantt>, si: usize, ti: usize, t: &Tarefa, n: usize, grupo: &str) -> Html {
    let nav = |campo: &str| format!("{grupo}-t{ti}-{campo}");

    let usada = t.id.as_deref().map(|id| dependentes(g, id)).unwrap_or(0);
    let bloqueio = (usada > 0).then(|| format!("{usada} tarefas dependem desta"));
    let acao = com(g, on, move |m: &mut Gantt, a: AcaoLinha| aplicar(&mut m.secoes[si].tarefas, ti, a));
    let mudar_nome = com(g, on, move |m: &mut Gantt, e: InputEvent| {
        let n = texto_do_input(e);
        tarefa_mut(m, si, ti).nome = if n.trim().is_empty() { "Tarefa".into() } else { n.trim().to_string() };
    });
    // Id: só entra no modelo se for id de verdade; no meio da digitação
    // (`t-`), o campo fica marcado e o diagrama, como estava.
    let mudar_id = com(g, on, move |m: &mut Gantt, e: InputEvent| {
        let v = texto_do_input(e);
        let v = v.trim();
        if v.is_empty() {
            tarefa_mut(m, si, ti).id = None;
        } else if id_valido(v) {
            tarefa_mut(m, si, ti).id = Some(v.to_string());
        }
    });
    let sem_inicio = t.inicio == Inicio::Anterior;
    let deps = dependencias(g, si, ti);
    let tipo_inicio = match &t.inicio {
        Inicio::Anterior => "anterior",
        Inicio::Data(_) => "data",
        Inicio::Depois(_) => "depois",
    };
    let mudar_tipo_inicio = {
        let primeira_dep = deps.first().map(|d| d.valor.clone());
        com(g, on, move |m: &mut Gantt, e: Event| {
            let inicio = match valor_do_select(e).as_str() {
                "data" => Inicio::Data(crate::date_util::today_string()),
                "depois" => match primeira_dep.as_deref().and_then(|v| id_de(m, v)) {
                    Some(id) => Inicio::Depois(vec![id]),
                    None => return,
                },
                _ => Inicio::Anterior,
            };
            let t = tarefa_mut(m, si, ti);
            t.inicio = inicio;
            // Sem início escrito, o mermaid não aceita id (não existe a
            // forma `id, duração`): o id sai junto.
            if t.inicio == Inicio::Anterior {
                t.id = None;
            }
        })
    };
    let escolher_data_inicio = com(g, on, move |m: &mut Gantt, d: String| tarefa_mut(m, si, ti).inicio = Inicio::Data(d));
    let mudar_dep = com(g, on, move |m: &mut Gantt, e: Event| {
        if let Some(id) = id_de(m, &valor_do_select(e)) {
            tarefa_mut(m, si, ti).inicio = Inicio::Depois(vec![id]);
        }
    });

    let tipo_fim = if matches!(t.fim, Fim::Data(_)) { "data" } else { "duracao" };
    let mudar_tipo_fim = com(g, on, move |m: &mut Gantt, e: Event| {
        tarefa_mut(m, si, ti).fim = match valor_do_select(e).as_str() {
            "data" => Fim::Data(crate::date_util::today_string()),
            _ => Fim::Duracao("3d".into()),
        };
    });
    let mudar_duracao = com(g, on, move |m: &mut Gantt, e: InputEvent| {
        let v = texto_do_input(e);
        if duracao_valida(v.trim()) {
            tarefa_mut(m, si, ti).fim = Fim::Duracao(v.trim().to_string());
        }
    });
    let escolher_data_fim = com(g, on, move |m: &mut Gantt, d: String| tarefa_mut(m, si, ti).fim = Fim::Data(d));

    let marcas = Marca::TODAS.iter().map(|marca| {
        let marca = *marca;
        let mudar = com(g, on, move |m: &mut Gantt, e: Event| {
            let t = tarefa_mut(m, si, ti);
            t.marcas.retain(|x| *x != marca);
            if marcado(e) {
                t.marcas.push(marca);
                // A ordem das marcas é a do mermaid, pra o código não mudar
                // por ter marcado numa ordem diferente.
                t.marcas.sort_by_key(|x| Marca::TODAS.iter().position(|y| y == x));
            }
        });
        html! {
            <label class="construtor-mermaid__marca">
                <input type="checkbox" checked={t.marcas.contains(&marca)} onchange={mudar}
                    data-nav-item={nav(marca.token())} data-nav-parent={grupo.to_string()} />
                { nome_da_marca(marca) }
            </label>
        }
    });

    html! {
        <li class="construtor-mermaid__linha construtor-mermaid__linha--tarefa">
            <input class="construtor-mermaid__campo construtor-mermaid__campo--largo" aria-label="Nome da tarefa"
                value={t.nome.clone()} oninput={mudar_nome} data-nav-item={nav("nome")} data-nav-parent={grupo.to_string()} />
            <input class="construtor-mermaid__campo construtor-mermaid__campo--curto" aria-label="Id da tarefa (pra outra depender dela)"
                placeholder="id" value={t.id.clone().unwrap_or_default()} oninput={mudar_id}
                disabled={sem_inicio || bloqueio.is_some()}
                title={if sem_inicio { "Com início logo depois da anterior, o mermaid não aceita id" } else { "Pra outra tarefa poder começar depois desta" }}
                data-nav-item={nav("id")} data-nav-parent={grupo.to_string()} />
            <select class="construtor-mermaid__campo" aria-label="Início" onchange={mudar_tipo_inicio}
                data-nav-item={nav("inicio")} data-nav-parent={grupo.to_string()}>
                <option value="anterior" selected={tipo_inicio == "anterior"}>{ "logo depois da anterior" }</option>
                <option value="data" selected={tipo_inicio == "data"}>{ "numa data" }</option>
                <option value="depois" selected={tipo_inicio == "depois"} disabled={deps.is_empty()}>{ "depois de…" }</option>
            </select>
            { match &t.inicio {
                Inicio::Data(d) => html! { <CampoData valor={Some(d.clone())} on_escolher={escolher_data_inicio} nav={nav("data-inicio")} grupo={grupo.to_string()} /> },
                Inicio::Depois(ids) => html! {
                    <select class="construtor-mermaid__campo" aria-label="Depois de qual tarefa" onchange={mudar_dep}
                        data-nav-item={nav("dep")} data-nav-parent={grupo.to_string()}>
                        { for deps.iter().map(|d| html! {
                            <option value={d.valor.clone()} selected={d.id.is_some() && ids.first() == d.id.as_ref()}>{ &d.rotulo }</option>
                        }) }
                    </select>
                },
                Inicio::Anterior => html! {},
            } }
            <select class="construtor-mermaid__campo" aria-label="Fim" onchange={mudar_tipo_fim}
                data-nav-item={nav("fim")} data-nav-parent={grupo.to_string()}>
                <option value="duracao" selected={tipo_fim == "duracao"}>{ "dura" }</option>
                <option value="data" selected={tipo_fim == "data"}>{ "até" }</option>
            </select>
            { match &t.fim {
                Fim::Duracao(d) => html! {
                    <input class="construtor-mermaid__campo construtor-mermaid__campo--curto" aria-label="Duração (3d, 2w, 12h)"
                        title="dias (d), semanas (w) ou horas (h)" value={d.clone()} oninput={mudar_duracao}
                        data-nav-item={nav("duracao")} data-nav-parent={grupo.to_string()} />
                },
                Fim::Data(d) => html! { <CampoData valor={Some(d.clone())} on_escolher={escolher_data_fim} nav={nav("data-fim")} grupo={grupo.to_string()} /> },
            } }
            <span class="construtor-mermaid__marcas">{ for marcas }</span>
            { botoes(grupo, ti, n, bloqueio, acao) }
        </li>
    }
}
