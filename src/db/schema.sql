CREATE TABLE profiles (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL UNIQUE
);
CREATE TABLE sources (
  id INTEGER PRIMARY KEY,
  name TEXT NOT NULL,
  kind TEXT NOT NULL CHECK (kind IN ('xtream','m3u_url','m3u_file')),
  url TEXT NOT NULL,
  username TEXT,
  password TEXT,
  user_agent TEXT,
  output_format TEXT,
  epg_url TEXT,
  account_json TEXT,
  last_refresh INTEGER,
  last_error TEXT
);
CREATE TABLE profile_sources (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  position INTEGER NOT NULL,
  PRIMARY KEY (profile_id, source_id)
);

CREATE TABLE categories (
  id INTEGER PRIMARY KEY,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  ext_key TEXT NOT NULL,
  name TEXT NOT NULL,
  provider_pos INTEGER NOT NULL,
  UNIQUE (source_id, media, ext_key)
);
CREATE TABLE streams (
  id INTEGER PRIMARY KEY,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  ext_key TEXT NOT NULL,
  category_key TEXT,
  name TEXT NOT NULL,
  num INTEGER,
  logo TEXT,
  url TEXT,
  container_ext TEXT,
  epg_id TEXT,
  archive_days INTEGER,
  headers_json TEXT,
  extra_json TEXT,
  provider_pos INTEGER NOT NULL,
  UNIQUE (source_id, media, ext_key)
);
CREATE INDEX streams_cat ON streams(source_id, media, category_key, provider_pos);
CREATE VIRTUAL TABLE streams_fts USING fts5(
  name, content='streams', content_rowid='id', tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE category_prefs (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  cat_key TEXT NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0,
  position INTEGER,
  fav_position INTEGER,
  PRIMARY KEY (profile_id, source_id, media, cat_key)
);
CREATE TABLE stream_prefs (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  stream_key TEXT NOT NULL,
  hidden INTEGER NOT NULL DEFAULT 0,
  fav_position INTEGER,
  PRIMARY KEY (profile_id, source_id, media, stream_key)
);
CREATE TABLE history (
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  media TEXT NOT NULL,
  stream_key TEXT NOT NULL,
  watched_at INTEGER NOT NULL,
  PRIMARY KEY (profile_id, source_id, media, stream_key)
);
-- profile_id 0 = global (NULL would defeat the primary key)
CREATE TABLE settings (
  profile_id INTEGER NOT NULL,
  key TEXT NOT NULL,
  value TEXT NOT NULL,
  PRIMARY KEY (profile_id, key)
);
