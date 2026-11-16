use crate::db::Db;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const KEY: &str = "sports";
/// Pre-sports key holding `{enabled, langs}`; read when `KEY` is absent.
const OLD_KEY: &str = "football";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct SportsSettings {
    pub enabled: bool,
    /// ISO 639-1 codes from `sports::lang::LANGUAGES`, in the user's order.
    pub langs: Vec<String>,
    /// Ids from `sports::registry::SPORTS`, in the order the user picked them.
    pub sports: Vec<String>,
}

impl Default for SportsSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            langs: vec!["en".into()],
            sports: vec!["football".into()],
        }
    }
}

/// Falls back to defaults when nothing is stored or the stored JSON is unreadable.
pub fn load(db: &Db) -> SportsSettings {
    let stored = |key| db.get_setting(None, key).ok().flatten();
    stored(KEY)
        .or_else(|| stored(OLD_KEY))
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save(db: &Db, settings: &SportsSettings) -> Result<()> {
    db.set_setting(None, KEY, &serde_json::to_string(settings)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_default() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(load(&db), SportsSettings::default());
        let s = SportsSettings {
            enabled: true,
            langs: vec!["fr".into(), "ar".into()],
            sports: vec!["tennis".into()],
        };
        save(&db, &s).unwrap();
        assert_eq!(load(&db), s);
    }

    #[test]
    fn garbage_gives_default() {
        let db = Db::open_in_memory().unwrap();
        db.set_setting(None, KEY, "not json").unwrap();
        assert_eq!(load(&db), SportsSettings::default());
    }

    #[test]
    fn old_key_migrates() {
        let db = Db::open_in_memory().unwrap();
        db.set_setting(None, OLD_KEY, r#"{"enabled":true,"langs":["de"]}"#)
            .unwrap();
        let s = load(&db);
        assert!(s.enabled);
        assert_eq!(s.langs, ["de"]);
        assert_eq!(s.sports, ["football"]);
    }
}
