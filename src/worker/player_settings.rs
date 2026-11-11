use crate::db::Db;
use crate::player::PlayerSettings;
use anyhow::Result;

const KEY: &str = "player";

/// Falls back to defaults when nothing is stored or the stored JSON is unreadable.
pub fn load(db: &Db) -> PlayerSettings {
    db.get_setting(None, KEY)
        .ok()
        .flatten()
        .and_then(|json| serde_json::from_str(&json).ok())
        .unwrap_or_default()
}

pub fn save(db: &Db, settings: &PlayerSettings) -> Result<()> {
    db.set_setting(None, KEY, &serde_json::to_string(settings)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::player::PlayerKind;

    #[test]
    fn roundtrip_and_default() {
        let db = Db::open_in_memory().unwrap();
        assert_eq!(load(&db), PlayerSettings::default());
        let s = PlayerSettings {
            default: PlayerKind::Vlc,
            ..Default::default()
        };
        save(&db, &s).unwrap();
        assert_eq!(load(&db), s);
    }
}
