CREATE TABLE stream_groups (
  id INTEGER PRIMARY KEY,
  profile_id INTEGER NOT NULL REFERENCES profiles ON DELETE CASCADE,
  media TEXT NOT NULL,
  name TEXT NOT NULL,
  position INTEGER NOT NULL
);
CREATE TABLE stream_group_items (
  group_id INTEGER NOT NULL REFERENCES stream_groups ON DELETE CASCADE,
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  stream_key TEXT NOT NULL,
  position INTEGER NOT NULL,
  last_name TEXT,
  last_logo TEXT,
  PRIMARY KEY (group_id, source_id, stream_key)
);
