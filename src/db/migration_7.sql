CREATE TABLE sports_cache (
  key TEXT PRIMARY KEY,
  schema INTEGER NOT NULL,
  fetched_at INTEGER NOT NULL,
  json TEXT NOT NULL
);
