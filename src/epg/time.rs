use chrono::NaiveDate;

/// Parses XMLTV `YYYYMMDDhhmm[ss] [+-hhmm]` into unix seconds; no offset means UTC.
pub fn parse_time(s: &str) -> Option<i64> {
    let s = s.trim();
    let (stamp, offset) = match s.split_once(char::is_whitespace) {
        Some((a, b)) => (a, parse_offset(b.trim())?),
        None => (s, 0),
    };
    if !stamp.bytes().all(|b| b.is_ascii_digit()) || !matches!(stamp.len(), 12 | 14) {
        return None;
    }
    let n = |r: std::ops::Range<usize>| stamp[r].parse::<u32>().ok();
    let sec = if stamp.len() == 14 { n(12..14)? } else { 0 };
    let date = NaiveDate::from_ymd_opt(stamp[0..4].parse().ok()?, n(4..6)?, n(6..8)?)?;
    let local = date.and_hms_opt(n(8..10)?, n(10..12)?, sec)?;
    Some(local.and_utc().timestamp() - offset)
}

fn parse_offset(s: &str) -> Option<i64> {
    let sign = match s.as_bytes().first()? {
        b'+' => 1,
        b'-' => -1,
        _ => return None,
    };
    let d = &s[1..];
    if d.len() != 4 || !d.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (h, m): (i64, i64) = (d[0..2].parse().ok()?, d[2..4].parse().ok()?);
    Some(sign * (h * 3600 + m * 60))
}

#[cfg(test)]
mod tests {
    use super::*;

    const BASE: i64 = 1_767_268_800; // 2026-01-01 12:00:00 UTC

    #[test]
    fn offsets() {
        assert_eq!(parse_time("20260101120000 +0100"), Some(BASE - 3600));
        assert_eq!(parse_time("20260101120000 -0530"), Some(BASE + 19800));
    }

    #[test]
    fn no_offset_and_short_form() {
        assert_eq!(parse_time("20260101120000"), Some(BASE));
        assert_eq!(parse_time("202601011200"), Some(BASE));
    }

    #[test]
    fn garbage() {
        for s in [
            "",
            "abc",
            "20261301120000",
            "20260101120000 +01",
            "2026010112",
        ] {
            assert_eq!(parse_time(s), None, "{s}");
        }
    }
}
