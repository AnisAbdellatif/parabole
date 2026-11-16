CREATE TABLE media_links (
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  stream_key TEXT NOT NULL,
  meta_id TEXT,               -- 'tt…' or 'trakt:<movie|show>:<id>'; NULL = searched, no confident match
  manual INTEGER NOT NULL DEFAULT 0,
  matched_at INTEGER NOT NULL,
  PRIMARY KEY (source_id, media, stream_key)
);
CREATE TABLE media_meta (
  meta_id TEXT PRIMARY KEY,
  fetched_at INTEGER NOT NULL,
  json TEXT NOT NULL          -- serde_json of model::Meta
);
