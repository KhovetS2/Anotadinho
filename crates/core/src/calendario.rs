//! A geometria da grade mensal do calendário (ciclo 306).
//!
//! Morava em `ui/src/components/embeds/inline_calendar.rs`, e era a
//! única cópia: a janela montava as semanas e alocava as faixas dos
//! eventos ali dentro. Quando o terminal passou a desenhar a mesma
//! grade, a escolha era copiar o algoritmo ou movê-lo — e duas cópias
//! de "em que faixa vai este evento" discordariam no primeiro ajuste.
//! Moveu, igual o `embed` e o `date_util` no ciclo 149: é aritmética
//! pura, sem DOM.

use crate::date_util;
use crate::embed::CalendarEntry;
use std::collections::BTreeSet;

/// Rótulos das colunas da semana, de domingo a sábado — os mesmos que a
/// janela mostra em cima da grade.
pub const WEEKDAY_LABELS: [&str; 7] = ["D", "S", "T", "Q", "Q", "S", "S"];

/// Quantas faixas de evento cabem num dia antes de virar "+N mais".
pub const MAX_LANES: usize = 3;

/// Uma barra de evento posicionada num intervalo de dias visíveis (semana
/// inteira na visão Mês, ou a janela de 1/7 dias das visões Dia/Semana).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bar {
    /// Índice do evento em `entries`.
    pub entry_idx: usize,
    /// Faixa (linha) dentro da semana, a partir de 0.
    pub lane: usize,
    /// Primeira coluna (dia) que a barra ocupa.
    pub start_col: usize,
    /// Última coluna (inclusiva).
    pub end_col: usize,
}

/// Aloca as barras de `entries` que tocam `day_dates` em lanes sem
/// sobreposição (algoritmo guloso: ordena por início, cada evento vai na
/// primeira lane livre). Eventos que não cabem nas `MAX_LANES` visíveis
/// incrementam o contador de overflow nas colunas (dias) que tocam.
/// Genérico sobre o tamanho da janela — usado tanto pela semana inteira
/// (7 dias, visão Mês) quanto pela janela de Dia/Semana.
///
/// `exclude_timed`: quando `true`, eventos com `start_time` ficam de fora
/// (usado pra faixa de dia inteiro das visões Semana/Dia, onde um evento
/// com horário já ganha um bloco posicionado na grade de horas — mostrar
/// ele nos dois lugares seria duplicado). Na visão Mês (sem grade de
/// horas pra mostrar o horário de outro jeito) passa `false`, todo evento
/// vira barra independente de ter horário ou não.
pub fn pack_days(entries: &[CalendarEntry], day_dates: &[String], exclude_timed: bool) -> (Vec<Bar>, Vec<usize>) {
    pack_days_ate(entries, day_dates, exclude_timed, MAX_LANES)
}

/// `pack_days` com o limite de faixas dado (ciclo 316). A visão Semana
/// do terminal não tem grade de horas pra onde mandar o excesso, então
/// mostra todas as faixas em vez de "+N mais".
pub fn pack_days_ate(
    entries: &[CalendarEntry],
    day_dates: &[String],
    exclude_timed: bool,
    max_lanes: usize,
) -> (Vec<Bar>, Vec<usize>) {
    let n = day_dates.len();
    let mut overflow = vec![0usize; n];
    if n == 0 {
        return (Vec::new(), overflow);
    }
    let window_start = day_dates[0].as_str();
    let window_end = day_dates[n - 1].as_str();

    let mut touching: Vec<(usize, usize, usize)> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        if exclude_timed && e.start_time.is_some() {
            continue;
        }
        // Evento sem data (na gaveta) não aparece na grade.
        let Some(e_start) = e.date.as_deref() else { continue };
        let e_end = e.end_date.as_deref().unwrap_or(e_start);
        if e_end < window_start || e_start > window_end {
            continue;
        }
        let clipped_start = if e_start > window_start { e_start } else { window_start };
        let clipped_end = if e_end < window_end { e_end } else { window_end };
        let start_col = date_util::days_between(window_start, clipped_start).unwrap_or(0).max(0) as usize;
        let end_col = date_util::days_between(window_start, clipped_end).unwrap_or(0).max(0) as usize;
        touching.push((i, start_col, end_col));
    }
    touching.sort_by_key(|&(_, start_col, _)| start_col);

    let mut lane_end: Vec<i64> = Vec::new();
    let mut bars = Vec::new();

    for (entry_idx, start_col, end_col) in touching {
        let mut placed = false;
        for (lane, last_end) in lane_end.iter_mut().enumerate() {
            if *last_end < start_col as i64 {
                *last_end = end_col as i64;
                bars.push(Bar { entry_idx, lane, start_col, end_col });
                placed = true;
                break;
            }
        }
        if !placed {
            if lane_end.len() < max_lanes {
                lane_end.push(end_col as i64);
                bars.push(Bar { entry_idx, lane: lane_end.len() - 1, start_col, end_col });
            } else {
                for c in overflow.iter_mut().take(end_col + 1).skip(start_col) {
                    *c += 1;
                }
            }
        }
    }
    (bars, overflow)
}

