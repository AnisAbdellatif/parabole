use super::de::{bool01, opt_i64, opt_string};
use serde_json::Value;

const SOON_SECS: i64 = 7 * 86_400;

/// The parts of `user_info` the account panel shows.
#[derive(Clone, Debug, PartialEq)]
pub struct AccountInfo {
    pub status: String,
    /// None = the account never expires.
    pub exp_date: Option<i64>,
    pub active_cons: Option<i64>,
    pub max_connections: Option<i64>,
    pub is_trial: bool,
    pub created_at: Option<i64>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Expiry {
    Unlimited,
    Expired,
    Soon,
    Later,
}

impl AccountInfo {
    /// Reads the `account_json` stored by a refresh.
    pub fn parse(json: &str) -> Option<Self> {
        let value: Value = serde_json::from_str(json).ok()?;
        let u = value.get("user_info").filter(|u| u.is_object())?;
        Some(Self {
            status: opt_string(u.get("status")).unwrap_or_else(|| "Unknown".into()),
            exp_date: opt_i64(u.get("exp_date")),
            active_cons: opt_i64(u.get("active_cons")),
            max_connections: opt_i64(u.get("max_connections")),
            is_trial: bool01(u.get("is_trial")),
            created_at: opt_i64(u.get("created_at")),
        })
    }

    pub fn expiry(&self, now: i64) -> Expiry {
        match self.exp_date {
            None => Expiry::Unlimited,
            Some(t) if t <= now => Expiry::Expired,
            Some(t) if t - now < SOON_SECS => Expiry::Soon,
            Some(_) => Expiry::Later,
        }
    }

    pub fn expiry_text(&self) -> String {
        self.exp_date.map_or("unlimited".into(), format_date)
    }

    pub fn connections_text(&self) -> String {
        let n = |v: Option<i64>| v.map_or("?".into(), |v| v.to_string());
        format!("{}/{}", n(self.active_cons), n(self.max_connections))
    }

    pub fn connections_full(&self) -> bool {
        matches!((self.active_cons, self.max_connections), (Some(a), Some(m)) if m > 0 && a >= m)
    }

    pub fn summary(&self) -> String {
        format!(
            "{}, expires {}, {} connections",
            self.status,
            self.expiry_text(),
            self.connections_text()
        )
    }
}

/// UTC `YYYY-MM-DD` from a unix timestamp (civil-from-days, no date crate needed).
pub fn format_date(ts: i64) -> String {
    let z = ts.div_euclid(86_400) + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

#[cfg(test)]
mod tests {
    use super::*;

    const JSON: &str = r#"{"user_info":{"status":"Active","exp_date":"1700000000","active_cons":"2","max_connections":2,"is_trial":"0","created_at":1600000000}}"#;

    #[test]
    fn dates() {
        assert_eq!(format_date(0), "1970-01-01");
        assert_eq!(format_date(951_782_400), "2000-02-29");
        assert_eq!(format_date(1_700_000_000), "2023-11-14");
    }

    #[test]
    fn parses_and_classifies() {
        let a = AccountInfo::parse(JSON).unwrap();
        assert_eq!(a.summary(), "Active, expires 2023-11-14, 2/2 connections");
        assert!(a.connections_full() && !a.is_trial);
        assert_eq!(a.expiry(1_700_000_001), Expiry::Expired);
        assert_eq!(a.expiry(1_700_000_000 - 86_400), Expiry::Soon);
        assert_eq!(a.expiry(1_000_000_000), Expiry::Later);
    }

    #[test]
    fn null_expiry_is_unlimited() {
        let a = AccountInfo::parse(r#"{"user_info":{"exp_date":null}}"#).unwrap();
        assert_eq!(a.expiry(0), Expiry::Unlimited);
        assert_eq!(a.expiry_text(), "unlimited");
        assert!(!a.connections_full());
        assert!(AccountInfo::parse("{}").is_none());
    }
}
