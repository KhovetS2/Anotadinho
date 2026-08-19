//! Ponte da UI pra `anotadinho_core::date_util` + as funções que leem o
//! relógio do navegador.
//!
//! A aritmética de data pura (JDN, parse/format, mês anterior/próximo)
//! foi pro `core` no ciclo 149, pro `embed` e pro CLI alcançarem ela sem
//! WASM. O que sobra aqui é o que depende de `js_sys::Date` — que não
//! compila fora do navegador. Todo mundo continua importando
//! `crate::date_util::*` como antes.

pub use anotadinho_core::date_util::*;

/// Data de hoje via `js_sys::Date` (mesma fonte já usada em
/// `card_detail_modal.rs`).
pub fn today() -> (i32, u32, u32) {
    let d = js_sys::Date::new_0();
    (d.get_full_year() as i32, d.get_month() + 1, d.get_date())
}

/// Data de hoje formatada `"YYYY-MM-DD"`.
pub fn today_string() -> String {
    let (y, m, d) = today();
    format_date(y, m, d)
}

/// Hora atual via `js_sys::Date`, em minutos desde meia-noite — usado pra
/// desenhar a linha do horário atual na grade de horas.
pub fn now_minutes() -> u32 {
    let d = js_sys::Date::new_0();
    minutes_since_midnight(d.get_hours(), d.get_minutes())
}
