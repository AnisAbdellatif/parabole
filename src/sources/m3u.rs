use crate::model::ParsedPlaylist;
use anyhow::Result;
use std::io::BufRead;

/// Streaming M3U parser (PLAN.md §5.1).
pub fn parse<R: BufRead>(reader: R) -> Result<ParsedPlaylist> {
    let _ = reader;
    todo!()
}
