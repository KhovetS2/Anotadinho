//! Matemática de data pura, sem dependência externa — o projeto removeu
//! `chrono` deliberadamente, então tudo aqui usa aritmética inteira
//! sobre o número de dia juliano (JDN), que lida com virada de
//! mês/ano/bissexto sem casos especiais.
//!
//! Mora no `core` (e não na UI, de onde veio no ciclo 149) porque o
//! `embed` também mora aqui e depende desta aritmética — e porque o CLI
//! precisa das duas coisas sem passar por WASM. As funções que leem o
//! RELÓGIO (`today`, `today_string`, `now_minutes`) ficaram na UI:
//! dependem de `js_sys::Date`, que não existe fora do navegador.

/// Converte `(ano, mês, dia)` num número de dia juliano (inteiro
/// contínuo, cresce 1 por dia — permite somar/subtrair dias e comparar
/// datas com aritmética simples). Algoritmo padrão (Fliegel & Van
/// Flandern).
fn to_jdn(y: i32, m: u32, d: u32) -> i64 {
    let a = (14 - m as i64) / 12;
    let y2 = y as i64 + 4800 - a;
    let m2 = m as i64 + 12 * a - 3;
    d as i64 + (153 * m2 + 2) / 5 + 365 * y2 + y2 / 4 - y2 / 100 + y2 / 400 - 32045
}

/// Inverso de [`to_jdn`].
fn from_jdn(jdn: i64) -> (i32, u32, u32) {
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;
    let day = (e - (153 * m + 2) / 5 + 1) as u32;
    let month = (m + 3 - 12 * (m / 10)) as u32;
    let year = (100 * b + d - 4800 + m / 10) as i32;
    (year, month, day)
}

/// Ano bissexto (regra gregoriana completa).
pub fn is_leap_year(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Quantos dias tem o mês `m` (1-12) do ano `y`.
pub fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(y) => 29,
        2 => 28,
        _ => 30,
    }
}

/// Parseia `"YYYY-MM-DD"`. `None` se o formato não bater.
pub fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let mut parts = s.trim().splitn(3, '-');
    let y: i32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    let d: u32 = parts.next()?.parse().ok()?;
    if (1..=12).contains(&m) && (1..=31).contains(&d) {
        Some((y, m, d))
    } else {
        None
    }
}

/// Formata `(ano, mês, dia)` como `"YYYY-MM-DD"`.
pub fn format_date(y: i32, m: u32, d: u32) -> String {
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Dia da semana de `(y, m, d)`: `0` = domingo … `6` = sábado.
pub fn weekday_of(y: i32, m: u32, d: u32) -> u32 {
    ((to_jdn(y, m, d) + 1).rem_euclid(7)) as u32
}

/// Diferença em dias entre duas datas `"YYYY-MM-DD"` (`b - a`). `None` se
/// alguma das duas não parsear.
pub fn days_between(a: &str, b: &str) -> Option<i64> {
    let (ay, am, ad) = parse_date(a)?;
    let (by, bm, bd) = parse_date(b)?;
    Some(to_jdn(by, bm, bd) - to_jdn(ay, am, ad))
}

/// Soma `delta` dias (pode ser negativo) a uma data `"YYYY-MM-DD"`.
pub fn add_days(date: &str, delta: i64) -> Option<String> {
    let (y, m, d) = parse_date(date)?;
    let (ny, nm, nd) = from_jdn(to_jdn(y, m, d) + delta);
    Some(format_date(ny, nm, nd))
}

/// Mês anterior a `(y, m)`, com virada de ano.
pub fn prev_month(y: i32, m: u32) -> (i32, u32) {
    if m == 1 { (y - 1, 12) } else { (y, m - 1) }
}

/// Mês seguinte a `(y, m)`, com virada de ano.
pub fn next_month(y: i32, m: u32) -> (i32, u32) {
    if m == 12 { (y + 1, 1) } else { (y, m + 1) }
}

/// Nome do mês em português (`1` = janeiro … `12` = dezembro).
pub fn month_name(m: u32) -> &'static str {
    const NAMES: [&str; 12] = [
        "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
        "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
    ];
    NAMES.get(m.saturating_sub(1) as usize).copied().unwrap_or("")
}

/// Parseia `"HH:MM"`. `None` se o formato não bater.
pub fn parse_time(s: &str) -> Option<(u32, u32)> {
    let mut parts = s.trim().splitn(2, ':');
    let h: u32 = parts.next()?.parse().ok()?;
    let m: u32 = parts.next()?.parse().ok()?;
    if h < 24 && m < 60 {
        Some((h, m))
    } else {
        None
    }
}

/// Formata `(hora, minuto)` como `"HH:MM"`.
pub fn format_time(h: u32, m: u32) -> String {
    format!("{:02}:{:02}", h, m)
}

/// Minutos desde meia-noite — usado pra posicionar um evento na grade de
/// horas (`top`/`height` em função do horário).
pub fn minutes_since_midnight(h: u32, m: u32) -> u32 {
    h * 60 + m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn weekday_known_dates() {
        // 2026-08-06 é uma quinta-feira.
        assert_eq!(weekday_of(2026, 8, 6), 4);
        // 2000-01-01 é um sábado.
        assert_eq!(weekday_of(2000, 1, 1), 6);
    }

    #[test]
    fn days_in_month_leap_year() {
        assert_eq!(days_in_month(2024, 2), 29);
        assert_eq!(days_in_month(2023, 2), 28);
        assert_eq!(days_in_month(2000, 2), 29); // bissexto por regra dos 400
        assert_eq!(days_in_month(1900, 2), 28); // não-bissexto (100 sem ser 400)
    }

    #[test]
    fn add_days_crosses_month_and_year() {
        assert_eq!(add_days("2026-08-30", 5).as_deref(), Some("2026-09-04"));
        assert_eq!(add_days("2026-12-28", 5).as_deref(), Some("2027-01-02"));
        assert_eq!(add_days("2026-03-05", -10).as_deref(), Some("2026-02-23"));
    }

    #[test]
    fn days_between_roundtrip() {
        assert_eq!(days_between("2026-08-01", "2026-08-10"), Some(9));
        assert_eq!(days_between("2026-08-10", "2026-08-01"), Some(-9));
        assert_eq!(days_between("2026-01-01", "2027-01-01"), Some(365));
    }

    #[test]
    fn parse_date_rejects_malformed() {
        assert_eq!(parse_date("2026-08-06"), Some((2026, 8, 6)));
        assert_eq!(parse_date("not-a-date"), None);
        assert_eq!(parse_date("2026-13-01"), None);
    }

    #[test]
    fn prev_next_month_wrap_year() {
        assert_eq!(prev_month(2026, 1), (2025, 12));
        assert_eq!(next_month(2026, 12), (2027, 1));
    }

    #[test]
    fn parse_time_valid_and_invalid() {
        assert_eq!(parse_time("09:30"), Some((9, 30)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("09:60"), None);
        assert_eq!(parse_time("not-a-time"), None);
    }

    #[test]
    fn format_time_zero_padded() {
        assert_eq!(format_time(9, 5), "09:05");
        assert_eq!(format_time(14, 30), "14:30");
    }

    #[test]
    fn minutes_since_midnight_calc() {
        assert_eq!(minutes_since_midnight(0, 0), 0);
        assert_eq!(minutes_since_midnight(9, 30), 570);
        assert_eq!(minutes_since_midnight(23, 59), 1439);
    }
}
