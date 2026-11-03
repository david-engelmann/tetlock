//! Civil date arithmetic and this repo's own date
//! acceptance (WW-18a). The wall command's date is
//! read with date.fromisoformat's wide acceptance:
//! calendar and ISO week forms, extended and basic.
//! Dates ride as days since the Unix epoch so
//! comparisons are plain integer comparisons.

/// Days from a civil date (Howard Hinnant's
/// algorithm), proleptic Gregorian.
pub fn days_from_civil(y: i64, m: u32, d: u32) -> i64 {
    let y_adj = if m <= 2 { y - 1 } else { y };
    let era = if y_adj >= 0 { y_adj } else { y_adj - 399 } / 400;
    let yoe = y_adj - era * 400;
    let mp = ((m + 9) % 12) as i64;
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// The civil date at a day count.
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

/// Weekday at a day count, Monday 0 to Sunday 6.
/// The epoch day was a Thursday.
pub fn weekday_from_days(z: i64) -> u32 {
    (((z + 3) % 7 + 7) % 7) as u32
}

pub fn is_leap(y: i64) -> bool {
    y % 4 == 0 && (y % 100 != 0 || y % 400 == 0)
}

pub fn days_in_month(y: i64, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap(y) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// The number of ISO weeks in a year: 53 when the
/// year starts on a Thursday, or on a Wednesday in
/// a leap year.
pub fn iso_weeks_in_year(y: i64) -> u32 {
    let jan1 = weekday_from_days(days_from_civil(y, 1, 1));
    if jan1 == 3 || (jan1 == 2 && is_leap(y)) {
        53
    } else {
        52
    }
}

/// The Monday of ISO week 1 of a year: the Monday
/// of the week holding January 4.
fn iso_week1_monday(y: i64) -> i64 {
    let jan4 = days_from_civil(y, 1, 4);
    jan4 - weekday_from_days(jan4) as i64
}

/// ISO calendar at a day count: (iso year, iso
/// week, iso weekday with Monday 1).
pub fn iso_calendar(z: i64) -> (i64, u32, u32) {
    let weekday = weekday_from_days(z) + 1;
    let thursday = z + (4 - weekday as i64);
    let (iso_year, _, _) = civil_from_days(thursday);
    let week = ((z - iso_week1_monday(iso_year)) / 7 + 1) as u32;
    (iso_year, week, weekday)
}

fn digits(bytes: &[u8]) -> Option<i64> {
    if bytes.is_empty() || !bytes.iter().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let mut value: i64 = 0;
    for b in bytes {
        value = value * 10 + (b - b'0') as i64;
    }
    Some(value)
}

/// Parse a date with date.fromisoformat's
/// acceptance and return its day count: calendar
/// extended (YYYY-MM-DD) and basic (YYYYMMDD),
/// ISO week extended (YYYY-Www-D) and basic
/// (YYYYWwwD). Reduced and ordinal forms refuse,
/// the year is exactly four digits, and the whole
/// string is consumed.
pub fn parse_date_wide(text: &str) -> Option<i64> {
    let b = text.as_bytes();
    if b.len() < 8 {
        return None;
    }
    let year = digits(&b[0..4])?;
    if !(1..=9999).contains(&year) {
        return None;
    }
    let rest = &b[4..];
    if rest[0] == b'-' {
        if rest.len() != 6 {
            return None;
        }
        if rest[1] == b'W' {
            // Week extended: -Www-D.
            if rest[4] != b'-' {
                return None;
            }
            return parse_week(year, &rest[2..4], &rest[5..6]);
        }
        // Calendar extended: -MM-DD.
        if rest[3] != b'-' {
            return None;
        }
        let month = digits(&rest[1..3])?;
        let day = digits(&rest[4..6])?;
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > days_in_month(year, month as u32) as i64 {
            return None;
        }
        return Some(days_from_civil(year, month as u32, day as u32));
    }
    if rest[0] == b'W' {
        // Week basic: WwwD, four bytes after the year.
        if rest.len() != 4 {
            return None;
        }
        return parse_week(year, &rest[1..3], &rest[3..4]);
    }
    // Calendar basic: MMDD, four bytes after the year.
    if rest.len() != 4 {
        return None;
    }
    let month = digits(&rest[0..2])?;
    let day = digits(&rest[2..4])?;
    if !(1..=12).contains(&month) {
        return None;
    }
    if day < 1 || day > days_in_month(year, month as u32) as i64 {
        return None;
    }
    Some(days_from_civil(year, month as u32, day as u32))
}

fn parse_week(year: i64, week_b: &[u8], day_b: &[u8]) -> Option<i64> {
    let week = digits(week_b)?;
    let day = digits(day_b)?;
    if week < 1 || week > iso_weeks_in_year(year) as i64 {
        return None;
    }
    if !(1..=7).contains(&day) {
        return None;
    }
    Some(iso_week1_monday(year) + (week - 1) * 7 + (day - 1))
}

/// The UTC calendar date of an instant in
/// microseconds since the epoch, as a day count.
pub fn utc_day_of(micros: i128) -> i64 {
    micros.div_euclid(86_400_000_000) as i64
}
