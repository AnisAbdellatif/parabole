ALTER TABLE stream_prefs ADD COLUMN last_name TEXT;
ALTER TABLE stream_prefs ADD COLUMN last_logo TEXT;
UPDATE stream_prefs SET
  last_name = (SELECT s.name FROM streams s WHERE s.source_id = stream_prefs.source_id
               AND s.media = stream_prefs.media AND s.ext_key = stream_prefs.stream_key),
  last_logo = (SELECT s.logo FROM streams s WHERE s.source_id = stream_prefs.source_id
               AND s.media = stream_prefs.media AND s.ext_key = stream_prefs.stream_key)
WHERE fav_position IS NOT NULL;
