use crate::db::Db;
use anyhow::Result;
use serde::{Deserialize, Serialize};

const KEY: &str = "football";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct FootballSettings {
    pub enabled: bool,
    /// ISO 639-1 codes from `sports::lang::LANGUAGES`, in the user's order.
    pub langs: Vec<String>,
}

impl Default for FootballSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            langs: vec!["en".into()],
        }
    }
}

/// Falls back to defaults when nothing is stored or the stored JSON is unreadable.
pub fn load(db: &Db) -> FootballSettings {
    db.get_setting(None, KEY)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save(db: &Db, settings: &FootballSettings) -> Result<()> {
    db.set_setting(None, KEY, &serde_json::to_string(settings)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_default() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(load(&db), FootballSettings::default());
        let s = FootballSettings {
            enabled: true,
            langs: vec!["fr".into(), "ar".into()],
        };
        save(&db, &s).unwrap();
        assert_eq!(load(&db), s);
    }

    #[test]
    fn garbage_gives_default() {
        let db = Db::open_in_memory().unwrap();
        db.set_setting(None, KEY, "not json").unwrap();
        assert_eq!(load(&db), FootballSettings::default());
    }
}
