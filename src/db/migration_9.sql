CREATE TABLE epg_programmes (
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  channel TEXT NOT NULL,
  start INTEGER NOT NULL,
  stop INTEGER NOT NULL,
  title TEXT NOT NULL,
  descr TEXT,
  PRIMARY KEY (source_id, channel, start)
) WITHOUT ROWID;
ALTER TABLE sources ADD COLUMN epg_refreshed_at INTEGER;
-- The single place that decides which guide channel a stream uses.
CREATE VIEW stream_epg AS
  SELECT id AS stream_id, source_id, lower(trim(epg_id)) AS channel
  FROM streams
  WHERE media = 'live' AND epg_id IS NOT NULL AND trim(epg_id) <> '';