/// As 42 células (6 semanas × 7 dias) da visão Mês de `(vy, vm)`, como
/// `(ano, mês, dia, é_do_mês)` — com os dias do mês anterior e do
/// seguinte completando a primeira e as últimas semanas.
pub fn month_cells(vy: i32, vm: u32) -> Vec<(i32, u32, u32, bool)> {
    let first_weekday = date_util::weekday_of(vy, vm, 1);
    let days_in_month = date_util::days_in_month(vy, vm);
    let (py, pm) = date_util::prev_month(vy, vm);
    let days_in_prev = date_util::days_in_month(py, pm);
    let (ny, nm) = date_util::next_month(vy, vm);

    let mut cells: Vec<(i32, u32, u32, bool)> = Vec::with_capacity(42);
    for i in 0..first_weekday {
        cells.push((py, pm, days_in_prev - (first_weekday - 1 - i), false));
    }
    for d in 1..=days_in_month {
        cells.push((vy, vm, d, true));
    }
    let mut trailing = 1;
    while cells.len() < 42 {
        cells.push((ny, nm, trailing, false));
        trailing += 1;
    }
    cells
}

/// Todas as tags usadas pelos eventos, ordenadas e sem repetição — é a
/// lista de "opções" que `badge_class` usa pra dar a cor de cada
/// evento, então a mesma tag sai da mesma cor em todo o calendário.
pub fn existing_tags(entries: &[CalendarEntry]) -> Vec<String> {
    let set: BTreeSet<String> = entries.iter().flat_map(|e| e.all_tags()).collect();
    set.into_iter().collect()
}

/// Os eventos de um calendário em modo vault: uma entrada por página com
/// `date` (frontmatter ou `date::` no corpo), com `end_date` e `time`
/// quando houver e o caminho da página (ciclo 317).
///
/// Morava na janela (`scan_vault_calendar_entries`), colado à chamada de
/// IPC. A varredura continua sendo de quem roda; a TRADUÇÃO de página em
/// evento é uma só, pros dois lados mostrarem o mesmo calendário.
pub fn entradas_do_vault(paginas: &[crate::PageIndexEntry]) -> Vec<CalendarEntry> {
    paginas
        .iter()
        .filter_map(|page| {
            let date = page.properties.get("date")?.clone();
            Some(CalendarEntry {
                date: Some(date),
                title: page.title.clone(),
                end_date: page.properties.get("end_date").cloned(),
                tags: Vec::new(),
                legacy_tag: None,
                start_time: page.properties.get("time").cloned(),
                end_time: None,
                page_path: Some(page.path.clone()),
            })
        })
        .collect()
}

/// Os meses `(ano, mês)` que algum evento com data toca, em ordem.
///
/// Sem "mês corrente" — o terminal e o CLI não têm navegação — a grade
/// mostra todo mês onde há evento. Um evento que atravessa a virada
/// aparece nos dois.
pub fn months_with_events(entries: &[CalendarEntry]) -> Vec<(i32, u32)> {
    let mut meses: BTreeSet<(i32, u32)> = BTreeSet::new();
    for e in entries {
        let Some(inicio) = e.date.as_deref().and_then(date_util::parse_date) else { continue };
        let fim = e
            .end_date
            .as_deref()
            .and_then(date_util::parse_date)
            .filter(|f| (f.0, f.1) >= (inicio.0, inicio.1))
            .unwrap_or(inicio);
        let (mut y, mut m) = (inicio.0, inicio.1);
        loop {
            meses.insert((y, m));
            if (y, m) >= (fim.0, fim.1) {
                break;
            }
            (y, m) = date_util::next_month(y, m);
        }
    }
    meses.into_iter().collect()
}

