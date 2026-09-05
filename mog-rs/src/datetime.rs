//! Tiny dependency-free date/time helpers for the built-in `{{@today}}` /
//! `{{@now}}` identifiers. Uses Howard Hinnant's civil<->days algorithms so we
//! avoid pulling in a calendar crate (the engine stays small). UTC only.

use anyhow::{anyhow, bail, Result};

/// Days since 1970-01-01 for a proleptic-Gregorian civil date. `m` in 1..=12.
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400; // [0, 399]
    let mp = if m > 2 { m - 3 } else { m + 9 }; // [0, 11]
    let doy = (153 * mp + 2) / 5 + d - 1; // [0, 365]
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy; // [0, 146096]
    era * 146097 + doe - 719468
}

/// Civil date (year, month 1..=12, day 1..=31) from days since 1970-01-01.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097; // [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365; // [0, 399]
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = doy - (153 * mp + 2) / 5 + 1; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 }; // [1, 12]
    (y + if m <= 2 { 1 } else { 0 }, m as u32, d as u32)
}

/// Split an epoch-seconds value into (days-since-epoch, seconds-of-day), using
/// Euclidean division so negatives (pre-1970) still land on the right day.
fn split_epoch(epoch: i64) -> (i64, i64) {
    (epoch.div_euclid(86400), epoch.rem_euclid(86400))
}

/// `YYYY-MM-DD` (UTC) for an epoch-seconds value.
pub fn format_date(epoch: i64) -> String {
    let (days, _) = split_epoch(epoch);
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// `YYYY-MM-DDTHH:MM:SSZ` (UTC) for an epoch-seconds value.
pub fn format_datetime(epoch: i64) -> String {
    let (days, sod) = split_epoch(epoch);
    let (y, m, d) = civil_from_days(days);
    let (h, mi, s) = (sod / 3600, (sod % 3600) / 60, sod % 60);
    format!("{y:04}-{m:02}-{d:02}T{h:02}:{mi:02}:{s:02}Z")
}

/// The calendar year (UTC) for an epoch-seconds value.
pub fn year(epoch: i64) -> i64 {
    let (days, _) = split_epoch(epoch);
    civil_from_days(days).0
}

/// Current time as epoch seconds (UTC). Reads the real clock; the CLI pins it via
/// `--pin-now` for reproducible runs, so the engine itself never reads the clock.
pub fn now_epoch() -> i64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(d) => d.as_secs() as i64,
        Err(e) => -(e.duration().as_secs() as i64),
    }
}

/// Parse a `--pin-now` value into epoch seconds. Accepts an integer (epoch seconds),
/// `YYYY-MM-DD` (midnight UTC), or `YYYY-MM-DDTHH:MM:SS` with an optional `Z`.
pub fn parse_now(s: &str) -> Result<i64> {
    let s = s.trim();
    if let Ok(epoch) = s.parse::<i64>() {
        return Ok(epoch);
    }
    let (date, time) = match s.split_once(['T', ' ']) {
        Some((d, t)) => (d, Some(t)),
        None => (s, None),
    };
    let dp: Vec<&str> = date.split('-').collect();
    if dp.len() != 3 {
        bail!("--pin-now: expected epoch seconds or YYYY-MM-DD[THH:MM:SS], got '{s}'");
    }
    let y: i64 = dp[0]
        .parse()
        .map_err(|_| anyhow!("--pin-now: bad year in '{s}'"))?;
    let mo: i64 = dp[1]
        .parse()
        .map_err(|_| anyhow!("--pin-now: bad month in '{s}'"))?;
    let d: i64 = dp[2]
        .parse()
        .map_err(|_| anyhow!("--pin-now: bad day in '{s}'"))?;
    if !(1..=12).contains(&mo) || !(1..=31).contains(&d) {
        bail!("--pin-now: month/day out of range in '{s}'");
    }
    let mut secs = days_from_civil(y, mo, d) * 86400;
    if let Some(t) = time {
        let t = t.strip_suffix('Z').unwrap_or(t);
        let tp: Vec<&str> = t.split(':').collect();
        if tp.len() != 3 {
            bail!("--pin-now: expected HH:MM:SS in '{s}'");
        }
        let h: i64 = tp[0]
            .parse()
            .map_err(|_| anyhow!("--pin-now: bad hour in '{s}'"))?;
        let mi: i64 = tp[1]
            .parse()
            .map_err(|_| anyhow!("--pin-now: bad minute in '{s}'"))?;
        let se: i64 = tp[2]
            .parse()
            .map_err(|_| anyhow!("--pin-now: bad second in '{s}'"))?;
        secs += h * 3600 + mi * 60 + se;
    }
    Ok(secs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_epochs_format() {
        // 0 = 1970-01-01T00:00:00Z
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_datetime(0), "1970-01-01T00:00:00Z");
        // 1_700_000_000 = 2023-11-14T22:13:20Z
        assert_eq!(format_date(1_700_000_000), "2023-11-14");
        assert_eq!(format_datetime(1_700_000_000), "2023-11-14T22:13:20Z");
        assert_eq!(year(1_700_000_000), 2023);
    }

    #[test]
    fn parse_now_forms_roundtrip() {
        assert_eq!(parse_now("0").unwrap(), 0);
        assert_eq!(parse_now("2023-11-14").unwrap(), 1_699_920_000);
        assert_eq!(parse_now("2023-11-14T22:13:20Z").unwrap(), 1_700_000_000);
        assert_eq!(format_date(parse_now("2024-02-29").unwrap()), "2024-02-29");
        assert!(parse_now("nonsense").is_err());
        assert!(parse_now("2023-13-01").is_err());
    }
}
