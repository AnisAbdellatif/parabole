//! Online details (IMDb, Trakt as fallback) of movies and series, cached in SQLite.
use super::{Event, Worker};
use crate::db::Db;
use crate::meta::imdb::{self, sized};
use crate::meta::pick::{Candidate, pick};
use crate::meta::title;
use crate::model::{MediaKind, Meta, SourceId};
use crate::trakt;
use anyhow::Result;
use reqwest::blocking::Client;
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

const KEY: &str = "imdb_details";
const META_TTL_S: i64 = 14 * 86400;
const MISS_TTL_S: i64 = 7 * 86400;

/// On unless the user switched it off.
pub fn enabled(db: &Db) -> bool {
    db.get_setting(None, KEY).ok().flatten().as_deref() != Some("0")
}

pub fn set_enabled(db: &Db, on: bool) -> Result<()> {
    db.set_setting(None, KEY, if on { "1" } else { "0" })
}

fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() as i64)
        .unwrap_or(0)
}

fn trakt_kind(media: MediaKind) -> &'static str {
    if media == MediaKind::Series {
        "show"
    } else {
        "movie"
    }
}

/// IMDb details of a `tt` id, or Trakt's when IMDb fails.
fn fetch_tt(client: &Client, id: &str, media: MediaKind) -> Result<Meta> {
    match imdb::details::fetch(client, id) {
        Ok(m) => Ok(m),
        Err(e) => match trakt::meta::by_imdb(client, id, media) {
            Ok(Some((mut m, _))) => {
                m.imdb_id = Some(id.to_string());
                Ok(m)
            }
            _ => Err(e),
        },
    }
}

fn from_candidate(client: &Client, c: &Candidate, media: MediaKind) -> Meta {
    fetch_tt(client, &c.id, media).unwrap_or_else(|_| Meta {
        origin: "IMDb".into(),
        imdb_id: Some(c.id.clone()),
        title: c.title.clone(),
        year: c.year,
        poster: c.poster.as_deref().map(|p| sized(p, 600)),
        ..Default::default()
    })
}

fn resolve(
    db: &Db,
    client: &Client,
    source: SourceId,
    media: MediaKind,
    key: &str,
) -> Result<Option<Meta>> {
    if !enabled(db) {
        return Ok(None);
    }
    match db.media_link(source, media, key)? {
        Some((Some(id), _)) => {
            let cached = db.media_meta(&id)?;
            if let Some((m, at)) = &cached
                && now() - at < META_TTL_S
            {
                return Ok(Some(m.clone()));
            }
            if id.starts_with("tt") {
                return match fetch_tt(client, &id, media) {
                    Ok(m) => {
                        db.put_media_meta(&id, &m)?;
                        Ok(Some(m))
                    }
                    // stale details beat none when offline
                    Err(e) => cached.map(|c| Some(c.0)).ok_or(e),
                };
            }
            if let Some((m, _)) = cached {
                return Ok(Some(m));
            }
        }
        Some((None, at)) if now() - at < MISS_TTL_S => return Ok(None),
        _ => {}
    }
    search(db, client, source, media, key)
}

fn search(
    db: &Db,
    client: &Client,
    source: SourceId,
    media: MediaKind,
    key: &str,
) -> Result<Option<Meta>> {
    let Some(name) = db.stream_name(source, media, key)? else {
        return Ok(None);
    };
    let want = title::clean(&name);
    if want.title.is_empty() {
        return Ok(None);
    }
    let mut found: Option<(Meta, Option<i64>)> = None;
    let mut failure = None;
    match imdb::suggest::search(client, &want.title) {
        Ok(cands) => {
            found = pick(&cands, &want, media).map(|c| (from_candidate(client, c, media), None));
        }
        Err(e) => failure = Some(e),
    }
    if found.is_none() && trakt::client::is_configured() {
        match trakt::meta::search(client, &want, media) {
            Ok(f) => found = f,
            Err(e) => failure = failure.or(Some(e)),
        }
    }
    if found.is_none()
        && let Some(e) = failure
    {
        return Err(e);
    }
    let meta_id = found.as_ref().and_then(|(m, trakt_id)| {
        m.imdb_id
            .clone()
            .or_else(|| trakt_id.map(|t| format!("trakt:{}:{t}", trakt_kind(media))))
    });
    db.set_media_link(source, media, key, meta_id.as_deref(), false)?;
    let meta = found.map(|(m, _)| m);
    if let (Some(id), Some(m)) = (&meta_id, &meta) {
        db.put_media_meta(id, m)?;
    }
    Ok(meta)
}

