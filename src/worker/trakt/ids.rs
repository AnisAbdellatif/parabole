//! Finds the Trakt/IMDb ids of a stream, reusing the links made for the details view.
use super::super::meta;
use crate::db::Db;
use crate::meta::title;
use crate::model::{MediaKind, SourceId};
use crate::trakt::scrobble::Ids;
use reqwest::blocking::Client;

fn link_id(db: &Db, source: SourceId, media: MediaKind, key: &str) -> Option<String> {
    db.media_link(source, media, key).ok()??.0
}

/// With IMDb details on, the details lookup does the matching; otherwise Trakt's search does.
pub(super) fn ids_for(
    db: &Db,
    client: &Client,
    source: SourceId,
    media: MediaKind,
    key: &str,
) -> Option<Ids> {
    let link = db.media_link(source, media, key).ok()?;
    if let Some((Some(id), _)) = &link {
        return Ids::from_meta_id(id);
    }
    if meta::enabled(db) {
        meta::resolve(db, client, source, media, key).ok()?;
        return Ids::from_meta_id(&link_id(db, source, media, key)?);
    }
    if link.is_some() {
        return None;
    }
    Ids::from_meta_id(&search_link(db, client, source, media, key)?)
}

fn search_link(
    db: &Db,
    client: &Client,
    source: SourceId,
    media: MediaKind,
    key: &str,
) -> Option<String> {
    let name = db.stream_name(source, media, key).ok()??;
    let want = title::clean(&name);
    let found = crate::trakt::meta::search(client, &want, media).ok()?;
    let kind = if media == MediaKind::Series {
        "show"
    } else {
        "movie"
    };
    let meta_id = found.and_then(|(m, trakt_id)| {
        m.imdb_id
            .or_else(|| trakt_id.map(|t| format!("trakt:{kind}:{t}")))
    });
    db.set_media_link(source, media, key, meta_id.as_deref(), false)
        .ok()?;
    meta_id
}
