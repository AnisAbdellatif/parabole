use super::Db;
use crate::model::*;
use anyhow::Result;
use rusqlite::Row;
use std::collections::HashMap;

fn programme(r: &Row, at: usize) -> rusqlite::Result<Programme> {
    Ok(Programme {
        channel: r.get(at)?,
        start: r.get(at + 1)?,
        stop: r.get(at + 2)?,
        title: r.get(at + 3)?,
        descr: r.get(at + 4)?,
    })
}

impl Db {
    /// Current and next programme per stream id; streams with nothing airing at `now` are absent.
    pub fn epg_now_next(&self, stream_ids: &[i64], now: i64) -> Result<HashMap<i64, NowNext>> {
        let mut out = HashMap::new();
        for chunk in stream_ids.chunks(500) {
            let marks = vec!["?"; chunk.len()].join(",");
            let mut stmt = self.conn.prepare(&format!(
                "SELECT e.stream_id, p.channel, p.start, p.stop, p.title, p.descr
                 FROM stream_epg e JOIN epg_programmes p
                   ON p.source_id = e.source_id AND p.channel = e.channel AND p.stop > ?
                 WHERE e.stream_id IN ({marks}) ORDER BY e.stream_id, p.start"
            ))?;
            let params = std::iter::once(now).chain(chunk.iter().copied());
            let mut rows = stmt.query(rusqlite::params_from_iter(params))?;
            let mut seen: HashMap<i64, Vec<Programme>> = HashMap::new();
            while let Some(r) = rows.next()? {
                let list = seen.entry(r.get(0)?).or_default();
                if list.len() < 2 {
                    list.push(programme(r, 1)?);
                }
            }
            for (id, list) in seen {
                let mut it = list.into_iter();
                if let Some(cur) = it.next().filter(|p| p.start <= now) {
                    out.insert(
                        id,
                        NowNext {
                            now: cur,
                            next: it.next(),
                        },
                    );
                }
            }
        }
        Ok(out)
    }

    /// Programmes of one stream overlapping [from, to), by start time.
    pub fn epg_schedule(&self, stream_id: i64, from: i64, to: i64) -> Result<Vec<Programme>> {
        let mut stmt = self.conn.prepare(
            "SELECT p.channel, p.start, p.stop, p.title, p.descr
             FROM stream_epg e JOIN epg_programmes p
               ON p.source_id = e.source_id AND p.channel = e.channel
             WHERE e.stream_id = ?1 AND p.stop > ?2 AND p.start < ?3 ORDER BY p.start",
        )?;
        let rows = stmt.query_map((stream_id, from, to), |r| programme(r, 0))?;
        Ok(rows.collect::<rusqlite::Result<_>>()?)
    }
}
