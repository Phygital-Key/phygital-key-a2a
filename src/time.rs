//! Timestamps as A2A wants them: ISO 8601, UTC, millisecond precision, `Z`.

/// Unix milliseconds now.
pub fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| i64::try_from(d.as_millis()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

/// `2026-10-17T09:30:00.123Z` for Unix milliseconds.
pub fn iso(ms: i64) -> String {
    let secs = ms.div_euclid(1000);
    let milli = ms.rem_euclid(1000);
    let sod = secs.rem_euclid(86_400);
    format!(
        "{}T{:02}:{:02}:{:02}.{milli:03}Z",
        phygital_key_gate::util::date(secs),
        sod / 3600,
        sod % 3600 / 60,
        sod % 60
    )
}

fn num(s: &str, digits: usize) -> Option<i64> {
    if s.len() != digits || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// Days since 1970-01-01 for a civil date (Howard Hinnant's algorithm).
fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146_097 + doe - 719_468
}

/// Parse an ISO 8601 / RFC 3339 timestamp (`Z` or `+HH:MM` offset, any
/// fraction) into Unix milliseconds.
pub fn parse_iso(s: &str) -> Option<i64> {
    let (date, rest) = s.split_once(['T', 't', ' '])?;
    let mut ymd = date.split('-');
    let (y, m, d) = (
        num(ymd.next()?, 4)?,
        num(ymd.next()?, 2)?,
        num(ymd.next()?, 2)?,
    );
    if ymd.next().is_some() || !(1..=12).contains(&m) || !(1..=31).contains(&d) {
        return None;
    }
    let (time, offset_secs) = if let Some(t) = rest.strip_suffix(['Z', 'z']) {
        (t, 0)
    } else {
        let at = rest.rfind(['+', '-'])?;
        let (t, tz) = (rest.get(..at)?, rest.get(at..)?);
        let sign = if tz.starts_with('-') { -1 } else { 1 };
        let (hh, mm) = tz.get(1..)?.split_once(':')?;
        let (hh, mm) = (num(hh, 2)?, num(mm, 2)?);
        if hh > 23 || mm > 59 {
            return None;
        }
        (t, sign * (hh * 3600 + mm * 60))
    };
    let (hms, frac) = match time.split_once('.') {
        Some((a, f)) => (a, Some(f)),
        None => (time, None),
    };
    let mut parts = hms.split(':');
    let (h, mi, sec) = (
        num(parts.next()?, 2)?,
        num(parts.next()?, 2)?,
        num(parts.next()?, 2)?,
    );
    if parts.next().is_some() || h > 23 || mi > 59 || sec > 60 {
        return None;
    }
    let ms = match frac {
        None => 0,
        Some(f) if !f.is_empty() && f.len() <= 9 && f.bytes().all(|b| b.is_ascii_digit()) => {
            let three: String = f.chars().chain("000".chars()).take(3).collect();
            three.parse::<i64>().ok()?
        }
        Some(_) => return None,
    };
    let secs = days_from_civil(y, m, d) * 86_400 + h * 3600 + mi * 60 + sec - offset_secs;
    secs.checked_mul(1000)?.checked_add(ms)
}

#[cfg(test)]
#[allow(clippy::indexing_slicing)] // serde_json `v["key"]` in test asserts
mod tests {
    use super::*;

    #[test]
    fn formats_utc_with_millis_and_z() {
        assert_eq!(iso(0), "1970-01-01T00:00:00.000Z");
        assert_eq!(iso(1_792_229_400_123), "2026-10-17T09:30:00.123Z");
        assert_eq!(iso(951_782_400_000 + 5), "2000-02-29T00:00:00.005Z");
    }

    #[test]
    fn parses_what_it_formats_and_common_forms() {
        for ms in [0, 1_792_229_400_123, 951_782_400_005, 4_102_444_799_999] {
            assert_eq!(parse_iso(&iso(ms)), Some(ms));
        }
        assert_eq!(parse_iso("2026-10-17T09:30:00Z"), Some(1_792_229_400_000));
        assert_eq!(
            parse_iso("2026-10-17T11:30:00.5+02:00"),
            Some(1_792_229_400_500)
        );
        assert_eq!(
            parse_iso("2026-10-17T09:30:00.123456789Z"),
            Some(1_792_229_400_123)
        );
    }

    #[test]
    fn refuses_junk() {
        for s in [
            "",
            "yesterday",
            "2026-13-01T00:00:00Z",
            "2026-10-17",
            "2026-10-17T25:00:00Z",
            "2026-10-17T09:30:00",
            "2026-10-17T09:30:00.Z",
            "26-10-17T09:30:00Z",
        ] {
            assert_eq!(parse_iso(s), None, "{s}");
        }
    }
}
