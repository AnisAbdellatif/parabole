DROP VIEW stream_epg;
DROP TABLE epg_programmes;
ALTER TABLE sources DROP COLUMN epg_refreshed_at;
CREATE TABLE epg_guides (
  id INTEGER PRIMARY KEY,
  source_id INTEGER UNIQUE REFERENCES sources ON DELETE CASCADE, -- provider guide; NULL for a user-added one
  url TEXT,                                                      -- user-added guides only
  name TEXT NOT NULL,
  enabled INTEGER NOT NULL DEFAULT 1,
  position INTEGER NOT NULL DEFAULT 0,
  refreshed_at INTEGER,
  last_error TEXT,
  CHECK ((source_id IS NULL) <> (url IS NULL))
);
INSERT INTO epg_guides(source_id, name) SELECT id, name FROM sources;
CREATE TABLE epg_guide_channels (
  guide_id INTEGER NOT NULL REFERENCES epg_guides ON DELETE CASCADE,
  channel TEXT NOT NULL,
  names TEXT NOT NULL,                  -- display names joined with char(10)
  programmes INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (guide_id, channel)
) WITHOUT ROWID;
CREATE TABLE epg_programmes (
  guide_id INTEGER NOT NULL REFERENCES epg_guides ON DELETE CASCADE,
  channel TEXT NOT NULL, start INTEGER NOT NULL, stop INTEGER NOT NULL,
  title TEXT NOT NULL, descr TEXT,
  PRIMARY KEY (guide_id, channel, start)
) WITHOUT ROWID;
CREATE TABLE epg_links (
  source_id INTEGER NOT NULL REFERENCES sources ON DELETE CASCADE,
  stream_key TEXT NOT NULL,             -- streams.ext_key of a live stream
  guide_id INTEGER NOT NULL REFERENCES epg_guides ON DELETE CASCADE,
  channel TEXT NOT NULL,
  manual INTEGER NOT NULL DEFAULT 0,
  PRIMARY KEY (source_id, stream_key)
);
-- The single place that decides which guide channel a stream shows.
CREATE VIEW stream_epg AS
  SELECT s.id AS stream_id, l.guide_id, l.channel
  FROM epg_links l JOIN streams s ON s.source_id = l.source_id AND s.media = 'live' AND s.ext_key = l.stream_key;
