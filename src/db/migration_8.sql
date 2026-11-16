CREATE TABLE stream_quality (
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  stream_key TEXT NOT NULL,
  width INTEGER, height INTEGER, fps REAL, video_codec TEXT, bitrate REAL, startup_ms INTEGER,
  fails INTEGER NOT NULL DEFAULT 0,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (source_id, stream_key)
);
CREATE TABLE scan_categories (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  cat_key TEXT NOT NULL,
  PRIMARY KEY (profile_id, source_id, cat_key)
);
