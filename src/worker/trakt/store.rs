//! Per-profile Trakt sign-in and the "report what I watch" switch, kept in `settings`.
use crate::db::Db;
use crate::model::ProfileId;
use crate::trakt::auth::Tokens;
use anyhow::{Context, Result};

const AUTH: &str = "trakt_auth";
const SCROBBLE: &str = "trakt_scrobble";

pub fn tokens(db: &Db, profile: ProfileId) -> Option<Tokens> {
    let json = db.get_setting(Some(profile), AUTH).ok()??;
    serde_json::from_str(&json).ok()
}

pub fn save_tokens(db: &Db, profile: ProfileId, t: &Tokens) -> Result<()> {
    let json = serde_json::to_string(t).context("serialize Trakt tokens")?;
    db.set_setting(Some(profile), AUTH, &json)
}

pub fn clear_tokens(db: &Db, profile: ProfileId) -> Result<()> {
    db.delete_setting(Some(profile), AUTH)
}

/// On unless the user switched it off.
pub fn scrobble_enabled(db: &Db, profile: ProfileId) -> bool {
    db.get_setting(Some(profile), SCROBBLE)
        .ok()
        .flatten()
        .as_deref()
        != Some("0")
}

pub fn set_scrobble_enabled(db: &Db, profile: ProfileId, on: bool) -> Result<()> {
    db.set_setting(Some(profile), SCROBBLE, if on { "1" } else { "0" })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_round_trip_and_clear() {
        let db = Db::open_in_memory().unwrap();
        let p = db.ensure_default_profile().unwrap();
        assert_eq!(tokens(&db, p), None);
        let t = Tokens {
            access_token: "a".into(),
            refresh_token: "r".into(),
            expires_at: 5,
            username: Some("u".into()),
        };
        save_tokens(&db, p, &t).unwrap();
        assert_eq!(tokens(&db, p), Some(t));
        clear_tokens(&db, p).unwrap();
        assert_eq!(tokens(&db, p), None);
    }

    #[test]
    fn scrobble_switch_defaults_on() {
        let db = Db::open_in_memory().unwrap();
        let p = db.ensure_default_profile().unwrap();
        assert!(scrobble_enabled(&db, p));
        set_scrobble_enabled(&db, p, false).unwrap();
        assert!(!scrobble_enabled(&db, p));
        set_scrobble_enabled(&db, p, true).unwrap();
        assert!(scrobble_enabled(&db, p));
    }
}