#[cfg(test)]
mod testes {
    use super::*;

    fn evento(date: &str, end: Option<&str>, title: &str) -> CalendarEntry {
        CalendarEntry {
            date: Some(date.into()),
            end_date: end.map(Into::into),
            title: title.into(),
            ..Default::default()
        }
    }

    #[test]
    fn agosto_de_2026_comeca_no_sabado() {
        let cells = month_cells(2026, 8);
        assert_eq!(cells.len(), 42);
        // Domingo 26 de julho abre a grade; 1º de agosto é a sétima.
        assert_eq!(cells[0], (2026, 7, 26, false));
        assert_eq!(cells[6], (2026, 8, 1, true));
        assert_eq!(cells[36], (2026, 8, 31, true));
        assert_eq!(cells[37], (2026, 9, 1, false));
    }

    #[test]
    fn eventos_que_se_cruzam_vao_pra_faixas_diferentes() {
        let semana: Vec<String> = (9..=15).map(|d| format!("2026-08-{d:02}")).collect();
        let entries = vec![
            evento("2026-08-10", Some("2026-08-14"), "Sprint"),
            evento("2026-08-12", None, "Reunião"),
            evento("2026-08-15", None, "Folga"),
        ];
        let (bars, overflow) = pack_days(&entries, &semana, false);
        assert_eq!(bars[0], Bar { entry_idx: 0, lane: 0, start_col: 1, end_col: 5 });
        assert_eq!(bars[1], Bar { entry_idx: 1, lane: 1, start_col: 3, end_col: 3 });
        // Começa depois que a Sprint acabou: volta pra faixa 0.
        assert_eq!(bars[2], Bar { entry_idx: 2, lane: 0, start_col: 6, end_col: 6 });
        assert_eq!(overflow, vec![0; 7]);
    }

    #[test]
    fn a_quarta_faixa_vira_overflow() {
        let semana: Vec<String> = (9..=15).map(|d| format!("2026-08-{d:02}")).collect();
        let entries: Vec<CalendarEntry> =
            (0..4).map(|i| evento("2026-08-11", None, &format!("E{i}"))).collect();
        let (bars, overflow) = pack_days(&entries, &semana, false);
        assert_eq!(bars.len(), MAX_LANES);
        assert_eq!(overflow[2], 1);
    }

    #[test]
    fn evento_que_atravessa_o_mes_aparece_nos_dois() {
        let entries = vec![
            evento("2026-08-30", Some("2026-09-02"), "Virada"),
            evento("2026-08-06", None, "Revisão"),
        ];
        assert_eq!(months_with_events(&entries), vec![(2026, 8), (2026, 9)]);
    }

    #[test]
    fn tags_existentes_saem_ordenadas_e_sem_repeticao() {
        let mut a = evento("2026-08-06", None, "A");
        a.tags = vec!["urgente".into()];
        let mut b = evento("2026-08-10", None, "B");
        b.tags = vec!["infra".into(), "urgente".into()];
        assert_eq!(existing_tags(&[a, b]), vec!["infra".to_string(), "urgente".to_string()]);
    }

    #[test]
    fn pagina_com_date_vira_evento_do_vault() {
        let pagina = |path: &str, props: &[(&str, &str)]| crate::PageIndexEntry {
            path: path.into(),
            title: path.trim_end_matches(".md").rsplit('/').next().unwrap().into(),
            properties: props.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
            ..Default::default()
        };
        let eventos = entradas_do_vault(&[
            pagina("journals/diario.md", &[("date", "2026-08-19"), ("time", "10:00")]),
            pagina("pages/sem-data.md", &[("status", "rascunho")]),
            pagina("pages/viagem.md", &[("date", "2026-08-20"), ("end_date", "2026-08-22")]),
        ]);
        assert_eq!(eventos.len(), 2);
        assert_eq!(eventos[0].title, "diario");
        assert_eq!(eventos[0].start_time.as_deref(), Some("10:00"));
        assert_eq!(eventos[0].page_path.as_deref(), Some("journals/diario.md"));
        assert_eq!(eventos[1].end_date.as_deref(), Some("2026-08-22"));
    }
}