impl Worker {
    pub(super) fn load_meta(
        &mut self,
        source: SourceId,
        media: MediaKind,
        key: String,
    ) -> Result<()> {
        let (tx, repaint, path) = (
            self.ev_tx.clone(),
            self.repaint.clone(),
            self.db_path.clone(),
        );
        thread::Builder::new().name("meta".into()).spawn(move || {
            let run = || -> Result<Option<Meta>> {
                resolve(
                    &Db::open(&path)?,
                    &crate::meta::client()?,
                    source,
                    media,
                    &key,
                )
            };
            // passive enrichment: a failure just leaves the provider details, no toast
            let meta = run().unwrap_or(None);
            let _ = tx.send(Event::MetaLoaded {
                source,
                media,
                key,
                meta,
            });
            repaint();
        })?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{Source, SourceKind};

    fn setup() -> (Db, i64, SourceId) {
        let db = Db::open_in_memory().unwrap();
        let p = db.ensure_default_profile().unwrap();
        let src = Source {
            id: 0,
            name: "A".into(),
            kind: SourceKind::M3uUrl,
            url: "http://example.test/list.m3u".into(),
            username: None,
            password: None,
            user_agent: None,
            mirrors: Vec::new(),
            output_format: None,
            last_refresh: None,
            last_error: None,
        };
        let s = db.add_source(p, &src).unwrap();
        (db, p, s)
    }

    fn meta(title: &str) -> Meta {
        Meta {
            origin: "IMDb".into(),
            title: title.into(),
            ..Default::default()
        }
    }

    #[test]
    fn setting_defaults_on() {
        let (db, _, _) = setup();
        assert!(enabled(&db));
        set_enabled(&db, false).unwrap();
        assert!(!enabled(&db));
        set_enabled(&db, true).unwrap();
        assert!(enabled(&db));
    }

    #[test]
    fn fresh_cache_is_returned_offline() {
        let (db, _, s) = setup();
        let client = crate::meta::client().unwrap();
        db.set_media_link(s, MediaKind::Movie, "k", Some("tt1"), false)
            .unwrap();
        db.put_media_meta("tt1", &meta("Heat")).unwrap();
        let got = resolve(&db, &client, s, MediaKind::Movie, "k").unwrap();
        assert_eq!(got, Some(meta("Heat")));
        set_enabled(&db, false).unwrap();
        assert_eq!(
            resolve(&db, &client, s, MediaKind::Movie, "k").unwrap(),
            None
        );
    }

    #[test]
    fn fresh_miss_is_not_searched_again() {
        let (db, _, s) = setup();
        let client = crate::meta::client().unwrap();
        db.set_media_link(s, MediaKind::Series, "k", None, false)
            .unwrap();
        assert_eq!(
            resolve(&db, &client, s, MediaKind::Series, "k").unwrap(),
            None
        );
    }

    #[test]
    fn unknown_stream_is_none() {
        let (db, _, s) = setup();
        let client = crate::meta::client().unwrap();
        assert_eq!(
            resolve(&db, &client, s, MediaKind::Movie, "nope").unwrap(),
            None
        );
    }

    #[test]
    fn stale_trakt_cache_is_kept() {
        let (db, _, s) = setup();
        let client = crate::meta::client().unwrap();
        db.set_media_link(s, MediaKind::Movie, "k", Some("trakt:movie:5"), false)
            .unwrap();
        db.put_media_meta("trakt:movie:5", &meta("Heat")).unwrap();
        let got = resolve(&db, &client, s, MediaKind::Movie, "k").unwrap();
        assert_eq!(got, Some(meta("Heat")));
    }
}
